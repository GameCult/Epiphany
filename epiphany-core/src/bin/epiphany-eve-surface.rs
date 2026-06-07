use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_EYES_CONTRACT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_GLOBAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_HANDS_CONTRACT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_SOUL_CONTRACT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION;
use epiphany_core::EpiphanyLocalVerseContext;
use epiphany_core::GJALLAR_AFFORDANCE_SCHEMA_VERSION;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::seed_epiphany_local_verse_context;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

const EVE_PROVIDER_ADVERTISEMENT_SCHEMA: &str = "gamecult.eve.provider_advertisement.v1";
const EVE_SURFACE_SCHEMA: &str = "gamecult.eve.surface.v1";
const EVE_COMMAND_SCHEMA: &str = "gamecult.eve.command.v1";
const PROVIDER_ID: &str = "epiphany";
const SERVICE_ID: &str = "epiphany.agent";
const SURFACE_ID: &str = "epiphany.operator.surface";

fn main() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "advertisement" => {
            let context = load_context(&args.store, &args.runtime_id)?;
            print_json(provider_advertisement(&args, context.as_ref()))?;
        }
        "surface" => {
            let context = load_context(&args.store, &args.runtime_id)?;
            print_json(surface_document(&args, context.as_ref()))?;
        }
        "bundle" => {
            let context = load_context(&args.store, &args.runtime_id)?;
            print_json(json!({
                "schema": "epiphany.eve_surface_export.v0",
                "providerAdvertisement": provider_advertisement(&args, context.as_ref()),
                "surface": surface_document(&args, context.as_ref()),
            }))?;
        }
        "smoke" => run_smoke(args)?,
        other => {
            anyhow::bail!("unknown command {other:?}; use advertisement, surface, bundle, or smoke")
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
    updated_at: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "bundle".to_string());
        let mut store = PathBuf::from(".epiphany-run/cultmesh/epiphany-local.ccmp");
        let mut runtime_id = "epiphany-local".to_string();
        let mut updated_at = Utc::now().to_rfc3339();

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => {
                    store = PathBuf::from(values.next().context("missing --store value")?);
                }
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?;
                }
                "--updated-at" => {
                    updated_at = values.next().context("missing --updated-at value")?;
                }
                _ => anyhow::bail!("unknown argument {arg:?}"),
            }
        }

        Ok(Self {
            command,
            store,
            runtime_id,
            updated_at,
        })
    }
}

fn load_context(store: &Path, runtime_id: &str) -> Result<Option<EpiphanyLocalVerseContext>> {
    if !store.exists() {
        return Ok(None);
    }
    query_epiphany_local_verse_context(store, runtime_id.to_string()).map(Some)
}

fn provider_advertisement(args: &Args, context: Option<&EpiphanyLocalVerseContext>) -> Value {
    let state = if context.is_some() {
        "fresh"
    } else {
        "degraded"
    };
    json!({
        "schema": EVE_PROVIDER_ADVERTISEMENT_SCHEMA,
        "providerId": PROVIDER_ID,
        "serviceId": SERVICE_ID,
        "verseId": EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID,
        "title": "Epiphany Operator",
        "kind": "service.operator",
        "updatedAt": args.updated_at,
        "freshness": {
            "state": state,
            "lastSeenAt": context
                .and_then(|context| context.operator_status.as_ref())
                .map(|status| status.generated_at_utc.as_str())
                .unwrap_or(args.updated_at.as_str()),
            "maxAgeMs": 15000,
            "note": if context.is_some() {
                "Advertisement is derived from typed Epiphany CultMesh state."
            } else {
                "CultMesh store was not found; advertisement lists intended read-only provider boundaries."
            }
        },
        "schemas": schema_catalog(context),
        "witnesses": witnesses(args, context),
        "surfaces": [{
            "surfaceId": SURFACE_ID,
            "schema": EVE_SURFACE_SCHEMA,
            "transport": "cultmesh-document",
            "key": "cultmesh://epiphany/operator/surface",
            "audience": "operator",
            "mode": "read-only",
            "styleProfile": "epiphany.operator",
            "commands": command_names(),
            "canonical": true
        }],
        "commands": commands(),
        "nestedVerses": nested_verses(context),
        "styleCapabilities": [{
            "styleProfile": "epiphany.operator",
            "tokenGroups": ["epiphany.authority", "epiphany.freshness", "epiphany.faculty"],
            "preferredLowerings": ["tui", "browser", "direct2d", "native-eve"],
            "lossiness": {
                "tui": "read-only dense operator panels; no graphical relationship map",
                "browser": "reference lowering until native Eve provider publication lands"
            }
        }],
        "contacts": [{
            "id": "huginn",
            "role": "Persona and .cc inspection runtime steward",
            "boundary": "Coordinates schema migration, projection health, CultMesh publication, and Eve DSL inspection; does not own Epiphany runtime truth."
        }],
        "stewardship": {
            "persona": {
                "schema": "gamecult.persona_state.v0",
                "owner": "public Persona or Persona projection",
                "runtimeSteward": "Huginn",
                "boundary": "Private work-organ state remains epiphany.work_organ_state.v0 or organ-specific typed state; Persona is not a dumping ground for runtime authority."
            },
            "jsonCompatibility": ".epiphany-run JSON is a compatibility witness only, never the owner of accepted Epiphany state."
        }
    })
}

