//! Eureka pipeline store opener and writer lease (cut map D2, D3, ruling 10).
//!
//! One single-file store per clone, in the clone's **main working tree** at
//! `PIPELINE_STORE_PATH`. A caller in a linked worktree resolves through the
//! git common dir and admits into that same store, so a clone has one store and
//! one lease however many worktrees it has. `PipelineStore` reads any repo root
//! and takes no lease. `PipelineWriter` holds the one writer lease: an OS
//! byte-range lock in the git common dir, which the OS releases when the
//! holding process dies. The holder file beside it is display-only, is written
//! only while the lock is held, and is removed when the lease is dropped.

use crate::pipeline_documents::{
    Bounded, EpiphanyPipelineIdentity, EpiphanyPipelineWriterHolder, PIPELINE_SCHEMA_EPOCH,
    PipelineKind, PipelineWriterHolder, Short, register_pipeline_document_types,
    validate_pipeline_writes,
};
use crate::process_observation::capture_process_instance;
use crate::reasoning_context::TypedCommitStore;
use crate::repository_body_observer::repository_git_command;
use crate::runtime_store_backend::{RuntimeSpineBackingStore, runtime_spine_backing_store};
use anyhow::Result;
use cultcache_rs::{
    CacheBackingStore, CultCache, CultCacheEnvelope, DatabaseEntry,
    SingleFileMessagePackBackingStore,
};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Output;

pub const PIPELINE_STORE_PATH: &str = ".epiphany/pipeline/pipeline.cc";
const STORE_BINARY_LINE: &str = "/.epiphany/pipeline/pipeline.cc binary";
const LOCK_IGNORE_LINE: &str = "/.epiphany/pipeline/*.lock";
const WRITER_LEASE: &str = "epiphany-pipeline-writer.lock";
const WRITER_HOLDER: &str = "epiphany-pipeline-writer.cc";
const HOLDER_KEY: &str = "holder";

/// Typed refusals of the pipeline documents, opener and lease. Cut 3b and 3c
/// add the admission and merge refusals alongside the rules that raise them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineRefusal {
    FieldBound { field: String, limit: u32, actual: u32 },
    InvalidFormat { field: String, value: String },
    InvalidIdentity { kind: PipelineKind, key: String, expected: String },
    NotRepoRoot { repo_root: String },
    NoMainWorkTree { common_dir: String },
    StoreNotMarkedBinary { line: String },
    LockNotIgnored { line: String },
    WrongBranch { expected: String, actual: String },
    MissingIdentity,
    ForeignEpoch { found: String, expected: String },
    ForeignStore { r#type: String },
    WriterLeaseHeld { holder: Option<PipelineWriterHolder> },
    Unavailable { detail: String },
}

impl std::fmt::Display for PipelineRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "pipeline refusal: {self:?}")
    }
}

impl std::error::Error for PipelineRefusal {}

fn refusal(error: anyhow::Error) -> PipelineRefusal {
    error
        .downcast::<PipelineRefusal>()
        .unwrap_or_else(|error| PipelineRefusal::Unavailable { detail: format!("{error:#}") })
}

fn unavailable(error: impl std::fmt::Display) -> PipelineRefusal {
    PipelineRefusal::Unavailable { detail: error.to_string() }
}

#[cfg_attr(not(test), expect(dead_code, reason = "Cut 3b admission commits through this profile"))]
pub(crate) const PIPELINE_COMMIT_STORE: TypedCommitStore = TypedCommitStore {
    store_id: "epiphany-pipeline",
    backing_store: runtime_spine_backing_store,
    open_cache: pipeline_cache,
    validate_writes: validate_pipeline_writes,
};

