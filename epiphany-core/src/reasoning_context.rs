use crate::runtime_store_backend::runtime_spine_backing_store;
use crate::{
    EpiphanyRoleWorkerLaunchDocument, EpiphanyRuntimeWorkerLaunchRequest,
    EpiphanyWorkerLaunchDocument, PersonaInterpreterInput, PersonaProjectorInput, PersonaTurnInput,
    runtime_spine_cache,
};
use anyhow::{Result, anyhow};
use cultcache_rs::{CacheBackingStore, CultCacheEnvelope, DatabaseEntry};
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_tool_adapter::{
    EpiphanyToolInvocationIntent, EpiphanyToolInvocationReceipt, receipt_output_for_model,
    tool_invocation_intent_key, tool_invocation_receipt_key,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;

pub const REASONING_BASIS_SCHEMA_VERSION: &str = "epiphany.reasoning_basis.v1";
pub const DECISION_CONTEXT_SCHEMA_VERSION: &str = "epiphany.decision_context.v1";
pub const MIND_COMMIT_RECEIPT_SCHEMA_VERSION: &str = "epiphany.mind_commit_receipt.v1";
pub const WORKER_REASONING_PROJECTION_POLICY: &str =
    "epiphany.reasoning_projection.worker_launch.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyMindDocumentVersion {
    pub store_id: String,
    pub document_type: String,
    pub document_key: String,
    pub schema_id: Option<String>,
    pub payload_msgpack: Vec<u8>,
    pub payload_sha256: String,
}

impl EpiphanyMindDocumentVersion {
    pub fn from_envelope(store_id: &str, envelope: &CultCacheEnvelope) -> Result<Self> {
        require_non_empty(store_id, "Mind document store id")?;
        require_non_empty(&envelope.r#type, "Mind document type")?;
        require_non_empty(&envelope.key, "Mind document key")?;
        let payload_sha256 = sha256(&envelope.payload);
        Ok(Self {
            store_id: store_id.to_string(),
            document_type: envelope.r#type.clone(),
            document_key: envelope.key.clone(),
            schema_id: envelope.schema_id.clone(),
            payload_msgpack: envelope.payload.clone(),
            payload_sha256,
        })
    }

    pub fn validate(&self) -> Result<()> {
        require_non_empty(&self.store_id, "Mind document store id")?;
        require_non_empty(&self.document_type, "Mind document type")?;
        require_non_empty(&self.document_key, "Mind document key")?;
        if self.payload_sha256 != sha256(&self.payload_msgpack) {
            return Err(anyhow!(
                "Mind document {:?}/{:?} payload digest mismatch",
                self.document_type,
                self.document_key
            ));
        }
        Ok(())
    }

    pub fn identity(&self) -> (&str, &str) {
        (&self.document_type, &self.document_key)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EpiphanyReasoningProjection {
    RoleLaunch(EpiphanyRoleWorkerLaunchDocument),
    ReorientLaunch(crate::EpiphanyReorientWorkerLaunchDocument),
    PersonaProjector(PersonaProjectorInput),
    PersonaTurn(PersonaTurnInput),
    PersonaInterpreter(PersonaInterpreterInput),
}

impl From<EpiphanyWorkerLaunchDocument> for EpiphanyReasoningProjection {
    fn from(value: EpiphanyWorkerLaunchDocument) -> Self {
        match value {
            EpiphanyWorkerLaunchDocument::Role(document) => Self::RoleLaunch(document),
            EpiphanyWorkerLaunchDocument::Reorient(document) => Self::ReorientLaunch(document),
        }
    }
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.reasoning_basis.v1",
    schema = "EpiphanyReasoningBasis"
)]
pub struct EpiphanyReasoningBasis {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub basis_id: String,
    #[cultcache(key = 2)]
    pub pass_id: String,
    #[cultcache(key = 3)]
    pub organ_id: String,
    #[cultcache(key = 4)]
    pub projection_policy_id: String,
    #[cultcache(key = 5)]
    pub source_documents: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 6)]
    pub projection_msgpack: Vec<u8>,
    #[cultcache(key = 7, default)]
    pub predecessor_decision_context_ids: Vec<String>,
}

