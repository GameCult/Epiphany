use anyhow::{Result, anyhow, bail};
use cultcache_rs::{CacheBackingStore, DatabaseEntry, SingleFileMessagePackBackingStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const REQUEST_SCHEMA: &str = "epiphany.self.admitted_model_direction_consideration_request.v1";
pub const REQUEST_CONTRACT: &str = "epiphany.admitted_model_direction_consideration_request.v1";
pub const RESULT_SCHEMA: &str =
    "epiphany.imagination.admitted_model_direction_consideration_result.v1";
pub const RESULT_CONTRACT: &str = "epiphany.admitted_model_direction_consideration_result.v1";
pub const MAX_OPTION_DRAFTS: usize = 3;

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
    pub model_projection_digest: String,
    #[cultcache(key = 5)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
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
    pub model_projection_digest: String,
    #[cultcache(key = 6)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
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
    let model = crate::assemble_repo_model_view(runtime_store)?;
    let model_basis = model.reasoning_basis();
    let mut terminal = cache
        .get_all::<AdmittedModelDirectionConsiderationResult>()?
        .into_iter()
        .filter(|result| result.terminal)
        .collect::<Vec<_>>();
    terminal.sort_by(|left, right| left.proposed_at.cmp(&right.proposed_at));
    if terminal.iter().any(|result| {
        result.model_projection_digest == model_basis.projection_digest
            && result.model_source_documents == model_basis.source_documents
    }) {
        return Ok(None);
    }
    let previous_terminal_result_id = terminal.last().map(|result| result.result_id.clone());
    let request_id = crate::admitted_model_direction_request_id(
        &identity.runtime_id,
        &model_basis.projection_digest,
        previous_terminal_result_id.as_deref(),
    );
    let request = AdmittedModelDirectionConsiderationRequest {
        schema_version: REQUEST_SCHEMA.into(),
        request_id: request_id.clone(),
        runtime_id: identity.runtime_id,
        thread_id: request_id.clone(),
        model_projection_digest: model_basis.projection_digest,
        model_source_documents: model_basis.source_documents,
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
    validate_request(request)?;
    if (crate::EpiphanyRepoModelBasis {
        projection_digest: request.model_projection_digest.clone(),
        source_documents: request.model_source_documents.clone(),
    })
    .validate_against_cache(cache)
    .is_err()
    {
        bail!("model direction consideration request is stale");
    }
    Ok(())
}

pub fn validate_request(request: &AdmittedModelDirectionConsiderationRequest) -> Result<()> {
    if request.schema_version != REQUEST_SCHEMA
        || request.contract != REQUEST_CONTRACT
        || request.private_state_included
        || request.request_id.trim().is_empty()
        || request.runtime_id.trim().is_empty()
        || request.thread_id.trim().is_empty()
        || request
            .previous_terminal_result_id
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        || chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
    {
        bail!("invalid model direction consideration request");
    }
    crate::EpiphanyRepoModelBasis {
        projection_digest: request.model_projection_digest.clone(),
        source_documents: request.model_source_documents.clone(),
    }
    .validate()
    .map_err(|_| anyhow!("invalid model direction consideration request"))?;
    Ok(())
}

pub(crate) fn request_is_superseded(
    cache: &cultcache_rs::CultCache,
    request: &AdmittedModelDirectionConsiderationRequest,
) -> Result<bool> {
    validate_request(request)?;
    Ok(crate::EpiphanyRepoModelBasis {
        projection_digest: request.model_projection_digest.clone(),
        source_documents: request.model_source_documents.clone(),
    }
    .validate_against_cache(cache)
    .is_err())
}

pub fn validate_result(
    request: &AdmittedModelDirectionConsiderationRequest,
    result: &AdmittedModelDirectionConsiderationResult,
) -> Result<()> {
    let requested_at = chrono::DateTime::parse_from_rfc3339(&request.requested_at)
        .map_err(|_| anyhow!("model direction request timestamp must be RFC3339"))?;
    let proposed_at = chrono::DateTime::parse_from_rfc3339(&result.proposed_at)
        .map_err(|_| anyhow!("model direction result timestamp must be RFC3339"))?;
    if result.schema_version != RESULT_SCHEMA
        || result.contract != RESULT_CONTRACT
        || !result.proposal_only
        || !result.terminal
        || result.request_id != request.request_id
        || result.runtime_id != request.runtime_id
        || result.thread_id != request.thread_id
        || result.model_projection_digest != request.model_projection_digest
        || result.model_source_documents != request.model_source_documents
        || result.result_id.trim().is_empty()
        || result.summary.trim().is_empty()
        || proposed_at < requested_at
    {
        bail!("model direction result substituted causal identity");
    }
    if result.disposition == AdmittedModelDirectionDisposition::Suggest
        && result.option_drafts.is_empty()
    {
        bail!("model direction suggestion requires at least one option");
    }
    if result.option_drafts.len() > MAX_OPTION_DRAFTS {
        bail!("model direction result exceeds bounded option fan-out");
    }
    Ok(())
}

pub fn result_id_for_launch(request_id: &str, job_id: &str) -> String {
    format!(
        "admitted-model-direction-consideration-result-{:x}",
        Sha256::digest(format!("{request_id}:{job_id}").as_bytes())
    )
}

pub(crate) fn render_prompt(request: &AdmittedModelDirectionConsiderationRequest) -> String {
    format!(
        "Act as Epiphany Imagination for one proposal-only direction consideration. Inspect the exact current keyed Modeling map bound by request {} at projection digest {}. Suggest options or hold. Do not adopt, edit, execute, release, or deploy.",
        request.request_id, request.model_projection_digest
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn model_basis() -> crate::EpiphanyRepoModelBasis {
        let source_documents = vec![crate::EpiphanyMindDocumentVersion {
            store_id: "epiphany-mind".into(),
            document_type: "epiphany.mind.repo_model.identity.v1".into(),
            document_key: crate::REPO_MODEL_IDENTITY_KEY.into(),
            schema_id: Some("EpiphanyRepoModelIdentityDocument".into()),
            payload_msgpack: vec![1],
            payload_sha256: format!("sha256:{:x}", Sha256::digest([1])),
        }];
        crate::EpiphanyRepoModelBasis {
            projection_digest: format!(
                "sha256:{:x}",
                Sha256::digest(rmp_serde::to_vec_named(&source_documents).unwrap())
            ),
            source_documents,
        }
    }

    fn request() -> AdmittedModelDirectionConsiderationRequest {
        let basis = model_basis();
        AdmittedModelDirectionConsiderationRequest {
            schema_version: REQUEST_SCHEMA.into(),
            request_id: "request-1".into(),
            runtime_id: "runtime-1".into(),
            thread_id: "thread-1".into(),
            model_projection_digest: basis.projection_digest,
            model_source_documents: basis.source_documents,
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
            model_projection_digest: request.model_projection_digest.clone(),
            model_source_documents: request.model_source_documents.clone(),
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
    fn result_refuses_unbounded_autonomous_proposal_fanout() {
        let request = request();
        let mut result = result(&request);
        result.disposition = AdmittedModelDirectionDisposition::Suggest;
        result.option_drafts = (0..=MAX_OPTION_DRAFTS)
            .map(|ordinal| crate::ImaginationOptionDraft {
                title: format!("Option {ordinal}"),
                summary: "Bounded option.".into(),
            })
            .collect();
        assert!(validate_result(&request, &result).is_err());
    }

    #[test]
    fn intrinsic_request_validation_does_not_claim_current_repo_model_authority() {
        let request = request();
        assert!(validate_request(&request).is_ok());
        assert!(validate_current_request(&cultcache_rs::CultCache::new(), &request).is_err());

        let mut substituted = request.clone();
        substituted.model_source_documents[0].payload_msgpack = vec![2];
        assert!(validate_request(&substituted).is_err());

        let mut malformed = request;
        malformed.requested_at = "not-a-time".into();
        assert!(validate_request(&malformed).is_err());
    }

    #[test]
    fn result_requires_exact_request_model_receipt_and_proposal_only_terminal_authority() {
        let request = request();
        assert!(validate_result(&request, &result(&request)).is_ok());
        for mutation in 0..6 {
            let mut substituted = result(&request);
            match mutation {
                0 => substituted.request_id = "request-stale".into(),
                1 => substituted.model_projection_digest = format!("sha256:{}", "0".repeat(64)),
                2 => substituted.model_source_documents.clear(),
                3 => substituted.runtime_id = "runtime-substituted".into(),
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
    fn keyed_repo_model_without_thread_creates_direction_request() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        crate::initialize_runtime_spine(
            &store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "runtime-cold".into(),
                display_name: "Cold runtime".into(),
                created_at: "2026-07-18T00:00:00Z".into(),
            },
        )?;
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.put(
            crate::REPO_MODEL_IDENTITY_KEY,
            &crate::EpiphanyRepoModelIdentityDocument {
                schema_epoch: crate::REPO_MODEL_SCHEMA_EPOCH.into(),
                graph_id: "graph-cold".into(),
                runtime_id: "runtime-cold".into(),
                swarm_id: "swarm-cold".into(),
                workspace_id: "workspace-cold".into(),
                body_binding_sha256: "sha256:body-cold".into(),
            },
        )?;

        let request = commit_request(&store, "2026-07-18T00:01:00Z")?
            .ok_or_else(|| anyhow!("keyed RepoModel should create direction work"))?;
        assert_eq!(request.thread_id, request.request_id);
        Ok(())
    }
}
