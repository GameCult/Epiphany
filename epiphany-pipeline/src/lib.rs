//! Eureka pipeline documents: the typed shape of a Eureka campaign's state.
//!
//! This is a leaf type library, and deliberately nothing else. It owns document
//! shape, field bounds, formats, key derivation and the JSON schemas published
//! under `schemas/cultnet`; it owns no storage, no admission, no process and no
//! network. That is the whole reason it is a package: the memory organ depends
//! on these types without depending on the harness that used to hold them.
//!
//! Every kind is a plain `serde` + `JsonSchema` value inside a one-slot
//! `DatabaseEntry` wrapper, always prepared with `prepare_entry_named`, so the
//! stored payload is `[value]` and the value is the named map the published
//! schema describes. Keys are semantic: `pipeline_key` derives them from the
//! value, and the write validator refuses any envelope whose key differs.
//!
//! Every key is `<root>:<kind>:<local>`, three segments for every kind with
//! none excused: the root is a `Slug`, the kind is the literal kind name, and
//! the local is `Label`s joined by `.`, so no local part carries a dot and a
//! reader recovers the parts by splitting. A root's local is the constant
//! `self`; a resolution's local is its subject's kind and local.
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

use cultcache_rs::DatabaseEntry;
// Envelopes are a test-only shape here until the organ prepares, decodes and
// validates them; only the type ids survive into the live path. `anyhow` and
// `rmp-serde` come with them, which is why both are dev-dependencies: the live
// library's errors are all `PipelineRefusal`.
#[cfg(test)]
use anyhow::Result;
#[cfg(test)]
use cultcache_rs::{CultCache, CultCacheEnvelope};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Typed refusals of the pipeline documents. Bounds, formats and key identity
/// are the document half (D2); `ForeignStore` is raised by the decode path that
/// still lives here, and D2 hands it to the organ when admission moves there.
/// Identity, the schema epoch and the `ForeignEpoch` refusal are the organ's
/// (D2), and Cut 8 writes them there against a store a test can construct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PipelineRefusal {
    FieldBound { field: String, limit: u32, actual: u32 },
    InvalidFormat { field: String, value: String },
    InvalidIdentity { kind: PipelineKind, key: String, expected: String },
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
/// segments unambiguously; see `local`.
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
    ClaimOutcome { Holds, Falsified, Unproven }
    FindingConfidence { Confirmed, Plausible }
    FindingSeverity { Blocker, High, Medium, Low }
    RulingAuthority { Operator, Standing, Defaulted }
    FindingOrigin { Introduced, PreExisting }
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
    pub struct ReportCommit { sha: Sha, subject: Line, builds: bool }
    /// One mutation a report ran: a key-safe label a verdict claim can name,
    /// the exact edit, and the tree it was applied to.
    pub struct MutationRecord { label: Label, rule: Line, location: CodeLocation, before: Line, after: Line, commit: Sha, failed_as_expected: bool }
    pub struct Deviation { what: Line, why: Line }
    // D1 tables no maximum for these lists, so they take the shared list
    // default of 64 rather than a number invented for this field alone.
    pub struct StructuralDelta {
        lines_added: u32, lines_removed: u32,
        dependencies_added: Vec<Short>[64], dependencies_removed: Vec<Short>[64],
        formats_added: Vec<Short>[64], formats_removed: Vec<Short>[64],
        targets_added: Vec<Short>[64], targets_removed: Vec<Short>[64],
    }
    /// A name the cut landed, and where it lives.
    pub struct LandedName { name: Short, path: Short }
    /// A promise a report makes about what it landed. Not a document and not
    /// resolvable: it is identified by its report's id and its label, as a
    /// `TargetInvariant` is by its target's. Soul measures every one (ruling
    /// A), and the rule that every one is measured is admission's.
    pub struct Promise { label: Label, text: Line }
    /// The promise this claim measured, and the report's mutations it ran. An
    /// `Unproven` claim with a promise is one Soul could not reach.
    pub struct VerdictClaim {
        claim: Line, outcome: ClaimOutcome, evidence: Vec<Evidence>[8], findings: Vec<Short>[16],
        promise: Option<Label>, mutations: Vec<Label>[8],
    }

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
        operator_quote: Option<Para>, ruled_on: Date, precedents: Vec<ForeignRef>[8], authority: RulingAuthority,
    }
    pub struct PipelineCutSpec {
        campaign: Slug, cut: Label, revision: u32, title: Short, repo: OrgRepo, branch: Short, base: Sha,
        depends_on: Vec<Short>[8], first: Vec<Line>[16], deletes: Vec<CutDelete>[64], keeps_moves: Vec<Line>[64],
        adds: Vec<Line>[64], file_changes: Vec<FileChange>[256], authority_map: Option<AuthorityMap>,
        verification: CutVerification, estimate: StructuralDelta,
        rulings: Vec<Short>[32], questions: Vec<Short>[16],
    }
    pub struct PipelineCutReport {
        campaign: Slug, cut_spec: Short, attempt: u32, repo: OrgRepo, branch: Short,
        commits: Vec<ReportCommit>[128], range: CommitRange, verification: Vec<Evidence>[64],
        mutations: Vec<MutationRecord>[64], deviations: Vec<Deviation>[32], forks: Vec<Short>[8],
        structural_delta: StructuralDelta, landed_names: Vec<LandedName>[128], undone: Vec<Line>[32],
        promises: Vec<Promise>[64],
    }
    pub struct PipelineVerdict { campaign: Slug, cut_report: Short, pass: u32, range: CommitRange, claims: Vec<VerdictClaim>[64] }
    pub struct PipelineFinding {
        campaign: Slug, verdict: Short, label: Label, range: CommitRange, confidence: FindingConfidence,
        severity: FindingSeverity, claim: Line, invariants: Vec<Label>[8], locations: Vec<CodeLocation>[16],
        failure_scenario: Para, evidence: Vec<Evidence>[16], precedents: Vec<ForeignRef>[8], origin: FindingOrigin,
    }
    pub struct PipelineFollowUp {
        campaign: Slug, label: Label, source: PipelineRef, repo: OrgRepo, locations: Vec<CodeLocation>[16],
        item: Line, why_it_can_wait: Line, owner: Short,
    }
    pub struct PipelineResolution { subject: PipelineRef, outcome: ResolutionOutcome, rationale: Para, resolved_on: Date }

    /// A mind's identity document. A store is canonical to exactly one
    /// instance, and this says which; identity lives in the state, not in a
    /// path.
    pub struct PipelineInstance { instance: Slug, display_name: Short, created_at: Date, host: Short }
    /// Stewardship over a repo, as an assignment recorded in a mind. One
    /// instance may steward several repos, so the repo is part of the key.
    pub struct PipelineStewardship { instance: Slug, repo: OrgRepo, assigned_on: Date, note: Line }
    /// A reassignment of stewardship, recorded in both minds. `documents` names
    /// what travels with it.
    pub struct PipelineHandOff {
        from_instance: Slug, to_instance: Slug, repo: OrgRepo, documents: Vec<Short>[256],
        reason: Para, handed_on: Date,
    }
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

/// How a subject was resolved. Every referent is a parsed `PipelineRef`, so a
/// resolution names its records by ids of the kinds they declare, validated
/// where every other referent is; a `Fixed` commit is a `Sha` whose referent
/// is outside the document set (ruling B). Whether a named document exists,
/// and how many may supersede one subject, are admission's rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ResolutionOutcome {
    /// Overturned, in whole or in part, by later records. The list is bounded
    /// here because the enum is outside `value_types!`, the only place a list
    /// maximum is emitted automatically.
    Superseded {
        #[schemars(extend("maxItems" = 8))]
        by: Vec<PipelineRef>,
    },
    Answered { by: PipelineRef },
    Fixed { commit: Sha, by: Option<PipelineRef> },
    Deferred { to: PipelineRef },
    Recorded { reason: Line },
    Withdrawn { reason: Line },
}