impl EpiphanyReasoningBasis {
    pub fn new(
        pass_id: impl Into<String>,
        organ_id: impl Into<String>,
        projection_policy_id: impl Into<String>,
        mut source_documents: Vec<EpiphanyMindDocumentVersion>,
        projection: EpiphanyReasoningProjection,
    ) -> Result<Self> {
        let pass_id = pass_id.into();
        let organ_id = organ_id.into();
        let projection_policy_id = projection_policy_id.into();
        require_non_empty(&pass_id, "reasoning pass id")?;
        require_non_empty(&organ_id, "reasoning organ id")?;
        require_non_empty(&projection_policy_id, "reasoning projection policy id")?;
        canonicalize_source_documents(&mut source_documents)?;
        let projection_msgpack = rmp_serde::to_vec_named(&projection)?;
        let mut basis = Self {
            schema_version: REASONING_BASIS_SCHEMA_VERSION.to_string(),
            basis_id: String::new(),
            pass_id,
            organ_id,
            projection_policy_id,
            source_documents,
            projection_msgpack,
            predecessor_decision_context_ids: Vec::new(),
        };
        basis.basis_id = format!("reasoning-basis-{}", digest_without_basis_id(&basis)?);
        basis.validate()?;
        Ok(basis)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != REASONING_BASIS_SCHEMA_VERSION {
            return Err(anyhow!("unsupported reasoning basis schema"));
        }
        require_non_empty(&self.basis_id, "reasoning basis id")?;
        require_non_empty(&self.pass_id, "reasoning pass id")?;
        require_non_empty(&self.organ_id, "reasoning organ id")?;
        require_non_empty(&self.projection_policy_id, "reasoning projection policy id")?;
        let mut canonical = self.source_documents.clone();
        canonicalize_source_documents(&mut canonical)?;
        if canonical != self.source_documents {
            return Err(anyhow!(
                "reasoning basis source documents are not canonical"
            ));
        }
        let _: EpiphanyReasoningProjection = rmp_serde::from_slice(&self.projection_msgpack)
            .map_err(|error| anyhow!("reasoning projection is invalid: {error}"))?;
        if self
            .predecessor_decision_context_ids
            .iter()
            .any(|context_id| context_id.trim().is_empty())
            || self
                .predecessor_decision_context_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(anyhow!(
                "reasoning basis predecessor contexts are not canonical"
            ));
        }
        let expected = format!("reasoning-basis-{}", digest_without_basis_id(self)?);
        if self.basis_id != expected {
            return Err(anyhow!("reasoning basis identity digest mismatch"));
        }
        Ok(())
    }

    pub fn projection(&self) -> Result<EpiphanyReasoningProjection> {
        rmp_serde::from_slice(&self.projection_msgpack)
            .map_err(|error| anyhow!("reasoning projection is invalid: {error}"))
    }

    pub fn with_predecessor_contexts(
        mut self,
        mut predecessor_decision_context_ids: Vec<String>,
    ) -> Result<Self> {
        predecessor_decision_context_ids.sort();
        predecessor_decision_context_ids.dedup();
        self.predecessor_decision_context_ids = predecessor_decision_context_ids;
        self.basis_id.clear();
        self.basis_id = format!("reasoning-basis-{}", digest_without_basis_id(&self)?);
        self.validate()?;
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyDecisionToolObservation {
    pub intent: EpiphanyToolInvocationIntent,
    pub receipt: EpiphanyToolInvocationReceipt,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.decision_context.v1",
    schema = "EpiphanyDecisionContext"
)]
pub struct EpiphanyDecisionContext {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub context_id: String,
    #[cultcache(key = 2)]
    pub basis_id: String,
    #[cultcache(key = 3)]
    pub terminal_request_id: String,
    #[cultcache(key = 4)]
    pub native_request_msgpack: Vec<u8>,
    #[cultcache(key = 5)]
    pub provider_request_msgpack: Vec<u8>,
    #[cultcache(key = 6)]
    pub tool_observations_msgpack: Vec<u8>,
}

impl EpiphanyDecisionContext {
    pub fn new(
        basis: &EpiphanyReasoningBasis,
        native_request: EpiphanyModelRequest,
        provider_request: EpiphanyOpenAiModelRequest,
        tool_observations: Vec<EpiphanyDecisionToolObservation>,
    ) -> Result<Self> {
        basis.validate()?;
        validate_request_pair(basis, &native_request, &provider_request)?;
        validate_tool_observations(&native_request, &tool_observations)?;
        let mut context = Self {
            schema_version: DECISION_CONTEXT_SCHEMA_VERSION.to_string(),
            context_id: String::new(),
            basis_id: basis.basis_id.clone(),
            terminal_request_id: native_request.request_id.clone(),
            native_request_msgpack: rmp_serde::to_vec_named(&native_request)?,
            provider_request_msgpack: rmp_serde::to_vec_named(&provider_request)?,
            tool_observations_msgpack: rmp_serde::to_vec_named(&tool_observations)?,
        };
        context.context_id = format!("decision-context-{}", digest_without_context_id(&context)?);
        context.validate(basis)?;
        Ok(context)
    }

    pub fn validate(&self, basis: &EpiphanyReasoningBasis) -> Result<()> {
        if self.schema_version != DECISION_CONTEXT_SCHEMA_VERSION {
            return Err(anyhow!("unsupported decision context schema"));
        }
        let native_request = self.native_request()?;
        let provider_request = self.provider_request()?;
        let tool_observations = self.tool_observations()?;
        if self.basis_id != basis.basis_id || self.terminal_request_id != native_request.request_id
        {
            return Err(anyhow!("decision context ownership mismatch"));
        }
        validate_request_pair(basis, &native_request, &provider_request)?;
        validate_tool_observations(&native_request, &tool_observations)?;
        let expected = format!("decision-context-{}", digest_without_context_id(self)?);
        if self.context_id != expected {
            return Err(anyhow!("decision context identity digest mismatch"));
        }
        Ok(())
    }

