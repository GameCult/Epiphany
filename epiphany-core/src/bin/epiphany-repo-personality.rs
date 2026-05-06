use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use ignore::WalkBuilder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const TERRAIN_SCHEMA_VERSION: &str = "epiphany.repo_terrain_report.v0";
const PROFILE_SCHEMA_VERSION: &str = "epiphany.repo_personality_profile.v0";
const ROLE_PROJECTION_SCHEMA_VERSION: &str = "epiphany.role_personality_projection.v0";
const ARTIFACT_SCHEMA_VERSION: &str = "epiphany.repo_personality_artifacts.v0";
const DISTILLER_PACKET_SCHEMA_VERSION: &str = "epiphany.repo_personality_distiller_packet.v0";
const REPO_PERSONALITY_DISTILLER_PROMPT: &str =
    include_str!("../prompts/repo_personality_distiller.md");

const ROLES: &[&str] = &[
    "coordinator",
    "face",
    "imagination",
    "research",
    "modeling",
    "implementation",
    "verification",
    "reorientation",
];

const AXES: &[&str] = &[
    "contract_strictness",
    "protocol_intolerance",
    "boundary_severity",
    "actuation_risk",
    "runtime_proximity",
    "state_hygiene",
    "evidence_appetite",
    "source_fidelity",
    "content_canon_bias",
    "verification_environment_need",
    "burstiness",
    "consolidation_drive",
    "production_pressure",
    "temporal_pressure",
    "experimental_heat",
    "churn_spiral_risk",
    "interface_orientation",
    "aesthetic_appetite",
    "social_surface",
    "sensory_salience",
    "editorial_restraint",
    "speech_pressure",
    "novelty_hunger",
    "guardedness",
    "rumination_bias",
    "initiative_drive",
    "mood_lability",
];

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repo_terrain_report",
    schema = "EpiphanyRepoTerrainReport"
)]
pub struct RepoTerrainReport {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub repo_id: String,
    #[cultcache(key = 2)]
    pub name: String,
    #[cultcache(key = 3)]
    pub path: String,
    #[cultcache(key = 4)]
    pub remote_urls: Vec<String>,
    #[cultcache(key = 5)]
    pub source_families: Vec<String>,
    #[cultcache(key = 6)]
    pub languages: Vec<RepoSignalCount>,
    #[cultcache(key = 7)]
    pub state_surfaces: Vec<String>,
    #[cultcache(key = 8)]
    pub instruction_surfaces: Vec<String>,
    #[cultcache(key = 9)]
    pub test_surfaces: Vec<String>,
    #[cultcache(key = 10)]
    pub runtime_surfaces: Vec<String>,
    #[cultcache(key = 11)]
    pub history_metrics: RepoHistoryMetrics,
    #[cultcache(key = 12)]
    pub axis_scores: BTreeMap<String, f64>,
    #[cultcache(key = 13)]
    pub axis_evidence: BTreeMap<String, Vec<String>>,
    #[cultcache(key = 14)]
    pub confidence: f64,
    #[cultcache(key = 15)]
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoSignalCount {
    pub label: String,
    pub count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoHistoryMetrics {
    pub commit_count: usize,
    pub sampled_commits: usize,
    pub active_days: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub changed_files: usize,
    pub state_doc_touches: usize,
    pub test_receipt_touches: usize,
    pub runtime_touches: usize,
    pub protocol_touches: usize,
    pub ui_touches: usize,
    pub keyword_hits: BTreeMap<String, usize>,
    pub recent_messages: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repo_personality_profile",
    schema = "EpiphanyRepoPersonalityProfile"
)]
pub struct RepoPersonalityProfile {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub repo_id: String,
    #[cultcache(key = 2)]
    pub summary: String,
    #[cultcache(key = 3)]
    pub source_family_weights: BTreeMap<String, f64>,
    #[cultcache(key = 4)]
    pub axis_scores: BTreeMap<String, f64>,
    #[cultcache(key = 5)]
    pub axis_confidence: BTreeMap<String, f64>,
    #[cultcache(key = 6)]
    pub dominant_pressures: Vec<String>,
    #[cultcache(key = 7)]
    pub risk_pressures: Vec<String>,
    #[cultcache(key = 8)]
    pub role_modulations: Vec<RolePersonalityProjection>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.role_personality_projection",
    schema = "EpiphanyRolePersonalityProjection"
)]
pub struct RolePersonalityProjection {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub projection_id: String,
    #[cultcache(key = 2)]
    pub repo_id: String,
    #[cultcache(key = 3)]
    pub role_id: String,
    #[cultcache(key = 4)]
    pub trait_deltas: BTreeMap<String, f64>,
    #[cultcache(key = 5)]
    pub heartbeat_deltas: BTreeMap<String, f64>,
    #[cultcache(key = 6)]
    pub default_mood_pressure: BTreeMap<String, f64>,
    #[cultcache(key = 7)]
    pub semantic_memory_candidates: Vec<String>,
    #[cultcache(key = 8)]
    pub goal_candidates: Vec<String>,
    #[cultcache(key = 9)]
    pub value_candidates: Vec<String>,
    #[cultcache(key = 10)]
    pub private_note_candidates: Vec<String>,
    #[cultcache(key = 11)]
    pub reason: String,
    #[cultcache(key = 12)]
    pub evidence_refs: Vec<String>,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };
    let result = match command.as_str() {
        "scout" => run_scout(parse_scout_args(args)?),
        "project" => run_project(parse_project_args(args)?),
        "agent-packet" => run_agent_packet(parse_agent_packet_args(args)?),
        "status" => run_status(parse_status_args(args)?),
        other => Err(anyhow!("unknown command {other:?}")),
    }?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct ScoutArgs {
    root: PathBuf,
    artifact_dir: PathBuf,
    max_repos: Option<usize>,
}

#[derive(Clone, Debug)]
struct ProjectArgs {
    repo: PathBuf,
    baseline: PathBuf,
    artifact_dir: PathBuf,
}

#[derive(Clone, Debug)]
struct StatusArgs {
    store: PathBuf,
}

#[derive(Clone, Debug)]
struct AgentPacketArgs {
    store: PathBuf,
    artifact_dir: PathBuf,
    repo_id: Option<String>,
}

fn parse_scout_args(args: impl Iterator<Item = String>) -> Result<ScoutArgs> {
    let mut root = None;
    let mut artifact_dir = None;
    let mut max_repos = None;
    parse_named_args(args, |name, value| match name {
        "--root" => {
            root = Some(PathBuf::from(value));
            Ok(())
        }
        "--artifact-dir" => {
            artifact_dir = Some(PathBuf::from(value));
            Ok(())
        }
        "--max-repos" => {
            max_repos = Some(value.parse().context("--max-repos must be a number")?);
            Ok(())
        }
        other => Err(anyhow!("unexpected scout argument {other}")),
    })?;
    Ok(ScoutArgs {
        root: root.context("missing --root")?,
        artifact_dir: artifact_dir.context("missing --artifact-dir")?,
        max_repos,
    })
}

