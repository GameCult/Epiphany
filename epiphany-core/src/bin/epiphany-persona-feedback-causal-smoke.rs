use anyhow::{Context, Result, bail};
use epiphany_core::*;
use epiphany_state_model::{
    EpiphanyMemoryDomain, EpiphanyMemoryLifecycle, EpiphanyMemoryNode, EpiphanyMemoryNodeKind,
    EpiphanyMemoryProfile, EpiphanyThreadState, RepoModelPatchPurpose,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

const AT: &str = "2026-07-18T00:00:00Z";

fn main() -> Result<()> {
    let args = arguments()?;
    let feedback_store = path(&args, "--persona-feedback-store")?;
    let runtime_store = path(&args, "--runtime-store")?;
    let resident_store = path(&args, "--resident-store")?;
    let forbidden = [
        "--mind-store",
        "--release-store",
        "--local-verse-store",
        "--public-store",
    ]
    .map(|flag| path(&args, flag))
    .into_iter()
    .collect::<Result<Vec<_>>>()?;
    let before = forbidden
        .iter()
        .map(|p| Ok((p.clone(), bytes(p)?)))
        .collect::<Result<Vec<_>>>()?;

    seed_fixture_runtime(&runtime_store)?;
    let admitted = admitted_persona_feedback(&feedback_store, "epiphany-yggdrasil")?;
    let classes = admitted
        .iter()
        .map(|f| (f.source_visibility.as_str(), f.data_classification.as_str()))
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        ("public", "public_feedback"),
        ("organization", "organization_feedback"),
        ("private", "private_feedback"),
    ]);
    if classes != expected {
        bail!("fixture requires exact public, organization, and private feedback classes");
    }

    let inserted = ingest_resident_self_domain_pressure(
        &resident_store,
        &runtime_store,
        &feedback_store,
        "epiphany-yggdrasil",
        1_752_796_800_000,
    )?;
    if inserted < admitted.len() {
        bail!("resident Self did not ingest every admitted feedback item");
    }
    for feedback in admitted {
        let request = commit_imagination_consideration_request(
            &runtime_store,
            &feedback_store,
            &feedback.feedback_id,
            "GameCult/Epiphany",
            "epiphany",
            "resident-feedback-consideration-v0",
            AT,
        )?;
        if request.source_visibility != feedback.source_visibility
            || request.data_classification != feedback.data_classification
            || request.source_room_id != feedback.source_room_id
        {
            bail!("consideration lost classified source provenance");
        }
        let mut runtime = runtime_spine_cache(&runtime_store)?;
        runtime.pull_all_backing_stores()?;
        let state = runtime
            .get::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
            .context("fixture runtime lost its current thread state")?
            .state()?;
        let launch = build_epiphany_imagination_consideration_launch_request(
            "fixture-thread",
            Some(state.revision),
            Some(30),
            &state,
            request.request_id.clone(),
        )
        .map_err(anyhow::Error::msg)?;
        let job = format!("fixture-imagination-{}", feedback.feedback_id);
        let plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &runtime_store,
            format!("launcher-{job}"),
            job.clone(),
        )?;
        let committed = commit_coordinator_job_launch(
            &runtime_store,
            "fixture-thread",
            &state,
            &launch,
            &plan,
            AT.into(),
        )?;
        let candidate = ImaginationConsiderationCandidate {
            schema_version: IMAGINATION_CONSIDERATION_CANDIDATE_SCHEMA_VERSION.into(),
            candidate_id: imagination_consideration_candidate_id_for_launch(
                &request.request_id,
                &committed.backend_job_id,
            ),
            request_id: request.request_id.clone(),
            feedback_id: request.feedback_id.clone(),
            feedback_packet_sha256: request.feedback_packet_sha256.clone(),
            source_room_id: request.source_room_id.clone(),
            source_visibility: request.source_visibility.clone(),
            data_classification: request.data_classification.clone(),
            model_revision: request.model_revision,
            model_hash: request.model_hash.clone(),
            disposition: ImaginationConsiderationDisposition::Suggest,
            title: "Fixture proposal".into(),
            summary: "Reviewable option only".into(),
            rationale: "Compared exact admitted feedback with the current Modeling map.".into(),
            option_drafts: vec![ImaginationOptionDraft {
                title: "Review".into(),
                summary: "Ask Modeling to review; do not mutate state.".into(),
            }],
            uncertainties: Vec::new(),
            evidence_refs: request.quoted_evidence.source_discussion_refs.clone(),
            recommended_review_route: ImaginationConsiderationReviewRoute::ModelingReview,
            proposed_at: AT.into(),
            contract: IMAGINATION_CONSIDERATION_CANDIDATE_CONTRACT.into(),
        };
        validate_imagination_consideration_candidate(&request, &candidate)?;
        let mut runtime = runtime_spine_cache(&runtime_store)?;
        runtime.pull_all_backing_stores()?;
        let current = runtime
            .get::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
            .context("fixture runtime lost launched thread state")?
            .state()?;
        EpiphanyCoordinatorService::new(&runtime_store).interrupt_job(
            "fixture-thread",
            &current,
            EpiphanyJobInterruptRequest {
                expected_revision: Some(current.revision),
                binding_id: committed.binding_id,
                reason: Some("fixture advances to the next classified feedback".into()),
            },
        )?;
    }
    for (path, original) in before {
        if bytes(&path)? != original {
            bail!("fixture mutated forbidden store {}", path.display());
        }
    }
    println!(
        "{{\"schemaVersion\":\"epiphany.persona_feedback_causal_smoke.v0\",\"status\":\"passed\",\"privateStateExposed\":false}}"
    );
    Ok(())
}

