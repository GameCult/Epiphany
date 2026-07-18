use anyhow::{Result, anyhow, bail};
use cultcache_rs::{CacheBackingStore, DatabaseEntry, SingleFileMessagePackBackingStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const REQUEST_SCHEMA: &str = "epiphany.self.admitted_model_direction_consideration_request.v0";
pub const REQUEST_CONTRACT: &str = "epiphany.admitted_model_direction_consideration_request.v0";
pub const RESULT_SCHEMA: &str =
    "epiphany.imagination.admitted_model_direction_consideration_result.v0";
pub const RESULT_CONTRACT: &str = "epiphany.admitted_model_direction_consideration_result.v0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmittedModelDirectionDisposition {
    Suggest,
    Hold,
    NoFit,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.admitted_model_direction_consideration_request",
    schema = "AdmittedModelDirectionConsiderationRequest"
)]
pub struct AdmittedModelDirectionConsiderationRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub thread_id: String,
    #[cultcache(key = 4)]
    pub model_revision: u64,
    #[cultcache(key = 5)]
    pub model_hash: String,
    #[cultcache(key = 6)]
    pub model_admission_receipt_id: String,
    #[cultcache(key = 7, default)]
    pub previous_terminal_result_id: Option<String>,
    #[cultcache(key = 8)]
    pub requested_at: String,
    #[cultcache(key = 9)]
    pub contract: String,
    #[cultcache(key = 10, default)]
    pub private_state_included: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.imagination.admitted_model_direction_consideration_result",
    schema = "AdmittedModelDirectionConsiderationResult"
)]
pub struct AdmittedModelDirectionConsiderationResult {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub result_id: String,
    #[cultcache(key = 2)]
    pub request_id: String,
    #[cultcache(key = 3)]
    pub runtime_id: String,
    #[cultcache(key = 4)]
    pub thread_id: String,
    #[cultcache(key = 5)]
    pub model_revision: u64,
    #[cultcache(key = 6)]
    pub model_hash: String,
    #[cultcache(key = 7)]
    pub model_admission_receipt_id: String,
    #[cultcache(key = 8)]
    pub disposition: AdmittedModelDirectionDisposition,
    #[cultcache(key = 9)]
    pub summary: String,
    #[cultcache(key = 10, default)]
    pub option_drafts: Vec<crate::ImaginationOptionDraft>,
    #[cultcache(key = 11, default)]
    pub uncertainties: Vec<String>,
    #[cultcache(key = 12, default)]
    pub evidence_refs: Vec<String>,
    #[cultcache(key = 13)]
    pub proposed_at: String,
    #[cultcache(key = 14)]
    pub contract: String,
    #[cultcache(key = 15, default)]
    pub proposal_only: bool,
    #[cultcache(key = 16, default)]
    pub terminal: bool,
}

