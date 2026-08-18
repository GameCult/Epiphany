use anyhow::{Context, Result, bail};
use epiphany_core::*;
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_tool_adapter::{
    EPIPHANY_TOOL_RUNTIME_ADAPTER_ID, EpiphanyToolInvocationIntent, EpiphanyToolInvocationReceipt,
    tool_invocation_intent_key, tool_invocation_receipt_key,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const WORKER_JOB_ID: &str = "public-source-no-grant-worker";
const MODEL_REQUEST_ID: &str = "public-source-no-grant-model-request";
const MODEL_JOB_ID: &str = "public-source-no-grant-model-job";
const MODEL_SESSION_ID: &str = "public-source-no-grant-model-session";
const HOSTILE_INTENT_ID: &str = "public-source-no-grant-intent";

fn main() -> Result<()> {
    let (command, fixture_root) = parse_command()?;
    let proof = match command.as_str() {
        "admission-no-grant" => {
            let store = create_fixture_root(&fixture_root)?;
            prove_admission_no_grant_refusal(&store)?
        }
        "prepare-execution-no-grant" => {
            let store = create_fixture_root(&fixture_root)?;
            prepare_execution_no_grant(&fixture_root, &store)?
        }
        "verify-execution-no-grant" => verify_execution_no_grant(&fixture_root)?,
        _ => bail!(usage()),
    };
    println!("{}", serde_json::to_string_pretty(&proof)?);
    Ok(())
}

fn parse_command() -> Result<(String, PathBuf)> {
    let mut args = env::args().skip(1);
    let command = args.next().context("missing command")?;
    if args.next().as_deref() != Some("--fixture-root") {
        bail!(usage());
    }
    let root = args.next().context("missing --fixture-root value")?;
    if args.next().is_some() {
        bail!("unexpected trailing arguments");
    }
    let root = PathBuf::from(root);
    if !root.is_absolute() {
        bail!("--fixture-root must be absolute");
    }
    Ok((command, root))
}

fn usage() -> &'static str {
    "usage: epiphany-public-source-gate-probe <admission-no-grant|prepare-execution-no-grant|verify-execution-no-grant> --fixture-root PATH"
}

fn create_fixture_root(fixture_root: &Path) -> Result<PathBuf> {
    if fixture_root.exists() {
        bail!(
            "public-source gate probe refuses existing fixture root {}",
            fixture_root.display()
        );
    }
    fs::create_dir_all(fixture_root)
        .with_context(|| format!("creating fixture root {}", fixture_root.display()))?;
    Ok(fixture_root.join("runtime.cc"))
}

