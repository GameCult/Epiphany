//! Eureka pipeline documents (cut map D1).
//!
//! Every kind is a plain `serde` + `JsonSchema` value inside a one-slot
//! `DatabaseEntry` wrapper, always prepared with `prepare_entry_named`, so the
//! stored payload is `[value]` and the value is the named map the published
//! schema describes. Keys are semantic: `pipeline_key` derives them from the
//! value, and the write validator refuses any envelope whose key differs.
//!
//! **Validation follows the type.** `value_types!` is the single field list: it
//! emits the struct and its `Bounded` impl from the same tokens, so a field
//! cannot be declared without being validated. Format rules live in the field's
//! type (`Label`, `Slug`, `OrgRepo`, `Sha`, `Sha256Hex`, `Date`), never in a
//! hand-written impl that a later field can slip past. A `Vec` field must carry
//! a maximum, because `Vec<T>` has no `Bounded` impl of its own.
//!
//! The wrappers are crate-private: outside code reaches the store through the
//! admission path, never by registering or preparing a pipeline type itself.

use anyhow::Result;
use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PIPELINE_SCHEMA_EPOCH: &str = "epiphany.pipeline.epoch.v1";

/// Typed refusals of the pipeline documents. Bounds, formats and key identity
/// are the document half (D2); `ForeignEpoch` and `ForeignStore` are raised by
/// the decode and envelope-validation paths that still live here, and D2 hands
/// them to the organ when admission moves there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineRefusal {
    FieldBound { field: String, limit: u32, actual: u32 },
    InvalidFormat { field: String, value: String },
    InvalidIdentity { kind: PipelineKind, key: String, expected: String },
    ForeignEpoch { found: String, expected: String },
    ForeignStore { r#type: String },
}

impl std::fmt::Display for PipelineRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "pipeline refusal: {self:?}")
    }
}

impl std::error::Error for PipelineRefusal {}

pub(crate) trait Bounded {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal>;
}

impl<T: Bounded> Bounded for Option<T> {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
        self.as_ref().map_or(Ok(()), |value| value.validate(field))
    }
}

macro_rules! unbounded_scalars {
    ($($ty:ty),* $(,)?) => {$(
        impl Bounded for $ty {
            fn validate(&self, _field: &str) -> Result<(), PipelineRefusal> {
                Ok(())
            }
        }
    )*};
}

unbounded_scalars!(u32, u64, bool);

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

/// A key label: `[A-Za-z0-9_-]{1,64}`. Dots are excluded so that a composed key
/// segments unambiguously; see `parent_cut` and the finding key.
fn label_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    let valid = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
    if !valid || value.is_empty() || value.len() > 64 {
        return Err(format_error(field, value));
    }
    Ok(())
}

/// A dotted key segment: label parts joined by `.`, at most 64 bytes. Every
/// part must be a label, so `..`, a leading or trailing dot, and an empty part
/// are all refused.
fn dotted_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    if value.is_empty() || value.len() > 64 {
        return Err(format_error(field, value));
    }
    for part in value.split('.') {
        label_text(field, part)?;
    }
    Ok(())
}

fn org_repo_text(field: &str, value: &str) -> Result<(), PipelineRefusal> {
    match value.split_once('/') {
        Some((org, repo))
            if !org.is_empty() && !repo.is_empty() && !repo.contains('/') && value.len() <= 200 =>
        {
            Ok(())
        }
        _ => Err(format_error(field, value)),
    }
}

macro_rules! bounded_text {
    ($($(#[$doc:meta])* $name:ident = $limit:literal, $check:expr;)*) => {$(
        $(#[$doc])*
        #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        #[schemars(extend("maxLength" = $limit))]
        pub struct $name(pub String);
        impl Bounded for $name {
            fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
                let check: fn(&str, &str) -> Result<(), PipelineRefusal> = $check;
                check(field, &self.0)
            }
        }
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.into())
            }
        }
    )*};
}

