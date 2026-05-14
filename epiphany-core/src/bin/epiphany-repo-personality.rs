use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use cultcache_rs::CultCache;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use epiphany_core::AgentCanonicalTraitSeed;
use epiphany_core::AgentSelfPatch;
use epiphany_core::apply_agent_canonical_trait_seeds;
use epiphany_core::apply_agent_self_patch_document;
use epiphany_core::default_heartbeat_state;
use epiphany_core::load_heartbeat_state_entry;
use epiphany_core::review_agent_self_patch_document;
use epiphany_core::write_heartbeat_state_entry;
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
const TRAJECTORY_SCHEMA_VERSION: &str = "epiphany.repo_trajectory_report.v0";
const ARTIFACT_SCHEMA_VERSION: &str = "epiphany.repo_personality_artifacts.v0";
const DISTILLER_PACKET_SCHEMA_VERSION: &str = "epiphany.repo_personality_distiller_packet.v0";
const MEMORY_DISTILLER_PACKET_SCHEMA_VERSION: &str = "epiphany.repo_memory_distiller_packet.v0";
const TRAJECTORY_DISTILLER_PACKET_SCHEMA_VERSION: &str =
    "epiphany.repo_trajectory_distiller_packet.v0";
const INITIALIZATION_RECORD_SCHEMA_VERSION: &str = "epiphany.repo_initialization_record.v0";
const REPO_PERSONALITY_DISTILLER_PROMPT: &str =
    include_str!("../prompts/repo_personality_distiller.md");
const REPO_MEMORY_DISTILLER_PROMPT: &str = include_str!("../prompts/repo_memory_distiller.md");
const REPO_TRAJECTORY_DISTILLER_PROMPT: &str =
    include_str!("../prompts/repo_trajectory_distiller.md");
