use crate::memory_graph::semantic_projector::{
    classify_memory_semantic_projection_for_pulse, execute_memory_semantic_projection,
    idunn_acquire_memory_semantic_projection_with_config,
    owned_running_memory_semantic_projection_claim,
};
use crate::memory_graph::{
    MemorySemanticProjectorPulseClassification, MemorySemanticProjectorPulseOutcome,
    MemorySemanticProjectorPulsePort, MemorySemanticProjectorPulser,
};
use crate::{
    MemorySemanticIndexConfig, MemorySemanticProjectionInput,
    runtime_modeling_semantic_projection_input,
};
use anyhow::{Result, anyhow};
use chrono::{SecondsFormat, Utc};
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

/// Trusted in-process authority held by the one Idunn-managed projector body.
/// The constructor generates process-stable incarnations; callers cannot label
/// themselves as an existing Idunn or executor incarnation.
const SEMANTIC_PROJECTOR_EXECUTOR_ID: &str = "epiphany-memory-semantic-projector";

struct LocalIdunnSemanticProjectorSession {
    modeling_store: PathBuf,
    executor_id: String,
    executor_incarnation: String,
    idunn_incarnation: String,
    config: MemorySemanticIndexConfig,
}

struct SemanticProjectorSingletonGuard {
    #[cfg(windows)]
    handle: HANDLE,
    #[cfg(unix)]
    file: File,
}

impl Drop for SemanticProjectorSingletonGuard {
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

fn acquire_semantic_projector_singleton(
    modeling_store: impl AsRef<Path>,
) -> Result<SemanticProjectorSingletonGuard> {
    let modeling = canonical_file(&modeling_store, "Modeling")?;
    let identity = format!(
        "{:x}",
        Sha256::digest(modeling.to_string_lossy().to_lowercase().as_bytes())
    );
    #[cfg(windows)]
    {
        let name = OsStr::new(&format!("Global\\EpiphanySemanticProjector-{identity}"))
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr()) };
        if handle.is_null() {
            return Err(anyhow!(
                "semantic projector singleton mutex creation failed: {}",
                unsafe { GetLastError() }
            ));
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            return Err(anyhow!(
                "semantic projector owner already exists for canonical store pair"
            ));
        }
        return Ok(SemanticProjectorSingletonGuard { handle });
    }
    #[cfg(unix)]
    {
        let path = std::env::temp_dir().join(format!("epiphany-semantic-{identity}.lock"));
        let file = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(anyhow!(
                "semantic projector owner already exists for canonical store pair"
            ));
        }
        return Ok(SemanticProjectorSingletonGuard { file });
    }
    #[allow(unreachable_code)]
    Err(anyhow!(
        "semantic projector singleton is unsupported on this platform"
    ))
}

impl LocalIdunnSemanticProjectorSession {
    fn new(
        modeling_store: impl AsRef<Path>,
        executor_id: impl Into<String>,
        config: MemorySemanticIndexConfig,
    ) -> Result<Self> {
        let modeling_store = canonical_file(&modeling_store, "Modeling")?;
        let executor_id = executor_id.into();
        if executor_id.trim().is_empty() {
            return Err(anyhow!("semantic projector executor id must not be empty"));
        }
        Ok(Self {
            modeling_store,
            executor_id,
            executor_incarnation: format!("projector-{}", Uuid::new_v4()),
            idunn_incarnation: format!("idunn-{}", Uuid::new_v4()),
            config,
        })
    }

    pub fn provider_incarnation(&self) -> &str {
        &self.executor_incarnation
    }

    fn store_for(&self, input: &MemorySemanticProjectionInput) -> Result<&Path> {
        match input.obligation().partition.as_str() {
            "modeling" => Ok(&self.modeling_store),
            other => Err(anyhow!(
                "unsupported semantic projector partition {other:?}"
            )),
        }
    }
}

/// The only public authority-bearing semantic projector body. Construction
/// atomically takes the host-wide singleton before minting a process session,
/// and the guard remains owned for the body's full lifetime.
pub struct SemanticProjectorServiceBody {
    _singleton: SemanticProjectorSingletonGuard,
    pulser: MemorySemanticProjectorPulser<LocalIdunnSemanticProjectorSession>,
    provider_incarnation: String,
}

pub struct SemanticProjectorServicePulse {
    pub outcome: MemorySemanticProjectorPulseOutcome,
    pub inputs: Vec<MemorySemanticProjectionInput>,
    pub source_fault_count: u32,
}

