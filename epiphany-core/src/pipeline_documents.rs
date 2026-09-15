//! Eureka pipeline documents (cut map D1).
//!
//! Every kind is a plain `serde` + `JsonSchema` value inside a one-slot
//! `DatabaseEntry` wrapper, always prepared with `prepare_entry_named`, so the
//! stored payload is `[value]` and the value is the named map the published
//! schema describes. Keys are semantic: `pipeline_key` derives them from the
//! value, and the write validator refuses any envelope whose key differs.

use crate::pipeline_store::PipelineRefusal;
use anyhow::Result;
use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PIPELINE_SCHEMA_EPOCH: &str = "epiphany.pipeline.epoch.v1";

pub(crate) trait Bounded {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal>;
}

impl<T: Bounded> Bounded for Option<T> {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
        self.as_ref().map_or(Ok(()), |value| value.validate(field))
    }
}

fn bound(field: &str, limit: usize, actual: usize) -> Result<(), PipelineRefusal> {
    if actual > limit {
        return Err(PipelineRefusal::FieldBound {
            field: field.into(),
            limit: limit as u32,
            actual: actual as u32,
        });
    }
    Ok(())
}

/// List maximums only. Minimum counts are admission rules with their own
/// refusals (Cut 3b), so a bounds pass never pre-empts them.
fn list<T: Bounded>(field: &str, items: &[T], max: usize) -> Result<(), PipelineRefusal> {
    bound(field, max, items.len())?;
    for (index, item) in items.iter().enumerate() {
        item.validate(&format!("{field}[{index}]"))?;
    }
    Ok(())
}

fn format_error(field: &str, value: &str) -> PipelineRefusal {
    PipelineRefusal::InvalidFormat {
        field: field.into(),
        value: value.into(),
    }
}

fn hex(field: &str, value: &str, lengths: std::ops::RangeInclusive<usize>) -> Result<(), PipelineRefusal> {
    let lower_hex = value.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
    if !lengths.contains(&value.len()) || !lower_hex {
        return Err(format_error(field, value));
    }
    Ok(())
}

/// `<slug>` and `<local>` key segments: `[A-Za-z0-9._-]{1,64}`.
fn label<'a>(field: &str, value: &'a str) -> Result<&'a str, PipelineRefusal> {
    let valid = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if !valid || value.is_empty() || value.len() > 64 {
        return Err(format_error(field, value));
    }
    Ok(value)
}

fn org_repo(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    match value.split_once('/') {
        Some((org, repo)) if !org.is_empty() && !repo.is_empty() && !repo.contains('/') => Ok(()),
        _ => Err(format_error(field, value)),
    }
}

macro_rules! bounded_text {
    ($($(#[$doc:meta])* $name:ident = $limit:literal;)*) => {$(
        $(#[$doc])*
        #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(pub String);
        impl Bounded for $name {
            fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
                bound(field, $limit, self.0.len())
            }
        }
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.into())
            }
        }
    )*};
}

bounded_text! {
    /// Text of at most 200 UTF-8 bytes.
    Short = 200;
    /// Text of at most 1,000 UTF-8 bytes.
    Line = 1000;
    /// Text of at most 4,000 UTF-8 bytes; longer narrative is cited by `DocRef`.
    Para = 4000;
}

/// A git commit id: 7-40 lowercase hex characters.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Sha(pub String);

impl Bounded for Sha {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
        hex(field, &self.0, 7..=40)
    }
}

/// A calendar date, `YYYY-MM-DD`.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Date(pub String);

impl Bounded for Date {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
        match chrono::NaiveDate::parse_from_str(&self.0, "%Y-%m-%d") {
            Ok(_) if self.0.len() == 10 => Ok(()),
            _ => Err(format_error(field, &self.0)),
        }
    }
}

macro_rules! bounded_struct {
    ($($ty:ident { $($field:ident $([$max:literal])?),* $(,)? })*) => {$(
        impl Bounded for $ty {
            fn validate(&self, at: &str) -> Result<(), PipelineRefusal> {
                $(bounded_field!(self.$field, &format!("{at}.{}", stringify!($field)) $(, $max)?);)*
                Ok(())
            }
        }
    )*};
}