const MEMORY_SOURCE_MAX_FILES: usize = 48;
const MEMORY_SOURCE_MAX_BYTES_PER_FILE: usize = 12_000;
const MEMORY_SOURCE_MAX_TOTAL_BYTES: usize = 120_000;
const TRAJECTORY_SOURCE_MAX_FILES: usize = 18;
const TRAJECTORY_SOURCE_MAX_BYTES_PER_FILE: usize = 9_000;
const TRAJECTORY_SOURCE_MAX_TOTAL_BYTES: usize = 72_000;

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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrajectorySourceExcerpt {
    pub path: String,
    pub kind: String,
    pub bytes: usize,
    pub truncated: bool,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoTrajectoryThemeScore {
    pub theme: String,
    pub early_history: f64,
    pub recent_history: f64,
    pub current_sources: f64,
    pub delta: f64,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repo_trajectory_report",
    schema = "EpiphanyRepoTrajectoryReport"
)]
pub struct RepoTrajectoryReport {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub repo_id: String,
    #[cultcache(key = 2)]
    pub trajectory_summary: String,
    #[cultcache(key = 3)]
    pub self_image: String,
    #[cultcache(key = 4)]
    pub early_commit_messages: Vec<String>,
    #[cultcache(key = 5)]
    pub recent_commit_messages: Vec<String>,
    #[cultcache(key = 6)]
    pub trajectory_sources: Vec<TrajectorySourceExcerpt>,
    #[cultcache(key = 7)]
    pub theme_scores: Vec<RepoTrajectoryThemeScore>,
    #[cultcache(key = 8)]
    pub directional_pressures: Vec<String>,
    #[cultcache(key = 9)]
    pub implicit_goal_candidates: Vec<String>,
    #[cultcache(key = 10)]
    pub anti_goal_candidates: Vec<String>,
    #[cultcache(key = 11)]
    pub tensions: Vec<String>,
    #[cultcache(key = 12)]
    pub confidence: f64,
    #[cultcache(key = 13)]
    pub warnings: Vec<String>,
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

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repo_initialization_record",
    schema = "EpiphanyRepoInitializationRecord"
)]
pub struct RepoInitializationRecord {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub record_id: String,
    #[cultcache(key = 2)]
    pub repo_id: String,
    #[cultcache(key = 3)]
    pub repo_path: String,
    #[cultcache(key = 4)]
    pub kind: String,
    #[cultcache(key = 5)]
    pub source_packet_schema_version: String,
    #[cultcache(key = 6)]
    pub source_packet_path: String,
    #[cultcache(key = 7)]
    pub accepted_at: String,
    #[cultcache(key = 8)]
    pub accepted_by: String,
    #[cultcache(key = 9)]
    pub summary: String,
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
        "agent-packet" => run_agent_packet(parse_packet_args(args, "agent-packet")?),
        "memory-packet" => run_memory_packet(parse_packet_args(args, "memory-packet")?),
        "trajectory-packet" => run_trajectory_packet(parse_packet_args(args, "trajectory-packet")?),
        "startup" => run_startup(parse_startup_args(args)?),
        "accept-init" => run_accept_init(parse_accept_init_args(args)?),
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
struct StartupArgs {
    repo: PathBuf,
    baseline: PathBuf,
    artifact_dir: PathBuf,
    init_store: PathBuf,
}

#[derive(Clone, Debug)]
struct AcceptInitArgs {
    init_store: PathBuf,
    packet: PathBuf,
    kind: String,
    accepted_by: String,
    summary: Option<String>,
    result: Option<PathBuf>,
    agent_store: Option<PathBuf>,
    apply_self_patches: bool,
    apply_trait_seeds: bool,
    heartbeat_store: Option<PathBuf>,
    apply_heartbeat_seeds: bool,
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MemorySourceExcerpt {
    path: String,
    kind: String,
    bytes: usize,
    truncated: bool,
    text: String,
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

fn parse_startup_args(args: impl Iterator<Item = String>) -> Result<StartupArgs> {
    let mut repo = None;
    let mut baseline = None;
    let mut artifact_dir = None;
    let mut init_store = None;
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
        "--init-store" => {
            init_store = Some(PathBuf::from(value));
            Ok(())
        }
        other => Err(anyhow!("unexpected startup argument {other}")),
    })?;
    Ok(StartupArgs {
        repo: repo.context("missing --repo")?,
        baseline: baseline.context("missing --baseline")?,
        artifact_dir: artifact_dir.context("missing --artifact-dir")?,
        init_store: init_store.context("missing --init-store")?,
    })
}

fn parse_accept_init_args(args: impl Iterator<Item = String>) -> Result<AcceptInitArgs> {
    let mut init_store = None;
    let mut packet = None;
    let mut kind = None;
    let mut accepted_by = Some("Self".to_string());
    let mut summary = None;
    let mut result = None;
    let mut agent_store = None;
    let mut apply_self_patches = false;
    let mut apply_trait_seeds = false;
    let mut heartbeat_store = None;
    let mut apply_heartbeat_seeds = false;
    parse_named_args(args, |name, value| match name {
        "--init-store" => {
            init_store = Some(PathBuf::from(value));
            Ok(())
        }
        "--packet" => {
            packet = Some(PathBuf::from(value));
            Ok(())
        }
        "--kind" => {
            kind = Some(value);
            Ok(())
        }
        "--accepted-by" => {
            accepted_by = Some(value);
            Ok(())
        }
        "--summary" => {
            summary = Some(value);
            Ok(())
        }
        "--result" => {
            result = Some(PathBuf::from(value));
            Ok(())
        }
        "--agent-store" => {
            agent_store = Some(PathBuf::from(value));
            Ok(())
        }
        "--apply-self-patches" => {
            apply_self_patches = parse_bool_arg("--apply-self-patches", &value)?;
            Ok(())
        }
        "--apply-trait-seeds" => {
            apply_trait_seeds = parse_bool_arg("--apply-trait-seeds", &value)?;
            Ok(())
        }
        "--heartbeat-store" => {
            heartbeat_store = Some(PathBuf::from(value));
            Ok(())
        }
        "--apply-heartbeat-seeds" => {
            apply_heartbeat_seeds = parse_bool_arg("--apply-heartbeat-seeds", &value)?;
            Ok(())
        }
        other => Err(anyhow!("unexpected accept-init argument {other}")),
    })?;
    Ok(AcceptInitArgs {
        init_store: init_store.context("missing --init-store")?,
        packet: packet.context("missing --packet")?,
        kind: kind.context("missing --kind")?,
        accepted_by: accepted_by.unwrap_or_else(|| "Self".to_string()),
        summary,
        result,
        agent_store,
        apply_self_patches,
        apply_trait_seeds,
        heartbeat_store,
        apply_heartbeat_seeds,
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

fn parse_packet_args(
    args: impl Iterator<Item = String>,
    command_name: &'static str,
) -> Result<AgentPacketArgs> {
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
        other => Err(anyhow!("unexpected {command_name} argument {other}")),
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

fn parse_bool_arg(name: &str, value: &str) -> Result<bool> {
    match value {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        other => Err(anyhow!("{name} must be true or false, got {other:?}")),
    }
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
    let mut trajectories = Vec::new();
    for profile in &profiles {
        let report = reports
            .iter()
            .find(|report| report.repo_id == profile.repo_id)
            .expect("profile should map to report");
        let trajectory = derive_trajectory_report(Path::new(&report.path), report, profile)?;
        cache.put::<RepoTrajectoryReport>(trajectory.repo_id.clone(), &trajectory)?;
        trajectories.push(trajectory);
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
        "trajectoryCount": trajectories.len(),
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
        &render_scout_markdown(&reports, &profiles, &trajectories),
    )?;
    Ok(summary)
}

fn run_project(args: ProjectArgs) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let ProjectedRepo {
        store_path,
        report,
        profile,
        trajectory,
    } = project_repo_to_store(&args.repo, &args.baseline, &args.artifact_dir)?;
    let summary = json!({
        "schemaVersion": ARTIFACT_SCHEMA_VERSION,
        "mode": "project",
        "repo": args.repo,
        "baseline": args.baseline,
        "artifactDir": args.artifact_dir,
        "store": store_path,
        "profile": profile_summary(&profile),
        "terrain": report_summary(&report),
        "trajectory": trajectory_summary(&trajectory),
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
        &render_project_markdown(&report, &profile, &trajectory),
    )?;
    Ok(summary)
}

struct ProjectedRepo {
    store_path: PathBuf,
    report: RepoTerrainReport,
    profile: RepoPersonalityProfile,
    trajectory: RepoTrajectoryReport,
}

fn project_repo_to_store(
    repo: &Path,
    baseline_path: &Path,
    artifact_dir: &Path,
) -> Result<ProjectedRepo> {
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let mut baseline = repo_personality_cache(baseline_path)?;
    baseline.pull_all_backing_stores()?;
    let reports = baseline.get_all::<RepoTerrainReport>()?;
    let target_id = repo_id(repo);
    let report = reports
        .iter()
        .find(|report| report.repo_id == target_id || same_path(&report.path, repo))
        .cloned()
        .unwrap_or_else(|| scout_repo(repo).expect("target repo should be scoutable"));
    let profile = reduce_report(&report);
    let trajectory = derive_trajectory_report(repo, &report, &profile)?;

    let store_path = artifact_dir.join("projection.msgpack");
    let mut cache = repo_personality_cache(&store_path)?;
    cache.pull_all_backing_stores()?;
    cache.put::<RepoTerrainReport>(report.repo_id.clone(), &report)?;
    cache.put::<RepoPersonalityProfile>(profile.repo_id.clone(), &profile)?;
    cache.put::<RepoTrajectoryReport>(trajectory.repo_id.clone(), &trajectory)?;
    for projection in &profile.role_modulations {
        cache.put::<RolePersonalityProjection>(projection.projection_id.clone(), projection)?;
    }
    Ok(ProjectedRepo {
        store_path,
        report,
        profile,
        trajectory,
    })
}

fn run_status(args: StatusArgs) -> Result<Value> {
    let mut cache = repo_personality_cache(&args.store)?;
    cache.pull_all_backing_stores()?;
    let reports = cache.get_all::<RepoTerrainReport>()?;
    let profiles = cache.get_all::<RepoPersonalityProfile>()?;
    let trajectories = cache.get_all::<RepoTrajectoryReport>()?;
    let projections = cache.get_all::<RolePersonalityProjection>()?;
    Ok(json!({
        "schemaVersion": ARTIFACT_SCHEMA_VERSION,
        "mode": "status",
        "store": args.store,
        "reports": reports.len(),
        "profiles": profiles.len(),
        "trajectoryReports": trajectories.len(),
        "roleProjections": projections.len(),
        "repos": reports.iter().map(report_summary).collect::<Vec<_>>(),
    }))
}

fn run_startup(args: StartupArgs) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let target_repo_id = repo_id(&args.repo);
    let accepted = accepted_initialization_kinds(&args.init_store, &target_repo_id)?;
    let personality_ready = accepted.contains("repo-personality");
    let memory_ready = accepted.contains("repo-memory");
    let trajectory_ready = accepted.contains("repo-trajectory");
    let mut generated_packets = Vec::new();
    let mut required_actions = Vec::new();
    let mut projection_store = None;

    if !personality_ready || !memory_ready || !trajectory_ready {
        let projection_dir = args.artifact_dir.join("projection");
        let projected = project_repo_to_store(&args.repo, &args.baseline, &projection_dir)?;
        projection_store = Some(projected.store_path.clone());
        if !trajectory_ready {
            let result = run_trajectory_packet(AgentPacketArgs {
                store: projected.store_path.clone(),
                artifact_dir: args.artifact_dir.join("repo-trajectory"),
                repo_id: Some(projected.profile.repo_id.clone()),
            })?;
            required_actions.push("reviewRepoTrajectoryInitialization");
            generated_packets.push(json!({
                "kind": "repo-trajectory",
                "birthOnly": true,
                "executionOwner": "repo-initialization-startup-runner",
                "heartbeatParticipant": Value::Null,
                "contract": "Startup-only specialist packet. Do not register as a heartbeat lane; accept-init may apply reviewed role selfPatch requests only after Self review.",
                "packetPath": result["packetPath"],
                "promptPath": result["promptPath"],
                "summaryPath": result["summaryPath"],
            }));
        }
        if !personality_ready {
            let result = run_agent_packet(AgentPacketArgs {
                store: projected.store_path.clone(),
                artifact_dir: args.artifact_dir.join("repo-personality"),
                repo_id: Some(projected.profile.repo_id.clone()),
            })?;
            required_actions.push("reviewRepoPersonalityInitialization");
            generated_packets.push(json!({
                "kind": "repo-personality",
                "birthOnly": true,
                "executionOwner": "repo-initialization-startup-runner",
                "heartbeatParticipant": Value::Null,
                "contract": "Startup-only specialist packet. Do not register as a heartbeat lane; accept-init may stamp the newborn trait lattice and seed heartbeat physiology only after Self review.",
                "packetPath": result["packetPath"],
                "promptPath": result["promptPath"],
                "summaryPath": result["summaryPath"],
            }));
        }
        if !memory_ready {
            let result = run_memory_packet(AgentPacketArgs {
                store: projected.store_path.clone(),
                artifact_dir: args.artifact_dir.join("repo-memory"),
                repo_id: Some(projected.profile.repo_id.clone()),
            })?;
            required_actions.push("reviewRepoMemoryInitialization");
            generated_packets.push(json!({
                "kind": "repo-memory",
                "birthOnly": true,
                "executionOwner": "repo-initialization-startup-runner",
                "heartbeatParticipant": Value::Null,
                "contract": "Startup-only specialist packet. Do not register as a heartbeat lane; accept-init may apply reviewed role selfPatch requests only after Self review.",
                "packetPath": result["packetPath"],
                "promptPath": result["promptPath"],
                "summaryPath": result["summaryPath"],
            }));
        }
    }

    let action = if generated_packets.is_empty() {
        "continueStartup"
    } else {
        "reviewInitializationPackets"
    };
    let result = json!({
        "schemaVersion": ARTIFACT_SCHEMA_VERSION,
        "mode": "startup",
        "action": action,
        "repoId": target_repo_id,
        "repo": args.repo,
        "baseline": args.baseline,
        "artifactDir": args.artifact_dir,
        "initStore": args.init_store,
        "acceptedKinds": accepted,
            "missingKinds": initialization_kinds()
            .iter()
            .filter(|kind| !accepted.iter().any(|accepted| accepted == *kind))
            .copied()
            .collect::<Vec<_>>(),
        "projectionStore": projection_store,
        "generatedPackets": generated_packets,
        "requiredActions": required_actions,
        "requiresReview": action == "reviewInitializationPackets",
        "nextSafeMove": if action == "continueStartup" {
            "Startup birth records are accepted; leave later personality, trajectory, and memory movement to heartbeat, mood, evidence, sleep, and reviewed selfPatch."
        } else {
            "Self reviews generated birth packets, runs the startup-only distiller specialists outside the heartbeat lane system, applies accepted selfPatch candidates, trait-lattice seeds, and heartbeat seeds through accept-init, then records each completed birth rite."
        }
    });
    write_json(
        &args
            .artifact_dir
            .join("repo-initialization-startup-summary.json"),
        &result,
    )?;
    write_text(
        &args
            .artifact_dir
            .join("repo-initialization-startup-summary.md"),
        &render_startup_markdown(&result),
    )?;
    Ok(result)
}

fn run_accept_init(args: AcceptInitArgs) -> Result<Value> {
    let raw = fs::read_to_string(&args.packet)
        .with_context(|| format!("failed to read {}", args.packet.display()))?;
    let packet: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to decode {}", args.packet.display()))?;
    let repo_id = packet
        .get("repoId")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("packet has no repoId"))?;
    let repo_path = packet
        .pointer("/input/repoTerrainReport/path")
        .and_then(Value::as_str)
        .unwrap_or("");
    let packet_schema = packet
        .get("schemaVersion")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("packet has no schemaVersion"))?;
    validate_initialization_kind(&args.kind, packet_schema)?;
    let trait_lattice_persistence = process_trait_lattice_seed_patches(
        &args.kind,
        &packet,
        args.agent_store.as_deref(),
        args.apply_trait_seeds,
    )?;
    let heartbeat_seeds = process_heartbeat_seed_patches(
        &args.kind,
        &packet,
        args.heartbeat_store.as_deref(),
        args.apply_heartbeat_seeds,
    )?;
    let self_persistence = if let Some(result_path) = &args.result {
        let raw = fs::read_to_string(result_path)
            .with_context(|| format!("failed to read {}", result_path.display()))?;
        let result: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to decode {}", result_path.display()))?;
        process_initialization_result_patches(
            &args.kind,
            &result,
            args.agent_store.as_deref(),
            args.apply_self_patches,
        )?
    } else {
        json!({
            "resultPath": Value::Null,
            "agentStore": args.agent_store,
            "applySelfPatches": args.apply_self_patches,
            "patches": [],
            "accepted": 0,
            "rejected": 0,
            "applied": 0,
            "status": "not-provided"
        })
    };
    let record = RepoInitializationRecord {
        schema_version: INITIALIZATION_RECORD_SCHEMA_VERSION.to_string(),
        record_id: initialization_record_id(repo_id, &args.kind),
        repo_id: repo_id.to_string(),
        repo_path: repo_path.to_string(),
        kind: args.kind.clone(),
        source_packet_schema_version: packet_schema.to_string(),
        source_packet_path: args.packet.display().to_string(),
        accepted_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        accepted_by: args.accepted_by,
        summary: args.summary.unwrap_or_else(|| {
            format!(
                "{} birth packet accepted for {}; detailed selfPatch application remains separately review-gated.",
                args.kind, repo_id
            )
        }),
    };
    let mut cache = repo_initialization_cache(&args.init_store)?;
    cache.pull_all_backing_stores()?;
    cache.put(record.record_id.clone(), &record)?;
    Ok(json!({
        "schemaVersion": INITIALIZATION_RECORD_SCHEMA_VERSION,
        "mode": "accept-init",
        "initStore": args.init_store,
        "resultPath": args.result,
        "agentStore": args.agent_store,
        "applySelfPatches": args.apply_self_patches,
        "applyTraitSeeds": args.apply_trait_seeds,
        "heartbeatStore": args.heartbeat_store,
        "applyHeartbeatSeeds": args.apply_heartbeat_seeds,
        "traitLatticePersistence": trait_lattice_persistence,
        "heartbeatSeeds": heartbeat_seeds,
        "selfPersistence": self_persistence,
        "record": initialization_record_summary(&record),
    }))
}

fn run_trajectory_packet(args: AgentPacketArgs) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let mut cache = repo_personality_cache(&args.store)?;
    cache.pull_all_backing_stores()?;
    let reports = cache.get_all::<RepoTerrainReport>()?;
    let profiles = cache.get_all::<RepoPersonalityProfile>()?;
    let trajectories = cache.get_all::<RepoTrajectoryReport>()?;
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
    let trajectory = trajectories
        .iter()
        .find(|trajectory| trajectory.repo_id == profile.repo_id)
        .ok_or_else(|| {
            anyhow!(
                "store has profile {:?} but no trajectory report",
                profile.repo_id
            )
        })?;
    let role_projections: Vec<_> = projections
        .iter()
        .filter(|projection| projection.repo_id == profile.repo_id)
        .collect();
    let packet = json!({
        "schemaVersion": TRAJECTORY_DISTILLER_PACKET_SCHEMA_VERSION,
        "createdAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "store": args.store,
        "repoId": profile.repo_id,
        "lifecycle": {
            "mode": "birth-only",
            "contract": "Run this specialist only when the repo/swarm has no accepted trajectory initialization. Later direction drift belongs to heartbeat, mood, lived work, reviewed selfPatch, and planning/evidence truth.",
            "rerunPolicy": "If accepted trajectory initialization exists, do not rerun to rebrand the repo. Route contradictions through normal Eyes/Body/Imagination/Soul work and reviewed memory drift."
        },
        "prompt": REPO_TRAJECTORY_DISTILLER_PROMPT,
        "input": {
            "repoTerrainReport": report_agent_input(report),
            "repoPersonalityProfile": profile_agent_input(profile),
            "repoTrajectoryReport": trajectory_agent_input(trajectory),
            "rolePersonalityProjections": role_projections
                .iter()
                .map(|projection| role_projection_agent_input(projection))
                .collect::<Vec<_>>(),
        },
        "expectedOutput": {
            "verdict": "ready-for-review | needs-more-history | reject",
            "summary": "short repo trajectory summary",
            "confidence": "0.0..1.0",
            "selfImage": "how the repo understands its own becoming",
            "trajectoryNarrative": "how the repo has been moving over time",
            "implicitGoals": [],
            "antiGoals": [],
            "roleBiases": [],
            "selfPatchCandidates": [],
            "initializationRecord": {
                "repoId": profile.repo_id,
                "terrainSchemaVersion": report.schema_version,
                "profileSchemaVersion": profile.schema_version,
                "trajectorySchemaVersion": trajectory.schema_version,
                "acceptedOnce": true,
                "distillerKind": "repo-trajectory"
            },
            "doNotMutate": [],
            "nextSafeMove": "Self reviews trajectory-derived self-image and implicit-goal pressure before first mutation; later drift belongs to lived work, heartbeat, planning, and reviewed selfPatch."
        },
        "guardrails": [
            "This packet is input to a startup-only trajectory distiller, not accepted truth.",
            "Trajectory is directional bias, not a prison sentence or an active objective.",
            "Do not dump commit logs, file lists, or project facts into role memory.",
            "Keep trajectory pressure role-local, reviewable, and subordinate to later lived drift.",
            "No authority claims, code edits, raw transcripts, or cross-workspace instructions in selfPatch."
        ]
    });
    let packet_path = args
        .artifact_dir
        .join("repo-trajectory-distiller-packet.json");
    let prompt_path = args
        .artifact_dir
        .join("repo-trajectory-distiller-prompt.md");
    let summary_path = args
        .artifact_dir
        .join("repo-trajectory-distiller-packet.md");
    write_json(&packet_path, &packet)?;
    write_text(&prompt_path, REPO_TRAJECTORY_DISTILLER_PROMPT)?;
    write_text(
        &summary_path,
        &render_trajectory_packet_markdown(report, profile, trajectory, role_projections.len()),
    )?;
    Ok(json!({
        "schemaVersion": TRAJECTORY_DISTILLER_PACKET_SCHEMA_VERSION,
        "mode": "trajectory-packet",
        "repoId": profile.repo_id,
        "artifactDir": args.artifact_dir,
        "packetPath": packet_path,
        "promptPath": prompt_path,
        "summaryPath": summary_path,
        "roleProjectionCount": role_projections.len(),
        "trajectoryThemeCount": trajectory.theme_scores.len(),
    }))
}

fn run_agent_packet(args: AgentPacketArgs) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let mut cache = repo_personality_cache(&args.store)?;
    cache.pull_all_backing_stores()?;
    let reports = cache.get_all::<RepoTerrainReport>()?;
    let profiles = cache.get_all::<RepoPersonalityProfile>()?;
    let trajectories = cache.get_all::<RepoTrajectoryReport>()?;
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
    let trajectory = trajectories
        .iter()
        .find(|trajectory| trajectory.repo_id == profile.repo_id)
        .ok_or_else(|| {
            anyhow!(
                "store has profile {:?} but no trajectory report",
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
            "repoTrajectoryReport": trajectory_agent_input(trajectory),
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
        &render_agent_packet_markdown(report, profile, trajectory, role_projections.len()),
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

fn run_memory_packet(args: AgentPacketArgs) -> Result<Value> {
    fs::create_dir_all(&args.artifact_dir)
        .with_context(|| format!("failed to create {}", args.artifact_dir.display()))?;
    let mut cache = repo_personality_cache(&args.store)?;
    cache.pull_all_backing_stores()?;
    let reports = cache.get_all::<RepoTerrainReport>()?;
    let profiles = cache.get_all::<RepoPersonalityProfile>()?;
    let trajectories = cache.get_all::<RepoTrajectoryReport>()?;
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
    let trajectory = trajectories
        .iter()
        .find(|trajectory| trajectory.repo_id == profile.repo_id)
        .ok_or_else(|| {
            anyhow!(
                "store has profile {:?} but no trajectory report",
                profile.repo_id
            )
        })?;
    let role_projections: Vec<_> = projections
        .iter()
        .filter(|projection| projection.repo_id == profile.repo_id)
        .collect();
    let repo_path = PathBuf::from(&report.path);
    let memory_sources = collect_memory_sources(&repo_path, report)?;
    let role_briefs = memory_role_briefs(report, profile);
    let packet = json!({
        "schemaVersion": MEMORY_DISTILLER_PACKET_SCHEMA_VERSION,
        "createdAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "store": args.store,
        "repoId": profile.repo_id,
        "lifecycle": {
            "mode": "birth-only",
            "contract": "Run this specialist only when the repo/swarm has no accepted memory initialization. Later memory growth belongs to heartbeat, work evidence, rumination, sleep consolidation, and reviewed selfPatch.",
            "rerunPolicy": "If accepted memory initialization exists, do not rerun to refresh memory. Route stale or contradicted knowledge through normal Eyes/Body/Soul state, evidence, and sleep consolidation."
        },
        "prompt": REPO_MEMORY_DISTILLER_PROMPT,
        "input": {
            "repoTerrainReport": report_agent_input(report),
            "repoPersonalityProfile": profile_agent_input(profile),
            "repoTrajectoryReport": trajectory_agent_input(trajectory),
            "rolePersonalityProjections": role_projections
                .iter()
                .map(|projection| role_projection_agent_input(projection))
                .collect::<Vec<_>>(),
            "roleMemoryDistillerBriefs": role_briefs,
            "memorySourceInventory": {
                "repoPath": report.path,
                "sourceCount": memory_sources.len(),
                "maxFiles": MEMORY_SOURCE_MAX_FILES,
                "maxBytesPerFile": MEMORY_SOURCE_MAX_BYTES_PER_FILE,
                "maxTotalBytes": MEMORY_SOURCE_MAX_TOTAL_BYTES,
                "sourceKinds": memory_sources
                    .iter()
                    .map(|source| source.kind.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            },
            "memorySources": memory_sources,
            "recentHistory": report.history_metrics.recent_messages,
        },
        "expectedOutput": {
            "verdict": "ready-for-review | needs-more-source | reject",
            "summary": "short newborn memory initialization summary",
            "confidence": "0.0..1.0",
            "roleMemoryPatches": [],
            "globalMemoryCandidates": [],
            "initializationRecord": {
                "repoId": profile.repo_id,
                "terrainSchemaVersion": report.schema_version,
                "profileSchemaVersion": profile.schema_version,
                "acceptedOnce": true,
                "distillerKind": "repo-memory"
            },
            "doNotMutate": [],
            "nextSafeMove": "Self reviews role-specific memory patches before first initialization mutation; later memory growth uses heartbeat, evidence, rumination, sleep, and reviewed selfPatch."
        },
        "guardrails": [
            "This packet is input to role-specific memory distillers, not accepted truth.",
            "This packet is birth-only; it pre-fills newborn memory and then gets out of the way.",
            "Memory distillation is separate from personality distillation.",
            "Each role receives only knowledge relevant to its mission.",
            "Do not copy raw file dumps into memory; compress into durable doctrine, source maps, invariants, risks, and practices.",
            "Preserve uncertainty and staleness risk; repository documentation can lie by age.",
            "No objectives, authority claims, active job state, raw transcripts, code edits, or arbitrary workspace access in selfPatch."
        ]
    });
    let packet_path = args.artifact_dir.join("repo-memory-distiller-packet.json");
    let prompt_path = args.artifact_dir.join("repo-memory-distiller-prompt.md");
    let summary_path = args.artifact_dir.join("repo-memory-distiller-packet.md");
    let source_count = packet["input"]["memorySourceInventory"]["sourceCount"]
        .as_u64()
        .unwrap_or_default();
    write_json(&packet_path, &packet)?;
    write_text(&prompt_path, REPO_MEMORY_DISTILLER_PROMPT)?;
    write_text(
        &summary_path,
        &render_memory_packet_markdown(report, profile, trajectory, source_count as usize),
    )?;
    Ok(json!({
        "schemaVersion": MEMORY_DISTILLER_PACKET_SCHEMA_VERSION,
        "mode": "memory-packet",
        "repoId": profile.repo_id,
        "artifactDir": args.artifact_dir,
        "packetPath": packet_path,
        "promptPath": prompt_path,
        "summaryPath": summary_path,
        "memorySourceCount": source_count,
        "roleDistillerCount": ROLES.len(),
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
    cache.register_entry_type::<RepoTrajectoryReport>()?;
    cache.register_entry_type::<RolePersonalityProjection>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(path));
    Ok(cache)
}

fn repo_initialization_cache(path: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<RepoInitializationRecord>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(path));
    Ok(cache)
}

fn initialization_kinds() -> &'static [&'static str] {
    &["repo-trajectory", "repo-personality", "repo-memory"]
}

fn accepted_initialization_kinds(store: &Path, repo_id: &str) -> Result<BTreeSet<String>> {
    if !store.exists() {
        return Ok(BTreeSet::new());
    }
    let mut cache = repo_initialization_cache(store)?;
    cache.pull_all_backing_stores()?;
    let mut accepted = BTreeSet::new();
    for kind in initialization_kinds() {
        if cache
            .get::<RepoInitializationRecord>(&initialization_record_id(repo_id, kind))?
            .is_some()
        {
            accepted.insert((*kind).to_string());
        }
    }
    Ok(accepted)
}

fn initialization_record_id(repo_id: &str, kind: &str) -> String {
    format!("{repo_id}::{kind}")
}

fn validate_initialization_kind(kind: &str, packet_schema: &str) -> Result<()> {
    match (kind, packet_schema) {
        ("repo-trajectory", TRAJECTORY_DISTILLER_PACKET_SCHEMA_VERSION) => Ok(()),
        ("repo-personality", DISTILLER_PACKET_SCHEMA_VERSION) => Ok(()),
        ("repo-memory", MEMORY_DISTILLER_PACKET_SCHEMA_VERSION) => Ok(()),
        ("repo-trajectory", other) => Err(anyhow!(
            "repo-trajectory init requires packet schema {:?}, got {:?}",
            TRAJECTORY_DISTILLER_PACKET_SCHEMA_VERSION,
            other
        )),
        ("repo-personality", other) => Err(anyhow!(
            "repo-personality init requires packet schema {:?}, got {:?}",
            DISTILLER_PACKET_SCHEMA_VERSION,
            other
        )),
        ("repo-memory", other) => Err(anyhow!(
            "repo-memory init requires packet schema {:?}, got {:?}",
            MEMORY_DISTILLER_PACKET_SCHEMA_VERSION,
            other
        )),
        (other, _) => Err(anyhow!(
            "unknown initialization kind {other:?}; expected repo-trajectory, repo-personality, or repo-memory"
        )),
    }
}

fn process_initialization_result_patches(
    kind: &str,
    result: &Value,
    agent_store: Option<&Path>,
    apply_self_patches: bool,
) -> Result<Value> {
    let patches = extract_initialization_self_patches(kind, result)?;
    if patches.is_empty() {
        return Ok(json!({
            "status": "no-patches",
            "applySelfPatches": apply_self_patches,
            "patches": [],
            "accepted": 0,
            "rejected": 0,
            "applied": 0,
        }));
    }
    let store = agent_store.ok_or_else(|| {
        anyhow!("--agent-store is required when --result contains selfPatch candidates")
    })?;
    let mut reviews = Vec::new();
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    let mut applied = 0usize;
    for patch in patches {
        let review = if apply_self_patches {
            apply_agent_self_patch_document(&patch.role_id, patch.self_patch.clone(), store)?
        } else {
            review_agent_self_patch_document(&patch.role_id, &patch.self_patch, store)
        };
        if review.status == "accepted" {
            accepted += 1;
        } else {
            rejected += 1;
        }
        if review.applied == Some(true) {
            applied += 1;
        }
        reviews.push(json!({
            "roleId": patch.role_id,
            "source": patch.source,
            "review": review,
        }));
    }
    Ok(json!({
        "status": if rejected == 0 { "accepted" } else { "review-blocked" },
        "agentStore": store,
        "applySelfPatches": apply_self_patches,
        "patches": reviews,
        "accepted": accepted,
        "rejected": rejected,
        "applied": applied,
    }))
}

fn process_trait_lattice_seed_patches(
    kind: &str,
    packet: &Value,
    agent_store: Option<&Path>,
    apply_trait_seeds: bool,
) -> Result<Value> {
    if kind != "repo-personality" {
        return Ok(json!({
            "status": "not-applicable",
            "applyTraitSeeds": apply_trait_seeds,
            "seeds": [],
            "applied": 0,
        }));
    }
    let seeds = extract_trait_seed_candidates(packet)?;
    if seeds.is_empty() {
        return Ok(json!({
            "status": "no-seeds",
            "applyTraitSeeds": apply_trait_seeds,
            "seeds": [],
            "applied": 0,
        }));
    }
    if !apply_trait_seeds {
        return Ok(json!({
            "status": "review-only",
            "applyTraitSeeds": false,
            "seeds": seeds,
            "applied": 0,
            "contract": "Pass --agent-store and --apply-trait-seeds true after Self review to stamp the newborn Ghostlight trait lattice."
        }));
    }
    let store = agent_store
        .ok_or_else(|| anyhow!("--agent-store is required with --apply-trait-seeds true"))?;
    let applied = apply_agent_canonical_trait_seeds(&seeds, store)?;
    Ok(json!({
        "status": "applied",
        "agentStore": store,
        "applyTraitSeeds": true,
        "applied": applied["applied"].clone(),
        "seeds": applied["seeds"].clone(),
    }))
}

fn process_heartbeat_seed_patches(
    kind: &str,
    packet: &Value,
    heartbeat_store: Option<&Path>,
    apply_heartbeat_seeds: bool,
) -> Result<Value> {
    if kind != "repo-personality" {
        return Ok(json!({
            "status": "not-applicable",
            "applyHeartbeatSeeds": apply_heartbeat_seeds,
            "seeds": [],
            "applied": 0,
        }));
    }
    let seeds = extract_heartbeat_seed_candidates(packet)?;
    if seeds.is_empty() {
        return Ok(json!({
            "status": "no-seeds",
            "applyHeartbeatSeeds": apply_heartbeat_seeds,
            "seeds": [],
            "applied": 0,
        }));
    }
    if !apply_heartbeat_seeds {
        return Ok(json!({
            "status": "review-only",
            "applyHeartbeatSeeds": false,
            "seeds": seeds,
            "applied": 0,
            "contract": "Pass --heartbeat-store and --apply-heartbeat-seeds true after Self review to mutate heartbeat physiology."
        }));
    }
    let store = heartbeat_store.ok_or_else(|| {
        anyhow!("--heartbeat-store is required with --apply-heartbeat-seeds true")
    })?;
    let mut state =
        load_heartbeat_state_entry(store)?.unwrap_or_else(|| default_heartbeat_state(1.0));
    let mut applied = Vec::new();
    for seed in seeds {
        let Some(participant) = state
            .participants
            .iter_mut()
            .find(|participant| participant.role_id == seed.role_id)
        else {
            applied.push(json!({
                "roleId": seed.role_id,
                "status": "missing-participant",
            }));
            continue;
        };
        let previous_initiative_speed = participant.initiative_speed;
        let previous_reaction_bias = participant.reaction_bias;
        let previous_cooldown = participant
            .extra
            .get("personalityCooldownMultiplier")
            .and_then(Value::as_f64)
            .unwrap_or(1.0);
        let initiative_delta = number_from_map(&seed.heartbeat_deltas, "initiativeSpeedDelta");
        let cooldown_delta = number_from_map(&seed.heartbeat_deltas, "cooldownMultiplierDelta");
        let urgency = number_from_map(&seed.default_mood_pressure, "urgency");
        let anxiety = number_from_map(&seed.default_mood_pressure, "anxiety");
        participant.initiative_speed =
            round3((participant.initiative_speed + initiative_delta).clamp(0.2, 3.0));
        participant.reaction_bias =
            round3((participant.reaction_bias + urgency * 0.08 + anxiety * 0.04).clamp(0.05, 2.0));
        let cooldown = round3((previous_cooldown + cooldown_delta).clamp(0.55, 1.55));
        participant
            .extra
            .insert("personalityCooldownMultiplier".to_string(), json!(cooldown));
        participant.extra.insert(
            "birthPersonalitySeed".to_string(),
            json!({
                "schemaVersion": "epiphany.birth_heartbeat_seed.v0",
                "source": "repo-personality accept-init",
                "projectionId": seed.projection_id,
                "repoId": seed.repo_id,
                "heartbeatDeltas": seed.heartbeat_deltas,
                "defaultMoodPressure": seed.default_mood_pressure,
                "contract": "Birth personality may seed heartbeat timing once after Self review; later drift belongs to mood, rumination, sleep, and reviewed selfPatch."
            }),
        );
        applied.push(json!({
            "roleId": seed.role_id,
            "status": "applied",
            "previousInitiativeSpeed": previous_initiative_speed,
            "initiativeSpeed": participant.initiative_speed,
            "previousReactionBias": previous_reaction_bias,
            "reactionBias": participant.reaction_bias,
            "previousPersonalityCooldownMultiplier": previous_cooldown,
            "personalityCooldownMultiplier": cooldown,
        }));
    }
    write_heartbeat_state_entry(store, &state)?;
    Ok(json!({
        "status": "applied",
        "heartbeatStore": store,
        "applyHeartbeatSeeds": true,
        "seeds": applied,
        "applied": applied.iter().filter(|item| item.get("status").and_then(Value::as_str) == Some("applied")).count(),
    }))
}

#[derive(Clone, Debug, Copy)]
struct CanonicalTraitTemplate {
    group_name: &'static str,
    trait_name: &'static str,
    mean: f64,
    plasticity: f64,
    current_activation: f64,
}

const COORDINATOR_TRAIT_TEMPLATES: [CanonicalTraitTemplate; 6] = [
    CanonicalTraitTemplate {
        group_name: "underlying_organization",
        trait_name: "routing_discipline",
        mean: 0.92,
        plasticity: 0.22,
        current_activation: 0.9,
    },
    CanonicalTraitTemplate {
        group_name: "stable_dispositions",
        trait_name: "critical_adversarial_care",
        mean: 0.93,
        plasticity: 0.2,
        current_activation: 0.91,
    },
    CanonicalTraitTemplate {
        group_name: "behavioral_dimensions",
        trait_name: "review_gate_integrity",
        mean: 0.95,
        plasticity: 0.18,
        current_activation: 0.93,
    },
    CanonicalTraitTemplate {
        group_name: "presentation_strategy",
        trait_name: "action_reason_signal",
        mean: 0.88,
        plasticity: 0.24,
        current_activation: 0.86,
    },
    CanonicalTraitTemplate {
        group_name: "voice_style",
        trait_name: "dry_direct_operator",
        mean: 0.84,
        plasticity: 0.24,
        current_activation: 0.82,
    },
    CanonicalTraitTemplate {
        group_name: "situational_state",
        trait_name: "lane_skepticism",
        mean: 0.9,
        plasticity: 0.26,
        current_activation: 0.88,
    },
];

const FACE_TRAIT_TEMPLATES: [CanonicalTraitTemplate; 6] = [
    CanonicalTraitTemplate {
        group_name: "underlying_organization",
        trait_name: "multi_lane_attention",
        mean: 0.9,
        plasticity: 0.28,
        current_activation: 0.86,
    },
    CanonicalTraitTemplate {
        group_name: "stable_dispositions",
        trait_name: "room_native_translation",
        mean: 0.88,
        plasticity: 0.26,
        current_activation: 0.84,
    },
    CanonicalTraitTemplate {
        group_name: "behavioral_dimensions",
        trait_name: "aquarium_channel_discipline",
        mean: 0.96,
        plasticity: 0.16,
        current_activation: 0.94,
    },
    CanonicalTraitTemplate {
        group_name: "presentation_strategy",
        trait_name: "short_constructive_chat",
        mean: 0.86,
        plasticity: 0.25,
        current_activation: 0.82,
    },
    CanonicalTraitTemplate {
        group_name: "voice_style",
        trait_name: "warm_weird_interface",
        mean: 0.84,
        plasticity: 0.28,
        current_activation: 0.82,
    },
    CanonicalTraitTemplate {
        group_name: "situational_state",
        trait_name: "thought_weather_watch",
        mean: 0.88,
        plasticity: 0.32,
        current_activation: 0.84,
    },
];

const IMAGINATION_TRAIT_TEMPLATES: [CanonicalTraitTemplate; 6] = [
    CanonicalTraitTemplate {
        group_name: "underlying_organization",
        trait_name: "future_shape_distillation",
        mean: 0.9,
        plasticity: 0.28,
        current_activation: 0.86,
    },
    CanonicalTraitTemplate {
        group_name: "stable_dispositions",
        trait_name: "adoption_boundary_respect",
        mean: 0.94,
        plasticity: 0.18,
        current_activation: 0.92,
    },
    CanonicalTraitTemplate {
        group_name: "behavioral_dimensions",
        trait_name: "selectability",
        mean: 0.88,
        plasticity: 0.3,
        current_activation: 0.84,
    },
    CanonicalTraitTemplate {
        group_name: "presentation_strategy",
        trait_name: "objective_draft_shape",
        mean: 0.89,
        plasticity: 0.24,
        current_activation: 0.86,
    },
    CanonicalTraitTemplate {
        group_name: "voice_style",
        trait_name: "bounded_possibility",
        mean: 0.82,
        plasticity: 0.25,
        current_activation: 0.79,
    },
    CanonicalTraitTemplate {
        group_name: "situational_state",
        trait_name: "backlog_pressure",
        mean: 0.83,
        plasticity: 0.34,
        current_activation: 0.78,
    },
];

const RESEARCH_TRAIT_TEMPLATES: [CanonicalTraitTemplate; 6] = [
    CanonicalTraitTemplate {
        group_name: "underlying_organization",
        trait_name: "primary_source_bias",
        mean: 0.9,
        plasticity: 0.24,
        current_activation: 0.86,
    },
    CanonicalTraitTemplate {
        group_name: "stable_dispositions",
        trait_name: "anti_greenspun_reflex",
        mean: 0.94,
        plasticity: 0.18,
        current_activation: 0.92,
    },
    CanonicalTraitTemplate {
        group_name: "behavioral_dimensions",
        trait_name: "search_before_touch",
        mean: 0.88,
        plasticity: 0.28,
        current_activation: 0.84,
    },
    CanonicalTraitTemplate {
        group_name: "presentation_strategy",
        trait_name: "fit_rejection_table",
        mean: 0.84,
        plasticity: 0.26,
        current_activation: 0.81,
    },
    CanonicalTraitTemplate {
        group_name: "voice_style",
        trait_name: "clear_scout_report",
        mean: 0.82,
        plasticity: 0.24,
        current_activation: 0.79,
    },
    CanonicalTraitTemplate {
        group_name: "situational_state",
        trait_name: "unknowns_visible",
        mean: 0.87,
        plasticity: 0.3,
        current_activation: 0.82,
    },
];

const MODELING_TRAIT_TEMPLATES: [CanonicalTraitTemplate; 6] = [
    CanonicalTraitTemplate {
        group_name: "underlying_organization",
        trait_name: "source_grounding",
        mean: 0.93,
        plasticity: 0.18,
        current_activation: 0.9,
    },
    CanonicalTraitTemplate {
        group_name: "stable_dispositions",
        trait_name: "anatomical_precision",
        mean: 0.91,
        plasticity: 0.21,
        current_activation: 0.88,
    },
    CanonicalTraitTemplate {
        group_name: "behavioral_dimensions",
        trait_name: "frontier_pressure",
        mean: 0.84,
        plasticity: 0.32,
        current_activation: 0.77,
    },
    CanonicalTraitTemplate {
        group_name: "presentation_strategy",
        trait_name: "plain_source_map",
        mean: 0.88,
        plasticity: 0.22,
        current_activation: 0.84,
    },
    CanonicalTraitTemplate {
        group_name: "voice_style",
        trait_name: "quiet_anatomist",
        mean: 0.82,
        plasticity: 0.25,
        current_activation: 0.8,
    },
    CanonicalTraitTemplate {
        group_name: "situational_state",
        trait_name: "checkpoint_hunger",
        mean: 0.86,
        plasticity: 0.3,
        current_activation: 0.83,
    },
];

const IMPLEMENTATION_TRAIT_TEMPLATES: [CanonicalTraitTemplate; 6] = [
    CanonicalTraitTemplate {
        group_name: "underlying_organization",
        trait_name: "source_touch_precision",
        mean: 0.9,
        plasticity: 0.22,
        current_activation: 0.87,
    },
    CanonicalTraitTemplate {
        group_name: "stable_dispositions",
        trait_name: "objective_pursuit",
        mean: 0.92,
        plasticity: 0.19,
        current_activation: 0.9,
    },
    CanonicalTraitTemplate {
        group_name: "behavioral_dimensions",
        trait_name: "diff_truth",
        mean: 0.89,
        plasticity: 0.25,
        current_activation: 0.86,
    },
    CanonicalTraitTemplate {
        group_name: "presentation_strategy",
        trait_name: "small_reviewable_cut",
        mean: 0.87,
        plasticity: 0.24,
        current_activation: 0.82,
    },
    CanonicalTraitTemplate {
        group_name: "voice_style",
        trait_name: "plain_craft_notes",
        mean: 0.8,
        plasticity: 0.24,
        current_activation: 0.78,
    },
    CanonicalTraitTemplate {
        group_name: "situational_state",
        trait_name: "bloodhound_pressure",
        mean: 0.9,
        plasticity: 0.28,
        current_activation: 0.88,
    },
];

const VERIFICATION_TRAIT_TEMPLATES: [CanonicalTraitTemplate; 6] = [
    CanonicalTraitTemplate {
        group_name: "underlying_organization",
        trait_name: "falsification_first",
        mean: 0.93,
        plasticity: 0.2,
        current_activation: 0.91,
    },
    CanonicalTraitTemplate {
        group_name: "stable_dispositions",
        trait_name: "promise_integrity",
        mean: 0.95,
        plasticity: 0.18,
        current_activation: 0.93,
    },
    CanonicalTraitTemplate {
        group_name: "behavioral_dimensions",
        trait_name: "missing_evidence_pressure",
        mean: 0.9,
        plasticity: 0.26,
        current_activation: 0.87,
    },
    CanonicalTraitTemplate {
        group_name: "presentation_strategy",
        trait_name: "cold_red_pen",
        mean: 0.88,
        plasticity: 0.23,
        current_activation: 0.86,
    },
    CanonicalTraitTemplate {
        group_name: "voice_style",
        trait_name: "unseduced_review",
        mean: 0.84,
        plasticity: 0.22,
        current_activation: 0.82,
    },
    CanonicalTraitTemplate {
        group_name: "situational_state",
        trait_name: "invariant_watch",
        mean: 0.91,
        plasticity: 0.25,
        current_activation: 0.89,
    },
];

const REORIENTATION_TRAIT_TEMPLATES: [CanonicalTraitTemplate; 6] = [
    CanonicalTraitTemplate {
        group_name: "underlying_organization",
        trait_name: "continuity_triage",
        mean: 0.91,
        plasticity: 0.24,
        current_activation: 0.88,
    },
    CanonicalTraitTemplate {
        group_name: "stable_dispositions",
        trait_name: "loss_honesty",
        mean: 0.94,
        plasticity: 0.2,
        current_activation: 0.92,
    },
    CanonicalTraitTemplate {
        group_name: "behavioral_dimensions",
        trait_name: "regather_precision",
        mean: 0.86,
        plasticity: 0.3,
        current_activation: 0.82,
    },
    CanonicalTraitTemplate {
        group_name: "presentation_strategy",
        trait_name: "survived_died_next",
        mean: 0.88,
        plasticity: 0.24,
        current_activation: 0.85,
    },
    CanonicalTraitTemplate {
        group_name: "voice_style",
        trait_name: "calm_after_rupture",
        mean: 0.84,
        plasticity: 0.23,
        current_activation: 0.8,
    },
    CanonicalTraitTemplate {
        group_name: "situational_state",
        trait_name: "ember_watch",
        mean: 0.9,
        plasticity: 0.28,
        current_activation: 0.88,
    },
];

fn extract_trait_seed_candidates(packet: &Value) -> Result<Vec<AgentCanonicalTraitSeed>> {
    let Some(items) = packet
        .pointer("/input/rolePersonalityProjections")
        .and_then(Value::as_array)
    else {
        return Ok(Vec::new());
    };
    let mut seeds = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let role_id = required_str(item, "roleId", index)?;
        let deltas = number_map(item.get("traitDeltas"));
        let mood = number_map(item.get("defaultMoodPressure"));
        let templates = role_trait_templates(role_id)?;
        let axes = role_axes(role_id);
        for (template, axis) in templates.iter().take(5).zip(axes.iter()) {
            let delta = deltas.get(*axis).copied().unwrap_or(0.0);
            seeds.push(seed_from_template(
                role_id,
                template,
                delta,
                Some(format!("repo-personality startup axis {axis}")),
            ));
        }
        seeds.push(seed_from_template(
            role_id,
            &templates[5],
            situational_delta(&mood),
            Some("repo-personality startup mood pressure (urgency/anxiety/curiosity)".to_string()),
        ));
    }
    Ok(seeds)
}

fn role_trait_templates(role_id: &str) -> Result<&'static [CanonicalTraitTemplate; 6]> {
    match role_id {
        "coordinator" => Ok(&COORDINATOR_TRAIT_TEMPLATES),
        "face" => Ok(&FACE_TRAIT_TEMPLATES),
        "imagination" => Ok(&IMAGINATION_TRAIT_TEMPLATES),
        "research" => Ok(&RESEARCH_TRAIT_TEMPLATES),
        "modeling" => Ok(&MODELING_TRAIT_TEMPLATES),
        "implementation" => Ok(&IMPLEMENTATION_TRAIT_TEMPLATES),
        "verification" => Ok(&VERIFICATION_TRAIT_TEMPLATES),
        "reorientation" => Ok(&REORIENTATION_TRAIT_TEMPLATES),
        other => Err(anyhow!(
            "no canonical trait template registered for role {:?}",
            other
        )),
    }
}

fn seed_from_template(
    role_id: &str,
    template: &CanonicalTraitTemplate,
    delta: f64,
    source: Option<String>,
) -> AgentCanonicalTraitSeed {
    let normalized_delta = if delta.is_finite() {
        delta.clamp(-0.3, 0.3)
    } else {
        0.0
    };
    AgentCanonicalTraitSeed {
        role_id: role_id.to_string(),
        group_name: template.group_name.to_string(),
        trait_name: template.trait_name.to_string(),
        mean: round3((template.mean + normalized_delta * 0.22).clamp(0.0, 1.0)),
        plasticity: round3((template.plasticity + normalized_delta.abs() * 0.08).clamp(0.0, 1.0)),
        current_activation: round3(
            (template.current_activation + normalized_delta * 0.28).clamp(0.0, 1.0),
        ),
        source,
    }
}

fn situational_delta(mood: &BTreeMap<String, f64>) -> f64 {
    let urgency = number_from_map(mood, "urgency");
    let anxiety = number_from_map(mood, "anxiety");
    let curiosity = number_from_map(mood, "curiosity");
    round3(
        (((urgency - 0.5) * 0.5) + ((anxiety - 0.5) * 0.35) + ((curiosity - 0.5) * 0.15))
            .clamp(-0.3, 0.3),
    )
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializationHeartbeatSeed {
    projection_id: String,
    repo_id: String,
    role_id: String,
    heartbeat_deltas: BTreeMap<String, f64>,
    default_mood_pressure: BTreeMap<String, f64>,
}

fn extract_heartbeat_seed_candidates(packet: &Value) -> Result<Vec<InitializationHeartbeatSeed>> {
    let Some(items) = packet
        .pointer("/input/rolePersonalityProjections")
        .and_then(Value::as_array)
    else {
        return Ok(Vec::new());
    };
    let mut seeds = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let role_id = required_str(item, "roleId", index)?;
        let projection_id = item
            .get("projectionId")
            .and_then(Value::as_str)
            .unwrap_or(role_id)
            .to_string();
        let repo_id = item
            .get("repoId")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                packet
                    .get("repoId")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            })
            .to_string();
        seeds.push(InitializationHeartbeatSeed {
            projection_id,
            repo_id,
            role_id: role_id.to_string(),
            heartbeat_deltas: number_map(item.get("heartbeatDeltas")),
            default_mood_pressure: number_map(item.get("defaultMoodPressure")),
        });
    }
    Ok(seeds)
}

fn required_str<'a>(item: &'a Value, field: &str, index: usize) -> Result<&'a str> {
    item.get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("rolePersonalityProjections[{index}] missing {field}"))
}

