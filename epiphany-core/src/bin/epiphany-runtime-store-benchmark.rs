use anyhow::{Context, Result, anyhow};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, RedbMessagePackBackingStore,
    SingleFileMessagePackBackingStore,
};
use epiphany_core::{EpiphanyRuntimeJob, RUNTIME_JOB_TYPE};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::Instant;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkReceipt {
    schema_version: &'static str,
    store: String,
    job_id: String,
    job_created_at: String,
    selection_window_milliseconds: i64,
    selected_stored_ats: Vec<String>,
    selected_entry_count: usize,
    selected_identities: Vec<String>,
    selected_envelope_set_sha256: String,
    read_milliseconds: u128,
    cas_milliseconds: Vec<u128>,
}

enum Store {
    Snapshot(SingleFileMessagePackBackingStore),
    Keyed(RedbMessagePackBackingStore),
}

impl Store {
    fn open(path: &Path) -> Result<Self> {
        match path.extension().and_then(|value| value.to_str()) {
            Some("cc" | "msgpack") => {
                Ok(Self::Snapshot(SingleFileMessagePackBackingStore::new(path)))
            }
            Some("redb") => Ok(Self::Keyed(RedbMessagePackBackingStore::new(path)?)),
            extension => Err(anyhow!(
                "unsupported benchmark store extension {extension:?}"
            )),
        }
    }

    fn pull_all(&self) -> Result<Vec<CultCacheEnvelope>> {
        match self {
            Self::Snapshot(store) => store.pull_all(),
            Self::Keyed(store) => store.pull_all(),
        }
    }

    fn compare_and_swap_batch(
        &self,
        expected: &[CultCacheEnvelope],
        replacements: Vec<CultCacheEnvelope>,
    ) -> Result<bool> {
        match self {
            Self::Snapshot(store) => store.compare_and_swap_batch(expected, replacements),
            Self::Keyed(store) => store.compare_and_swap_batch(expected, replacements),
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let store_path = args
        .next()
        .ok_or_else(|| anyhow!("usage: epiphany-runtime-store-benchmark <disposable-store> <historical-job-id> [repetitions]"))?;
    let job_id = args
        .next()
        .ok_or_else(|| anyhow!("missing historical job id"))?;
    let repetitions = args
        .next()
        .map(|value| value.parse::<usize>())
        .transpose()
        .context("invalid repetitions")?
        .unwrap_or(3);
    if repetitions == 0 || args.next().is_some() {
        return Err(anyhow!("repetitions must be positive and arguments exact"));
    }
    let path = Path::new(&store_path);
    let store = Store::open(path)?;
    let read_started = Instant::now();
    let entries = store.pull_all()?;
    let read_milliseconds = read_started.elapsed().as_millis();
    let job = entries
        .iter()
        .find(|entry| entry.r#type == RUNTIME_JOB_TYPE && entry.key == job_id)
        .ok_or_else(|| anyhow!("historical runtime job envelope is absent"))?;
    let runtime_job: EpiphanyRuntimeJob = rmp_serde::from_slice(&job.payload)?;
    let job_created_at = runtime_job.created_at;
    let job_created = chrono::DateTime::parse_from_rfc3339(&job_created_at)?;
    let selection_window_milliseconds = 2_000_i64;
    let mut selected = entries
        .into_iter()
        .filter(|entry| {
            chrono::DateTime::parse_from_rfc3339(&entry.stored_at)
                .map(|stored| {
                    (stored.timestamp_millis() - job_created.timestamp_millis()).abs()
                        <= selection_window_milliseconds
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| (&left.r#type, &left.key).cmp(&(&right.r#type, &right.key)));
    if selected.is_empty() {
        return Err(anyhow!("historical job commit selected no envelopes"));
    }
    let selected_identities = selected
        .iter()
        .map(|entry| format!("{}:{}", entry.r#type, entry.key))
        .collect::<Vec<_>>();
    let selected_stored_ats = selected
        .iter()
        .map(|entry| entry.stored_at.clone())
        .collect::<Vec<_>>();
    let selected_envelope_set_sha256 =
        format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&selected)?));
    let mut cas_milliseconds = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        let started = Instant::now();
        if !store.compare_and_swap_batch(&selected, selected.clone())? {
            return Err(anyhow!("exact historical batch CAS unexpectedly refused"));
        }
        cas_milliseconds.push(started.elapsed().as_millis());
    }
    let receipt = BenchmarkReceipt {
        schema_version: "epiphany.runtime_store_benchmark_receipt.v1",
        store: path.display().to_string(),
        job_id,
        job_created_at,
        selection_window_milliseconds,
        selected_stored_ats,
        selected_entry_count: selected.len(),
        selected_identities,
        selected_envelope_set_sha256,
        read_milliseconds,
        cas_milliseconds,
    };
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