    pub fn native_request(&self) -> Result<EpiphanyModelRequest> {
        rmp_serde::from_slice(&self.native_request_msgpack)
            .map_err(|error| anyhow!("decision native request is invalid: {error}"))
    }

    pub fn provider_request(&self) -> Result<EpiphanyOpenAiModelRequest> {
        rmp_serde::from_slice(&self.provider_request_msgpack)
            .map_err(|error| anyhow!("decision provider request is invalid: {error}"))
    }

    pub fn tool_observations(&self) -> Result<Vec<EpiphanyDecisionToolObservation>> {
        rmp_serde::from_slice(&self.tool_observations_msgpack)
            .map_err(|error| anyhow!("decision tool observations are invalid: {error}"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind_commit_receipt.v1",
    schema = "EpiphanyMindCommitReceipt"
)]
pub struct EpiphanyMindCommitReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub authority: EpiphanyMindCommitAuthority,
    #[cultcache(key = 3)]
    pub invariant_owner: String,
    #[cultcache(key = 4)]
    pub strong_reads: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 5)]
    pub writes: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 6)]
    pub committed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpiphanyMindCommitAuthority {
    ModelDecisionContext {
        decision_context_id: String,
    },
    OperatorProvenance {
        provenance: EpiphanyMindDocumentVersion,
    },
    TypedOrganProvenance {
        organ: String,
        provenance: EpiphanyMindDocumentVersion,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EpiphanyMindCommitOutcome {
    Committed(EpiphanyMindCommitReceipt),
    Conflict {
        document_identities: Vec<(String, String)>,
    },
}

pub fn worker_reasoning_basis(
    store_path: &Path,
    launch: &EpiphanyRuntimeWorkerLaunchRequest,
) -> Result<EpiphanyReasoningBasis> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let launch_envelope = cache
        .get_envelope::<EpiphanyRuntimeWorkerLaunchRequest>(&launch.job_id)?
        .ok_or_else(|| anyhow!("worker reasoning basis lost its launch envelope"))?;
    let source_documents = vec![EpiphanyMindDocumentVersion::from_envelope(
        "epiphany-mind",
        &launch_envelope,
    )?];
    EpiphanyReasoningBasis::new(
        &launch.job_id,
        &launch.role,
        WORKER_REASONING_PROJECTION_POLICY,
        source_documents,
        launch.launch_document()?.into(),
    )
}

pub fn put_reasoning_basis(
    store_path: &Path,
    basis: &EpiphanyReasoningBasis,
) -> Result<EpiphanyReasoningBasis> {
    basis.validate()?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    for source in &basis.source_documents {
        if source.store_id != "epiphany-mind" {
            return Err(anyhow!(
                "reasoning basis source store {:?} is not an admitted Mind store",
                source.store_id
            ));
        }
        let live = cache
            .snapshot_envelopes()
            .into_iter()
            .find(|envelope| {
                envelope.r#type == source.document_type && envelope.key == source.document_key
            })
            .ok_or_else(|| {
                anyhow!(
                    "reasoning basis source {:?}/{:?} is absent",
                    source.document_type,
                    source.document_key
                )
            })?;
        if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &live)? != *source {
            return Err(anyhow!(
                "reasoning basis source {:?}/{:?} changed before sealing",
                source.document_type,
                source.document_key
            ));
        }
    }
    if let Some(existing) = cache.get::<EpiphanyReasoningBasis>(&basis.basis_id)? {
        if existing != *basis {
            return Err(anyhow!("reasoning basis identity collision"));
        }
        return Ok(existing);
    }
    let envelope = cache.prepare_entry(&basis.basis_id, basis)?.0;
    if !runtime_spine_backing_store(store_path)?.compare_and_swap_batch(&[], vec![envelope])? {
        return put_reasoning_basis(store_path, basis);
    }
    Ok(basis.clone())
}

pub fn put_decision_context(
    store_path: &Path,
    context: &EpiphanyDecisionContext,
) -> Result<EpiphanyDecisionContext> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let basis = cache
        .get::<EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("decision context lost its reasoning basis"))?;
    context.validate(&basis)?;
    validate_context_store_ownership(&cache, context)?;
    if let Some(existing) = cache.get::<EpiphanyDecisionContext>(&context.context_id)? {
        if existing != *context {
            return Err(anyhow!("decision context identity collision"));
        }
        return Ok(existing);
    }
    let basis_envelope = cache
        .get_envelope::<EpiphanyReasoningBasis>(&basis.basis_id)?
        .ok_or_else(|| anyhow!("decision context lost its basis envelope"))?;
    let context_envelope = cache.prepare_entry(&context.context_id, context)?.0;
    if !runtime_spine_backing_store(store_path)?.compare_and_swap_batch(
        &[basis_envelope.clone()],
        vec![basis_envelope, context_envelope],
    )? {
        return put_decision_context(store_path, context);
    }
    Ok(context.clone())
}