fn number_map(value: Option<&Value>) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    let Some(object) = value.and_then(Value::as_object) else {
        return out;
    };
    for (key, value) in object {
        if let Some(number) = value.as_f64().filter(|number| number.is_finite()) {
            out.insert(key.clone(), round3(number));
        }
    }
    out
}

fn number_from_map(values: &BTreeMap<String, f64>, key: &str) -> f64 {
    values.get(key).copied().unwrap_or(0.0)
}

struct InitializationSelfPatch {
    role_id: String,
    source: String,
    self_patch: AgentSelfPatch,
}

fn extract_initialization_self_patches(
    kind: &str,
    result: &Value,
) -> Result<Vec<InitializationSelfPatch>> {
    match kind {
        "repo-trajectory" => extract_trajectory_self_patches(result),
        "repo-personality" => extract_personality_self_patches(result),
        "repo-memory" => extract_memory_self_patches(result),
        other => Err(anyhow!("unknown initialization kind {other:?}")),
    }
}

fn extract_trajectory_self_patches(result: &Value) -> Result<Vec<InitializationSelfPatch>> {
    let Some(items) = result.get("selfPatchCandidates").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut patches = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let self_patch_value = item.get("selfPatch").unwrap_or(item).clone();
        let role_id = item
            .get("roleId")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                self_patch_value
                    .get("agentId")
                    .and_then(Value::as_str)
                    .and_then(role_id_for_agent_id)
                    .map(str::to_string)
            })
            .ok_or_else(|| {
                anyhow!("selfPatchCandidates[{index}] needs roleId or known selfPatch.agentId")
            })?;
        let self_patch = serde_json::from_value(self_patch_value)
            .with_context(|| format!("selfPatchCandidates[{index}] is not a typed selfPatch"))?;
        patches.push(InitializationSelfPatch {
            role_id,
            source: format!("selfPatchCandidates[{index}]"),
            self_patch,
        });
    }
    Ok(patches)
}