impl SemanticProjectorServiceBody {
    pub fn new(
        modeling_store: impl AsRef<Path>,
        config: MemorySemanticIndexConfig,
    ) -> Result<Self> {
        let singleton = acquire_semantic_projector_singleton(&modeling_store)?;
        let session = LocalIdunnSemanticProjectorSession::new(
            modeling_store,
            SEMANTIC_PROJECTOR_EXECUTOR_ID,
            config,
        )?;
        let provider_incarnation = session.provider_incarnation().to_string();
        Ok(Self {
            _singleton: singleton,
            pulser: MemorySemanticProjectorPulser::new(session),
            provider_incarnation,
        })
    }

    pub fn provider_incarnation(&self) -> &str {
        &self.provider_incarnation
    }

    pub fn pulse(&self) -> SemanticProjectorServicePulse {
        let session = &self.pulser.port;
        let (inputs, source_fault_count) =
            match runtime_modeling_semantic_projection_input(&session.modeling_store) {
                Ok(input) => (vec![input], 0),
                Err(_) => (Vec::new(), 1),
            };
        let outcome = inputs.first().map_or_else(
            || MemorySemanticProjectorPulseOutcome {
                status: crate::MemorySemanticProjectorPulseStatus::Idle,
                selected_scope_id: None,
                inspections: Vec::new(),
                error: None,
            },
            |input| self.pulser.pulse(input),
        );
        SemanticProjectorServicePulse {
            outcome,
            inputs,
            source_fault_count,
        }
    }
}

impl MemorySemanticProjectorPulsePort for LocalIdunnSemanticProjectorSession {
    fn classify(
        &self,
        input: &MemorySemanticProjectionInput,
    ) -> Result<MemorySemanticProjectorPulseClassification> {
        let store = self.store_for(input)?;
        let classification = classify_memory_semantic_projection_for_pulse(store, input)?;
        if classification == MemorySemanticProjectorPulseClassification::Running {
            if let Some(claim_id) = owned_running_memory_semantic_projection_claim(
                store,
                input,
                &self.executor_id,
                &self.executor_incarnation,
            )? {
                return Ok(MemorySemanticProjectorPulseClassification::RunningOwned { claim_id });
            }
        }
        Ok(classification)
    }

    fn acquire(
        &self,
        input: &MemorySemanticProjectionInput,
        purpose: &str,
    ) -> Result<Option<String>> {
        let store = self.store_for(input)?;
        match idunn_acquire_memory_semantic_projection_with_config(
            store,
            input,
            &self.executor_id,
            &self.executor_incarnation,
            purpose,
            &self.idunn_incarnation,
            &Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            &self.config,
        ) {
            Ok(acquisition) => Ok(Some(acquisition.claim.claim_id)),
            Err(_error)
                if classify_memory_semantic_projection_for_pulse(store, input)?
                    == MemorySemanticProjectorPulseClassification::Running =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    fn execute(&self, input: &MemorySemanticProjectionInput, claim_id: &str) -> Result<()> {
        execute_memory_semantic_projection(self.store_for(input)?, input, claim_id, &self.config)?;
        Ok(())
    }
}

fn canonical_file(path: &impl AsRef<Path>, label: &str) -> Result<PathBuf> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err(anyhow!(
            "{label} canonical store is missing: {}",
            path.display()
        ));
    }
    path.canonicalize()
        .map_err(|error| anyhow!("{label} canonical store identity is unavailable: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton_refuses_a_second_process_owner_identity() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let modeling = temp.path().join("modeling.ccmp");
        std::fs::write(&modeling, [])?;
        let first = acquire_semantic_projector_singleton(&modeling)?;
        assert!(acquire_semantic_projector_singleton(&modeling).is_err());
        drop(first);
        assert!(acquire_semantic_projector_singleton(&modeling).is_ok());
        Ok(())
    }

    #[test]
    fn authority_body_has_no_public_raw_session_or_session_local_windows_lock() {
        let source = include_str!("semantic_projector_service.rs");
        let public_session = ["pub struct Local", "IdunnSemanticProjectorSession"].concat();
        let public_singleton = ["pub fn acquire_semantic", "_projector_singleton"].concat();
        let local_namespace = ["Local", "\\\\EpiphanySemanticProjector"].concat();
        assert!(!source.contains(&public_session));
        assert!(!source.contains(&public_singleton));
        assert!(!source.contains(&local_namespace));
        assert!(source.contains("Global\\\\EpiphanySemanticProjector"));
        assert!(source.contains("pub struct SemanticProjectorServiceBody"));
    }
}
