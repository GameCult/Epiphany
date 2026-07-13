#[derive(Debug, Deserialize)]
pub(super) struct RepoCollaborationPolicy {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    policy: CollaborationPolicyProposal,
    body: CollaborationBody,
    verses: std::collections::BTreeMap<String, toml::Value>,
    eve: CollaborationEve,
    persona: std::collections::BTreeMap<String, bool>,
    imagination: CollaborationImagination,
    authority: std::collections::BTreeMap<String, bool>,
}
impl RepoCollaborationPolicy {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_collaboration_policy.v0"
            && self.safe_action_family == "repo.collaboration_policy"
    }
    pub(super) fn remains_proposal(&self) -> bool {
        self.policy.status == "proposed"
            && self.policy.authoring_owner == "Imagination"
            && self.policy.required_reviewers == ["Persona", "Mind"]
            && !self.policy.policy_admitted
            && self.policy.publication_owner == "Bifrost"
    }
    pub(super) fn preserves_provider_truth(&self) -> bool {
        self.body.provider_owns_truth && !self.body.renderer_owns_truth
    }
    pub(super) fn has_verse_boundaries(&self) -> bool {
        self.verses.get("private").and_then(toml::Value::as_str) == Some("epiphany-internal")
            && self.verses.get("local").and_then(toml::Value::as_str) == Some("gamecult-local")
            && self.verses.get("public").and_then(toml::Value::as_str) == Some("epiphany-global")
            && self
                .verses
                .get("private_state_may_leave_repo")
                .and_then(toml::Value::as_bool)
                == Some(false)
            && self
                .verses
                .get("odin_discoverable")
                .and_then(toml::Value::as_bool)
                == Some(true)
    }
    pub(super) fn has_eve_request(&self) -> bool {
        self.eve.surface.starts_with("eve://epiphany/repo/")
            && self.eve.compact_tui_required
            && self.eve.connection_receipt_required
            && self.eve.supported_actions == ["read-queue", "discuss", "submit-feedback"]
    }
    pub(super) fn has_persona_route(&self) -> bool {
        [
            "public_discussion_allowed",
            "human_discussion_allowed",
            "peer_persona_discussion_allowed",
            "speech_audit_required",
            "feedback_must_route_to_imagination",
        ]
        .iter()
        .all(|key| self.persona.get(*key) == Some(&true))
    }
    pub(super) fn has_imagination_route(&self) -> bool {
        self.imagination
            .feedback_route
            .starts_with("imagination://repo/")
            && self.imagination.consensus_required_before_adoption
            && self.imagination.candidate_actions_non_authoritative
            && self.imagination.mind_adoption_required
            && self.imagination.bifrost_publication_required
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        [
            "direct_hands_authority",
            "direct_mind_state_commit",
            "direct_publication_authority",
            "direct_merge_authority",
            "service_lifecycle_authority",
            "cross_body_mutation_authority",
            "private_verse_rummaging",
        ]
        .iter()
        .all(|key| self.authority.get(*key) == Some(&false))
            && self.authority.get("requires_cultmesh_receipts") == Some(&true)
    }
}
#[derive(Debug, Deserialize)]
struct CollaborationPolicyProposal {
    status: String,
    authoring_owner: String,
    required_reviewers: Vec<String>,
    policy_admitted: bool,
    publication_owner: String,
}
#[derive(Debug, Deserialize)]
struct CollaborationBody {
    provider_owns_truth: bool,
    renderer_owns_truth: bool,
}
#[derive(Debug, Deserialize)]
struct CollaborationEve {
    surface: String,
    compact_tui_required: bool,
    connection_receipt_required: bool,
    supported_actions: Vec<String>,
}
#[derive(Debug, Deserialize)]
struct CollaborationImagination {
    feedback_route: String,
    consensus_required_before_adoption: bool,
    candidate_actions_non_authoritative: bool,
    mind_adoption_required: bool,
    bifrost_publication_required: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct RepoCollaborationTopic {
    pub(super) schema_version: String,
    pub(super) safe_action_family: String,
    pub(super) summary: String,
    pub(super) private_state_exposed: bool,
    topic: CollaborationTopicProposal,
    imagination: CollaborationTopicImagination,
    authority: std::collections::BTreeMap<String, bool>,
}
impl RepoCollaborationTopic {
    pub(super) fn has_canonical_identity(&self) -> bool {
        self.schema_version == "epiphany.repo_collaboration_topic.v0"
            && self.safe_action_family == "repo.collaboration_topic"
    }
    pub(super) fn remains_unpublished_proposal(&self) -> bool {
        let t = &self.topic;
        t.status == "proposed"
            && t.authoring_owner == "Imagination"
            && t.discussion_owner == "Persona"
            && t.publication_owner == "Bifrost"
            && t.requested_public_room
                .starts_with("epiphany-global/persona-collaboration/")
            && t.requested_eve_surface.starts_with("eve://epiphany/repo/")
            && !t.public_room_published
            && !t.eve_surface_published
            && t.provider_receipt_required
            && t.persona_discussion_allowed
            && t.human_discussion_allowed
    }
    pub(super) fn has_imagination_route(&self) -> bool {
        self.imagination
            .consensus_route
            .starts_with("imagination://repo/")
            && self.imagination.consensus_required_before_action
            && self.imagination.candidate_actions_are_non_authoritative
            && self.imagination.mind_adoption_required
            && self.imagination.bifrost_publication_required
    }
    pub(super) fn has_authority_seals(&self) -> bool {
        [
            "adoption_authorized",
            "hands_action_authorized",
            "publication_authorized",
            "cross_body_mutation_authorized",
            "private_verse_rummaging",
        ]
        .iter()
        .all(|key| self.authority.get(*key) == Some(&false))
    }
}
#[derive(Debug, Deserialize)]
struct CollaborationTopicProposal {
    status: String,
    authoring_owner: String,
    discussion_owner: String,
    publication_owner: String,
    requested_public_room: String,
    requested_eve_surface: String,
    public_room_published: bool,
    eve_surface_published: bool,
    provider_receipt_required: bool,
    persona_discussion_allowed: bool,
    human_discussion_allowed: bool,
}
#[derive(Debug, Deserialize)]
struct CollaborationTopicImagination {
    consensus_route: String,
    consensus_required_before_action: bool,
    candidate_actions_are_non_authoritative: bool,
    mind_adoption_required: bool,
    bifrost_publication_required: bool,
}

pub(super) fn parse_repo_collaboration_policy(text: &str) -> Result<RepoCollaborationPolicy> {
    toml::from_str(text).context("collaboration policy is not valid typed TOML")
}
pub(super) fn parse_repo_collaboration_topic(text: &str) -> Result<RepoCollaborationTopic> {
    toml::from_str(text).context("collaboration topic is not valid typed TOML")
}

include!("collaboration_tests.rs");
