use crate::packaged_release::{EpiphanyPackagedReleaseEntry, EpiphanyPackagedReleaseHead};
use anyhow::{Result, anyhow};
use cultcache_rs::DatabaseEntry;
use cultmesh_rs::{CultMesh, CultMeshNode, CultMeshNodeOptions, cultmesh_documents};
use std::path::Path;

pub const EPIPHANY_CULTMESH_SWARM_BRAKE_TYPE: &str = "epiphany.cultmesh.swarm_brake";
pub const EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.swarm_brake.v0";
pub const EPIPHANY_CULTMESH_SWARM_BRAKE_KEY: &str = "epiphany-local/swarm-brake";
pub const EPIPHANY_CANONICAL_SWARM_BRAKE_ID: &str = "epiphany/swarm-brake";
pub const EPIPHANY_CANONICAL_SWARM_BRAKE_OWNER: &str = "epiphany.swarm-brake";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.swarm_brake",
    schema = "EpiphanyCultMeshSwarmBrakeEntry"
)]
pub struct EpiphanyCultMeshSwarmBrakeEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub brake_id: String,
    #[cultcache(key = 2)]
    pub status: String,
    #[cultcache(key = 3)]
    pub scope: String,
    #[cultcache(key = 4)]
    pub reason: String,
    #[cultcache(key = 5)]
    pub operator_agent_id: String,
    #[cultcache(key = 6)]
    pub affected_clusters: Vec<String>,
    #[cultcache(key = 7)]
    pub protected_surfaces: Vec<String>,
    #[cultcache(key = 8)]
    pub created_at_utc: String,
    #[cultcache(key = 9)]
    pub expires_at_utc: Option<String>,
    #[cultcache(key = 10)]
    pub private_state_exposed: bool,
    #[cultcache(key = 11)]
    pub notes: Vec<String>,
    #[cultcache(key = 12, default)]
    pub runtime_id: String,
}

cultmesh_documents!(EpiphanyCultMeshDocuments {
    EpiphanyPackagedReleaseEntry => crate::packaged_release::EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION,
    EpiphanyPackagedReleaseHead => crate::packaged_release::EPIPHANY_PACKAGED_RELEASE_HEAD_SCHEMA_VERSION,
    EpiphanyCultMeshSwarmBrakeEntry => EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION,
    crate::persona_feedback_admission::LocalAdmittedPersonaFeedback => crate::persona_feedback_admission::LOCAL_PERSONA_FEEDBACK_SCHEMA_VERSION,
});

pub fn open_epiphany_cultmesh_node(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<CultMeshNode> {
    CultMesh::create_node(
        store_path,
        EpiphanyCultMeshDocuments,
        CultMeshNodeOptions {
            runtime_id: runtime_id.into(),
            ..CultMeshNodeOptions::default()
        },
    )
}

pub fn canonical_epiphany_swarm_brake_protected_surfaces() -> Vec<String> {
    vec![
        "resident.self".to_string(),
        "coordinator.run".to_string(),
        "persona.public_speech".to_string(),
        "hands.consequence".to_string(),
        "atlas.publish".to_string(),
        "atlas.project".to_string(),
        "atlas.impact_ingress".to_string(),
    ]
}

pub fn default_epiphany_cultmesh_swarm_brake(
    generated_at_utc: impl Into<String>,
) -> EpiphanyCultMeshSwarmBrakeEntry {
    EpiphanyCultMeshSwarmBrakeEntry {
        schema_version: EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION.to_string(),
        brake_id: EPIPHANY_CANONICAL_SWARM_BRAKE_ID.to_string(),
        status: "released".to_string(),
        scope: "swarm".to_string(),
        reason: "No swarm brake is engaged; unattended cognition still requires typed authority."
            .to_string(),
        operator_agent_id: EPIPHANY_CANONICAL_SWARM_BRAKE_OWNER.to_string(),
        affected_clusters: Vec::new(),
        protected_surfaces: canonical_epiphany_swarm_brake_protected_surfaces(),
        created_at_utc: generated_at_utc.into(),
        expires_at_utc: None,
        private_state_exposed: false,
        notes: vec!["The swarm brake pauses Epiphany cognition and authored consequences.".into()],
        runtime_id: String::new(),
    }
}

pub fn engage_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    reason: impl Into<String>,
    actor_id: impl Into<String>,
    created_at_utc: impl Into<String>,
    allow_engaged_adoption: bool,
) -> Result<EpiphanyCultMeshSwarmBrakeEntry> {
    let runtime_id = runtime_id.into();
    let actor_id = actor_id.into();
    if actor_id.trim().is_empty() {
        return Err(anyhow!("swarm brake engagement requires an actor identity"));
    }
    if allow_engaged_adoption && actor_id != "Idunn" {
        return Err(anyhow!("only Idunn may adopt an already-engaged legacy brake"));
    }
    if let Some(current) = load_epiphany_cultmesh_swarm_brake(&store_path, runtime_id.clone())? {
        let foreign = current.brake_id != EPIPHANY_CANONICAL_SWARM_BRAKE_ID
            || current.operator_agent_id != EPIPHANY_CANONICAL_SWARM_BRAKE_OWNER;
        if current.status == "engaged" && foreign && !allow_engaged_adoption {
            return Err(anyhow!(
                "refusing to replace engaged foreign swarm brake {} owned by {}",
                current.brake_id,
                current.operator_agent_id
            ));
        }
    }
    let mut brake = default_epiphany_cultmesh_swarm_brake(created_at_utc);
    brake.status = "engaged".to_string();
    brake.scope = "all".to_string();
    brake.reason = reason.into();
    brake.affected_clusters = vec![runtime_id.clone()];
    brake.notes = vec![format!("Explicit brake engagement by {actor_id}.")];
    write_epiphany_cultmesh_swarm_brake(store_path, runtime_id, brake)
}

