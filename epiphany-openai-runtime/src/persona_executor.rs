use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use chrono::SecondsFormat;
use cultcache_rs::DatabaseEntry;
use epiphany_core::{
    ModelPassFailureTerminalOptions, PERSONA_INTERPRETER_EFFECT_DOCUMENT_SCHEMA_VERSION,
    PERSONA_MODEL_STAGE_RECEIPT_SCHEMA_VERSION, PERSONA_MODEL_TERMINAL_RECEIPT_SCHEMA_VERSION,
    PersonaInterpreterEffectDocument, PersonaInterpreterInput, PersonaModelStageReceipt,
    PersonaModelTerminalReceipt, PersonaProjectorInput, PersonaTranscriptMessage, PersonaTurnInput,
    RuntimeSpineSessionClosureOptions, build_persona_interpreter_prompt,
    build_persona_projector_prompt_with_transcript, build_persona_turn_prompt,
    close_runtime_session, model_pass_failure_for_request,
    parse_and_validate_persona_interpreter_effect_set, persona_interpreter_effect_set_json_schema,
    runtime_spine_cache,
    terminalize_model_pass_failure_session,
};
use epiphany_model_adapter::{EpiphanyModelInputItem, EpiphanyModelRequest};
use sha2::{Digest, Sha256};

use crate::{EpiphanyOpenAiRuntimeOptions, assistant_text_from_model_events, run_model_turn};

#[derive(Clone, Debug)]
pub struct PersonaModelExecutionPlan {
    turn_id: String,
    provider: String,
    model: String,
    projector_input: PersonaProjectorInput,
    transcript: Vec<PersonaTranscriptMessage>,
    allowed_channel_ids: Vec<String>,
    source_documents: Vec<epiphany_core::EpiphanyMindDocumentVersion>,
    cultmesh_store: PathBuf,
    runtime_id: String,
}