fn within(limit: usize) -> impl Fn(&str, &str) -> Result<(), PipelineRefusal> {
    move |field: &str, value: &str| bound(field, limit, value.len())
}

bounded_text! {
    /// Text of at most 200 UTF-8 bytes.
    Short = 200, |field, value| within(200)(field, value);
    /// Text of at most 1,000 UTF-8 bytes.
    Line = 1000, |field, value| within(1000)(field, value);
    /// Text of at most 4,000 UTF-8 bytes; longer narrative is cited by `DocRef`.
    Para = 4000, |field, value| within(4000)(field, value);
    /// A key label: `[A-Za-z0-9_-]{1,64}`.
    Label = 64, label_text;
    /// A campaign slug or key segment: label parts joined by `.`.
    Slug = 64, dotted_text;
    /// A GitHub repository as `Org/Repo`.
    OrgRepo = 200, org_repo_text;
    /// A git commit id: 7-40 lowercase hex characters.
    Sha = 40, |field, value| hex(field, value, 7..=40);
    /// A full git commit id: exactly 40 lowercase hex characters.
    FullSha = 40, |field, value| hex(field, value, 40..=40);
    /// A SHA-256 digest: exactly 64 lowercase hex characters.
    Sha256Hex = 64, |field, value| hex(field, value, 64..=64);
}

/// A calendar date, `YYYY-MM-DD`.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("maxLength" = 10))]
pub struct Date(pub String);

impl Bounded for Date {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
        match chrono::NaiveDate::parse_from_str(&self.0, "%Y-%m-%d") {
            Ok(_) if self.0.len() == 10 => Ok(()),
            _ => Err(format_error(field, &self.0)),
        }
    }
}

macro_rules! bounded_field {
    ($value:expr, $field:expr) => {
        Bounded::validate(&$value, $field)?
    };
    ($value:expr, $field:expr, $max:literal) => {
        list($field, &$value, $max)?
    };
}