fn prepare_authority_family(store: &Path) -> Result<EpiphanyToolInvocationIntent> {
    initialize_runtime_spine(
        store,
        RuntimeSpineInitOptions {
            runtime_id: "public-source-no-grant-probe".into(),
            display_name: "Public source no-grant probe".into(),
            created_at: "2026-08-13T00:00:00Z".into(),
        },
    )?;
    let launch_document = EpiphanyWorkerLaunchDocument::Role(EpiphanyRoleWorkerLaunchDocument {
        thread_id: "public-source-no-grant-thread".into(),
        role_id: "research".into(),
        objective: Some("Prove the exact public-source grant boundary.".into()),
        dynamic_prompt_context: None,
        repository_body_observation_basis: None,
        proposal_modeling_context: None,
        frontier_verdict_modeling_context: None,
        frontier_planning_context: None,
        frontier_research_context: None,
        frontier_verification_context: None,
        frontier_plan_mind_context: None,
        imagination_consideration_context: None,
        admitted_model_direction_consideration_context: None,
        active_subgoal_id: None,
        active_subgoals: Vec::new(),
        active_graph_node_ids: Vec::new(),
        investigation_checkpoint: None,
        scratch: None,
        invariants: Vec::new(),
        graphs: None,
        recent_evidence: Vec::new(),
        recent_observations: Vec::new(),
        graph_frontier: None,
        graph_checkpoint: None,
        planning: None,
        churn: None,
    });
    let authority_scope = "epiphany.eyes.public_source_gate_probe";
    open_runtime_spine_heartbeat_job(
        store,
        RuntimeSpineHeartbeatJobOptions {
            runtime_id: "public-source-no-grant-probe".into(),
            display_name: "Public source no-grant probe".into(),
            session_id: EPIPHANY_RUNTIME_ROOT_SESSION_ID.into(),
            objective: "Prove the exact public-source grant boundary.".into(),
            coordinator_note: "isolated gate probe".into(),
            job_id: WORKER_JOB_ID.into(),
            role: EPIPHANY_RESEARCH_OWNER_ROLE.into(),
            binding_id: EPIPHANY_RESEARCH_ROLE_BINDING_ID.into(),
            authority_scope: authority_scope.into(),
            instruction: "Attempt one immutable public source lookup.".into(),
            output_contract_id: ROLE_WORKER_OUTPUT_CONTRACT_ID.into(),
            organ_launch_contract: default_launch_organ_contract(
                authority_scope,
                "role",
                ROLE_WORKER_OUTPUT_CONTRACT_ID,
            ),
            launch_document,
            proposal_modeling_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verification_request_id: None,
            created_at: "2026-08-13T00:00:01Z".into(),
        },
    )?;
    runtime_spine_cache(store)?.put(
        &format!("substrate-grant-{WORKER_JOB_ID}"),
        &substrate_gate_repo_access_grant_for_worker(
            format!("substrate-grant-{WORKER_JOB_ID}"),
            WORKER_JOB_ID.into(),
            EPIPHANY_RESEARCH_ROLE_BINDING_ID.into(),
            EPIPHANY_RESEARCH_OWNER_ROLE.into(),
            authority_scope.into(),
            true,
            "2026-08-13T00:00:01Z".into(),
        ),
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

    Ok(EpiphanyToolInvocationIntent::new(
        HOSTILE_INTENT_ID,
        EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
        "epiphany_public",
        "github_file",
        r#"{"owner":"GameCult","repository":"Epiphany","revision":"0123456789abcdef0123456789abcdef01234567","path":"README.md"}"#,
        "gate-probe",
        "Prove missing publicSourceRead refuses before adapter execution.",
        "2026-08-13T00:00:03Z",
    )
    .with_model_call("public-source-no-grant-call", MODEL_REQUEST_ID))
}

fn remove_public_source_grant(store: &Path) -> Result<()> {
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
    Ok(())
}

fn prove_admission_no_grant_refusal(store: &Path) -> Result<serde_json::Value> {
    let hostile = prepare_authority_family(store)?;
    remove_public_source_grant(store)?;
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
        "schemaVersion": "epiphany.public_source_admission_no_grant_gate_proof.v0",
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

fn prepare_execution_no_grant(fixture_root: &Path, store: &Path) -> Result<serde_json::Value> {
    let hostile = prepare_authority_family(store)?;
    put_runtime_tool_execution_intent(
        store,
        MODEL_SESSION_ID,
        MODEL_JOB_ID,
        &hostile,
        "2026-08-13T00:00:03Z",
    )?;
    remove_public_source_grant(store)?;
    let digest = sha256(&fs::read(store)?);
    fs::write(fixture_root.join("before.sha256"), format!("{digest}\n"))?;
    Ok(json!({
        "schemaVersion": "epiphany.public_source_execution_no_grant_gate_fixture.v0",
        "status": "prepared",
        "fixtureStore": store,
        "intentId": HOSTILE_INTENT_ID,
        "requiredOperation": "publicSourceRead",
        "storeSha256": digest,
        "expectedPackagedCommand": ["epiphany-tool-mcp-runtime", "run", "--store", store, "--intent-id", HOSTILE_INTENT_ID],
        "privateStateExposed": false
    }))
}

fn verify_execution_no_grant(fixture_root: &Path) -> Result<serde_json::Value> {
    let store = fixture_root.join("runtime.cc");
    let expected_digest = fs::read_to_string(fixture_root.join("before.sha256"))?
        .trim()
        .to_string();
    let actual_digest = sha256(&fs::read(&store)?);
    if actual_digest != expected_digest {
        bail!("packaged no-grant refusal changed the canonical runtime store");
    }
    let mut cache = runtime_spine_cache(&store)?;
    cache.pull_all_backing_stores()?;
    let binding_present = cache
        .get::<EpiphanyRuntimeToolExecutionBinding>(HOSTILE_INTENT_ID)?
        .is_some();
    let intent_present = cache
        .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(HOSTILE_INTENT_ID))?
        .is_some();
    let receipt_absent = cache
        .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(HOSTILE_INTENT_ID))?
        .is_none();
    if !(binding_present && intent_present && receipt_absent) {
        bail!("packaged no-grant refusal left an invalid execution family");
    }
    Ok(json!({
        "schemaVersion": "epiphany.public_source_execution_no_grant_gate_proof.v0",
        "status": "passed",
        "fixtureStore": store,
        "intentId": HOSTILE_INTENT_ID,
        "executionOwner": "require_runtime_tool_execution_binding",
        "storeByteIdentical": true,
        "bindingPresent": binding_present,
        "intentPresent": intent_present,
        "receiptAbsent": receipt_absent,
        "refusedBeforeAdapterExecution": true,
        "privateStateExposed": false
    }))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
