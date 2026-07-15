use crate::{
    BODY_BINDING_KEY, BODY_BINDING_TYPE, BODY_HEAD_KEY, BODY_HEAD_TYPE, BODY_MANIFEST_TYPE,
    BODY_OBSERVATION_TYPE, RepositoryBodyHead, RepositoryBodyObservationBasis,
    WorkspaceCoverageChunkDescriptor, WorkspaceCoverageObligation, WorkspaceCoveragePolicy,
    WorkspaceCoverageProjectionPlan, derive_workspace_coverage_obligation,
    derive_workspace_coverage_projection_plan, read_verified_repository_body_bytes,
    refine_workspace_coverage_obligation_utf8, runtime_repository_body_store_binding,
};
use anyhow::{Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const CLAIM_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_claim";
const ATTEMPT_TYPE: &str = "gamecult.epiphany.workspace_coverage_projection_attempt";
const CLAIM_KEY: &str = "workspace-coverage-projector-current";
const CLAIM_SCHEMA: &str = "gamecult.epiphany.workspace_coverage_projection_claim.v0";
const ATTEMPT_SCHEMA: &str = "gamecult.epiphany.workspace_coverage_projection_attempt.v0";
const PROJECTION_SCHEMA: &str = "gamecult.epiphany.workspace_bytes_projection.v0";
const CHUNKER_ID: &str = "utf8_lines_96_overlap_8_v0";
const MAXIMUM_FILE_BYTES: u64 = 4 * 1024 * 1024;
const CHUNK_LINES: usize = 96;
const CHUNK_OVERLAP_LINES: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_claim",
    schema = "WorkspaceCoverageProjectionClaim"
)]
pub(crate) struct WorkspaceCoverageProjectionClaim {
    #[cultcache(key = 0)]
    schema_version: String,
    #[cultcache(key = 1)]
    claim_id: String,
    #[cultcache(key = 2)]
    claim_epoch: u64,
    #[cultcache(key = 3)]
    plan_id: String,
    #[cultcache(key = 4)]
    attempt_id: String,
    #[cultcache(key = 5)]
    obligation_id: String,
    #[cultcache(key = 6)]
    body_observation_id: String,
    #[cultcache(key = 7)]
    body_generation: u64,
    #[cultcache(key = 8)]
    manifest_root_sha256: String,
    #[cultcache(key = 9)]
    status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.workspace_coverage_projection_attempt",
    schema = "WorkspaceCoverageProjectionAttempt"
)]
pub(crate) struct WorkspaceCoverageProjectionAttempt {
    #[cultcache(key = 0)]
    schema_version: String,
    #[cultcache(key = 1)]
    attempt_id: String,
    #[cultcache(key = 2)]
    claim_id: String,
    #[cultcache(key = 3)]
    claim_epoch: u64,
    #[cultcache(key = 4)]
    plan_id: String,
    #[cultcache(key = 5)]
    status: String,
    #[cultcache(key = 6)]
    started_at: String,
    #[cultcache(key = 7)]
    completed_at: Option<String>,
    #[cultcache(key = 8)]
    error: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct WorkspaceCoverageAcquisition {
    pub body_store: PathBuf,
    pub obligation: WorkspaceCoverageObligation,
    pub plan: WorkspaceCoverageProjectionPlan,
    pub claim: WorkspaceCoverageProjectionClaim,
    pub attempt: WorkspaceCoverageProjectionAttempt,
}

pub(crate) fn acquire_workspace_coverage_projection(
    runtime_store: &Path,
    basis: &RepositoryBodyObservationBasis,
    embedding_provider_id: &str,
    embedding_model: &str,
    vector_dimensions: u32,
) -> Result<WorkspaceCoverageAcquisition> {
    let policy = WorkspaceCoveragePolicy::bounded_regular_files_v0(MAXIMUM_FILE_BYTES)?;
    let raw_obligation = derive_workspace_coverage_obligation(runtime_store, basis, &policy)?;
    let mut verified_text = Vec::new();
    let mut non_utf8_paths = Vec::new();
    for entry in &raw_obligation.classifications {
        if !matches!(
            entry.disposition,
            crate::WorkspaceCoverageDisposition::IncludeRegularFile
        ) {
            continue;
        }
        let verified = read_verified_repository_body_bytes(runtime_store, basis, &entry.path)?;
        if verified.relative_path != entry.path || verified.raw_sha256 != entry.raw_sha256 {
            bail!("verified Body bytes lost their manifest identity");
        }
        if std::str::from_utf8(&verified.bytes).is_ok() {
            verified_text.push(verified);
        } else {
            non_utf8_paths.push(entry.path.clone());
        }
    }
    let obligation = refine_workspace_coverage_obligation_utf8(&raw_obligation, &non_utf8_paths)?;
    let mut descriptors = Vec::new();
    for verified in verified_text {
        descriptors.extend(chunk_descriptors(
            &verified.relative_path,
            &verified.raw_sha256,
            &verified.bytes,
        )?);
    }
    let plan = derive_workspace_coverage_projection_plan(
        &obligation,
        PROJECTION_SCHEMA,
        CHUNKER_ID,
        embedding_provider_id,
        embedding_model,
        vector_dimensions,
        descriptors,
    )?;
    let route = runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body-store binding"))?;
    let body_store = PathBuf::from(route.body_store_path);
    let backing = SingleFileMessagePackBackingStore::new(&body_store);
    let opening = backing.pull_all()?;
    let authority = exact_body_authority(&opening, basis)?;
    let existing_claim = find(&opening, CLAIM_TYPE, CLAIM_KEY);
    let claim_epoch = match existing_claim {
        Some(envelope) => {
            let prior: WorkspaceCoverageProjectionClaim = decode(envelope)?;
            if prior.status == "running" {
                bail!("workspace coverage projection already has a running claim");
            }
            prior
                .claim_epoch
                .checked_add(1)
                .ok_or_else(|| anyhow!("claim epoch exhausted"))?
        }
        None => 1,
    };
    let claim_id = uuid::Uuid::new_v4().to_string();
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let claim = WorkspaceCoverageProjectionClaim {
        schema_version: CLAIM_SCHEMA.into(),
        claim_id: claim_id.clone(),
        claim_epoch,
        plan_id: plan.plan_id.clone(),
        attempt_id: attempt_id.clone(),
        obligation_id: obligation.obligation_id.clone(),
        body_observation_id: basis.observation_id.clone(),
        body_generation: basis.generation,
        manifest_root_sha256: basis.manifest_root_sha256.clone(),
        status: "running".into(),
    };
    let attempt = WorkspaceCoverageProjectionAttempt {
        schema_version: ATTEMPT_SCHEMA.into(),
        attempt_id: attempt_id.clone(),
        claim_id,
        claim_epoch,
        plan_id: plan.plan_id.clone(),
        status: "running".into(),
        started_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
        error: None,
    };
    let mut expected = authority.clone();
    if let Some(existing) = existing_claim {
        expected.push(existing.clone());
    }
    let mut replacements = authority;
    let obligation_env = envelope(
        "gamecult.epiphany.workspace_coverage_obligation",
        &obligation.obligation_id,
        &obligation,
    )?;
    let plan_env = envelope(
        "gamecult.epiphany.workspace_coverage_projection_plan",
        &plan.plan_id,
        &plan,
    )?;
    preserve_or_insert_immutable(&opening, &mut expected, &mut replacements, obligation_env)?;
    preserve_or_insert_immutable(&opening, &mut expected, &mut replacements, plan_env)?;
    replacements.push(envelope(CLAIM_TYPE, CLAIM_KEY, &claim)?);
    replacements.push(envelope(ATTEMPT_TYPE, &attempt_id, &attempt)?);
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        bail!("workspace coverage acquisition lost exact Body/claim CAS");
    }
    Ok(WorkspaceCoverageAcquisition {
        body_store,
        obligation,
        plan,
        claim,
        attempt,
    })
}

