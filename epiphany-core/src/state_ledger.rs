use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const STATE_LEDGER_STORE_TYPE: &str = "epiphany.state_ledger";
pub const STATE_LEDGER_KEY: &str = "default";
pub const STATE_LEDGER_SCHEMA_VERSION: &str = "epiphany.state_ledger.v0";

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.state_ledger", schema = "EpiphanyStateLedgerEntry")]
pub struct EpiphanyStateLedgerEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub branches: Vec<EpiphanyBranchRecord>,
    #[cultcache(key = 2)]
    pub evidence: Vec<EpiphanyLedgerEvidenceRecord>,
    #[cultcache(key = 3, default)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyBranchRecord {
    pub id: String,
    pub hypothesis: String,
    pub status: String,
    pub artifacts: Vec<String>,
    pub notes: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyLedgerEvidenceRecord {
    pub ts: String,
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub status: String,
    pub note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct BranchProjection {
    pub branches: Vec<EpiphanyBranchRecord>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

pub fn migrate_state_ledgers_to_cultcache(
    branches_path: impl AsRef<Path>,
    evidence_path: impl AsRef<Path>,
    store_path: impl AsRef<Path>,
) -> Result<Value> {
    let branches_path = branches_path.as_ref();
    let evidence_path = evidence_path.as_ref();
    let store_path = store_path.as_ref();
    let branch_raw = fs::read_to_string(branches_path)
        .with_context(|| format!("failed to read {}", branches_path.display()))?;
    let branches: BranchProjection = serde_json::from_str(&branch_raw)
        .with_context(|| format!("failed to decode {}", branches_path.display()))?;
    let evidence = read_evidence_jsonl(evidence_path)?;
    let entry = EpiphanyStateLedgerEntry {
        schema_version: STATE_LEDGER_SCHEMA_VERSION.to_string(),
        branches: branches.branches,
        evidence,
        extra: branches.extra,
    };
    let mut cache = state_ledger_cache(store_path)?;
    cache.put(STATE_LEDGER_KEY, &entry)?;
    Ok(serde_json::json!({
        "ok": true,
        "store": store_path,
        "branches": entry.branches.len(),
        "evidence": entry.evidence.len(),
    }))
}

pub fn state_ledger_status(store_path: impl AsRef<Path>) -> Result<Value> {
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(serde_json::json!({
            "ok": false,
            "present": false,
            "store": store_path,
            "entryType": STATE_LEDGER_STORE_TYPE,
        }));
    }
    let entry = load_state_ledger(store_path)?;
    let active = entry
        .branches
        .iter()
        .filter(|branch| branch.status == "active")
        .count();
    Ok(serde_json::json!({
        "ok": true,
        "present": true,
        "store": store_path,
        "entryType": STATE_LEDGER_STORE_TYPE,
        "branches": entry.branches.len(),
        "activeBranches": active,
        "evidence": entry.evidence.len(),
        "latestEvidence": entry.evidence.iter().rev().take(8).collect::<Vec<_>>(),
    }))
}

pub fn load_state_ledger(store_path: impl AsRef<Path>) -> Result<EpiphanyStateLedgerEntry> {
    let store_path = store_path.as_ref();
    let mut cache = state_ledger_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get_required::<EpiphanyStateLedgerEntry>(STATE_LEDGER_KEY)
}

pub fn append_state_evidence(
    store_path: impl AsRef<Path>,
    record: EpiphanyLedgerEvidenceRecord,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut cache = state_ledger_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut entry = cache
        .get::<EpiphanyStateLedgerEntry>(STATE_LEDGER_KEY)?
        .unwrap_or_else(default_state_ledger);
    validate_evidence(&record)?;
    entry.evidence.push(record.clone());
    cache.put(STATE_LEDGER_KEY, &entry)?;
    Ok(serde_json::json!({
        "ok": true,
        "store": store_path,
        "appended": record,
        "evidence": entry.evidence.len(),
    }))
}

pub fn add_state_branch(
    store_path: impl AsRef<Path>,
    branch: EpiphanyBranchRecord,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut cache = state_ledger_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut entry = cache
        .get::<EpiphanyStateLedgerEntry>(STATE_LEDGER_KEY)?
        .unwrap_or_else(default_state_ledger);
    validate_branch(&branch)?;
    if entry
        .branches
        .iter()
        .any(|candidate| candidate.id == branch.id)
    {
        return Err(anyhow!("Branch {:?} already exists.", branch.id));
    }
    entry.branches.push(branch.clone());
    cache.put(STATE_LEDGER_KEY, &entry)?;
    Ok(serde_json::json!({
        "ok": true,
        "store": store_path,
        "branch": branch,
    }))
}

pub fn close_state_branch(
    store_path: impl AsRef<Path>,
    branch_id: &str,
    status: &str,
    note: Option<String>,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let mut cache = state_ledger_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut entry = cache.get_required::<EpiphanyStateLedgerEntry>(STATE_LEDGER_KEY)?;
    if !matches!(status, "accepted" | "rejected" | "archived") {
        return Err(anyhow!("invalid branch status {:?}", status));
    }
    let Some(branch) = entry
        .branches
        .iter_mut()
        .find(|candidate| candidate.id == branch_id)
    else {
        return Err(anyhow!("Branch {:?} was not found.", branch_id));
    };
    branch.status = status.to_string();
    if let Some(note) = note {
        branch.notes = note;
    }
    let branch = branch.clone();
    cache.put(STATE_LEDGER_KEY, &entry)?;
    Ok(serde_json::json!({
        "ok": true,
        "store": store_path,
        "branch": branch,
    }))
}

pub fn project_state_ledgers_to_json(
    store_path: impl AsRef<Path>,
    branches_path: impl AsRef<Path>,
    evidence_path: impl AsRef<Path>,
) -> Result<Value> {
    let entry = load_state_ledger(store_path.as_ref())?;
    let branches_path = branches_path.as_ref();
    let evidence_path = evidence_path.as_ref();
    if let Some(parent) = branches_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        branches_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&BranchProjection {
                branches: entry.branches.clone(),
                extra: entry.extra.clone(),
            })?
        ),
    )
    .with_context(|| format!("failed to write {}", branches_path.display()))?;
    if let Some(parent) = evidence_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut lines = String::new();
    for record in &entry.evidence {
        lines.push_str(&serde_json::to_string(record)?);
        lines.push('\n');
    }
    fs::write(evidence_path, lines)
        .with_context(|| format!("failed to write {}", evidence_path.display()))?;
    Ok(serde_json::json!({
        "ok": true,
        "branchesPath": branches_path,
        "evidencePath": evidence_path,
        "branches": entry.branches.len(),
        "evidence": entry.evidence.len(),
    }))
}

