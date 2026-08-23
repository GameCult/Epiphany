use super::{
    EpiphanyHeartbeatArtifactRetentionPlan, EpiphanyHeartbeatArtifactRetentionReceipt,
    HEARTBEAT_ARTIFACT_RETENTION_PLAN_LATEST_KEY, HEARTBEAT_ARTIFACT_RETENTION_PLAN_SCHEMA_VERSION,
    HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_LATEST_KEY,
    HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_SCHEMA_VERSION, HeartbeatArtifactRetentionMember,
    heartbeat_state_cache,
};
use anyhow::{Context, Result, anyhow, bail};
use cultcache_rs::{DatabaseEntry, SingleFileMessagePackBackingStore};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn retain_heartbeat_pulse_artifacts(
    store_path: impl AsRef<Path>,
    artifact_root: impl AsRef<Path>,
    retain_pulse_count: usize,
    batch_size: usize,
    completed_at_utc: &str,
) -> Result<Option<EpiphanyHeartbeatArtifactRetentionReceipt>> {
    if retain_pulse_count == 0 || batch_size == 0 {
        bail!("heartbeat artifact retention requires non-zero retain and batch counts");
    }
    if completed_at_utc.trim().is_empty() {
        bail!("heartbeat artifact retention requires completion time");
    }
    let artifact_root = artifact_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", artifact_root.as_ref().display()))?;
    if !artifact_root.is_dir() {
        bail!("heartbeat artifact root is not a directory");
    }

    if let Some(pending) = pending_retention_plan(store_path.as_ref())? {
        if PathBuf::from(&pending.artifact_root) != artifact_root {
            bail!("pending heartbeat retention plan belongs to another artifact root");
        }
        return reconcile_retention_plan(
            store_path.as_ref(),
            &artifact_root,
            &pending,
            completed_at_utc,
        )
        .map(Some);
    }

    let pulses = pulse_directories(&artifact_root)?;
    let threshold = retain_pulse_count
        .checked_add(batch_size)
        .ok_or_else(|| anyhow!("heartbeat artifact retention threshold overflow"))?;
    if pulses.len() <= threshold {
        return Ok(None);
    }
    let retire_count = batch_size.min(pulses.len() - retain_pulse_count);
    let members = pulses
        .iter()
        .take(retire_count)
        .map(|(_, path)| artifact_member(&artifact_root, path))
        .collect::<Result<Vec<_>>>()?;
    let plan_id = retention_plan_id(&artifact_root, retain_pulse_count, batch_size, &members);
    let plan = EpiphanyHeartbeatArtifactRetentionPlan {
        schema_version: HEARTBEAT_ARTIFACT_RETENTION_PLAN_SCHEMA_VERSION.to_string(),
        plan_id,
        artifact_root: artifact_root.display().to_string(),
        retain_pulse_count: retain_pulse_count as u64,
        batch_size: batch_size as u64,
        members,
        planned_at_utc: completed_at_utc.to_string(),
        private_state_exposed: false,
    };
    validate_retention_plan(&plan)?;
    write_retention_plan(store_path.as_ref(), &plan)?;
    reconcile_retention_plan(store_path.as_ref(), &artifact_root, &plan, completed_at_utc).map(Some)
}

fn pending_retention_plan(
    store_path: &Path,
) -> Result<Option<EpiphanyHeartbeatArtifactRetentionPlan>> {
    let cache = heartbeat_state_cache(store_path)?;
    let plan = cache.get::<EpiphanyHeartbeatArtifactRetentionPlan>(
        HEARTBEAT_ARTIFACT_RETENTION_PLAN_LATEST_KEY,
    )?;
    let receipt = cache.get::<EpiphanyHeartbeatArtifactRetentionReceipt>(
        HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_LATEST_KEY,
    )?;
    Ok(plan.filter(|plan| {
        receipt
            .as_ref()
            .is_none_or(|receipt| receipt.plan_id != plan.plan_id)
    }))
}