fn surface_document(args: &Args, context: Option<&EpiphanyLocalVerseContext>) -> Value {
    json!({
        "type": "surface-state",
        "schema": EVE_SURFACE_SCHEMA,
        "providerId": PROVIDER_ID,
        "providerKind": "service.operator",
        "title": "Epiphany Operator Surface",
        "version": 1,
        "updatedAt": args.updated_at,
        "surface": {
            "root": {
                "id": "epiphany.operator.root",
                "kind": "surface",
                "props": {
                    "authority": "provider-owned-read-only",
                    "runtimeId": args.runtime_id,
                    "verseId": EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID
                },
                "children": [
                    operator_status_panel(context),
                    local_verse_panel(context),
                    daemon_affordance_panel(context),
                    runtime_spine_panel(context),
                    memory_graph_panel(context),
                    persona_panel()
                ]
            },
            "styles": {
                "tokens": {
                    "epiphany.authority.accepted": "#8fbf9f",
                    "epiphany.authority.witness": "#c8a24d",
                    "epiphany.authority.denied": "#bf6b6b"
                }
            }
        },
        "commands": commands()
    })
}

fn operator_status_panel(context: Option<&EpiphanyLocalVerseContext>) -> Value {
    let status = context.and_then(|context| context.operator_status.as_ref());
    json!({
        "id": "epiphany.operator.status",
        "kind": "inspector.kv",
        "props": {
            "title": "Operator Status",
            "authority": "Epiphany CultMesh operator status",
            "freshness": status.map(|status| status.generated_at_utc.as_str()).unwrap_or("missing"),
            "rows": [
                {"label": "runtime", "value": status.map(|status| status.runtime_id.as_str()).unwrap_or("missing")},
                {"label": "state", "value": status.map(|status| status.status.as_str()).unwrap_or("missing")},
                {"label": "bridge", "value": status.map(|status| status.codex_bridge_role.as_str()).unwrap_or("compatibility transport")},
                {"label": "authority", "value": status.map(|status| status.epiphany_authority_role.as_str()).unwrap_or("Epiphany owns runtime truth")},
                {"label": "prompt", "value": status.map(|status| status.prompt_authority.as_str()).unwrap_or("prompt authority is separated from runtime state")}
            ]
        },
        "children": []
    })
}

