use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use epiphany_core::EpiphanyAgentMemoryEntry;
use epiphany_core::GhostlightAgent;
use epiphany_core::GhostlightCanonicalState;
use epiphany_core::GhostlightIdentity;
use epiphany_core::GhostlightMemories;
use epiphany_core::GhostlightMemory;
use epiphany_core::GhostlightValue;
use epiphany_core::GhostlightWorld;
use epiphany_core::PersonaIdentity;
use epiphany_core::PersonaInterpreterInput;
use epiphany_core::PersonaMemoryCacheConfig;
use epiphany_core::PersonaProjectorInput;
use epiphany_core::PersonaTurnInput;
use epiphany_core::WeksaInterlinguaInput;
use epiphany_core::WeksaSpeakerContext;
use epiphany_core::build_persona_interpreter_prompt;
use epiphany_core::build_persona_memory_chunks;
use epiphany_core::build_persona_projector_prompt;
use epiphany_core::build_persona_turn_prompt;
use epiphany_core::build_weksa_interlingua_packet;
use epiphany_core::build_weksa_target_lowering_request;
use epiphany_core::record_weksa_target_lowering_receipt;
use epiphany_core::render_dynamic_persona_memory_recall_for_output;
use epiphany_core::render_persona_memory_recall_with_cache;
use epiphany_core::render_persona_semantic_memory_recall;
use epiphany_core::semantic_memory_recall_from_heartbeat_action;
use epiphany_state_model::EpiphanyMemoryContextPacket;
use epiphany_state_model::EpiphanyMemoryFreshnessStatus;
use epiphany_state_model::EpiphanyMemoryNode;
use epiphany_state_model::EpiphanyMemorySummary;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct Args {
    root: PathBuf,
    smoke_root: PathBuf,
    fresh_repo_summary: Option<PathBuf>,
    readiness_summary: Option<PathBuf>,
    bifrost_accounting_summary: Option<PathBuf>,
    daemon_survival_summary: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut smoke_root = root.join(".epiphany-smoke");
        let mut fresh_repo_summary = None;
        let mut readiness_summary = None;
        let mut bifrost_accounting_summary = None;
        let mut daemon_survival_summary = None;
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                "--smoke-root" => smoke_root = take_path(&mut args, "--smoke-root")?,
                "--fresh-repo-summary" => {
                    fresh_repo_summary = Some(take_path(&mut args, "--fresh-repo-summary")?)
                }
                "--readiness-summary" => {
                    readiness_summary = Some(take_path(&mut args, "--readiness-summary")?)
                }
                "--bifrost-accounting-summary" => {
                    bifrost_accounting_summary =
                        Some(take_path(&mut args, "--bifrost-accounting-summary")?)
                }
                "--daemon-survival-summary" => {
                    daemon_survival_summary =
                        Some(take_path(&mut args, "--daemon-survival-summary")?)
                }
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self {
            root,
            smoke_root,
            fresh_repo_summary,
            readiness_summary,
            bifrost_accounting_summary,
            daemon_survival_summary,
        })
    }
}

