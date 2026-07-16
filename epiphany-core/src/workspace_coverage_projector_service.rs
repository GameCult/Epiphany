use crate::repository_body_observer::RepositoryBodyReadSession;
use crate::semantic_backend::{OllamaConfig, OllamaEmbedder, QdrantBackend, QdrantConfig};
use crate::workspace_coverage_projector::{
    WorkspaceCoverageAcquireResult, WorkspaceCoverageCurrentState,
    acquire_workspace_coverage_projection, classify_current_workspace_coverage,
    execute_workspace_coverage_projection, prepare_workspace_coverage_projection,
    retire_workspace_coverage_collections,
};
use crate::{
    EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID, load_current_runtime_repository_body_basis,
    runtime_repository_body_store_binding,
};
use anyhow::{Context, Result, anyhow, bail};
use sha2::{Digest, Sha256};
#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex};

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

struct WorkspaceCoverageProjectorSingletonGuard {
    #[cfg(windows)]
    handle: HANDLE,
    #[cfg(unix)]
    file: File,
}

impl Drop for WorkspaceCoverageProjectorSingletonGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            let _ = ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
        #[cfg(unix)]
        unsafe {
            let _ = libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

fn acquire_workspace_coverage_projector_singleton(
    runtime_store: &Path,
    body_store: &Path,
) -> Result<WorkspaceCoverageProjectorSingletonGuard> {
    let identity = format!(
        "{:x}",
        Sha256::digest(
            format!(
                "{}|{}",
                runtime_store.to_string_lossy().to_lowercase(),
                body_store.to_string_lossy().to_lowercase()
            )
            .as_bytes()
        )
    );
    #[cfg(windows)]
    {
        let name = OsStr::new(&format!(
            "Global\\EpiphanyWorkspaceCoverageProjector-{identity}"
        ))
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
        let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr()) };
        if handle.is_null() {
            return Err(anyhow!(
                "workspace coverage projector singleton mutex creation failed: {}",
                unsafe { GetLastError() }
            ));
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            bail!("workspace coverage projector owner already exists for this runtime Body route");
        }
        return Ok(WorkspaceCoverageProjectorSingletonGuard { handle });
    }
    #[cfg(unix)]
    {
        let path =
            std::env::temp_dir().join(format!("epiphany-workspace-coverage-{identity}.lock"));
        let file = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            bail!("workspace coverage projector owner already exists for this runtime Body route");
        }
        return Ok(WorkspaceCoverageProjectorSingletonGuard { file });
    }
    #[allow(unreachable_code)]
    Err(anyhow!(
        "workspace coverage projector singleton is unsupported on this platform"
    ))
}

pub struct WorkspaceCoverageProjectorServiceBody {
    _singleton: WorkspaceCoverageProjectorSingletonGuard,
    runtime_store: PathBuf,
    executor_incarnation: String,
    managed_process_launch_id: String,
    embedder: OllamaEmbedder,
    qdrant: QdrantBackend,
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
        let body_store = PathBuf::from(route.body_store_path);
        let singleton =
            acquire_workspace_coverage_projector_singleton(&runtime_store, &body_store)?;
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
            _singleton: singleton,
            runtime_store,
            executor_incarnation,
            managed_process_launch_id,
            embedder,
            qdrant,
        })
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
        let route = runtime_repository_body_store_binding(&self.runtime_store)?
            .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
        // Retirement is derived from typed Body history and is deliberately
        // performed before the idle fast path. Qdrant never nominates its own
        // garbage and incompatible same-name collections stop the pulse.
        retire_workspace_coverage_collections(&route.body_store_path, &mut self.qdrant)?;
        let basis = load_current_runtime_repository_body_basis(&self.runtime_store)?;
        // Tags are mutable. Re-resolve the installed artifact every pulse so a
        // same-dimensional model replacement cannot reuse an older index.
        let embedding_model_identity = self.embedder.model_artifact()?.canonical_identity();
        let dimensions = self.embedder.embedding_dimensions()?;
        if let WorkspaceCoverageCurrentState::Current(receipt) =
            classify_current_workspace_coverage(
                &self.runtime_store,
                &basis,
                EMBEDDING_PROVIDER_ID,
                &embedding_model_identity,
                dimensions,
            )?
        {
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
                let receipt = execute_workspace_coverage_projection(
                    &acquisition,
                    &prepared,
                    &self.embedder,
                    &mut self.qdrant,
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
    fn singleton_is_host_wide_for_one_canonical_runtime_body_pair() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let runtime = temp.path().join("runtime.ccmp");
        let body = temp.path().join("body.ccmp");
        std::fs::write(&runtime, [])?;
        std::fs::write(&body, [])?;
        let runtime = runtime.canonicalize()?;
        let body = body.canonicalize()?;
        let first = acquire_workspace_coverage_projector_singleton(&runtime, &body)?;
        assert!(acquire_workspace_coverage_projector_singleton(&runtime, &body).is_err());
        drop(first);
        assert!(acquire_workspace_coverage_projector_singleton(&runtime, &body).is_ok());
        Ok(())
    }
}
