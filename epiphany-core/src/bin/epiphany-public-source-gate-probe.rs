use anyhow::{Context, Result, bail};
use epiphany_core::*;
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_tool_adapter::{
    EPIPHANY_TOOL_RUNTIME_ADAPTER_ID, EpiphanyToolInvocationIntent, EpiphanyToolInvocationReceipt,
    tool_invocation_intent_key, tool_invocation_receipt_key,
};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const WORKER_JOB_ID: &str = "public-source-no-grant-worker";
const MODEL_REQUEST_ID: &str = "public-source-no-grant-model-request";
const MODEL_JOB_ID: &str = "public-source-no-grant-model-job";
const MODEL_SESSION_ID: &str = "public-source-no-grant-model-session";
const HOSTILE_INTENT_ID: &str = "public-source-no-grant-intent";

fn main() -> Result<()> {
    let fixture_root = parse_fixture_root()?;
    if fixture_root.exists() {
        bail!(
            "public-source gate probe refuses existing fixture root {}",
            fixture_root.display()
        );
    }
    fs::create_dir_all(&fixture_root)
        .with_context(|| format!("creating fixture root {}", fixture_root.display()))?;
    let store = fixture_root.join("runtime.cc");
    let proof = prove_no_grant_refusal(&store)?;
    println!("{}", serde_json::to_string_pretty(&proof)?);
    Ok(())
}

fn parse_fixture_root() -> Result<PathBuf> {
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("no-grant")
        || args.next().as_deref() != Some("--fixture-root")
    {
        bail!("usage: epiphany-public-source-gate-probe no-grant --fixture-root NEW_PATH");
    }
    let root = args.next().context("missing --fixture-root value")?;
    if args.next().is_some() {
        bail!("unexpected trailing arguments");
    }
    let root = PathBuf::from(root);
    if !root.is_absolute() {
        bail!("--fixture-root must be absolute");
    }
    Ok(root)
}

fn prove_no_grant_refusal(store: &Path) -> Result<serde_json::Value> {
    initialize_runtime_spine(
        store,
        RuntimeSpineInitOptions {
            runtime_id: "public-source-no-grant-probe".into(),
            display_name: "Public source no-grant probe".into(),
            created_at: "2026-08-13T00:00:00Z".into(),
        },
    )?;
    let state = EpiphanyThreadState::default();
    let launch = build_epiphany_role_launch_request(
        "public-source-no-grant-thread",
        EpiphanyRoleResultRoleId::Research,
        Some(state.revision),
        Some(60),
        &state,
    )
    .map_err(anyhow::Error::msg)?;
    let plan = plan_coordinator_job_launch(
        &state,
        &launch,
        store,
        "public-source-no-grant-launcher".into(),
        WORKER_JOB_ID.into(),
    )?;
    commit_coordinator_job_launch(
        store,
        "public-source-no-grant-thread",
        &state,
        &launch,
        &plan,
        "2026-08-13T00:00:01Z".into(),
    )?;

    let mut model_request = EpiphanyModelRequest::new(
        MODEL_REQUEST_ID,
        "public-source-no-grant-conversation",
        "openai-codex",
        "gpt-probe",
        "Attempt one immutable public source read.",
    );
    model_request.source_worker_job_id = Some(WORKER_JOB_ID.into());
    let provider_request = EpiphanyOpenAiModelRequest::new(
        MODEL_REQUEST_ID,
        "public-source-no-grant-conversation",
        "gpt-probe",
        "Attempt one immutable public source read.",
    );
    open_runtime_model_execution(
        store,
        RuntimeSpineSessionOptions {
            session_id: MODEL_SESSION_ID.into(),
            objective: "Prove refusal before public-source execution.".into(),
            created_at: "2026-08-13T00:00:02Z".into(),
            coordinator_note: "isolated packaged gate probe".into(),
        },
        RuntimeSpineJobOptions {
            job_id: MODEL_JOB_ID.into(),
            session_id: MODEL_SESSION_ID.into(),
            role: "openai-model-adapter".into(),
            created_at: "2026-08-13T00:00:02Z".into(),
            summary: "Bound hostile public-source model turn.".into(),
            artifact_refs: Vec::new(),
        },
        &model_request,
        &provider_request,
        "2026-08-13T00:00:02Z",
    )?;

    let grant_id = format!("substrate-grant-{WORKER_JOB_ID}");
    let mut cache = runtime_spine_cache(store)?;
    cache.pull_all_backing_stores()?;
    let mut grant = cache
        .get::<SubstrateGateRepoAccessGrantReceipt>(&grant_id)?
        .context("Research launch omitted its exact Substrate Gate grant")?;
    grant
        .granted_operations
        .retain(|operation| operation != "publicSourceRead");
    cache.put(&grant_id, &grant)?;

    let hostile = EpiphanyToolInvocationIntent::new(
        HOSTILE_INTENT_ID,
        EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
        "epiphany_public",
        "github_file",
        r#"{"owner":"GameCult","repository":"Epiphany","revision":"0123456789abcdef0123456789abcdef01234567","path":"README.md"}"#,
        "gate-probe",
        "Prove missing publicSourceRead refuses before adapter execution.",
        "2026-08-13T00:00:03Z",
    )
    .with_model_call("public-source-no-grant-call", MODEL_REQUEST_ID);
    let before = fs::read(store)?;
    let error = put_runtime_tool_execution_intent(
        store,
        MODEL_SESSION_ID,
        MODEL_JOB_ID,
        &hostile,
        "2026-08-13T00:00:03Z",
    )
    .expect_err("hostile public-source intent unexpectedly acquired authority");
    let after = fs::read(store)?;
    if before != after {
        bail!("no-grant refusal changed the canonical runtime store");
    }

    let mut reloaded = runtime_spine_cache(store)?;
    reloaded.pull_all_backing_stores()?;
    let binding_absent = reloaded
        .get::<EpiphanyRuntimeToolExecutionBinding>(HOSTILE_INTENT_ID)?
        .is_none();
    let intent_absent = reloaded
        .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(HOSTILE_INTENT_ID))?
        .is_none();
    let receipt_absent = reloaded
        .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(HOSTILE_INTENT_ID))?
        .is_none();
    if !(binding_absent && intent_absent && receipt_absent) {
        bail!("no-grant refusal persisted forbidden execution authority");
    }

    Ok(json!({
        "schemaVersion": "epiphany.public_source_no_grant_gate_proof.v0",
        "status": "passed",
        "fixtureStore": store,
        "requiredOperation": "publicSourceRead",
        "admissionOwner": "put_runtime_tool_execution_intent",
        "refusedBeforeAdapterExecution": true,
        "storeByteIdentical": true,
        "bindingAbsent": binding_absent,
        "intentAbsent": intent_absent,
        "receiptAbsent": receipt_absent,
        "error": format!("{error:#}"),
        "privateStateExposed": false
    }))
}