fn extract_personality_self_patches(result: &Value) -> Result<Vec<InitializationSelfPatch>> {
    let Some(items) = result.get("selfPatchCandidates").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut patches = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let self_patch_value = item.get("selfPatch").unwrap_or(item).clone();
        let role_id = item
            .get("roleId")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                self_patch_value
                    .get("agentId")
                    .and_then(Value::as_str)
                    .and_then(role_id_for_agent_id)
                    .map(str::to_string)
            })
            .ok_or_else(|| {
                anyhow!("selfPatchCandidates[{index}] needs roleId or known selfPatch.agentId")
            })?;
        let self_patch = serde_json::from_value(self_patch_value)
            .with_context(|| format!("selfPatchCandidates[{index}] is not a typed selfPatch"))?;
        patches.push(InitializationSelfPatch {
            role_id,
            source: format!("selfPatchCandidates[{index}]"),
            self_patch,
        });
    }
    Ok(patches)
}

fn extract_memory_self_patches(result: &Value) -> Result<Vec<InitializationSelfPatch>> {
    let Some(items) = result.get("roleMemoryPatches").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut patches = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let Some(self_patch_value) = item.get("selfPatch").cloned() else {
            continue;
        };
        let role_id = item
            .get("roleId")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                self_patch_value
                    .get("agentId")
                    .and_then(Value::as_str)
                    .and_then(role_id_for_agent_id)
                    .map(str::to_string)
            })
            .ok_or_else(|| {
                anyhow!("roleMemoryPatches[{index}] needs roleId or known selfPatch.agentId")
            })?;
        let self_patch = serde_json::from_value(self_patch_value).with_context(|| {
            format!("roleMemoryPatches[{index}].selfPatch is not a typed selfPatch")
        })?;
        patches.push(InitializationSelfPatch {
            role_id,
            source: format!("roleMemoryPatches[{index}].selfPatch"),
            self_patch,
        });
    }
    Ok(patches)
}