fn parse_project_args(args: impl Iterator<Item = String>) -> Result<ProjectArgs> {
    let mut repo = None;
    let mut baseline = None;
    let mut artifact_dir = None;
    parse_named_args(args, |name, value| match name {
        "--repo" => {
            repo = Some(PathBuf::from(value));
            Ok(())
        }
        "--baseline" => {
            baseline = Some(PathBuf::from(value));
            Ok(())
        }
        "--artifact-dir" => {
            artifact_dir = Some(PathBuf::from(value));
            Ok(())
        }
        other => Err(anyhow!("unexpected project argument {other}")),
    })?;
    Ok(ProjectArgs {
        repo: repo.context("missing --repo")?,
        baseline: baseline.context("missing --baseline")?,
        artifact_dir: artifact_dir.context("missing --artifact-dir")?,
    })
}

fn parse_status_args(args: impl Iterator<Item = String>) -> Result<StatusArgs> {
    let mut store = None;
    parse_named_args(args, |name, value| match name {
        "--store" => {
            store = Some(PathBuf::from(value));
            Ok(())
        }
        other => Err(anyhow!("unexpected status argument {other}")),
    })?;
    Ok(StatusArgs {
        store: store.context("missing --store")?,
    })
}

fn parse_agent_packet_args(args: impl Iterator<Item = String>) -> Result<AgentPacketArgs> {
    let mut store = None;
    let mut artifact_dir = None;
    let mut repo_id = None;
    parse_named_args(args, |name, value| match name {
        "--store" => {
            store = Some(PathBuf::from(value));
            Ok(())
        }
        "--artifact-dir" => {
            artifact_dir = Some(PathBuf::from(value));
            Ok(())
        }
        "--repo-id" => {
            repo_id = Some(value);
            Ok(())
        }
        other => Err(anyhow!("unexpected agent-packet argument {other}")),
    })?;
    Ok(AgentPacketArgs {
        store: store.context("missing --store")?,
        artifact_dir: artifact_dir.context("missing --artifact-dir")?,
        repo_id,
    })
}

fn parse_named_args(
    args: impl Iterator<Item = String>,
    mut handle: impl FnMut(&str, String) -> Result<()>,
) -> Result<()> {
    let mut args = args.peekable();
    while let Some(name) = args.next() {
        if !name.starts_with("--") {
            return Err(anyhow!("expected named argument, got {name:?}"));
        }
        let value = args
            .next()
            .ok_or_else(|| anyhow!("missing value for {name}"))?;
        handle(&name, value)?;
    }
    Ok(())
}

fn run_scout(args: ScoutArgs) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let store_path = args.artifact_dir.join("baseline.msgpack");
    let mut cache = repo_personality_cache(&store_path)?;
    cache.pull_all_backing_stores()?;

    let mut repos = discover_git_repos(&args.root)?;
    repos.sort();
    if let Some(limit) = args.max_repos {
        repos.truncate(limit);
    }

    let mut reports = Vec::new();
    for repo in repos {
        let report = scout_repo(&repo)?;
        cache.put::<RepoTerrainReport>(report.repo_id.clone(), &report)?;
        reports.push(report);
    }
    let profiles = reduce_reports(&reports);
    for profile in &profiles {
        cache.put::<RepoPersonalityProfile>(profile.repo_id.clone(), profile)?;
        for projection in &profile.role_modulations {
            cache.put::<RolePersonalityProjection>(projection.projection_id.clone(), projection)?;
        }
    }

    let summary = json!({
        "schemaVersion": ARTIFACT_SCHEMA_VERSION,
        "mode": "scout",
        "root": args.root,
        "artifactDir": args.artifact_dir,
        "store": store_path,
        "repoCount": reports.len(),
        "profileCount": profiles.len(),
        "repos": reports.iter().map(report_summary).collect::<Vec<_>>(),
    });
    write_json(
        &args
            .artifact_dir
            .join("repo-personality-scout-summary.json"),
        &summary,
    )?;
    write_text(
        &args.artifact_dir.join("repo-personality-scout-summary.md"),
        &render_scout_markdown(&reports, &profiles),
    )?;
    Ok(summary)
}

fn run_project(args: ProjectArgs) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let mut baseline = repo_personality_cache(&args.baseline)?;
    baseline.pull_all_backing_stores()?;
    let reports = baseline.get_all::<RepoTerrainReport>()?;
    let target_id = repo_id(&args.repo);
    let report = reports
        .iter()
        .find(|report| report.repo_id == target_id || same_path(&report.path, &args.repo))
        .cloned()
        .unwrap_or_else(|| scout_repo(&args.repo).expect("target repo should be scoutable"));
    let profile = reduce_report(&report);

    let store_path = args.artifact_dir.join("projection.msgpack");
    let mut cache = repo_personality_cache(&store_path)?;
    cache.pull_all_backing_stores()?;
    cache.put::<RepoTerrainReport>(report.repo_id.clone(), &report)?;
    cache.put::<RepoPersonalityProfile>(profile.repo_id.clone(), &profile)?;
    for projection in &profile.role_modulations {
        cache.put::<RolePersonalityProjection>(projection.projection_id.clone(), projection)?;
    }

    let summary = json!({
        "schemaVersion": ARTIFACT_SCHEMA_VERSION,
        "mode": "project",
        "repo": args.repo,
        "baseline": args.baseline,
        "artifactDir": args.artifact_dir,
        "store": store_path,
        "profile": profile_summary(&profile),
        "terrain": report_summary(&report),
    });
    write_json(
        &args
            .artifact_dir
            .join("repo-personality-project-summary.json"),
        &summary,
    )?;
    write_text(
        &args
            .artifact_dir
            .join("repo-personality-project-summary.md"),
        &render_project_markdown(&report, &profile),
    )?;
    Ok(summary)
}

fn run_status(args: StatusArgs) -> Result<Value> {
    let mut cache = repo_personality_cache(&args.store)?;
    cache.pull_all_backing_stores()?;
    let reports = cache.get_all::<RepoTerrainReport>()?;
    let profiles = cache.get_all::<RepoPersonalityProfile>()?;
    let projections = cache.get_all::<RolePersonalityProjection>()?;
    Ok(json!({
        "schemaVersion": ARTIFACT_SCHEMA_VERSION,
        "mode": "status",
        "store": args.store,
        "reports": reports.len(),
        "profiles": profiles.len(),
        "roleProjections": projections.len(),
        "repos": reports.iter().map(report_summary).collect::<Vec<_>>(),
    }))
}

