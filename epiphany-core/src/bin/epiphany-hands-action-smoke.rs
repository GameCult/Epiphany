use anyhow::Result;
use epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION;
use epiphany_core::HandsActionIntent;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::hands_action_review_for_intent;
use epiphany_core::hands_command_receipt_for_review;
use epiphany_core::hands_commit_receipt_for_review;
use epiphany_core::hands_patch_receipt_for_review;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::put_hands_action_intent;
use epiphany_core::put_hands_action_review;
use epiphany_core::put_hands_command_receipt;
use epiphany_core::put_hands_commit_receipt;
use epiphany_core::put_hands_patch_receipt;
use epiphany_core::runtime_hands_action_intent;
use epiphany_core::runtime_hands_action_review;
use epiphany_core::runtime_hands_command_receipt;
use epiphany_core::runtime_hands_commit_receipt;
use epiphany_core::runtime_hands_patch_receipt;
use serde_json::json;
use std::path::PathBuf;

fn main() -> Result<()> {
    let store = PathBuf::from(".epiphany-smoke")
        .join("runtime-spine")
        .join("hands-action-smoke.msgpack");

    initialize_runtime_spine(
        &store,
        RuntimeSpineInitOptions {
            runtime_id: "epiphany-hands-action-smoke".to_string(),
            display_name: "Epiphany Hands Action Smoke".to_string(),
            created_at: "2026-06-02T00:00:00Z".to_string(),
        },
    )?;

    let intent = HandsActionIntent {
        schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
        intent_id: "hands-intent-smoke".to_string(),
        runtime_job_id: "job-implementation-smoke".to_string(),
        binding_id: "implementation-worker".to_string(),
        role: "epiphany-hands".to_string(),
        authority_scope: "epiphany.role.implementation".to_string(),
        requested_action: "patch".to_string(),
        requested_paths: vec!["tools/epiphany_local_run.ps1".to_string()],
        substrate_gate_grant_receipt_id: "substrate-grant-smoke".to_string(),
        requested_at: "2026-06-02T00:00:10Z".to_string(),
        contract: "Smoke intent proves Hands action starts as typed runtime-spine state."
            .to_string(),
        frontier_route_id: String::new(),
        plan_candidate_sha256: String::new(),
        plan_action: String::new(),
    };
    put_hands_action_intent(&store, &intent)?;

    let review = hands_action_review_for_intent(
        "hands-review-smoke".to_string(),
        &intent,
        "approved".to_string(),
        vec![
            "patch".to_string(),
            "command".to_string(),
            "commit".to_string(),
            "pr".to_string(),
        ],
        vec![
            "Substrate Gate grant id is named; this smoke does not execute the patch.".to_string(),
        ],
        "2026-06-02T00:00:20Z".to_string(),
    );
    put_hands_action_review(&store, &review)?;

    let patch = hands_patch_receipt_for_review(
        "hands-patch-smoke".to_string(),
        &intent,
        &review,
        vec!["tools/epiphany_local_run.ps1".to_string()],
        "Recorded a typed patch receipt for the bounded Hands action path.".to_string(),
        "2026-06-02T00:00:30Z".to_string(),
    );
    put_hands_patch_receipt(&store, &patch)?;

    let command = hands_command_receipt_for_review(
        "hands-command-smoke".to_string(),
        &intent,
        &review,
        "cargo test --manifest-path .\\epiphany-core\\Cargo.toml hands_gateway".to_string(),
        "0".to_string(),
        ".epiphany-smoke/runtime-spine/hands-action-smoke.stdout.log".to_string(),
        ".epiphany-smoke/runtime-spine/hands-action-smoke.stderr.log".to_string(),
        "Recorded a typed command receipt for the bounded Hands action path.".to_string(),
        "2026-06-02T00:00:40Z".to_string(),
    );
    put_hands_command_receipt(&store, &command)?;

    let commit = hands_commit_receipt_for_review(
        "hands-commit-smoke".to_string(),
        &intent,
        &review,
        "dry-run-no-commit".to_string(),
        "main".to_string(),
        vec!["tools/epiphany_local_run.ps1".to_string()],
        "Recorded a typed commit receipt shape for the bounded Hands action path.".to_string(),
        "2026-06-02T00:00:50Z".to_string(),
    );
    put_hands_commit_receipt(&store, &commit)?;

    let stored_intent =
        runtime_hands_action_intent(&store, "hands-intent-smoke")?.expect("stored Hands intent");
    let stored_review =
        runtime_hands_action_review(&store, "hands-review-smoke")?.expect("stored Hands review");
    let stored_patch =
        runtime_hands_patch_receipt(&store, "hands-patch-smoke")?.expect("stored Hands patch");
    let stored_command = runtime_hands_command_receipt(&store, "hands-command-smoke")?
        .expect("stored Hands command");
    let stored_commit =
        runtime_hands_commit_receipt(&store, "hands-commit-smoke")?.expect("stored Hands commit");

    if stored_review.intent_id != stored_intent.intent_id {
        anyhow::bail!("Hands review lost its intent edge");
    }
    if stored_patch.intent_id != stored_intent.intent_id
        || stored_patch.review_id != stored_review.review_id
    {
        anyhow::bail!("Hands patch receipt lost its intent/review edge");
    }
    if stored_command.intent_id != stored_intent.intent_id
        || stored_command.review_id != stored_review.review_id
    {
        anyhow::bail!("Hands command receipt lost its intent/review edge");
    }
    if stored_commit.intent_id != stored_intent.intent_id
        || stored_commit.review_id != stored_review.review_id
    {
        anyhow::bail!("Hands commit receipt lost its intent/review edge");
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "ok",
            "store": store,
            "intentId": stored_intent.intent_id,
            "reviewId": stored_review.review_id,
            "patchReceiptId": stored_patch.receipt_id,
            "commandReceiptId": stored_command.receipt_id,
            "commitReceiptId": stored_commit.receipt_id,
            "changedPaths": stored_patch.changed_paths,
        }))?
    );

    Ok(())
}