macro_rules! bounded_field {
    ($value:expr, $field:expr) => {
        Bounded::validate(&$value, $field)?
    };
    ($value:expr, $field:expr, $max:literal) => {
        list($field, &$value, $max)?
    };
}

macro_rules! value_types {
    ($($(#[$attr:meta])* pub struct $name:ident { $($field:ident: $ty:ty),* $(,)? })*) => {$(
        $(#[$attr])*
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        pub struct $name { $(pub $field: $ty),* }
    )*};
}

macro_rules! unit_enums {
    ($($name:ident { $($variant:ident),* $(,)? })*) => {$(
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
        pub enum $name { $($variant),* }
    )*};
}

unit_enums! {
    EvidenceKind { Command, Test, Mutation, Probe, SourceRead, Capture }
    Faculty { SelfFaculty, Imagination, Hands, Soul, MindSteward, Eyes, Operator }
    LandedNameKind { Module, Type, Function, Constant, Test, Schema, Binary, File }
    ClaimOutcome { Holds, Falsified, Unproven }
    FindingConfidence { Confirmed, Plausible }
    FindingSeverity { Blocker, High, Medium, Low }
}

value_types! {
    pub struct PipelineRef { kind: PipelineKind, id: Short }
    pub struct CodeLocation { path: Short, line: u32, end_line: Option<u32> }
    pub struct CommitRange { base: Sha, head: Sha }
    pub struct Evidence { kind: EvidenceKind, locator: Line, result: Line }
    /// A citation of a document in another repo's pipeline store (D6).
    pub struct ForeignRef { repo: Short, commit: Sha, kind: PipelineKind, id: Short, payload_sha256: String }
    pub struct DocRef { path: Short, start_line: u32, end_line: u32, commit: Sha }
    pub struct TargetInvariant { label: Short, statement: Line }
    pub struct QuestionOption { label: Short, text: Line }
    pub struct CutDelete { path: Short, lines: u32, note: Line }
    pub struct FileChange { location: CodeLocation, change: Line }
    pub struct AuthorityMap {
        owner: Line, inputs: Vec<Line>, outputs: Vec<Line>, derived_state: Vec<Line>,
        forbidden_writers: Vec<Line>, shared_paths: Vec<Line>, deletion_line: Line,
    }
    pub struct VerificationTest { name: Short, pins: Line }
    pub struct NegativeCheck { pattern: Short, scope: Line }
    pub struct CutVerification {
        builds: Vec<Line>, tests: Vec<VerificationTest>, negative: Vec<NegativeCheck>, operator: Vec<Line>,
    }
    pub struct SubtractionEstimate { lines_removed: u32, lines_added: u32, removed: Vec<Short>, added: Vec<Short> }
    pub struct ReportCommit { sha: Sha, subject: Line, builds: bool }
    pub struct MutationRecord { rule: Line, mutation: Line, failed_as_expected: bool }
    pub struct Deviation { what: Line, why: Line }
    pub struct StructuralDelta {
        lines_added: u32, lines_removed: u32,
        dependencies_added: Vec<Short>, dependencies_removed: Vec<Short>,
        formats_added: Vec<Short>, formats_removed: Vec<Short>,
        targets_added: Vec<Short>, targets_removed: Vec<Short>,
    }
    pub struct LandedName { name: Short, kind: LandedNameKind, location: CodeLocation }
    pub struct VerdictClaim { claim: Line, outcome: ClaimOutcome, evidence: Vec<Evidence>, findings: Vec<Short> }

    pub struct PipelineCampaign { slug: Short, title: Short, repos: Vec<Short>, working_branch: Short, target_doc: DocRef }
    pub struct PipelineTarget {
        campaign: Short, revision: u32, invariants: Vec<TargetInvariant>, not_in_scope: Vec<Line>,
        canonical_implementations: Vec<Line>, doc: DocRef,
    }
    pub struct PipelineQuestion {
        campaign: Short, label: Short, question: Para, options: Vec<QuestionOption>, recommended: Short,
        depends: Vec<Line>, raised_in: Option<PipelineRef>, asked_on: Date,
    }
    pub struct PipelineRuling {
        campaign: Short, label: Short, answers: Option<Short>, choice: Option<Short>, ruling: Para,
        operator_quote: Option<Para>, ruled_on: Date, precedents: Vec<ForeignRef>,
    }
    pub struct PipelineCutSpec {
        campaign: Short, cut: Short, revision: u32, title: Short, repo: Short, branch: Short, base: Sha,
        depends_on: Vec<Short>, first: Vec<Line>, deletes: Vec<CutDelete>, keeps_moves: Vec<Line>,
        adds: Vec<Line>, file_changes: Vec<FileChange>, authority_map: Option<AuthorityMap>,
        verification: CutVerification, subtraction_estimate: SubtractionEstimate,
        rulings: Vec<Short>, questions: Vec<Short>,
    }
    pub struct PipelineCutReport {
        campaign: Short, cut_spec: Short, attempt: Short, repo: Short, branch: Short,
        commits: Vec<ReportCommit>, range: CommitRange, verification: Vec<Evidence>,
        mutations: Vec<MutationRecord>, deviations: Vec<Deviation>, forks: Vec<Short>,
        structural_delta: StructuralDelta, landed_names: Vec<LandedName>, undone: Vec<Line>,
    }
    pub struct PipelineVerdict { campaign: Short, cut_report: Short, pass: Short, range: CommitRange, claims: Vec<VerdictClaim> }
    pub struct PipelineFinding {
        campaign: Short, verdict: Short, label: Short, range: CommitRange, confidence: FindingConfidence,
        severity: FindingSeverity, claim: Line, invariants: Vec<Short>, locations: Vec<CodeLocation>,
        failure_scenario: Para, evidence: Vec<Evidence>, precedents: Vec<ForeignRef>,
    }
    pub struct PipelineFollowUp {
        campaign: Short, label: Short, source: PipelineRef, repo: Short, locations: Vec<CodeLocation>,
        item: Line, why_it_can_wait: Line, owner: Short,
    }
    pub struct PipelineResolution { subject: PipelineRef, outcome: ResolutionOutcome, rationale: Para, resolved_on: Date }

    /// Who declared an admission. Attribution only; no rule trusts it (D4).
    pub struct PipelineProvenance { faculty: Faculty, agent: Short, session: Short, tool: Short }
    /// Display-only record of the process holding a repo's writer lease (D3).
    pub struct PipelineWriterHolder { pid: u32, host: Short, session: Short, attached_at: String }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ResolutionOutcome {
    Superseded { by: Short },
    Answered { by: Short },
    Fixed { by: Short },
    Deferred { to: Short },
    Recorded { reason: Line },
    Withdrawn { reason: Line },
}

impl Bounded for ResolutionOutcome {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
        match self {
            Self::Superseded { by } | Self::Answered { by } | Self::Fixed { by } => by.validate(field),
            Self::Deferred { to } => to.validate(field),
            Self::Recorded { reason } | Self::Withdrawn { reason } => reason.validate(field),
        }
    }
}

impl Bounded for ForeignRef {
    fn validate(&self, at: &str) -> Result<(), PipelineRefusal> {
        org_repo(&format!("{at}.repo"), &self.repo.0)?;
        hex(&format!("{at}.commit"), &self.commit.0, 40..=40)?;
        self.id.validate(&format!("{at}.id"))?;
        hex(&format!("{at}.payload_sha256"), &self.payload_sha256, 64..=64)
    }
}

impl Bounded for PipelineCampaign {
    fn validate(&self, at: &str) -> Result<(), PipelineRefusal> {
        self.slug.validate(&format!("{at}.slug"))?;
        self.title.validate(&format!("{at}.title"))?;
        list(&format!("{at}.repos"), &self.repos, 8)?;
        for (index, repo) in self.repos.iter().enumerate() {
            org_repo(&format!("{at}.repos[{index}]"), &repo.0)?;
        }
        self.working_branch.validate(&format!("{at}.working_branch"))?;
        self.target_doc.validate(&format!("{at}.target_doc"))
    }
}

bounded_struct! {
    PipelineRef { id }
    CodeLocation { path }
    CommitRange { base, head }
    Evidence { locator, result }
    DocRef { path, commit }
    TargetInvariant { label, statement }
    QuestionOption { label, text }
    CutDelete { path, note }
    FileChange { location, change }
    AuthorityMap { owner, inputs[16], outputs[16], derived_state[16], forbidden_writers[16], shared_paths[16], deletion_line }
    VerificationTest { name, pins }
    NegativeCheck { pattern, scope }
    CutVerification { builds[64], tests[64], negative[64], operator[64] }
    SubtractionEstimate { removed[64], added[64] }
    ReportCommit { sha, subject }
    MutationRecord { rule, mutation }
    Deviation { what, why }
    StructuralDelta {
        dependencies_added[32], dependencies_removed[32], formats_added[32], formats_removed[32],
        targets_added[32], targets_removed[32],
    }
    LandedName { name, location }
    VerdictClaim { claim, evidence[8], findings[16] }
    PipelineTarget { campaign, invariants[32], not_in_scope[32], canonical_implementations[16], doc }
    PipelineQuestion { campaign, label, question, options[8], recommended, depends[8], raised_in, asked_on }
    PipelineRuling { campaign, label, answers, choice, ruling, operator_quote, ruled_on, precedents[8] }
    PipelineCutSpec {
        campaign, cut, title, repo, branch, base, depends_on[8], first[16], deletes[64], keeps_moves[64],
        adds[64], file_changes[256], authority_map, verification, subtraction_estimate, rulings[32], questions[16],
    }
    PipelineCutReport {
        campaign, cut_spec, attempt, repo, branch, commits[128], range, verification[64], mutations[64],
        deviations[32], forks[8], structural_delta, landed_names[128], undone[32],
    }
    PipelineVerdict { campaign, cut_report, pass, range, claims[64] }
    PipelineFinding {
        campaign, verdict, label, range, claim, invariants[8], locations[16], failure_scenario,
        evidence[16], precedents[8],
    }
    PipelineFollowUp { campaign, label, source, repo, locations[16], item, why_it_can_wait, owner }
    PipelineResolution { subject, outcome, rationale, resolved_on }
    PipelineProvenance { agent, session, tool }
    PipelineWriterHolder { host, session }
}

macro_rules! pipeline_kinds {
    ($($variant:ident($value:ident) => $document:ident, $name:literal, $type_id:tt, $schema:tt;)*) => {
        $(
            #[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
            #[cultcache(type = $type_id, schema = $schema)]
            pub struct $document {
                #[cultcache(key = 0)]
                pub value: $value,
            }
        )*

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
        pub enum PipelineKind { $($variant),* }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum PipelineDocument { $($variant($value)),* }

        impl PipelineKind {
            pub const ALL: &'static [PipelineKind] = &[$(Self::$variant),*];

            /// The kind segment of a document key.
            pub fn name(self) -> &'static str {
                match self { $(Self::$variant => $name),* }
            }

            pub fn type_id(self) -> &'static str {
                match self { $(Self::$variant => <$document as DatabaseEntry>::TYPE),* }
            }

            #[cfg(test)]
            pub(crate) fn derived_schema(self) -> schemars::Schema {
                match self { $(Self::$variant => schemars::schema_for!($value)),* }
            }
        }

        impl PipelineDocument {
            pub fn kind(&self) -> PipelineKind {
                match self { $(Self::$variant(_) => PipelineKind::$variant),* }
            }

            /// Field bounds (D1), in UTF-8 bytes and list maximums.
            pub fn validate(&self) -> Result<(), PipelineRefusal> {
                match self { $(Self::$variant(value) => value.validate($name)),* }
            }

            #[cfg_attr(not(test), expect(dead_code, reason = "Cut 3b admission prepares writes through this"))]
            pub(crate) fn prepare(&self, cache: &CultCache) -> Result<CultCacheEnvelope> {
                let key = pipeline_key(self)?;
                Ok(match self {
                    $(Self::$variant(value) => {
                        cache.prepare_entry_named(key, &$document { value: value.clone() })?.0
                    })*
                })
            }

            pub(crate) fn decode(envelope: &CultCacheEnvelope) -> Result<Self, PipelineRefusal> {
                let invalid = |error: rmp_serde::decode::Error| format_error("payload", &error.to_string());
                $(if envelope.r#type == <$document as DatabaseEntry>::TYPE {
                    let document: $document = rmp_serde::from_slice(&envelope.payload).map_err(invalid)?;
                    return Ok(Self::$variant(document.value));
                })*
                Err(PipelineRefusal::ForeignStore { r#type: envelope.r#type.clone() })
            }
        }

        /// Registers every type a pipeline store may hold: the ten kinds, the
        /// store identity, admission provenance, and the commit receipt.
        pub(crate) fn register_pipeline_document_types(cache: &mut CultCache) -> Result<()> {
            $(cache.register_entry_type::<$document>()?;)*
            cache.register_entry_type::<EpiphanyPipelineIdentity>()?;
            cache.register_entry_type::<EpiphanyPipelineProvenance>()?;
            cache.register_entry_type::<crate::EpiphanyMindCommitReceipt>()?;
            Ok(())
        }
    };
}

pipeline_kinds! {
    Campaign(PipelineCampaign) => EpiphanyPipelineCampaignDocument, "campaign",
        "epiphany.pipeline.campaign.v1", "EpiphanyPipelineCampaignDocument";
    Target(PipelineTarget) => EpiphanyPipelineTargetDocument, "target",
        "epiphany.pipeline.target.v1", "EpiphanyPipelineTargetDocument";
    Question(PipelineQuestion) => EpiphanyPipelineQuestionDocument, "question",
        "epiphany.pipeline.question.v1", "EpiphanyPipelineQuestionDocument";
    Ruling(PipelineRuling) => EpiphanyPipelineRulingDocument, "ruling",
        "epiphany.pipeline.ruling.v1", "EpiphanyPipelineRulingDocument";
    CutSpec(PipelineCutSpec) => EpiphanyPipelineCutSpecDocument, "cut_spec",
        "epiphany.pipeline.cut_spec.v1", "EpiphanyPipelineCutSpecDocument";
    CutReport(PipelineCutReport) => EpiphanyPipelineCutReportDocument, "cut_report",
        "epiphany.pipeline.cut_report.v1", "EpiphanyPipelineCutReportDocument";
    Verdict(PipelineVerdict) => EpiphanyPipelineVerdictDocument, "verdict",
        "epiphany.pipeline.verdict.v1", "EpiphanyPipelineVerdictDocument";
    Finding(PipelineFinding) => EpiphanyPipelineFindingDocument, "finding",
        "epiphany.pipeline.finding.v1", "EpiphanyPipelineFindingDocument";
    FollowUp(PipelineFollowUp) => EpiphanyPipelineFollowUpDocument, "follow_up",
        "epiphany.pipeline.follow_up.v1", "EpiphanyPipelineFollowUpDocument";
    Resolution(PipelineResolution) => EpiphanyPipelineResolutionDocument, "resolution",
        "epiphany.pipeline.resolution.v1", "EpiphanyPipelineResolutionDocument";
}

/// The store's schema identity, keyed by its epoch string (D2).
#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.pipeline.identity.v1", schema = "EpiphanyPipelineIdentity")]
pub struct EpiphanyPipelineIdentity {
    #[cultcache(key = 0)]
    pub schema_epoch: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.pipeline.provenance.v1", schema = "EpiphanyPipelineProvenance")]
pub struct EpiphanyPipelineProvenance {
    #[cultcache(key = 0)]
    pub value: PipelineProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.pipeline.writer_holder.v1", schema = "EpiphanyPipelineWriterHolder")]
pub struct EpiphanyPipelineWriterHolder {
    #[cultcache(key = 0)]
    pub value: PipelineWriterHolder,
}

/// The local segment of a parent id `<campaign>:<kind>:<local>` in `campaign`.
fn parent_local<'a>(
    field: &str,
    id: &'a str,
    campaign: &str,
    kind: PipelineKind,
) -> Result<&'a str, PipelineRefusal> {
    match id.splitn(3, ':').collect::<Vec<_>>()[..] {
        [owner, segment, local] if owner == campaign && segment == kind.name() => Ok(local),
        _ => Err(format_error(field, id)),
    }
}

/// The cut label inside a parent local `cut-<label>.<marker><N>`.
fn parent_cut<'a>(field: &str, local: &'a str, marker: char) -> Result<&'a str, PipelineRefusal> {
    local
        .strip_prefix("cut-")
        .and_then(|rest| rest.rsplit_once('.'))
        .filter(|(cut, suffix)| !cut.is_empty() && suffix.starts_with(marker))
        .map(|(cut, _)| cut)
        .ok_or_else(|| format_error(field, local))
}