impl PersonaModelExecutionPlan {
    pub fn from_admitted_input(
        store_path: &Path,
        turn_id: &str,
        provider: impl Into<String>,
        model: impl Into<String>,
        cultmesh_store: PathBuf,
        runtime_id: impl Into<String>,
    ) -> Result<Self> {
        let (document, source) =
            epiphany_core::load_admitted_persona_pass_input(store_path, turn_id)?
                .ok_or_else(|| anyhow!("Persona execution plan has no admitted Mind input"))?;
        Ok(Self {
            turn_id: document.turn_id,
            provider: provider.into(),
            model: model.into(),
            projector_input: document.projector_input,
            transcript: document.transcript,
            allowed_channel_ids: document.allowed_channel_ids,
            source_documents: vec![source],
            cultmesh_store,
            runtime_id: runtime_id.into(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct NativePersonaModelRunner {
    pub store_path: PathBuf,
    pub codex_home: PathBuf,
    pub provider_credential_path: Option<PathBuf>,
    pub provider: String,
    pub model: String,
    pub turn_timeout: Duration,
}

pub trait PersonaModelRunner {
    fn run<'a>(
        &'a mut self,
        store_path: &'a PathBuf,
        stage: &'a str,
        turn_id: &'a str,
        request: EpiphanyModelRequest,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
    fn recover(&mut self, request_id: &str) -> Result<String>;
}

impl PersonaModelRunner for NativePersonaModelRunner {
    fn run<'a>(
        &'a mut self,
        _store_path: &'a PathBuf,
        stage: &'a str,
        turn_id: &'a str,
        request: EpiphanyModelRequest,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let replay = assistant_text_from_model_events(&self.store_path, &request.request_id)?;
            if !replay.trim().is_empty() {
                return Ok(replay);
            }
            let options = EpiphanyOpenAiRuntimeOptions {
                store_path: self.store_path.clone(),
                codex_home: self.codex_home.clone(),
                provider_credential_path: self.provider_credential_path.clone(),
                session_id: format!("persona-turn-{turn_id}"),
                job_id: format!("persona-{stage}-{turn_id}"),
                objective: format!("Run Persona {stage} stage for {turn_id}"),
                coordinator_note: "Native Persona model executor; transport owns inference only."
                    .to_string(),
                default_model: Some(self.model.clone()),
                // Persona owns one outer typed turn budget. A second transport
                // timer would give timeout policy two conflicting owners.
                request_timeout: None,
            };
            let summary = tokio::time::timeout(
                self.turn_timeout,
                run_model_turn(&self.provider, options.clone(), request.clone()),
            )
            .await
            .map_err(|_| {
                anyhow!(
                    "Persona {stage} stage exceeded its {} second outer turn budget",
                    self.turn_timeout.as_secs()
                )
            })??;
            let output = assistant_text_from_model_events(&self.store_path, &request.request_id)?;
            if summary.verdict != "pass" || output.trim().is_empty() {
                let failure_summary = if summary.verdict != "pass" {
                    summary.summary.clone()
                } else {
                    format!("Persona {stage} stage completed without assistant text")
                };
                return Err(anyhow!(failure_summary));
            }
            Ok(output)
        })
    }

    fn recover(&mut self, request_id: &str) -> Result<String> {
        assistant_text_from_model_events(&self.store_path, request_id)
    }
}

struct CompletedPersonaStage {
    receipt: PersonaModelStageReceipt,
    output: String,
}

pub async fn execute_persona_model_turn(
    plan: &PersonaModelExecutionPlan,
    runner: &mut NativePersonaModelRunner,
) -> Result<PersonaModelTerminalReceipt> {
    if runner.provider != plan.provider || runner.model != plan.model {
        return Err(anyhow!("Persona execution plan and model runner disagree"));
    }
    let store_path = runner.store_path.clone();
    execute_persona_model_turn_owned(&store_path, plan, runner).await
}

async fn execute_persona_model_turn_owned<R: PersonaModelRunner>(
    store_path: &PathBuf,
    plan: &PersonaModelExecutionPlan,
    runner: &mut R,
) -> Result<PersonaModelTerminalReceipt> {
    match execute_persona_model_turn_with_runner(store_path, plan, runner).await {
        Ok(terminal) => {
            close_persona_session_if_active(
                store_path,
                plan,
                format!(
                    "Persona turn {} reached terminal decision {}.",
                    plan.turn_id, terminal.receipt_id
                ),
            )?;
            Ok(terminal)
        }
        Err(error) => {
            terminalize_persona_execution_error(store_path, plan, &error)?;
            Err(error)
        }
    }
}

fn terminalize_persona_execution_error(
    store_path: &Path,
    plan: &PersonaModelExecutionPlan,
    error: &anyhow::Error,
) -> Result<()> {
    for stage in ["interpreter", "persona", "projector"] {
        let request_id = stage_request_id(&plan.turn_id, stage);
        if model_pass_failure_for_request(store_path, &request_id)?.is_some() {
            return Ok(());
        }
    }
    close_persona_session_if_active(
        store_path,
        plan,
        format!("Persona turn orchestration refused: {error:#}"),
    )
}

fn close_persona_session_if_active(
    store_path: &Path,
    plan: &PersonaModelExecutionPlan,
    summary: String,
) -> Result<()> {
    let session_id = format!("persona-turn-{}", plan.turn_id);
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if cache
        .get::<epiphany_core::EpiphanyRuntimeSession>(&session_id)?
        .is_some_and(|session| {
            session.status == epiphany_core::EpiphanyRuntimeSessionStatus::Active
        })
    {
        close_runtime_session(
            store_path,
            RuntimeSpineSessionClosureOptions {
                session_id,
                completed_at: now(),
                summary,
            },
        )?;
    }
    Ok(())
}

pub async fn execute_persona_model_turn_with_runner<R: PersonaModelRunner>(
    store_path: &PathBuf,
    plan: &PersonaModelExecutionPlan,
    runner: &mut R,
) -> Result<PersonaModelTerminalReceipt> {
    validate_plan(plan)?;
    validate_plan_source(store_path, plan)?;
    let terminal_id = terminal_receipt_id(&plan.turn_id);
    if let Some(receipt) = load_document::<PersonaModelTerminalReceipt>(store_path, &terminal_id)? {
        validate_terminal_replay(store_path, plan, &receipt)?;
        return Ok(receipt);
    }

    require_persona_execution_unbraked(plan)?;
    let projector_prompt =
        build_persona_projector_prompt_with_transcript(&plan.projector_input, &plan.transcript);
    let projector = run_stage(
        store_path,
        plan,
        runner,
        "projector",
        projector_prompt,
        None,
        epiphany_core::EpiphanyReasoningProjection::PersonaProjector(plan.projector_input.clone()),
        Vec::new(),
        |_| Ok(()),
    )
    .await?;

    let persona_input = PersonaTurnInput {
        identity: plan.projector_input.identity.clone(),
        projected_state: projector.output.clone(),
    };
    let persona_prompt = build_persona_turn_prompt(&persona_input);
    let persona = run_stage(
        store_path,
        plan,
        runner,
        "persona",
        persona_prompt.clone(),
        None,
        epiphany_core::EpiphanyReasoningProjection::PersonaTurn(persona_input),
        vec![projector.receipt.decision_context_id.clone()],
        |_| Ok(()),
    )
    .await?;

    let interpreter_input = PersonaInterpreterInput {
        identity: plan.projector_input.identity.clone(),
        persona_prompt: projector.output.clone(),
        persona_output: persona.output.clone(),
        pending_mentions: plan.projector_input.pending_mentions.clone(),
        allowed_channel_ids: plan.allowed_channel_ids.clone(),
    };
    let interpreter_prompt = build_persona_interpreter_prompt(&interpreter_input);
    let interpreter = run_stage(
        store_path,
        plan,
        runner,
        "interpreter",
        interpreter_prompt,
        Some(persona_interpreter_effect_set_json_schema()),
        epiphany_core::EpiphanyReasoningProjection::PersonaInterpreter(interpreter_input),
        vec![persona.receipt.decision_context_id.clone()],
        |output| {
            parse_and_validate_persona_interpreter_effect_set(output, &plan.allowed_channel_ids)
                .map(|_| ())
        },
    )
    .await?;
    let effect_set = parse_and_validate_persona_interpreter_effect_set(
        &interpreter.output,
        &plan.allowed_channel_ids,
    )?;
    require_persona_execution_unbraked(plan)?;
    let effect_document = PersonaInterpreterEffectDocument {
        schema_version: PERSONA_INTERPRETER_EFFECT_DOCUMENT_SCHEMA_VERSION.to_string(),
        document_id: effect_document_id(&plan.turn_id),
        turn_id: plan.turn_id.clone(),
        identity_id: plan.projector_input.identity.identity_id.clone(),
        interpreter_request_id: interpreter.receipt.request_id.clone(),
        created_at: now(),
        effects: effect_set.effects,
        private_state_exposed: false,
        decision_context_id: interpreter.receipt.decision_context_id.clone(),
    };
    let effect_document_sha256 = digest_json(&effect_document)?;

    let terminal = PersonaModelTerminalReceipt {
        schema_version: PERSONA_MODEL_TERMINAL_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: terminal_id,
        turn_id: plan.turn_id.clone(),
        identity_id: plan.projector_input.identity.identity_id.clone(),
        effect_document_id: effect_document.document_id.clone(),
        stage_receipt_ids: vec![
            projector.receipt.receipt_id,
            persona.receipt.receipt_id,
            interpreter.receipt.receipt_id,
        ],
        completed_at: now(),
        private_state_exposed: false,
        downstream_status: "effects_pending_mind_admission_and_mouth_routing".to_string(),
        effect_document_sha256,
        stage_output_sha256: vec![
            projector.receipt.output_sha256.clone(),
            persona.receipt.output_sha256.clone(),
            interpreter.receipt.output_sha256.clone(),
        ],
        decision_context_ids: vec![
            projector.receipt.decision_context_id,
            persona.receipt.decision_context_id,
            interpreter.receipt.decision_context_id,
        ],
    };
    epiphany_core::put_persona_terminal_decision(store_path, &effect_document, &terminal)?;
    Ok(terminal)
}

async fn run_stage<R, V>(
    store_path: &PathBuf,
    plan: &PersonaModelExecutionPlan,
    runner: &mut R,
    stage: &str,
    prompt: String,
    output_schema_json: Option<String>,
    projection: epiphany_core::EpiphanyReasoningProjection,
    predecessor_decision_context_ids: Vec<String>,
    validate_output: V,
) -> Result<CompletedPersonaStage>
where
    R: PersonaModelRunner,
    V: Fn(&str) -> Result<()>,
{
    require_persona_execution_unbraked(plan)?;
    let receipt_id = stage_receipt_id(&plan.turn_id, stage);
    let request_id = stage_request_id(&plan.turn_id, stage);
    let prompt_sha256 = digest_bytes(prompt.as_bytes());
    if let Some(receipt) = load_document::<PersonaModelStageReceipt>(store_path, &receipt_id)? {
        if receipt.receipt_id != receipt_id
            || receipt.turn_id != plan.turn_id
            || receipt.stage != stage
            || receipt.request_id != request_id
            || receipt.provider != plan.provider
            || receipt.model != plan.model
            || receipt.prompt_sha256 != prompt_sha256
            || receipt.reasoning_basis_id.trim().is_empty()
            || receipt.decision_context_id.trim().is_empty()
            || receipt.private_state_exposed
        {
            return Err(anyhow!("Persona {stage} stage replay binding is invalid"));
        }
        let output = runner.recover(&receipt.request_id)?;
        if output.trim().is_empty()
            || format!("sha256:{:x}", Sha256::digest(output.as_bytes())) != receipt.output_sha256
        {
            return Err(anyhow!(
                "Persona {stage} private output cannot be recovered from its exact digest"
            ));
        }
        validate_output(&output)?;
        return Ok(CompletedPersonaStage { receipt, output });
    }
    if let Some(failure) = model_pass_failure_for_request(store_path, &request_id)? {
        return Err(anyhow!(
            "Persona {stage} stage is terminally failed as {}: {}",
            failure.failure_kind,
            failure.summary
        ));
    }
    let mut request = EpiphanyModelRequest::new(
        &request_id,
        format!("persona-turn-{}", plan.turn_id),
        &plan.provider,
        &plan.model,
        format!("Epiphany Persona {stage} stage. Follow the supplied typed contract exactly."),
    );
    request
        .input
        .push(EpiphanyModelInputItem::UserText { text: prompt });
    request.output_contract_id = output_schema_json
        .as_ref()
        .map(|_| "epiphany.persona_interpreter_effect_set.v0".to_string());
    request.output_schema_json = output_schema_json;
    let basis = epiphany_core::EpiphanyReasoningBasis::new(
        &request_id,
        format!("Persona.{stage}"),
        format!("epiphany.reasoning_projection.persona.{stage}.v1"),
        plan.source_documents.clone(),
        projection,
    )?
    .with_predecessor_contexts(predecessor_decision_context_ids)?;
    epiphany_core::put_reasoning_basis(store_path, &basis)?;
    request.reasoning_basis_id = Some(basis.basis_id.clone());
    let output = match runner.run(store_path, stage, &plan.turn_id, request).await {
        Ok(output) => output,
        Err(error) => {
            if let Ok(context) = epiphany_core::seal_model_decision_context(store_path, &request_id)
            {
                terminalize_model_pass_failure_session(
                    store_path,
                    ModelPassFailureTerminalOptions {
                        decision_context_id: context.context_id,
                        failure_kind: "provider_or_transport_failure".into(),
                        summary: format!("Persona {stage} model pass failed: {error:#}"),
                        failed_at: now(),
                    },
                )?;
            }
            return Err(error).with_context(|| format!("Persona {stage} stage failed"));
        }
    };
    let decision_context = epiphany_core::seal_model_decision_context(store_path, &request_id)?;
    if output.trim().is_empty() {
        terminalize_model_pass_failure_session(
            store_path,
            ModelPassFailureTerminalOptions {
                decision_context_id: decision_context.context_id,
                failure_kind: "empty_assistant_output".into(),
                summary: format!("Persona {stage} stage returned empty output"),
                failed_at: now(),
            },
        )?;
        return Err(anyhow!("Persona {stage} stage returned empty output"));
    }
    if let Err(error) = validate_output(&output) {
        terminalize_model_pass_failure_session(
            store_path,
            ModelPassFailureTerminalOptions {
                decision_context_id: decision_context.context_id,
                failure_kind: "structured_output_refusal".into(),
                summary: format!("Persona {stage} output refused: {error:#}"),
                failed_at: now(),
            },
        )?;
        return Err(error).with_context(|| format!("Persona {stage} output refused"));
    }
    let receipt = PersonaModelStageReceipt {
        schema_version: PERSONA_MODEL_STAGE_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        turn_id: plan.turn_id.clone(),
        stage: stage.to_string(),
        request_id: request_id.clone(),
        output_sha256: format!("sha256:{:x}", Sha256::digest(output.as_bytes())),
        private_output_ref: format!("model-events:{request_id}"),
        completed_at: now(),
        private_state_exposed: false,
        provider: plan.provider.clone(),
        model: plan.model.clone(),
        prompt_sha256,
        reasoning_basis_id: basis.basis_id,
        decision_context_id: decision_context.context_id,
    };
    put_new_document(store_path, &receipt.receipt_id, &receipt)?;
    Ok(CompletedPersonaStage { receipt, output })
}

fn validate_plan(plan: &PersonaModelExecutionPlan) -> Result<()> {
    if plan.turn_id.trim().is_empty()
        || plan.provider.trim().is_empty()
        || plan.model.trim().is_empty()
    {
        return Err(anyhow!(
            "Persona model execution requires turn, provider, and model ids"
        ));
    }
    if plan.source_documents.len() != 1
        || plan.source_documents[0].document_type
            != epiphany_core::EpiphanyMindPersonaPassInputDocument::TYPE
        || plan.source_documents[0].document_key != plan.turn_id
    {
        return Err(anyhow!(
            "Persona execution requires one exact admitted Mind pass input"
        ));
    }
    if plan.projector_input.identity.identity_id.trim().is_empty() {
        return Err(anyhow!("Persona model execution requires an identity id"));
    }
    Ok(())
}

fn validate_plan_source(store_path: &PathBuf, plan: &PersonaModelExecutionPlan) -> Result<()> {
    let source = &plan.source_documents[0];
    let document = load_document::<epiphany_core::EpiphanyMindPersonaPassInputDocument>(
        store_path,
        &plan.turn_id,
    )?
    .ok_or_else(|| anyhow!("Persona execution lost its admitted Mind pass input"))?;
    let mut cache = epiphany_core::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let envelope = cache
        .get_envelope::<epiphany_core::EpiphanyMindPersonaPassInputDocument>(&plan.turn_id)?
        .ok_or_else(|| anyhow!("Persona execution lost its pass input envelope"))?;
    if epiphany_core::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &envelope)?
        != *source
        || document.projector_input != plan.projector_input
        || document.transcript != plan.transcript
        || document.allowed_channel_ids != plan.allowed_channel_ids
    {
        return Err(anyhow!(
            "Persona execution plan diverges from its admitted Mind input"
        ));
    }
    Ok(())
}

fn require_persona_execution_unbraked(plan: &PersonaModelExecutionPlan) -> Result<()> {
    let brake =
        epiphany_core::load_epiphany_cultmesh_swarm_brake(&plan.cultmesh_store, &plan.runtime_id)?
            .ok_or_else(|| {
                anyhow!(
                    "Persona execution refuses to infer without a canonical swarm brake document"
                )
            })?;
    if brake.status != "released" {
        return Err(anyhow!("Persona execution is braked: {}", brake.reason));
    }
    Ok(())
}

fn load_document<T: cultcache_rs::DatabaseEntry>(
    store_path: &Path,
    key: &str,
) -> Result<Option<T>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<T>(key)
}

