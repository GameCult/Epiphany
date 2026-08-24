use crate::open_epiphany_cultmesh_node;
use anyhow::{Context, Result, bail};
use chrono::Utc;
use cultcache_rs::SingleFileMessagePackBackingStore;
use std::path::Path;

pub use epiphany_release_construction::*;

const RELEASE_KEY_PREFIX: &str = "epiphany-local/packaged-release/by-id/";
const RELEASE_HEAD_KEY: &str = "epiphany-local/packaged-release/current";

pub fn publish_epiphany_packaged_release(
    store: &Path,
    runtime_id: &str,
    entry: EpiphanyPackagedReleaseEntry,
) -> Result<EpiphanyPackagedReleaseEntry> {
    validate_epiphany_packaged_release(&entry)?;
    verify_epiphany_packaged_release_files(&entry)?;
    if entry.runtime_id != runtime_id {
        bail!("release witness runtime disagrees with target Verse");
    }
    let node = open_epiphany_cultmesh_node(store, runtime_id.to_string())?;
    let identity_key = format!("{RELEASE_KEY_PREFIX}{}", entry.release_id);
    let existing = node.get::<EpiphanyPackagedReleaseEntry>(&identity_key)?;
    if existing.as_ref().is_some_and(|current| current != &entry) {
        bail!("immutable packaged release identity collision");
    }
    let head = EpiphanyPackagedReleaseHead {
        schema_version: EPIPHANY_PACKAGED_RELEASE_HEAD_SCHEMA_VERSION.into(),
        runtime_id: runtime_id.into(),
        release_id: entry.release_id.clone(),
        witness_sha256: epiphany_packaged_release_witness_sha256(&entry)?,
        published_at_utc: Utc::now().to_rfc3339(),
    };
    let prior_head = node
        .cache()
        .get_envelope::<EpiphanyPackagedReleaseHead>(RELEASE_HEAD_KEY)?;
    let mut expected = Vec::new();
    if let Some(envelope) = prior_head {
        expected.push(envelope);
    }
    let mut replacements = Vec::new();
    if existing.is_none() {
        replacements.push(node.cache().prepare_entry(&identity_key, &entry)?.0);
    }
    replacements.push(node.cache().prepare_entry(RELEASE_HEAD_KEY, &head)?.0);
    if !SingleFileMessagePackBackingStore::new(store)
        .compare_and_swap_batch(&expected, replacements)?
    {
        bail!("packaged release publication lost current-head compare-and-swap");
    }
    Ok(entry)
}

pub fn load_epiphany_packaged_release(
    store: &Path,
    runtime_id: &str,
    release_id: &str,
) -> Result<Option<EpiphanyPackagedReleaseEntry>> {
    open_epiphany_cultmesh_node(store, runtime_id.to_string())?
        .get(&format!("{RELEASE_KEY_PREFIX}{release_id}"))
}

pub fn authenticate_epiphany_packaged_release(
    store: &Path,
    runtime_id: &str,
    release_id: &str,
    expected_witness_sha256: &str,
) -> Result<EpiphanyPackagedReleaseEntry> {
    validate_epiphany_packaged_release_sha256(expected_witness_sha256)?;
    let entry = load_epiphany_packaged_release(store, runtime_id, release_id)?
        .context("packaged release witness is absent")?;
    validate_epiphany_packaged_release(&entry)?;
    verify_epiphany_packaged_release_files(&entry)?;
    if epiphany_packaged_release_witness_sha256(&entry)? != expected_witness_sha256 {
        bail!("packaged release witness digest disagrees with pinned task authority");
    }
    Ok(entry)
}