fn run_agent_packet(args: AgentPacketArgs) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let mut cache = repo_personality_cache(&args.store)?;
    cache.pull_all_backing_stores()?;
    let reports = cache.get_all::<RepoTerrainReport>()?;
    let profiles = cache.get_all::<RepoPersonalityProfile>()?;
    let projections = cache.get_all::<RolePersonalityProjection>()?;
    let profile = select_profile(&profiles, args.repo_id.as_deref())?;
    let report = reports
        .iter()
        .find(|report| report.repo_id == profile.repo_id)
        .ok_or_else(|| {
            anyhow!(
                "store has profile {:?} but no terrain report",
                profile.repo_id
            )
        })?;
    let role_projections: Vec<_> = projections
        .iter()
        .filter(|projection| projection.repo_id == profile.repo_id)
        .collect();
    let packet = json!({
        "schemaVersion": DISTILLER_PACKET_SCHEMA_VERSION,
        "createdAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "store": args.store,
        "repoId": profile.repo_id,
        "lifecycle": {
            "mode": "birth-only",
            "contract": "Run this specialist only when the repo/swarm has no accepted personality initialization. Later personality movement belongs to heartbeat, mood, rumination, sleep consolidation, lived evidence, and reviewed selfPatch.",
            "rerunPolicy": "If an accepted initialization exists, do not rerun to refresh personality. Route major terrain surprises to Eyes/Body or Self review as normal state/model work, not personality reset."
        },
        "prompt": REPO_PERSONALITY_DISTILLER_PROMPT,
        "input": {
            "repoTerrainReport": report_agent_input(report),
            "repoPersonalityProfile": profile_agent_input(profile),
            "rolePersonalityProjections": role_projections
                .iter()
                .map(|projection| role_projection_agent_input(projection))
                .collect::<Vec<_>>(),
        },
        "expectedOutput": {
            "verdict": "ready-for-review | needs-more-terrain | reject",
            "summary": "short repo personality pressure summary",
            "confidence": "0.0..1.0",
            "roleQuirks": [],
            "selfPatchCandidates": [],
            "initializationRecord": {
                "repoId": profile.repo_id,
                "terrainSchemaVersion": report.schema_version,
                "profileSchemaVersion": profile.schema_version,
                "acceptedOnce": true
            },
            "doNotMutate": [],
            "nextSafeMove": "Self reviews candidate pressure deltas before first initialization mutation; later drift uses heartbeat/mood/sleep/selfPatch."
        },
        "guardrails": [
            "This packet is input to a specialist agent, not accepted truth.",
            "This packet is birth-only; do not rerun after an accepted initialization just because startup happened.",
            "Repo facts stay in terrain/model/planning/evidence surfaces.",
            "Role memory may receive only subtle, bounded, Self-reviewed personality pressure.",
            "No objectives, file lists, raw transcripts, code edits, or authority claims in selfPatch."
        ]
    });
    let packet_path = args
        .artifact_dir
        .join("repo-personality-distiller-packet.json");
    let prompt_path = args
        .artifact_dir
        .join("repo-personality-distiller-prompt.md");
    let summary_path = args
        .artifact_dir
        .join("repo-personality-distiller-packet.md");
    write_json(&packet_path, &packet)?;
    write_text(&prompt_path, REPO_PERSONALITY_DISTILLER_PROMPT)?;
    write_text(
        &summary_path,
        &render_agent_packet_markdown(report, profile, role_projections.len()),
    )?;
    Ok(json!({
        "schemaVersion": DISTILLER_PACKET_SCHEMA_VERSION,
        "mode": "agent-packet",
        "repoId": profile.repo_id,
        "artifactDir": args.artifact_dir,
        "packetPath": packet_path,
        "promptPath": prompt_path,
        "summaryPath": summary_path,
        "roleProjectionCount": role_projections.len(),
    }))
}

fn select_profile<'a>(
    profiles: &'a [RepoPersonalityProfile],
    repo_id: Option<&str>,
) -> Result<&'a RepoPersonalityProfile> {
    if let Some(repo_id) = repo_id {
        return profiles
            .iter()
            .find(|profile| profile.repo_id == repo_id)
            .ok_or_else(|| anyhow!("no profile for repo id {repo_id:?}"));
    }
    match profiles {
        [profile] => Ok(profile),
        [] => Err(anyhow!("store has no repo personality profiles")),
        _ => Err(anyhow!(
            "store has multiple profiles; pass --repo-id to choose one"
        )),
    }
}

fn repo_personality_cache(path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<RepoTerrainReport>()?;
    cache.register_entry_type::<RepoPersonalityProfile>()?;
    cache.register_entry_type::<RolePersonalityProjection>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(path));
    Ok(cache)
}

fn discover_git_repos(root: &Path) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();
    if root.join(".git").exists() {
        repos.push(root.to_path_buf());
        return Ok(repos);
    }
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if name.starts_with('_') || matches!(name, "node_modules" | "target") {
            continue;
        }
        if path.join(".git").exists() {
            repos.push(path);
        }
    }
    Ok(repos)
}

fn scout_repo(repo: &Path) -> Result<RepoTerrainReport> {
    let repo = repo
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", repo.display()))?;
    let name = repo
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("repo")
        .to_string();
    let inventory = inventory_repo(&repo)?;
    let history_metrics = history_metrics(&repo)?;
    let mut axis_evidence = BTreeMap::new();
    let source_families = source_families(&name, &inventory, &history_metrics);
    let axis_scores = score_axes(
        &inventory,
        &history_metrics,
        &source_families,
        &mut axis_evidence,
    );
    let confidence = confidence_for(&inventory, &history_metrics);
    let warnings = warnings_for(&inventory, &history_metrics);
    Ok(RepoTerrainReport {
        schema_version: TERRAIN_SCHEMA_VERSION.to_string(),
        repo_id: repo_id(&repo),
        name,
        path: repo.display().to_string(),
        remote_urls: git_remote_urls(&repo),
        source_families,
        languages: top_counts(inventory.extensions, 12),
        state_surfaces: inventory.state_surfaces,
        instruction_surfaces: inventory.instruction_surfaces,
        test_surfaces: inventory.test_surfaces,
        runtime_surfaces: inventory.runtime_surfaces,
        history_metrics,
        axis_scores,
        axis_evidence,
        confidence,
        warnings,
    })
}

#[derive(Clone, Debug, Default)]
struct RepoInventory {
    extensions: BTreeMap<String, usize>,
    path_classes: BTreeMap<String, usize>,
    state_surfaces: Vec<String>,
    instruction_surfaces: Vec<String>,
    test_surfaces: Vec<String>,
    runtime_surfaces: Vec<String>,
    file_count: usize,
}