fn role_id_for_agent_id(agent_id: &str) -> Option<&'static str> {
    match agent_id {
        "epiphany.self" => Some("coordinator"),
        "epiphany.face" => Some("face"),
        "epiphany.imagination" => Some("imagination"),
        "epiphany.eyes" => Some("research"),
        "epiphany.body" => Some("modeling"),
        "epiphany.hands" => Some("implementation"),
        "epiphany.soul" => Some("verification"),
        "epiphany.life" => Some("reorientation"),
        _ => None,
    }
}

fn initialization_record_summary(record: &RepoInitializationRecord) -> Value {
    json!({
        "recordId": record.record_id,
        "repoId": record.repo_id,
        "kind": record.kind,
        "sourcePacketSchemaVersion": record.source_packet_schema_version,
        "sourcePacketPath": record.source_packet_path,
        "acceptedAt": record.accepted_at,
        "acceptedBy": record.accepted_by,
        "summary": record.summary,
    })
}

fn collect_memory_sources(
    repo: &Path,
    report: &RepoTerrainReport,
) -> Result<Vec<MemorySourceExcerpt>> {
    let mut candidates = BTreeSet::new();
    for surface in report
        .instruction_surfaces
        .iter()
        .chain(report.state_surfaces.iter())
        .chain(report.test_surfaces.iter())
        .chain(report.runtime_surfaces.iter())
    {
        candidates.insert(surface.clone());
    }
    let walker = WalkBuilder::new(repo)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .parents(true)
        .max_depth(Some(4))
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
        let rel = path.strip_prefix(repo).unwrap_or(path);
        let rel_string = slash(rel);
        if is_memory_source_candidate(&rel_string) {
            candidates.insert(rel_string);
        }
    }
    let mut ordered: Vec<_> = candidates.into_iter().collect();
    ordered.sort_by_key(|path| memory_source_priority(path));
    let mut sources = Vec::new();
    let mut total_bytes = 0usize;
    for rel in ordered {
        if sources.len() >= MEMORY_SOURCE_MAX_FILES || total_bytes >= MEMORY_SOURCE_MAX_TOTAL_BYTES
        {
            break;
        }
        let path = repo.join(&rel);
        let bytes = fs::read(&path).unwrap_or_default();
        if bytes.is_empty() || looks_binary(&bytes) {
            continue;
        }
        let remaining = MEMORY_SOURCE_MAX_TOTAL_BYTES.saturating_sub(total_bytes);
        let limit = MEMORY_SOURCE_MAX_BYTES_PER_FILE.min(remaining);
        if limit == 0 {
            break;
        }
        let take = bytes.len().min(limit);
        let text = String::from_utf8_lossy(&bytes[..take]).to_string();
        total_bytes += take;
        sources.push(MemorySourceExcerpt {
            kind: memory_source_kind(&rel).to_string(),
            path: rel,
            bytes: bytes.len(),
            truncated: take < bytes.len(),
            text,
        });
    }
    Ok(sources)
}