pub fn seal_model_decision_context(
    store_path: &Path,
    terminal_request_id: &str,
) -> Result<EpiphanyDecisionContext> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let native = cache
        .get::<EpiphanyModelRequest>(terminal_request_id)?
        .ok_or_else(|| anyhow!("terminal native model request is absent"))?;
    let provider = cache
        .get::<EpiphanyOpenAiModelRequest>(terminal_request_id)?
        .ok_or_else(|| anyhow!("terminal provider model request is absent"))?;
    let basis_id = native
        .reasoning_basis_id
        .as_deref()
        .ok_or_else(|| anyhow!("terminal model request has no reasoning basis"))?;
    let basis = cache
        .get::<EpiphanyReasoningBasis>(basis_id)?
        .ok_or_else(|| anyhow!("terminal model request lost its reasoning basis"))?;
    let intents = cache.get_all::<EpiphanyToolInvocationIntent>()?;
    let mut observations = Vec::new();
    for input in &native.input {
        let epiphany_model_adapter::EpiphanyModelInputItem::ToolCall {
            call_id,
            name,
            arguments,
        } = input
        else {
            continue;
        };
        let mut matches = intents.iter().filter(|intent| {
            intent.call_id.as_deref() == Some(call_id.as_str())
                && format!("mcp__{}__{}", intent.server, intent.tool_name) == *name
                && intent.arguments_json == *arguments
        });
        let intent = matches
            .next()
            .ok_or_else(|| anyhow!("terminal tool call has no exact governed intent"))?;
        if matches.next().is_some() {
            return Err(anyhow!(
                "terminal tool call has ambiguous governed ownership"
            ));
        }
        let receipt = cache
            .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(&intent.intent_id))?
            .ok_or_else(|| anyhow!("terminal tool call has no governed receipt"))?;
        observations.push(EpiphanyDecisionToolObservation {
            intent: intent.clone(),
            receipt,
        });
    }
    let context = EpiphanyDecisionContext::new(&basis, native, provider, observations)?;
    put_decision_context(store_path, &context)
}

fn validate_context_store_ownership(
    cache: &cultcache_rs::CultCache,
    context: &EpiphanyDecisionContext,
) -> Result<()> {
    let native = context.native_request()?;
    let provider = context.provider_request()?;
    let observations = context.tool_observations()?;
    if cache
        .get::<EpiphanyModelRequest>(&native.request_id)?
        .as_ref()
        != Some(&native)
        || cache
            .get::<EpiphanyOpenAiModelRequest>(&provider.request_id)?
            .as_ref()
            != Some(&provider)
    {
        return Err(anyhow!(
            "decision context terminal request family is absent or substituted"
        ));
    }
    let model_binding = cache
        .get::<crate::EpiphanyRuntimeModelExecutionBinding>(&native.request_id)?
        .ok_or_else(|| anyhow!("decision context terminal request has no runtime binding"))?;
    if model_binding.request_id != native.request_id
        || model_binding.reasoning_basis_id.as_deref() != Some(context.basis_id.as_str())
        || model_binding.source_worker_job_id != native.source_worker_job_id
    {
        return Err(anyhow!(
            "decision context terminal request has foreign runtime ownership"
        ));
    }
    for observation in observations {
        let intent = cache
            .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(
                &observation.intent.intent_id,
            ))?
            .ok_or_else(|| anyhow!("decision context tool intent is absent"))?;
        let receipt = cache
            .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(
                &observation.intent.intent_id,
            ))?
            .ok_or_else(|| anyhow!("decision context tool receipt is absent"))?;
        let tool_binding = cache
            .get::<crate::EpiphanyRuntimeToolExecutionBinding>(&observation.intent.intent_id)?
            .ok_or_else(|| anyhow!("decision context tool binding is absent"))?;
        if intent != observation.intent
            || receipt != observation.receipt
            || !matches!(receipt.status.as_str(), "completed" | "failed")
            || tool_binding.intent_id != intent.intent_id
            || tool_binding.model_request_id != intent.model_request_id
        {
            return Err(anyhow!(
                "decision context tool observation has foreign stored ownership"
            ));
        }
        if let Some(request_id) = intent.model_request_id.as_deref() {
            let continuation = cache
                .get::<crate::EpiphanyRuntimeModelExecutionBinding>(request_id)?
                .ok_or_else(|| anyhow!("decision tool continuation request is absent"))?;
            if continuation.session_id != model_binding.session_id
                || continuation.source_worker_job_id != model_binding.source_worker_job_id
                || continuation.reasoning_basis_id != model_binding.reasoning_basis_id
                || tool_binding.session_id != continuation.session_id
                || tool_binding.job_id != continuation.job_id
            {
                return Err(anyhow!(
                    "decision tool continuation belongs to another reasoning pass"
                ));
            }
        } else if tool_binding.job_id != native.source_worker_job_id.as_deref().unwrap_or_default()
        {
            return Err(anyhow!(
                "request-owned tool observation belongs to another worker"
            ));
        }
    }
    Ok(())
}