/// Opens a pipeline store image. Refuses, before attaching the store, any
/// record of an unregistered type (`ForeignStore`), records without the
/// identity (`MissingIdentity`), and an identity at another epoch
/// (`ForeignEpoch`). It never reads the runtime or Mind epoch.
fn pipeline_cache(backing_store: RuntimeSpineBackingStore) -> Result<CultCache> {
    let envelopes = backing_store.pull_all()?;
    let mut registry = CultCache::new();
    register_pipeline_document_types(&mut registry)?;
    for envelope in &envelopes {
        if registry.put_raw_envelope(envelope.clone()).is_err() {
            return Err(PipelineRefusal::ForeignStore { r#type: envelope.r#type.clone() }.into());
        }
    }
    let identities = envelopes
        .iter()
        .filter(|envelope| envelope.r#type == EpiphanyPipelineIdentity::TYPE)
        .collect::<Vec<_>>();
    if !envelopes.is_empty() && identities.is_empty() {
        return Err(PipelineRefusal::MissingIdentity.into());
    }
    for envelope in identities {
        let found = rmp_serde::from_slice::<EpiphanyPipelineIdentity>(&envelope.payload)
            .map(|identity| identity.schema_epoch)
            .unwrap_or_else(|_| envelope.key.clone());
        if found != PIPELINE_SCHEMA_EPOCH || envelope.key != PIPELINE_SCHEMA_EPOCH {
            return Err(PipelineRefusal::ForeignEpoch {
                found,
                expected: PIPELINE_SCHEMA_EPOCH.into(),
            }
            .into());
        }
    }
    let mut cache = CultCache::new();
    register_pipeline_document_types(&mut cache)?;
    cache.add_generic_backing_store(backing_store)?;
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

/// Every git child runs with the machine's attribute and exclude files nulled,
/// on top of the system and global config `repository_git_command` already
/// nulls, so a check reads the same way on every workstation.
fn git(repo_root: &Path, args: &[&str]) -> Result<Output, PipelineRefusal> {
    repository_git_command(repo_root)
        .map_err(refusal)?
        .args(["-c", "core.attributesFile=", "-c", "core.excludesFile="])
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|error| unavailable(format!("git {}: {error}", args.join(" "))))
}

fn git_line(repo_root: &Path, args: &[&str]) -> Result<String, PipelineRefusal> {
    let output = git(repo_root, args)?;
    if !output.status.success() {
        return Err(unavailable(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Whether the committed `file` at `HEAD` carries `line` verbatim.
///
/// The rule is about what the repository commits, so this reads the committed
/// blob only. The working tree, `.git/info/exclude` and `.git/info/attributes`
/// deliberately do not count: they are per-clone state that no other clone of
/// the campaign would carry.
fn committed_line(repo_root: &Path, file: &str, line: &str) -> Result<bool, PipelineRefusal> {
    let output = git(repo_root, &["cat-file", "blob", &format!("HEAD:{file}")])?;
    if !output.status.success() {
        return Ok(false);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|committed| committed.trim_end_matches('\r') == line))
}

/// `repo_root` must be a work-tree top level (D2).
fn require_repo_root(repo_root: &Path) -> Result<PathBuf, PipelineRefusal> {
    let not_root = || PipelineRefusal::NotRepoRoot { repo_root: repo_root.display().to_string() };
    let canonical = std::fs::canonicalize(repo_root).map_err(|_| not_root())?;
    let toplevel = git_line(repo_root, &["rev-parse", "--show-toplevel"]).map_err(|_| not_root())?;
    if std::fs::canonicalize(toplevel).ok() != Some(canonical) {
        return Err(not_root());
    }
    Ok(repo_root.to_path_buf())
}

/// The clone's main working tree, from any of its worktrees (ruling 10).
///
/// The common dir is the main tree's `.git`, so the main tree is its parent,
/// and that parent must itself be a work-tree top level. A bare clone, a
/// separate git dir, or any other layout without one main tree refuses typed:
/// there is no mode in which a worktree keeps its own store.
fn main_work_tree(repo_root: &Path) -> Result<PathBuf, PipelineRefusal> {
    let common_dir = repo_root.join(git_line(repo_root, &["rev-parse", "--git-common-dir"])?);
    let no_main = || PipelineRefusal::NoMainWorkTree { common_dir: common_dir.display().to_string() };
    if common_dir.file_name() != Some(std::ffi::OsStr::new(".git")) {
        return Err(no_main());
    }
    // The path is verified by canonical comparison but returned as git spells
    // it, so no caller inherits a verbatim `\\?\` path.
    let main = common_dir.parent().ok_or_else(no_main)?.to_path_buf();
    let toplevel = git_line(&main, &["rev-parse", "--show-toplevel"]).map_err(|_| no_main())?;
    if std::fs::canonicalize(toplevel).ok() != std::fs::canonicalize(&main).ok() {
        return Err(no_main());
    }
    Ok(main)
}

fn common_dir_of(repo_root: &Path) -> Result<PathBuf, PipelineRefusal> {
    let common_dir = repo_root.join(git_line(repo_root, &["rev-parse", "--git-common-dir"])?);
    std::fs::canonicalize(&common_dir)
        .map_err(|_| PipelineRefusal::NoMainWorkTree { common_dir: common_dir.display().to_string() })
}

/// A read-only image of one repo's pipeline store. Takes no lease.
pub struct PipelineStore {
    repo_root: PathBuf,
    cache: CultCache,
}

impl PipelineStore {
    /// Opens the clone's one store. `repo_root` may be any of the clone's
    /// worktrees; the image read is always the main working tree's.
    pub fn open(repo_root: impl AsRef<Path>) -> Result<Self, PipelineRefusal> {
        let repo_root = main_work_tree(&require_repo_root(repo_root.as_ref())?)?;
        let backing_store = RuntimeSpineBackingStore::new(repo_root.join(PIPELINE_STORE_PATH));
        let cache = pipeline_cache(backing_store).map_err(refusal)?;
        Ok(Self { repo_root, cache })
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    pub fn envelopes(&self) -> Vec<CultCacheEnvelope> {
        self.cache.snapshot_envelopes()
    }
}

/// The one writer lease for a clone. Dropping the handle releases it and
/// removes the holder record.
pub struct PipelineWriter {
    repo_root: PathBuf,
    common_dir: PathBuf,
    _lease: File,
}

impl PipelineWriter {
    /// Attaches the clone's one writer lease from any of its worktrees.
    ///
    /// The process identity in the holder record is taken from this process,
    /// never from the caller: a declared pid would let a stale record name a
    /// holder that is not running. `host` and `session` stay caller-declared,
    /// because they are conventions of the process that attaches.
    pub fn attach(
        repo_root: impl AsRef<Path>,
        host: Short,
        session: Short,
    ) -> Result<Self, PipelineRefusal> {
        let holder = live_holder(host, session)?;
        holder.validate("holder")?;
        let repo_root = main_work_tree(&require_repo_root(repo_root.as_ref())?)?;
        if !committed_line(&repo_root, ".gitattributes", STORE_BINARY_LINE)? {
            return Err(PipelineRefusal::StoreNotMarkedBinary { line: STORE_BINARY_LINE.into() });
        }
        if !committed_line(&repo_root, ".gitignore", LOCK_IGNORE_LINE)? {
            return Err(PipelineRefusal::LockNotIgnored { line: LOCK_IGNORE_LINE.into() });
        }
        let common_dir = common_dir_of(&repo_root)?;
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(common_dir.join(WRITER_LEASE))
            .map_err(unavailable)?;
        if let Err(error) = lease.try_lock_exclusive() {
            if error.raw_os_error() != fs2::lock_contended_error().raw_os_error() {
                return Err(unavailable(error));
            }
            return Err(PipelineRefusal::WriterLeaseHeld {
                holder: current_holder(&common_dir, &holder),
            });
        }
        write_writer_holder(&common_dir, holder).map_err(refusal)?;
        Ok(Self { repo_root, common_dir, _lease: lease })
    }

    /// The clone's main working tree, whichever worktree attached the lease.
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    pub fn store_path(&self) -> PathBuf {
        self.repo_root.join(PIPELINE_STORE_PATH)
    }

    /// Branch binding (D2): a write is refused unless the main working tree's
    /// HEAD is the campaign's working branch. Admission supplies `expected`
    /// from the campaign. A campaign whose branch is checked out only in a
    /// linked worktree is refused here, not quietly given a second store.
    pub fn require_branch(&self, expected: &str) -> Result<(), PipelineRefusal> {
        let actual = git_line(&self.repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        if actual != expected {
            return Err(PipelineRefusal::WrongBranch { expected: expected.into(), actual });
        }
        Ok(())
    }

    /// Test-only stand-in for Cut 3b admission, and the only path in this crate
    /// that commits to a pipeline store. It takes `&self`, so no test can write
    /// without holding the lease.
    #[cfg(test)]
    fn commit_held(
        &self,
        writes: Vec<CultCacheEnvelope>,
        companions: Vec<CultCacheEnvelope>,
        provenance: crate::EpiphanyMindDocumentVersion,
    ) -> Result<crate::reasoning_context::EpiphanyMindCommitOutcome> {
        crate::reasoning_context::commit_authorized_mind_mutation(
            &PIPELINE_COMMIT_STORE,
            &self.store_path(),
            crate::reasoning_context::EpiphanyMindCommitAuthority::TypedOrganProvenance {
                organ: "eureka".into(),
                provenance,
            },
            "eureka-pipeline-test",
            Vec::new(),
            writes,
            companions,
            "2026-09-15T00:00:00Z",
        )
    }
}

impl Drop for PipelineWriter {
    /// The holder record exists only while a lease does. Removing it on drop
    /// means a record that outlives its process is evidence of a crash, not of
    /// a live holder.
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.common_dir.join(WRITER_HOLDER));
    }
}

/// This process's holder record.
fn live_holder(host: Short, session: Short) -> Result<PipelineWriterHolder, PipelineRefusal> {
    let pid = std::process::id();
    let identity = capture_process_instance(pid).map_err(unavailable)?;
    Ok(PipelineWriterHolder {
        pid,
        creation_token: identity.creation_token,
        host,
        session,
        attached_at: Short(
            identity
                .created_at_rfc3339
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        ),
    })
}

/// The recorded holder, but only when it is plausibly the process holding the
/// lease now: same host, and a live process of the same incarnation. A record
/// left by a crash names nobody.
fn current_holder(
    common_dir: &Path,
    contender: &PipelineWriterHolder,
) -> Option<PipelineWriterHolder> {
    let held = read_writer_holder(common_dir)?;
    if held.host != contender.host {
        return None;
    }
    let observed = capture_process_instance(held.pid).ok()?;
    (observed.creation_token == held.creation_token).then_some(held)
}

fn holder_cache(common_dir: &Path) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyPipelineWriterHolder>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(
        common_dir.join(WRITER_HOLDER),
    ))?;
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

fn read_writer_holder(common_dir: &Path) -> Option<PipelineWriterHolder> {
    let cache = holder_cache(common_dir).ok()?;
    Some(cache.get::<EpiphanyPipelineWriterHolder>(HOLDER_KEY).ok()??.value)
}

fn write_writer_holder(common_dir: &Path, value: PipelineWriterHolder) -> Result<()> {
    let mut cache = holder_cache(common_dir)?;
    let envelope = cache
        .prepare_entry_named(HOLDER_KEY, &EpiphanyPipelineWriterHolder { value })?
        .0;
    cache.put_envelope::<EpiphanyPipelineWriterHolder>(envelope)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EpiphanyMindDocumentVersion;
    use crate::pipeline_documents::*;
    use crate::reasoning_context::EpiphanyMindCommitOutcome;
    use std::collections::BTreeSet;
    use tempfile::{TempDir, tempdir};

    const CAMPAIGN: &str = "eureka-state";

    fn s(value: &str) -> Short {
        value.into()
    }

    fn l(value: &str) -> Label {
        value.into()
    }

    fn slug(value: &str) -> Slug {
        value.into()
    }

    fn id(kind: &str, local: &str) -> Short {
        Short(format!("{CAMPAIGN}:{kind}:{local}"))
    }

    fn sha() -> Sha {
        Sha("5f98228d".into())
    }

    fn date() -> Date {
        Date("2026-09-15".into())
    }

    fn range() -> CommitRange {
        CommitRange { base: sha(), head: sha() }
    }

    fn location() -> CodeLocation {
        CodeLocation { path: s("epiphany-core/src/pipeline_store.rs"), line: 1, end_line: Some(9) }
    }

    fn evidence() -> Evidence {
        Evidence { kind: EvidenceKind::Test, locator: "cargo test".into(), result: "ok".into() }
    }

    fn doc_ref() -> DocRef {
        DocRef { path: s("notes/eureka-pipeline-state-target.md"), start_line: 1, end_line: 9, commit: sha() }
    }

    /// One valid document of every kind, with its expected key.
    fn samples() -> Vec<(PipelineDocument, String)> {
        use PipelineDocument as D;
        let repo = || OrgRepo("GameCult/Epiphany".into());
        let branch = || s("codex/eureka-pipeline-state");
        vec![
            (D::Campaign(PipelineCampaign {
                slug: slug(CAMPAIGN), title: s("Eureka pipeline state"), repos: vec![repo()],
                working_branch: branch(), target_doc: doc_ref(),
            }), CAMPAIGN.into()),
            (D::Target(PipelineTarget {
                campaign: slug(CAMPAIGN), revision: 2,
                invariants: vec![TargetInvariant { label: l("mind-admits"), statement: "Only admission writes.".into() }],
                not_in_scope: vec!["Eve browsing".into()], canonical_implementations: vec!["CultLib".into()], doc: doc_ref(),
            }), format!("{CAMPAIGN}:target:r2")),
            (D::Question(PipelineQuestion {
                campaign: slug(CAMPAIGN), label: l("Q1"), question: "Store layout?".into(),
                options: vec![
                    QuestionOption { label: l("A"), text: "single file".into() },
                    QuestionOption { label: l("B"), text: "directory store".into() },
                ],
                recommended: l("A"), depends: vec!["Cut 3a".into()],
                raised_in: Some(PipelineRef { kind: PipelineKind::CutSpec, id: id("cut_spec", "cut-3a.r1") }),
                asked_on: date(),
            }), format!("{CAMPAIGN}:question:Q1")),
            (D::Ruling(PipelineRuling {
                campaign: slug(CAMPAIGN), label: l("R8"), answers: Some(id("question", "Q1")), choice: Some(l("A")),
                ruling: "One single-file store per repo.".into(), operator_quote: Some("all recommendations, go ahead".into()),
                ruled_on: date(),
                precedents: vec![ForeignRef {
                    repo: OrgRepo("GameCult/Aetheria".into()), commit: FullSha("a".repeat(40)), kind: PipelineKind::Ruling,
                    id: s("cultcache:ruling:R1"), payload_sha256: Sha256Hex("b".repeat(64)),
                }],
            }), format!("{CAMPAIGN}:ruling:R8")),
            (D::CutSpec(PipelineCutSpec {
                campaign: slug(CAMPAIGN), cut: l("3a"), revision: 1, title: s("Pipeline documents"), repo: repo(),
                branch: branch(), base: sha(), depends_on: vec![s("2")], first: vec!["Read the spec.".into()],
                deletes: vec![CutDelete { path: s("old.rs"), lines: 3, note: "dead".into() }],
                keeps_moves: vec!["commit owner".into()], adds: vec!["pipeline_store.rs".into()],
                file_changes: vec![FileChange { location: location(), change: "add".into() }],
                authority_map: Some(AuthorityMap {
                    owner: "core".into(), inputs: vec!["repo root".into()], outputs: vec!["handles".into()],
                    derived_state: vec!["holder".into()], forbidden_writers: vec!["MCP".into()],
                    shared_paths: vec!["attach".into()], deletion_line: "n/a".into(),
                }),
                verification: CutVerification {
                    builds: vec!["cargo check".into()],
                    tests: vec![VerificationTest { name: s("lease"), pins: "one writer".into() }],
                    negative: vec![NegativeCheck { pattern: s("Vec<u8>"), scope: "documents".into() }],
                    operator: vec!["none".into()],
                },
                subtraction_estimate: SubtractionEstimate {
                    lines_removed: 0, lines_added: 900, removed: vec![], added: vec![s("schemars")],
                },
                rulings: vec![id("ruling", "R8")], questions: vec![id("question", "Q1")],
            }), format!("{CAMPAIGN}:cut_spec:cut-3a.r1")),
            (D::CutReport(PipelineCutReport {
                campaign: slug(CAMPAIGN), cut_spec: id("cut_spec", "cut-3a.r1"), attempt: 1, repo: repo(), branch: branch(),
                commits: vec![ReportCommit { sha: sha(), subject: "Add pipeline store".into(), builds: true }],
                range: range(), verification: vec![evidence()],
                mutations: vec![MutationRecord { rule: "lease".into(), mutation: "marker file".into(), failed_as_expected: true }],
                deviations: vec![Deviation { what: "names".into(), why: "glob exports".into() }],
                forks: vec![id("question", "Q1")],
                structural_delta: StructuralDelta {
                    lines_added: 900, lines_removed: 0, dependencies_added: vec![s("schemars")],
                    dependencies_removed: vec![], formats_added: vec![s("epiphany.pipeline.*.v1")],
                    formats_removed: vec![], targets_added: vec![], targets_removed: vec![],
                },
                landed_names: vec![LandedName { name: s("PipelineStore"), path: s("epiphany-core/src/pipeline_store.rs") }],
                undone: vec!["admission".into()],
            }), format!("{CAMPAIGN}:cut_report:cut-3a.h1")),
            (D::Verdict(PipelineVerdict {
                campaign: slug(CAMPAIGN), cut_report: id("cut_report", "cut-3a.h1"), pass: 2, range: range(),
                claims: vec![VerdictClaim {
                    claim: "The lease excludes a second writer.".into(), outcome: ClaimOutcome::Falsified,
                    evidence: vec![evidence()], findings: vec![id("finding", "cut-3a.s2.F4")],
                }],
            }), format!("{CAMPAIGN}:verdict:cut-3a.s2")),
            (D::Finding(PipelineFinding {
                campaign: slug(CAMPAIGN), verdict: id("verdict", "cut-3a.s2"), label: l("F4"), range: range(),
                confidence: FindingConfidence::Confirmed, severity: FindingSeverity::High,
                claim: "A worktree takes its own lease.".into(), invariants: vec![l("mind-admits")],
                locations: vec![location()], failure_scenario: "Two runners write one clone.".into(),
                evidence: vec![evidence()], precedents: vec![],
            }), format!("{CAMPAIGN}:finding:cut-3a.s2.F4")),
            (D::FollowUp(PipelineFollowUp {
                campaign: slug(CAMPAIGN), label: l("FU-4"),
                source: PipelineRef { kind: PipelineKind::Finding, id: id("finding", "cut-3a.s2.F4") },
                repo: repo(), locations: vec![location()], item: "Cross-machine exclusion.".into(),
                why_it_can_wait: "The merge tool covers it.".into(), owner: s("Hands"),
            }), format!("{CAMPAIGN}:follow_up:FU-4")),
            (D::Resolution(PipelineResolution {
                subject: PipelineRef { kind: PipelineKind::Question, id: id("question", "Q1") },
                outcome: ResolutionOutcome::Answered { by: id("ruling", "R8") },
                rationale: "Ruled A.".into(), resolved_on: date(),
            }), format!("resolution:{CAMPAIGN}:question:Q1")),
        ]
    }

    fn campaign_sample() -> PipelineDocument {
        samples().remove(0).0
    }

    fn report_sample() -> PipelineCutReport {
        let PipelineDocument::CutReport(report) = samples().remove(5).0 else { unreachable!() };
        report
    }

    fn verdict_sample() -> PipelineVerdict {
        let PipelineDocument::Verdict(verdict) = samples().remove(6).0 else { unreachable!() };
        verdict
    }

    fn finding_sample() -> PipelineFinding {
        let PipelineDocument::Finding(finding) = samples().remove(7).0 else { unreachable!() };
        finding
    }

    fn resolution_sample() -> PipelineResolution {
        let PipelineDocument::Resolution(resolution) = samples().remove(9).0 else { unreachable!() };
        resolution
    }

    fn schema_cache() -> Result<CultCache> {
        let mut cache = CultCache::new();
        register_pipeline_document_types(&mut cache)?;
        Ok(cache)
    }

    fn identity(cache: &CultCache, epoch: &str) -> Result<CultCacheEnvelope> {
        Ok(cache
            .prepare_entry_named(epoch, &EpiphanyPipelineIdentity { schema_epoch: epoch.into() })?
            .0)
    }

    fn write_store(path: &Path, envelopes: &[CultCacheEnvelope]) -> Result<()> {
        std::fs::create_dir_all(path.parent().expect("a store path has a parent"))?;
        let mut store = SingleFileMessagePackBackingStore::new(path);
        for envelope in envelopes {
            store.push(envelope)?;
        }
        Ok(())
    }

    fn provenance_envelope() -> Result<CultCacheEnvelope> {
        Ok(schema_cache()?
            .prepare_entry_named(
                "test-provenance",
                &EpiphanyPipelineProvenance {
                    value: PipelineProvenance {
                        faculty: Faculty::Hands, agent: s("hands"), session: s("test"), tool: s("test"),
                    },
                },
            )?
            .0)
    }

    /// Commits through the lease, the way Cut 3b admission will.
    fn commit(writer: &PipelineWriter, writes: Vec<CultCacheEnvelope>) -> Result<EpiphanyMindCommitOutcome> {
        let provenance = provenance_envelope()?;
        writer.commit_held(
            writes,
            vec![provenance.clone()],
            EpiphanyMindDocumentVersion::from_envelope("epiphany-pipeline", &provenance)?,
        )
    }

    /// Test git setup. Children are `git` only; none re-launches this binary.
    fn git_ok(dir: &Path, args: &[&str]) -> Result<()> {
        let output = repository_git_command(dir)?
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=pipeline-test", "-c", "user.email=pipeline-test@example.invalid"])
            .args(args)
            .output()?;
        anyhow::ensure!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
        Ok(())
    }

    /// A committed repo at `<temp>/repo` on `main`, with the D2 lines present
    /// or absent in the commit.
    fn campaign_repo(binary: bool, ignored: bool) -> Result<(TempDir, PathBuf)> {
        let temp = tempdir()?;
        let root = temp.path().join("repo");
        std::fs::create_dir_all(&root)?;
        git_ok(&root, &["init", "-q", "-b", "main"])?;
        let line = |present: bool, line: &str| if present { format!("{line}\n") } else { "\n".into() };
        std::fs::write(root.join(".gitattributes"), line(binary, STORE_BINARY_LINE))?;
        std::fs::write(root.join(".gitignore"), line(ignored, LOCK_IGNORE_LINE))?;
        git_ok(&root, &["add", ".gitattributes", ".gitignore"])?;
        git_ok(&root, &["commit", "-q", "-m", "seed"])?;
        Ok((temp, root))
    }

    fn attach(root: &Path, session: &str) -> Result<PipelineWriter, PipelineRefusal> {
        PipelineWriter::attach(root, s("test-host"), s(session))
    }

    fn holder(session: &str) -> PipelineWriterHolder {
        live_holder(s("test-host"), s(session)).expect("this process has an identity")
    }

    #[test]
    fn every_pipeline_kind_round_trips_through_named_slot_zero() -> Result<()> {
        let cache = schema_cache()?;
        let samples = samples();
        let kinds = samples.iter().map(|(document, _)| document.kind()).collect::<BTreeSet<_>>();
        assert_eq!(kinds.len(), PipelineKind::ALL.len(), "every kind has a sample");
        for (document, _) in samples {
            document.validate()?;
            let envelope = document.prepare(&cache)?;
            assert_eq!(envelope.r#type, document.kind().type_id());
            assert_eq!(envelope.payload[0], 0x91, "{:?} payload is a one-element array", document.kind());
            assert!(
                matches!(envelope.payload[1], 0x80..=0x8f | 0xde | 0xdf),
                "{:?} slot 0 is a named map",
                document.kind()
            );
            assert_eq!(PipelineDocument::decode(&envelope)?, document);
            Ok::<_, anyhow::Error>(())?;
        }
        Ok(())
    }

    #[test]
    fn bounds_refuse_in_utf8_bytes() {
        let PipelineDocument::Campaign(mut campaign) = campaign_sample() else { unreachable!() };
        campaign.title = Short("é".repeat(100));
        assert_eq!(PipelineDocument::Campaign(campaign.clone()).validate(), Ok(()));
        campaign.title = Short(format!("{}a", "é".repeat(100)));
        assert_eq!(
            PipelineDocument::Campaign(campaign.clone()).validate(),
            Err(PipelineRefusal::FieldBound { field: "campaign.title".into(), limit: 200, actual: 201 })
        );
        campaign.title = s("ok");
        campaign.repos = vec![OrgRepo("GameCult/Epiphany".into()); 9];
        assert_eq!(
            PipelineDocument::Campaign(campaign).validate(),
            Err(PipelineRefusal::FieldBound { field: "campaign.repos".into(), limit: 8, actual: 9 })
        );
    }

    #[test]
    fn repo_fields_must_be_org_slash_repo() {
        let mut report = report_sample();
        report.repo = OrgRepo("Epiphany".into());
        assert_eq!(
            PipelineDocument::CutReport(report).validate(),
            Err(PipelineRefusal::InvalidFormat { field: "cut_report.repo".into(), value: "Epiphany".into() })
        );
        let PipelineDocument::CutSpec(mut spec) = samples().remove(4).0 else { unreachable!() };
        spec.repo = OrgRepo("GameCult/Epiphany/extra".into());
        assert!(matches!(
            PipelineDocument::CutSpec(spec).validate(),
            Err(PipelineRefusal::InvalidFormat { field, .. }) if field == "cut_spec.repo"
        ));
        let PipelineDocument::FollowUp(mut follow_up) = samples().remove(8).0 else { unreachable!() };
        follow_up.repo = OrgRepo("/Epiphany".into());
        assert!(matches!(
            PipelineDocument::FollowUp(follow_up).validate(),
            Err(PipelineRefusal::InvalidFormat { field, .. }) if field == "follow_up.repo"
        ));
    }

    #[test]
    fn keys_are_derived_and_mismatch_refuses() -> Result<()> {
        let cache = schema_cache()?;
        for (document, key) in samples() {
            assert_eq!(pipeline_key(&document), Ok(key.clone()));
            let mut envelope = document.prepare(&cache)?;
            envelope.key = format!("{key}-forged");
            let error = validate_pipeline_writes(&cache, std::slice::from_ref(&envelope)).unwrap_err();
            assert_eq!(
                error.downcast_ref::<PipelineRefusal>(),
                Some(&PipelineRefusal::InvalidIdentity { kind: document.kind(), key: envelope.key, expected: key })
            );
        }
        let mut resolution = resolution_sample();
        let answered = pipeline_key(&PipelineDocument::Resolution(resolution.clone()));
        resolution.outcome = ResolutionOutcome::Withdrawn { reason: "moot".into() };
        assert_eq!(
            pipeline_key(&PipelineDocument::Resolution(resolution)),
            answered,
            "a subject has one resolution key whatever the outcome"
        );
        Ok(())
    }

    /// F2: numeric attempt and pass, and a dot-free finding label, mean two
    /// different documents can never compose one key. These are Soul's pairs.
    #[test]
    fn composed_keys_cannot_collide() {
        let key = |document: PipelineDocument| pipeline_key(&document);

        let mut wide_label = finding_sample();
        wide_label.label = l("F1.G");
        assert!(
            matches!(key(PipelineDocument::Finding(wide_label)), Err(PipelineRefusal::InvalidFormat { .. })),
            "a finding label carrying a dot is refused, not silently composed"
        );
        let mut deep_verdict = finding_sample();
        deep_verdict.verdict = id("verdict", "cut-3a.s1.F1");
        deep_verdict.label = l("G");
        assert!(
            matches!(key(PipelineDocument::Finding(deep_verdict)), Err(PipelineRefusal::InvalidFormat { .. })),
            "a verdict local that is not cut-<label>.s<N> is refused"
        );
        let mut plain = finding_sample();
        plain.verdict = id("verdict", "cut-3a.s1");
        plain.label = l("F1");
        assert_eq!(key(PipelineDocument::Finding(plain)), Ok(format!("{CAMPAIGN}:finding:cut-3a.s1.F1")));

        let mut deep_spec = report_sample();
        deep_spec.cut_spec = id("cut_spec", "cut-3a.h1.r1");
        deep_spec.attempt = 2;
        assert!(
            matches!(key(PipelineDocument::CutReport(deep_spec)), Err(PipelineRefusal::InvalidFormat { .. })),
            "a cut label carrying a dot is refused, so cut-3a.h1.h2 has one source"
        );
        let mut attempt_two = report_sample();
        attempt_two.attempt = 2;
        assert_eq!(key(PipelineDocument::CutReport(attempt_two)), Ok(format!("{CAMPAIGN}:cut_report:cut-3a.h2")));
    }

    /// F3: every parent id is parsed strictly. These are the bad parents Soul
    /// found accepted, plus the wrong-kind and marker rules that mutations S1
    /// and S3 remove.
    #[test]
    fn parent_ids_are_parsed_strictly() {
        let refused = |parent: &str| {
            let mut report = report_sample();
            report.cut_spec = Short(parent.into());
            let key = pipeline_key(&PipelineDocument::CutReport(report));
            assert!(
                matches!(&key, Err(PipelineRefusal::InvalidFormat { field, .. }) if field == "cut_report.cut_spec"),
                "parent {parent:?} must be refused, got {key:?}"
            );
        };
        refused(&format!("{CAMPAIGN}:cut_spec:cut-3a.r1:junk"));
        refused(&format!("{CAMPAIGN}:cut_spec:cut-3a.rX"));
        refused(&format!("{CAMPAIGN}:cut_spec:cut-..r1"));
        refused(&format!("{CAMPAIGN}:cut_spec:cut-.r1"));
        refused(&format!("{CAMPAIGN}:cut_spec:cut-3a.r1 "));
        refused(&format!("{CAMPAIGN}:cut_spec:cut-3a.r"));
        refused(&format!("{CAMPAIGN}:cut_report:cut-3a.h1"));
        // Right kind and a well-formed number, but the marker belongs to
        // another kind: the marker rule is pinned on its own.
        refused(&format!("{CAMPAIGN}:cut_spec:cut-3a.h1"));
        refused(&format!("{CAMPAIGN}:cut_spec:cut-3a.s1"));
        refused(&format!("{CAMPAIGN}:CUT_SPEC:cut-3a.r1"));
        refused(&format!("EUREKA-STATE:cut_spec:cut-3a.r1"));
        refused(&format!("{CAMPAIGN}:cut_spec:cut-3a.r1:"));
        refused(&format!("{CAMPAIGN}::cut-3a.r1"));
        refused("cut-3a.r1");
        let mut other_campaign = report_sample();
        other_campaign.campaign = slug("other-campaign");
        assert!(
            matches!(
                pipeline_key(&PipelineDocument::CutReport(other_campaign)),
                Err(PipelineRefusal::InvalidFormat { .. })
            ),
            "a parent in another campaign is refused"
        );
        let mut verdict = verdict_sample();
        verdict.cut_report = id("cut_report", "cut-3a.r1");
        assert!(
            matches!(pipeline_key(&PipelineDocument::Verdict(verdict)), Err(PipelineRefusal::InvalidFormat { .. })),
            "a verdict's parent must carry the report marker, not the spec marker"
        );
    }

    /// F4: a resolution's subject id is a full id of the declared kind.
    #[test]
    fn resolution_subject_is_a_full_id_of_its_kind() {
        let refused = |kind: PipelineKind, subject: &str| {
            let mut resolution = resolution_sample();
            resolution.subject = PipelineRef { kind, id: Short(subject.into()) };
            let document = PipelineDocument::Resolution(resolution);
            assert!(
                matches!(document.validate(), Err(PipelineRefusal::InvalidFormat { .. })),
                "subject {subject:?} must fail validation"
            );
            assert!(
                matches!(pipeline_key(&document), Err(PipelineRefusal::InvalidFormat { .. })),
                "subject {subject:?} must not compose a key"
            );
        };
        refused(PipelineKind::Question, "");
        refused(PipelineKind::Question, "not an id: spaces and : colons");
        refused(PipelineKind::Question, "eureka-state:question:1１");
        refused(PipelineKind::Question, &format!("{CAMPAIGN}:ruling:R8"));
        refused(PipelineKind::Ruling, &format!("{CAMPAIGN}:question:Q1"));
        refused(PipelineKind::Question, &format!("{CAMPAIGN}:question:.."));

        let mut valid = resolution_sample();
        valid.subject = PipelineRef { kind: PipelineKind::Ruling, id: id("ruling", "R8") };
        assert_eq!(
            pipeline_key(&PipelineDocument::Resolution(valid)),
            Ok(format!("resolution:{CAMPAIGN}:ruling:R8"))
        );
    }

    #[test]
    fn opener_refuses_foreign_epoch_missing_identity_and_runtime_store_byte_identically() -> Result<()> {
        let (_temp, root) = campaign_repo(true, true)?;
        let store = root.join(PIPELINE_STORE_PATH);
        let cache = schema_cache()?;
        let campaign = campaign_sample().prepare(&cache)?;
        let writer = attach(&root, "opener")?;
        let refused = |expected: &dyn Fn(&PipelineRefusal) -> bool| -> Result<()> {
            let before = std::fs::read(&store)?;
            let opened = PipelineStore::open(&root).err().expect("the opener refuses");
            assert!(expected(&opened), "{opened:?}");
            let committed = commit(&writer, vec![campaign.clone()]).unwrap_err();
            assert_eq!(committed.downcast_ref::<PipelineRefusal>(), Some(&opened), "the commit profile refuses too");
            assert_eq!(std::fs::read(&store)?, before);
            Ok(())
        };
        write_store(&store, &[identity(&cache, "epiphany.pipeline.epoch.v0")?, campaign.clone()])?;
        refused(&|refusal| {
            refusal
                == &PipelineRefusal::ForeignEpoch {
                    found: "epiphany.pipeline.epoch.v0".into(),
                    expected: "epiphany.pipeline.epoch.v1".into(),
                }
        })?;
        std::fs::remove_file(&store)?;
        write_store(&store, &[campaign.clone()])?;
        refused(&|refusal| refusal == &PipelineRefusal::MissingIdentity)?;
        std::fs::remove_file(&store)?;
        crate::runtime_spine::initialize_runtime_spine(
            &store,
            crate::runtime_spine::RuntimeSpineInitOptions {
                runtime_id: "pipeline-foreign".into(),
                display_name: "Pipeline foreign".into(),
                created_at: "2026-09-15T00:00:00Z".into(),
            },
        )?;
        refused(&|refusal| {
            matches!(refusal, PipelineRefusal::ForeignStore { r#type } if !r#type.starts_with("epiphany.pipeline."))
        })
    }

    #[test]
    fn runtime_spine_cache_refuses_a_pipeline_store() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("pipeline.cc");
        let cache = schema_cache()?;
        write_store(&store, &[identity(&cache, PIPELINE_SCHEMA_EPOCH)?, campaign_sample().prepare(&cache)?])?;
        let error = crate::runtime_spine_cache(&store)
            .and_then(|mut runtime| runtime.pull_all_backing_stores())
            .unwrap_err();
        assert!(error.to_string().contains("epiphany.pipeline."), "{error:#}");
        Ok(())
    }

    #[test]
    fn pipeline_commit_profile_registers_the_receipt_and_commits_the_first_write() -> Result<()> {
        let (_temp, root) = campaign_repo(true, true)?;
        let store = root.join(PIPELINE_STORE_PATH);
        let cache = schema_cache()?;
        let campaign = campaign_sample().prepare(&cache)?;
        let writer = attach(&root, "first-write")?;
        let EpiphanyMindCommitOutcome::Committed(receipt) =
            commit(&writer, vec![identity(&cache, PIPELINE_SCHEMA_EPOCH)?, campaign.clone()])?
        else {
            panic!("the first pipeline write commits");
        };
        assert!(receipt.writes.iter().all(|version| version.store_id == "epiphany-pipeline"));
        let envelopes = PipelineStore::open(&root)?.envelopes();
        assert!(envelopes.iter().any(|entry| entry.key == campaign.key && entry.payload == campaign.payload));
        assert!(envelopes.iter().any(|entry| entry.key == receipt.receipt_id));
        let before = std::fs::read(&store)?;
        let mut forged = campaign;
        forged.key = "forged".into();
        let error = commit(&writer, vec![forged]).unwrap_err();
        assert!(matches!(error.downcast_ref(), Some(PipelineRefusal::InvalidIdentity { .. })), "{error:#}");
        assert_eq!(std::fs::read(&store)?, before);
        Ok(())
    }

    /// F1 and ruling 12: the profile that owns the commit refuses a first write
    /// without the identity, so it can never admit a store its opener refuses.
    #[test]
    fn a_first_commit_without_the_identity_is_refused() -> Result<()> {
        let (_temp, root) = campaign_repo(true, true)?;
        let store = root.join(PIPELINE_STORE_PATH);
        let cache = schema_cache()?;
        let campaign = campaign_sample().prepare(&cache)?;
        let writer = attach(&root, "identity")?;
        let error = commit(&writer, vec![campaign.clone()]).unwrap_err();
        assert_eq!(error.downcast_ref::<PipelineRefusal>(), Some(&PipelineRefusal::MissingIdentity));
        assert!(!store.exists(), "the refused first write leaves no store behind");
        commit(&writer, vec![identity(&cache, PIPELINE_SCHEMA_EPOCH)?, campaign.clone()])?;
        let after_identity = std::fs::read(&store)?;
        let mut second = campaign;
        second.key = pipeline_key(&campaign_sample())?;
        commit(&writer, vec![second])?;
        assert!(
            PipelineStore::open(&root).is_ok(),
            "a store whose first write carried the identity always reopens"
        );
        assert!(!after_identity.is_empty());
        Ok(())
    }

    #[test]
    fn second_writer_is_refused_naming_the_holder_and_released_on_drop() -> Result<()> {
        let (_temp, root) = campaign_repo(true, true)?;
        let cache = schema_cache()?;
        let campaign = campaign_sample().prepare(&cache)?;
        let first = attach(&root, "first")?;
        commit(&first, vec![identity(&cache, PIPELINE_SCHEMA_EPOCH)?, campaign.clone()])?;
        let contender = root.clone();
        let second = std::thread::spawn(move || attach(&contender, "second").err())
            .join()
            .expect("the contender thread completes");
        assert_eq!(second, Some(PipelineRefusal::WriterLeaseHeld { holder: Some(holder("first")) }));
        let read = PipelineStore::open(&root)?;
        assert!(read.envelopes().iter().any(|entry| entry.key == campaign.key), "reads stay allowed while leased");
        let common_dir = common_dir_of(&root)?;
        drop(first);
        assert!(
            !common_dir.join(WRITER_HOLDER).exists(),
            "the holder record is removed with the lease"
        );
        let second = attach(&root, "second")?;
        assert_eq!(second.store_path(), root.join(PIPELINE_STORE_PATH));
        Ok(())
    }

    /// F7: a record that outlives its process names nobody.
    #[test]
    fn a_stale_holder_is_not_named() -> Result<()> {
        let (_temp, root) = campaign_repo(true, true)?;
        let common_dir = common_dir_of(&root)?;
        let stale = |mutate: &dyn Fn(&mut PipelineWriterHolder)| -> Result<Option<PipelineWriterHolder>> {
            let mut record = holder("crashed");
            mutate(&mut record);
            write_writer_holder(&common_dir, record)?;
            // The crash window: a record on disk, and the lease held by a
            // handle that is not a PipelineWriter.
            let blocker = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(common_dir.join(WRITER_LEASE))?;
            blocker.try_lock_exclusive()?;
            let refused = attach(&root, "contender").err();
            let Some(PipelineRefusal::WriterLeaseHeld { holder }) = refused else {
                panic!("a held lease refuses, got {refused:?}");
            };
            FileExt::unlock(&blocker)?;
            Ok(holder)
        };
        assert_eq!(
            stale(&|record| record.creation_token = record.creation_token.wrapping_add(1))?,
            None,
            "a record of a dead incarnation is not named"
        );
        assert_eq!(
            stale(&|record| record.pid = 0xFFFF_FFFE)?,
            None,
            "a record of an absent pid is not named"
        );
        assert_eq!(
            stale(&|record| record.host = s("another-machine"))?,
            None,
            "a record from another host is not named"
        );
        assert_eq!(
            stale(&|_| ())?,
            Some(holder("crashed")),
            "a record of this live process is still named"
        );
        Ok(())
    }

    /// F6 and ruling 10: one store per clone, in the main working tree.
    #[test]
    fn a_worktree_writes_the_main_trees_store() -> Result<()> {
        let (temp, root) = campaign_repo(true, true)?;
        let worktree = temp.path().join("worktree");
        git_ok(&root, &["worktree", "add", "-q", "-b", "side", &worktree.to_string_lossy()])?;

        let from_worktree = attach(&worktree, "worktree")?;
        assert_eq!(
            std::fs::canonicalize(from_worktree.repo_root())?,
            std::fs::canonicalize(&root)?,
            "a worktree caller resolves to the main working tree"
        );
        assert_eq!(
            from_worktree.require_branch("side"),
            Err(PipelineRefusal::WrongBranch { expected: "side".into(), actual: "main".into() }),
            "the branch binds to the main tree, not the caller's worktree"
        );
        assert_eq!(from_worktree.require_branch("main"), Ok(()));

        let cache = schema_cache()?;
        let campaign = campaign_sample().prepare(&cache)?;
        commit(&from_worktree, vec![identity(&cache, PIPELINE_SCHEMA_EPOCH)?, campaign.clone()])?;
        assert!(root.join(PIPELINE_STORE_PATH).exists(), "the store lives in the main tree");
        assert!(!worktree.join(PIPELINE_STORE_PATH).exists(), "and nowhere else");
        assert!(
            PipelineStore::open(&worktree)?
                .envelopes()
                .iter()
                .any(|entry| entry.key == campaign.key),
            "a worktree read sees the same store"
        );

        let contender = root.clone();
        assert_eq!(
            std::thread::spawn(move || attach(&contender, "main").err())
                .join()
                .expect("the contender thread completes"),
            Some(PipelineRefusal::WriterLeaseHeld { holder: Some(holder("worktree")) }),
            "the lease is one per clone, whichever tree holds it"
        );
        Ok(())
    }

    #[test]
    fn writer_refuses_unmarked_binary_and_unignored_lock_and_wrong_branch() -> Result<()> {
        let (_unmarked_temp, unmarked) = campaign_repo(false, true)?;
        assert_eq!(
            attach(&unmarked, "t").err(),
            Some(PipelineRefusal::StoreNotMarkedBinary { line: STORE_BINARY_LINE.into() })
        );
        let (_unignored_temp, unignored) = campaign_repo(true, false)?;
        assert_eq!(
            attach(&unignored, "t").err(),
            Some(PipelineRefusal::LockNotIgnored { line: LOCK_IGNORE_LINE.into() })
        );
        let (_temp, root) = campaign_repo(true, true)?;
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested)?;
        assert_eq!(
            attach(&nested, "t").err(),
            Some(PipelineRefusal::NotRepoRoot { repo_root: nested.display().to_string() })
        );
        let writer = attach(&root, "t")?;
        assert_eq!(
            writer.require_branch("codex/eureka-pipeline-state"),
            Err(PipelineRefusal::WrongBranch {
                expected: "codex/eureka-pipeline-state".into(),
                actual: "main".into(),
            })
        );
        assert_eq!(writer.require_branch("main"), Ok(()));
        Ok(())
    }

    /// F8: the git checks read the committed state, so they answer the same on
    /// every machine. Working-tree edits and `.git/info/exclude` do not count.
    #[test]
    fn git_checks_read_the_committed_state_only() -> Result<()> {
        let (_temp, root) = campaign_repo(true, true)?;
        std::fs::write(root.join(".gitattributes"), "")?;
        std::fs::write(root.join(".gitignore"), "")?;
        assert!(
            attach(&root, "committed").is_ok(),
            "an uncommitted deletion does not withdraw the committed lines"
        );

        let (_bare_temp, uncommitted) = campaign_repo(false, false)?;
        std::fs::write(uncommitted.join(".gitattributes"), format!("{STORE_BINARY_LINE}\n"))?;
        std::fs::write(uncommitted.join(".gitignore"), format!("{LOCK_IGNORE_LINE}\n"))?;
        assert_eq!(
            attach(&uncommitted, "uncommitted").err(),
            Some(PipelineRefusal::StoreNotMarkedBinary { line: STORE_BINARY_LINE.into() }),
            "a working-tree line that is not committed does not satisfy the rule"
        );

        let (_info_temp, info) = campaign_repo(true, false)?;
        let info_dir = info.join(".git").join("info");
        std::fs::create_dir_all(&info_dir)?;
        std::fs::write(info_dir.join("exclude"), format!("{LOCK_IGNORE_LINE}\n"))?;
        assert_eq!(
            attach(&info, "info").err(),
            Some(PipelineRefusal::LockNotIgnored { line: LOCK_IGNORE_LINE.into() }),
            "a per-clone exclude file does not satisfy a rule about committed state"
        );
        Ok(())
    }

    #[test]
    fn a_bare_clone_is_not_a_repo_root() -> Result<()> {
        let temp = tempdir()?;
        let bare = temp.path().join("bare.git");
        std::fs::create_dir_all(&bare)?;
        git_ok(&bare, &["init", "-q", "--bare"])?;
        assert!(
            matches!(attach(&bare, "bare").err(), Some(PipelineRefusal::NotRepoRoot { .. })),
            "a bare clone refuses typed, with no worktree-only mode"
        );
        Ok(())
    }

    /// Ruling 10's odd-layout arm: a work-tree top level whose git dir is not a
    /// sibling `.git` has no main working tree to resolve, and refuses typed.
    #[test]
    fn a_separate_git_dir_has_no_main_work_tree() -> Result<()> {
        let temp = tempdir()?;
        let root = temp.path().join("repo");
        let git_dir = temp.path().join("elsewhere.git");
        std::fs::create_dir_all(&root)?;
        git_ok(&root, &["init", "-q", "-b", "main", "--separate-git-dir", &git_dir.to_string_lossy()])?;
        assert!(
            matches!(attach(&root, "separate").err(), Some(PipelineRefusal::NoMainWorkTree { .. })),
            "a separate git dir refuses typed, with no worktree-only mode"
        );
        assert!(
            matches!(PipelineStore::open(&root).err(), Some(PipelineRefusal::NoMainWorkTree { .. })),
            "and the read path refuses it the same way"
        );
        Ok(())
    }

    #[test]
    fn pipeline_published_schemas_match_derivation() -> Result<()> {
        let published = Path::new(env!("CARGO_MANIFEST_DIR")).join("../schemas/cultnet");
        let index: serde_json::Value = serde_json::from_slice(&std::fs::read(published.join("index.json"))?)?;
        let derived_dir = std::env::temp_dir().join("epiphany-pipeline-schemas");
        let mut stale = Vec::new();
        for kind in PipelineKind::ALL {
            let file = format!("{}.schema.json", kind.type_id());
            let mut schema = serde_json::to_value(kind.derived_schema())?;
            schema["$id"] = format!("https://gamecult.dev/epiphany/cultnet/{file}").into();
            let derived = format!("{}\n", serde_json::to_string_pretty(&schema)?);
            if std::fs::read(published.join(&file)).ok().as_deref() != Some(derived.as_bytes()) {
                std::fs::create_dir_all(&derived_dir)?;
                std::fs::write(derived_dir.join(&file), &derived)?;
                stale.push(derived_dir.join(&file));
            }
            let entry = serde_json::json!({
                "schemaId": format!("https://gamecult.dev/epiphany/cultnet/{file}"),
                "kind": "document_payload",
                "wireContracts": ["cultnet.schema.v0"],
                "schemaVersion": kind.type_id(),
                "documentType": kind.type_id(),
                "title": format!("Epiphany Pipeline {kind:?} v1"),
                "path": file,
            });
            assert!(
                index["schemas"].as_array().is_some_and(|schemas| schemas.contains(&entry)),
                "index.json lacks {entry}"
            );
        }
        assert!(stale.is_empty(), "published pipeline schemas differ from the Rust derivation; derived copies: {stale:?}");
        Ok(())
    }

    /// The child is `powershell.exe` running a fixed script that byte-range
    /// locks the lease file and sleeps. It cannot launch this test binary.
    #[cfg(windows)]
    #[test]
    fn writer_lease_releases_when_the_holder_process_dies() -> Result<()> {
        struct KillOnDrop(std::process::Child);
        impl Drop for KillOnDrop {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        fn eventually(condition: impl Fn() -> bool) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            while !condition() {
                assert!(std::time::Instant::now() < deadline, "condition not reached in 30 s");
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        let (_temp, root) = campaign_repo(true, true)?;
        let lease = common_dir_of(&root)?.join(WRITER_LEASE);
        let script = format!(
            "$f=[IO.File]::Open('{}','OpenOrCreate','ReadWrite','ReadWrite'); \
             while($true){{try{{$f.Lock(0,[long]::MaxValue);break}}catch{{Start-Sleep -Milliseconds 50}}}}; \
             Start-Sleep -Seconds 120",
            lease.display()
        );
        let mut child = KillOnDrop(
            std::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", &script])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?,
        );
        eventually(|| matches!(attach(&root, "probe"), Err(PipelineRefusal::WriterLeaseHeld { .. })));
        assert_eq!(
            attach(&root, "probe").err(),
            Some(PipelineRefusal::WriterLeaseHeld { holder: None }),
            "another process holds the lease and left no record of its own"
        );
        child.0.kill()?;
        child.0.wait()?;
        eventually(|| attach(&root, "after").is_ok());
        Ok(())
    }
}