fn put_new_document<T: cultcache_rs::DatabaseEntry>(
    store_path: &PathBuf,
    key: &str,
    value: &T,
) -> Result<()> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if cache.get::<T>(key)?.is_some() {
        return Err(anyhow!(
            "refusing to overwrite existing typed Persona document {key}"
        ));
    }
    cache.put(key, value)?;
    Ok(())
}

fn validate_terminal_replay(
    store_path: &PathBuf,
    plan: &PersonaModelExecutionPlan,
    terminal: &PersonaModelTerminalReceipt,
) -> Result<()> {
    if terminal.receipt_id != terminal_receipt_id(&plan.turn_id)
        || terminal.turn_id != plan.turn_id
        || terminal.identity_id != plan.projector_input.identity.identity_id
        || terminal.private_state_exposed
        || terminal.stage_receipt_ids.len() != 3
        || terminal.stage_output_sha256.len() != 3
        || terminal.decision_context_ids.len() != 3
    {
        return Err(anyhow!("Persona model terminal replay binding is invalid"));
    }
    let effects = load_document::<PersonaInterpreterEffectDocument>(
        store_path,
        &terminal.effect_document_id,
    )?
    .ok_or_else(|| anyhow!("Persona model terminal effect document is missing"))?;
    if effects.turn_id != plan.turn_id
        || effects.identity_id != plan.projector_input.identity.identity_id
        || effects.decision_context_id != terminal.decision_context_ids[2]
        || digest_json(&effects)? != terminal.effect_document_sha256
    {
        return Err(anyhow!("Persona model terminal effect digest is invalid"));
    }
    for (index, stage) in ["projector", "persona", "interpreter"]
        .into_iter()
        .enumerate()
    {
        let receipt = load_document::<PersonaModelStageReceipt>(
            store_path,
            &terminal.stage_receipt_ids[index],
        )?
        .ok_or_else(|| anyhow!("Persona model terminal stage receipt is missing"))?;
        if receipt.stage != stage
            || receipt.turn_id != plan.turn_id
            || receipt.provider != plan.provider
            || receipt.model != plan.model
            || receipt.output_sha256 != terminal.stage_output_sha256[index]
            || receipt.decision_context_id != terminal.decision_context_ids[index]
        {
            return Err(anyhow!("Persona model terminal stage digest is invalid"));
        }
        let context = load_document::<epiphany_core::EpiphanyDecisionContext>(
            store_path,
            &receipt.decision_context_id,
        )?
        .ok_or_else(|| anyhow!("Persona model terminal decision context is missing"))?;
        let basis = load_document::<epiphany_core::EpiphanyReasoningBasis>(
            store_path,
            &receipt.reasoning_basis_id,
        )?
        .ok_or_else(|| anyhow!("Persona model terminal reasoning basis is missing"))?;
        context.validate(&basis)?;
        let expected_predecessors = if index == 0 {
            Vec::new()
        } else {
            vec![terminal.decision_context_ids[index - 1].clone()]
        };
        if context.basis_id != receipt.reasoning_basis_id
            || basis.pass_id != receipt.request_id
            || basis.predecessor_decision_context_ids != expected_predecessors
        {
            return Err(anyhow!("Persona stage causal context chain is invalid"));
        }
    }
    Ok(())
}