pub fn commit_mind_mutation(
    store_path: &Path,
    decision_context_id: &str,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    require_non_empty(decision_context_id, "Mind mutation decision context id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let context = cache
        .get::<EpiphanyDecisionContext>(decision_context_id)?
        .ok_or_else(|| anyhow!("Mind mutation decision context does not exist"))?;
    let basis = cache
        .get::<EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("Mind mutation decision context lost its basis"))?;
    context.validate(&basis)?;
    let authority = EpiphanyMindCommitAuthority::ModelDecisionContext {
        decision_context_id: decision_context_id.to_string(),
    };
    commit_authorized_mind_mutation(
        store_path,
        authority,
        invariant_owner,
        strong_reads,
        writes,
        Vec::new(),
        committed_at,
    )
}

pub fn commit_operator_mind_mutation(
    store_path: &Path,
    provenance: CultCacheEnvelope,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    commit_operator_mind_mutation_with_derived_companions(
        store_path,
        provenance,
        invariant_owner,
        strong_reads,
        writes,
        Vec::new(),
        committed_at,
    )
}

pub(crate) fn commit_operator_mind_mutation_with_derived_companions(
    store_path: &Path,
    provenance: CultCacheEnvelope,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    derived_companions: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    let provenance_version =
        EpiphanyMindDocumentVersion::from_envelope("epiphany-operator", &provenance)?;
    let authority = EpiphanyMindCommitAuthority::OperatorProvenance {
        provenance: provenance_version,
    };
    let mut companions = Vec::with_capacity(derived_companions.len() + 1);
    companions.push(provenance);
    companions.extend(derived_companions);
    commit_authorized_mind_mutation(
        store_path,
        authority,
        invariant_owner,
        strong_reads,
        writes,
        companions,
        committed_at,
    )
}

pub fn commit_typed_organ_mind_mutation(
    store_path: &Path,
    organ: &str,
    provenance: CultCacheEnvelope,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    require_non_empty(organ, "Mind mutation organ")?;
    let provenance_version =
        EpiphanyMindDocumentVersion::from_envelope("epiphany-organ", &provenance)?;
    commit_authorized_mind_mutation(
        store_path,
        EpiphanyMindCommitAuthority::TypedOrganProvenance {
            organ: organ.to_string(),
            provenance: provenance_version,
        },
        invariant_owner,
        strong_reads,
        writes,
        vec![provenance],
        committed_at,
    )
}

