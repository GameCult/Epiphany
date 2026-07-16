use crate::coordinator_state_transaction::{
    commit_coordinator_state_transaction, open_coordinator_state_transaction,
};
use crate::{
    EpiphanyCoordinatorStateApplied, EpiphanyStateUpdate, EpiphanyStateUpdatedField,
    apply_coordinator_state_update_to_state, coordinator_acceptance_cache, read_coordinator_state,
};
use anyhow::{Result, anyhow};
use cultcache_rs::DatabaseEntry;
use epiphany_state_model::EpiphanyThreadState;
use sha2::{Digest, Sha256};
use std::path::Path;

pub const USER_OBJECTIVE_INTAKE_SCHEMA_VERSION: &str = "gamecult.epiphany.user_objective_intake.v0";
pub const USER_OBJECTIVE_INTAKE_TYPE: &str = "epiphany.coordinator.user_objective_intake";
pub const USER_OBJECTIVE_INTAKE_CONTRACT: &str = "The human supplies the initial objective. Self records that assertion and the first canonical thread state in one CAS. Repeated identical intake is read-idempotent; replacement requires a separate reviewed adoption flow.";

#[derive(Debug, Clone)]
pub struct UserObjectiveIntakeInput {
    pub thread_id: String,
    pub objective: String,
    pub source_actor: String,
    pub source_ref: String,
    pub submitted_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator.user_objective_intake",
    schema = "UserObjectiveIntake"
)]
pub struct UserObjectiveIntake {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intake_id: String,
    #[cultcache(key = 2)]
    pub thread_id: String,
    #[cultcache(key = 3)]
    pub objective: String,
    #[cultcache(key = 4)]
    pub objective_sha256: String,
    #[cultcache(key = 5)]
    pub source_actor: String,
    #[cultcache(key = 6)]
    pub source_ref: String,
    #[cultcache(key = 7)]
    pub submitted_at: String,
    #[cultcache(key = 8)]
    pub contract: String,
}

#[derive(Debug, Clone)]
pub struct UserObjectiveIntakeApplied {
    pub intake: UserObjectiveIntake,
    pub state: EpiphanyCoordinatorStateApplied,
    pub changed: bool,
}

pub fn intake_user_objective(
    store: &Path,
    input: UserObjectiveIntakeInput,
) -> Result<UserObjectiveIntakeApplied> {
    let thread_id = input.thread_id.trim();
    let objective = input.objective.trim();
    let source_actor = input.source_actor.trim();
    let source_ref = input.source_ref.trim();
    if thread_id.is_empty()
        || objective.is_empty()
        || source_actor.is_empty()
        || source_ref.is_empty()
        || chrono::DateTime::parse_from_rfc3339(input.submitted_at.trim()).is_err()
    {
        return Err(anyhow!("invalid typed user-objective intake"));
    }
    let objective_sha256 = format!("{:x}", Sha256::digest(objective.as_bytes()));
    let intake_id = format!("user-objective-{:x}", Sha256::digest(thread_id.as_bytes()));

    if let Some(state) = read_coordinator_state(store)? {
        let cache = coordinator_acceptance_cache(store)?;
        let existing = cache
            .get::<UserObjectiveIntake>(&intake_id)?
            .ok_or_else(|| anyhow!("authoritative state has no typed user-objective intake"))?;
        if existing.thread_id != thread_id
            || existing.objective != objective
            || existing.objective_sha256 != objective_sha256
            || existing.source_actor != source_actor
            || existing.source_ref != source_ref
            || state.objective.as_deref().map(str::trim) != Some(objective)
        {
            return Err(anyhow!(
                "refusing to replace the authoritative coordinator objective; use a typed objective-adoption flow"
            ));
        }
        return Ok(UserObjectiveIntakeApplied {
            intake: existing,
            state: EpiphanyCoordinatorStateApplied {
                revision: state.revision,
                changed_fields: Vec::new(),
                state,
            },
            changed: false,
        });
    }

    let current = EpiphanyThreadState::default();
    let update = EpiphanyStateUpdate {
        expected_revision: Some(0),
        objective: Some(objective.to_string()),
        ..Default::default()
    };
    let next = apply_coordinator_state_update_to_state(&current, update, None)?;
    let intake = UserObjectiveIntake {
        schema_version: USER_OBJECTIVE_INTAKE_SCHEMA_VERSION.into(),
        intake_id: intake_id.clone(),
        thread_id: thread_id.to_string(),
        objective: objective.to_string(),
        objective_sha256,
        source_actor: source_actor.to_string(),
        source_ref: source_ref.to_string(),
        submitted_at: input.submitted_at,
        contract: USER_OBJECTIVE_INTAKE_CONTRACT.into(),
    };
    let mut transaction = open_coordinator_state_transaction(store, &current)?;
    let envelope = transaction.prepare_entry(&intake_id, &intake)?.0;
    commit_coordinator_state_transaction(
        &mut transaction,
        thread_id,
        &next,
        vec![envelope],
        Vec::new(),
    )?;
    Ok(UserObjectiveIntakeApplied {
        intake,
        state: EpiphanyCoordinatorStateApplied {
            revision: next.revision,
            changed_fields: vec![EpiphanyStateUpdatedField::Objective],
            state: next,
        },
        changed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(objective: &str) -> UserObjectiveIntakeInput {
        UserObjectiveIntakeInput {
            thread_id: "thread-1".into(),
            objective: objective.into(),
            source_actor: "operator".into(),
            source_ref: "cli://epiphany-mvp-coordinator".into(),
            submitted_at: "2026-07-16T14:00:00Z".into(),
        }
    }

    #[test]
    fn intake_is_atomic_idempotent_and_seed_only() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        let first = intake_user_objective(&store, input("Map the machine"))?;
        assert!(first.changed);
        assert_eq!(first.state.revision, 1);
        assert_eq!(
            first.state.state.objective.as_deref(),
            Some("Map the machine")
        );

        let repeated = intake_user_objective(&store, input(" Map the machine "))?;
        assert!(!repeated.changed);
        assert_eq!(repeated.intake, first.intake);

        let error = intake_user_objective(&store, input("Replace the machine"))
            .expect_err("objective replacement must be refused");
        assert!(error.to_string().contains("refusing to replace"));
        Ok(())
    }
}