impl Bounded for ResolutionOutcome {
    fn validate(&self, field: &str) -> Result<(), PipelineRefusal> {
        match self {
            Self::Superseded { by } => list(&format!("{field}.by"), by, 8),
            Self::Answered { by } => by.validate(&format!("{field}.by")),
            Self::Fixed { commit, by } => {
                commit.validate(&format!("{field}.commit"))?;
                by.validate(&format!("{field}.by"))
            }
            Self::Deferred { to } => to.validate(&format!("{field}.to")),
            Self::Recorded { reason } | Self::Withdrawn { reason } => reason.validate(&format!("{field}.reason")),
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

            /// Test scaffolding until the organ prepares writes (Cut 8). Every
            /// caller is a test, and the only cache carrying these types is the
            /// `cfg(test)` registrar beside it, so this is `cfg(test)` rather
            /// than a live path wearing a dead-code waiver.
            #[cfg(test)]
            pub(crate) fn prepare(&self, cache: &CultCache) -> Result<CultCacheEnvelope> {
                let key = pipeline_key(self)?;
                Ok(match self {
                    $(Self::$variant(value) => {
                        cache.prepare_entry_named(key, &$document { value: value.clone() })?.0
                    })*
                })
            }

            /// Test-only with its only caller, the write validator below, until
            /// the organ decodes admitted envelopes (Cut 8).
            #[cfg(test)]
            pub(crate) fn decode(envelope: &CultCacheEnvelope) -> Result<Self, PipelineRefusal> {
                let invalid = |error: rmp_serde::decode::Error| format_error("payload", &error.to_string());
                $(if envelope.r#type == <$document as DatabaseEntry>::TYPE {
                    let document: $document = rmp_serde::from_slice(&envelope.payload).map_err(invalid)?;
                    return Ok(Self::$variant(document.value));
                })*
                Err(PipelineRefusal::ForeignStore { r#type: envelope.r#type.clone() })
            }
        }

        /// Registers every type the document tests put in a cache: each kind
        /// and one foreign document. Test scaffolding until the organ registers
        /// its mind's types, which is why it is `cfg(test)` rather than a live
        /// path wearing a dead-code waiver.
        #[cfg(test)]
        pub(crate) fn register_pipeline_document_types(cache: &mut CultCache) -> Result<()> {
            $(cache.register_entry_type::<$document>()?;)*
            cache.register_entry_type::<ForeignDocument>()?;
            Ok(())
        }
    };
}

/// A document that is not a pipeline document, for the decode refusal and the
/// spine-registry rules to be pinned against something real. The organ's Mind
/// commit receipt is the case that matters: a mind's store legitimately holds
/// one beside pipeline documents, and it is still not a document this library
/// may decode. That type belongs to the harness, not here, so the rule is
/// pinned against a stand-in carrying its type id.
#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.mind_commit_receipt.v1", schema = "ForeignDocument")]
pub(crate) struct ForeignDocument {
    #[cultcache(key = 0)]
    pub(crate) marker: Short,
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
    Instance(PipelineInstance) => EpiphanyPipelineInstanceDocument, "instance",
        "epiphany.pipeline.instance.v1", "EpiphanyPipelineInstanceDocument";
    Stewardship(PipelineStewardship) => EpiphanyPipelineStewardshipDocument, "stewardship",
        "epiphany.pipeline.stewardship.v1", "EpiphanyPipelineStewardshipDocument";
    HandOff(PipelineHandOff) => EpiphanyPipelineHandOffDocument, "hand_off",
        "epiphany.pipeline.hand_off.v1", "EpiphanyPipelineHandOffDocument";
}

/// Parses a full document id: the grammar read backwards. A key and an id are
/// the same string, so this accepts exactly what `pipeline_key` derives:
/// `<root>:<kind>:<local>`, three segments for every kind. The kind segment
/// must equal `kind`'s name, the root is a `Slug`, and the local is `Label`s
/// joined by `.` and bounded whole, so `..`, an empty part, spaces and trailing
/// junk are all refused. Returns the root and the local; nothing is inferred
/// from either, and no kind is read any other way.
fn pipeline_id<'a>(
    field: &str,
    id: &'a str,
    kind: PipelineKind,
) -> Result<(&'a str, &'a str), PipelineRefusal> {
    let mut segments = id.split(':');
    let (Some(root), Some(name), Some(local), None) = (
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
    dotted_text(field, root)?;
    dotted_text(field, local)?;
    Ok((root, local))
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

/// The cut label inside a parent local `cut-<label>.<marker><N>`: exactly two
/// parts, the label a `Label` and `<N>` a non-empty run of digits.
fn parent_cut<'a>(field: &str, local: &'a str, marker: char) -> Result<&'a str, PipelineRefusal> {
    let invalid = || format_error(field, local);
    let mut parts = local.split('.');
    let (Some(head), Some(suffix), None) = (parts.next(), parts.next(), parts.next()) else {
        return Err(invalid());
    };
    let cut = head.strip_prefix("cut-").ok_or_else(invalid)?;
    label_text(field, cut)?;
    let digits = suffix.strip_prefix(marker).ok_or_else(invalid)?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    Ok(cut)
}

/// A `Slug` or an `OrgRepo` entering a local, escaped to one label. `/` is not
/// a `Label` byte, so it cannot survive into a key, and neither may `.`: no
/// local part carries the separator, or the boundary before it would have two
/// readings. A dotted slug and a dotted repo name are both ordinary, so both
/// bytes are escaped rather than refused. The escape has to be injective or two
/// values claim one key: replacing `/` with `_` alone is not, since
/// `GameCult_Epiphany/thing` and `GameCult/Epiphany_thing` both give
/// `GameCult_Epiphany_thing`. So `_` is escaped as well, and every code is two
/// bytes starting with `_`: `_` becomes `__`, `/` becomes `_-`, and `.` becomes
/// `_d`. Every other byte is passed through and is never `_`, so a reader going
/// left to right takes each `_` together with the byte after it and never has a
/// choice to make; the encoding is therefore reversible, and distinct values
/// give distinct segments. The caller validates the value as its own type first
/// and checks the result against the local's rules, which is where an over-long
/// or otherwise unlabelled value is refused.
fn key_segment(value: &str) -> String {
    value.replace('_', "__").replace('/', "_-").replace('.', "_d")
}

/// The local of a root. A campaign's or an instance's identity is entirely its
/// root segment, so its local carries none; a reader never consults it.
const ROOT_LOCAL: &str = "self";

/// The bound on a composed local, whole, in UTF-8 bytes. It is the only depth
/// limit on a chain of resolutions, and the depth test derives its expectation
/// from this name rather than restating the number.
const LOCAL_MAX: usize = 64;

/// Composes and validates a local: every part is a `Label`, and the join is
/// bounded whole. Both rules live here because this is the only way a local is
/// built; `pipeline_key` has no other path to a key string. Parts are a slice,
/// not a builder, so an arm's arity is written at its call site.
fn local(field: &str, parts: &[&str]) -> Result<String, PipelineRefusal> {
    for part in parts {
        label_text(field, part)?;
    }
    let joined = parts.join(".");
    if joined.len() > LOCAL_MAX {
        return Err(format_error(field, &joined));
    }
    Ok(joined)
}