fn commit_authorized_mind_mutation(
    store_path: &Path,
    authority: EpiphanyMindCommitAuthority,
    invariant_owner: &str,
    strong_reads: Vec<CultCacheEnvelope>,
    writes: Vec<CultCacheEnvelope>,
    companions: Vec<CultCacheEnvelope>,
    committed_at: &str,
) -> Result<EpiphanyMindCommitOutcome> {
    require_non_empty(invariant_owner, "Mind mutation invariant owner")?;
    require_non_empty(committed_at, "Mind mutation commit time")?;
    chrono::DateTime::parse_from_rfc3339(committed_at)
        .map_err(|error| anyhow!("Mind mutation commit time is invalid: {error}"))?;
    if writes.is_empty() {
        return Err(anyhow!("Mind mutation requires at least one write"));
    }
    for write in &writes {
        crate::mind_documents::validate_mind_write_envelope(write)?;
    }
    validate_unique_envelope_identities(&strong_reads, "strong read")?;
    validate_unique_envelope_identities(&writes, "write")?;
    validate_unique_envelope_identities(&companions, "companion")?;
    let expected_ids = strong_reads
        .iter()
        .map(|entry| (entry.r#type.as_str(), entry.key.as_str()))
        .collect::<BTreeSet<_>>();
    let write_ids = writes
        .iter()
        .map(|entry| (entry.r#type.clone(), entry.key.clone()))
        .collect::<BTreeSet<_>>();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut companion_expected = Vec::new();
    let mut companion_replacements = Vec::new();
    for companion in companions {
        if let Some(existing) = cache
            .snapshot_envelopes()
            .into_iter()
            .find(|entry| entry.r#type == companion.r#type && entry.key == companion.key)
        {
            if existing != companion {
                return Err(anyhow!("Mind mutation companion identity collision"));
            }
            companion_expected.push(existing.clone());
            companion_replacements.push(existing);
        } else {
            companion_replacements.push(companion);
        }
    }

    let strong_versions = strong_reads
        .iter()
        .map(|entry| EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", entry))
        .collect::<Result<Vec<_>>>()?;
    let write_versions = writes
        .iter()
        .map(|entry| EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", entry))
        .collect::<Result<Vec<_>>>()?;
    let receipt_id = mind_commit_receipt_id(
        &authority,
        invariant_owner,
        &strong_versions,
        &write_versions,
    )?;
    let receipt = EpiphanyMindCommitReceipt {
        schema_version: MIND_COMMIT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: receipt_id.clone(),
        authority,
        invariant_owner: invariant_owner.to_string(),
        strong_reads: strong_versions,
        writes: write_versions,
        committed_at: committed_at.to_string(),
    };
    if let Some(existing) = cache.get::<EpiphanyMindCommitReceipt>(&receipt_id)? {
        if existing != receipt {
            return Err(anyhow!("Mind commit receipt identity collision"));
        }
        return Ok(EpiphanyMindCommitOutcome::Committed(existing));
    }
    let mut replacements = writes;
    replacements.extend(
        strong_reads
            .iter()
            .filter(|entry| !write_ids.contains(&(entry.r#type.clone(), entry.key.clone())))
            .cloned(),
    );
    replacements.extend(companion_replacements);
    replacements.push(cache.prepare_entry(&receipt_id, &receipt)?.0);
    let mut expected = strong_reads.clone();
    expected.extend(companion_expected);
    if runtime_spine_backing_store(store_path)?.compare_and_swap_batch(&expected, replacements)? {
        return Ok(EpiphanyMindCommitOutcome::Committed(receipt));
    }
    let current = runtime_spine_backing_store(store_path)?.pull_all()?;
    let mut conflicts = strong_reads
        .iter()
        .filter(|expected| {
            current
                .iter()
                .find(|entry| entry.r#type == expected.r#type && entry.key == expected.key)
                != Some(*expected)
        })
        .map(|entry| (entry.r#type.clone(), entry.key.clone()))
        .collect::<Vec<_>>();
    if conflicts.is_empty() {
        conflicts = current_write_collisions(&current, &receipt.writes, &expected_ids);
    }
    conflicts.sort();
    conflicts.dedup();
    Ok(EpiphanyMindCommitOutcome::Conflict {
        document_identities: conflicts,
    })
}

fn validate_request_pair(
    basis: &EpiphanyReasoningBasis,
    native: &EpiphanyModelRequest,
    provider: &EpiphanyOpenAiModelRequest,
) -> Result<()> {
    if native.reasoning_basis_id.as_deref() != Some(basis.basis_id.as_str()) {
        return Err(anyhow!("model request does not bind its reasoning basis"));
    }
    if provider != &epiphany_openai_adapter::request_from_native(native) {
        return Err(anyhow!("native and provider terminal requests diverge"));
    }
    Ok(())
}

fn validate_tool_observations(
    request: &EpiphanyModelRequest,
    observations: &[EpiphanyDecisionToolObservation],
) -> Result<()> {
    let mut observed = BTreeSet::new();
    for observation in observations {
        if observation.intent.intent_id != observation.receipt.intent_id
            || observation.intent.adapter != observation.receipt.adapter
            || observation.intent.server != observation.receipt.server
            || observation.intent.tool_name != observation.receipt.tool_name
            || observation
                .intent
                .call_id
                .as_deref()
                .unwrap_or_default()
                .is_empty()
            || observation.intent.schema_id
                != epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID
            || observation.receipt.schema_id
                != epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID
            || observation.receipt.receipt_id.trim().is_empty()
            || observation.receipt.status.trim().is_empty()
        {
            return Err(anyhow!("decision tool observation ownership mismatch"));
        }
        if !observed.insert(observation.intent.intent_id.clone()) {
            return Err(anyhow!("decision context repeats a tool observation"));
        }
    }
    let mut request_observations = Vec::new();
    let mut index = 0usize;
    while index < request.input.len() {
        match &request.input[index] {
            epiphany_model_adapter::EpiphanyModelInputItem::ToolCall {
                call_id,
                name,
                arguments,
            } => {
                let Some(epiphany_model_adapter::EpiphanyModelInputItem::ToolResult {
                    call_id: result_call_id,
                    output,
                }) = request.input.get(index + 1)
                else {
                    return Err(anyhow!(
                        "terminal request tool call is not immediately paired with its result"
                    ));
                };
                if result_call_id != call_id {
                    return Err(anyhow!("terminal request tool result call id mismatch"));
                }
                request_observations.push((call_id, name, arguments, output));
                index += 2;
            }
            epiphany_model_adapter::EpiphanyModelInputItem::ToolResult { .. } => {
                return Err(anyhow!("terminal request contains an unpaired tool result"));
            }
            _ => index += 1,
        }
    }
    if request_observations.len() != observations.len() {
        return Err(anyhow!(
            "decision context tool observation count does not match terminal request"
        ));
    }
    for ((call_id, name, arguments, output), observation) in
        request_observations.into_iter().zip(observations)
    {
        let expected_name = format!(
            "mcp__{}__{}",
            observation.intent.server, observation.intent.tool_name
        );
        if observation.intent.call_id.as_deref() != Some(call_id.as_str())
            || name != &expected_name
            || arguments != &observation.intent.arguments_json
            || output != &receipt_output_for_model(&observation.intent, &observation.receipt)
        {
            return Err(anyhow!(
                "decision context tool observation does not match terminal request bytes"
            ));
        }
    }
    Ok(())
}

fn canonicalize_source_documents(documents: &mut Vec<EpiphanyMindDocumentVersion>) -> Result<()> {
    for document in documents.iter() {
        document.validate()?;
    }
    documents.sort_by(|left, right| {
        left.store_id
            .cmp(&right.store_id)
            .then(left.document_type.cmp(&right.document_type))
            .then(left.document_key.cmp(&right.document_key))
    });
    for pair in documents.windows(2) {
        if pair[0].store_id == pair[1].store_id
            && pair[0].document_type == pair[1].document_type
            && pair[0].document_key == pair[1].document_key
        {
            return Err(anyhow!(
                "reasoning basis repeats a source document identity"
            ));
        }
    }
    Ok(())
}

fn digest_without_basis_id(basis: &EpiphanyReasoningBasis) -> Result<String> {
    let mut canonical = basis.clone();
    canonical.basis_id.clear();
    Ok(sha256(&rmp_serde::to_vec_named(&canonical)?))
}

fn digest_without_context_id(context: &EpiphanyDecisionContext) -> Result<String> {
    let mut canonical = context.clone();
    canonical.context_id.clear();
    Ok(sha256(&rmp_serde::to_vec_named(&canonical)?))
}

fn mind_commit_receipt_id(
    authority: &EpiphanyMindCommitAuthority,
    owner: &str,
    strong_reads: &[EpiphanyMindDocumentVersion],
    writes: &[EpiphanyMindDocumentVersion],
) -> Result<String> {
    Ok(format!(
        "mind-commit-{}",
        sha256(&rmp_serde::to_vec_named(&(
            authority,
            owner,
            strong_reads,
            writes
        ))?)
    ))
}

fn validate_unique_envelope_identities(entries: &[CultCacheEnvelope], label: &str) -> Result<()> {
    let mut identities = BTreeSet::new();
    for entry in entries {
        if !identities.insert((entry.r#type.as_str(), entry.key.as_str())) {
            return Err(anyhow!("Mind mutation repeats {label} identity"));
        }
    }
    Ok(())
}

fn current_write_collisions(
    current: &[CultCacheEnvelope],
    writes: &[EpiphanyMindDocumentVersion],
    expected_ids: &BTreeSet<(&str, &str)>,
) -> Vec<(String, String)> {
    writes
        .iter()
        .filter(|write| {
            !expected_ids.contains(&(write.document_type.as_str(), write.document_key.as_str()))
                && current.iter().any(|entry| {
                    entry.r#type == write.document_type && entry.key == write.document_key
                })
        })
        .map(|write| (write.document_type.clone(), write.document_key.clone()))
        .collect()
}

fn require_non_empty(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} cannot be empty"));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EpiphanyRuntimeJobStatus, RuntimeSpineInitOptions, initialize_runtime_spine};
    use cultcache_rs::{CacheBackingStore, SingleFileMessagePackBackingStore};
    use epiphany_model_adapter::EpiphanyModelInputItem;
    use tempfile::tempdir;

    fn basis() -> Result<EpiphanyReasoningBasis> {
        let projection =
            EpiphanyReasoningProjection::RoleLaunch(EpiphanyRoleWorkerLaunchDocument {
                thread_id: "creation-thread".into(),
                role_id: "Modeling".into(),
                state_revision: 7,
                objective: Some("Map the Body".into()),
                dynamic_prompt_context: Some("typed projection".into()),
                repository_body_observation_basis: None,
                proposal_modeling_context: None,
                claim_repair_context: None,
                frontier_planning_context: None,
                frontier_research_context: None,
                frontier_plan_mind_context: None,
                imagination_consideration_context: None,
                admitted_model_direction_consideration_context: None,
                active_subgoal_id: None,
                active_subgoals: Vec::new(),
                active_graph_node_ids: Vec::new(),
                investigation_checkpoint: None,
                scratch: None,
                invariants: Vec::new(),
                graphs: None,
                recent_evidence: Vec::new(),
                recent_observations: Vec::new(),
                graph_frontier: None,
                graph_checkpoint: None,
                planning: None,
                churn: None,
            });
        EpiphanyReasoningBasis::new(
            "pass-1",
            "Modeling",
            WORKER_REASONING_PROJECTION_POLICY,
            Vec::new(),
            projection,
        )
    }

    fn requests(
        basis: &EpiphanyReasoningBasis,
    ) -> (EpiphanyModelRequest, EpiphanyOpenAiModelRequest) {
        let mut native = EpiphanyModelRequest::new(
            "request-1",
            "conversation-1",
            "openai-codex",
            "gpt-test",
            "inspect",
        );
        native.reasoning_basis_id = Some(basis.basis_id.clone());
        native.input.push(EpiphanyModelInputItem::UserText {
            text: "projection".into(),
        });
        let provider = EpiphanyOpenAiModelRequest {
            schema_id: epiphany_openai_adapter::OPENAI_ADAPTER_REQUEST_SCHEMA_ID.into(),
            request_id: native.request_id.clone(),
            conversation_id: native.conversation_id.clone(),
            model: native.model.clone(),
            instructions: native.instructions.clone(),
            input: vec![epiphany_openai_adapter::EpiphanyOpenAiInputItem::UserText {
                text: "projection".into(),
            }],
            reasoning_effort: None,
            reasoning_summary: None,
            service_tier: None,
            output_contract_id: None,
            previous_response_id: None,
            tools: Vec::new(),
            output_schema_json: None,
        };
        (native, provider)
    }

    #[test]
    fn basis_and_context_are_content_addressed_and_reject_substitution() -> Result<()> {
        let reasoning_basis = basis()?;
        assert_eq!(reasoning_basis, basis()?);
        let (native, provider) = requests(&reasoning_basis);
        let context =
            EpiphanyDecisionContext::new(&reasoning_basis, native.clone(), provider, Vec::new())?;
        context.validate(&reasoning_basis)?;
        let mut substituted = context.clone();
        substituted.native_request_msgpack.push(0xff);
        assert!(substituted.validate(&reasoning_basis).is_err());
        Ok(())
    }

    #[test]
    fn disjoint_mind_mutations_merge_and_same_identity_conflicts() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("mind.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "mind-test".into(),
                display_name: "Mind test".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;
        let basis = put_reasoning_basis(&store, &basis()?)?;
        let (native, provider) = requests(&basis);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineSessionOptions {
                session_id: "session-1".into(),
                objective: "Test sealed decision context".into(),
                created_at: "2026-08-14T00:00:01Z".into(),
                coordinator_note: "reasoning context test".into(),
            },
            crate::RuntimeSpineJobOptions {
                job_id: "model-job-1".into(),
                session_id: "session-1".into(),
                role: "model-adapter".into(),
                created_at: "2026-08-14T00:00:01Z".into(),
                summary: "Test terminal request publication".into(),
                artifact_refs: Vec::new(),
            },
            &native,
            &provider,
            "2026-08-14T00:00:01Z",
        )?;
        let context = put_decision_context(
            &store,
            &EpiphanyDecisionContext::new(&basis, native, provider, Vec::new())?,
        )?;
        let backing = SingleFileMessagePackBackingStore::new(&store);
        let make = |key: &str, summary: &str| -> Result<CultCacheEnvelope> {
            let cache = runtime_spine_cache(&store)?;
            let document = crate::EpiphanyMindObservationDocument {
                value: epiphany_state_model::EpiphanyObservation {
                    id: key.into(),
                    summary: summary.into(),
                    source_kind: "test".into(),
                    status: "accepted".into(),
                    code_refs: Vec::new(),
                    evidence_ids: Vec::new(),
                },
            };
            Ok(cache.prepare_entry(key, &document)?.0)
        };
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                Vec::new(),
                vec![make("persona", "one")?],
                "2026-08-14T00:00:02Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                Vec::new(),
                vec![make("hands", "two")?],
                "2026-08-14T00:00:03Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        let current = backing
            .pull_all()?
            .into_iter()
            .find(|entry| {
                entry.r#type == crate::EpiphanyMindObservationDocument::TYPE
                    && entry.key == "persona"
            })
            .unwrap();
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![current.clone()],
                vec![make("persona", "winner")?],
                "2026-08-14T00:00:04Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![current],
                vec![make("persona", "loser")?],
                "2026-08-14T00:00:05Z"
            )?,
            EpiphanyMindCommitOutcome::Conflict { .. }
        ));
        let hands = backing
            .pull_all()?
            .into_iter()
            .find(|entry| {
                entry.r#type == crate::EpiphanyMindObservationDocument::TYPE && entry.key == "hands"
            })
            .unwrap();
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![hands.clone()],
                vec![make("modeling", "read-only dependency")?],
                "2026-08-14T00:00:06Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![hands.clone()],
                vec![make("hands", "dependency changed")?],
                "2026-08-14T00:00:07Z"
            )?,
            EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert!(matches!(
            commit_mind_mutation(
                &store,
                &context.context_id,
                "test-owner",
                vec![hands],
                vec![make("verification", "must not partially appear")?],
                "2026-08-14T00:00:08Z"
            )?,
            EpiphanyMindCommitOutcome::Conflict { .. }
        ));
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert!(
            cache
                .get::<crate::EpiphanyMindObservationDocument>("verification")?
                .is_none()
        );
        let _ = EpiphanyRuntimeJobStatus::Completed;
        Ok(())
    }
}
