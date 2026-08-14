use anyhow::{Result, bail};
use cultnet_rs::ServiceIdentitySigner;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::contracts::*;
use super::identity::{
    AtlasRepositorySigningIdentity, atlas_source_payload_msgpack, canonical_atlas_payload_msgpack,
    sha256, sign_atlas_publication,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasLocalMindDocument {
    pub payload: AtlasPublicationPayload,
    pub source: AtlasMindSourceVersion,
    pub mind_commit: AtlasMindCommitBinding,
}

impl AtlasLocalMindDocument {
    fn validate(&self, publisher: &AtlasRepositoryIdentity) -> Result<String> {
        self.payload.validate()?;
        self.source.validate()?;
        self.mind_commit.validate()?;
        if !self.payload.requires_mind_binding()
            || self.payload.owner() != publisher
            || self.source != self.mind_commit.source
            || self.source.document_type != self.payload.schema()
            || self.source.schema_id.as_deref() != Some(self.payload.schema())
            || self.source.document_key != self.payload.key()
        {
            bail!("Atlas local publication input is not an exact owner/Mind source/commit binding")
        }
        let digest = sha256(&atlas_source_payload_msgpack(&self.payload)?);
        if self.source.payload_sha256 != digest {
            bail!("Atlas local Mind source digest does not match its CultCache payload")
        }
        Ok(digest)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasLatestPublicationPointer {
    pub publisher: AtlasRepositoryIdentity,
    pub source_schema: String,
    pub source_key: String,
    pub source_payload_sha256: String,
    pub publication_id: String,
    pub publication_sequence: u64,
}

impl AtlasLatestPublicationPointer {
    pub fn validate(&self) -> Result<()> {
        self.publisher.validate()?;
        validate_identifier(&self.source_schema, "Atlas latest pointer source schema")?;
        validate_identifier(&self.source_key, "Atlas latest pointer source key")?;
        validate_sha256(
            &self.source_payload_sha256,
            "Atlas latest pointer source payload digest",
        )?;
        validate_sha256(&self.publication_id, "Atlas latest pointer publication id")?;
        if self.publication_sequence == 0 {
            bail!("Atlas latest pointer sequence must be positive")
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasLatestPublicationPointerIntent {
    pub intent_id: String,
    pub publisher: AtlasRepositoryIdentity,
    pub source_schema: String,
    pub source_key: String,
    pub expected_current_publication_id: Option<String>,
    pub next: AtlasLatestPublicationPointer,
}

impl AtlasLatestPublicationPointerIntent {
    pub fn validate(&self) -> Result<()> {
        validate_sha256(&self.intent_id, "Atlas latest pointer intent id")?;
        self.publisher.validate()?;
        validate_identifier(
            &self.source_schema,
            "Atlas latest pointer intent source schema",
        )?;
        validate_identifier(&self.source_key, "Atlas latest pointer intent source key")?;
        if let Some(publication_id) = &self.expected_current_publication_id {
            validate_sha256(
                publication_id,
                "Atlas latest pointer expected publication id",
            )?;
        }
        self.next.validate()?;
        if self.publisher != self.next.publisher
            || self.source_schema != self.next.source_schema
            || self.source_key != self.next.source_key
        {
            bail!("Atlas latest pointer intent substituted its target identity")
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasLocalPublicationSnapshot {
    pub publisher: AtlasRepositoryIdentity,
    pub current_mind_documents: Vec<AtlasLocalMindDocument>,
    pub latest_pointers: Vec<AtlasLatestPublicationPointer>,
}

/// The publisher can see only the local repository snapshot passed through this
/// port. There is deliberately no path, workspace id, or foreign-store method.
pub trait AtlasLocalPublicationStore {
    fn load_local_atlas_snapshot(&self) -> Result<AtlasLocalPublicationSnapshot>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasPublisherContext {
    pub runtime_id: String,
    pub runtime_incarnation_id: String,
    pub body_basis: crate::RepositoryBodyObservationBasis,
    pub verse_id: String,
    pub heartbeat_sequence: u64,
    pub heartbeat_at_unix_ms: u64,
    pub publisher_state: AtlasPublisherState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasPublicationBatch {
    pub publications: Vec<AtlasPublicationEnvelope>,
    pub latest_pointer_intents: Vec<AtlasLatestPublicationPointerIntent>,
}

pub fn publish_local_atlas_state(
    store: &impl AtlasLocalPublicationStore,
    signer: &ServiceIdentitySigner<AtlasRepositorySigningIdentity>,
    context: &AtlasPublisherContext,
) -> Result<AtlasPublicationBatch> {
    let mut snapshot = store.load_local_atlas_snapshot()?;
    snapshot.publisher.validate()?;
    validate_context(&snapshot.publisher, context)?;

    let mut latest = BTreeMap::<(String, String), AtlasLatestPublicationPointer>::new();
    let mut maximum_sequence = 0_u64;
    for pointer in snapshot.latest_pointers.drain(..) {
        pointer.validate()?;
        if pointer.publisher != snapshot.publisher {
            bail!("Atlas local store returned a foreign latest pointer")
        }
        maximum_sequence = maximum_sequence.max(pointer.publication_sequence);
        let key = (pointer.source_schema.clone(), pointer.source_key.clone());
        if latest.insert(key, pointer).is_some() {
            bail!("Atlas local store returned duplicate latest pointer identities")
        }
    }

    let mut current = BTreeMap::<(String, String), (AtlasLocalMindDocument, String)>::new();
    for document in snapshot.current_mind_documents.drain(..) {
        let digest = document.validate(&snapshot.publisher)?;
        let key = (document.payload.schema().into(), document.payload.key());
        if current.insert(key, (document, digest)).is_some() {
            bail!("Atlas local Mind snapshot returned duplicate semantic documents")
        }
    }

    let mut publications = Vec::new();
    let mut latest_pointer_intents = Vec::new();
    let mut watermarks = Vec::with_capacity(current.len());
    for ((source_schema, source_key), (document, payload_sha256)) in current {
        let previous = latest.get(&(source_schema.clone(), source_key.clone()));
        let pointer =
            if previous.is_some_and(|pointer| pointer.source_payload_sha256 == payload_sha256) {
                previous.cloned().expect("checked above")
            } else {
                maximum_sequence = maximum_sequence
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("Atlas publication sequence exhausted"))?;
                let publication = sign_atlas_publication(
                    signer,
                    snapshot.publisher.clone(),
                    maximum_sequence,
                    context.runtime_id.clone(),
                    context.runtime_incarnation_id.clone(),
                    context.body_basis.clone(),
                    context.verse_id.clone(),
                    Some(document.source),
                    Some(document.mind_commit),
                    context.heartbeat_at_unix_ms,
                    document.payload,
                )?;
                let pointer = AtlasLatestPublicationPointer {
                    publisher: snapshot.publisher.clone(),
                    source_schema: source_schema.clone(),
                    source_key: source_key.clone(),
                    source_payload_sha256: payload_sha256.clone(),
                    publication_id: publication.statement.publication_id.clone(),
                    publication_sequence: maximum_sequence,
                };
                latest_pointer_intents.push(pointer_intent(previous, pointer.clone())?);
                publications.push(publication);
                pointer
            };
        watermarks.push(AtlasPublicationWatermark {
            source_schema,
            source_key,
            source_payload_sha256: payload_sha256,
            publication_sequence: pointer.publication_sequence,
        });
    }

    let status = AtlasPublisherStatus {
        publisher: snapshot.publisher.clone(),
        runtime_id: context.runtime_id.clone(),
        runtime_incarnation_id: context.runtime_incarnation_id.clone(),
        heartbeat_sequence: context.heartbeat_sequence,
        heartbeat_at_unix_ms: context.heartbeat_at_unix_ms,
        state: context.publisher_state,
        watermarks,
    };
    status.validate()?;
    let status_payload = AtlasPublicationPayload::PublisherStatus(status);
    let status_schema = status_payload.schema().to_string();
    let status_key = status_payload.key();
    let status_sha256 = sha256(&canonical_atlas_payload_msgpack(&status_payload)?);
    let previous_status = latest.get(&(status_schema.clone(), status_key.clone()));
    if !previous_status.is_some_and(|pointer| pointer.source_payload_sha256 == status_sha256) {
        maximum_sequence = maximum_sequence
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("Atlas publication sequence exhausted"))?;
        let publication = sign_atlas_publication(
            signer,
            snapshot.publisher.clone(),
            maximum_sequence,
            context.runtime_id.clone(),
            context.runtime_incarnation_id.clone(),
            context.body_basis.clone(),
            context.verse_id.clone(),
            None,
            None,
            context.heartbeat_at_unix_ms,
            status_payload,
        )?;
        let pointer = AtlasLatestPublicationPointer {
            publisher: snapshot.publisher,
            source_schema: status_schema,
            source_key: status_key,
            source_payload_sha256: status_sha256,
            publication_id: publication.statement.publication_id.clone(),
            publication_sequence: maximum_sequence,
        };
        latest_pointer_intents.push(pointer_intent(previous_status, pointer)?);
        publications.push(publication);
    }

    Ok(AtlasPublicationBatch {
        publications,
        latest_pointer_intents,
    })
}

fn validate_context(
    publisher: &AtlasRepositoryIdentity,
    context: &AtlasPublisherContext,
) -> Result<()> {
    validate_identifier(&context.runtime_id, "Atlas publisher runtime id")?;
    validate_identifier(
        &context.runtime_incarnation_id,
        "Atlas publisher runtime incarnation id",
    )?;
    validate_identifier(&context.verse_id, "Atlas publisher Verse id")?;
    if context.heartbeat_sequence == 0
        || context.heartbeat_at_unix_ms == 0
        || context.body_basis.swarm_id != publisher.swarm_id
        || context.body_basis.workspace_id != publisher.workspace_id
        || context.body_basis.runtime_id != context.runtime_id
        || context.body_basis.generation == 0
    {
        bail!("Atlas publisher context is not bound to the local repository Body")
    }
    Ok(())
}

fn pointer_intent(
    previous: Option<&AtlasLatestPublicationPointer>,
    next: AtlasLatestPublicationPointer,
) -> Result<AtlasLatestPublicationPointerIntent> {
    next.validate()?;
    let expected_current_publication_id = previous.map(|pointer| pointer.publication_id.clone());
    let intent_id = sha256(&rmp_serde::to_vec_named(&(
        "gamecult.model.atlas-latest-pointer-intent.v0",
        &next.publisher,
        &next.source_schema,
        &next.source_key,
        &expected_current_publication_id,
        &next.publication_id,
        next.publication_sequence,
    ))?);
    let intent = AtlasLatestPublicationPointerIntent {
        intent_id,
        publisher: next.publisher.clone(),
        source_schema: next.source_schema.clone(),
        source_key: next.source_key.clone(),
        expected_current_publication_id,
        next,
    };
    intent.validate()?;
    Ok(intent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultnet_rs::enroll_service_identity_at;
    use semver::Version;
    use uuid::Uuid;

    #[derive(Clone)]
    struct MemoryStore(AtlasLocalPublicationSnapshot);

    impl AtlasLocalPublicationStore for MemoryStore {
        fn load_local_atlas_snapshot(&self) -> Result<AtlasLocalPublicationSnapshot> {
            Ok(self.0.clone())
        }
    }

    fn digest(seed: &str) -> String {
        sha256(seed.as_bytes())
    }

    fn body_digest(seed: &str) -> String {
        digest(seed).trim_start_matches("sha256-").to_owned()
    }

    fn repository() -> AtlasRepositoryIdentity {
        AtlasRepositoryIdentity::new("swarm", "workspace").unwrap()
    }

    fn body() -> crate::RepositoryBodyObservationBasis {
        crate::RepositoryBodyObservationBasis {
            schema_version: "gamecult.epiphany.repository_body_observation_basis.v0".into(),
            workspace_id: "workspace".into(),
            swarm_id: "swarm".into(),
            runtime_id: "runtime".into(),
            scope: "whole_repository".into(),
            body_binding_sha256: body_digest("body-binding"),
            observation_id: "body-observation".into(),
            generation: 1,
            manifest_root_sha256: body_digest("manifest"),
            scan_started_at: "2026-08-14T00:00:00Z".into(),
            scan_finished_at: "2026-08-14T00:00:01Z".into(),
        }
    }

    fn context() -> AtlasPublisherContext {
        AtlasPublisherContext {
            runtime_id: "runtime".into(),
            runtime_incarnation_id: "incarnation".into(),
            body_basis: body(),
            verse_id: "cultmesh://gamecult-local/swarm/atlas".into(),
            heartbeat_sequence: 1,
            heartbeat_at_unix_ms: 1_800_000_000_000,
            publisher_state: AtlasPublisherState::Serving,
        }
    }

    fn local_offer(owner: AtlasRepositoryIdentity) -> AtlasLocalMindDocument {
        let payload = AtlasPublicationPayload::SurfaceOffer(AtlasSurfaceOffer {
            schema_version: ATLAS_SURFACE_OFFER_SCHEMA.into(),
            provider: owner,
            surface_id: Uuid::from_u128(1),
            contract: AtlasContractDescriptor::Semver {
                contract_id: "contract.surface".into(),
                version: Version::new(1, 0, 0),
            },
            lifecycle: AtlasOfferLifecycle::Active,
            label: "Published surface".into(),
            body_evidence: vec![AtlasBodyEvidenceRef {
                path: "Cargo.toml".into(),
                raw_sha256: "0".repeat(64),
            }],
        });
        let payload_sha256 = sha256(&atlas_source_payload_msgpack(&payload).unwrap());
        let source = AtlasMindSourceVersion {
            store_id: "epiphany-mind".into(),
            document_type: payload.schema().into(),
            document_key: payload.key(),
            schema_id: Some(payload.schema().into()),
            payload_sha256,
        };
        AtlasLocalMindDocument {
            payload,
            mind_commit: AtlasMindCommitBinding {
                receipt_id: "mind-receipt".into(),
                receipt_sha256: digest("mind-receipt"),
                invariant_owner: "Mind".into(),
                source: source.clone(),
            },
            source,
        }
    }

    #[test]
    fn publishes_only_local_mind_documents_and_status_with_pointer_intents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = enroll_service_identity_at::<AtlasRepositorySigningIdentity>(
            &temp.path().join("atlas-identity.cc"),
        )?;
        let owner = repository();
        let batch = publish_local_atlas_state(
            &MemoryStore(AtlasLocalPublicationSnapshot {
                publisher: owner.clone(),
                current_mind_documents: vec![local_offer(owner.clone())],
                latest_pointers: Vec::new(),
            }),
            &signer,
            &context(),
        )?;

        assert_eq!(batch.publications.len(), 2);
        assert_eq!(batch.latest_pointer_intents.len(), 2);
        assert!(batch.publications.iter().all(|publication| {
            publication.statement.publisher == owner && publication.signature.len() == 64
        }));
        assert!(matches!(
            batch.publications[0].statement.payload,
            AtlasPublicationPayload::SurfaceOffer(_)
        ));
        assert!(matches!(
            batch.publications[1].statement.payload,
            AtlasPublicationPayload::PublisherStatus(_)
        ));
        assert!(batch.latest_pointer_intents.iter().all(|intent| {
            intent.expected_current_publication_id.is_none() && intent.validate().is_ok()
        }));
        Ok(())
    }

    #[test]
    fn unchanged_local_snapshot_is_idempotent_at_the_same_heartbeat() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = enroll_service_identity_at::<AtlasRepositorySigningIdentity>(
            &temp.path().join("atlas-identity.cc"),
        )?;
        let owner = repository();
        let first = publish_local_atlas_state(
            &MemoryStore(AtlasLocalPublicationSnapshot {
                publisher: owner.clone(),
                current_mind_documents: vec![local_offer(owner.clone())],
                latest_pointers: Vec::new(),
            }),
            &signer,
            &context(),
        )?;
        let latest_pointers = first
            .latest_pointer_intents
            .iter()
            .map(|intent| intent.next.clone())
            .collect();
        let repeated = publish_local_atlas_state(
            &MemoryStore(AtlasLocalPublicationSnapshot {
                publisher: owner.clone(),
                current_mind_documents: vec![local_offer(owner)],
                latest_pointers,
            }),
            &signer,
            &context(),
        )?;

        assert!(repeated.publications.is_empty());
        assert!(repeated.latest_pointer_intents.is_empty());
        Ok(())
    }

    #[test]
    fn foreign_mind_document_is_refused_without_store_access() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = enroll_service_identity_at::<AtlasRepositorySigningIdentity>(
            &temp.path().join("atlas-identity.cc"),
        )?;
        let owner = repository();
        let foreign = AtlasRepositoryIdentity::new("swarm", "foreign")?;
        let result = publish_local_atlas_state(
            &MemoryStore(AtlasLocalPublicationSnapshot {
                publisher: owner,
                current_mind_documents: vec![local_offer(foreign)],
                latest_pointers: Vec::new(),
            }),
            &signer,
            &context(),
        );
        assert!(result.is_err());
        Ok(())
    }
}