pub fn release_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    reason: impl Into<String>,
    actor_id: impl Into<String>,
    created_at_utc: impl Into<String>,
) -> Result<EpiphanyCultMeshSwarmBrakeEntry> {
    let runtime_id = runtime_id.into();
    let actor_id = actor_id.into();
    if actor_id.trim().is_empty() {
        return Err(anyhow!("swarm brake release requires an actor identity"));
    }
    let mut brake = load_epiphany_cultmesh_swarm_brake(&store_path, runtime_id.clone())?
        .ok_or_else(|| anyhow!("refusing to release an absent swarm brake"))?;
    if brake.brake_id != EPIPHANY_CANONICAL_SWARM_BRAKE_ID
        || brake.operator_agent_id != EPIPHANY_CANONICAL_SWARM_BRAKE_OWNER
    {
        return Err(anyhow!(
            "refusing to release foreign swarm brake {} owned by {}",
            brake.brake_id,
            brake.operator_agent_id
        ));
    }
    brake.status = "released".to_string();
    brake.reason = reason.into();
    brake.created_at_utc = created_at_utc.into();
    brake.expires_at_utc = None;
    brake.notes = vec![format!("Explicit brake release by {actor_id}.")];
    write_epiphany_cultmesh_swarm_brake(store_path, runtime_id, brake)
}

pub fn write_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    mut brake: EpiphanyCultMeshSwarmBrakeEntry,
) -> Result<EpiphanyCultMeshSwarmBrakeEntry> {
    let runtime_id = runtime_id.into();
    brake.runtime_id = runtime_id.clone();
    validate_swarm_brake(&brake)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let written = node.put(EPIPHANY_CULTMESH_SWARM_BRAKE_KEY, &brake)?;
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshSwarmBrakeEntry>> {
    let runtime_id = runtime_id.into();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id.clone())?;
    Ok(node
        .get::<EpiphanyCultMeshSwarmBrakeEntry>(EPIPHANY_CULTMESH_SWARM_BRAKE_KEY)?
        .filter(|brake| brake.runtime_id.is_empty() || brake.runtime_id == runtime_id))
}

fn validate_swarm_brake(brake: &EpiphanyCultMeshSwarmBrakeEntry) -> Result<()> {
    if brake.private_state_exposed {
        return Err(anyhow!("swarm brake must not expose private state"));
    }
    if brake.brake_id.trim().is_empty()
        || brake.scope.trim().is_empty()
        || brake.runtime_id.trim().is_empty()
        || brake.created_at_utc.trim().is_empty()
    {
        return Err(anyhow!("swarm brake identity, scope, runtime, and timestamp are required"));
    }
    if !matches!(brake.status.as_str(), "released" | "engaged") {
        return Err(anyhow!("swarm brake status must be released or engaged"));
    }
    if brake.status == "engaged"
        && (brake.reason.trim().is_empty()
            || brake.operator_agent_id.trim().is_empty()
            || (brake.affected_clusters.is_empty() && brake.protected_surfaces.is_empty()))
    {
        return Err(anyhow!("engaged swarm brake requires operator id, reason, and scope"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swarm_brake_round_trip_preserves_canonical_authority() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("brake.ccmp");
        let engaged = engage_epiphany_cultmesh_swarm_brake(
            &store,
            "runtime",
            "test",
            "operator",
            "2026-08-23T00:00:00Z",
            false,
        )?;
        assert_eq!(engaged.status, "engaged");
        assert_eq!(engaged.brake_id, EPIPHANY_CANONICAL_SWARM_BRAKE_ID);
        assert!(engaged.protected_surfaces.iter().any(|value| value == "hands.consequence"));
        assert_eq!(
            load_epiphany_cultmesh_swarm_brake(&store, "runtime")?,
            Some(engaged)
        );
        Ok(())
    }

    #[test]
    fn swarm_brake_refuses_private_or_unreasoned_engagement() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("brake.ccmp");
        let mut brake = default_epiphany_cultmesh_swarm_brake("2026-08-23T00:00:00Z");
        brake.private_state_exposed = true;
        assert!(write_epiphany_cultmesh_swarm_brake(&store, "runtime", brake).is_err());

        let mut brake = default_epiphany_cultmesh_swarm_brake("2026-08-23T00:00:00Z");
        brake.status = "engaged".into();
        brake.reason.clear();
        assert!(write_epiphany_cultmesh_swarm_brake(&store, "runtime", brake).is_err());
        Ok(())
    }
}
