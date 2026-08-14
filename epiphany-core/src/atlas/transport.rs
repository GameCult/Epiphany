use std::net::{SocketAddr, UdpSocket};
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use cultmesh_rs::{
    CULTMESH_RUDP_DOCUMENT_CATALOG_CONNECTION_ID, CultMesh, CultMeshNodeOptions,
    CultMeshRudpDocumentPublishOptions,
};
use cultnet_rs::{
    CultNetMessage, CultNetRawPayloadEncoding, CultNetRudpSocketTransportConnection,
    CultNetRudpSocketTransportOptions, CultNetWireContract, decode_cultnet_message_from_slice,
    encode_cultnet_message_to_vec,
};
use uuid::Uuid;

use super::{
    ATLAS_MAX_PUBLICATION_BYTES, ATLAS_PROJECTION_SCHEMA, ATLAS_PUBLICATION_SCHEMA,
    AtlasEntanglementProjection, AtlasEveDocuments, AtlasPublicationEnvelope,
    EVE_PROVIDER_ADVERTISEMENT_SCHEMA, EVE_SURFACE_SCHEMA, EveProviderAdvertisement,
    EveSurfaceDocument, MODEL_ATLAS_PROVIDER_ID, MODEL_ATLAS_SURFACE_ID,
};

cultmesh_rs::cultmesh_documents!(AtlasFederationDocuments {
    AtlasPublicationEnvelope => ATLAS_PUBLICATION_SCHEMA,
    AtlasEntanglementProjection => ATLAS_PROJECTION_SCHEMA,
    EveProviderAdvertisement => EVE_PROVIDER_ADVERTISEMENT_SCHEMA,
    EveSurfaceDocument => EVE_SURFACE_SCHEMA,
});

pub const ATLAS_RUDP_TRANSPORT_TAG: &str = "gamecult.model.atlas-rudp.v0";

pub fn persist_atlas_publications(
    local_store: &Path,
    runtime_id: &str,
    publications: &[AtlasPublicationEnvelope],
) -> Result<()> {
    require_id(runtime_id, "Atlas publication runtime")?;
    let mut node = CultMesh::create_node(
        local_store,
        AtlasFederationDocuments,
        CultMeshNodeOptions {
            runtime_id: runtime_id.into(),
            pull_on_start: true,
        },
    )?;
    for publication in publications {
        if rmp_serde::to_vec(publication)?.len() > ATLAS_MAX_PUBLICATION_BYTES {
            bail!("Atlas publication exceeds the transport size limit")
        }
        node.put(publication.statement.publication_id.clone(), publication)?;
    }
    node.flush()
}

pub fn publish_atlas_publications_rudp(
    local_store: &Path,
    endpoint: SocketAddr,
    runtime_id: &str,
    source_agent_id: &str,
    publications: &[AtlasPublicationEnvelope],
) -> Result<()> {
    require_id(runtime_id, "Atlas publication runtime")?;
    require_id(source_agent_id, "Atlas publication agent")?;
    let node = CultMesh::create_node(
        local_store,
        AtlasFederationDocuments,
        CultMeshNodeOptions {
            runtime_id: runtime_id.into(),
            pull_on_start: true,
        },
    )?;
    for publication in publications {
        node.publish_document_to_rudp_catalog(
            publication.statement.publication_id.clone(),
            publication,
            CultMeshRudpDocumentPublishOptions {
                target: endpoint,
                runtime_id: runtime_id.into(),
                source_agent_id: Some(source_agent_id.into()),
                source_role: Some("epiphany-atlas-publisher".into()),
                tags: vec![ATLAS_RUDP_TRANSPORT_TAG.into()],
                ..CultMeshRudpDocumentPublishOptions::default()
            },
        )?;
    }
    Ok(())
}

pub fn persist_atlas_projection(
    projector_store: &Path,
    runtime_id: &str,
    projection: &AtlasEntanglementProjection,
) -> Result<()> {
    require_id(runtime_id, "Atlas projector runtime")?;
    let mut node = CultMesh::create_node(
        projector_store,
        AtlasFederationDocuments,
        CultMeshNodeOptions {
            runtime_id: runtime_id.into(),
            pull_on_start: true,
        },
    )?;
    node.put(projection.audience_id.clone(), projection)?;
    node.flush()
}

pub fn persist_atlas_projection_and_eve(
    projector_store: &Path,
    runtime_id: &str,
    projection: &AtlasEntanglementProjection,
    eve: &AtlasEveDocuments,
) -> Result<()> {
    require_id(runtime_id, "Atlas projector runtime")?;
    let mut node = CultMesh::create_node(
        projector_store,
        AtlasFederationDocuments,
        CultMeshNodeOptions {
            runtime_id: runtime_id.into(),
            pull_on_start: true,
        },
    )?;
    node.put(projection.audience_id.clone(), projection)?;
    node.put(MODEL_ATLAS_PROVIDER_ID, &eve.advertisement)?;
    node.put(MODEL_ATLAS_SURFACE_ID, &eve.surface)?;
    node.flush()
}