/// The single field list: one invocation emits the value struct and its
/// `Bounded` impl, so a field can never be left out of validation.
macro_rules! value_types {
    ($($(#[$attr:meta])* pub struct $name:ident { $($field:ident: $ty:ty $([$max:literal])?),* $(,)? })*) => {$(
        $(#[$attr])*
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        pub struct $name {
            $(
                $(#[schemars(extend("maxItems" = $max))])?
                pub $field: $ty
            ),*
        }

        impl Bounded for $name {
            fn validate(&self, at: &str) -> Result<(), PipelineRefusal> {
                $(bounded_field!(self.$field, &format!("{at}.{}", stringify!($field)) $(, $max)?);)*
                Ok(())
            }
        }
    )*};
}

macro_rules! unit_enums {
    ($($name:ident { $($variant:ident),* $(,)? })*) => {$(
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
        pub enum $name { $($variant),* }
        impl Bounded for $name {
            fn validate(&self, _field: &str) -> Result<(), PipelineRefusal> {
                Ok(())
            }
        }
    )*};
}

unit_enums! {
    EvidenceKind { Command, Test, Mutation, Probe, SourceRead, Capture }
    Faculty { SelfFaculty, Imagination, Hands, Soul, MindSteward, Eyes, Operator }
    ClaimOutcome { Holds, Falsified, Unproven }
    FindingConfidence { Confirmed, Plausible }
    FindingSeverity { Blocker, High, Medium, Low }
}

value_types! {
    pub struct CodeLocation { path: Short, line: u32, end_line: Option<u32> }
    pub struct CommitRange { base: Sha, head: Sha }
    pub struct Evidence { kind: EvidenceKind, locator: Line, result: Line }
    /// A citation of a document in another repo's pipeline store (D6).
    pub struct ForeignRef { repo: OrgRepo, commit: FullSha, kind: PipelineKind, id: Short, payload_sha256: Sha256Hex }
    pub struct DocRef { path: Short, start_line: u32, end_line: u32, commit: Sha }
    pub struct TargetInvariant { label: Label, statement: Line }
    pub struct QuestionOption { label: Label, text: Line }
    pub struct CutDelete { path: Short, lines: u32, note: Line }
    pub struct FileChange { location: CodeLocation, change: Line }
    pub struct AuthorityMap {
        owner: Line, inputs: Vec<Line>[16], outputs: Vec<Line>[16], derived_state: Vec<Line>[16],
        forbidden_writers: Vec<Line>[16], shared_paths: Vec<Line>[16], deletion_line: Line,
    }
    pub struct VerificationTest { name: Short, pins: Line }
    pub struct NegativeCheck { pattern: Short, scope: Line }
    pub struct CutVerification {
        builds: Vec<Line>[64], tests: Vec<VerificationTest>[64],
        negative: Vec<NegativeCheck>[64], operator: Vec<Line>[64],
    }
    // D1 tables no maximum for these lists, so they take the shared list
    // default of 64 rather than a number invented for this field alone.
    pub struct SubtractionEstimate {
        lines_removed: u32, lines_added: u32, removed: Vec<Short>[64], added: Vec<Short>[64],
    }
    pub struct ReportCommit { sha: Sha, subject: Line, builds: bool }
    pub struct MutationRecord { rule: Line, mutation: Line, failed_as_expected: bool }
    pub struct Deviation { what: Line, why: Line }
    // As for `SubtractionEstimate`: D1 tables no maximum, so these lists take
    // the shared list default.
    pub struct StructuralDelta {
        lines_added: u32, lines_removed: u32,
        dependencies_added: Vec<Short>[64], dependencies_removed: Vec<Short>[64],
        formats_added: Vec<Short>[64], formats_removed: Vec<Short>[64],
        targets_added: Vec<Short>[64], targets_removed: Vec<Short>[64],
    }
    /// A name the cut landed, and where it lives.
    pub struct LandedName { name: Short, path: Short }
    pub struct VerdictClaim { claim: Line, outcome: ClaimOutcome, evidence: Vec<Evidence>[8], findings: Vec<Short>[16] }

    pub struct PipelineCampaign { slug: Slug, title: Short, repos: Vec<OrgRepo>[8], working_branch: Short, target_doc: DocRef }
    pub struct PipelineTarget {
        campaign: Slug, revision: u32, invariants: Vec<TargetInvariant>[32], not_in_scope: Vec<Line>[32],
        canonical_implementations: Vec<Line>[16], doc: DocRef,
    }
    pub struct PipelineQuestion {
        campaign: Slug, label: Label, question: Para, options: Vec<QuestionOption>[8], recommended: Label,
        depends: Vec<Line>[8], raised_in: Option<PipelineRef>, asked_on: Date,
    }
    pub struct PipelineRuling {
        campaign: Slug, label: Label, answers: Option<Short>, choice: Option<Label>, ruling: Para,
        operator_quote: Option<Para>, ruled_on: Date, precedents: Vec<ForeignRef>[8],
    }
    pub struct PipelineCutSpec {
        campaign: Slug, cut: Label, revision: u32, title: Short, repo: OrgRepo, branch: Short, base: Sha,
        depends_on: Vec<Short>[8], first: Vec<Line>[16], deletes: Vec<CutDelete>[64], keeps_moves: Vec<Line>[64],
        adds: Vec<Line>[64], file_changes: Vec<FileChange>[256], authority_map: Option<AuthorityMap>,
        verification: CutVerification, subtraction_estimate: SubtractionEstimate,
        rulings: Vec<Short>[32], questions: Vec<Short>[16],
    }
    pub struct PipelineCutReport {
        campaign: Slug, cut_spec: Short, attempt: u32, repo: OrgRepo, branch: Short,
        commits: Vec<ReportCommit>[128], range: CommitRange, verification: Vec<Evidence>[64],
        mutations: Vec<MutationRecord>[64], deviations: Vec<Deviation>[32], forks: Vec<Short>[8],
        structural_delta: StructuralDelta, landed_names: Vec<LandedName>[128], undone: Vec<Line>[32],
    }
    pub struct PipelineVerdict { campaign: Slug, cut_report: Short, pass: u32, range: CommitRange, claims: Vec<VerdictClaim>[64] }
    pub struct PipelineFinding {
        campaign: Slug, verdict: Short, label: Label, range: CommitRange, confidence: FindingConfidence,
        severity: FindingSeverity, claim: Line, invariants: Vec<Label>[8], locations: Vec<CodeLocation>[16],
        failure_scenario: Para, evidence: Vec<Evidence>[16], precedents: Vec<ForeignRef>[8],
    }
    pub struct PipelineFollowUp {
        campaign: Slug, label: Label, source: PipelineRef, repo: OrgRepo, locations: Vec<CodeLocation>[16],
        item: Line, why_it_can_wait: Line, owner: Short,
    }
    pub struct PipelineResolution { subject: PipelineRef, outcome: ResolutionOutcome, rationale: Para, resolved_on: Date }

    /// Who declared an admission. Attribution only; no rule trusts it (D4).
    pub struct PipelineProvenance { faculty: Faculty, agent: Short, session: Short, tool: Short }
}

/// A reference to another document. The id is parsed as a full pipeline id of
/// the declared `kind`, so the kind and the id's kind segment cannot disagree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PipelineRef {
    pub kind: PipelineKind,
    pub id: Short,
}

impl Bounded for PipelineRef {
    fn validate(&self, at: &str) -> Result<(), PipelineRefusal> {
        self.id.validate(&format!("{at}.id"))?;
        pipeline_id(&format!("{at}.id"), &self.id.0, self.kind).map(|_| ())
    }
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

macro_rules! pipeline_kinds {
    ($($variant:ident($value:ident) => $document:ident, $name:literal, $type_id:tt, $schema:tt;)*) => {
        $(
            #[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
            #[cultcache(type = $type_id, schema = $schema)]
            pub(crate) struct $document {
                #[cultcache(key = 0)]
                pub(crate) value: $value,
            }
        )*

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
        pub enum PipelineKind { $($variant),* }

        impl Bounded for PipelineKind {
            fn validate(&self, _field: &str) -> Result<(), PipelineRefusal> {
                Ok(())
            }
        }

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

            /// Field bounds and formats (D1), in UTF-8 bytes and list maximums.
            pub fn validate(&self) -> Result<(), PipelineRefusal> {
                match self { $(Self::$variant(value) => value.validate($name)),* }
            }

            #[cfg_attr(not(test), expect(dead_code, reason = "the organ's admission prepares writes through this"))]
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
        #[cfg_attr(not(test), expect(dead_code, reason = "the organ registers its mind's types through this"))]
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
pub(crate) struct EpiphanyPipelineIdentity {
    #[cultcache(key = 0)]
    pub(crate) schema_epoch: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.pipeline.provenance.v1", schema = "EpiphanyPipelineProvenance")]
pub(crate) struct EpiphanyPipelineProvenance {
    #[cultcache(key = 0)]
    pub(crate) value: PipelineProvenance,
}

/// Parses a full document id. `<campaign>:<kind>:<local>` for every kind but
/// `campaign`, which is keyed by its slug alone. The segment count is exact,
/// the kind segment must match `kind`, and both the campaign and the local
/// segment are validated, so `..`, an empty part, spaces and trailing junk are
/// all refused.
fn pipeline_id<'a>(
    field: &str,
    id: &'a str,
    kind: PipelineKind,
) -> Result<(&'a str, &'a str), PipelineRefusal> {
    if kind == PipelineKind::Campaign {
        dotted_text(field, id)?;
        return Ok((id, id));
    }
    let mut segments = id.split(':');
    let (Some(campaign), Some(name), Some(local), None) = (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
    ) else {
        return Err(format_error(field, id));
    };
    if name != kind.name() {
        return Err(format_error(field, id));
    }
    dotted_text(field, campaign)?;
    dotted_text(field, local)?;
    Ok((campaign, local))
}

/// The local segment of a parent id in `campaign`, of the expected kind.
fn parent_local<'a>(
    field: &str,
    id: &'a str,
    campaign: &str,
    kind: PipelineKind,
) -> Result<&'a str, PipelineRefusal> {
    let (owner, local) = pipeline_id(field, id, kind)?;
    if owner != campaign {
        return Err(format_error(field, id));
    }
    Ok(local)
}

/// The cut label inside a parent local `cut-<label>.<marker><N>`. The label
/// carries no dot and `<N>` is a non-empty run of digits, so the composed key
/// segments one way only.
fn parent_cut<'a>(field: &str, local: &'a str, marker: char) -> Result<&'a str, PipelineRefusal> {
    let invalid = || format_error(field, local);
    let (cut, suffix) = local
        .strip_prefix("cut-")
        .and_then(|rest| rest.rsplit_once('.'))
        .ok_or_else(invalid)?;
    label_text(field, cut)?;
    let digits = suffix.strip_prefix(marker).ok_or_else(invalid)?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    Ok(cut)
}

/// Derives a document's identity key (D1, "Keys: identity, not convenience").
pub fn pipeline_key(document: &PipelineDocument) -> Result<String, PipelineRefusal> {
    use PipelineDocument as D;
    let (campaign, local) = match document {
        D::Campaign(value) => {
            dotted_text("campaign.slug", &value.slug.0)?;
            return Ok(value.slug.0.clone());
        }
        D::Resolution(value) => {
            let field = "resolution.subject.id";
            pipeline_id(field, &value.subject.id.0, value.subject.kind)?;
            return Ok(format!("resolution:{}", value.subject.id.0));
        }
        D::Target(value) => (&value.campaign, format!("r{}", value.revision)),
        D::Question(value) => (&value.campaign, value.label.0.clone()),
        D::Ruling(value) => (&value.campaign, value.label.0.clone()),
        D::FollowUp(value) => (&value.campaign, value.label.0.clone()),
        D::CutSpec(value) => (&value.campaign, format!("cut-{}.r{}", value.cut.0, value.revision)),
        D::CutReport(value) => {
            let spec = parent_local("cut_report.cut_spec", &value.cut_spec.0, &value.campaign.0, PipelineKind::CutSpec)?;
            let cut = parent_cut("cut_report.cut_spec", spec, 'r')?;
            (&value.campaign, format!("cut-{cut}.h{}", value.attempt))
        }
        D::Verdict(value) => {
            let report = parent_local("verdict.cut_report", &value.cut_report.0, &value.campaign.0, PipelineKind::CutReport)?;
            let cut = parent_cut("verdict.cut_report", report, 'h')?;
            (&value.campaign, format!("cut-{cut}.s{}", value.pass))
        }
        D::Finding(value) => {
            let verdict = parent_local("finding.verdict", &value.verdict.0, &value.campaign.0, PipelineKind::Verdict)?;
            parent_cut("finding.verdict", verdict, 's')?;
            label_text("finding.label", &value.label.0)?;
            (&value.campaign, format!("{verdict}.{}", value.label.0))
        }
    };
    let kind = document.kind().name();
    dotted_text(&format!("{kind}.campaign"), &campaign.0)?;
    dotted_text(&format!("{kind}.key"), &local)?;
    Ok(format!("{}:{kind}:{local}", campaign.0))
}

/// Bounds, formats, then key recomputation, for one envelope. Per-kind
/// admission rules belong to the organ's admission path.
#[cfg_attr(not(test), expect(dead_code, reason = "the organ's admission validates every write through this"))]
fn validate_pipeline_write_envelope(envelope: &CultCacheEnvelope) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::Path;

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
        CodeLocation { path: s("epiphany-core/src/pipeline_documents.rs"), line: 1, end_line: Some(9) }
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
                campaign: slug(CAMPAIGN), label: l("Q1"), question: "Who owns the state?".into(),
                options: vec![
                    QuestionOption { label: l("A"), text: "an instance".into() },
                    QuestionOption { label: l("B"), text: "a repo".into() },
                ],
                recommended: l("A"), depends: vec!["Cut 3a".into()],
                raised_in: Some(PipelineRef { kind: PipelineKind::CutSpec, id: id("cut_spec", "cut-3a.r1") }),
                asked_on: date(),
            }), format!("{CAMPAIGN}:question:Q1")),
            (D::Ruling(PipelineRuling {
                campaign: slug(CAMPAIGN), label: l("R8"), answers: Some(id("question", "Q1")), choice: Some(l("A")),
                ruling: "An instance owns its mind.".into(), operator_quote: Some("all recommendations, go ahead".into()),
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
                keeps_moves: vec!["commit owner".into()], adds: vec!["pipeline_documents.rs".into()],
                file_changes: vec![FileChange { location: location(), change: "add".into() }],
                authority_map: Some(AuthorityMap {
                    owner: "core".into(), inputs: vec!["typed documents".into()], outputs: vec!["envelopes".into()],
                    derived_state: vec!["derived keys".into()], forbidden_writers: vec!["MCP".into()],
                    shared_paths: vec!["admission".into()], deletion_line: "n/a".into(),
                }),
                verification: CutVerification {
                    builds: vec!["cargo check".into()],
                    tests: vec![VerificationTest { name: s("keys"), pins: "one derived key".into() }],
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
                commits: vec![ReportCommit { sha: sha(), subject: "Add the pipeline documents".into(), builds: true }],
                range: range(), verification: vec![evidence()],
                mutations: vec![MutationRecord { rule: "key derivation".into(), mutation: "drop the marker rule".into(), failed_as_expected: true }],
                deviations: vec![Deviation { what: "names".into(), why: "glob exports".into() }],
                forks: vec![id("question", "Q1")],
                structural_delta: StructuralDelta {
                    lines_added: 900, lines_removed: 0, dependencies_added: vec![s("schemars")],
                    dependencies_removed: vec![], formats_added: vec![s("epiphany.pipeline.*.v1")],
                    formats_removed: vec![], targets_added: vec![], targets_removed: vec![],
                },
                landed_names: vec![LandedName { name: s("PipelineDocument"), path: s("epiphany-core/src/pipeline_documents.rs") }],
                undone: vec!["admission".into()],
            }), format!("{CAMPAIGN}:cut_report:cut-3a.h1")),
            (D::Verdict(PipelineVerdict {
                campaign: slug(CAMPAIGN), cut_report: id("cut_report", "cut-3a.h1"), pass: 2, range: range(),
                claims: vec![VerdictClaim {
                    claim: "A composed key has one source.".into(), outcome: ClaimOutcome::Falsified,
                    evidence: vec![evidence()], findings: vec![id("finding", "cut-3a.s2.F4")],
                }],
            }), format!("{CAMPAIGN}:verdict:cut-3a.s2")),
            (D::Finding(PipelineFinding {
                campaign: slug(CAMPAIGN), verdict: id("verdict", "cut-3a.s2"), label: l("F4"), range: range(),
                confidence: FindingConfidence::Confirmed, severity: FindingSeverity::High,
                claim: "A dotted label composes two keys.".into(), invariants: vec![l("mind-admits")],
                locations: vec![location()], failure_scenario: "Two documents claim one key.".into(),
                evidence: vec![evidence()], precedents: vec![],
            }), format!("{CAMPAIGN}:finding:cut-3a.s2.F4")),
            (D::FollowUp(PipelineFollowUp {
                campaign: slug(CAMPAIGN), label: l("FU-4"),
                source: PipelineRef { kind: PipelineKind::Finding, id: id("finding", "cut-3a.s2.F4") },
                repo: repo(), locations: vec![location()], item: "Per-kind admission rules.".into(),
                why_it_can_wait: "The organ owns admission.".into(), owner: s("Hands"),
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
            let error = validate_pipeline_write_envelope(&envelope).unwrap_err();
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
        // A well-formed number with no marker at all. Only the marker rule
        // refuses this one: the digits rule is satisfied either way.
        refused(&format!("{CAMPAIGN}:cut_spec:cut-3a.1"));
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
}