fn inventory_repo(repo: &Path) -> Result<RepoInventory> {
    let mut inventory = RepoInventory::default();
    let walker = WalkBuilder::new(repo)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .parents(true)
        .build();
    for entry in walker.filter_map(Result::ok) {
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }
        let path = entry.path();
        if is_ignored_path(path) {
            continue;
        }
        inventory.file_count += 1;
        let rel = path.strip_prefix(repo).unwrap_or(path);
        let rel_string = slash(rel);
        if let Some(ext) = path.extension().and_then(|value| value.to_str()) {
            *inventory
                .extensions
                .entry(format!(".{}", ext.to_ascii_lowercase()))
                .or_default() += 1;
        }
        for class in path_classes(&rel_string) {
            *inventory.path_classes.entry(class.to_string()).or_default() += 1;
        }
        if is_instruction_surface(&rel_string) {
            inventory.instruction_surfaces.push(rel_string.clone());
        }
        if is_state_surface(&rel_string) {
            inventory.state_surfaces.push(rel_string.clone());
        }
        if is_test_surface(&rel_string) {
            inventory.test_surfaces.push(rel_string.clone());
        }
        if is_runtime_surface(&rel_string) {
            inventory.runtime_surfaces.push(rel_string.clone());
        }
    }
    inventory.state_surfaces.sort();
    inventory.instruction_surfaces.sort();
    inventory.test_surfaces.sort();
    inventory.runtime_surfaces.sort();
    Ok(inventory)
}

fn is_ignored_path(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        matches!(
            value.as_ref(),
            ".git" | "node_modules" | "target" | "Library" | "Temp" | "obj" | "bin" | "dist"
        )
    })
}

fn path_classes(path: &str) -> Vec<&'static str> {
    let lower = path.to_ascii_lowercase();
    let mut classes = Vec::new();
    if lower.contains("state/")
        || lower.contains("/notes/")
        || lower.contains("handoff")
        || lower.ends_with("agents.md")
        || lower.contains("evidence")
        || lower.contains("ledger")
        || lower.contains("memory")
    {
        classes.push("state_doc");
    }
    if lower.contains("test")
        || lower.contains("smoke")
        || lower.contains("fixture")
        || lower.contains("artifact")
        || lower.contains("ci")
        || lower.contains(".github")
    {
        classes.push("test_receipt");
    }
    if lower.contains("schema")
        || lower.contains("contract")
        || lower.contains("protocol")
        || lower.contains("cultnet")
        || lower.contains("cultcache")
        || lower.contains("messagepack")
        || lower.contains("envelope")
    {
        classes.push("protocol");
    }
    if lower.contains("assets/")
        || lower.contains("projectsettings")
        || lower.contains("unity")
        || lower.contains("editor")
        || lower.contains("scene")
        || lower.ends_with(".prefab")
        || lower.ends_with(".unity")
    {
        classes.push("runtime");
    }
    if lower.contains("ui")
        || lower.contains("components")
        || lower.contains("web")
        || lower.contains("tsx")
        || lower.contains("css")
        || lower.contains("scss")
        || lower.contains("tauri")
    {
        classes.push("ui");
    }
    if lower.contains("lore")
        || lower.contains("content")
        || lower.contains("quartz")
        || lower.contains("post")
        || lower.ends_with(".md")
    {
        classes.push("content");
    }
    if lower.contains("auth")
        || lower.contains("oauth")
        || lower.contains("discord")
        || lower.contains("postgres")
        || lower.contains("deploy")
        || lower.contains("runbook")
        || lower.contains("service")
    {
        classes.push("ops_social");
    }
    classes
}

fn is_instruction_surface(path: &str) -> bool {
    path.eq_ignore_ascii_case("AGENTS.md") || path.ends_with("/AGENTS.md")
}

fn is_state_surface(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("state/")
        || lower.contains("/state/")
        || lower.starts_with(".epiphany/state/")
        || lower.contains("fresh-workspace-handoff")
        || lower.contains("map.yaml")
        || lower.contains("memory.json")
}

fn is_test_surface(path: &str) -> bool {
    path_classes(path).contains(&"test_receipt")
}

fn is_runtime_surface(path: &str) -> bool {
    path_classes(path).contains(&"runtime")
}

fn history_metrics(repo: &Path) -> Result<RepoHistoryMetrics> {
    let commit_count = git_output(repo, &["rev-list", "--count", "HEAD"])
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0);
    let log = git_output(
        repo,
        &[
            "log",
            "-n",
            "80",
            "--numstat",
            "--date=short",
            "--pretty=format:--EPIPHANY-COMMIT--%x09%cs%x09%s",
        ],
    )
    .unwrap_or_default();
    let mut metrics = RepoHistoryMetrics {
        commit_count,
        ..RepoHistoryMetrics::default()
    };
    let mut days = BTreeSet::new();
    for line in log.lines() {
        if let Some(rest) = line.strip_prefix("--EPIPHANY-COMMIT--\t") {
            metrics.sampled_commits += 1;
            let mut parts = rest.splitn(2, '\t');
            if let Some(day) = parts.next() {
                days.insert(day.to_string());
            }
            if let Some(message) = parts.next() {
                if metrics.recent_messages.len() < 12 {
                    metrics.recent_messages.push(message.to_string());
                }
                count_message_keywords(&mut metrics.keyword_hits, message);
            }
            continue;
        }
        let mut parts = line.split('\t');
        let Some(insertions) = parts.next() else {
            continue;
        };
        let Some(deletions) = parts.next() else {
            continue;
        };
        let Some(path) = parts.next() else {
            continue;
        };
        if let Ok(value) = insertions.parse::<usize>() {
            metrics.insertions += value;
        }
        if let Ok(value) = deletions.parse::<usize>() {
            metrics.deletions += value;
        }
        metrics.changed_files += 1;
        let classes = path_classes(path);
        if classes.contains(&"state_doc") {
            metrics.state_doc_touches += 1;
        }
        if classes.contains(&"test_receipt") {
            metrics.test_receipt_touches += 1;
        }
        if classes.contains(&"runtime") {
            metrics.runtime_touches += 1;
        }
        if classes.contains(&"protocol") {
            metrics.protocol_touches += 1;
        }
        if classes.contains(&"ui") {
            metrics.ui_touches += 1;
        }
    }
    metrics.active_days = days.len();
    Ok(metrics)
}

fn count_message_keywords(hits: &mut BTreeMap<String, usize>, message: &str) {
    let lower = message.to_ascii_lowercase();
    for (class, words) in [
        (
            "consolidation",
            &[
                "refactor", "extract", "remove", "cut", "replace", "rename", "prune",
            ][..],
        ),
        (
            "production",
            &[
                "fix", "bug", "deploy", "hosted", "policy", "queue", "auth", "ci",
            ][..],
        ),
        (
            "experimental",
            &[
                "experiment",
                "prototype",
                "scaffold",
                "study",
                "probe",
                "explore",
            ][..],
        ),
        (
            "protocol",
            &[
                "schema", "contract", "interop", "wire", "envelope", "cultnet",
            ][..],
        ),
        (
            "evidence",
            &["test", "smoke", "verify", "receipt", "artifact", "audit"][..],
        ),
        (
            "content",
            &["lore", "site", "quartz", "copy", "polish", "canon"][..],
        ),
    ] {
        if words.iter().any(|word| lower.contains(word)) {
            *hits.entry(class.to_string()).or_default() += 1;
        }
    }
}