pub fn publish_atlas_projection_and_eve_rudp(
    projector_store: &Path,
    endpoint: SocketAddr,
    runtime_id: &str,
    source_agent_id: &str,
    projection: &AtlasEntanglementProjection,
    eve: &AtlasEveDocuments,
) -> Result<()> {
    require_id(runtime_id, "Atlas projector runtime")?;
    require_id(source_agent_id, "Atlas projector agent")?;
    let node = CultMesh::create_node(
        projector_store,
        AtlasFederationDocuments,
        CultMeshNodeOptions {
            runtime_id: runtime_id.into(),
            pull_on_start: true,
        },
    )?;
    let options = |role: &str| CultMeshRudpDocumentPublishOptions {
        target: endpoint,
        runtime_id: runtime_id.into(),
        source_agent_id: Some(source_agent_id.into()),
        source_role: Some(role.into()),
        tags: vec![ATLAS_RUDP_TRANSPORT_TAG.into()],
        ..CultMeshRudpDocumentPublishOptions::default()
    };
    node.publish_document_to_rudp_catalog(
        projection.audience_id.clone(),
        projection,
        options("epiphany-model-entanglement-projector"),
    )?;
    node.publish_document_to_rudp_catalog(
        MODEL_ATLAS_PROVIDER_ID,
        &eve.advertisement,
        options("epiphany-model-atlas-eve-provider"),
    )?;
    node.publish_document_to_rudp_catalog(
        MODEL_ATLAS_SURFACE_ID,
        &eve.surface,
        options("epiphany-model-atlas-eve-surface"),
    )?;
    Ok(())
}

pub fn load_atlas_projection(
    projector_store: &Path,
    audience_id: &str,
) -> Result<Option<AtlasEntanglementProjection>> {
    let node = CultMesh::create_node(
        projector_store,
        AtlasFederationDocuments,
        CultMeshNodeOptions::default(),
    )?;
    node.get(audience_id)
}

pub fn query_atlas_publications_rudp(
    endpoint: SocketAddr,
    runtime_id: &str,
) -> Result<Vec<AtlasPublicationEnvelope>> {
    require_id(runtime_id, "Atlas projector query runtime")?;
    let message_id = format!("atlas-snapshot-{}", Uuid::new_v4());
    let response = exchange_catalog(
        endpoint,
        runtime_id,
        CultNetMessage::SnapshotRequest {
            message_id: message_id.clone(),
            schema_ids: Some(vec![ATLAS_PUBLICATION_SCHEMA.into()]),
            record_keys: None,
        },
    )?;
    let CultNetMessage::SnapshotResponseRaw {
        message_id: response_id,
        documents,
    } = response
    else {
        bail!("Odin Atlas query did not return a raw typed snapshot")
    };
    if response_id != message_id {
        bail!("Odin Atlas snapshot response substituted its request identity")
    }
    let mut publications = Vec::with_capacity(documents.len());
    for document in documents {
        if document.schema_id != ATLAS_PUBLICATION_SCHEMA
            || document.payload_encoding != CultNetRawPayloadEncoding::Messagepack
            || document.payload.len() > ATLAS_MAX_PUBLICATION_BYTES
        {
            bail!("Odin Atlas snapshot contained an unknown, non-MessagePack, or oversized record")
        }
        let publication: AtlasPublicationEnvelope = rmp_serde::from_slice(&document.payload)
            .context("Odin Atlas publication payload is malformed")?;
        if rmp_serde::to_vec(&publication)? != document.payload
            || publication.statement.publication_id != document.record_key
        {
            bail!("Odin Atlas publication payload is noncanonical or key-substituted")
        }
        publications.push(publication);
    }
    publications.sort_by(|left, right| {
        left.statement
            .publication_id
            .cmp(&right.statement.publication_id)
    });
    Ok(publications)
}

fn exchange_catalog(
    endpoint: SocketAddr,
    runtime_id: &str,
    request: CultNetMessage,
) -> Result<CultNetMessage> {
    let bind = if endpoint.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind)?;
    socket.set_read_timeout(Some(Duration::from_millis(50)))?;
    let mut transport =
        CultNetRudpSocketTransportConnection::new(CultNetRudpSocketTransportOptions::client(
            runtime_id,
            socket,
            endpoint,
            CULTMESH_RUDP_DOCUMENT_CATALOG_CONNECTION_ID,
        ))?;
    transport.connect(Vec::new())?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while !transport.connected() {
        let _ = transport.receive_once()?;
        transport.poll_resends()?;
        if Instant::now() >= deadline {
            bail!("Odin Atlas catalog handshake timed out")
        }
    }
    transport.send(
        "schema",
        encode_cultnet_message_to_vec(&request, CultNetWireContract::CultNetSchemaV0)?,
    )?;
    loop {
        if let Some(frame) = transport.receive_once()? {
            if frame.channel_id != "schema" {
                bail!("Odin Atlas catalog replied on an unknown channel")
            }
            return decode_cultnet_message_from_slice(
                &frame.payload,
                CultNetWireContract::CultNetSchemaV0,
            );
        }
        transport.poll_resends()?;
        if Instant::now() >= deadline {
            bail!("Odin Atlas catalog snapshot timed out")
        }
    }
}

fn require_id(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} is empty")
    }
    Ok(())
}