/// Derives a document's identity key (D1, "Keys: identity, not convenience").
/// Thirteen arms, one exit: every arm names its root and composes its local
/// through `local`, and the key is formatted here and nowhere else.
pub fn pipeline_key(document: &PipelineDocument) -> Result<String, PipelineRefusal> {
    use PipelineDocument as D;
    let kind = document.kind().name();
    let key_field = format!("{kind}.key");
    let campaign_field = format!("{kind}.campaign");
    let (root_field, root, composed) = match document {
        D::Campaign(value) => ("campaign.slug", value.slug.0.as_str(), local(&key_field, &[ROOT_LOCAL])?),
        D::Instance(value) => ("instance.instance", value.instance.0.as_str(), local(&key_field, &[ROOT_LOCAL])?),
        // A resolution is keyed inside its subject's root, with the subject's
        // kind and local as its own local. A resolution's own key is an
        // ordinary id, so it composes as a subject like any other; each nesting
        // prepends `resolution.` (11 bytes), so a chain `n` deep over a
        // depth-one local of `L` bytes composes `11 * (n - 1) + L` bytes
        // against `LOCAL_MAX` in `local`, which is the only depth limit and
        // needs no guard.
        D::Resolution(value) => {
            let field = "resolution.subject.id";
            let (subject_root, subject_local) = pipeline_id(field, &value.subject.id.0, value.subject.kind)?;
            let parts = std::iter::once(value.subject.kind.name()).chain(subject_local.split('.')).collect::<Vec<_>>();
            (field, subject_root, local(&key_field, &parts)?)
        }
        // Stewardship and hand-off hang off an instance rather than a campaign.
        // The root is the whole difference; the key shape is the same.
        D::Stewardship(value) => {
            org_repo_text("stewardship.repo", &value.repo.0)?;
            ("stewardship.instance", value.instance.0.as_str(), local(&key_field, &[&key_segment(&value.repo.0)])?)
        }
        D::HandOff(value) => {
            dotted_text("hand_off.to_instance", &value.to_instance.0)?;
            org_repo_text("hand_off.repo", &value.repo.0)?;
            value.handed_on.validate("hand_off.handed_on")?;
            (
                "hand_off.from_instance",
                value.from_instance.0.as_str(),
                local(&key_field, &[&key_segment(&value.to_instance.0), &key_segment(&value.repo.0), &value.handed_on.0])?,
            )
        }
        D::Target(value) => (campaign_field.as_str(), value.campaign.0.as_str(), local(&key_field, &[&format!("r{}", value.revision)])?),
        D::Question(value) => (campaign_field.as_str(), value.campaign.0.as_str(), local(&key_field, &[&value.label.0])?),
        D::Ruling(value) => (campaign_field.as_str(), value.campaign.0.as_str(), local(&key_field, &[&value.label.0])?),
        D::FollowUp(value) => (campaign_field.as_str(), value.campaign.0.as_str(), local(&key_field, &[&value.label.0])?),
        D::CutSpec(value) => (
            campaign_field.as_str(),
            value.campaign.0.as_str(),
            local(&key_field, &[&format!("cut-{}", value.cut.0), &format!("r{}", value.revision)])?,
        ),
        D::CutReport(value) => {
            let spec = parent_local("cut_report.cut_spec", &value.cut_spec.0, &value.campaign.0, PipelineKind::CutSpec)?;
            let cut = parent_cut("cut_report.cut_spec", spec, 'r')?;
            (
                campaign_field.as_str(),
                value.campaign.0.as_str(),
                local(&key_field, &[&format!("cut-{cut}"), &format!("h{}", value.attempt)])?,
            )
        }
        D::Verdict(value) => {
            let report = parent_local("verdict.cut_report", &value.cut_report.0, &value.campaign.0, PipelineKind::CutReport)?;
            let cut = parent_cut("verdict.cut_report", report, 'h')?;
            (
                campaign_field.as_str(),
                value.campaign.0.as_str(),
                local(&key_field, &[&format!("cut-{cut}"), &format!("s{}", value.pass)])?,
            )
        }
        D::Finding(value) => {
            let verdict = parent_local("finding.verdict", &value.verdict.0, &value.campaign.0, PipelineKind::Verdict)?;
            parent_cut("finding.verdict", verdict, 's')?;
            let parts = verdict.split('.').chain(std::iter::once(value.label.0.as_str())).collect::<Vec<_>>();
            (campaign_field.as_str(), value.campaign.0.as_str(), local(&key_field, &parts)?)
        }
    };
    dotted_text(root_field, root)?;
    Ok(format!("{root}:{kind}:{composed}"))
}