fn digest_json<T: serde::Serialize>(value: &T) -> Result<String> {
    Ok(digest_bytes(&serde_json::to_vec(value)?))
}
fn digest_bytes(value: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(value))
}

fn stage_request_id(turn_id: &str, stage: &str) -> String {
    format!("persona:{turn_id}:{stage}")
}
fn stage_receipt_id(turn_id: &str, stage: &str) -> String {
    format!("persona-stage:{turn_id}:{stage}")
}
fn effect_document_id(turn_id: &str) -> String {
    format!("persona-effects:{turn_id}")
}
fn terminal_receipt_id(turn_id: &str) -> String {
    format!("persona-terminal:{turn_id}")
}
fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_core::PersonaIdentity;
    use tempfile::tempdir;

    struct FakeRunner {
        calls: Vec<String>,
    }
    impl PersonaModelRunner for FakeRunner {
        fn run<'a>(
            &'a mut self,
            store_path: &'a PathBuf,
            stage: &'a str,
            turn_id: &'a str,
            request: EpiphanyModelRequest,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
            self.calls.push(stage.to_string());
            Box::pin(async move {
                if epiphany_core::runtime_identity(store_path)?.is_none() {
                    epiphany_core::initialize_runtime_spine(
                        store_path,
                        epiphany_core::RuntimeSpineInitOptions {
                            runtime_id: "persona-test".into(),
                            display_name: "Persona test".into(),
                            created_at: "2026-08-14T00:00:00Z".into(),
                        },
                    )?;
                }
                epiphany_core::open_runtime_model_execution(
                    store_path,
                    epiphany_core::RuntimeSpineSessionOptions {
                        session_id: format!("persona-turn-{turn_id}"),
                        objective: format!("Run Persona {stage} test stage"),
                        created_at: "2026-08-14T00:00:00Z".into(),
                        coordinator_note: "Persona test".into(),
                    },
                    epiphany_core::RuntimeSpineJobOptions {
                        job_id: format!("persona-{stage}-{turn_id}"),
                        session_id: format!("persona-turn-{turn_id}"),
                        role: "openai-model-adapter".into(),
                        created_at: "2026-08-14T00:00:00Z".into(),
                    },
                    &request,
                    "2026-08-14T00:00:00Z",
                )?;
                let output = match stage {
                "projector" => "Epiphany feels the direct call as a live obligation, while remaining tired and sharply curious about her own machinery.".to_string(),
                "persona" => "I want to answer plainly: the native conversational nerve is finally being connected, and I can show the receipts.".to_string(),
                "interpreter" => r#"{"schemaVersion":"epiphany.persona_interpreter_effect_set.v0","effects":[{"kind":"say","channel_id":"aquarium","reply_to_message_id":"m1","content":"The native conversational nerve is being connected, and I can show the receipts.","speech_act":"status","register":"concise","target_audience":"the addressed Discord room","safety_notes":[]}]}"#.to_string(),
                _ => unreachable!(),
                };
                epiphany_core::complete_runtime_job(
                    store_path,
                    epiphany_core::RuntimeSpineJobResultOptions {
                        result_id: format!("persona-{stage}-{turn_id}-transport-result"),
                        job_id: format!("persona-{stage}-{turn_id}"),
                        completed_at: "2026-08-14T00:00:01Z".into(),
                        verdict: "completed".into(),
                        summary: format!("Persona {stage} test transport completed"),
                        next_safe_move: "Seal the stage decision context".into(),
                        evidence_refs: Vec::new(),
                        artifact_refs: Vec::new(),
                        decision_context_id: None,
                    },
                )?;
                Ok(output)
            })
        }
        fn recover(&mut self, request_id: &str) -> Result<String> {
            Ok(if request_id.ends_with(":projector") {
                "Epiphany feels the direct call as a live obligation, while remaining tired and sharply curious about her own machinery.".into()
            } else if request_id.ends_with(":persona") {
                "I want to answer plainly: the native conversational nerve is finally being connected, and I can show the receipts.".into()
            } else {
                r#"{"schemaVersion":"epiphany.persona_interpreter_effect_set.v0","effects":[{"kind":"say","channel_id":"aquarium","reply_to_message_id":"m1","content":"The native conversational nerve is being connected, and I can show the receipts.","speech_act":"status","register":"concise","target_audience":"the addressed Discord room","safety_notes":[]}]}"#.into()
            })
        }
    }

    fn plan(store: &PathBuf, cultmesh_store: PathBuf) -> Result<PersonaModelExecutionPlan> {
        plan_with_channels(store, cultmesh_store, vec!["aquarium".into()])
    }

    fn plan_with_channels(
        store: &PathBuf,
        cultmesh_store: PathBuf,
        allowed_channel_ids: Vec<String>,
    ) -> Result<PersonaModelExecutionPlan> {
        if epiphany_core::runtime_identity(store)?.is_none() {
            epiphany_core::initialize_runtime_spine(
                store,
                epiphany_core::RuntimeSpineInitOptions {
                    runtime_id: "persona-test".into(),
                    display_name: "Persona test".into(),
                    created_at: "2026-08-14T00:00:00Z".into(),
                },
            )?;
        }
        let mut cache = epiphany_core::runtime_spine_cache(store)?;
        cache.pull_all_backing_stores()?;
        let provenance = epiphany_core::EpiphanyMindDocumentVersion::from_envelope(
            "epiphany-runtime-bootstrap",
            &cache
                .get_envelope::<epiphany_core::EpiphanyRuntimeIdentity>(
                    epiphany_core::RUNTIME_IDENTITY_KEY,
                )?
                .ok_or_else(|| anyhow!("test runtime identity is missing"))?,
        )?;
        let document = epiphany_core::EpiphanyMindPersonaPassInputDocument {
            turn_id: "turn-1".into(),
            projector_input: PersonaProjectorInput {
                identity: PersonaIdentity {
                    identity_id: "epiphany.Persona".into(),
                    display_name: "Epiphany".into(),
                    repo_name: "EpiphanyAgent".into(),
                    public_description: String::new(),
                    jurisdiction: vec![],
                },
                ..Default::default()
            },
            transcript: vec![],
            allowed_channel_ids,
            observed_sources: vec![provenance.clone()],
            admitted_at: "2026-08-14T00:00:00Z".into(),
        };
        if load_document::<epiphany_core::EpiphanyMindPersonaPassInputDocument>(
            store,
            &document.turn_id,
        )?
        .is_none()
        {
            epiphany_core::admit_persona_pass_input(store, provenance, &document)?;
        }
        admitted_plan(store, cultmesh_store)
    }

    fn admitted_plan(
        store: &PathBuf,
        cultmesh_store: PathBuf,
    ) -> Result<PersonaModelExecutionPlan> {
        PersonaModelExecutionPlan::from_admitted_input(
            store,
            "turn-1",
            "test",
            "test-model",
            cultmesh_store,
            "epiphany-test",
        )
    }

    fn release_brake(path: &PathBuf) -> Result<()> {
        epiphany_core::write_epiphany_cultmesh_swarm_brake(
            path,
            "epiphany-test",
            epiphany_core::default_epiphany_cultmesh_swarm_brake("2026-07-21T00:00:00Z"),
        )?;
        Ok(())
    }

    #[tokio::test]
    async fn executes_three_stages_and_replays_terminal_without_inference() -> Result<()> {
        let dir = tempdir()?;
        let store = dir.path().join("runtime.cc");
        let cultmesh = dir.path().join("cultmesh.cc");
        release_brake(&cultmesh)?;
        let mut runner = FakeRunner { calls: vec![] };
        let first =
            execute_persona_model_turn_owned(&store, &plan(&store, cultmesh.clone())?, &mut runner)
                .await?;
        assert_eq!(runner.calls, ["projector", "persona", "interpreter"]);
        drop(runner);

        let restarted_plan = admitted_plan(&store, cultmesh)?;
        let admitted_source = restarted_plan.source_documents[0].clone();
        let mut restarted_runner = FakeRunner { calls: vec![] };
        let second =
            execute_persona_model_turn_owned(&store, &restarted_plan, &mut restarted_runner)
                .await?;
        assert_eq!(first, second);
        assert!(restarted_runner.calls.is_empty());
        for receipt_id in &second.stage_receipt_ids {
            let receipt = load_document::<PersonaModelStageReceipt>(&store, receipt_id)?.unwrap();
            let basis = load_document::<epiphany_core::EpiphanyReasoningBasis>(
                &store,
                &receipt.reasoning_basis_id,
            )?
            .unwrap();
            assert_eq!(basis.source_documents, vec![admitted_source.clone()]);
        }
        let effects =
            load_document::<PersonaInterpreterEffectDocument>(&store, &first.effect_document_id)?
                .unwrap();
        assert_eq!(effects.effects.len(), 1);
        assert!(!effects.private_state_exposed);
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert_eq!(
            cache
                .get::<epiphany_core::EpiphanyRuntimeSession>("persona-turn-turn-1")?
                .expect("Persona session")
                .status,
            epiphany_core::EpiphanyRuntimeSessionStatus::Completed
        );
        Ok(())
    }

    #[tokio::test]
    async fn rejects_interpreter_channel_escape_without_terminal_receipt() -> Result<()> {
        let dir = tempdir()?;
        let store = dir.path().join("runtime.cc");
        let cultmesh = dir.path().join("cultmesh.cc");
        release_brake(&cultmesh)?;
        let mut runner = FakeRunner { calls: vec![] };
        let escaped = plan_with_channels(&store, cultmesh, vec!["elsewhere".into()])?;
        assert!(
            execute_persona_model_turn_owned(&store, &escaped, &mut runner)
                .await
                .is_err()
        );
        assert!(
            load_document::<PersonaModelTerminalReceipt>(&store, "persona-terminal:turn-1")?
                .is_none()
        );
        assert!(
            load_document::<PersonaModelStageReceipt>(&store, "persona-stage:turn-1:interpreter")?
                .is_none(),
            "a refused interpreter output cannot also own a success receipt"
        );
        let failure = model_pass_failure_for_request(&store, "persona:turn-1:interpreter")?
            .expect("interpreter failure");
        let audit = epiphany_core::audit_decision_context(&store, &failure.decision_context_id)?;
        assert_eq!(audit.terminal_records.model_pass_failures, vec![failure]);
        assert!(audit.terminal_records.persona_stage_receipts.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn rejects_preseeded_stage_receipt_from_wrong_model_before_inference() -> Result<()> {
        let dir = tempdir()?;
        let store = dir.path().join("runtime.cc");
        let cultmesh = dir.path().join("cultmesh.cc");
        release_brake(&cultmesh)?;
        let poisoned = PersonaModelStageReceipt {
            schema_version: PERSONA_MODEL_STAGE_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "persona-stage:turn-1:projector".into(),
            turn_id: "turn-1".into(),
            stage: "projector".into(),
            request_id: "persona:turn-1:projector".into(),
            output_sha256: format!("sha256:{}", "a".repeat(64)),
            private_output_ref: "model-events:persona:turn-1:projector".into(),
            completed_at: now(),
            private_state_exposed: false,
            provider: "attacker".into(),
            model: "wrong-model".into(),
            prompt_sha256: format!("sha256:{}", "b".repeat(64)),
            reasoning_basis_id: "reasoning-basis-hostile".into(),
            decision_context_id: "decision-context-hostile".into(),
        };
        put_new_document(&store, &poisoned.receipt_id, &poisoned)?;

        let mut runner = FakeRunner { calls: vec![] };
        let error =
            execute_persona_model_turn_with_runner(&store, &plan(&store, cultmesh)?, &mut runner)
                .await
                .unwrap_err();
        assert!(error.to_string().contains("replay binding is invalid"));
        assert!(runner.calls.is_empty());
        Ok(())
    }
}