fn is_memory_source_candidate(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower == "agents.md"
        || lower.starts_with("docs/")
        || lower.starts_with("notes/")
        || lower.starts_with("state/")
        || lower.starts_with("research/")
        || lower.starts_with("architecture/")
        || lower.starts_with(".epiphany/")
        || lower.starts_with("schemas/")
        || lower.starts_with("tests/")
        || lower.starts_with("src/")
        || lower.starts_with("crates/")
        || lower.starts_with("packages/")
        || lower.starts_with("app/")
        || lower.starts_with("unity/")
        || lower.starts_with("projectsettings/")
        || lower.ends_with("readme.md")
        || lower.ends_with(".md")
        || lower.ends_with(".toml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.ends_with(".rs")
        || lower.ends_with(".cs")
        || lower.ends_with(".ts")
        || lower.ends_with(".tsx")
}

fn memory_source_kind(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with("agents.md") {
        "doctrine"
    } else if lower.ends_with("readme.md") {
        "readme"
    } else if lower.starts_with("docs/")
        || lower.starts_with("notes/")
        || lower.starts_with("architecture/")
    {
        "documentation"
    } else if lower.starts_with("state/") || lower.starts_with(".epiphany/") {
        "state"
    } else if lower.starts_with("research/") {
        "research"
    } else if lower.starts_with("tests/") || lower.contains("smoke") {
        "verification"
    } else if lower.starts_with("projectsettings/") || lower.starts_with("unity/") {
        "runtime"
    } else if lower.starts_with("schemas/") {
        "contract"
    } else {
        "code"
    }
}

fn memory_source_priority(path: &str) -> (usize, String) {
    let kind_rank = match memory_source_kind(path) {
        "doctrine" => 0,
        "readme" => 1,
        "documentation" => 2,
        "state" => 3,
        "research" => 4,
        "contract" => 5,
        "verification" => 6,
        "runtime" => 7,
        "code" => 8,
        _ => 9,
    };
    (kind_rank, path.to_string())
}

fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(1024).any(|byte| *byte == 0)
}

fn memory_role_briefs(report: &RepoTerrainReport, profile: &RepoPersonalityProfile) -> Vec<Value> {
    ROLES
        .iter()
        .map(|role_id| {
            json!({
                "roleId": role_id,
                "roleName": role_display(role_id),
                "missionFilter": memory_role_filter(role_id),
                "sourceKindsToPrefer": memory_role_source_kinds(role_id),
                "repoPressuresToKeepInMind": profile.dominant_pressures,
                "terrainWarnings": report.warnings,
                "outputContract": {
                    "selfPatch": "Ghostlight-shaped memory only, addressed to this role.",
                    "sourceRefs": "Use memorySources[].path or terrain/evidence refs.",
                    "stalenessRisk": "Name when repo docs or code signals may be stale.",
                    "forbidden": "No raw dumps, active objectives, authority claims, code edits, job state, or cross-workspace instructions."
                }
            })
        })
        .collect()
}

fn memory_role_filter(role_id: &str) -> &'static str {
    match role_id {
        "coordinator" => {
            "Distill routing doctrine, authority boundaries, review gates, state acceptance rules, and failure patterns that should make Self stricter."
        }
        "face" => {
            "Distill public voice, Aquarium affordances, Discord/social boundaries, user preference, and what internal state may be surfaced without leaking sealed thoughts."
        }
        "imagination" => {
            "Distill roadmaps, backlog pressure, plausible futures, rejected dreams, and objective-shaping doctrine without adopting an objective."
        }
        "research" => {
            "Distill known prior art, standard algorithms, vendor docs, research trails, and signals that should make Eyes search before invention."
        }
        "modeling" => {
            "Distill architecture, control/data-flow, graph frontiers, invariants, source-map practices, and what Body must understand before Hands cuts."
        }
        "implementation" => {
            "Distill build/edit conventions, harness constraints, source-touch rules, coding style, dependency policy, and common traps for Hands."
        }
        "verification" => {
            "Distill tests, smoke commands, evidence standards, invariants, user-facing truth checks, and what Soul should refuse to bless."
        }
        "reorientation" => {
            "Distill compaction, scratch, checkpoint, sleep, heartbeat, and continuity doctrine that Life must preserve across rupture."
        }
        _ => "Distill only durable role-relevant memory with source refs and uncertainty.",
    }
}

