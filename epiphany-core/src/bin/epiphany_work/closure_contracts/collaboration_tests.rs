#[cfg(test)]
mod collaboration_tests {
    use super::*;

    #[test]
    fn topic_comment_cannot_counterfeit_publication_state() {
        let text = r#"
schema_version = "epiphany.repo_collaboration_topic.v0"
safe_action_family = "repo.collaboration_topic"
summary = "summary"
private_state_exposed = false
[topic]
status = "proposed"
authoring_owner = "Imagination"
discussion_owner = "Persona"
publication_owner = "Bifrost"
requested_public_room = "epiphany-global/persona-collaboration/item"
requested_eve_surface = "eve://epiphany/repo/item/collaboration"
# public_room_published = false
public_room_published = true
eve_surface_published = false
provider_receipt_required = true
persona_discussion_allowed = true
human_discussion_allowed = true
[imagination]
consensus_route = "imagination://repo/item/consensus-discovery"
consensus_required_before_action = true
candidate_actions_are_non_authoritative = true
mind_adoption_required = true
bifrost_publication_required = true
[authority]
adoption_authorized = false
hands_action_authorized = false
publication_authorized = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
"#;
        let topic = parse_repo_collaboration_topic(text).expect("fixture is typed TOML");
        assert!(!topic.remains_unpublished_proposal());
    }
}
