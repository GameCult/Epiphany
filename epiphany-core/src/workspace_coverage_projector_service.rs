use crate::repository_body_observer::RepositoryBodyReadSession;
use crate::semantic_backend::{OllamaConfig, OllamaEmbedder, QdrantBackend, QdrantConfig};
use crate::workspace_coverage_process_documents::publish_workspace_coverage_terminal_sight;
use crate::workspace_coverage_projector::{
    WorkspaceCoverageAcquireResult, WorkspaceCoverageCurrentState,
    WorkspaceCoverageProjectionExecutionAuthority, acquire_workspace_coverage_projection,
    classify_current_workspace_coverage, compact_workspace_coverage_history,
    execute_workspace_coverage_projection, prepare_workspace_coverage_projection,
    retire_workspace_coverage_collections,
};
use crate::{
    EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID, HostIncarnationIdentityEntry,
    WorkspaceCoverageAuthority, load_current_runtime_repository_body_basis,
    open_workspace_coverage_authority, runtime_repository_body_store_binding,
};
use anyhow::{Context, Result, anyhow, bail};
use ed25519_dalek::SigningKey;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const EMBEDDING_PROVIDER_ID: &str = "gamecult-ollama-embedding";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCoverageProjectorConfig {
    pub qdrant_url: String,
    pub qdrant_api_key: Option<String>,
    pub qdrant_timeout_ms: u64,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub ollama_timeout_ms: u64,
}

impl WorkspaceCoverageProjectorConfig {
    pub fn from_env() -> Self {
        let semantic = crate::MemorySemanticIndexConfig::from_env();
        Self {
            qdrant_url: semantic.qdrant_url,
            qdrant_api_key: semantic.qdrant_api_key,
            qdrant_timeout_ms: semantic.qdrant_timeout_ms,
            ollama_base_url: semantic.ollama_base_url,
            ollama_model: semantic.ollama_model,
            ollama_timeout_ms: semantic.ollama_timeout_ms,
        }
    }

