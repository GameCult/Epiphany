use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use anyhow::{Context, Result, anyhow};
use chrono::SecondsFormat;
use epiphany_core::{
    PERSONA_INTERPRETER_EFFECT_DOCUMENT_SCHEMA_VERSION, PERSONA_MODEL_STAGE_RECEIPT_SCHEMA_VERSION,
    PERSONA_MODEL_TERMINAL_RECEIPT_SCHEMA_VERSION, PersonaInterpreterEffectDocument,
    PersonaInterpreterInput, PersonaModelStageReceipt, PersonaModelTerminalReceipt,
    PersonaProjectorInput, PersonaTranscriptMessage, PersonaTurnInput,
    build_persona_interpreter_prompt, build_persona_projector_prompt, build_persona_turn_prompt,
    parse_and_validate_persona_interpreter_effect_set, persona_interpreter_effect_set_json_schema,
    persona_projected_surface_is_clean, runtime_spine_cache,
};
use epiphany_model_adapter::{EpiphanyModelInputItem, EpiphanyModelRequest};
use sha2::{Digest, Sha256};

use crate::{EpiphanyOpenAiRuntimeOptions, assistant_text_from_model_events, run_model_turn};

#[derive(Clone, Debug)]
pub struct PersonaModelExecutionPlan {
    pub turn_id: String,
    pub provider: String,
    pub model: String,
    pub projector_input: PersonaProjectorInput,
    pub transcript: Vec<PersonaTranscriptMessage>,
    pub allowed_channel_ids: Vec<String>,
    pub dynamic_semantic_memory_recall: String,
    pub cultmesh_store: PathBuf,
    pub runtime_id: String,
}

#[derive(Clone, Debug)]
pub struct NativePersonaModelRunner {
    pub store_path: PathBuf,
    pub codex_home: PathBuf,
    pub provider: String,
    pub model: String,
}

pub trait PersonaModelRunner {
    fn run<'a>(
        &'a mut self,
        stage: &'a str,
        turn_id: &'a str,
        request: EpiphanyModelRequest,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>>;
    fn recover(&mut self, request_id: &str) -> Result<String>;
}

impl PersonaModelRunner for NativePersonaModelRunner {
    fn run<'a>(
        &'a mut self,
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
                session_id: format!("persona-turn-{turn_id}"),
                job_id: format!("persona-{stage}-{turn_id}"),
                objective: format!("Run Persona {stage} stage for {turn_id}"),
                coordinator_note: "Native Persona model executor; transport owns inference only."
                    .to_string(),
                default_model: Some(self.model.clone()),
            };
            run_model_turn(&self.provider, options, request.clone()).await?;
            let output = assistant_text_from_model_events(&self.store_path, &request.request_id)?;
            if output.trim().is_empty() {
                return Err(anyhow!(
                    "Persona {stage} stage completed without assistant text"
                ));
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
    execute_persona_model_turn_with_runner(&runner.store_path.clone(), plan, runner).await
}

pub async fn execute_persona_model_turn_with_runner<R: PersonaModelRunner>(
    store_path: &PathBuf,
    plan: &PersonaModelExecutionPlan,
    runner: &mut R,
) -> Result<PersonaModelTerminalReceipt> {
    validate_plan(plan)?;
    let terminal_id = terminal_receipt_id(&plan.turn_id);
    if let Some(receipt) = load_document::<PersonaModelTerminalReceipt>(store_path, &terminal_id)? {
        validate_terminal_replay(store_path, plan, &receipt)?;
        return Ok(receipt);
    }

    require_persona_execution_unbraked(plan)?;
    let projector_prompt = build_persona_projector_prompt(&plan.projector_input);
    let projector = run_stage(
        store_path,
        plan,
        runner,
        "projector",
        projector_prompt,
        None,
    )
    .await?;
    if !persona_projected_surface_is_clean(&projector.output) {
        return Err(anyhow!(
            "Persona Projector leaked action or substrate syntax"
        ));
    }

    let persona_prompt = build_persona_turn_prompt(&PersonaTurnInput {
        identity: plan.projector_input.identity.clone(),
        projected_state: projector.output.clone(),
        semantic_memory_recall: plan.projector_input.semantic_memory_recall.clone(),
        pending_mentions: plan.projector_input.pending_mentions.clone(),
        repo_activity: plan.projector_input.repo_activity.clone(),
        social_affordances: plan.projector_input.social_affordances.clone(),
        transcript: plan.transcript.clone(),
    });
    let persona = run_stage(
        store_path,
        plan,
        runner,
        "persona",
        persona_prompt.clone(),
        None,
    )
    .await?;

    let interpreter_prompt = build_persona_interpreter_prompt(&PersonaInterpreterInput {
        identity: plan.projector_input.identity.clone(),
        persona_prompt,
        persona_output: persona.output.clone(),
        semantic_memory_recall: plan.projector_input.semantic_memory_recall.clone(),
        dynamic_semantic_memory_recall: plan.dynamic_semantic_memory_recall.clone(),
        pending_mentions: plan.projector_input.pending_mentions.clone(),
        allowed_channel_ids: plan.allowed_channel_ids.clone(),
    });
    let interpreter = run_stage(
        store_path,
        plan,
        runner,
        "interpreter",
        interpreter_prompt,
        Some(persona_interpreter_effect_set_json_schema()),
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
    };
    put_new_document(store_path, &effect_document.document_id, &effect_document)?;
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
    };
    put_new_document(store_path, &terminal.receipt_id, &terminal)?;
    Ok(terminal)
}