fn reconcile_retention_plan(
    store_path: &Path,
    artifact_root: &Path,
    plan: &EpiphanyHeartbeatArtifactRetentionPlan,
    completed_at_utc: &str,
) -> Result<EpiphanyHeartbeatArtifactRetentionReceipt> {
    validate_retention_plan(plan)?;
    for member in &plan.members {
        let path = artifact_root.join(&member.directory_name);
        if !path.exists() {
            continue;
        }
        let observed = artifact_member(artifact_root, &path)?;
        if observed != *member {
            bail!(
                "heartbeat artifact {:?} changed after retention planning",
                member.directory_name
            );
        }
        fs::remove_dir_all(&path)
            .with_context(|| format!("failed to retire heartbeat artifact {}", path.display()))?;
    }
    for member in &plan.members {
        if artifact_root.join(&member.directory_name).exists() {
            bail!("heartbeat artifact retirement did not remove planned directory");
        }
    }
    let receipt = EpiphanyHeartbeatArtifactRetentionReceipt {
        schema_version: HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: format!("artifact-retention-receipt-{}", plan.plan_id),
        plan_id: plan.plan_id.clone(),
        status: "completed".to_string(),
        deleted_directories: plan
            .members
            .iter()
            .map(|member| member.directory_name.clone())
            .collect(),
        deleted_file_count: plan.members.iter().map(|member| member.file_count).sum(),
        deleted_byte_count: plan.members.iter().map(|member| member.byte_count).sum(),
        completed_at_utc: completed_at_utc.to_string(),
        private_state_exposed: false,
    };
    validate_retention_receipt(&receipt, plan)?;
    write_retention_receipt(store_path, &receipt)?;
    Ok(receipt)
}

fn pulse_directories(root: &Path) -> Result<Vec<(u64, PathBuf)>> {
    let mut pulses = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }
        if file_type.is_symlink() {
            bail!("heartbeat artifact retention refuses symlink directories");
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("heartbeat artifact directory name is not UTF-8"))?;
        let sequence = pulse_sequence(&name)
            .ok_or_else(|| anyhow!("heartbeat artifact retention refuses directory {name:?}"))?;
        pulses.push((sequence, entry.path()));
    }
    pulses.sort_by_key(|(sequence, _)| *sequence);
    if pulses.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        bail!("heartbeat artifact root contains duplicate pulse sequences");
    }
    Ok(pulses)
}

fn pulse_sequence(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("pulse-")?;
    (digits.len() == 6 && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| digits.parse().ok())
        .flatten()
}

fn artifact_member(root: &Path, directory: &Path) -> Result<HeartbeatArtifactRetentionMember> {
    let name = directory
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("heartbeat artifact directory has no UTF-8 name"))?;
    pulse_sequence(name).ok_or_else(|| anyhow!("invalid heartbeat pulse directory {name:?}"))?;
    let canonical = directory.canonicalize()?;
    if canonical.parent() != Some(root) {
        bail!("heartbeat artifact directory escapes its canonical root");
    }
    let mut rows = Vec::new();
    collect_manifest_rows(&canonical, &canonical, &mut rows)?;
    rows.sort();
    let mut digest = Sha256::new();
    let mut file_count = 0_u64;
    let mut byte_count = 0_u64;
    for row in rows {
        if row.kind == b'F' {
            file_count += 1;
            byte_count = byte_count
                .checked_add(row.byte_count)
                .ok_or_else(|| anyhow!("heartbeat artifact byte count overflow"))?;
        }
        digest.update([row.kind]);
        digest.update((row.relative_path.len() as u64).to_be_bytes());
        digest.update(row.relative_path.as_bytes());
        digest.update(row.byte_count.to_be_bytes());
        digest.update(row.sha256.as_bytes());
    }
    Ok(HeartbeatArtifactRetentionMember {
        directory_name: name.to_string(),
        manifest_sha256: format!("sha256-{:x}", digest.finalize()),
        file_count,
        byte_count,
    })
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct ManifestRow {
    kind: u8,
    relative_path: String,
    byte_count: u64,
    sha256: String,
}

fn collect_manifest_rows(root: &Path, directory: &Path, rows: &mut Vec<ManifestRow>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            bail!("heartbeat artifact retention refuses symlinks");
        }
        let path = entry.path();
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        if file_type.is_dir() {
            rows.push(ManifestRow {
                kind: b'D',
                relative_path: relative,
                byte_count: 0,
                sha256: String::new(),
            });
            collect_manifest_rows(root, &path, rows)?;
        } else if file_type.is_file() {
            let bytes = fs::read(&path)?;
            rows.push(ManifestRow {
                kind: b'F',
                relative_path: relative,
                byte_count: bytes.len() as u64,
                sha256: format!("sha256-{:x}", Sha256::digest(bytes)),
            });
        } else {
            bail!("heartbeat artifact retention refuses non-file artifacts");
        }
    }
    Ok(())
}