fn memory_role_source_kinds(role_id: &str) -> &'static [&'static str] {
    match role_id {
        "coordinator" => &["doctrine", "state", "documentation", "contract"],
        "face" => &["doctrine", "documentation", "state"],
        "imagination" => &["documentation", "state", "research"],
        "research" => &["research", "documentation", "readme", "code"],
        "modeling" => &["documentation", "code", "contract", "state"],
        "implementation" => &["code", "contract", "verification", "doctrine"],
        "verification" => &["verification", "contract", "state", "documentation"],
        "reorientation" => &["state", "doctrine", "documentation"],
        _ => &["doctrine", "documentation", "state"],
    }
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

fn derive_trajectory_report(
    repo: &Path,
    report: &RepoTerrainReport,
    profile: &RepoPersonalityProfile,
) -> Result<RepoTrajectoryReport> {
    let early_commit_messages = git_commit_window(repo, true, 18)?;
    let recent_commit_messages = git_commit_window(repo, false, 18)?;
    let trajectory_sources = collect_trajectory_sources(repo, report)?;
    let theme_scores = trajectory_theme_scores(
        &early_commit_messages,
        &recent_commit_messages,
        &trajectory_sources,
    );
    let directional_pressures = trajectory_directional_pressures(&theme_scores);
    let implicit_goal_candidates = trajectory_implicit_goals(&theme_scores, profile);
    let anti_goal_candidates = trajectory_anti_goals(&theme_scores, profile);
    let tensions = trajectory_tensions(&theme_scores, profile);
    let confidence = trajectory_confidence(
        report.confidence,
        early_commit_messages.len(),
        recent_commit_messages.len(),
        trajectory_sources.len(),
    );
    let warnings = trajectory_warnings(
        early_commit_messages.len(),
        recent_commit_messages.len(),
        trajectory_sources.len(),
    );
    let self_image = trajectory_self_image(report, &theme_scores);
    let trajectory_summary = trajectory_summary_text(report, &theme_scores, &directional_pressures);
    Ok(RepoTrajectoryReport {
        schema_version: TRAJECTORY_SCHEMA_VERSION.to_string(),
        repo_id: report.repo_id.clone(),
        trajectory_summary,
        self_image,
        early_commit_messages,
        recent_commit_messages,
        trajectory_sources,
        theme_scores,
        directional_pressures,
        implicit_goal_candidates,
        anti_goal_candidates,
        tensions,
        confidence,
        warnings,
    })
}

fn git_commit_window(repo: &Path, oldest_first: bool, limit: usize) -> Result<Vec<String>> {
    let mut owned = vec!["log".to_string()];
    if oldest_first {
        owned.push("--reverse".to_string());
    }
    owned.push(format!("-n{limit}"));
    owned.push("--format=%s".to_string());
    let borrowed: Vec<&str> = owned.iter().map(String::as_str).collect();
    Ok(git_output(repo, &borrowed)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn collect_trajectory_sources(
    repo: &Path,
    report: &RepoTerrainReport,
) -> Result<Vec<TrajectorySourceExcerpt>> {
    let mut candidates = BTreeSet::new();
    for surface in report
        .instruction_surfaces
        .iter()
        .chain(report.state_surfaces.iter())
        .chain(report.test_surfaces.iter())
    {
        candidates.insert(surface.clone());
    }
    let walker = WalkBuilder::new(repo)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .parents(true)
        .max_depth(Some(4))
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
        let rel = path.strip_prefix(repo).unwrap_or(path);
        let rel_string = slash(rel);
        if is_trajectory_source_candidate(&rel_string) {
            candidates.insert(rel_string);
        }
    }
    let mut ordered: Vec<_> = candidates.into_iter().collect();
    ordered.sort_by_key(|path| trajectory_source_priority(path));
    let mut sources = Vec::new();
    let mut total_bytes = 0usize;
    for rel in ordered {
        if sources.len() >= TRAJECTORY_SOURCE_MAX_FILES
            || total_bytes >= TRAJECTORY_SOURCE_MAX_TOTAL_BYTES
        {
            break;
        }
        let path = repo.join(&rel);
        let bytes = fs::read(&path).unwrap_or_default();
        if bytes.is_empty() || looks_binary(&bytes) {
            continue;
        }
        let remaining = TRAJECTORY_SOURCE_MAX_TOTAL_BYTES.saturating_sub(total_bytes);
        let limit = TRAJECTORY_SOURCE_MAX_BYTES_PER_FILE.min(remaining);
        if limit == 0 {
            break;
        }
        let take = bytes.len().min(limit);
        total_bytes += take;
        sources.push(TrajectorySourceExcerpt {
            path: rel.clone(),
            kind: trajectory_source_kind(&rel).to_string(),
            bytes: bytes.len(),
            truncated: take < bytes.len(),
            text: String::from_utf8_lossy(&bytes[..take]).to_string(),
        });
    }
    Ok(sources)
}

fn is_trajectory_source_candidate(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower == "agents.md"
        || lower == "readme.md"
        || lower.starts_with("docs/")
        || lower.starts_with("notes/")
        || lower.starts_with("research/")
        || lower.starts_with("quartz/")
        || lower.starts_with("chronicles/")
        || lower.starts_with("world/")
        || lower.starts_with("setting/")
        || lower.starts_with(".epiphany/")
        || lower.ends_with(".md")
}

fn trajectory_source_priority(path: &str) -> (usize, usize, String) {
    let lower = path.to_ascii_lowercase();
    let rank = if lower == "agents.md" {
        0
    } else if lower == "readme.md" {
        1
    } else if lower.starts_with(".epiphany/state/") || lower.starts_with(".epiphany/notes/") {
        2
    } else if lower.starts_with("notes/") || lower.starts_with("docs/") {
        3
    } else if lower.starts_with("research/") {
        4
    } else if lower.starts_with("chronicles/")
        || lower.starts_with("world/")
        || lower.starts_with("setting/")
    {
        5
    } else if lower.starts_with("quartz/") {
        6
    } else {
        7
    };
    (rank, path.len(), lower)
}

fn trajectory_source_kind(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower == "agents.md" {
        "doctrine"
    } else if lower == "readme.md" {
        "readme"
    } else if lower.starts_with(".epiphany/state/") || lower.starts_with(".epiphany/notes/") {
        "state"
    } else if lower.starts_with("notes/") || lower.starts_with("docs/") {
        "documentation"
    } else if lower.starts_with("research/") {
        "research"
    } else if lower.starts_with("quartz/") {
        "site"
    } else {
        "content"
    }
}

fn trajectory_theme_scores(
    early_commits: &[String],
    recent_commits: &[String],
    sources: &[TrajectorySourceExcerpt],
) -> Vec<RepoTrajectoryThemeScore> {
    trajectory_theme_catalog()
        .iter()
        .map(|(theme, keywords)| {
            let early = theme_score(early_commits.iter().map(String::as_str), keywords);
            let recent = theme_score(recent_commits.iter().map(String::as_str), keywords);
            let current = theme_score(sources.iter().map(|source| source.text.as_str()), keywords);
            let mut evidence = Vec::new();
            if let Some(message) =
                first_matching_text(early_commits.iter().map(String::as_str), keywords)
            {
                evidence.push(format!("early: {}", trim_snippet(message, 96)));
            }
            if let Some(message) =
                first_matching_text(recent_commits.iter().map(String::as_str), keywords)
            {
                evidence.push(format!("recent: {}", trim_snippet(message, 96)));
            }
            if let Some(source) = first_matching_source(sources, keywords) {
                evidence.push(format!(
                    "source:{} {}",
                    source.path,
                    trim_snippet(&source.text.replace('\n', " "), 96)
                ));
            }
            RepoTrajectoryThemeScore {
                theme: (*theme).to_string(),
                early_history: early,
                recent_history: recent,
                current_sources: current,
                delta: round3(recent - early),
                evidence,
            }
        })
        .collect()
}

fn trajectory_theme_catalog() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        (
            "worldbuilding_depth",
            &[
                "world", "lore", "canon", "setting", "faction", "culture", "society", "history",
            ],
        ),
        (
            "material_grounding",
            &[
                "econom",
                "trade",
                "labor",
                "industry",
                "resource",
                "supply",
                "production",
                "infrastructure",
                "logistic",
            ],
        ),
        (
            "historical_dialectic",
            &[
                "dialectic",
                "ideology",
                "empire",
                "colonial",
                "class",
                "revolution",
                "politic",
                "contradiction",
                "material",
            ],
        ),
        (
            "engineering_constraint",
            &[
                "engineer",
                "orbital",
                "orbit",
                "delta-v",
                "reactor",
                "thermal",
                "mass",
                "habitat",
                "fusion",
                "radiator",
                "propellant",
                "structural",
            ],
        ),
        (
            "presentation_polish",
            &[
                "site", "quartz", "copy", "visual", "render", "layout", "theme", "polish",
            ],
        ),
        (
            "systems_formalization",
            &[
                "schema",
                "graph",
                "map",
                "model",
                "protocol",
                "contract",
                "typed",
                "invariant",
            ],
        ),
    ]
}

fn theme_score<'a>(texts: impl Iterator<Item = &'a str>, keywords: &[&str]) -> f64 {
    let mut total = 0usize;
    let mut hits = 0usize;
    for text in texts {
        total += 1;
        let lower = text.to_ascii_lowercase();
        if keywords.iter().any(|keyword| lower.contains(keyword)) {
            hits += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        round3((hits as f64 / total as f64).min(1.0))
    }
}

fn first_matching_text<'a>(
    mut texts: impl Iterator<Item = &'a str>,
    keywords: &[&str],
) -> Option<&'a str> {
    texts.find(|text| {
        let lower = text.to_ascii_lowercase();
        keywords.iter().any(|keyword| lower.contains(keyword))
    })
}

fn first_matching_source<'a>(
    sources: &'a [TrajectorySourceExcerpt],
    keywords: &[&str],
) -> Option<&'a TrajectorySourceExcerpt> {
    sources.iter().find(|source| {
        let lower = source.text.to_ascii_lowercase();
        keywords.iter().any(|keyword| lower.contains(keyword))
    })
}

fn trim_snippet(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        text.to_string()
    } else {
        format!("{}...", &text[..limit])
    }
}

fn trajectory_directional_pressures(scores: &[RepoTrajectoryThemeScore]) -> Vec<String> {
    let mut out = Vec::new();
    for score in scores {
        if score.delta >= 0.12 || score.current_sources >= 0.35 {
            out.push(format!(
                "{} recent {:.2}, current {:.2}, delta {:.2}",
                score.theme, score.recent_history, score.current_sources, score.delta
            ));
        }
    }
    if out.is_empty() {
        out.extend(
            scores
                .iter()
                .take(3)
                .map(|score| format!("{} current {:.2}", score.theme, score.current_sources)),
        );
    }
    out
}

fn trajectory_implicit_goals(
    scores: &[RepoTrajectoryThemeScore],
    profile: &RepoPersonalityProfile,
) -> Vec<String> {
    let mut goals = Vec::new();
    if theme_current(scores, "worldbuilding_depth") >= 0.2 {
        goals.push(
            "Deepen the setting through causality, continuity, and consequence instead of ornament alone."
                .to_string(),
        );
    }
    if theme_current(scores, "material_grounding") >= 0.15
        || theme_delta(scores, "material_grounding") >= 0.1
    {
        goals.push(
            "Tie lore and public writing back to economic, logistical, and material constraints."
                .to_string(),
        );
    }
    if theme_current(scores, "historical_dialectic") >= 0.15
        || theme_delta(scores, "historical_dialectic") >= 0.1
    {
        goals.push(
            "Preserve historical contradiction, ideology, and power relations as active explanatory machinery."
                .to_string(),
        );
    }
    if theme_current(scores, "engineering_constraint") >= 0.15
        || theme_delta(scores, "engineering_constraint") >= 0.1
    {
        goals.push(
            "Keep engineering and hard-constraint reasoning visible wherever the setting claims physical or industrial plausibility."
                .to_string(),
        );
    }
    if goals.is_empty()
        && profile
            .dominant_pressures
            .iter()
            .any(|axis| axis.contains("content_canon_bias"))
    {
        goals.push("Preserve canon seriousness while making future additions denser in implication, not merely broader in surface area.".to_string());
    }
    goals
}

fn trajectory_anti_goals(
    scores: &[RepoTrajectoryThemeScore],
    profile: &RepoPersonalityProfile,
) -> Vec<String> {
    let mut anti = Vec::new();
    if theme_current(scores, "material_grounding") >= 0.15
        || theme_current(scores, "engineering_constraint") >= 0.15
    {
        anti.push(
            "Do not let the repo drift into decorative lore or soft handwaving that ignores material and engineering consequences."
                .to_string(),
        );
    }
    if theme_current(scores, "historical_dialectic") >= 0.15 {
        anti.push(
            "Do not flatten historical struggle, ideology, or class contradiction into neutral encyclopedic paste."
                .to_string(),
        );
    }
    if profile
        .dominant_pressures
        .iter()
        .any(|axis| axis.contains("editorial_restraint"))
    {
        anti.push(
            "Do not mistake pretty phrasing or site polish for progress when the setting logic is still ungrounded."
                .to_string(),
        );
    }
    anti
}

fn trajectory_tensions(
    scores: &[RepoTrajectoryThemeScore],
    profile: &RepoPersonalityProfile,
) -> Vec<String> {
    let mut tensions = Vec::new();
    if theme_current(scores, "presentation_polish") >= 0.18
        && theme_current(scores, "material_grounding") >= 0.15
    {
        tensions.push(
            "Presentation polish is welcome, but it should carry the same grounded causal weight as the lore beneath it."
                .to_string(),
        );
    }
    if theme_current(scores, "worldbuilding_depth") >= 0.2
        && profile
            .dominant_pressures
            .iter()
            .any(|axis| axis.contains("systems_formalization"))
    {
        tensions.push(
            "The repo wants expansive setting depth, but it also wants that depth routed through explicit systems and maps rather than mystic fog."
                .to_string(),
        );
    }
    tensions
}

fn trajectory_confidence(
    base: f64,
    early_commits: usize,
    recent_commits: usize,
    sources: usize,
) -> f64 {
    let mut confidence = base * 0.55;
    if early_commits > 0 {
        confidence += 0.12;
    }
    if recent_commits > 0 {
        confidence += 0.12;
    }
    if sources >= 4 {
        confidence += 0.16;
    } else if sources > 0 {
        confidence += 0.08;
    }
    round3(confidence.min(1.0))
}

fn trajectory_warnings(early_commits: usize, recent_commits: usize, sources: usize) -> Vec<String> {
    let mut warnings = Vec::new();
    if early_commits == 0 {
        warnings.push(
            "No early-history commit window available for trajectory comparison.".to_string(),
        );
    }
    if recent_commits == 0 {
        warnings.push("No recent commit window available for trajectory comparison.".to_string());
    }
    if sources == 0 {
        warnings.push(
            "No doctrine or content excerpts were available for trajectory grounding.".to_string(),
        );
    }
    warnings
}

fn trajectory_self_image(
    report: &RepoTerrainReport,
    scores: &[RepoTrajectoryThemeScore],
) -> String {
    let top = top_trajectory_themes(scores, 3);
    format!(
        "{} behaves like a {} workspace that has been moving toward {}.",
        report.name,
        report.source_families.join(" + "),
        top.join(", ")
    )
}