fn seed_fixture_runtime(store: &Path) -> Result<()> {
    initialize_runtime_spine(
        store,
        RuntimeSpineInitOptions {
            runtime_id: "epiphany-yggdrasil".into(),
            display_name: "fixture".into(),
            created_at: AT.into(),
        },
    )?;
    let state = EpiphanyThreadState::default();
    let mut cache = runtime_spine_cache(store)?;
    cache.put(
        THREAD_STATE_KEY,
        &EpiphanyThreadStateEntry::from_state("fixture-thread", &state)?,
    )?;
    let mut model = EpiphanyMemoryGraphSnapshot {
        schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.into()),
        graph_id: "fixture-model".into(),
        model_revision: 1,
        domains: vec![EpiphanyMemoryDomain {
            id: "repo".into(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "Repository".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        }],
        nodes: vec![EpiphanyMemoryNode {
            id: "fixture-claim".into(),
            domain_id: "repo".into(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            kind: EpiphanyMemoryNodeKind::RuntimeContract,
            title: "Feedback path".into(),
            claim: "Persona feedback is pressure, never authority.".into(),
            question: "Which proposal fits the Body?".into(),
            action_implication: "Review only.".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        }],
        ..Default::default()
    };
    model.model_hash = memory_graph_model_hash(&model)?;
    cache.put(
        MEMORY_GRAPH_KEY,
        &EpiphanyMemoryGraphEntry::from_snapshot(&model)?,
    )?;
    cache.put(
        "fixture-model-admission",
        &RepoModelAdmissionReceipt {
            schema_version: REPO_MODEL_ADMISSION_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "fixture-model-admission".into(),
            review_id: "fixture-review".into(),
            result_id: "fixture-result".into(),
            patch_id: "fixture-patch".into(),
            patch_sha256: "sha256-fixture".into(),
            previous_revision: 0,
            previous_hash: "none".into(),
            admitted_revision: 1,
            admitted_hash: model.model_hash,
            admitted_at: AT.into(),
            contract: REPO_MODEL_ADMISSION_CONTRACT.into(),
            purpose: RepoModelPatchPurpose::Evolution,
            frontier_route_id: String::new(),
            verification_request_id: String::new(),
            soul_verdict_receipt_id: String::new(),
            frontier_modeling_request_id: String::new(),
            proposal_modeling_request_id: String::new(),
            claim_repair_request_id: String::new(),
            frontier_plan_decision_id: String::new(),
            repository_body_observation_basis: None,
        },
    )?;
    Ok(())
}

fn arguments() -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    let mut args = env::args().skip(1);
    while let Some(k) = args.next() {
        map.insert(k, args.next().context("missing argument value")?);
    }
    Ok(map)
}
fn path(args: &BTreeMap<String, String>, key: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(
        args.get(key).with_context(|| format!("missing {key}"))?,
    ))
}
fn bytes(path: &Path) -> Result<Vec<u8>> {
    Ok(if path.exists() {
        fs::read(path)?
    } else {
        Vec::new()
    })
}