pub(crate) fn fail_workspace_coverage_projection(
    acquisition: &WorkspaceCoverageAcquisition,
    error: &str,
) -> Result<()> {
    if error.trim().is_empty() {
        bail!("projection failure requires an error");
    }
    let backing = SingleFileMessagePackBackingStore::new(&acquisition.body_store);
    let opening = backing.pull_all()?;
    let claim_env =
        find(&opening, CLAIM_TYPE, CLAIM_KEY).ok_or_else(|| anyhow!("running claim missing"))?;
    let attempt_env = find(&opening, ATTEMPT_TYPE, &acquisition.attempt.attempt_id)
        .ok_or_else(|| anyhow!("running attempt missing"))?;
    let claim: WorkspaceCoverageProjectionClaim = decode(claim_env)?;
    let attempt: WorkspaceCoverageProjectionAttempt = decode(attempt_env)?;
    let obligation_env = find(
        &opening,
        "gamecult.epiphany.workspace_coverage_obligation",
        &acquisition.obligation.obligation_id,
    )
    .ok_or_else(|| anyhow!("persisted projection obligation missing"))?;
    let plan_env = find(
        &opening,
        "gamecult.epiphany.workspace_coverage_projection_plan",
        &acquisition.plan.plan_id,
    )
    .ok_or_else(|| anyhow!("persisted projection plan missing"))?;
    if decode::<WorkspaceCoverageObligation>(obligation_env)? != acquisition.obligation
        || decode::<WorkspaceCoverageProjectionPlan>(plan_env)? != acquisition.plan
    {
        bail!("projection failure refuses substituted obligation or plan");
    }
    if claim != acquisition.claim || attempt != acquisition.attempt || claim.status != "running" {
        bail!("projection failure refuses substituted claim or attempt");
    }
    let mut failed_claim = claim.clone();
    failed_claim.status = "failed".into();
    let mut failed_attempt = attempt.clone();
    failed_attempt.status = "failed".into();
    failed_attempt.completed_at = Some(chrono::Utc::now().to_rfc3339());
    failed_attempt.error = Some(error.into());
    let expected = vec![
        obligation_env.clone(),
        plan_env.clone(),
        claim_env.clone(),
        attempt_env.clone(),
    ];
    let replacements = vec![
        obligation_env.clone(),
        plan_env.clone(),
        envelope(CLAIM_TYPE, CLAIM_KEY, &failed_claim)?,
        envelope(ATTEMPT_TYPE, &failed_attempt.attempt_id, &failed_attempt)?,
    ];
    if !backing.compare_and_swap_batch(&expected, replacements)? {
        bail!("projection failure lost exact claim/attempt CAS");
    }
    Ok(())
}