fn source_families(
    name: &str,
    inventory: &RepoInventory,
    history: &RepoHistoryMetrics,
) -> Vec<String> {
    let lower_name = name.to_ascii_lowercase();
    let mut families = Vec::new();
    if lower_name.contains("epiphany") {
        families.push("epiphany_spine".to_string());
    }
    if lower_name.contains("cult") || class_count(inventory, "protocol") > 4 {
        families.push("cult_protocol_storage".to_string());
    }
    if lower_name.contains("lore")
        || lower_name.contains("site")
        || lower_name.contains("quartz")
        || class_count(inventory, "content") > 20
    {
        families.push("gamecult_web_lore_ops".to_string());
    }
    if class_count(inventory, "runtime") > 12 || history.runtime_touches > 5 {
        families.push("unity_runtime_body".to_string());
    }
    if lower_name.contains("void")
        || lower_name.contains("heimdall")
        || lower_name.contains("stream")
        || class_count(inventory, "ops_social") > 8
    {
        families.push("service_product_app".to_string());
    }
    if lower_name.contains("ghostlight")
        || lower_name.contains("vibe")
        || lower_name.contains("mosaic")
        || lower_name.contains("repixel")
        || lower_name.contains("eusocial")
    {
        families.push("research_workbench".to_string());
    }
    if families.is_empty() {
        families.push("general_workspace".to_string());
    }
    families
}

fn score_axes(
    inventory: &RepoInventory,
    history: &RepoHistoryMetrics,
    families: &[String],
    evidence: &mut BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, f64> {
    let mut scores = BTreeMap::new();
    for axis in AXES {
        scores.insert((*axis).to_string(), 0.0);
    }
    let files = inventory.file_count.max(1) as f64;
    let changed = history.changed_files.max(1) as f64;
    let commits = history.sampled_commits.max(1) as f64;
    let state_ratio = (class_count(inventory, "state_doc") as f64 / files).min(1.0);
    let test_ratio = (class_count(inventory, "test_receipt") as f64 / files).min(1.0);
    let protocol_ratio = (class_count(inventory, "protocol") as f64 / files).min(1.0);
    let runtime_ratio = (class_count(inventory, "runtime") as f64 / files).min(1.0);
    let ui_ratio = (class_count(inventory, "ui") as f64 / files).min(1.0);
    let content_ratio = (class_count(inventory, "content") as f64 / files).min(1.0);
    let ops_ratio = (class_count(inventory, "ops_social") as f64 / files).min(1.0);
    macro_rules! record {
        ($axis:expr, $value:expr, $reason:expr) => {{
            let value = $value;
            set_score(&mut scores, evidence, $axis, value, $reason);
        }};
    }
    record!(
        "state_hygiene",
        (state_ratio * 6.0 + presence(inventory.state_surfaces.len()) * 0.4).min(1.0),
        "state, map, evidence, handoff, or memory surfaces"
    );
    record!(
        "evidence_appetite",
        (test_ratio * 7.0
            + touch_ratio(history.test_receipt_touches, changed) * 0.7
            + keyword(history, "evidence", commits) * 0.5)
            .min(1.0),
        "tests, smoke checks, artifacts, or verifier keywords"
    );
    record!(
        "contract_strictness",
        (protocol_ratio * 8.0
            + touch_ratio(history.protocol_touches, changed)
            + family(families, "cult_protocol_storage") * 0.35)
            .min(1.0),
        "schema, contract, protocol, CultCache, or CultNet surfaces"
    );
    record!(
        "protocol_intolerance",
        (score(&scores, "contract_strictness") * 0.85 + family(families, "epiphany_spine") * 0.25)
            .min(1.0),
        "strict contract surfaces imply low tolerance for ad hoc mutation"
    );
    record!(
        "runtime_proximity",
        (runtime_ratio * 8.0
            + touch_ratio(history.runtime_touches, changed)
            + family(families, "unity_runtime_body") * 0.35)
            .min(1.0),
        "Unity/editor/runtime/provider surfaces"
    );
    record!(
        "actuation_risk",
        (score(&scores, "runtime_proximity") * 0.5
            + ops_ratio * 3.0
            + family(families, "service_product_app") * 0.35)
            .min(1.0),
        "runtime, auth, ops, or service writes can hurt real users"
    );
    record!(
        "boundary_severity",
        (ops_ratio * 4.0
            + score(&scores, "contract_strictness") * 0.35
            + family(families, "service_product_app") * 0.35)
            .min(1.0),
        "auth, ops, workspace, protocol, or service boundaries"
    );
    record!(
        "source_fidelity",
        (score(&scores, "state_hygiene") * 0.25
            + content_ratio * 2.0
            + score(&scores, "runtime_proximity") * 0.4)
            .min(1.0),
        "state maps, lore/canon, or runtime truth surfaces"
    );
    record!(
        "content_canon_bias",
        (content_ratio * 5.0
            + family(families, "gamecult_web_lore_ops") * 0.35
            + keyword(history, "content", commits) * 0.4)
            .min(1.0),
        "lore, site, markdown, Quartz, canon, or editorial paths"
    );
    record!(
        "verification_environment_need",
        (score(&scores, "runtime_proximity") * 0.55
            + score(&scores, "interface_orientation") * 0.2
            + score(&scores, "actuation_risk") * 0.35)
            .min(1.0),
        "claims need runtime, editor, browser, provider, or service receipts"
    );
    record!(
        "burstiness",
        burstiness(history),
        "sampled commits compressed into few active days"
    );
    record!(
        "consolidation_drive",
        (keyword(history, "consolidation", commits) * 0.7 + deletion_pressure(history) * 0.5)
            .min(1.0),
        "refactor/remove/extract keywords or deletion-heavy history"
    );
    record!(
        "production_pressure",
        (keyword(history, "production", commits) * 0.7
            + family(families, "service_product_app") * 0.35
            + score(&scores, "actuation_risk") * 0.3)
            .min(1.0),
        "fix/deploy/auth/queue/CI signals"
    );
    record!(
        "experimental_heat",
        (keyword(history, "experimental", commits) * 0.8
            + family(families, "research_workbench") * 0.45
            + insertion_pressure(history) * 0.25)
            .min(1.0),
        "prototype, experiment, scaffold, or research-workbench signals"
    );
    record!(
        "churn_spiral_risk",
        (large_churn(history) * 0.45
            + score(&scores, "experimental_heat") * 0.25
            + (1.0 - score(&scores, "evidence_appetite")) * 0.2)
            .min(1.0),
        "large churn, experiment heat, and weak receipts"
    );
    record!(
        "interface_orientation",
        (ui_ratio * 5.0
            + touch_ratio(history.ui_touches, changed)
            + family(families, "epiphany_spine") * 0.2)
            .min(1.0),
        "UI, web, Tauri, component, DOM, or Aquarium surfaces"
    );
    record!(
        "aesthetic_appetite",
        (score(&scores, "interface_orientation") * 0.45
            + family(families, "research_workbench") * 0.25
            + content_ratio * 1.5)
            .min(1.0),
        "visual, lore, rendered, or artifact-heavy surfaces"
    );
    record!(
        "social_surface",
        (ops_ratio * 3.0
            + family(families, "service_product_app") * 0.35
            + family(families, "gamecult_web_lore_ops") * 0.15)
            .min(1.0),
        "Discord, auth, accounts, public site, or service boundaries"
    );
    record!(
        "sensory_salience",
        (score(&scores, "interface_orientation") * 0.45
            + score(&scores, "aesthetic_appetite") * 0.45
            + score(&scores, "runtime_proximity") * 0.15)
            .min(1.0),
        "motion, visuals, rendered outputs, scenes, or UI organisms"
    );
    record!(
        "editorial_restraint",
        (score(&scores, "content_canon_bias") * 0.6 + score(&scores, "source_fidelity") * 0.25)
            .min(1.0),
        "canon/source discipline under prose pressure"
    );
    record!(
        "speech_pressure",
        (score(&scores, "social_surface") * 0.45 + score(&scores, "interface_orientation") * 0.2)
            .min(1.0),
        "public speech or user-facing surfaces"
    );
    record!(
        "novelty_hunger",
        (score(&scores, "experimental_heat") * 0.55 + score(&scores, "aesthetic_appetite") * 0.25)
            .min(1.0),
        "experimental and aesthetic exploration pressure"
    );
    record!(
        "guardedness",
        (score(&scores, "boundary_severity") * 0.45
            + score(&scores, "actuation_risk") * 0.35
            + score(&scores, "contract_strictness") * 0.2)
            .min(1.0),
        "authority and mutation risk demand caution"
    );
    record!(
        "rumination_bias",
        (score(&scores, "state_hygiene") * 0.4
            + score(&scores, "consolidation_drive") * 0.25
            + score(&scores, "content_canon_bias") * 0.15)
            .min(1.0),
        "state hygiene and consolidation favor distillation before action"
    );
    record!(
        "initiative_drive",
        (score(&scores, "production_pressure") * 0.35
            + score(&scores, "temporal_pressure") * 0.25
            + score(&scores, "experimental_heat") * 0.2)
            .min(1.0),
        "work pressure and experiment heat increase heartbeat readiness"
    );
    record!(
        "mood_lability",
        (score(&scores, "churn_spiral_risk") * 0.35
            + score(&scores, "temporal_pressure") * 0.25
            + score(&scores, "actuation_risk") * 0.2)
            .min(1.0),
        "risk, urgency, and churn make reactions swing harder"
    );
    let temporal = (score(&scores, "production_pressure") * 0.45
        + family(families, "service_product_app") * 0.35
        + score(&scores, "runtime_proximity") * 0.2)
        .min(1.0);
    record!(
        "temporal_pressure",
        temporal,
        "service, runtime, queue, or live-provider timing pressure"
    );
    scores
}

fn set_score(
    scores: &mut BTreeMap<String, f64>,
    evidence: &mut BTreeMap<String, Vec<String>>,
    axis: &str,
    value: f64,
    reason: &str,
) {
    scores.insert(axis.to_string(), round3(value.clamp(0.0, 1.0)));
    evidence
        .entry(axis.to_string())
        .or_default()
        .push(reason.to_string());
}

fn score(scores: &BTreeMap<String, f64>, axis: &str) -> f64 {
    scores.get(axis).copied().unwrap_or(0.0)
}

fn class_count(inventory: &RepoInventory, class: &str) -> usize {
    inventory.path_classes.get(class).copied().unwrap_or(0)
}

fn presence(count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        (count as f64 / 8.0).min(1.0)
    }
}