fn run_smoke(args: Args) -> Result<Value> {
    let root = args
        .root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.root.display()))?;
    let manifest = root.join("epiphany-core").join("Cargo.toml");
    if !manifest.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest.display()
        ));
    }
    fs::create_dir_all(&args.smoke_root)
        .with_context(|| format!("failed to create {}", args.smoke_root.display()))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let smoke_dir = args.smoke_root.join(format!("repo-swarm-mvp-gate-{stamp}"));
    if smoke_dir.exists() {
        fs::remove_dir_all(&smoke_dir)
            .with_context(|| format!("failed to clear {}", smoke_dir.display()))?;
    }
    fs::create_dir_all(&smoke_dir)
        .with_context(|| format!("failed to create {}", smoke_dir.display()))?;

    let fresh_repo_summary = args
        .fresh_repo_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "fresh-repo-mvp-"));
    let readiness_summary = args
        .readiness_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "repo-work-readiness-"));
    let bifrost_accounting_summary = args
        .bifrost_accounting_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "repo-bifrost-accounting-bundle-"));
    let daemon_survival_summary = args
        .daemon_survival_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "daemon-survival-rehearsal-"));

    let fresh = read_json(&fresh_repo_summary)?;
    let readiness = read_json(&readiness_summary)?;
    let bifrost = read_json(&bifrost_accounting_summary)?;
    let daemon_survival = read_json(&daemon_survival_summary)?;

    verify_fresh_repo(&fresh)?;
    verify_readiness(&readiness)?;
    verify_bifrost_accounting(&bifrost)?;
    verify_daemon_survival(&daemon_survival)?;
    let weksa_interlingua = verify_weksa_interlingua()?;
    let persona_memory_recall = verify_persona_memory_recall()?;

    let gate_rows = vec![
        green(
            "fresh-repo-body",
            "Hands",
            "epiphany-repo init + branch + commit receipt",
        ),
        green("swarm-online", "Self", "epiphany-swarm online"),
        green("persona-intake", "Persona", "persona-intake work item"),
        green(
            "weksa-interlingua",
            "Weksa",
            "Persona SAY meaning lowers through Weksa interlingua packet/request/receipt without transport authority",
        ),
        green(
            "persona-memory-recall",
            "Persona/Mind",
            "Qdrant-backed Persona memory cache and typed graph fallback feed Projector, Persona, and output-triggered Interpreter prompts without exposing private state",
        ),
        green(
            "imagination-plan",
            "Imagination",
            "model-authored repo-status-section plan",
        ),
        green(
            "self-scheduler-stop",
            "Self",
            "iteration-limit stop classification is explicit and nonmutating",
        ),
        green(
            "soul-closure",
            "Soul",
            "closed item with passed verdict and family assertions",
        ),
        green(
            "upstream-main-sync",
            "Hands",
            "publish + git push/fetch + sync reports upstream main synced",
        ),
        green(
            "public-proof",
            "Bifrost",
            "operator-safe public proof exported without private state",
        ),
        green(
            "readiness-review",
            "Soul",
            "readiness report and review receipt are present",
        ),
        green(
            "receipt-directory",
            "Bifrost",
            "readiness review is queryable through the receipt directory",
        ),
        green(
            "bifrost-accounting",
            "Bifrost",
            "readiness, artifact acceptance, and metrics accounting rows are closed",
        ),
        green(
            "long-running-daemon-proof",
            "Idunn",
            "bounded serve rehearsal wrote two scheduler pulses and sealed scheduler receipt",
        ),
        warning(
            "idunn-elevated-service",
            "Idunn",
            "service install/start/remote deployment execution remains explicit operator authority",
        ),
    ];
    let green_gate_count = gate_rows
        .iter()
        .filter(|row| row["status"] == "green")
        .count();
    let warning_gate_count = gate_rows
        .iter()
        .filter(|row| row["status"] == "warning")
        .count();
    let blocker_gate_count = gate_rows
        .iter()
        .filter(|row| row["status"] == "blocker")
        .count();
    let gate_tui_rows = gate_rows
        .iter()
        .map(|row| {
            format!(
                "MVP-GATE | {} | status={} | owner={} | evidence={}",
                row["gate"].as_str().unwrap_or("unknown"),
                row["status"].as_str().unwrap_or("unknown"),
                row["owner"].as_str().unwrap_or("unknown"),
                row["evidence"].as_str().unwrap_or("unknown")
            )
        })
        .collect::<Vec<_>>();

    let summary = json!({
        "schemaVersion": "epiphany.repo_swarm_mvp_gate_smoke.v0",
        "status": "mvp-demo-ready-with-known-operator-gates",
        "smokeDir": smoke_dir,
        "freshRepoSummary": fresh_repo_summary,
        "readinessSummary": readiness_summary,
        "bifrostAccountingSummary": bifrost_accounting_summary,
        "daemonSurvivalSummary": daemon_survival_summary,
        "freshRepoSmokeDir": fresh["smokeDir"],
        "readinessSmokeDir": readiness["smokeDir"],
        "bifrostAccountingSmokeDir": bifrost["smokeDir"],
        "daemonSurvivalSmokeDir": daemon_survival["smokeDir"],
        "weksaInterlingua": weksa_interlingua,
        "personaMemoryRecall": persona_memory_recall,
        "mvpReady": false,
        "demoReady": true,
        "greenGateCount": green_gate_count,
        "warningGateCount": warning_gate_count,
        "blockerGateCount": blocker_gate_count,
        "gateRows": gate_rows,
        "gateTuiRows": gate_tui_rows,
        "knownRemainingAuthorityGates": [
            "Idunn elevated service install/start/remote deployment execution"
        ],
        "privateStateExposed": false,
    });
    write_json(&smoke_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

fn verify_fresh_repo(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.repo_swarm_fresh_repo_mvp_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_eq(value, &["onlineStatus"], "attention")?;
    require_eq(
        value,
        &["personaIntakeStatus"],
        "accepted-for-imagination-consensus",
    )?;
    require_eq(value, &["planStatus"], "planned-for-self-adoption")?;
    require_eq(value, &["swarmRunStopCategory"], "iteration-limit")?;
    require_eq(value, &["swarmRunStopOwner"], "Self")?;
    require_eq(
        value,
        &["swarmRunStopGate"],
        "self.scheduler-iteration-limit",
    )?;
    require_eq(value, &["closeStatus"], "closed")?;
    require_eq(value, &["soulVerdict"], "passed")?;
    require_eq(value, &["familyAssertionsStatus"], "passed")?;
    require_eq(value, &["publishStatus"], "publication-receipts-recorded")?;
    require_eq(value, &["syncStatus"], "upstream-main-synced")?;
    require_non_empty(value, &["personaMemoryRecallStatus"])?;
    require_non_empty(value, &["personaMemoryRecallCacheStatus"])?;
    require_non_empty(value, &["personaInterpreterDynamicRecallStatus"])?;
    require_non_empty(value, &["personaInterpreterDynamicRecallCacheStatus"])?;
    require_bool(value, &["upstreamMainSynced"], true)?;
    require_eq(value, &["publicProofStatus"], "public-proof-exported")?;
    require_bool(value, &["privateStateExposed"], false)
}

fn verify_readiness(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.repo_work_readiness_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_eq(value, &["readinessStatus"], "ready")?;
    require_eq(value, &["readinessReviewStatus"], "readiness-approved")?;
    require_eq(value, &["publishStatus"], "publication-receipts-recorded")?;
    require_eq(value, &["syncStatus"], "upstream-main-synced")?;
    require_bool(value, &["upstreamMainSynced"], true)?;
    require_bool(value, &["privateStateExposed"], false)?;
    require_bool(value, &["sightOnly"], true)?;
    require_bool(value, &["readinessApprovalAuthorized"], false)?;
    require_bool(value, &["publicationAuthorized"], false)?;
    require_bool(value, &["deploymentAuthority"], false)?;
    require_bool(value, &["serviceLifecycleAuthority"], false)?;
    require_bool(value, &["handsActionAuthorized"], false)?;
    require_row(
        value,
        &["bifrostReadinessReviewAccountingRow"],
        "repo-work-readiness-review",
        "closed",
        4,
        1,
    )
}

fn verify_bifrost_accounting(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.repo_bifrost_accounting_bundle_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_eq(value, &["artifactAcceptanceCloseStatus"], "closed")?;
    require_eq(value, &["metricsCloseStatus"], "closed")?;
    require_bool(value, &["operatorSafeProofBundle"], true)?;
    require_bool(value, &["planningAuthorityOnly"], true)?;
    require_bool(value, &["privateStateExposed"], false)?;
    require_row(
        value,
        &["artifactAcceptanceClosedAccountingRow"],
        "artifact-acceptance-request",
        "closed",
        1,
        1,
    )?;
    require_row(
        value,
        &["metricsClosedAccountingRow"],
        "metrics-request",
        "closed",
        2,
        1,
    )
}

fn verify_daemon_survival(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.daemon_survival_rehearsal_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_eq(value, &["policyStatus"], "written")?;
    require_eq(value, &["serveStatus"], "serveComplete")?;
    require_eq(value, &["schedulerReceiptStatus"], "tickComplete")?;
    require_bool(value, &["boundedProofMode"], true)?;
    require_bool(value, &["serviceManagerMutated"], false)?;
    require_bool(value, &["requiresElevatedAuthority"], false)?;
    require_bool(value, &["unattendedDaemonSurvivalRehearsed"], true)?;
    require_bool(value, &["privateStateExposed"], false)
}

fn verify_weksa_interlingua() -> Result<Value> {
    let packet = build_weksa_interlingua_packet(WeksaInterlinguaInput {
        packet_id: "repo-swarm-mvp-gate-weksa-packet".to_string(),
        source_interpreter_ref: "persona-interpreter:repo-swarm-mvp-gate".to_string(),
        source_speech_audit_ref: "persona-speech-audit:repo-swarm-mvp-gate".to_string(),
        speaker: WeksaSpeakerContext {
            persona_id: "epiphany.Persona".to_string(),
            display_name: "Epiphany".to_string(),
            source_surface: "eve://epiphany/persona".to_string(),
            source_language: "en".to_string(),
            utterance_state_ref: "state/agents.msgpack#Persona:utterance-state".to_string(),
        },
        meaning:
            "Tell a public repo room that Epiphany can keep branch-local work moving, while publication and merge remain Bifrost-gated."
                .to_string(),
        speech_act: "status-reply".to_string(),
        delivery_register: "warm-technical".to_string(),
        target_audience: "repo-public-room".to_string(),
        safety_notes: vec![
            "Do not claim publication, merge, deployment, or service lifecycle authority.".to_string(),
            "Do not expose private worker thought or private Verse payloads.".to_string(),
        ],
    })?;
    if packet.private_state_exposed {
        return Err(anyhow!("Weksa packet exposed private state"));
    }
    let request = build_weksa_target_lowering_request(
        "repo-swarm-mvp-gate-weksa-lowering-request",
        packet,
        "es",
        "warm-technical",
        "eve-public-room",
    )?;
    if request.private_state_exposed || !request.model_required {
        return Err(anyhow!(
            "Weksa lowering request lost model-required or private-state seal"
        ));
    }
    let receipt = record_weksa_target_lowering_receipt(
        &request,
        "repo-swarm-mvp-gate-weksa-lowering-receipt",
        "Epiphany puede mantener el trabajo local de rama en movimiento, mientras la publicacion y la fusion siguen bajo la puerta de Bifrost.",
        "deterministic-mvp-gate-smoke",
    )?;
    if receipt.private_state_exposed || !receipt.transport_authority.contains("must publish") {
        return Err(anyhow!(
            "Weksa lowering receipt lost private-state seal or transport boundary"
        ));
    }
    Ok(json!({
        "packetSchema": request.packet.schema_version,
        "requestSchema": request.schema_version,
        "receiptSchema": receipt.schema_version,
        "targetLanguage": receipt.target_language,
        "deliverySurface": receipt.delivery_surface,
        "modelRequired": request.model_required,
        "transportAuthority": "none",
        "publicationAuthority": false,
        "privateStateExposed": false,
    }))
}

fn verify_persona_memory_recall() -> Result<Value> {
    let fallback_recall = render_persona_semantic_memory_recall(&EpiphanyMemoryContextPacket {
        id: "memctx-repo-swarm-mvp-gate-persona".to_string(),
        query_id: "repo-swarm-mvp-gate-persona-current-turn".to_string(),
        summaries: vec![EpiphanyMemorySummary {
            id: "summary-repo-swarm-mvp-gate-persona-contracts".to_string(),
            target: "role:Persona".to_string(),
            claim:
                "Persona remembers that public speech is a reviewed mouth edge, not raw side effect authority."
                    .to_string(),
            action_implication:
                "Shape public voice, then let Mind, Bifrost, and the mouth edge route effects."
                    .to_string(),
            freshness: EpiphanyMemoryFreshnessStatus::Ready,
            confidence: 84,
            ..Default::default()
        }],
        nodes: vec![EpiphanyMemoryNode {
            id: "node-repo-swarm-mvp-gate-persona-qdrant-pressure".to_string(),
            title: "Persona memory retrieval pressure".to_string(),
            claim: "VoidBot rebuilds semantic Persona recall from typed memory before each Face turn."
                .to_string(),
            action_implication:
                "Epiphany Persona prompts must receive derived memory recall as hints before speech."
                    .to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });

    let memory_entry = persona_memory_entry();
    let chunks = build_persona_memory_chunks(&memory_entry, "state/agents.msgpack#Persona");
    if !chunks
        .iter()
        .any(|chunk| chunk.text.contains("public typed-contract zeal"))
    {
        return Err(anyhow!(
            "Persona memory chunks did not include public identity memory"
        ));
    }
    if chunks
        .iter()
        .any(|chunk| chunk.text.contains("sealed private note"))
    {
        return Err(anyhow!(
            "Persona memory chunks exposed sealed private notes"
        ));
    }

    let bridge = render_persona_memory_recall_with_cache(
        &memory_entry,
        "state/agents.msgpack#Persona",
        "typed-contract zeal before public speech",
        4,
        Some(&EpiphanyMemoryContextPacket {
            id: "memctx-repo-swarm-mvp-gate-fallback".to_string(),
            query_id: "repo-swarm-mvp-gate-persona-fallback".to_string(),
            summaries: vec![EpiphanyMemorySummary {
                id: "summary-repo-swarm-mvp-gate-fallback".to_string(),
                target: "role:Persona".to_string(),
                claim: fallback_recall,
                action_implication:
                    "This fallback is heartbeat-carried context, not direct state authority."
                        .to_string(),
                freshness: EpiphanyMemoryFreshnessStatus::Ready,
                confidence: 80,
                ..Default::default()
            }],
            nodes: vec![EpiphanyMemoryNode {
                id: "node-repo-swarm-mvp-gate-fallback".to_string(),
                title: "Smoke fallback memory".to_string(),
                claim: "Fallback typed memory graph context remains available.".to_string(),
                action_implication: "Do not pretend live Qdrant was required for this smoke."
                    .to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        &PersonaMemoryCacheConfig {
            qdrant_url: "http://127.0.0.1:1".to_string(),
            qdrant_api_key: None,
            qdrant_timeout_ms: 1,
            ollama_base_url: "http://127.0.0.1:1".to_string(),
            ollama_model: "qwen3-embedding:0.6b".to_string(),
            ollama_timeout_ms: 1,
            collection_name: "epiphany_persona_memory_mvp_gate".to_string(),
            query_instruction: "repo-swarm-mvp-gate".to_string(),
        },
    );
    if bridge.status != "fallback" {
        return Err(anyhow!(
            "expected Persona memory recall fallback status, got {:?}",
            bridge.status
        ));
    }
    if bridge.private_state_exposed || bridge.rendered_recall.contains("sealed private note") {
        return Err(anyhow!("Persona memory recall exposed private state"));
    }
    require_contains(
        &bridge.rendered_recall,
        "Fallback typed memory graph context",
        "Persona memory recall fallback",
    )?;

    let heartbeat_action = json!({
        "action_type": "persona_turn",
        "persona_memory_recall": {
            "privateStateExposed": false,
            "renderedRecall": bridge.rendered_recall,
        }
    });
    let recall = semantic_memory_recall_from_heartbeat_action(&heartbeat_action);
    require_contains(
        &recall,
        "Fallback typed memory graph context",
        "heartbeat Persona recall",
    )?;

    let identity = PersonaIdentity {
        identity_id: "epiphany".to_string(),
        display_name: "Epiphany".to_string(),
        repo_name: "EpiphanyAgent".to_string(),
        public_description: "Repo Persona for typed agent substrate.".to_string(),
        jurisdiction: vec!["typed state and review-gated agency".to_string()],
    };

    let projector_prompt = build_persona_projector_prompt(&PersonaProjectorInput {
        identity: identity.clone(),
        semantic_memory_recall: recall.clone(),
        ..Default::default()
    });
    require_contains(
        &projector_prompt,
        "Semantic memory recall",
        "Persona Projector prompt",
    )?;
    require_contains(
        &projector_prompt,
        "typed memory graph",
        "Persona Projector prompt",
    )?;
    require_contains(
        &projector_prompt,
        "not durable authority",
        "Persona Projector prompt",
    )?;

    let persona_prompt = build_persona_turn_prompt(&PersonaTurnInput {
        identity: identity.clone(),
        projected_state: "Epiphany feels the mouth edge as a public contract, not a vent."
            .to_string(),
        semantic_memory_recall: recall.clone(),
        ..Default::default()
    });
    require_contains(&persona_prompt, "Semantic memory recall", "Persona prompt")?;
    require_contains(
        &persona_prompt,
        "Fallback typed memory graph context",
        "Persona prompt",
    )?;

    let persona_output = "I can speak, but the effect needs a receipt.";
    let dynamic = render_dynamic_persona_memory_recall_for_output(
        &memory_entry,
        "state/agents.msgpack#Persona",
        &persona_prompt,
        persona_output,
        &recall,
        4,
        Some(&EpiphanyMemoryContextPacket {
            id: "memctx-repo-swarm-mvp-gate-dynamic-fallback".to_string(),
            query_id: "repo-swarm-mvp-gate-persona-dynamic-output".to_string(),
            summaries: vec![EpiphanyMemorySummary {
                id: "summary-repo-swarm-mvp-gate-dynamic-fallback".to_string(),
                target: "role:Persona".to_string(),
                claim: "Dynamic self-memory recall should inspect the Persona output before Mind interprets side effects."
                    .to_string(),
                action_implication:
                    "Interpreter should see output-triggered recall, not only the pre-turn prompt recall."
                        .to_string(),
                freshness: EpiphanyMemoryFreshnessStatus::Ready,
                confidence: 82,
                ..Default::default()
            }],
            ..Default::default()
        }),
        &PersonaMemoryCacheConfig {
            qdrant_url: "http://127.0.0.1:1".to_string(),
            qdrant_api_key: None,
            qdrant_timeout_ms: 1,
            ollama_base_url: "http://127.0.0.1:1".to_string(),
            ollama_model: "qwen3-embedding:0.6b".to_string(),
            ollama_timeout_ms: 1,
            collection_name: "epiphany_persona_memory_mvp_gate_dynamic".to_string(),
            query_instruction: "repo-swarm-mvp-gate-dynamic".to_string(),
        },
    );
    require_contains(
        &dynamic.rendered_recall,
        "Dynamic self-memory recall should inspect the Persona output",
        "dynamic Persona memory recall",
    )?;
    if dynamic.private_state_exposed || dynamic.rendered_recall.contains("sealed private note") {
        return Err(anyhow!(
            "dynamic Persona memory recall exposed private state"
        ));
    }

    let interpreter_prompt = build_persona_interpreter_prompt(&PersonaInterpreterInput {
        identity,
        persona_prompt,
        persona_output: persona_output.to_string(),
        semantic_memory_recall: recall,
        dynamic_semantic_memory_recall: dynamic.rendered_recall,
        pending_mentions: Vec::new(),
        allowed_channel_ids: vec!["aquarium".to_string()],
    });
    require_contains(
        &interpreter_prompt,
        "Dynamic self-memory recall",
        "Persona Interpreter prompt",
    )?;
    require_contains(
        &interpreter_prompt,
        "output-triggered recall",
        "Persona Interpreter prompt",
    )?;
    require_contains(
        &interpreter_prompt,
        "STATE NOTE",
        "Persona Interpreter prompt",
    )?;
    require_contains(&interpreter_prompt, "SAY", "Persona Interpreter prompt")?;

    Ok(json!({
        "schemaVersion": "epiphany.repo_swarm_mvp_persona_memory_recall_gate.v0",
        "status": bridge.status,
        "cacheStatus": bridge.cache_status,
        "dynamicCacheStatus": dynamic.cache_status,
        "chunkCount": bridge.chunk_count,
        "hitCount": bridge.hit_count,
        "dynamicHitCount": dynamic.hit_count,
        "fallbackContainsTypedGraph": true,
        "heartbeatRecallWired": true,
        "dynamicRecallWired": true,
        "personaLayers": ["projector", "persona", "interpreter"],
        "liveQdrantRequiredForSmoke": false,
        "privateStateExposed": false,
    }))
}

fn persona_memory_entry() -> EpiphanyAgentMemoryEntry {
    EpiphanyAgentMemoryEntry {
        schema_version: "ghostlight.agent_state.v0".to_string(),
        role_id: "Persona".to_string(),
        world: GhostlightWorld::default(),
        agent: GhostlightAgent {
            agent_id: "epiphany.Persona".to_string(),
            identity: GhostlightIdentity {
                name: "Epiphany".to_string(),
                roles: vec!["Persona".to_string()],
                origin: "EpiphanyAgent".to_string(),
                public_description: "public typed-contract zeal".to_string(),
                private_notes: vec!["sealed private note".to_string()],
            },
            memories: GhostlightMemories {
                semantic: vec![GhostlightMemory {
                    memory_id: "semantic-1".to_string(),
                    summary: "Persona recall should be semantically available before speech."
                        .to_string(),
                    salience: 0.9,
                    confidence: 0.9,
                    ..Default::default()
                }],
                ..Default::default()
            },
            canonical_state: GhostlightCanonicalState {
                values: vec![GhostlightValue {
                    value_id: "value-1".to_string(),
                    label: "Keep memory recall typed and sealed.".to_string(),
                    priority: 0.9,
                    unforgivable_if_betrayed: true,
                }],
                ..Default::default()
            },
            ..Default::default()
        },
        relationships: Vec::new(),
        events: Vec::new(),
        scenes: Vec::new(),
    }
}

fn green(gate: &str, owner: &str, evidence: &str) -> Value {
    json!({
        "gate": gate,
        "status": "green",
        "owner": owner,
        "evidence": evidence,
        "privateStateExposed": false,
    })
}

fn warning(gate: &str, owner: &str, evidence: &str) -> Value {
    json!({
        "gate": gate,
        "status": "warning",
        "owner": owner,
        "evidence": evidence,
        "privateStateExposed": false,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

fn latest_summary(smoke_root: &Path, parent_prefix: &str) -> PathBuf {
    let mut candidates = Vec::new();
    collect_summaries(smoke_root, parent_prefix, &mut candidates);
    candidates
        .into_iter()
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
        .unwrap_or_else(|| smoke_root.join(parent_prefix).join("summary.json"))
}

fn collect_summaries(root: &Path, parent_prefix: &str, summaries: &mut Vec<(u128, PathBuf)>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_summaries(&path, parent_prefix, summaries);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("summary.json")
            && path
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(parent_prefix))
        {
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis())
                .unwrap_or(0);
            summaries.push((modified, path));
        }
    }
}

fn require_eq(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected:?}, got {actual:?}",
            path.join(".")
        ))
    }
}

fn require_bool(value: &Value, path: &[&str], expected: bool) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_bool);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected}, got {:?}",
            path.join("."),
            actual
        ))
    }
}