fn preserve_or_insert_immutable(
    opening: &[CultCacheEnvelope],
    expected: &mut Vec<CultCacheEnvelope>,
    replacements: &mut Vec<CultCacheEnvelope>,
    proposed: CultCacheEnvelope,
) -> Result<()> {
    if let Some(existing) = find(opening, &proposed.r#type, &proposed.key) {
        let mut comparable = proposed;
        comparable.stored_at = existing.stored_at.clone();
        if &comparable != existing {
            bail!("immutable workspace coverage document identity collision");
        }
        expected.push(existing.clone());
        replacements.push(existing.clone());
    } else {
        replacements.push(proposed);
    }
    Ok(())
}

fn chunk_descriptors(
    path: &str,
    raw_sha256: &str,
    bytes: &[u8],
) -> Result<Vec<WorkspaceCoverageChunkDescriptor>> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| anyhow!("named text chunker refuses non-UTF-8 bytes"))?;
    let mut line_starts = vec![0usize];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' && index + 1 < bytes.len() {
            line_starts.push(index + 1);
        }
    }
    let mut result = Vec::new();
    let mut start_line = 0usize;
    let mut chunk_index = 0u32;
    while start_line < line_starts.len() {
        let end_line = (start_line + CHUNK_LINES).min(line_starts.len());
        let start = line_starts[start_line];
        let end = if end_line == line_starts.len() {
            bytes.len()
        } else {
            line_starts[end_line]
        };
        result.push(WorkspaceCoverageChunkDescriptor {
            body_path: path.into(),
            source_raw_sha256: raw_sha256.into(),
            source_raw_byte_length: bytes.len() as u64,
            chunk_index,
            byte_start: start as u64,
            byte_end: end as u64,
            chunk_sha256: format!("{:x}", Sha256::digest(&bytes[start..end])),
        });
        if end_line == line_starts.len() {
            break;
        }
        start_line = end_line - CHUNK_OVERLAP_LINES;
        chunk_index = chunk_index
            .checked_add(1)
            .ok_or_else(|| anyhow!("chunk index exhausted"))?;
    }
    Ok(result)
}

fn exact_body_authority(
    opening: &[CultCacheEnvelope],
    basis: &RepositoryBodyObservationBasis,
) -> Result<Vec<CultCacheEnvelope>> {
    let head_env = find(opening, BODY_HEAD_TYPE, BODY_HEAD_KEY)
        .ok_or_else(|| anyhow!("current Body head missing"))?;
    let head: RepositoryBodyHead = decode(head_env)?;
    if head.workspace_id != basis.workspace_id
        || head.generation != basis.generation
        || head.observation_id != basis.observation_id
        || head.manifest_root_sha256 != basis.manifest_root_sha256
    {
        bail!("workspace coverage acquisition requires the exact current Body head");
    }
    [
        (BODY_BINDING_TYPE, BODY_BINDING_KEY),
        (BODY_HEAD_TYPE, BODY_HEAD_KEY),
        (BODY_OBSERVATION_TYPE, basis.observation_id.as_str()),
        (BODY_MANIFEST_TYPE, basis.manifest_root_sha256.as_str()),
    ]
    .into_iter()
    .map(|(ty, key)| {
        find(opening, ty, key)
            .cloned()
            .ok_or_else(|| anyhow!("Body authority envelope {ty}/{key} missing"))
    })
    .collect()
}