fn touch_ratio(count: usize, total: f64) -> f64 {
    (count as f64 / total).min(1.0)
}

fn keyword(history: &RepoHistoryMetrics, class: &str, commits: f64) -> f64 {
    (*history.keyword_hits.get(class).unwrap_or(&0) as f64 / commits).min(1.0)
}

fn family(families: &[String], family: &str) -> f64 {
    families.iter().any(|candidate| candidate == family) as u8 as f64
}

fn burstiness(history: &RepoHistoryMetrics) -> f64 {
    if history.sampled_commits == 0 {
        return 0.0;
    }
    let active_days = history.active_days.max(1) as f64;
    ((history.sampled_commits as f64 / active_days) / 12.0).min(1.0)
}

fn deletion_pressure(history: &RepoHistoryMetrics) -> f64 {
    let total = (history.insertions + history.deletions).max(1) as f64;
    (history.deletions as f64 / total).min(1.0)
}

fn insertion_pressure(history: &RepoHistoryMetrics) -> f64 {
    let total = (history.insertions + history.deletions).max(1) as f64;
    (history.insertions as f64 / total).min(1.0)
}

fn large_churn(history: &RepoHistoryMetrics) -> f64 {
    if history.sampled_commits == 0 {
        return 0.0;
    }
    ((history.changed_files as f64 / history.sampled_commits as f64) / 18.0).min(1.0)
}

fn reduce_reports(reports: &[RepoTerrainReport]) -> Vec<RepoPersonalityProfile> {
    reports.iter().map(reduce_report).collect()
}

fn reduce_report(report: &RepoTerrainReport) -> RepoPersonalityProfile {
    let family_weights = source_family_weights(report);
    let axis_confidence = report
        .axis_scores
        .keys()
        .map(|axis| (axis.clone(), report.confidence))
        .collect();
    let dominant_pressures = top_axes(&report.axis_scores, 6);
    let risk_pressures = risk_axes(&report.axis_scores);
    let role_modulations = ROLES
        .iter()
        .map(|role| role_projection(report, role))
        .collect();
    RepoPersonalityProfile {
        schema_version: PROFILE_SCHEMA_VERSION.to_string(),
        repo_id: report.repo_id.clone(),
        summary: summary_for(report, &dominant_pressures),
        source_family_weights: family_weights,
        axis_scores: report.axis_scores.clone(),
        axis_confidence,
        dominant_pressures,
        risk_pressures,
        role_modulations,
    }
}

fn source_family_weights(report: &RepoTerrainReport) -> BTreeMap<String, f64> {
    let mut weights = BTreeMap::new();
    let count = report.source_families.len().max(1) as f64;
    for family in &report.source_families {
        weights.insert(family.clone(), round3(1.0 / count));
    }
    weights
}