async fn run_stage<R: PersonaModelRunner>(
    store_path: &PathBuf,
    plan: &PersonaModelExecutionPlan,
    runner: &mut R,
    stage: &str,
    prompt: String,
    output_schema_json: Option<String>,
) -> Result<CompletedPersonaStage> {
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
        return Ok(CompletedPersonaStage { receipt, output });
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
    let output = runner
        .run(stage, &plan.turn_id, request)
        .await
        .with_context(|| format!("Persona {stage} stage failed"))?;
    if output.trim().is_empty() {
        return Err(anyhow!("Persona {stage} stage returned empty output"));
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
    if plan.projector_input.identity.identity_id.trim().is_empty() {
        return Err(anyhow!("Persona model execution requires an identity id"));
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
    store_path: &PathBuf,
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
        {
            return Err(anyhow!("Persona model terminal stage digest is invalid"));
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
            stage: &'a str,
            _turn_id: &'a str,
            _request: EpiphanyModelRequest,
        ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'a>> {
            self.calls.push(stage.to_string());
            Box::pin(async move {
                Ok(match stage {
                "projector" => "Epiphany feels the direct call as a live obligation, while remaining tired and sharply curious about her own machinery.".to_string(),
                "persona" => "I want to answer plainly: the native conversational nerve is finally being connected, and I can show the receipts.".to_string(),
                "interpreter" => r#"{"schemaVersion":"epiphany.persona_interpreter_effect_set.v0","effects":[{"kind":"say","channel_id":"aquarium","reply_to_message_id":"m1","content":"The native conversational nerve is being connected, and I can show the receipts.","speech_act":"status","register":"concise","target_audience":"the addressed Discord room","safety_notes":[]}]}"#.to_string(),
                _ => unreachable!(),
            })
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

    fn plan(cultmesh_store: PathBuf) -> PersonaModelExecutionPlan {
        PersonaModelExecutionPlan {
            turn_id: "turn-1".into(),
            provider: "test".into(),
            model: "test-model".into(),
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
            allowed_channel_ids: vec!["aquarium".into()],
            dynamic_semantic_memory_recall: String::new(),
            cultmesh_store,
            runtime_id: "epiphany-test".into(),
        }
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
            execute_persona_model_turn_with_runner(&store, &plan(cultmesh.clone()), &mut runner)
                .await?;
        assert_eq!(runner.calls, ["projector", "persona", "interpreter"]);
        let second =
            execute_persona_model_turn_with_runner(&store, &plan(cultmesh), &mut runner).await?;
        assert_eq!(first, second);
        assert_eq!(runner.calls.len(), 3);
        let effects =
            load_document::<PersonaInterpreterEffectDocument>(&store, &first.effect_document_id)?
                .unwrap();
        assert_eq!(effects.effects.len(), 1);
        assert!(!effects.private_state_exposed);
        Ok(())
    }

    #[tokio::test]
    async fn rejects_interpreter_channel_escape_without_terminal_receipt() -> Result<()> {
        let dir = tempdir()?;
        let store = dir.path().join("runtime.cc");
        let cultmesh = dir.path().join("cultmesh.cc");
        release_brake(&cultmesh)?;
        let mut runner = FakeRunner { calls: vec![] };
        let mut escaped = plan(cultmesh);
        escaped.allowed_channel_ids = vec!["elsewhere".into()];
        assert!(
            execute_persona_model_turn_with_runner(&store, &escaped, &mut runner)
                .await
                .is_err()
        );
        assert!(
            load_document::<PersonaModelTerminalReceipt>(&store, "persona-terminal:turn-1")?
                .is_none()
        );
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
        };
        put_new_document(&store, &poisoned.receipt_id, &poisoned)?;

        let mut runner = FakeRunner { calls: vec![] };
        let error = execute_persona_model_turn_with_runner(&store, &plan(cultmesh), &mut runner)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("replay binding is invalid"));
        assert!(runner.calls.is_empty());
        Ok(())
    }
}