fn find<'a>(
    entries: &'a [CultCacheEnvelope],
    ty: &str,
    key: &str,
) -> Option<&'a CultCacheEnvelope> {
    entries
        .iter()
        .find(|entry| entry.r#type == ty && entry.key == key)
}
fn decode<T: serde::de::DeserializeOwned>(value: &CultCacheEnvelope) -> Result<T> {
    Ok(rmp_serde::from_slice(&value.payload)?)
}
fn envelope<T: Serialize>(ty: &str, key: &str, value: &T) -> Result<CultCacheEnvelope> {
    Ok(CultCacheEnvelope {
        key: key.into(),
        r#type: ty.into(),
        payload: rmp_serde::to_vec(value)?,
        stored_at: chrono::Utc::now().to_rfc3339(),
        schema_id: Some(ty.into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    #[test]
    fn named_chunker_preserves_utf8_boundaries_and_eight_line_overlap() -> Result<()> {
        let text = (0..105)
            .map(|index| format!("line {index} λ\n"))
            .collect::<String>();
        let chunks = chunk_descriptors("a", &"11".repeat(32), text.as_bytes())?;
        assert_eq!(chunks.len(), 2);
        let overlap = &text.as_bytes()[chunks[1].byte_start as usize..chunks[0].byte_end as usize];
        assert_eq!(overlap.iter().filter(|byte| **byte == b'\n').count(), 8);
        assert!(
            std::str::from_utf8(
                &text.as_bytes()[chunks[0].byte_start as usize..chunks[0].byte_end as usize]
            )
            .is_ok()
        );
        assert!(
            std::str::from_utf8(
                &text.as_bytes()[chunks[1].byte_start as usize..chunks[1].byte_end as usize]
            )
            .is_ok()
        );
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);
        assert!(chunk_descriptors("binary", &"22".repeat(32), &[0xff, 0xfe]).is_err());
        Ok(())
    }

    fn git(repo: &Path, args: &[&str]) -> Result<()> {
        let output = Command::new("git")
            .args(["-C", repo.to_str().unwrap()])
            .args(args)
            .output()?;
        if !output.status.success() {
            bail!("git failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    #[test]
    fn acquisition_is_exclusive_and_failure_can_terminalize_after_body_advance() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let state = tempfile::tempdir()?;
        git(repo.path(), &["init"])?;
        std::fs::write(repo.path().join("source.rs"), "fn one() {}")?;
        std::fs::write(repo.path().join("binary.dat"), [0xff, 0xfe, 0xfd])?;
        git(repo.path(), &["add", "."])?;
        let runtime = state.path().join("runtime.cc");
        let agents = state.path().join("agents.cc");
        let body = state.path().join("body.cc");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "projector-runtime".into(),
                display_name: "projector test".into(),
                created_at: "2026-07-15T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "projector-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-15T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body, &runtime, "projector-workspace")?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        let first =
            acquire_workspace_coverage_projection(&runtime, &basis, "provider", "model", 3)?;
        assert!(first.obligation.classifications.iter().any(|entry| {
            entry.path == "binary.dat"
                && entry.disposition
                    == crate::WorkspaceCoverageDisposition::ExcludeNonUtf8RegularFile
        }));
        assert_eq!(first.obligation.included_entry_count, 1);
        assert!(
            acquire_workspace_coverage_projection(&runtime, &basis, "provider", "model", 3)
                .is_err()
        );
        fail_workspace_coverage_projection(&first, "expected test failure")?;
        let second =
            acquire_workspace_coverage_projection(&runtime, &basis, "provider", "model", 3)?;
        assert_eq!(second.claim.claim_epoch, first.claim.claim_epoch + 1);
        std::fs::write(repo.path().join("source.rs"), "fn two() {}")?;
        let advanced = crate::observe_runtime_repository_body_basis(&runtime)?;
        assert!(advanced.generation > basis.generation);
        fail_workspace_coverage_projection(&second, "Body advanced during projection")?;
        let entries = SingleFileMessagePackBackingStore::new(&second.body_store).pull_all()?;
        assert!(
            find(
                &entries,
                "gamecult.epiphany.workspace_coverage_receipt",
                "current"
            )
            .is_none()
        );
        assert!(
            find(
                &entries,
                "gamecult.epiphany.workspace_coverage_head",
                "current"
            )
            .is_none()
        );
        Ok(())
    }
}