fn role_projection(report: &RepoTerrainReport, role_id: &str) -> RolePersonalityProjection {
    let axes = &report.axis_scores;
    let mut trait_deltas = BTreeMap::new();
    let mut heartbeat_deltas = BTreeMap::new();
    let mut mood = BTreeMap::new();
    let relevant = role_axes(role_id);
    for axis in relevant {
        trait_deltas.insert(axis.to_string(), round3((score(axes, axis) - 0.5) * 0.6));
    }
    heartbeat_deltas.insert(
        "initiativeSpeedDelta".to_string(),
        round3((role_axis_average(axes, &["initiative_drive", "production_pressure"]) - 0.5) * 0.4),
    );
    heartbeat_deltas.insert(
        "cooldownMultiplierDelta".to_string(),
        round3((role_axis_average(axes, &["guardedness", "rumination_bias"]) - 0.5) * 0.3),
    );
    mood.insert(
        "anxiety".to_string(),
        round3(role_axis_average(
            axes,
            &["actuation_risk", "churn_spiral_risk", "boundary_severity"],
        )),
    );
    mood.insert(
        "curiosity".to_string(),
        round3(role_axis_average(
            axes,
            &["novelty_hunger", "experimental_heat", "source_fidelity"],
        )),
    );
    mood.insert(
        "urgency".to_string(),
        round3(role_axis_average(
            axes,
            &["production_pressure", "temporal_pressure"],
        )),
    );
    let role_label = role_display(role_id);
    RolePersonalityProjection {
        schema_version: ROLE_PROJECTION_SCHEMA_VERSION.to_string(),
        projection_id: format!("{}::{}", report.repo_id, role_id),
        repo_id: report.repo_id.clone(),
        role_id: role_id.to_string(),
        trait_deltas,
        heartbeat_deltas,
        default_mood_pressure: mood,
        semantic_memory_candidates: vec![format!(
            "{role_label} should treat {} as a repo with dominant pressures: {}.",
            report.name,
            top_axes(&report.axis_scores, 3).join(", ")
        )],
        goal_candidates: vec![format!(
            "Adapt {role_label} behavior to {} without storing project facts in role memory.",
            report.name
        )],
        value_candidates: vec![role_value_candidate(role_id).to_string()],
        private_note_candidates: vec![format!(
            "Projection is deterministic and confidence-scored at {:.2}; Self must review before mutation.",
            report.confidence
        )],
        reason: format!(
            "Role projection from repo terrain, commit history, and persisted doctrine for {}.",
            report.name
        ),
        evidence_refs: report
            .axis_evidence
            .iter()
            .take(6)
            .map(|(axis, reasons)| format!("{axis}: {}", reasons.join("; ")))
            .collect(),
    }
}

fn role_axes(role_id: &str) -> &'static [&'static str] {
    match role_id {
        "coordinator" => &[
            "boundary_severity",
            "contract_strictness",
            "state_hygiene",
            "churn_spiral_risk",
            "production_pressure",
        ],
        "face" => &[
            "social_surface",
            "interface_orientation",
            "sensory_salience",
            "speech_pressure",
            "editorial_restraint",
        ],
        "imagination" => &[
            "experimental_heat",
            "aesthetic_appetite",
            "content_canon_bias",
            "novelty_hunger",
            "churn_spiral_risk",
        ],
        "research" => &[
            "source_fidelity",
            "protocol_intolerance",
            "runtime_proximity",
            "novelty_hunger",
            "verification_environment_need",
        ],
        "modeling" => &[
            "runtime_proximity",
            "contract_strictness",
            "state_hygiene",
            "content_canon_bias",
            "source_fidelity",
        ],
        "implementation" => &[
            "production_pressure",
            "actuation_risk",
            "contract_strictness",
            "consolidation_drive",
            "churn_spiral_risk",
        ],
        "verification" => &[
            "evidence_appetite",
            "verification_environment_need",
            "actuation_risk",
            "interface_orientation",
            "content_canon_bias",
        ],
        "reorientation" => &[
            "state_hygiene",
            "burstiness",
            "rumination_bias",
            "mood_lability",
            "temporal_pressure",
        ],
        _ => &["state_hygiene"],
    }
}

fn role_axis_average(axes: &BTreeMap<String, f64>, names: &[&str]) -> f64 {
    if names.is_empty() {
        return 0.0;
    }
    names.iter().map(|name| score(axes, name)).sum::<f64>() / names.len() as f64
}

fn role_display(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => "Self",
        "face" => "Face",
        "imagination" => "Imagination",
        "research" => "Eyes",
        "modeling" => "Body",
        "implementation" => "Hands",
        "verification" => "Soul",
        "reorientation" => "Life",
        _ => "Lane",
    }
}

fn role_value_candidate(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => {
            "Coordinate through typed authority and challenge pattern-completion theater."
        }
        "face" => {
            "Surface state through the public mouth without turning internals into chat endpoints."
        }
        "imagination" => {
            "Turn future-shape pressure into drafts and plans, not accidental active objectives."
        }
        "research" => "Find existing truth before invention.",
        "modeling" => "Build source-grounded maps before Hands cuts.",
        "implementation" => "Leave reviewable diffs or explicit failure artifacts.",
        "verification" => "Demand receipts from the environment that owns the claim.",
        "reorientation" => "Bank continuity before pressure turns memory into ash.",
        _ => "Serve the repo through typed state and reviewable evidence.",
    }
}

fn summary_for(report: &RepoTerrainReport, dominant: &[String]) -> String {
    format!(
        "{} projects as {} with dominant pressures: {}.",
        report.name,
        report.source_families.join(" + "),
        dominant.join(", ")
    )
}

fn top_axes(scores: &BTreeMap<String, f64>, limit: usize) -> Vec<String> {
    let mut items: Vec<_> = scores.iter().collect();
    items.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
    items
        .into_iter()
        .take(limit)
        .map(|(axis, value)| format!("{axis}:{value:.2}"))
        .collect()
}

fn risk_axes(scores: &BTreeMap<String, f64>) -> Vec<String> {
    [
        "actuation_risk",
        "churn_spiral_risk",
        "boundary_severity",
        "mood_lability",
    ]
    .iter()
    .filter_map(|axis| {
        let value = score(scores, axis);
        (value >= 0.45).then(|| format!("{axis}:{value:.2}"))
    })
    .collect()
}

fn confidence_for(inventory: &RepoInventory, history: &RepoHistoryMetrics) -> f64 {
    let mut confidence: f64 = 0.35;
    if !inventory.instruction_surfaces.is_empty() {
        confidence += 0.15;
    }
    if !inventory.state_surfaces.is_empty() {
        confidence += 0.15;
    }
    if history.sampled_commits > 0 {
        confidence += 0.15;
    }
    if inventory.file_count > 10 {
        confidence += 0.1;
    }
    if !inventory.test_surfaces.is_empty() || !inventory.runtime_surfaces.is_empty() {
        confidence += 0.1;
    }
    round3(confidence.min(1.0))
}

fn warnings_for(inventory: &RepoInventory, history: &RepoHistoryMetrics) -> Vec<String> {
    let mut warnings = Vec::new();
    if inventory.instruction_surfaces.is_empty() {
        warnings.push("No AGENTS.md or instruction surface found.".to_string());
    }
    if history.sampled_commits == 0 {
        warnings.push("No git history sample available.".to_string());
    }
    if inventory.file_count == 0 {
        warnings.push("No source files inventoried.".to_string());
    }
    warnings
}