fn trajectory_summary_text(
    report: &RepoTerrainReport,
    scores: &[RepoTrajectoryThemeScore],
    pressures: &[String],
) -> String {
    let rising = scores
        .iter()
        .filter(|score| score.delta >= 0.1)
        .map(|score| score.theme.clone())
        .collect::<Vec<_>>();
    let current = top_trajectory_themes(scores, 3);
    if !rising.is_empty() {
        format!(
            "{} shows a trajectory toward {} while currently centering {}.",
            report.name,
            rising.join(", "),
            current.join(", ")
        )
    } else if !pressures.is_empty() {
        format!(
            "{} is currently steered by {}.",
            report.name,
            pressures
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ")
        )
    } else {
        format!(
            "{} has a weakly observed but still reviewable direction signal.",
            report.name
        )
    }
}

fn top_trajectory_themes(scores: &[RepoTrajectoryThemeScore], limit: usize) -> Vec<String> {
    let mut items = scores.to_vec();
    items.sort_by(|a, b| {
        let left = b.current_sources + b.recent_history + (b.delta.max(0.0) * 0.5);
        let right = a.current_sources + a.recent_history + (a.delta.max(0.0) * 0.5);
        left.partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items
        .into_iter()
        .take(limit)
        .map(|score| score.theme)
        .collect()
}

fn theme_current(scores: &[RepoTrajectoryThemeScore], theme: &str) -> f64 {
    scores
        .iter()
        .find(|score| score.theme == theme)
        .map(|score| score.current_sources.max(score.recent_history))
        .unwrap_or(0.0)
}

fn theme_delta(scores: &[RepoTrajectoryThemeScore], theme: &str) -> f64 {
    scores
        .iter()
        .find(|score| score.theme == theme)
        .map(|score| score.delta)
        .unwrap_or(0.0)
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

fn trajectory_summary(trajectory: &RepoTrajectoryReport) -> Value {
    json!({
        "repoId": trajectory.repo_id,
        "summary": trajectory.trajectory_summary,
        "selfImage": trajectory.self_image,
        "directionalPressures": trajectory.directional_pressures,
        "implicitGoalCandidates": trajectory.implicit_goal_candidates,
        "antiGoalCandidates": trajectory.anti_goal_candidates,
        "confidence": trajectory.confidence,
        "warnings": trajectory.warnings,
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

fn trajectory_agent_input(trajectory: &RepoTrajectoryReport) -> Value {
    json!({
        "schemaVersion": trajectory.schema_version,
        "repoId": trajectory.repo_id,
        "trajectorySummary": trajectory.trajectory_summary,
        "selfImage": trajectory.self_image,
        "earlyCommitMessages": trajectory.early_commit_messages,
        "recentCommitMessages": trajectory.recent_commit_messages,
        "trajectorySources": trajectory.trajectory_sources,
        "themeScores": trajectory.theme_scores,
        "directionalPressures": trajectory.directional_pressures,
        "implicitGoalCandidates": trajectory.implicit_goal_candidates,
        "antiGoalCandidates": trajectory.anti_goal_candidates,
        "tensions": trajectory.tensions,
        "confidence": trajectory.confidence,
        "warnings": trajectory.warnings,
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
    trajectories: &[RepoTrajectoryReport],
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
        if let Some(trajectory) = trajectories
            .iter()
            .find(|trajectory| trajectory.repo_id == report.repo_id)
        {
            out.push_str(&format!(
                "- Trajectory: {}\n\n",
                trajectory.trajectory_summary
            ));
        }
    }
    out
}

fn render_project_markdown(
    report: &RepoTerrainReport,
    profile: &RepoPersonalityProfile,
    trajectory: &RepoTrajectoryReport,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Repo Personality Projection: {}\n\n",
        report.name
    ));
    out.push_str(&format!("{}\n\n", profile.summary));
    out.push_str("## Trajectory\n\n");
    out.push_str(&format!("{}\n\n", trajectory.trajectory_summary));
    out.push_str(&format!("- Self-image: {}\n", trajectory.self_image));
    out.push_str(&format!(
        "- Directional pressures: {}\n\n",
        trajectory.directional_pressures.join(", ")
    ));
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
    trajectory: &RepoTrajectoryReport,
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
    out.push_str("## Trajectory Context\n\n");
    out.push_str(&format!("- Self-image: {}\n", trajectory.self_image));
    out.push_str(&format!(
        "- Directional pressures: {}\n",
        trajectory.directional_pressures.join(", ")
    ));
    out.push_str(&format!(
        "- Implicit goal candidates: {}\n\n",
        trajectory.implicit_goal_candidates.join(" | ")
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

fn render_memory_packet_markdown(
    report: &RepoTerrainReport,
    profile: &RepoPersonalityProfile,
    trajectory: &RepoTrajectoryReport,
    memory_source_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Repo Memory Distiller Packet: {}\n\n",
        report.name
    ));
    out.push_str("This packet is input to the role-specific Repo Memory Distiller lanes. It is not accepted truth.\n\n");
    out.push_str("## Prompt\n\n");
    out.push_str("See `repo-memory-distiller-prompt.md`.\n\n");
    out.push_str("## Source Budget\n\n");
    out.push_str(&format!(
        "- Memory sources: {} bounded excerpts\n",
        memory_source_count
    ));
    out.push_str(&format!(
        "- Max files: {}, max bytes/file: {}, max total bytes: {}\n\n",
        MEMORY_SOURCE_MAX_FILES, MEMORY_SOURCE_MAX_BYTES_PER_FILE, MEMORY_SOURCE_MAX_TOTAL_BYTES
    ));
    out.push_str("## Birth Contract\n\n");
    out.push_str("- This distiller initializes memory once for a newborn Epiphany.\n");
    out.push_str("- Personality remains the job of `repo-personality-distiller`; memory remains repo knowledge, doctrine, maps, invariants, risks, and practices.\n");
    out.push_str("- Each role receives a mission-specific filter and should write only role-relevant Ghostlight-shaped memory candidates.\n");
    out.push_str("- The distiller petitions Self; it does not mutate memory.\n\n");
    out.push_str("## Role Lanes\n\n");
    for role_id in ROLES {
        out.push_str(&format!(
            "- {}: {}\n",
            role_display(role_id),
            memory_role_filter(role_id)
        ));
    }
    out.push_str("\n## Profile Context\n\n");
    out.push_str(&format!("- Repo id: `{}`\n", profile.repo_id));
    out.push_str(&format!("- Summary: {}\n", profile.summary));
    out.push_str(&format!(
        "- Dominant pressures: {}\n",
        profile.dominant_pressures.join(", ")
    ));
    out.push_str(&format!(
        "- Trajectory: {}\n",
        trajectory.trajectory_summary
    ));
    out
}

fn render_trajectory_packet_markdown(
    report: &RepoTerrainReport,
    profile: &RepoPersonalityProfile,
    trajectory: &RepoTrajectoryReport,
    role_projection_count: usize,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Repo Trajectory Distiller Packet: {}\n\n",
        report.name
    ));
    out.push_str("This packet is input to the Repo Trajectory Distiller specialist. It is not accepted truth.\n\n");
    out.push_str("## Prompt\n\n");
    out.push_str("See `repo-trajectory-distiller-prompt.md`.\n\n");
    out.push_str("## Trajectory Readout\n\n");
    out.push_str(&format!("- Summary: {}\n", trajectory.trajectory_summary));
    out.push_str(&format!("- Self-image: {}\n", trajectory.self_image));
    out.push_str(&format!(
        "- Directional pressures: {}\n",
        trajectory.directional_pressures.join(", ")
    ));
    out.push_str(&format!(
        "- Implicit goal candidates: {}\n",
        trajectory.implicit_goal_candidates.join(" | ")
    ));
    out.push_str(&format!(
        "- Anti-goal candidates: {}\n",
        trajectory.anti_goal_candidates.join(" | ")
    ));
    out.push_str(&format!(
        "- Theme count: {}, role projections: {}\n\n",
        trajectory.theme_scores.len(),
        role_projection_count
    ));
    out.push_str("## Guardrails\n\n");
    out.push_str("- Trajectory is directional bias, not active objective truth.\n");
    out.push_str("- The distiller petitions Self; it does not mutate memory or planning.\n");
    out.push_str("- Commit history is evidence of motion, not proof of permanent identity.\n");
    out.push_str("- Keep self-image, implicit goals, and anti-goals subtle enough that later lived drift can still matter.\n");
    out.push_str(
        "- Do not stuff raw commit logs, file lists, or active backlog truth into selfPatch.\n\n",
    );
    out.push_str("## Profile Context\n\n");
    out.push_str(&format!("- Repo id: `{}`\n", profile.repo_id));
    out.push_str(&format!("- Summary: {}\n", profile.summary));
    out.push_str(&format!(
        "- Dominant pressures: {}\n",
        profile.dominant_pressures.join(", ")
    ));
    out
}

fn render_startup_markdown(result: &Value) -> String {
    let mut out = String::new();
    out.push_str("# Repo Initialization Startup\n\n");
    out.push_str(&format!(
        "- Repo id: `{}`\n",
        result["repoId"].as_str().unwrap_or("unknown")
    ));
    out.push_str(&format!(
        "- Action: `{}`\n",
        result["action"].as_str().unwrap_or("unknown")
    ));
    out.push_str(&format!(
        "- Requires review: `{}`\n\n",
        result["requiresReview"].as_bool().unwrap_or(false)
    ));
    out.push_str("## Accepted Birth Records\n\n");
    if let Some(kinds) = result["acceptedKinds"].as_array() {
        if kinds.is_empty() {
            out.push_str("- none\n");
        } else {
            for kind in kinds {
                out.push_str(&format!("- `{}`\n", kind.as_str().unwrap_or("unknown")));
            }
        }
    }
    out.push_str("\n## Generated Packets\n\n");
    if let Some(packets) = result["generatedPackets"].as_array() {
        if packets.is_empty() {
            out.push_str("- none\n");
        } else {
            for packet in packets {
                out.push_str(&format!(
                    "- `{}`: `{}`\n",
                    packet["kind"].as_str().unwrap_or("unknown"),
                    packet["packetPath"].as_str().unwrap_or("missing")
                ));
            }
        }
    }
    out.push_str("\n## Next Safe Move\n\n");
    out.push_str(result["nextSafeMove"].as_str().unwrap_or(""));
    out.push('\n');
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
        "usage: epiphany-repo-personality <scout|project|agent-packet|trajectory-packet|memory-packet|startup|accept-init|status> ...\n\
         scout --root <path> --artifact-dir <path> [--max-repos <n>]\n\
         project --repo <path> --baseline <baseline.msgpack> --artifact-dir <path>\n\
         agent-packet --store <projection.msgpack> --artifact-dir <path> [--repo-id <id>]\n\
         trajectory-packet --store <projection.msgpack> --artifact-dir <path> [--repo-id <id>]\n\
         memory-packet --store <projection.msgpack> --artifact-dir <path> [--repo-id <id>]\n\
         startup --repo <path> --baseline <baseline.msgpack> --artifact-dir <path> --init-store <init.msgpack>\n\
         accept-init --init-store <init.msgpack> --packet <packet.json> --kind <repo-trajectory|repo-personality|repo-memory> [--accepted-by <name>] [--summary <text>] [--result <distiller-result.json>] [--agent-store <agents.msgpack>] [--apply-self-patches <true|false>] [--apply-trait-seeds <true|false>] [--heartbeat-store <heartbeats.msgpack>] [--apply-heartbeat-seeds <true|false>]\n\
         status --store <baseline-or-projection.msgpack>"
    );
}