pub fn commit_request(
    runtime_store: &Path,
    requested_at: &str,
) -> Result<Option<AdmittedModelDirectionConsiderationRequest>> {
    chrono::DateTime::parse_from_rfc3339(requested_at)
        .map_err(|_| anyhow!("model direction consideration timestamp must be RFC3339"))?;
    let mut cache = crate::runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let identity = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("model direction consideration requires runtime identity"))?;
    let Some(thread) = cache.get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
    else {
        return Ok(None);
    };
    let model = crate::runtime_current_repo_model(runtime_store)?
        .ok_or_else(|| anyhow!("model direction consideration requires admitted Modeling map"))?;
    let model_hash = crate::memory_graph_model_hash(&model)?;
    let receipts = cache
        .get_all::<crate::RepoModelAdmissionReceipt>()?
        .into_iter()
        .filter(|receipt| {
            receipt.admitted_revision == model.model_revision && receipt.admitted_hash == model_hash
        })
        .collect::<Vec<_>>();
    if receipts.len() != 1 {
        bail!("model direction consideration requires exactly one current model receipt");
    }
    let mut terminal = cache
        .get_all::<AdmittedModelDirectionConsiderationResult>()?
        .into_iter()
        .filter(|result| result.terminal)
        .collect::<Vec<_>>();
    terminal.sort_by(|left, right| left.proposed_at.cmp(&right.proposed_at));
    if terminal.iter().any(|result| {
        result.model_revision == model.model_revision
            && result.model_hash == model_hash
            && result.model_admission_receipt_id == receipts[0].receipt_id
    }) {
        return Ok(None);
    }
    let previous_terminal_result_id = terminal.last().map(|result| result.result_id.clone());
    let causal = rmp_serde::to_vec_named(&(
        &identity.runtime_id,
        &thread.thread_id,
        model.model_revision,
        &model_hash,
        &receipts[0].receipt_id,
        &previous_terminal_result_id,
    ))?;
    let request_id = format!(
        "admitted-model-direction-consideration-{:x}",
        Sha256::digest(causal)
    );
    let request = AdmittedModelDirectionConsiderationRequest {
        schema_version: REQUEST_SCHEMA.into(),
        request_id: request_id.clone(),
        runtime_id: identity.runtime_id,
        thread_id: thread.thread_id,
        model_revision: model.model_revision,
        model_hash,
        model_admission_receipt_id: receipts[0].receipt_id.clone(),
        previous_terminal_result_id,
        requested_at: requested_at.into(),
        contract: REQUEST_CONTRACT.into(),
        private_state_included: false,
    };
    validate_current_request(&cache, &request)?;
    if let Some(existing) = cache.get::<AdmittedModelDirectionConsiderationRequest>(&request_id)? {
        let mut replay = request;
        replay.requested_at = existing.requested_at.clone();
        return if replay == existing {
            Ok(Some(existing))
        } else {
            bail!("model direction request identity collision")
        };
    }
    let (entry, _) = cache.prepare_entry(&request_id, &request)?;
    SingleFileMessagePackBackingStore::new(runtime_store).push(&entry)?;
    Ok(Some(request))
}

pub fn validate_current_request(
    cache: &cultcache_rs::CultCache,
    request: &AdmittedModelDirectionConsiderationRequest,
) -> Result<()> {
    if request.schema_version != REQUEST_SCHEMA
        || request.contract != REQUEST_CONTRACT
        || request.private_state_included
        || request.request_id.trim().is_empty()
    {
        bail!("invalid model direction consideration request");
    }
    let model = cache
        .get::<crate::EpiphanyMemoryGraphEntry>(crate::MEMORY_GRAPH_KEY)?
        .ok_or_else(|| anyhow!("model direction consideration map disappeared"))?
        .snapshot()?;
    if request.model_revision != model.model_revision
        || request.model_hash != crate::memory_graph_model_hash(&model)?
    {
        bail!("model direction consideration request is stale");
    }
    let receipts = cache
        .get_all::<crate::RepoModelAdmissionReceipt>()?
        .into_iter()
        .filter(|receipt| {
            receipt.receipt_id == request.model_admission_receipt_id
                && receipt.admitted_revision == request.model_revision
                && receipt.admitted_hash == request.model_hash
        })
        .count();
    if receipts != 1 {
        bail!("model direction consideration lost its unique model receipt");
    }
    Ok(())
}

pub fn validate_result(
    request: &AdmittedModelDirectionConsiderationRequest,
    result: &AdmittedModelDirectionConsiderationResult,
) -> Result<()> {
    if result.schema_version != RESULT_SCHEMA
        || result.contract != RESULT_CONTRACT
        || !result.proposal_only
        || !result.terminal
        || result.request_id != request.request_id
        || result.runtime_id != request.runtime_id
        || result.thread_id != request.thread_id
        || result.model_revision != request.model_revision
        || result.model_hash != request.model_hash
        || result.model_admission_receipt_id != request.model_admission_receipt_id
        || result.result_id.trim().is_empty()
        || result.summary.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&result.proposed_at).is_err()
    {
        bail!("model direction result substituted causal identity");
    }
    if result.disposition == AdmittedModelDirectionDisposition::Suggest
        && result.option_drafts.is_empty()
    {
        bail!("model direction suggestion requires at least one option");
    }
    Ok(())
}

pub fn result_id_for_launch(request_id: &str, job_id: &str) -> String {
    format!(
        "admitted-model-direction-consideration-result-{:x}",
        Sha256::digest(format!("{request_id}:{job_id}").as_bytes())
    )
}