fn git_remote_urls(repo: &Path) -> Vec<String> {
    git_output(repo, &["remote", "-v"])
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn git_output(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        return Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn top_counts(counts: BTreeMap<String, usize>, limit: usize) -> Vec<RepoSignalCount> {
    let mut items: Vec<_> = counts.into_iter().collect();
    items.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    items
        .into_iter()
        .take(limit)
        .map(|(label, count)| RepoSignalCount { label, count })
        .collect()
}

fn report_summary(report: &RepoTerrainReport) -> Value {
    json!({
        "repoId": report.repo_id,
        "name": report.name,
        "path": report.path,
        "families": report.source_families,
        "confidence": report.confidence,
        "dominantAxes": top_axes(&report.axis_scores, 5),
        "warnings": report.warnings,
    })
}

fn profile_summary(profile: &RepoPersonalityProfile) -> Value {
    json!({
        "repoId": profile.repo_id,
        "summary": profile.summary,
        "dominantPressures": profile.dominant_pressures,
        "riskPressures": profile.risk_pressures,
        "roleProjectionCount": profile.role_modulations.len(),
    })
}

fn report_agent_input(report: &RepoTerrainReport) -> Value {
    json!({
        "schemaVersion": report.schema_version,
        "repoId": report.repo_id,
        "name": report.name,
        "path": report.path,
        "remoteUrls": report.remote_urls,
        "sourceFamilies": report.source_families,
        "languages": report.languages,
        "stateSurfaces": report.state_surfaces,
        "instructionSurfaces": report.instruction_surfaces,
        "testSurfaces": report.test_surfaces,
        "runtimeSurfaces": report.runtime_surfaces,
        "historyMetrics": report.history_metrics,
        "axisScores": report.axis_scores,
        "axisEvidence": report.axis_evidence,
        "confidence": report.confidence,
        "warnings": report.warnings,
    })
}

fn profile_agent_input(profile: &RepoPersonalityProfile) -> Value {
    json!({
        "schemaVersion": profile.schema_version,
        "repoId": profile.repo_id,
        "summary": profile.summary,
        "sourceFamilyWeights": profile.source_family_weights,
        "axisScores": profile.axis_scores,
        "axisConfidence": profile.axis_confidence,
        "dominantPressures": profile.dominant_pressures,
        "riskPressures": profile.risk_pressures,
    })
}

fn role_projection_agent_input(projection: &RolePersonalityProjection) -> Value {
    json!({
        "schemaVersion": projection.schema_version,
        "projectionId": projection.projection_id,
        "repoId": projection.repo_id,
        "roleId": projection.role_id,
        "traitDeltas": projection.trait_deltas,
        "heartbeatDeltas": projection.heartbeat_deltas,
        "defaultMoodPressure": projection.default_mood_pressure,
        "semanticMemoryCandidates": projection.semantic_memory_candidates,
        "goalCandidates": projection.goal_candidates,
        "valueCandidates": projection.value_candidates,
        "privateNoteCandidates": projection.private_note_candidates,
        "reason": projection.reason,
        "evidenceRefs": projection.evidence_refs,
    })
}

fn render_scout_markdown(
    reports: &[RepoTerrainReport],
    profiles: &[RepoPersonalityProfile],
) -> String {
    let mut out = String::new();
    out.push_str("# Repo Personality Scout\n\n");
    out.push_str(&format!("- Reports: {}\n", reports.len()));
    out.push_str(&format!("- Profiles: {}\n\n", profiles.len()));
    for report in reports {
        out.push_str(&format!("## {}\n\n", report.name));
        out.push_str(&format!("- Path: `{}`\n", report.path));
        out.push_str(&format!(
            "- Families: {}\n",
            report.source_families.join(", ")
        ));
        out.push_str(&format!("- Confidence: {:.2}\n", report.confidence));
        out.push_str(&format!(
            "- Dominant axes: {}\n\n",
            top_axes(&report.axis_scores, 5).join(", ")
        ));
    }
    out
}

fn render_project_markdown(report: &RepoTerrainReport, profile: &RepoPersonalityProfile) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Repo Personality Projection: {}\n\n",
        report.name
    ));
    out.push_str(&format!("{}\n\n", profile.summary));
    out.push_str("## Risk Pressures\n\n");
    for risk in &profile.risk_pressures {
        out.push_str(&format!("- {risk}\n"));
    }
    out.push_str("\n## Role Projections\n\n");
    for projection in &profile.role_modulations {
        out.push_str(&format!("### {}\n\n", role_display(&projection.role_id)));
        out.push_str(&format!("{}\n\n", projection.reason));
        out.push_str(&format!(
            "- Trait deltas: `{}`\n",
            serde_json::to_string(&projection.trait_deltas).unwrap_or_default()
        ));
        out.push_str(&format!(
            "- Mood: `{}`\n\n",
            serde_json::to_string(&projection.default_mood_pressure).unwrap_or_default()
        ));
    }
    out
}

fn render_agent_packet_markdown(
    report: &RepoTerrainReport,
    profile: &RepoPersonalityProfile,
    role_projection_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Repo Personality Distiller Packet: {}\n\n",
        report.name
    ));
    out.push_str("This packet is input to the Repo Personality Distiller specialist. It is not accepted truth.\n\n");
    out.push_str("## Prompt\n\n");
    out.push_str("See `repo-personality-distiller-prompt.md`.\n\n");
    out.push_str("## Profile\n\n");
    out.push_str(&format!("- Repo id: `{}`\n", profile.repo_id));
    out.push_str(&format!("- Summary: {}\n", profile.summary));
    out.push_str(&format!(
        "- Dominant pressures: {}\n",
        profile.dominant_pressures.join(", ")
    ));
    out.push_str(&format!(
        "- Risk pressures: {}\n",
        profile.risk_pressures.join(", ")
    ));
    out.push_str(&format!(
        "- Role projections: {}\n\n",
        role_projection_count
    ));
    out.push_str("## Guardrails\n\n");
    out.push_str("- The distiller petitions Self; it does not mutate memory.\n");
    out.push_str("- The distiller is birth-only; after accepted initialization, personality drift belongs to heartbeat, mood, rumination, sleep consolidation, lived evidence, and reviewed selfPatch.\n");
    out.push_str("- Repo facts stay out of selfPatch and remain in terrain, graph, planning, evidence, or checkpoint artifacts.\n");
    out.push_str("- Personality pressure must be role-local, subtle, bounded, and reviewable.\n");
    out.push_str(
        "- Low confidence becomes a request for more terrain, not a permanent personality brand.\n",
    );
    out
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn write_text(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, value).with_context(|| format!("failed to write {}", path.display()))
}

fn repo_id(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("repo")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn same_path(left: &str, right: &Path) -> bool {
    PathBuf::from(left)
        .canonicalize()
        .ok()
        .zip(right.canonicalize().ok())
        .is_some_and(|(left, right)| left == right)
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn print_usage() {
    eprintln!(
        "usage: epiphany-repo-personality <scout|project|agent-packet|status> ...\n\
         scout --root <path> --artifact-dir <path> [--max-repos <n>]\n\
         project --repo <path> --baseline <baseline.msgpack> --artifact-dir <path>\n\
         agent-packet --store <projection.msgpack> --artifact-dir <path> [--repo-id <id>]\n\
         status --store <baseline-or-projection.msgpack>"
    );
}