fn require_non_empty(value: &Value, path: &[&str]) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_str)
        .unwrap_or("");
    if actual.trim().is_empty() {
        Err(anyhow!("expected {} to be non-empty", path.join(".")))
    } else {
        Ok(())
    }
}

fn require_contains(haystack: &str, needle: &str, label: &str) -> Result<()> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!("expected {label} to contain {needle:?}"))
    }
}

fn require_row(
    value: &Value,
    path: &[&str],
    lane: &str,
    status: &str,
    review_receipt_count: u64,
    public_artifact_count: u64,
) -> Result<()> {
    let row = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .ok_or_else(|| anyhow!("missing accounting row {}", path.join(".")))?;
    let actual_lane = row
        .get("lane")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let actual_status = row
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let actual_review_receipt_count = row
        .get("reviewReceiptCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let actual_public_artifact_count = row
        .get("publicArtifactCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let actual_private = row
        .get("privateStateExposed")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if actual_lane == lane
        && actual_status == status
        && actual_review_receipt_count == review_receipt_count
        && actual_public_artifact_count == public_artifact_count
        && !actual_private
    {
        Ok(())
    } else {
        Err(anyhow!(
            "accounting row {} mismatch: lane={actual_lane:?}, status={actual_status:?}, reviewReceiptCount={actual_review_receipt_count}, publicArtifactCount={actual_public_artifact_count}, privateStateExposed={actual_private}",
            path.join(".")
        ))
    }
}

fn take_path(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .with_context(|| format!("missing value for {flag}"))
}