pub fn render_prompt(request: &AdmittedModelDirectionConsiderationRequest) -> String {
    format!(
        "Act as Epiphany Imagination for one proposal-only direction consideration. Inspect the exact current admitted Modeling map bound by request {} at revision/hash {}/{}. Suggest options or hold. Do not adopt, edit, execute, release, or deploy.",
        request.request_id, request.model_revision, request.model_hash
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn request() -> AdmittedModelDirectionConsiderationRequest {
        AdmittedModelDirectionConsiderationRequest {
            schema_version: REQUEST_SCHEMA.into(),
            request_id: "request-1".into(),
            runtime_id: "runtime-1".into(),
            thread_id: "thread-1".into(),
            model_revision: 7,
            model_hash: "sha256:model-7".into(),
            model_admission_receipt_id: "receipt-7".into(),
            previous_terminal_result_id: None,
            requested_at: "2026-07-18T00:00:00Z".into(),
            contract: REQUEST_CONTRACT.into(),
            private_state_included: false,
        }
    }

    fn result(
        request: &AdmittedModelDirectionConsiderationRequest,
    ) -> AdmittedModelDirectionConsiderationResult {
        AdmittedModelDirectionConsiderationResult {
            schema_version: RESULT_SCHEMA.into(),
            result_id: result_id_for_launch(&request.request_id, "job-1"),
            request_id: request.request_id.clone(),
            runtime_id: request.runtime_id.clone(),
            thread_id: request.thread_id.clone(),
            model_revision: request.model_revision,
            model_hash: request.model_hash.clone(),
            model_admission_receipt_id: request.model_admission_receipt_id.clone(),
            disposition: AdmittedModelDirectionDisposition::Hold,
            summary: "No direction should be promoted yet.".into(),
            option_drafts: Vec::new(),
            uncertainties: vec!["Current evidence is incomplete.".into()],
            evidence_refs: vec!["cultcache://runtime/repo-model/sha256:model-7".into()],
            proposed_at: "2026-07-18T00:01:00Z".into(),
            contract: RESULT_CONTRACT.into(),
            proposal_only: true,
            terminal: true,
        }
    }

    #[test]
    fn result_requires_exact_request_model_receipt_and_proposal_only_terminal_authority() {
        let request = request();
        assert!(validate_result(&request, &result(&request)).is_ok());
        for mutation in 0..6 {
            let mut substituted = result(&request);
            match mutation {
                0 => substituted.request_id = "request-stale".into(),
                1 => substituted.model_revision += 1,
                2 => substituted.model_hash = "sha256:substituted".into(),
                3 => substituted.model_admission_receipt_id = "receipt-substituted".into(),
                4 => substituted.proposal_only = false,
                _ => substituted.terminal = false,
            }
            assert!(validate_result(&request, &substituted).is_err());
        }
    }

    #[test]
    fn suggestion_requires_an_option_but_hold_remains_non_actuating() {
        let request = request();
        let mut proposed = result(&request);
        proposed.disposition = AdmittedModelDirectionDisposition::Suggest;
        assert!(validate_result(&request, &proposed).is_err());
        proposed.option_drafts.push(crate::ImaginationOptionDraft {
            title: "Explore typed route".into(),
            summary: "Ask Modeling to assess a bounded proposal.".into(),
        });
        assert!(validate_result(&request, &proposed).is_ok());
    }

    #[test]
    fn cold_runtime_without_thread_has_no_direction_request_yet() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.ccmp");
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.put(
            crate::RUNTIME_IDENTITY_KEY,
            &crate::EpiphanyRuntimeIdentity {
                schema_version: crate::RUNTIME_SPINE_SCHEMA_VERSION.into(),
                runtime_id: "runtime-cold".into(),
                display_name: "Cold runtime".into(),
                runtime_kind: "resident".into(),
                created_at: "2026-07-18T00:00:00Z".into(),
                updated_at: "2026-07-18T00:00:00Z".into(),
                supported_document_types: Vec::new(),
                metadata: BTreeMap::new(),
            },
        )?;

        assert!(commit_request(&store, "2026-07-18T00:01:00Z")?.is_none());
        let mut reloaded = crate::runtime_spine_cache(&store)?;
        reloaded.pull_all_backing_stores()?;
        assert!(
            reloaded
                .get_all::<AdmittedModelDirectionConsiderationRequest>()?
                .is_empty()
        );
        Ok(())
    }
}
