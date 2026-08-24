use crate::{
    EpiphanyMindCommitAuthority, EpiphanyMindCommitOutcome, EpiphanyMindCommitReceipt,
    EpiphanyMindDocumentVersion, EpiphanyMindObjectiveDocument, MIND_OBJECTIVE_KEY,
    assemble_mind_view, commit_operator_mind_mutation, runtime_spine_cache,
};
use anyhow::{Result, anyhow};
use cultcache_rs::DatabaseEntry;
use sha2::{Digest, Sha256};
use std::path::Path;

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
    pub thread_id: String,
    #[cultcache(key = 1)]
    pub objective: String,
    #[cultcache(key = 2)]
    pub source_actor: String,
    #[cultcache(key = 3)]
    pub source_ref: String,
    #[cultcache(key = 4)]
    pub submitted_at: String,
}

#[derive(Debug, Clone)]
pub struct UserObjectiveIntakeApplied {
    pub mind_projection_digest: String,
    pub commit_receipt: EpiphanyMindCommitReceipt,
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
    let intake_id = format!("user-objective-{:x}", Sha256::digest(thread_id.as_bytes()));

    let mut cache = runtime_spine_cache(store)?;
    cache.pull_all_backing_stores()?;
    if let Some(existing) = cache.get::<UserObjectiveIntake>(&intake_id)? {
        let objective_document = cache
            .get::<EpiphanyMindObjectiveDocument>(MIND_OBJECTIVE_KEY)?
            .ok_or_else(|| anyhow!("typed user-objective intake lost its Mind objective"))?;
        if existing.thread_id != thread_id
            || existing.objective != objective
            || existing.source_actor != source_actor
            || existing.source_ref != source_ref
            || objective_document.objective.trim() != objective
        {
            return Err(anyhow!(
                "refusing to replace the authoritative Mind objective; use a typed objective-adoption flow"
            ));
        }
        let provenance = EpiphanyMindDocumentVersion::from_envelope(
            "epiphany-operator",
            &cache.prepare_entry(&intake_id, &existing)?.0,
        )?;
        let receipt = cache
            .get_all::<EpiphanyMindCommitReceipt>()?
            .into_iter()
            .find(|receipt| {
                receipt.invariant_owner == "Self.user_objective_intake"
                    && matches!(
                        &receipt.authority,
                        EpiphanyMindCommitAuthority::OperatorProvenance {
                            provenance: receipt_provenance
                        } if receipt_provenance == &provenance
                    )
                    && receipt.writes.iter().any(|write| {
                        write.document_type == EpiphanyMindObjectiveDocument::TYPE
                            && write.document_key == MIND_OBJECTIVE_KEY
                    })
            })
            .ok_or_else(|| anyhow!("typed user-objective intake lost its Mind commit receipt"))?;
        return Ok(UserObjectiveIntakeApplied {
            mind_projection_digest: assemble_mind_view(store)?.projection_digest,
            commit_receipt: receipt,
            changed: false,
        });
    }
    if cache
        .get::<EpiphanyMindObjectiveDocument>(MIND_OBJECTIVE_KEY)?
        .is_some()
    {
        return Err(anyhow!(
            "authoritative Mind objective has no matching typed operator intake"
        ));
    }
    let intake = UserObjectiveIntake {
        thread_id: thread_id.to_string(),
        objective: objective.to_string(),
        source_actor: source_actor.to_string(),
        source_ref: source_ref.to_string(),
        submitted_at: input.submitted_at,
    };
    let provenance = cache.prepare_entry(&intake_id, &intake)?.0;
    let objective_write = crate::mind_documents::prepare_mind_document(
        &cache,
        MIND_OBJECTIVE_KEY,
        &EpiphanyMindObjectiveDocument {
            objective: objective.to_string(),
        },
    )?;
    let commit_receipt = match commit_operator_mind_mutation(
        store,
        provenance,
        "Self.user_objective_intake",
        Vec::new(),
        vec![objective_write],
        &intake.submitted_at,
    )? {
        EpiphanyMindCommitOutcome::Committed(receipt) => receipt,
        EpiphanyMindCommitOutcome::Conflict { .. } => {
            return Err(anyhow!("user-objective intake lost its atomic Mind commit"));
        }
    };
    Ok(UserObjectiveIntakeApplied {
        mind_projection_digest: assemble_mind_view(store)?.projection_digest,
        commit_receipt,
        changed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuntimeSpineInitOptions, initialize_runtime_spine};

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
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "objective-intake".into(),
                display_name: "Objective intake".into(),
                created_at: "2026-07-16T13:59:00Z".into(),
            },
        )?;
        let first = intake_user_objective(&store, input("Map the machine"))?;
        assert!(first.changed);
        assert_eq!(
            assemble_mind_view(&store)?.objective.as_deref(),
            Some("Map the machine")
        );
        assert_eq!(
            first.commit_receipt.invariant_owner,
            "Self.user_objective_intake"
        );
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert!(
            cache
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.thread_state")
        );
        let after_first = std::fs::read(&store)?;

        let repeated = intake_user_objective(&store, input(" Map the machine "))?;
        assert!(!repeated.changed);
        assert_eq!(
            repeated.mind_projection_digest,
            first.mind_projection_digest
        );
        assert_eq!(repeated.commit_receipt, first.commit_receipt);
        assert_eq!(std::fs::read(&store)?, after_first);

        let before = std::fs::read(&store)?;
        let error = intake_user_objective(&store, input("Replace the machine"))
            .expect_err("objective replacement must be refused");
        assert!(error.to_string().contains("refusing to replace"));
        assert_eq!(std::fs::read(&store)?, before);
        Ok(())
    }
}