/// Bounds, formats, then key recomputation, for one envelope. Per-kind
/// admission rules belong to the organ's admission path. Its only caller is a
/// test until the organ validates writes through it (Cut 8), so it is
/// `cfg(test)` rather than a live path wearing a dead-code waiver.
#[cfg(test)]
fn validate_pipeline_write_envelope(envelope: &CultCacheEnvelope) -> Result<()> {
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
    const INSTANCE: &str = "yggdrasil";

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
        CodeLocation { path: s("epiphany-pipeline/src/lib.rs"), line: 1, end_line: Some(9) }
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
            }), format!("{CAMPAIGN}:campaign:self")),
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
                authority: RulingAuthority::Operator,
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
                // Cut 3a's real numbers, and in this order: the tripwire test
                // reads them back, so a transposition is a failure, not a typo.
                estimate: StructuralDelta {
                    lines_added: 900, lines_removed: 0,
                    dependencies_added: vec![s("schemars")], dependencies_removed: vec![],
                    formats_added: vec![s("epiphany.pipeline.*.v1")], formats_removed: vec![],
                    targets_added: vec![], targets_removed: vec![],
                },
                rulings: vec![id("ruling", "R8")], questions: vec![id("question", "Q1")],
            }), format!("{CAMPAIGN}:cut_spec:cut-3a.r1")),
            (D::CutReport(PipelineCutReport {
                campaign: slug(CAMPAIGN), cut_spec: id("cut_spec", "cut-3a.r1"), attempt: 1, repo: repo(), branch: branch(),
                commits: vec![ReportCommit { sha: sha(), subject: "Add the pipeline documents".into(), builds: true }],
                range: range(), verification: vec![evidence()],
                mutations: vec![MutationRecord {
                    // `.into()` on both, not `l()` and `sha()`: the type-level
                    // mutations (M19, M20) widen these fields to `Short`, and
                    // a sample spelled with the narrow constructors would stop
                    // compiling instead of letting the forgery through.
                    label: "M1".into(), rule: "key derivation".into(), location: location(),
                    before: "parent_cut(field, spec, 'r')?".into(), after: "spec".into(), commit: "5f98228d".into(),
                    failed_as_expected: true,
                }],
                deviations: vec![Deviation { what: "names".into(), why: "glob exports".into() }],
                forks: vec![id("question", "Q1")],
                structural_delta: StructuralDelta {
                    lines_added: 900, lines_removed: 0, dependencies_added: vec![s("schemars")],
                    dependencies_removed: vec![], formats_added: vec![s("epiphany.pipeline.*.v1")],
                    formats_removed: vec![], targets_added: vec![], targets_removed: vec![],
                },
                landed_names: vec![LandedName { name: s("PipelineDocument"), path: s("epiphany-pipeline/src/lib.rs") }],
                undone: vec!["admission".into()],
                promises: vec![Promise { label: l("P1"), text: "One derived key per document.".into() }],
            }), format!("{CAMPAIGN}:cut_report:cut-3a.h1")),
            (D::Verdict(PipelineVerdict {
                campaign: slug(CAMPAIGN), cut_report: id("cut_report", "cut-3a.h1"), pass: 2, range: range(),
                claims: vec![VerdictClaim {
                    claim: "A composed key has one source.".into(), outcome: ClaimOutcome::Falsified,
                    evidence: vec![evidence()], findings: vec![id("finding", "cut-3a.s2.F4")],
                    promise: Some(l("P1")), mutations: vec![l("M1")],
                }],
            }), format!("{CAMPAIGN}:verdict:cut-3a.s2")),
            (D::Finding(PipelineFinding {
                campaign: slug(CAMPAIGN), verdict: id("verdict", "cut-3a.s2"), label: l("F4"), range: range(),
                confidence: FindingConfidence::Confirmed, severity: FindingSeverity::High,
                claim: "A dotted label composes two keys.".into(), invariants: vec![l("mind-admits")],
                locations: vec![location()], failure_scenario: "Two documents claim one key.".into(),
                evidence: vec![evidence()], precedents: vec![], origin: FindingOrigin::Introduced,
            }), format!("{CAMPAIGN}:finding:cut-3a.s2.F4")),
            (D::FollowUp(PipelineFollowUp {
                campaign: slug(CAMPAIGN), label: l("FU-4"),
                source: PipelineRef { kind: PipelineKind::Finding, id: id("finding", "cut-3a.s2.F4") },
                repo: repo(), locations: vec![location()], item: "Per-kind admission rules.".into(),
                why_it_can_wait: "The organ owns admission.".into(), owner: s("Hands"),
            }), format!("{CAMPAIGN}:follow_up:FU-4")),
            (D::Resolution(PipelineResolution {
                subject: PipelineRef { kind: PipelineKind::Question, id: id("question", "Q1") },
                outcome: ResolutionOutcome::Answered { by: PipelineRef { kind: PipelineKind::Ruling, id: id("ruling", "R8") } },
                rationale: "Ruled A.".into(), resolved_on: date(),
            }), format!("{CAMPAIGN}:resolution:question.Q1")),
            // The mind's own three kinds. They are appended rather than
            // inserted because the helpers below index this list by position.
            (D::Instance(PipelineInstance {
                instance: slug(INSTANCE), display_name: s("Yggdrasil mind"), created_at: date(),
                host: s("yggdrasil"),
            }), format!("{INSTANCE}:instance:self")),
            (D::Stewardship(PipelineStewardship {
                instance: slug(INSTANCE), repo: repo(), assigned_on: date(),
                note: "The Eureka campaign repo.".into(),
            }), format!("{INSTANCE}:stewardship:GameCult_-Epiphany")),
            (D::HandOff(PipelineHandOff {
                from_instance: slug(INSTANCE), to_instance: slug("thought-cage"), repo: repo(),
                documents: vec![s(CAMPAIGN), id("ruling", "R8")],
                reason: "The workstation mind takes the campaign.".into(), handed_on: date(),
            }), format!("{INSTANCE}:hand_off:thought-cage.GameCult_-Epiphany.{}", date().0)),
        ]
    }

    fn instance_sample() -> PipelineInstance {
        let PipelineDocument::Instance(instance) = samples().remove(10).0 else { unreachable!() };
        instance
    }

    fn stewardship_sample() -> PipelineStewardship {
        let PipelineDocument::Stewardship(stewardship) = samples().remove(11).0 else { unreachable!() };
        stewardship
    }

    fn hand_off_sample() -> PipelineHandOff {
        let PipelineDocument::HandOff(hand_off) = samples().remove(12).0 else { unreachable!() };
        hand_off
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

    /// Soul F2: a decode is type-matched in both directions. The round-trip
    /// test above pins the positive match; this pins the refusal, so an
    /// envelope belonging to another kind can never be decoded as whichever
    /// pipeline kind happens to parse its payload. The commit receipt is the
    /// sharp case: a mind's store may legitimately hold one, and it is still
    /// not a document; `ForeignDocument` stands in for it here.
    #[test]
    fn decode_refuses_an_envelope_of_a_foreign_type() -> Result<()> {
        let cache = schema_cache()?;
        let foreign = <ForeignDocument as DatabaseEntry>::TYPE;
        assert!(!foreign.starts_with("epiphany.pipeline."), "{foreign} is a foreign type id");
        let mut envelope = campaign_sample().prepare(&cache)?;
        envelope.r#type = foreign.into();
        assert_eq!(
            PipelineDocument::decode(&envelope),
            Err(PipelineRefusal::ForeignStore { r#type: foreign.into() })
        );
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

    /// F2, and Soul's second pass: a composed local segments one way only. The
    /// rule -- no local part carries the separator -- is pinned on the composer
    /// in `no_local_part_carries_the_separator`; the pairs below are the ones
    /// Soul measured through the documents, and each is a pair only because the
    /// rule holds.
    #[test]
    fn composed_keys_cannot_collide() {
        let key = |document: PipelineDocument| pipeline_key(&document);

        // Soul's hand-off pair. A dotted slug is ordinary on the receiver side
        // and a dotted repo name on the repo side, so both are escaped: left
        // unescaped these two both key to
        // `yggdrasil:hand_off:thought-cage.GameCult.Epiphany_-thing.2026-09-15`.
        let handed = |to: &str, repo: &str| {
            let mut hand_off = hand_off_sample();
            hand_off.to_instance = slug(to);
            hand_off.repo = OrgRepo(repo.into());
            key(PipelineDocument::HandOff(hand_off))
        };
        let dotted_receiver = handed("thought-cage.GameCult", "Epiphany/thing");
        let dotted_repo = handed("thought-cage", "GameCult.Epiphany/thing");
        assert_eq!(
            dotted_receiver,
            Ok(format!("{INSTANCE}:hand_off:thought-cage_dGameCult.Epiphany_-thing.2026-09-15"))
        );
        assert_eq!(
            dotted_repo,
            Ok(format!("{INSTANCE}:hand_off:thought-cage.GameCult_dEpiphany_-thing.2026-09-15"))
        );
        assert_ne!(dotted_receiver, dotted_repo, "a dotted repo and a dotted receiver cannot claim one key");

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
            Ok(format!("{CAMPAIGN}:resolution:ruling.R8"))
        );
    }

    /// Soul G2: a resolution is named by the key it has, and a `PipelineRef`
    /// of kind `Resolution` accepts exactly that id. `PipelineRef` takes any
    /// kind, so this is reachable; a follow-up sourced from a resolution is the
    /// live path. The grammar admits `<campaign>:resolution:R8` too: it is
    /// well-formed, and no resolution derives it, but whether a document with an
    /// id exists is admission's rule, not the grammar's, so the reader does not
    /// refuse it and this test does not ask it to.
    #[test]
    fn a_resolution_is_named_by_the_key_it_has() {
        let sourced = |id: &str| {
            let PipelineDocument::FollowUp(mut follow_up) = samples().remove(8).0 else { unreachable!() };
            follow_up.source = PipelineRef { kind: PipelineKind::Resolution, id: Short(id.into()) };
            PipelineDocument::FollowUp(follow_up).validate()
        };
        let key = pipeline_key(&PipelineDocument::Resolution(resolution_sample())).expect("the sample keys");
        assert_eq!(key, format!("{CAMPAIGN}:resolution:question.Q1"));
        assert_eq!(sourced(&key), Ok(()), "a resolution is named by the key it has");
        assert_eq!(sourced(&format!("{CAMPAIGN}:resolution:R8")), Ok(()), "well-formed grammar is not the reader's to refuse");

        for refused in [
            // The prefix shape the writer used to emit. No resolution carries it.
            format!("resolution:{CAMPAIGN}:question:Q1"),
            format!("{CAMPAIGN}:resolution:{CAMPAIGN}:question:Q1"),
            // No local, a subject local with an empty part, trailing junk, and a
            // kind segment that is not `resolution`.
            format!("{CAMPAIGN}:resolution:"),
            format!("{CAMPAIGN}:resolution:question..Q1"),
            format!("{CAMPAIGN}:resolution:question.Q1:junk"),
            format!("{CAMPAIGN}:question:Q1"),
        ] {
            assert!(
                matches!(sourced(&refused), Err(PipelineRefusal::InvalidFormat { .. })),
                "{refused:?} is not an id any resolution has"
            );
        }

        // A root subject reads back too: a resolution of a campaign carries the
        // campaign's kind and its `self` local.
        let mut of_campaign = resolution_sample();
        of_campaign.subject = PipelineRef { kind: PipelineKind::Campaign, id: Short(format!("{CAMPAIGN}:campaign:self")) };
        let root_key = pipeline_key(&PipelineDocument::Resolution(of_campaign)).expect("a root subject keys");
        assert_eq!(root_key, format!("{CAMPAIGN}:resolution:campaign.self"));
        assert_eq!(sourced(&root_key), Ok(()));
    }

    /// The three kinds a mind is keyed by. The samples above already round-trip
    /// every kind; this pins the shapes D2 gives these three specifically: an
    /// instance is a root keyed by its own slug, and the other two hang off an
    /// instance rather than a campaign, so neither borrows the campaign tail.
    #[test]
    fn instance_stewardship_and_hand_off_round_trip() -> Result<()> {
        let cache = schema_cache()?;
        let instance = PipelineDocument::Instance(instance_sample());
        assert_eq!(pipeline_key(&instance), Ok(format!("{INSTANCE}:instance:self")));
        for document in [
            instance,
            PipelineDocument::Stewardship(stewardship_sample()),
            PipelineDocument::HandOff(hand_off_sample()),
        ] {
            document.validate()?;
            let envelope = document.prepare(&cache)?;
            assert_eq!(PipelineDocument::decode(&envelope)?, document);
            let key = pipeline_key(&document)?;
            assert!(
                !key.starts_with(&format!("{CAMPAIGN}:")),
                "{:?} is keyed by its instance, not by a campaign: {key}",
                document.kind()
            );
            validate_pipeline_write_envelope(&envelope)?;
        }
        // A bounded list is still bounded: `documents` carries a maximum, as
        // every `Vec` field must.
        let mut wide = hand_off_sample();
        wide.documents = vec![s("x"); 257];
        assert_eq!(
            PipelineDocument::HandOff(wide).validate(),
            Err(PipelineRefusal::FieldBound { field: "hand_off.documents".into(), limit: 256, actual: 257 })
        );
        Ok(())
    }

    /// Soul F3: a key and an id are the same string, so every kind's key reads
    /// back as an id of that kind. The kinds a mind is keyed by were added to
    /// the key writer and left out of the id reader, and nothing noticed,
    /// because the key tests assert strings and never read one back: an
    /// instance keys to its slug and then fails to parse as an instance id, so
    /// no `PipelineRef` and no resolution could ever name one. Every kind now
    /// reads back with no kind excused, a resolution included, and a root kind
    /// read as the other root kind is refused: the reader checks the kind
    /// segment for the roots exactly as for every other kind.
    #[test]
    fn keys_read_back_as_ids_of_their_kind() -> Result<()> {
        for (document, expected) in samples() {
            let kind = document.kind();
            let key = pipeline_key(&document)?;
            assert_eq!(key, expected, "{kind:?} keys to its sample's key");
            assert_eq!(
                pipeline_id("read_back", &key, kind).map(|_| ()),
                Ok(()),
                "{kind:?} key {key} does not read back as an id of its kind"
            );
        }

        let campaign_key = pipeline_key(&campaign_sample())?;
        let instance_key = pipeline_key(&PipelineDocument::Instance(instance_sample()))?;
        assert_eq!(
            pipeline_id("read_back", &instance_key, PipelineKind::Instance),
            Ok((INSTANCE, ROOT_LOCAL)),
            "an instance key reads back as an instance"
        );
        assert_eq!(
            pipeline_id("read_back", &instance_key, PipelineKind::Campaign),
            Err(format_error("read_back", &instance_key)),
            "an instance key is not a campaign id"
        );
        assert_eq!(
            pipeline_id("read_back", &campaign_key, PipelineKind::Instance),
            Err(format_error("read_back", &campaign_key)),
            "a campaign key is not an instance id"
        );

        // Soul X1 and X11: the reader's root and local are each `dotted_text`
        // whole, so a trailing dot on either is refused by the reader itself,
        // not only by the writer that never composes one. A leading dot and
        // an empty part are the same rule on the local.
        for malformed in ["c.:target:x", "c:target:x.", "c:target:.x", "c:target:a..b"] {
            let read = pipeline_id("read_back", malformed, PipelineKind::Target);
            assert!(
                matches!(&read, Err(PipelineRefusal::InvalidFormat { field, .. }) if field == "read_back"),
                "{malformed:?} is not an id of any kind, got {read:?}"
            );
        }

        // The instance root's own key segment is validated, not merely bounded.
        // Nothing else stands between a `Slug` and a key: a space would compose
        // a store key carrying a byte no `Label` may hold.
        let mut spaced = instance_sample();
        spaced.instance = Slug("thought cage".into());
        assert_eq!(
            pipeline_key(&PipelineDocument::Instance(spaced)),
            Err(PipelineRefusal::InvalidFormat {
                field: "instance.instance".into(),
                value: "thought cage".into(),
            }),
            "an instance slug is validated where the key is composed"
        );
        Ok(())
    }

    /// The repo is one key segment, so its slash is escaped. Left unescaped it
    /// would both add a segment the reader cannot tell from a real one and put a
    /// byte in the key that no `Label` may carry; escaped to a bare `_` it would
    /// let two repos claim one key, which the colliding pair below pins.
    #[test]
    fn stewardship_key_escapes_the_repo_slash() {
        let stewardship = stewardship_sample();
        assert_eq!(stewardship.repo, OrgRepo("GameCult/Epiphany".into()));
        let key = pipeline_key(&PipelineDocument::Stewardship(stewardship.clone()));
        assert_eq!(key, Ok(format!("{INSTANCE}:stewardship:GameCult_-Epiphany")));
        assert!(!key.unwrap().contains('/'), "no key segment carries a slash");

        // The escape is not a cosmetic substitution, and the pair that proves it
        // is a pair: under `/` -> `_` alone both of these key to
        // `GameCult_Epiphany_thing`, and one of the two documents is lost.
        let keyed = |repo: &str| {
            let mut value = stewardship.clone();
            value.repo = OrgRepo(repo.into());
            pipeline_key(&PipelineDocument::Stewardship(value))
        };
        let underscored_org = keyed("GameCult_Epiphany/thing");
        let underscored_repo = keyed("GameCult/Epiphany_thing");
        assert_eq!(
            underscored_org,
            Ok(format!("{INSTANCE}:stewardship:GameCult__Epiphany_-thing"))
        );
        assert_eq!(
            underscored_repo,
            Ok(format!("{INSTANCE}:stewardship:GameCult_-Epiphany__thing"))
        );
        assert_ne!(underscored_org, underscored_repo, "two repos cannot claim one key");

        let mut no_org = stewardship.clone();
        no_org.repo = OrgRepo("Epiphany".into());
        assert_eq!(
            pipeline_key(&PipelineDocument::Stewardship(no_org)),
            Err(PipelineRefusal::InvalidFormat { field: "stewardship.repo".into(), value: "Epiphany".into() })
        );

        // An escaped repo is still bound by the key's segment rules.
        let mut long = stewardship;
        long.repo = OrgRepo(format!("GameCult/{}", "a".repeat(60)));
        assert!(
            matches!(
                pipeline_key(&PipelineDocument::Stewardship(long)),
                Err(PipelineRefusal::InvalidFormat { field, .. }) if field == "stewardship.key"
            ),
            "a repo segment longer than a key segment is refused"
        );
    }

    /// A hand-off is recorded in both minds, so both instances are in the key:
    /// the sender owns the first segment and the receiver the local. Changing
    /// either one moves the document.
    #[test]
    fn hand_off_names_both_instances() {
        let key = |hand_off: PipelineHandOff| pipeline_key(&PipelineDocument::HandOff(hand_off));
        let base = key(hand_off_sample()).expect("the sample keys");
        assert_eq!(base, format!("{INSTANCE}:hand_off:thought-cage.GameCult_-Epiphany.2026-09-15"));

        let mut other_sender = hand_off_sample();
        other_sender.from_instance = slug("thought-cage");
        assert_ne!(key(other_sender), Ok(base.clone()), "the sender is in the key");

        let mut other_receiver = hand_off_sample();
        other_receiver.to_instance = slug("mimir");
        assert_ne!(key(other_receiver), Ok(base.clone()), "the receiver is in the key");

        // Two hand-offs of the same repo between the same pair on different
        // days are different documents.
        let mut later = hand_off_sample();
        later.handed_on = Date("2026-09-16".into());
        assert_ne!(key(later), Ok(base), "the date is in the key");

        // A dotted receiver is a `Slug` entering a local, so it is escaped to
        // one label rather than widening the local by a part.
        let mut dotted_receiver = hand_off_sample();
        dotted_receiver.to_instance = slug("thought-cage.GameCult");
        assert_eq!(
            key(dotted_receiver),
            Ok(format!("{INSTANCE}:hand_off:thought-cage_dGameCult.GameCult_-Epiphany.2026-09-15"))
        );

        // The escape's `_` half is not only for repos: a receiver `a_db` and a
        // receiver `a.b` would both key to `a_db` if the slug's own `_` passed
        // through raw, so the slug is escaped as `a__db` and the two differ.
        let mut underscored_receiver = hand_off_sample();
        underscored_receiver.to_instance = slug("a_db");
        let mut dotted_twin = hand_off_sample();
        dotted_twin.to_instance = slug("a.b");
        let underscored_receiver = key(underscored_receiver).expect("an underscored receiver keys");
        assert!(underscored_receiver.contains("a__db"), "{underscored_receiver}: the slug's underscore is escaped");
        assert_ne!(key(dotted_twin), Ok(underscored_receiver), "`a_db` and `a.b` are two receivers");

        // Both instance slugs are validated as key segments, not merely bounded.
        let mut bad_receiver = hand_off_sample();
        bad_receiver.to_instance = Slug("thought cage".into());
        assert_eq!(
            key(bad_receiver),
            Err(PipelineRefusal::InvalidFormat {
                field: "hand_off.to_instance".into(),
                value: "thought cage".into(),
            })
        );
        let mut bad_date = hand_off_sample();
        bad_date.handed_on = Date("2026-9-15".into());
        assert!(
            matches!(key(bad_date), Err(PipelineRefusal::InvalidFormat { field, .. }) if field == "hand_off.handed_on"),
            "a malformed date does not compose a key"
        );
    }

    /// A resolution whose subject is the given document, keyed.
    fn resolution_of(kind: PipelineKind, id: &str) -> Result<String, PipelineRefusal> {
        let mut resolution = resolution_sample();
        resolution.subject = PipelineRef { kind, id: Short(id.into()) };
        pipeline_key(&PipelineDocument::Resolution(resolution))
    }

    /// R1, the grammar's own test: every key is `<root>:<kind>:<local>`, three
    /// segments with no kind excused, the root a `Slug`, the kind the literal
    /// name, and every local part a `Label`. A root that would add a segment is
    /// refused where the key is composed: `pipeline_key` is `pub` and does not
    /// validate the document first, so this is reachable without one.
    #[test]
    fn every_key_has_exactly_three_segments() {
        let nested = resolution_of(PipelineKind::Resolution, &format!("{CAMPAIGN}:resolution:question.Q1"))
            .expect("a resolution of a resolution keys");
        let keys = samples()
            .into_iter()
            .map(|(document, _)| (document.kind(), pipeline_key(&document).expect("every sample keys")))
            .chain([(PipelineKind::Resolution, nested)]);
        for (kind, key) in keys {
            let segments = key.split(':').collect::<Vec<_>>();
            assert_eq!(segments.len(), 3, "{kind:?} key {key} has three segments");
            assert_eq!(dotted_text("root", segments[0]), Ok(()), "{key}: the root is a slug");
            assert_eq!(segments[1], kind.name(), "{key}: the kind segment is the literal kind");
            assert!(PipelineKind::ALL.iter().any(|known| known.name() == segments[1]));
            for part in segments[2].split('.') {
                assert_eq!(label_text("local", part), Ok(()), "{key}: local part {part:?} is a label");
            }
        }

        // A root that would add a segment, one that would empty a part, and one
        // wider than a segment are all refused before a key is formatted. The
        // refusal names the part that failed, so an empty part reports `""`.
        let trailing_dot = format!("{CAMPAIGN}.");
        let leading_dot = format!(".{CAMPAIGN}");
        let over = "a".repeat(65);
        for root in ["a:b", trailing_dot.as_str(), leading_dot.as_str(), "a..b", over.as_str()] {
            let PipelineDocument::Target(mut target) = samples().remove(1).0 else { unreachable!() };
            target.campaign = Slug(root.into());
            let key = pipeline_key(&PipelineDocument::Target(target));
            assert!(
                matches!(&key, Err(PipelineRefusal::InvalidFormat { field, .. }) if field == "target.campaign"),
                "root {root:?} does not compose a key, got {key:?}"
            );
        }
    }

    /// Defect 1: the two roots are in distinct namespaces because the kind
    /// segment distinguishes them, the same mechanism that separates every
    /// other pair of kinds. Both key; only the pair sees a shared namespace.
    #[test]
    fn roots_of_different_kinds_do_not_share_a_key() {
        let PipelineDocument::Campaign(mut campaign) = campaign_sample() else { unreachable!() };
        campaign.slug = slug(INSTANCE);
        let campaign = pipeline_key(&PipelineDocument::Campaign(campaign));
        let instance = pipeline_key(&PipelineDocument::Instance(instance_sample()));
        assert_eq!(campaign, Ok(format!("{INSTANCE}:campaign:self")));
        assert_eq!(instance, Ok(format!("{INSTANCE}:instance:self")));
        assert_ne!(campaign, instance, "a campaign and an instance of one slug are two documents");
    }

    /// The root is a `Slug`, so a dotted root keys and reads back whole on
    /// both sides: the writer's root check and the reader's root check are
    /// each `dotted_text`, not `label_text`. A campaign `game.cult` and an
    /// instance `ygg.drasil` key, a document under the dotted campaign keys,
    /// and each reads back through `pipeline_id` recovering the root exactly.
    #[test]
    fn dotted_roots_key_and_read_back() {
        let PipelineDocument::Campaign(mut campaign) = campaign_sample() else { unreachable!() };
        campaign.slug = slug("game.cult");
        let campaign_key = pipeline_key(&PipelineDocument::Campaign(campaign)).expect("a dotted campaign keys");
        assert_eq!(campaign_key, "game.cult:campaign:self");
        assert_eq!(
            pipeline_id("read_back", &campaign_key, PipelineKind::Campaign),
            Ok(("game.cult", ROOT_LOCAL)),
            "the dotted campaign root reads back whole"
        );

        let mut instance = instance_sample();
        instance.instance = slug("ygg.drasil");
        let instance_key = pipeline_key(&PipelineDocument::Instance(instance)).expect("a dotted instance keys");
        assert_eq!(instance_key, "ygg.drasil:instance:self");
        assert_eq!(
            pipeline_id("read_back", &instance_key, PipelineKind::Instance),
            Ok(("ygg.drasil", ROOT_LOCAL)),
            "the dotted instance root reads back whole"
        );

        let PipelineDocument::Target(mut target) = samples().remove(1).0 else { unreachable!() };
        target.campaign = slug("game.cult");
        let target_key = pipeline_key(&PipelineDocument::Target(target)).expect("a target under a dotted campaign keys");
        assert_eq!(target_key, "game.cult:target:r2");
        assert_eq!(
            pipeline_id("read_back", &target_key, PipelineKind::Target),
            Ok(("game.cult", "r2")),
            "the dotted root and the local both read back"
        );

        let resolution = resolution_of(PipelineKind::Campaign, &campaign_key).expect("a resolution of a dotted campaign keys");
        assert_eq!(resolution, "game.cult:resolution:campaign.self");
        assert_eq!(
            pipeline_id("read_back", &resolution, PipelineKind::Resolution),
            Ok(("game.cult", "campaign.self")),
            "a resolution under a dotted root reads back"
        );
    }

    /// Cut 6c: a resolution's referent is a full id of the kind it declares,
    /// validated through `PipelineRef` like every other referent, so a
    /// well-formed id of the wrong kind is refused and the refusal names the
    /// entry. The positive case is Ghostlight's own: a ruling partly
    /// overturned by two later records, one of them a resolution named by the
    /// key it has, which only the 6b grammar made nameable. An id no
    /// resolution derives, `eureka-state:resolution:R8`, still validates:
    /// whether a document with an id exists is admission's rule.
    #[test]
    fn resolution_outcome_referents_are_parsed_ids_of_their_kind() {
        let resolved = |subject: PipelineRef, outcome: ResolutionOutcome| {
            let mut resolution = resolution_sample();
            resolution.subject = subject;
            resolution.outcome = outcome;
            PipelineDocument::Resolution(resolution).validate()
        };
        let refused = |field: &str, result: Result<(), PipelineRefusal>| {
            assert!(
                matches!(&result, Err(PipelineRefusal::InvalidFormat { field: at, .. }) if at == field),
                "expected InvalidFormat at {field}, got {result:?}"
            );
        };
        let ruling = |label: &str| PipelineRef { kind: PipelineKind::Ruling, id: id("ruling", label) };
        let question = |label: &str| PipelineRef { kind: PipelineKind::Question, id: id("question", label) };
        let inner = pipeline_key(&PipelineDocument::Resolution(resolution_sample())).expect("the sample keys");
        let resolution = PipelineRef { kind: PipelineKind::Resolution, id: Short(inner.clone()) };

        let superseded = |second: PipelineRef| ResolutionOutcome::Superseded { by: vec![ruling("R9"), second] };
        assert_eq!(resolved(ruling("R8"), superseded(resolution)), Ok(()));
        let forged = PipelineRef { kind: PipelineKind::Ruling, id: Short(inner) };
        refused("resolution.outcome.by[1].id", resolved(ruling("R8"), superseded(forged)));
        let underived = PipelineRef { kind: PipelineKind::Resolution, id: Short(format!("{CAMPAIGN}:resolution:R8")) };
        assert_eq!(resolved(ruling("R8"), superseded(underived)), Ok(()), "well-formed grammar is not the shape's to refuse");

        assert_eq!(resolved(question("Q1"), ResolutionOutcome::Answered { by: ruling("R8") }), Ok(()));
        let forged = PipelineRef { kind: PipelineKind::Question, id: id("ruling", "R8") };
        refused("resolution.outcome.by.id", resolved(question("Q1"), ResolutionOutcome::Answered { by: forged }));

        assert_eq!(resolved(question("Q1"), ResolutionOutcome::Deferred { to: question("Q2") }), Ok(()));
        let forged = PipelineRef { kind: PipelineKind::Ruling, id: id("question", "Q2") };
        refused("resolution.outcome.to.id", resolved(question("Q1"), ResolutionOutcome::Deferred { to: forged }));

        let finding = PipelineRef { kind: PipelineKind::Finding, id: id("finding", "cut-3a.s2.F4") };
        let fixed = |by: Option<PipelineRef>| ResolutionOutcome::Fixed { commit: sha(), by };
        assert_eq!(resolved(finding.clone(), fixed(None)), Ok(()));
        assert_eq!(resolved(finding.clone(), fixed(Some(ruling("R8")))), Ok(()));
        let forged = PipelineRef { kind: PipelineKind::Finding, id: id("ruling", "R8") };
        refused("resolution.outcome.by.id", resolved(finding, fixed(Some(forged))));

        // Soul F2: the validator's bound on a supersession is its own, not
        // only the schema's. Eight referents are the most a subject names;
        // nine are refused as a count before any item is read.
        let rulings = |count: usize| (1..=count).map(|n| ruling(&format!("R{n}"))).collect::<Vec<_>>();
        assert_eq!(resolved(ruling("R8"), ResolutionOutcome::Superseded { by: rulings(8) }), Ok(()));
        assert_eq!(
            resolved(ruling("R8"), ResolutionOutcome::Superseded { by: rulings(9) }),
            Err(PipelineRefusal::FieldBound { field: "resolution.outcome.by".into(), limit: 8, actual: 9 })
        );

        // Soul F3: the two reason-carrying outcomes are bounded like every
        // `Line`; the shared arm is pinned on both so neither can drop out.
        let wide = Line("a".repeat(1001));
        for outcome in [
            ResolutionOutcome::Recorded { reason: wide.clone() },
            ResolutionOutcome::Withdrawn { reason: wide },
        ] {
            let finding = PipelineRef { kind: PipelineKind::Finding, id: id("finding", "cut-3a.s2.F4") };
            assert_eq!(
                resolved(finding, outcome),
                Err(PipelineRefusal::FieldBound { field: "resolution.outcome.reason".into(), limit: 1000, actual: 1001 })
            );
        }
    }

    /// Ruling B: a fix names the tree where the finding stopped being true,
    /// as a `Sha`. Uppercase hex of a legal length is the forgery a length
    /// check passes and `hex` refuses.
    #[test]
    fn fixed_resolution_requires_a_commit_sha() {
        let fixed = |commit: &str, by: Option<PipelineRef>| {
            let mut resolution = resolution_sample();
            resolution.subject = PipelineRef { kind: PipelineKind::Finding, id: id("finding", "cut-3a.s2.F4") };
            resolution.outcome = ResolutionOutcome::Fixed { commit: Sha(commit.into()), by };
            PipelineDocument::Resolution(resolution).validate()
        };
        assert_eq!(fixed("5f98228d9c", None), Ok(()));
        for forged in ["5F98228D9C", "5f9822", "dirty-worktree", ""] {
            assert_eq!(
                fixed(forged, None),
                Err(PipelineRefusal::InvalidFormat { field: "resolution.outcome.commit".into(), value: forged.into() }),
                "{forged:?} is not a commit"
            );
        }
        // Soul F1: naming the record that fixed it does not excuse the commit.
        let ruling = PipelineRef { kind: PipelineKind::Ruling, id: id("ruling", "R8") };
        assert_eq!(
            fixed("dirty-worktree", Some(ruling)),
            Err(PipelineRefusal::InvalidFormat { field: "resolution.outcome.commit".into(), value: "dirty-worktree".into() }),
            "a fix with a named record still names a commit"
        );
    }

    /// A mutation record has a key-safe identity a verdict claim can name,
    /// and is pinned to a tree: its label is a `Label` and its commit a
    /// `Sha`, so a dotted label and free text where a sha belongs are refused.
    #[test]
    fn mutation_records_carry_a_dot_free_label_and_a_commit() {
        let recorded = |label: &str, commit: &str| {
            let mut report = report_sample();
            // Through `From<&str>`, which every text type has, so the
            // widened types under M19 and M20 still compile and the
            // forgeries reach validation.
            report.mutations[0].label = label.into();
            report.mutations[0].commit = commit.into();
            PipelineDocument::CutReport(report).validate()
        };
        assert_eq!(recorded("M1", "5f98228d"), Ok(()));
        assert_eq!(
            recorded("M1.a", "5f98228d"),
            Err(PipelineRefusal::InvalidFormat { field: "cut_report.mutations[0].label".into(), value: "M1.a".into() })
        );
        assert_eq!(
            recorded("M1", "dirty-worktree"),
            Err(PipelineRefusal::InvalidFormat {
                field: "cut_report.mutations[0].commit".into(),
                value: "dirty-worktree".into(),
            })
        );
    }

    /// Ruling A's shape half: a claim names the promise it measured and the
    /// mutations it ran, both as `Label`s, and names at most eight mutations.
    #[test]
    fn a_verdict_claim_names_the_promise_and_the_mutation_it_measured() {
        let sample = verdict_sample();
        assert_eq!(sample.claims[0].promise, Some(l("P1")));
        assert_eq!(sample.claims[0].mutations, vec![l("M1")]);
        assert_eq!(PipelineDocument::Verdict(sample).validate(), Ok(()));

        let claimed = |promise: Option<Label>, mutations: Vec<Label>| {
            let mut verdict = verdict_sample();
            verdict.claims[0].promise = promise;
            verdict.claims[0].mutations = mutations;
            PipelineDocument::Verdict(verdict).validate()
        };
        assert_eq!(claimed(None, vec![]), Ok(()), "a claim need not measure a promise");
        assert_eq!(
            claimed(Some(l("P1.a")), vec![]),
            Err(PipelineRefusal::InvalidFormat { field: "verdict.claims[0].promise".into(), value: "P1.a".into() })
        );
        assert_eq!(
            claimed(None, vec![l("M1.a")]),
            Err(PipelineRefusal::InvalidFormat { field: "verdict.claims[0].mutations[0]".into(), value: "M1.a".into() })
        );
        let labels = |count: usize| (1..=count).map(|n| l(&format!("M{n}"))).collect::<Vec<_>>();
        assert_eq!(claimed(None, labels(8)), Ok(()));
        assert_eq!(
            claimed(None, labels(9)),
            Err(PipelineRefusal::FieldBound { field: "verdict.claims[0].mutations".into(), limit: 8, actual: 9 })
        );
    }

    /// A tripwire on the one hazard of retyping the estimate: the delta's two
    /// `u32` fields are not interchangeable. The sample spec is Cut 3a's, a
    /// net addition of 900 lines, and the order is asserted.
    #[test]
    fn the_sample_cut_spec_estimates_a_net_addition() {
        let PipelineDocument::CutSpec(spec) = samples().remove(4).0 else { unreachable!() };
        assert_eq!(spec.estimate.lines_added, 900);
        assert_eq!(spec.estimate.lines_removed, 0);
        assert_eq!(spec.estimate, report_sample().structural_delta, "the estimate and the actual are one shape");
    }

    /// Defect 2: a resolution's key carries its subject's kind as a literal
    /// part of the local, so nothing infers it and two subjects of one local
    /// and different kinds resolve to two keys. Each reads back through the
    /// reader with the subject kind it was built from.
    #[test]
    fn a_resolution_names_its_subjects_kind() {
        let pairs = [
            (PipelineKind::Question, id("question", "Q1").0, PipelineKind::Ruling, id("ruling", "Q1").0),
            (
                PipelineKind::Campaign,
                format!("{CAMPAIGN}:campaign:self"),
                PipelineKind::Instance,
                format!("{CAMPAIGN}:instance:self"),
            ),
        ];
        for (left_kind, left_id, right_kind, right_id) in pairs {
            let left = resolution_of(left_kind, &left_id).expect("the left subject keys");
            let right = resolution_of(right_kind, &right_id).expect("the right subject keys");
            assert_ne!(left, right, "resolutions of {left_id} and {right_id} are two documents");
            for (kind, key) in [(left_kind, &left), (right_kind, &right)] {
                let (root, local) = pipeline_id("read_back", key, PipelineKind::Resolution).expect("reads back");
                assert_eq!(root, CAMPAIGN);
                assert_eq!(local.split('.').next(), Some(kind.name()), "{key} names its subject's kind");
            }
        }
    }

    /// Defect 3: a resolution's own key is an ordinary id, so it composes as a
    /// subject like any other, and the key reads back to the inner key. Depth
    /// is bounded by the local alone: each nesting prepends `resolution.`, 11
    /// bytes, so a chain `n` deep over a depth-one local of `L` bytes composes
    /// `11 * (n - 1) + L` bytes against `LOCAL_MAX`. The deepest chain that
    /// keys is therefore `(LOCAL_MAX - L) / 11 + 1`, and it depends on the
    /// subject: at 64, `ruling.A` (8 bytes) keys six deep, `question.Q1` (11
    /// bytes) five.
    #[test]
    fn a_resolution_of_a_resolution_reads_back() {
        let inner = resolution_of(PipelineKind::Question, &id("question", "Q1").0).expect("the inner keys");
        let mut outer = resolution_sample();
        outer.subject = PipelineRef { kind: PipelineKind::Resolution, id: Short(inner.clone()) };
        let outer = PipelineDocument::Resolution(outer);
        assert_eq!(outer.validate(), Ok(()));
        let key = pipeline_key(&outer).expect("a resolution of a resolution keys");
        assert_eq!(key, format!("{CAMPAIGN}:resolution:resolution.question.Q1"));
        let (root, local) = pipeline_id("read_back", &key, PipelineKind::Resolution).expect("reads back");
        let (subject_kind, subject_local) = local.split_once('.').expect("the local names a subject");
        assert_eq!(format!("{root}:{subject_kind}:{subject_local}"), inner, "the recovered subject is the inner key");

        let prefix = format!("{CAMPAIGN}:resolution:").len();
        let nesting = "resolution.".len();
        for (subject_kind, subject_id) in [(PipelineKind::Ruling, id("ruling", "A")), (PipelineKind::Question, id("question", "Q1"))] {
            let mut key = resolution_of(subject_kind, &subject_id.0).expect("the depth-one resolution keys");
            let depth_one = key.len() - prefix;
            let deepest = (LOCAL_MAX - depth_one) / nesting + 1;
            for depth in 2..=deepest {
                key = resolution_of(PipelineKind::Resolution, &key)
                    .unwrap_or_else(|error| panic!("{subject_id:?} depth {depth}: {error}"));
                assert_eq!(key.len() - prefix, nesting * (depth - 1) + depth_one, "{key}: depth {depth} local length");
            }
            assert!(key.len() - prefix <= LOCAL_MAX, "{key}: the deepest chain fits the local");
            assert!(
                matches!(
                    resolution_of(PipelineKind::Resolution, &key),
                    Err(PipelineRefusal::InvalidFormat { field, .. }) if field == "resolution.key"
                ),
                "{key}: nesting {} is refused on the local bound",
                deepest + 1
            );
        }
    }

    /// The total bound, in the one place it lives: parts that are each a legal
    /// label compose a local wider than 64 bytes and are refused as a whole.
    #[test]
    fn a_composed_local_is_bounded_whole() {
        let mut wide = hand_off_sample();
        wide.to_instance = Slug("a".repeat(60));
        assert_eq!(wide.to_instance.validate("hand_off.to_instance"), Ok(()), "the receiver alone is legal");
        assert_eq!(
            label_text("part", &key_segment(&wide.repo.0)),
            Ok(()),
            "the repo alone is legal"
        );
        let key = pipeline_key(&PipelineDocument::HandOff(wide));
        assert!(
            matches!(&key, Err(PipelineRefusal::InvalidFormat { field, value }) if field == "hand_off.key" && value.len() > 64),
            "a local of legal parts is still bounded whole, got {key:?}"
        );

        // The bound is 64 exactly, on both sides, whether the part that fills
        // it is a plain label or an escaped repo. A hand-off local is
        // `<receiver>.<repo>.<date>`, the repo `GameCult_-Epiphany` and the
        // date ten bytes, so the receiver fills the rest; a stewardship local is
        // the escaped repo alone, `GameCult_-` and the name.
        let hand_off_rest = "GameCult_-Epiphany".len() + ".".len() + date().0.len() + ".".len();
        let received = |receiver: usize| {
            let mut hand_off = hand_off_sample();
            hand_off.to_instance = Slug("a".repeat(receiver));
            pipeline_key(&PipelineDocument::HandOff(hand_off))
        };
        let stewarded = |name: usize| {
            let mut stewardship = stewardship_sample();
            stewardship.repo = OrgRepo(format!("GameCult/{}", "a".repeat(name)));
            pipeline_key(&PipelineDocument::Stewardship(stewardship))
        };
        for (label, at_64, at_65) in [
            ("plain label", received(64 - hand_off_rest), received(65 - hand_off_rest)),
            ("escaped repo", stewarded(64 - "GameCult_-".len()), stewarded(65 - "GameCult_-".len())),
        ] {
            let keyed = at_64.unwrap_or_else(|error| panic!("{label}: a 64-byte local keys: {error}"));
            assert_eq!(keyed.split(':').nth(2).map(str::len), Some(64), "{label}: {keyed} fills the local exactly");
            assert!(
                matches!(&at_65, Err(PipelineRefusal::InvalidFormat { value, .. }) if value.len() == 65),
                "{label}: a 65-byte local is refused, got {at_65:?}"
            );
        }
    }

    /// R2, on the composer every kind goes through: no local part carries the
    /// separator, the head included. The live case is a hand-off with a dotted
    /// receiver, which keys to an escaped label rather than a wider local.
    #[test]
    fn no_local_part_carries_the_separator() {
        assert_eq!(
            local("probe", &["a.b"]),
            Err(PipelineRefusal::InvalidFormat { field: "probe".into(), value: "a.b".into() }),
            "the head may not carry the separator"
        );
        assert_eq!(
            local("probe", &["a", "b.c"]),
            Err(PipelineRefusal::InvalidFormat { field: "probe".into(), value: "b.c".into() }),
            "a tail part may not carry the separator"
        );
        assert_eq!(local("probe", &["a", "b", "c"]), Ok("a.b.c".into()));

        let mut dotted = hand_off_sample();
        dotted.to_instance = slug("thought-cage.GameCult");
        let key = pipeline_key(&PipelineDocument::HandOff(dotted)).expect("a dotted receiver keys");
        let local = key.split(':').nth(2).expect("three segments");
        assert_eq!(local.split('.').count(), 3, "{key}: the receiver is one part, not two");
        assert_eq!(local.split('.').next(), Some("thought-cage_dGameCult"));
    }

    #[test]
    fn pipeline_published_schemas_match_derivation() -> Result<()> {
        let published = Path::new(env!("CARGO_MANIFEST_DIR")).join("../schemas/cultnet");
        let index: serde_json::Value = serde_json::from_slice(&std::fs::read(published.join("index.json"))?)?;
        // A directory this run alone writes, so everything in it is what this
        // run derived. A previous run's leftovers -- a mutation run's
        // especially, since those deliberately derive schemas no committed type
        // produces -- are not this derivation, and copying one of those into
        // `schemas/cultnet` publishes a schema no Rust type produces. A shared
        // directory cleared first would have to defend that clear against a
        // held handle; an unshared one has nothing to defend. The cost is that
        // nothing ever removes one. The directory is only created when
        // something is stale, so a passing run leaves none -- but a failing run
        // leaves its copies behind for good, one directory per failing run
        // where the shared design left one however often it failed. That is the
        // trade: the copies are the evidence the failure message names, and
        // they outlive the run that wrote them. Whoever reads the failure is
        // the one who deletes them.
        let derived_dir = std::env::temp_dir().join(format!(
            "epiphany-pipeline-schemas-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
        ));
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