fn local_verse_panel(context: Option<&EpiphanyLocalVerseContext>) -> Value {
    let verse_rows: Vec<Value> = context
        .map(|context| {
            context
                .verse_policies
                .iter()
                .map(|policy| {
                    json!({
                        "label": policy.verse_id,
                        "value": format!("{}; private={}; untrustedIngress={}; yggdrasil={}", policy.tier, policy.private_state_allowed, policy.untrusted_ingress_allowed, policy.yggdrasil_tunnel_allowed)
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    json!({
        "id": "epiphany.local_verse.context",
        "kind": "inspector.kv",
        "props": {
            "title": "Local Verse Context",
            "authority": "CultMesh Verse policies and contract summaries",
            "freshness": context.map(|context| context.store_path.as_str()).unwrap_or("store missing"),
            "rows": verse_rows
        },
        "children": []
    })
}

fn daemon_affordance_panel(context: Option<&EpiphanyLocalVerseContext>) -> Value {
    let rows: Vec<Value> = context
        .map(|context| {
            context
                .daemon_affordances
                .iter()
                .take(12)
                .map(|affordance| {
                    json!({
                        "label": affordance.affordance_id,
                        "value": format!(
                            "{}:{}; authority={}; status={}; source={}",
                            affordance.surface_kind,
                            affordance.action,
                            affordance.authority,
                            affordance.status,
                            affordance.source_record
                        )
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    json!({
        "id": "epiphany.daemon.affordances",
        "kind": "inspector.kv",
        "props": {
            "title": "Daemon Affordances",
            "authority": "Gjallar affordance records derived from Odin-owned sight",
            "freshness": context
                .and_then(|context| context.daemon_affordances.first())
                .map(|affordance| affordance.observed_at.as_str())
                .unwrap_or("missing"),
            "rows": rows,
            "emptyState": "No Gjallar affordances loaded; import an Odin/Gjallar CultMesh store rather than probing sockets or calling MCP."
        },
        "children": []
    })
}

fn runtime_spine_panel(context: Option<&EpiphanyLocalVerseContext>) -> Value {
    let snapshot = context.and_then(|context| context.latest_operator_snapshot.as_ref());
    let intent = context.and_then(|context| context.latest_operator_run_intent.as_ref());
    let receipt = context.and_then(|context| context.latest_operator_run_receipt.as_ref());
    json!({
        "id": "epiphany.runtime.spine",
        "kind": "inspector.kv",
        "props": {
            "title": "Runtime Spine",
            "authority": "runtime-spine documents plus operator run intent/receipt projections",
            "rows": [
                {"label": "snapshot", "value": snapshot.map(|snapshot| snapshot.snapshot_id.as_str()).unwrap_or("missing")},
                {"label": "snapshotStatus", "value": snapshot.map(|snapshot| snapshot.status.as_str()).unwrap_or("missing")},
                {"label": "lastIntent", "value": intent.map(|intent| intent.run_id.as_str()).unwrap_or("missing")},
                {"label": "lastReceipt", "value": receipt.map(|receipt| receipt.run_id.as_str()).unwrap_or("missing")}
            ]
        },
        "children": []
    })
}

fn memory_graph_panel(context: Option<&EpiphanyLocalVerseContext>) -> Value {
    let graph_contracts = context
        .map(|context| {
            context
                .contract_summaries
                .iter()
                .filter(|contract| {
                    contract.document_type.contains("memory")
                        || contract.document_type.contains("state")
                        || contract.authority == "mind"
                })
                .count()
        })
        .unwrap_or(0);
    json!({
        "id": "epiphany.memory.graph",
        "kind": "inspector.kv",
        "props": {
            "title": "Memory Graph",
            "authority": "Mind and memory graph typed state; private graph details stay behind organ contracts",
            "rows": [
                {"label": "profileAvailability", "value": if graph_contracts > 0 { "contracted" } else { "not advertised in loaded store" }},
                {"label": "thinStateWarning", "value": "surface reports availability and freshness only; it does not leak private graph payloads"},
                {"label": "contractCount", "value": graph_contracts}
            ]
        },
        "children": []
    })
}

fn persona_panel() -> Value {
    json!({
        "id": "epiphany.persona.projection",
        "kind": "inspector.kv",
        "props": {
            "title": "Persona Projection",
            "authority": "gamecult.persona_state.v0 for public Persona/Persona projections only",
            "rows": [
                {"label": "schema", "value": "gamecult.persona_state.v0"},
                {"label": "steward", "value": "Huginn coordinates Persona-state and .cc inspection runtime stewardship"},
                {"label": "boundary", "value": "work organs stay in organ state; Persona does not own runtime truth"}
            ]
        },
        "children": []
    })
}

fn schema_catalog(context: Option<&EpiphanyLocalVerseContext>) -> Vec<Value> {
    let mut schemas = vec![
        schema_entry(
            EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION,
            "epiphany",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/operator-status",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION,
            "epiphany",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/operator-snapshot/latest",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION,
            "epiphany",
            "command-intent",
            "cultcache-cc",
            "cultmesh://epiphany/operator-run-intent/latest",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION,
            "epiphany",
            "receipt",
            "cultcache-cc",
            "cultmesh://epiphany/operator-run-receipt/latest",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION,
            "epiphany",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/verse-policy/{verseId}",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION,
            "epiphany",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/global-room/{roomId}",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION,
            "mind",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/contracts/mind/{contractId}",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_SCHEMA_VERSION,
            "substrateGate",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/contracts/substrate-gate/{contractId}",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_EYES_CONTRACT_SCHEMA_VERSION,
            "eyes",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/contracts/eyes/{contractId}",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_HANDS_CONTRACT_SCHEMA_VERSION,
            "hands",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/contracts/hands/{contractId}",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_SOUL_CONTRACT_SCHEMA_VERSION,
            "soul",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/contracts/soul/{contractId}",
            true,
        ),
        schema_entry(
            EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_SCHEMA_VERSION,
            "continuity",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/contracts/continuity/{contractId}",
            true,
        ),
        schema_entry(
            "epiphany.work_organ_state.v0",
            "epiphany",
            "accepted",
            "cultcache-cc",
            "cultmesh://epiphany/organs/{organId}/state",
            true,
        ),
        schema_entry(
            "gamecult.persona_state.v0",
            "public Persona owner; Huginn steward",
            "accepted-projection",
            "external-authority-projection",
            "cultmesh://huginn/persona/{personaId}",
            true,
        ),
        schema_entry(
            GJALLAR_AFFORDANCE_SCHEMA_VERSION,
            "gjallar",
            "imported-affordance",
            "cultcache-cc",
            "cultmesh://odin/gjallar/affordances",
            true,
        ),
    ];
    if let Some(context) = context {
        for contract in &context.contract_summaries {
            schemas.push(schema_entry(
                &contract.document_type,
                &contract.authority,
                "contracted",
                "cultcache-cc",
                &format!("cultmesh://epiphany/contracts/{}", contract.contract_id),
                true,
            ));
        }
    }
    schemas
}

fn schema_entry(
    schema: &str,
    owner: &str,
    authority: &str,
    storage: &str,
    cult_mesh_key: &str,
    portable: bool,
) -> Value {
    json!({
        "schema": schema,
        "owner": owner,
        "authority": authority,
        "storage": storage,
        "cultMeshKey": cult_mesh_key,
        "portable": portable
    })
}

fn witnesses(args: &Args, context: Option<&EpiphanyLocalVerseContext>) -> Vec<Value> {
    vec![
        json!({
            "id": "epiphany.local-verse.store",
            "kind": "ccmp-store",
            "path": args.store,
            "schemas": [
                EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION,
                EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION,
                EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION,
                GJALLAR_AFFORDANCE_SCHEMA_VERSION
            ],
            "redaction": "private worker transcripts and raw model payloads are not exposed",
            "freshness": {
                "state": if context.is_some() { "fresh" } else { "missing" },
                "updatedAt": context
                    .and_then(|context| context.operator_status.as_ref())
                    .map(|status| status.generated_at_utc.as_str())
                    .unwrap_or(args.updated_at.as_str())
            }
        }),
        json!({
            "id": "epiphany.run.compatibility",
            "kind": "json-compatibility-witness",
            "path": ".epiphany-run",
            "schemas": [EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION],
            "redaction": "operator-safe distillation only; JSON files are not accepted state owners",
            "freshness": {
                "state": "witness-only",
                "updatedAt": args.updated_at
            }
        }),
    ]
}

fn nested_verses(context: Option<&EpiphanyLocalVerseContext>) -> Vec<Value> {
    let mut verses = vec![
        nested_verse(
            EPIPHANY_CULTMESH_INTERNAL_VERSE_ID,
            EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID,
            "private-runtime",
            "Epiphany internal organs own private runtime truth; Eve receives operator-safe projection only.",
            vec![SURFACE_ID],
            vec![
                "epiphany.work_organ_state.v0",
                EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION,
            ],
        ),
        nested_verse(
            EPIPHANY_CULTMESH_GLOBAL_VERSE_ID,
            EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID,
            "public-persona",
            "Public Persona/Persona projection crosses this boundary through Persona state and Mind adoption gates.",
            vec![SURFACE_ID],
            vec!["gamecult.persona_state.v0"],
        ),
    ];

    if let Some(context) = context {
        for contract in &context.contract_summaries {
            let verse_id = format!("epiphany.faculty.{}", contract.authority);
            verses.push(nested_verse(
                &verse_id,
                contract.verse_id.as_str(),
                "faculty",
                &format!(
                    "{} owns {} operations; commands remain provider-accepted intent, not renderer callbacks.",
                    contract.authority, contract.document_type
                ),
                vec![SURFACE_ID],
                vec![contract.document_type.as_str()],
            ));
        }
    }
    verses
}

fn nested_verse(
    verse_id: &str,
    parent_verse_id: &str,
    kind: &str,
    authority_boundary: &str,
    surface_ids: Vec<&str>,
    state_schemas: Vec<&str>,
) -> Value {
    json!({
        "verseId": verse_id,
        "parentVerseId": parent_verse_id,
        "kind": kind,
        "authorityBoundary": authority_boundary,
        "surfaceIds": surface_ids,
        "stateSchemas": state_schemas,
        "carryRules": {
            "style": "inherit epiphany.operator tokens unless provider republishes",
            "identity": "Persona projections require gamecult.persona_state.v0 provenance",
            "commands": "provider-accepted-only"
        }
    })
}

fn commands() -> Vec<Value> {
    vec![
        command_entry(
            "epiphany.operator_run.request",
            "operator-or-coordinator",
            "accepted-denied-or-reconciled",
        ),
        command_entry(
            "epiphany.job_interrupt.request",
            "operator-or-coordinator",
            "accepted-denied-or-reconciled",
        ),
        command_entry(
            "epiphany.role_launch.request",
            "coordinator",
            "accepted-denied-or-reconciled",
        ),
        command_entry(
            "epiphany.persona_projection.request",
            "huginn-stewarded-persona-owner",
            "accepted-denied-or-reconciled",
        ),
    ]
}

fn command_entry(command: &str, authority: &str, result: &str) -> Value {
    json!({
        "command": command,
        "surfaceId": SURFACE_ID,
        "transport": "cultmesh-command",
        "schema": EVE_COMMAND_SCHEMA,
        "authority": authority,
        "result": result,
        "boundary": "This read-only export does not execute commands; Eve must send command intent through the provider's typed command boundary."
    })
}

fn command_names() -> Vec<&'static str> {
    vec![
        "epiphany.operator_run.request",
        "epiphany.job_interrupt.request",
        "epiphany.role_launch.request",
        "epiphany.persona_projection.request",
    ]
}

fn run_smoke(args: Args) -> Result<()> {
    let temp_root = env::temp_dir().join(format!(
        "epiphany-eve-surface-smoke-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    fs::create_dir_all(&temp_root)?;
    let store = temp_root.join("epiphany-local.ccmp");
    seed_epiphany_local_verse_context(&store, args.runtime_id.clone(), "2026-06-03T00:00:00Z")?;
    let smoke_args = Args {
        command: "bundle".to_string(),
        store: store.clone(),
        runtime_id: args.runtime_id,
        updated_at: "2026-06-03T00:00:00Z".to_string(),
    };
    let context = load_context(&store, &smoke_args.runtime_id)?
        .context("smoke failed to load seeded local Verse context")?;
    let advertisement = provider_advertisement(&smoke_args, Some(&context));
    let surface = surface_document(&smoke_args, Some(&context));

    require_eq(&advertisement, "/schema", EVE_PROVIDER_ADVERTISEMENT_SCHEMA)?;
    require_eq(&surface, "/schema", EVE_SURFACE_SCHEMA)?;
    require_contains(&advertisement, "gamecult.persona_state.v0")?;
    require_contains(&advertisement, "epiphany.work_organ_state.v0")?;
    require_contains(
        &advertisement,
        ".epiphany-run JSON is a compatibility witness only",
    )?;
    require_contains(&advertisement, "Huginn")?;
    require_contains(&surface, "Persona Projection")?;
    require_contains(&surface, "Operator Status")?;

    print_json(json!({
        "status": "ok",
        "store": store,
        "providerSchema": EVE_PROVIDER_ADVERTISEMENT_SCHEMA,
        "surfaceSchema": EVE_SURFACE_SCHEMA,
        "schemas": advertisement["schemas"].as_array().map(|schemas| schemas.len()).unwrap_or_default(),
        "nestedVerses": advertisement["nestedVerses"].as_array().map(|verses| verses.len()).unwrap_or_default(),
        "commands": advertisement["commands"].as_array().map(|commands| commands.len()).unwrap_or_default()
    }))
}

fn require_eq(value: &Value, pointer: &str, expected: &str) -> Result<()> {
    let actual = value
        .pointer(pointer)
        .and_then(Value::as_str)
        .with_context(|| format!("missing {pointer}"))?;
    if actual != expected {
        anyhow::bail!("{pointer} expected {expected:?}, got {actual:?}");
    }
    Ok(())
}

fn require_contains(value: &Value, needle: &str) -> Result<()> {
    let rendered = serde_json::to_string(value)?;
    if !rendered.contains(needle) {
        anyhow::bail!("export did not contain {needle:?}");
    }
    Ok(())
}

fn print_json(value: Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