fn retention_plan_id(
    root: &Path,
    retain_pulse_count: usize,
    batch_size: usize,
    members: &[HeartbeatArtifactRetentionMember],
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"epiphany.heartbeat.artifact-retention-plan.v0\0");
    digest.update(root.to_string_lossy().as_bytes());
    digest.update((retain_pulse_count as u64).to_be_bytes());
    digest.update((batch_size as u64).to_be_bytes());
    for member in members {
        digest.update(member.directory_name.as_bytes());
        digest.update(member.manifest_sha256.as_bytes());
        digest.update(member.file_count.to_be_bytes());
        digest.update(member.byte_count.to_be_bytes());
    }
    format!("sha256-{:x}", digest.finalize())
}

fn validate_retention_plan(plan: &EpiphanyHeartbeatArtifactRetentionPlan) -> Result<()> {
    if plan.schema_version != HEARTBEAT_ARTIFACT_RETENTION_PLAN_SCHEMA_VERSION
        || plan.private_state_exposed
        || plan.plan_id.trim().is_empty()
        || plan.artifact_root.trim().is_empty()
        || plan.retain_pulse_count == 0
        || plan.batch_size == 0
        || plan.members.is_empty()
        || plan.planned_at_utc.trim().is_empty()
    {
        bail!("invalid heartbeat artifact retention plan");
    }
    if plan.members.len() as u64 > plan.batch_size {
        bail!("heartbeat artifact retention plan exceeds its batch size");
    }
    let mut names = std::collections::BTreeSet::new();
    for member in &plan.members {
        if pulse_sequence(&member.directory_name).is_none()
            || !member.manifest_sha256.starts_with("sha256-")
            || !names.insert(member.directory_name.as_str())
        {
            bail!("invalid heartbeat artifact retention member");
        }
    }
    Ok(())
}

fn validate_retention_receipt(
    receipt: &EpiphanyHeartbeatArtifactRetentionReceipt,
    plan: &EpiphanyHeartbeatArtifactRetentionPlan,
) -> Result<()> {
    if receipt.schema_version != HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_SCHEMA_VERSION
        || receipt.private_state_exposed
        || receipt.status != "completed"
        || receipt.plan_id != plan.plan_id
        || receipt.deleted_directories
            != plan
                .members
                .iter()
                .map(|member| member.directory_name.clone())
                .collect::<Vec<_>>()
        || receipt.completed_at_utc.trim().is_empty()
    {
        bail!("invalid heartbeat artifact retention receipt");
    }
    Ok(())
}

fn write_retention_plan(
    store_path: &Path,
    plan: &EpiphanyHeartbeatArtifactRetentionPlan,
) -> Result<()> {
    let cache = heartbeat_state_cache(store_path)?;
    let (plan_entry, _) = cache.prepare_entry(&plan.plan_id, plan)?;
    let (latest_entry, _) =
        cache.prepare_entry(HEARTBEAT_ARTIFACT_RETENTION_PLAN_LATEST_KEY, plan)?;
    let expected = cache
        .snapshot_envelopes()
        .into_iter()
        .filter(|entry| {
            entry.r#type == <EpiphanyHeartbeatArtifactRetentionPlan as DatabaseEntry>::TYPE
                && (entry.key == plan.plan_id
                    || entry.key == HEARTBEAT_ARTIFACT_RETENTION_PLAN_LATEST_KEY)
        })
        .collect::<Vec<_>>();
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, vec![plan_entry, latest_entry])?
    {
        bail!("heartbeat artifact retention plan lost exact compare-and-swap");
    }
    Ok(())
}