    fn validate(&self) -> Result<()> {
        for (label, value) in [
            ("Qdrant URL", self.qdrant_url.as_str()),
            ("Ollama base URL", self.ollama_base_url.as_str()),
            ("Ollama model", self.ollama_model.as_str()),
        ] {
            if value.trim().is_empty() {
                bail!("workspace coverage projector requires a non-empty {label}");
            }
        }
        if self.qdrant_timeout_ms == 0 || self.ollama_timeout_ms == 0 {
            bail!("workspace coverage projector timeouts must be positive");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceCoverageProjectorPulseStatus {
    Idle,
    Executed,
    Contended,
    Refused,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceCoverageProjectorServicePulse {
    pub status: WorkspaceCoverageProjectorPulseStatus,
    pub body_observation_id: Option<String>,
    pub body_generation: Option<u64>,
    pub plan_id: Option<String>,
    pub receipt_id: Option<String>,
    pub fault: Option<String>,
}

pub struct WorkspaceCoverageProjectorServiceBody {
    coverage_authority: WorkspaceCoverageAuthority,
    runtime_store: PathBuf,
    executor_incarnation: String,
    managed_process_launch_id: String,
    embedder: OllamaEmbedder,
    qdrant: QdrantBackend,
    execution_authority: Option<WorkspaceCoverageProjectorExecutionAuthority>,
    operation_timeout_ms: u64,
}

#[derive(Clone)]
struct WorkspaceCoverageProjectorExecutionAuthority {
    local_verse_store: PathBuf,
    runtime_id: String,
    trusted_host: HostIncarnationIdentityEntry,
    provider_signing_key: SigningKey,
}

impl WorkspaceCoverageProjectorServiceBody {
    pub fn new(
        runtime_store: impl AsRef<Path>,
        runtime_id: impl Into<String>,
        config: WorkspaceCoverageProjectorConfig,
        executor_incarnation: impl Into<String>,
        managed_process_launch_id: impl Into<String>,
    ) -> Result<Self> {
        config.validate()?;
        let executor_incarnation = executor_incarnation.into();
        if executor_incarnation.trim().is_empty() {
            bail!("workspace coverage projector requires its executor incarnation");
        }
        let managed_process_launch_id = managed_process_launch_id.into();
        Uuid::parse_str(&managed_process_launch_id).context(
            "workspace coverage projector requires its authenticated managed process launch id",
        )?;
        let runtime_id = runtime_id.into();
        if runtime_id.trim().is_empty() {
            bail!("workspace coverage projector requires its CultMesh runtime id");
        }
        let runtime_store = runtime_store
            .as_ref()
            .canonicalize()
            .context("workspace coverage projector runtime store is missing")?;
        if !runtime_store.is_file() {
            bail!("workspace coverage projector runtime store is not a file");
        }
        let route = runtime_repository_body_store_binding(&runtime_store)?
            .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
        if route.runtime_id != runtime_id {
            bail!(
                "workspace coverage projector runtime Body route disagrees with authenticated runtime id"
            );
        }
        let coverage_authority = open_workspace_coverage_authority(&runtime_store)?;
        let operation_timeout_ms = config.qdrant_timeout_ms;
        let qdrant = QdrantBackend::new(QdrantConfig {
            url: config.qdrant_url,
            api_key: config.qdrant_api_key,
            timeout_ms: config.qdrant_timeout_ms,
        })?;
        let embedder = OllamaEmbedder::new(OllamaConfig {
            base_url: config.ollama_base_url,
            model: config.ollama_model,
            timeout_ms: config.ollama_timeout_ms,
            query_instruction: String::new(),
        })?;
        Ok(Self {
            coverage_authority,
            runtime_store,
            executor_incarnation,
            managed_process_launch_id,
            embedder,
            qdrant,
            execution_authority: None,
            operation_timeout_ms,
        })
    }

    pub fn install_execution_authority(
        &mut self,
        local_verse_store: impl AsRef<Path>,
        runtime_id: impl Into<String>,
        trusted_host: HostIncarnationIdentityEntry,
        provider_signing_key: SigningKey,
    ) -> Result<()> {
        let runtime_id = runtime_id.into();
        if runtime_id.trim().is_empty() {
            bail!("workspace coverage execution authority requires its runtime id");
        }
        let bound = runtime_repository_body_store_binding(&self.runtime_store)?
            .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
        if bound.runtime_id != runtime_id {
            bail!("workspace coverage execution authority runtime disagrees with Body route");
        }
        self.execution_authority = Some(WorkspaceCoverageProjectorExecutionAuthority {
            local_verse_store: local_verse_store.as_ref().to_path_buf(),
            runtime_id,
            trusted_host,
            provider_signing_key,
        });
        Ok(())
    }

    pub fn provider_incarnation(&self) -> &str {
        &self.executor_incarnation
    }

    pub fn managed_process_launch_id(&self) -> &str {
        &self.managed_process_launch_id
    }

    pub fn pulse(&mut self) -> WorkspaceCoverageProjectorServicePulse {
        match self.pulse_inner() {
            Ok(pulse) => pulse,
            Err(_error) => {
                eprintln!("workspace coverage pulse failed: {_error:#}");
                #[cfg(feature = "workspace-coverage-recovery-smoke")]
                if std::env::var_os("EPIPHANY_WORKSPACE_COVERAGE_SMOKE_DIAGNOSTICS").is_some() {
                    eprintln!("workspace coverage smoke diagnostic: {_error:#}");
                }
                WorkspaceCoverageProjectorServicePulse {
                    status: WorkspaceCoverageProjectorPulseStatus::Refused,
                    body_observation_id: None,
                    body_generation: None,
                    plan_id: None,
                    receipt_id: None,
                    // Managed stdout is an operator-safe projection. Raw backend
                    // bodies and filesystem paths remain private to the process.
                    fault: Some("workspace_coverage_pulse_refused".into()),
                }
            }
        }
    }

    fn pulse_inner(&mut self) -> Result<WorkspaceCoverageProjectorServicePulse> {
        // Retirement is derived from typed Body history and is deliberately
        // performed before the idle fast path. Qdrant never nominates its own
        // garbage and incompatible same-name collections stop the pulse.
        retire_workspace_coverage_collections(&self.coverage_authority, &mut self.qdrant)?;
        let _ = compact_workspace_coverage_history(&self.runtime_store, &self.coverage_authority)?;
        let basis = load_current_runtime_repository_body_basis(&self.runtime_store)?;
        // Tags are mutable. Re-resolve the installed artifact every pulse so a
        // same-dimensional model replacement cannot reuse an older index.
        let embedding_model_identity = self.embedder.model_artifact()?.canonical_identity();
        let dimensions = self.embedder.embedding_dimensions()?;
        if let WorkspaceCoverageCurrentState::Current(receipt) =
            classify_current_workspace_coverage(
                &self.runtime_store,
                &self.coverage_authority.store,
                &basis,
                EMBEDDING_PROVIDER_ID,
                &embedding_model_identity,
                dimensions,
            )?
        {
            let authority = self.execution_authority.as_ref().ok_or_else(|| {
                anyhow!("workspace coverage projector has no installed execution authority")
            })?;
            publish_workspace_coverage_terminal_sight(
                &authority.local_verse_store,
                &self.runtime_store,
                &self.coverage_authority,
                &authority.runtime_id,
                &self.managed_process_launch_id,
                &authority.trusted_host,
                &authority.provider_signing_key,
                chrono::Utc::now(),
            )?;
            return Ok(WorkspaceCoverageProjectorServicePulse {
                status: WorkspaceCoverageProjectorPulseStatus::Idle,
                body_observation_id: Some(basis.observation_id),
                body_generation: Some(basis.generation),
                plan_id: Some(receipt.plan_id.clone()),
                receipt_id: Some(receipt.receipt_id),
                fault: None,
            });
        }
        let body = RepositoryBodyReadSession::open(&self.runtime_store, &basis)?;
        let prepared = prepare_workspace_coverage_projection(
            &body,
            EMBEDDING_PROVIDER_ID,
            &embedding_model_identity,
            dimensions,
        )?;
        let plan_id = prepared.plan.plan_id.clone();
        match acquire_workspace_coverage_projection(
            &prepared,
            &self.coverage_authority.store,
            EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID,
            &self.executor_incarnation,
            &self.managed_process_launch_id,
        )? {
            WorkspaceCoverageAcquireResult::Contended => {
                Ok(WorkspaceCoverageProjectorServicePulse {
                    status: WorkspaceCoverageProjectorPulseStatus::Contended,
                    body_observation_id: Some(basis.observation_id),
                    body_generation: Some(basis.generation),
                    plan_id: Some(plan_id),
                    receipt_id: None,
                    fault: None,
                })
            }
            WorkspaceCoverageAcquireResult::Current(receipt) => {
                let authority = self.execution_authority.as_ref().ok_or_else(|| {
                    anyhow!("workspace coverage projector has no installed execution authority")
                })?;
                publish_workspace_coverage_terminal_sight(
                    &authority.local_verse_store,
                    &self.runtime_store,
                    &self.coverage_authority,
                    &authority.runtime_id,
                    &self.managed_process_launch_id,
                    &authority.trusted_host,
                    &authority.provider_signing_key,
                    chrono::Utc::now(),
                )?;
                Ok(WorkspaceCoverageProjectorServicePulse {
                    status: WorkspaceCoverageProjectorPulseStatus::Idle,
                    body_observation_id: Some(basis.observation_id),
                    body_generation: Some(basis.generation),
                    plan_id: Some(plan_id),
                    receipt_id: Some(receipt.receipt_id),
                    fault: None,
                })
            }
            WorkspaceCoverageAcquireResult::Acquired(acquisition) => {
                let authority = self.execution_authority.as_ref().ok_or_else(|| {
                    anyhow!("workspace coverage projector has no installed execution authority")
                })?;
                let receipt = execute_workspace_coverage_projection(
                    &acquisition,
                    &prepared,
                    &self.embedder,
                    &mut self.qdrant,
                    &WorkspaceCoverageProjectionExecutionAuthority {
                        coverage_authority: &self.coverage_authority,
                        runtime_store: &self.runtime_store,
                        local_verse_store: &authority.local_verse_store,
                        runtime_id: &authority.runtime_id,
                        trusted_host: &authority.trusted_host,
                        provider_signing_key: &authority.provider_signing_key,
                        operation_timeout_ms: self.operation_timeout_ms,
                    },
                )?;
                Ok(WorkspaceCoverageProjectorServicePulse {
                    status: WorkspaceCoverageProjectorPulseStatus::Executed,
                    body_observation_id: Some(basis.observation_id),
                    body_generation: Some(basis.generation),
                    plan_id: Some(plan_id),
                    receipt_id: Some(receipt.receipt_id),
                    fault: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> WorkspaceCoverageProjectorConfig {
        WorkspaceCoverageProjectorConfig {
            qdrant_url: "http://127.0.0.1:6333".into(),
            qdrant_api_key: None,
            qdrant_timeout_ms: 1,
            ollama_base_url: "http://127.0.0.1:11434".into(),
            ollama_model: "test-model".into(),
            ollama_timeout_ms: 1,
        }
    }

    #[test]
    fn config_refuses_empty_endpoints_and_zero_timeouts() {
        let mut value = config();
        value.ollama_model.clear();
        assert!(value.validate().is_err());
        let mut value = config();
        value.qdrant_timeout_ms = 0;
        assert!(value.validate().is_err());
    }

    #[test]
    fn constructor_requires_lifecycle_identity_before_touching_runtime() {
        assert!(
            WorkspaceCoverageProjectorServiceBody::new(
                "missing",
                "runtime",
                config(),
                "",
                "not-a-uuid"
            )
            .is_err()
        );
        assert!(
            WorkspaceCoverageProjectorServiceBody::new(
                "missing",
                "runtime",
                config(),
                "incarnation",
                "not-a-uuid",
            )
            .is_err()
        );
    }

    #[test]
    fn live_service_exclusively_owns_the_bound_coverage_store() -> Result<()> {
        let state = tempfile::tempdir()?;
        let repo = tempfile::tempdir()?;
        let initialized = std::process::Command::new("git")
            .arg("init")
            .current_dir(repo.path())
            .output()?;
        if !initialized.status.success() {
            bail!("singleton fixture failed to initialize repository");
        }
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body = state.path().join("body.cc");
        let coverage = std::fs::canonicalize(state.path())?.join("workspace-coverage.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "singleton-runtime".into(),
                display_name: "singleton test".into(),
                created_at: "2026-07-17T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "singleton-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-17T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body, &runtime, "singleton-workspace")?;
        crate::bind_runtime_workspace_coverage_store(&runtime, &coverage, "2026-07-17T00:00:02Z")?;
        let launch_id = "019bff10-7426-7a1c-9ce4-33f91660a3a7";
        let first = WorkspaceCoverageProjectorServiceBody::new(
            &runtime,
            "singleton-runtime",
            config(),
            "first-incarnation",
            launch_id,
        )?;
        assert!(
            WorkspaceCoverageProjectorServiceBody::new(
                &runtime,
                "singleton-runtime",
                config(),
                "second-incarnation",
                launch_id,
            )
            .is_err(),
            "a second live projector must not acquire the same owned store"
        );
        drop(first);
        WorkspaceCoverageProjectorServiceBody::new(
            &runtime,
            "singleton-runtime",
            config(),
            "replacement-incarnation",
            launch_id,
        )?;
        Ok(())
    }
}