/// Derives a document's identity key (D1, "Keys: identity, not convenience").
pub fn pipeline_key(document: &PipelineDocument) -> Result<String, PipelineRefusal> {
    use PipelineDocument as D;
    let (campaign, local) = match document {
        D::Campaign(value) => return Ok(label("campaign.slug", &value.slug.0)?.to_string()),
        D::Resolution(value) => return Ok(format!("resolution:{}", value.subject.id.0)),
        D::Target(value) => (&value.campaign, format!("r{}", value.revision)),
        D::Question(value) => (&value.campaign, value.label.0.clone()),
        D::Ruling(value) => (&value.campaign, value.label.0.clone()),
        D::FollowUp(value) => (&value.campaign, value.label.0.clone()),
        D::CutSpec(value) => (&value.campaign, format!("cut-{}.r{}", value.cut.0, value.revision)),
        D::CutReport(value) => {
            let spec = parent_local("cut_report.cut_spec", &value.cut_spec.0, &value.campaign.0, PipelineKind::CutSpec)?;
            let cut = parent_cut("cut_report.cut_spec", spec, 'r')?;
            (&value.campaign, format!("cut-{cut}.h{}", value.attempt.0))
        }
        D::Verdict(value) => {
            let report = parent_local("verdict.cut_report", &value.cut_report.0, &value.campaign.0, PipelineKind::CutReport)?;
            let cut = parent_cut("verdict.cut_report", report, 'h')?;
            (&value.campaign, format!("cut-{cut}.s{}", value.pass.0))
        }
        D::Finding(value) => {
            let verdict = parent_local("finding.verdict", &value.verdict.0, &value.campaign.0, PipelineKind::Verdict)?;
            (&value.campaign, format!("{verdict}.{}", value.label.0))
        }
    };
    let kind = document.kind().name();
    label(&format!("{kind}.campaign"), &campaign.0)?;
    label(&format!("{kind}.key"), &local)?;
    Ok(format!("{}:{kind}:{local}", campaign.0))
}

/// The pipeline commit profile's write validator: bounds, then key
/// recomputation. Per-kind admission rules live in Cut 3b.
pub(crate) fn validate_pipeline_write_envelope(envelope: &CultCacheEnvelope) -> Result<()> {
    if envelope.r#type == EpiphanyPipelineIdentity::TYPE {
        let identity: EpiphanyPipelineIdentity = rmp_serde::from_slice(&envelope.payload)?;
        if identity.schema_epoch != PIPELINE_SCHEMA_EPOCH || envelope.key != identity.schema_epoch {
            return Err(PipelineRefusal::ForeignEpoch {
                found: identity.schema_epoch,
                expected: PIPELINE_SCHEMA_EPOCH.into(),
            }
            .into());
        }
        return Ok(());
    }
    let document = PipelineDocument::decode(envelope)?;
    document.validate()?;
    let expected = pipeline_key(&document)?;
    if envelope.key != expected {
        return Err(PipelineRefusal::InvalidIdentity {
            kind: document.kind(),
            key: envelope.key.clone(),
            expected,
        }
        .into());
    }
    Ok(())
}
