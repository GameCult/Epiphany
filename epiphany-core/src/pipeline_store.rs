//! Eureka pipeline store opener and writer lease (cut map D2, D3).
//!
//! One single-file store per repo at `PIPELINE_STORE_PATH`. `PipelineStore`
//! reads any repo root and takes no lease. `PipelineWriter` holds the one
//! writer lease per clone: an OS byte-range lock in the git common dir, which
//! the OS releases when the holding process dies. The holder file beside it
//! is display-only.

use crate::pipeline_documents::{
    Bounded, EpiphanyPipelineIdentity, EpiphanyPipelineWriterHolder, PIPELINE_SCHEMA_EPOCH,
    PipelineKind, PipelineWriterHolder, register_pipeline_document_types,
    validate_pipeline_write_envelope,
};
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
const STORE_LOCK_PATH: &str = ".epiphany/pipeline/pipeline.cc.lock";
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
    validate_write: validate_pipeline_write_envelope,
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

fn git(repo_root: &Path, args: &[&str]) -> Result<Output, PipelineRefusal> {
    repository_git_command(repo_root)
        .map_err(refusal)?
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

/// A read-only image of one repo's pipeline store. Takes no lease.
pub struct PipelineStore {
    repo_root: PathBuf,
    cache: CultCache,
}

impl PipelineStore {
    pub fn open(repo_root: impl AsRef<Path>) -> Result<Self, PipelineRefusal> {
        let repo_root = require_repo_root(repo_root.as_ref())?;
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

/// The one writer lease for a clone. Dropping the handle releases it.
pub struct PipelineWriter {
    repo_root: PathBuf,
    _lease: File,
}

impl PipelineWriter {
    pub fn attach(
        repo_root: impl AsRef<Path>,
        holder: PipelineWriterHolder,
    ) -> Result<Self, PipelineRefusal> {
        holder.validate("holder")?;
        let repo_root = require_repo_root(repo_root.as_ref())?;
        let binary = git_line(&repo_root, &["check-attr", "binary", "--", PIPELINE_STORE_PATH])?;
        if !binary.ends_with(": binary: set") {
            return Err(PipelineRefusal::StoreNotMarkedBinary { line: STORE_BINARY_LINE.into() });
        }
        match git(&repo_root, &["check-ignore", "-q", "--no-index", STORE_LOCK_PATH])?.status.code() {
            Some(0) => {}
            Some(1) => return Err(PipelineRefusal::LockNotIgnored { line: LOCK_IGNORE_LINE.into() }),
            code => return Err(unavailable(format!("git check-ignore exited with {code:?}"))),
        }
        let common_dir = repo_root.join(git_line(&repo_root, &["rev-parse", "--git-common-dir"])?);
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
            return Err(PipelineRefusal::WriterLeaseHeld { holder: read_writer_holder(&common_dir) });
        }
        write_writer_holder(&common_dir, holder).map_err(refusal)?;
        Ok(Self { repo_root, _lease: lease })
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    pub fn store_path(&self) -> PathBuf {
        self.repo_root.join(PIPELINE_STORE_PATH)
    }

    /// Branch binding (D2): a write is refused unless HEAD is the campaign's
    /// working branch. Admission supplies `expected` from the campaign.
    pub fn require_branch(&self, expected: &str) -> Result<(), PipelineRefusal> {
        let actual = git_line(&self.repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        if actual != expected {
            return Err(PipelineRefusal::WrongBranch { expected: expected.into(), actual });
        }
        Ok(())
    }
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
    use crate::reasoning_context::{
        EpiphanyMindCommitAuthority, EpiphanyMindCommitOutcome, commit_authorized_mind_mutation,
    };
    use std::collections::BTreeSet;
    use tempfile::{TempDir, tempdir};

    const CAMPAIGN: &str = "eureka-state";

    fn s(value: &str) -> Short {
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
        let repo = || s("GameCult/Epiphany");
        let branch = || s("codex/eureka-pipeline-state");
        vec![
            (D::Campaign(PipelineCampaign {
                slug: s(CAMPAIGN), title: s("Eureka pipeline state"), repos: vec![repo()],
                working_branch: branch(), target_doc: doc_ref(),
            }), CAMPAIGN.into()),
            (D::Target(PipelineTarget {
                campaign: s(CAMPAIGN), revision: 2,
                invariants: vec![TargetInvariant { label: s("mind-admits"), statement: "Only admission writes.".into() }],
                not_in_scope: vec!["Eve browsing".into()], canonical_implementations: vec!["CultLib".into()], doc: doc_ref(),
            }), format!("{CAMPAIGN}:target:r2")),
            (D::Question(PipelineQuestion {
                campaign: s(CAMPAIGN), label: s("Q1"), question: "Store layout?".into(),
                options: vec![
                    QuestionOption { label: s("A"), text: "single file".into() },
                    QuestionOption { label: s("B"), text: "directory store".into() },
                ],
                recommended: s("A"), depends: vec!["Cut 3a".into()],
                raised_in: Some(PipelineRef { kind: PipelineKind::CutSpec, id: id("cut_spec", "cut-3a.r1") }),
                asked_on: date(),
            }), format!("{CAMPAIGN}:question:Q1")),
            (D::Ruling(PipelineRuling {
                campaign: s(CAMPAIGN), label: s("R8"), answers: Some(id("question", "Q1")), choice: Some(s("A")),
                ruling: "One single-file store per repo.".into(), operator_quote: Some("all recommendations, go ahead".into()),
                ruled_on: date(),
                precedents: vec![ForeignRef {
                    repo: s("GameCult/Aetheria"), commit: Sha("a".repeat(40)), kind: PipelineKind::Ruling,
                    id: s("cultcache:ruling:R1"), payload_sha256: "b".repeat(64),
                }],
            }), format!("{CAMPAIGN}:ruling:R8")),
            (D::CutSpec(PipelineCutSpec {
                campaign: s(CAMPAIGN), cut: s("3a"), revision: 1, title: s("Pipeline documents"), repo: repo(),
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
                campaign: s(CAMPAIGN), cut_spec: id("cut_spec", "cut-3a.r1"), attempt: s("1"), repo: repo(), branch: branch(),
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
                landed_names: vec![LandedName { name: s("PipelineStore"), kind: LandedNameKind::Type, location: location() }],
                undone: vec!["admission".into()],
            }), format!("{CAMPAIGN}:cut_report:cut-3a.h1")),
            (D::Verdict(PipelineVerdict {
                campaign: s(CAMPAIGN), cut_report: id("cut_report", "cut-3a.h1"), pass: s("2"), range: range(),
                claims: vec![VerdictClaim {
                    claim: "The lease excludes a second writer.".into(), outcome: ClaimOutcome::Falsified,
                    evidence: vec![evidence()], findings: vec![id("finding", "cut-3a.s2.F4")],
                }],
            }), format!("{CAMPAIGN}:verdict:cut-3a.s2")),
            (D::Finding(PipelineFinding {
                campaign: s(CAMPAIGN), verdict: id("verdict", "cut-3a.s2"), label: s("F4"), range: range(),
                confidence: FindingConfidence::Confirmed, severity: FindingSeverity::High,
                claim: "A worktree takes its own lease.".into(), invariants: vec![s("mind-admits")],
                locations: vec![location()], failure_scenario: "Two runners write one clone.".into(),
                evidence: vec![evidence()], precedents: vec![],
            }), format!("{CAMPAIGN}:finding:cut-3a.s2.F4")),
            (D::FollowUp(PipelineFollowUp {
                campaign: s(CAMPAIGN), label: s("FU-4"),
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

    fn commit(store: &Path, writes: Vec<CultCacheEnvelope>) -> Result<EpiphanyMindCommitOutcome> {
        let provenance = schema_cache()?
            .prepare_entry_named(
                "test-provenance",
                &EpiphanyPipelineProvenance {
                    value: PipelineProvenance {
                        faculty: Faculty::Hands, agent: s("hands"), session: s("test"), tool: s("test"),
                    },
                },
            )?
            .0;
        commit_authorized_mind_mutation(
            &PIPELINE_COMMIT_STORE,
            store,
            EpiphanyMindCommitAuthority::TypedOrganProvenance {
                organ: "eureka".into(),
                provenance: EpiphanyMindDocumentVersion::from_envelope("epiphany-pipeline", &provenance)?,
            },
            "eureka-pipeline-test",
            Vec::new(),
            writes,
            vec![provenance],
            "2026-09-15T00:00:00Z",
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
    /// or absent.
    fn campaign_repo(binary: bool, ignored: bool) -> Result<(TempDir, PathBuf)> {
        let temp = tempdir()?;
        let root = temp.path().join("repo");
        std::fs::create_dir_all(&root)?;
        git_ok(&root, &["init", "-q", "-b", "main"])?;
        let line = |present: bool, line: &str| if present { format!("{line}\n") } else { "\n".into() };
        std::fs::write(root.join(".gitattributes"), line(binary, "/.epiphany/pipeline/pipeline.cc binary"))?;
        std::fs::write(root.join(".gitignore"), line(ignored, "/.epiphany/pipeline/*.lock"))?;
        git_ok(&root, &["add", ".gitattributes", ".gitignore"])?;
        git_ok(&root, &["commit", "-q", "-m", "seed"])?;
        Ok((temp, root))
    }

    fn holder(session: &str) -> PipelineWriterHolder {
        PipelineWriterHolder {
            pid: std::process::id(), host: s("test-host"), session: s(session),
            attached_at: "2026-09-15T00:00:00Z".into(),
        }
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
            validate_pipeline_write_envelope(&envelope)?;
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
        campaign.repos = vec![s("GameCult/Epiphany"); 9];
        assert_eq!(
            PipelineDocument::Campaign(campaign).validate(),
            Err(PipelineRefusal::FieldBound { field: "campaign.repos".into(), limit: 8, actual: 9 })
        );
    }

    #[test]
    fn keys_are_derived_and_mismatch_refuses() -> Result<()> {
        let cache = schema_cache()?;
        for (document, key) in samples() {
            assert_eq!(pipeline_key(&document), Ok(key.clone()));
            let mut envelope = document.prepare(&cache)?;
            envelope.key = format!("{key}-forged");
            let error = validate_pipeline_write_envelope(&envelope).unwrap_err();
            assert_eq!(
                error.downcast_ref::<PipelineRefusal>(),
                Some(&PipelineRefusal::InvalidIdentity { kind: document.kind(), key: envelope.key, expected: key })
            );
        }
        let PipelineDocument::Resolution(mut resolution) = samples().remove(9).0 else { unreachable!() };
        let answered = pipeline_key(&PipelineDocument::Resolution(resolution.clone()));
        resolution.outcome = ResolutionOutcome::Withdrawn { reason: "moot".into() };
        assert_eq!(
            pipeline_key(&PipelineDocument::Resolution(resolution)),
            answered,
            "a subject has one resolution key whatever the outcome"
        );
        Ok(())
    }

    #[test]
    fn opener_refuses_foreign_epoch_missing_identity_and_runtime_store_byte_identically() -> Result<()> {
        let (_temp, root) = campaign_repo(true, true)?;
        let store = root.join(PIPELINE_STORE_PATH);
        let cache = schema_cache()?;
        let campaign = campaign_sample().prepare(&cache)?;
        let refused = |expected: &dyn Fn(&PipelineRefusal) -> bool| -> Result<()> {
            let before = std::fs::read(&store)?;
            let opened = PipelineStore::open(&root).err().expect("the opener refuses");
            assert!(expected(&opened), "{opened:?}");
            let committed = commit(&store, vec![campaign.clone()]).unwrap_err();
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
        let EpiphanyMindCommitOutcome::Committed(receipt) =
            commit(&store, vec![identity(&cache, PIPELINE_SCHEMA_EPOCH)?, campaign.clone()])?
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
        let error = commit(&store, vec![forged]).unwrap_err();
        assert!(matches!(error.downcast_ref(), Some(PipelineRefusal::InvalidIdentity { .. })), "{error:#}");
        assert_eq!(std::fs::read(&store)?, before);
        Ok(())
    }

    #[test]
    fn second_writer_is_refused_naming_the_holder_and_released_on_drop() -> Result<()> {
        let (_temp, root) = campaign_repo(true, true)?;
        let cache = schema_cache()?;
        let campaign = campaign_sample().prepare(&cache)?;
        commit(&root.join(PIPELINE_STORE_PATH), vec![identity(&cache, PIPELINE_SCHEMA_EPOCH)?, campaign.clone()])?;
        let first = PipelineWriter::attach(&root, holder("first"))?;
        let contender = root.clone();
        let second = std::thread::spawn(move || PipelineWriter::attach(&contender, holder("second")).err())
            .join()
            .expect("the contender thread completes");
        assert_eq!(second, Some(PipelineRefusal::WriterLeaseHeld { holder: Some(holder("first")) }));
        let read = PipelineStore::open(&root)?;
        assert!(read.envelopes().iter().any(|entry| entry.key == campaign.key), "reads stay allowed while leased");
        drop(first);
        let second = PipelineWriter::attach(&root, holder("second"))?;
        assert_eq!(second.store_path(), root.join(PIPELINE_STORE_PATH));
        Ok(())
    }

    #[test]
    fn writer_lease_is_shared_across_worktrees_of_one_clone() -> Result<()> {
        let (temp, root) = campaign_repo(true, true)?;
        let worktree = temp.path().join("worktree");
        git_ok(&root, &["worktree", "add", "-q", "-b", "side", &worktree.to_string_lossy()])?;
        let _held = PipelineWriter::attach(&root, holder("main"))?;
        assert_eq!(
            PipelineWriter::attach(&worktree, holder("worktree")).err(),
            Some(PipelineRefusal::WriterLeaseHeld { holder: Some(holder("main")) })
        );
        Ok(())
    }

    #[test]
    fn writer_refuses_unmarked_binary_and_unignored_lock_and_wrong_branch() -> Result<()> {
        let (_unmarked_temp, unmarked) = campaign_repo(false, true)?;
        assert_eq!(
            PipelineWriter::attach(&unmarked, holder("t")).err(),
            Some(PipelineRefusal::StoreNotMarkedBinary { line: "/.epiphany/pipeline/pipeline.cc binary".into() })
        );
        let (_unignored_temp, unignored) = campaign_repo(true, false)?;
        assert_eq!(
            PipelineWriter::attach(&unignored, holder("t")).err(),
            Some(PipelineRefusal::LockNotIgnored { line: "/.epiphany/pipeline/*.lock".into() })
        );
        let (_temp, root) = campaign_repo(true, true)?;
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested)?;
        assert_eq!(
            PipelineWriter::attach(&nested, holder("t")).err(),
            Some(PipelineRefusal::NotRepoRoot { repo_root: nested.display().to_string() })
        );
        let writer = PipelineWriter::attach(&root, holder("t"))?;
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
        let lease = root.join(".git").join(WRITER_LEASE);
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
        eventually(|| {
            matches!(
                PipelineWriter::attach(&root, holder("probe")),
                Err(PipelineRefusal::WriterLeaseHeld { .. })
            )
        });
        child.0.kill()?;
        child.0.wait()?;
        eventually(|| PipelineWriter::attach(&root, holder("after")).is_ok());
        Ok(())
    }
}