fn write_retention_receipt(
    store_path: &Path,
    receipt: &EpiphanyHeartbeatArtifactRetentionReceipt,
) -> Result<()> {
    let cache = heartbeat_state_cache(store_path)?;
    if let Some(existing) =
        cache.get::<EpiphanyHeartbeatArtifactRetentionReceipt>(&receipt.receipt_id)?
    {
        if existing == *receipt {
            return Ok(());
        }
        bail!("heartbeat artifact retention receipt identity collision");
    }
    let (receipt_entry, _) = cache.prepare_entry(&receipt.receipt_id, receipt)?;
    let (latest_entry, _) =
        cache.prepare_entry(HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_LATEST_KEY, receipt)?;
    let expected = cache
        .snapshot_envelopes()
        .into_iter()
        .filter(|entry| {
            entry.r#type == <EpiphanyHeartbeatArtifactRetentionReceipt as DatabaseEntry>::TYPE
                && (entry.key == receipt.receipt_id
                    || entry.key == HEARTBEAT_ARTIFACT_RETENTION_RECEIPT_LATEST_KEY)
        })
        .collect::<Vec<_>>();
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, vec![receipt_entry, latest_entry])?
    {
        bail!("heartbeat artifact retention receipt lost exact compare-and-swap");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialize_heartbeat_store;

    fn pulse(root: &Path, sequence: u64, content: &str) -> Result<PathBuf> {
        let path = root.join(format!("pulse-{sequence:06}"));
        fs::create_dir_all(path.join("nested"))?;
        fs::write(path.join("nested").join("artifact.json"), content)?;
        Ok(path)
    }

    #[test]
    fn retention_is_receipted_bounded_and_idempotent() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("heartbeat.cc");
        let artifacts = temp.path().join("artifacts");
        fs::create_dir(&artifacts)?;
        initialize_heartbeat_store(&store, 1.0)?;
        for sequence in 1..=5 {
            pulse(&artifacts, sequence, &format!("artifact-{sequence}"))?;
        }

        let receipt =
            retain_heartbeat_pulse_artifacts(&store, &artifacts, 2, 2, "2026-08-08T21:00:00Z")?
                .expect("retention should run above hysteresis threshold");
        assert_eq!(
            receipt.deleted_directories,
            vec!["pulse-000001", "pulse-000002"]
        );
        assert_eq!(receipt.deleted_file_count, 2);
        assert_eq!(pulse_directories(&artifacts)?.len(), 3);
        assert!(
            retain_heartbeat_pulse_artifacts(&store, &artifacts, 2, 2, "2026-08-08T21:00:01Z",)?
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn pending_plan_refuses_changed_artifact_and_preserves_it() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("heartbeat.cc");
        let artifacts = temp.path().join("artifacts");
        fs::create_dir(&artifacts)?;
        initialize_heartbeat_store(&store, 1.0)?;
        let first = pulse(&artifacts, 1, "before")?;
        for sequence in 2..=5 {
            pulse(&artifacts, sequence, "stable")?;
        }
        let members = vec![artifact_member(&artifacts.canonicalize()?, &first)?];
        let root = artifacts.canonicalize()?;
        let plan = EpiphanyHeartbeatArtifactRetentionPlan {
            schema_version: HEARTBEAT_ARTIFACT_RETENTION_PLAN_SCHEMA_VERSION.to_string(),
            plan_id: retention_plan_id(&root, 2, 2, &members),
            artifact_root: root.display().to_string(),
            retain_pulse_count: 2,
            batch_size: 2,
            members,
            planned_at_utc: "2026-08-08T21:01:00Z".to_string(),
            private_state_exposed: false,
        };
        write_retention_plan(&store, &plan)?;
        fs::write(first.join("nested").join("artifact.json"), "after")?;

        assert!(
            retain_heartbeat_pulse_artifacts(&store, &artifacts, 2, 2, "2026-08-08T21:01:01Z",)
                .is_err()
        );
        assert!(first.exists());
        Ok(())
    }

    #[test]
    fn unknown_directory_fails_closed_before_deletion() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("heartbeat.cc");
        let artifacts = temp.path().join("artifacts");
        fs::create_dir(&artifacts)?;
        initialize_heartbeat_store(&store, 1.0)?;
        for sequence in 1..=5 {
            pulse(&artifacts, sequence, "stable")?;
        }
        fs::create_dir(artifacts.join("operator-evidence"))?;

        assert!(
            retain_heartbeat_pulse_artifacts(&store, &artifacts, 2, 2, "2026-08-08T21:02:00Z",)
                .is_err()
        );
        assert!(artifacts.join("pulse-000001").exists());
        Ok(())
    }
}