fn state_ledger_cache(store_path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyStateLedgerEntry>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path));
    Ok(cache)
}

fn default_state_ledger() -> EpiphanyStateLedgerEntry {
    EpiphanyStateLedgerEntry {
        schema_version: STATE_LEDGER_SCHEMA_VERSION.to_string(),
        branches: Vec::new(),
        evidence: Vec::new(),
        extra: BTreeMap::new(),
    }
}

fn read_evidence_jsonl(path: &Path) -> Result<Vec<EpiphanyLedgerEvidenceRecord>> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut records = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record: EpiphanyLedgerEvidenceRecord = serde_json::from_str(line)
            .with_context(|| format!("failed to decode {} line {}", path.display(), index + 1))?;
        validate_evidence(&record).with_context(|| {
            format!("invalid evidence at {} line {}", path.display(), index + 1)
        })?;
        records.push(record);
    }
    Ok(records)
}

fn validate_branch(branch: &EpiphanyBranchRecord) -> Result<()> {
    if branch.id.trim().is_empty() {
        return Err(anyhow!("branch id must be non-empty"));
    }
    if branch.hypothesis.trim().is_empty() {
        return Err(anyhow!("branch hypothesis must be non-empty"));
    }
    if branch.status.trim().is_empty() {
        return Err(anyhow!("branch status must be non-empty"));
    }
    Ok(())
}

fn validate_evidence(record: &EpiphanyLedgerEvidenceRecord) -> Result<()> {
    if record.ts.trim().is_empty() {
        return Err(anyhow!("evidence ts must be non-empty"));
    }
    if record.evidence_type.trim().is_empty() {
        return Err(anyhow!("evidence type must be non-empty"));
    }
    if record.status.trim().is_empty() {
        return Err(anyhow!("evidence status must be non-empty"));
    }
    if record.note.trim().is_empty() {
        return Err(anyhow!("evidence note must be non-empty"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn state_ledgers_migrate_and_append_native_evidence() -> Result<()> {
        let temp = tempdir()?;
        let branches = temp.path().join("branches.json");
        let evidence = temp.path().join("evidence.jsonl");
        let store = temp.path().join("ledgers.msgpack");
        fs::write(
            &branches,
            serde_json::json!({
                "branches": [{
                    "id": "main",
                    "hypothesis": "Typed state reduces drift.",
                    "status": "active",
                    "artifacts": ["state/map.yaml"],
                    "notes": "test"
                }]
            })
            .to_string(),
        )?;
        fs::write(
            &evidence,
            serde_json::to_string(&EpiphanyLedgerEvidenceRecord {
                ts: "2026-05-06T00:00:00+00:00".to_string(),
                evidence_type: "test".to_string(),
                status: "ok".to_string(),
                note: "The initial ledger record exists.".to_string(),
                branch: None,
                extra: BTreeMap::new(),
            })? + "\n",
        )?;
        migrate_state_ledgers_to_cultcache(&branches, &evidence, &store)?;
        let entry = load_state_ledger(&store)?;
        assert_eq!(entry.branches.len(), 1);
        assert_eq!(entry.evidence.len(), 1);
        append_state_evidence(
            &store,
            EpiphanyLedgerEvidenceRecord {
                ts: "2026-05-06T00:01:00+00:00".to_string(),
                evidence_type: "implementation".to_string(),
                status: "ok".to_string(),
                note: "Native evidence append works.".to_string(),
                branch: Some("main".to_string()),
                extra: BTreeMap::new(),
            },
        )?;
        assert_eq!(load_state_ledger(&store)?.evidence.len(), 2);
        Ok(())
    }
}
