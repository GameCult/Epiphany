use anyhow::Result;
use cultcache_rs::CultCache;
use cultcache_rs::CultSoaTable;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultcache_rs::SoaDocument;
use cultnet_rs::CultNetDocumentPutOptions;
use cultnet_rs::CultNetDocumentRegistry;
use cultnet_rs::CultNetRudpSocketTransportConnection;
use cultnet_rs::CultNetRudpSocketTransportOptions;
use cultnet_rs::CultNetWireContract;
use cultnet_rs::encode_cultnet_message_to_vec;
use serde::Serialize;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::Instant;

pub const CULTMESH_RUDP_DOCUMENT_CATALOG_CONNECTION_ID: u32 = 0x0d1d_0002;

pub trait CultMeshDocumentSet: Clone + Send + Sync + 'static {
    fn register_cache(&self, cache: &mut CultCache) -> Result<()>;
    fn register_documents(&self, registry: &mut CultNetDocumentRegistry) -> Result<()>;
}

#[macro_export]
macro_rules! cultmesh_documents {
    ($name:ident { $($entry:ty => $schema_version:expr),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name;

        impl $crate::CultMeshDocumentSet for $name {
            fn register_cache(
                &self,
                cache: &mut cultcache_rs::CultCache,
            ) -> ::anyhow::Result<()> {
                $(
                    cache.register_entry_type::<$entry>()?;
                )*
                Ok(())
            }

            fn register_documents(
                &self,
                registry: &mut cultnet_rs::CultNetDocumentRegistry,
            ) -> ::anyhow::Result<()> {
                $(
                    registry.register(
                        cultnet_rs::CultNetDocumentBinding::for_entry_with_schema_id::<$entry>(
                            $schema_version.to_string(),
                            $schema_version.to_string(),
                        ),
                    );
                )*
                Ok(())
            }
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CultMeshNodeOptions {
    pub runtime_id: String,
    pub pull_on_start: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CultMeshRudpDocumentPublishOptions {
    pub target: SocketAddr,
    pub runtime_id: String,
    pub connection_id: u32,
    pub connect_timeout: Duration,
    pub flush_timeout: Duration,
    pub poll_interval: Duration,
    pub resend_delay_ms: u64,
    pub source_agent_id: Option<String>,
    pub source_role: Option<String>,
    pub tags: Vec<String>,
}

impl CultMeshRudpDocumentPublishOptions {
    pub fn odin(target: SocketAddr, runtime_id: impl Into<String>) -> Self {
        Self {
            target,
            runtime_id: runtime_id.into(),
            ..Self::default()
        }
    }
}

impl Default for CultMeshRudpDocumentPublishOptions {
    fn default() -> Self {
        Self {
            target: SocketAddr::from(([127, 0, 0, 1], 17871)),
            runtime_id: "cultmesh-rudp-document-publisher".to_string(),
            connection_id: CULTMESH_RUDP_DOCUMENT_CATALOG_CONNECTION_ID,
            connect_timeout: Duration::from_secs(1),
            flush_timeout: Duration::from_millis(150),
            poll_interval: Duration::from_millis(5),
            resend_delay_ms: 100,
            source_agent_id: None,
            source_role: None,
            tags: Vec::new(),
        }
    }
}

impl Default for CultMeshNodeOptions {
    fn default() -> Self {
        Self {
            runtime_id: "cultmesh-local".to_string(),
            pull_on_start: true,
        }
    }
}

pub struct CultMeshNode {
    runtime_id: String,
    cache: CultCache,
    documents: CultNetDocumentRegistry,
}

impl CultMeshNode {
    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn cache(&self) -> &CultCache {
        &self.cache
    }

    pub fn documents(&self) -> &CultNetDocumentRegistry {
        &self.documents
    }

    pub fn get<T: DatabaseEntry>(&self, key: &str) -> Result<Option<T>> {
        self.cache.get::<T>(key)
    }

    pub fn get_required<T: DatabaseEntry>(&self, key: &str) -> Result<T> {
        self.cache.get_required::<T>(key)
    }

    pub fn get_all_with_keys<T: DatabaseEntry>(&self) -> Result<Vec<(String, T)>> {
        self.cache.get_all_with_keys::<T>()
    }

    pub fn soa<T: SoaDocument>(&self) -> Result<CultSoaTable<T>> {
        self.cache.soa::<T>()
    }

    pub fn put<T: DatabaseEntry>(&mut self, key: impl Into<String>, value: &T) -> Result<T> {
        self.cache.put(key, value)
    }

    pub fn delete<T: DatabaseEntry>(&mut self, key: &str) -> Result<bool> {
        self.cache.delete::<T>(key)
    }

    pub fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn publish_document_to_rudp_catalog<T>(
        &self,
        key: impl Into<String>,
        value: &T,
        options: CultMeshRudpDocumentPublishOptions,
    ) -> Result<()>
    where
        T: DatabaseEntry + Serialize,
    {
        let key = key.into();
        let message = self.documents.create_raw_document_put_message(
            format!("{}:{}:{}", options.runtime_id, T::TYPE, key),
            key,
            value,
            CultNetDocumentPutOptions {
                source_runtime_id: Some(options.runtime_id.clone()),
                source_agent_id: options.source_agent_id.clone(),
                source_role: options.source_role.clone(),
                tags: if options.tags.is_empty() {
                    None
                } else {
                    Some(options.tags.clone())
                },
                ..CultNetDocumentPutOptions::default()
            },
        )?;
        publish_cultnet_message_to_rudp_catalog(&message, options)
    }
}

fn publish_cultnet_message_to_rudp_catalog(
    message: &cultnet_rs::CultNetMessage,
    options: CultMeshRudpDocumentPublishOptions,
) -> Result<()> {
    if options.runtime_id.trim().is_empty() {
        anyhow::bail!("runtime_id must be non-empty");
    }
    let bind_addr = if options.target.is_ipv4() {
        SocketAddr::from(([0, 0, 0, 0], 0))
    } else {
        SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], 0))
    };
    let socket = UdpSocket::bind(bind_addr)?;
    socket.set_read_timeout(Some(options.poll_interval))?;
    let mut client = CultNetRudpSocketTransportConnection::new(
        CultNetRudpSocketTransportOptions {
            runtime_id: options.runtime_id.clone(),
            socket,
            mode: cultnet_rs::CultNetRudpSocketMode::Client,
            remote_addr: Some(options.target),
            connection_id: options.connection_id,
            initial_sequence: 1,
            resend_delay_ms: options.resend_delay_ms,
            transport_id: Some("cultmesh-rudp-document-publisher".to_string()),
            max_payload_bytes: None,
            max_fragment_bytes: Some(1200),
            max_pending_reliable_packets: Some(64),
            media_reliable_expire_after_ms: None,
        },
    )?;
    client.connect(Vec::new())?;
    let connect_deadline = Instant::now() + options.connect_timeout;
    while !client.connected() && Instant::now() < connect_deadline {
        let _ = client.receive_once()?;
        client.poll_resends()?;
        thread::sleep(options.poll_interval);
    }
    if !client.connected() {
        anyhow::bail!(
            "Timed out connecting CultMesh RUDP document publisher {} to {}",
            options.runtime_id,
            options.target
        );
    }

    let payload = encode_cultnet_message_to_vec(message, CultNetWireContract::CultNetSchemaV0)?;
    client.send("schema", payload)?;
    let flush_deadline = Instant::now() + options.flush_timeout;
    while Instant::now() < flush_deadline {
        let _ = client.receive_once()?;
        client.poll_resends()?;
        thread::sleep(options.poll_interval);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CultMesh;

impl CultMesh {
    pub fn create_node<D>(
        store_path: impl AsRef<Path>,
        documents: D,
        options: CultMeshNodeOptions,
    ) -> Result<CultMeshNode>
    where
        D: CultMeshDocumentSet,
    {
        let mut cache = CultCache::new();
        documents.register_cache(&mut cache)?;
        cache
            .add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path.as_ref()));
        if options.pull_on_start {
            cache.pull_all_backing_stores()?;
        }

        let mut registry = CultNetDocumentRegistry::new();
        documents.register_documents(&mut registry)?;

        Ok(CultMeshNode {
            runtime_id: options.runtime_id,
            cache,
            documents: registry,
        })
    }

    pub fn start_node<D>(
        store_path: impl AsRef<Path>,
        documents: D,
        options: CultMeshNodeOptions,
    ) -> Result<CultMeshNode>
    where
        D: CultMeshDocumentSet,
    {
        Self::create_node(store_path, documents, options)
    }
}

pub fn create_node<D>(
    store_path: impl AsRef<Path>,
    documents: D,
    options: CultMeshNodeOptions,
) -> Result<CultMeshNode>
where
    D: CultMeshDocumentSet,
{
    CultMesh::create_node(store_path, documents, options)
}

pub fn start_node<D>(
    store_path: impl AsRef<Path>,
    documents: D,
    options: CultMeshNodeOptions,
) -> Result<CultMeshNode>
where
    D: CultMeshDocumentSet,
{
    CultMesh::start_node(store_path, documents, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;

    #[derive(Clone, Debug, PartialEq, Eq, cultcache_rs::DatabaseEntry)]
    #[cultcache(type = "cultmesh.test.note", schema = "CultMeshTestNote")]
    struct Note {
        #[cultcache(key = 0)]
        body: String,
        #[cultcache(key = 1, default)]
        owner: String,
    }

    impl cultcache_rs::SoaDocument for Note {
        fn soa_columns(rows: &[Self]) -> BTreeMap<&'static str, cultcache_rs::CultSoaColumnValues> {
            let mut columns = BTreeMap::new();
            columns.insert(
                "body",
                cultcache_rs::CultSoaColumnValues::new(
                    rows.iter().map(|row| row.body.clone()).collect::<Vec<_>>(),
                ),
            );
            columns.insert(
                "owner",
                cultcache_rs::CultSoaColumnValues::new(
                    rows.iter().map(|row| row.owner.clone()).collect::<Vec<_>>(),
                ),
            );
            columns
        }
    }

    cultmesh_documents!(TestDocuments {
        Note => "cultmesh.test.note.v0",
    });

    #[test]
    fn node_round_trips_registered_documents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("cultmesh.ccmp");
        let note = Note {
            body: "blessed circuit".to_string(),
            owner: "self".to_string(),
        };

        let mut node = CultMesh::create_node(
            &store_path,
            TestDocuments,
            CultMeshNodeOptions {
                runtime_id: "test-runtime".to_string(),
                ..CultMeshNodeOptions::default()
            },
        )?;
        assert_eq!(node.runtime_id(), "test-runtime");
        node.put("note", &note)?;
        node.flush()?;

        let reloaded = CultMesh::create_node(&store_path, TestDocuments, Default::default())?;
        assert_eq!(reloaded.get_required::<Note>("note")?, note);
        assert!(
            reloaded
                .documents()
                .binding_by_schema_id("cultmesh.test.note.v0")
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn node_publishes_registered_document_to_rudp_catalog() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("cultmesh.ccmp");
        let note = Note {
            body: "respect odin once".to_string(),
            owner: "self".to_string(),
        };
        let node = CultMesh::create_node(
            &store_path,
            TestDocuments,
            CultMeshNodeOptions {
                runtime_id: "epiphany-test".to_string(),
                ..CultMeshNodeOptions::default()
            },
        )?;

        let socket = UdpSocket::bind("127.0.0.1:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(10)))?;
        let target = socket.local_addr()?;
        let (sender, receiver) = std::sync::mpsc::channel();
        let server = thread::spawn(move || -> Result<()> {
            let mut server = cultnet_rs::CultNetRudpSocketTransportConnection::new(
                cultnet_rs::CultNetRudpSocketTransportOptions::server(
                    "odin-test-catalog",
                    socket,
                    CULTMESH_RUDP_DOCUMENT_CATALOG_CONNECTION_ID,
                ),
            )?;
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline {
                if let Some(frame) = server.receive_once()? {
                    let message = cultnet_rs::decode_cultnet_message_from_slice(
                        &frame.payload,
                        cultnet_rs::CultNetWireContract::CultNetSchemaV0,
                    )?;
                    if let cultnet_rs::CultNetMessage::DocumentPutRaw { document, .. } = message {
                        sender
                            .send((
                                document.schema_id,
                                document.record_key,
                                document.source_runtime_id,
                            ))
                            .ok();
                        return Ok(());
                    }
                }
                server.poll_resends()?;
            }
            anyhow::bail!("timed out waiting for RUDP document put")
        });

        node.publish_document_to_rudp_catalog(
            "note",
            &note,
            CultMeshRudpDocumentPublishOptions::odin(target, "epiphany-test"),
        )?;
        let received = receiver.recv_timeout(Duration::from_secs(2))?;
        server.join().expect("server thread should not panic")?;

        assert_eq!(received.0, "cultmesh.test.note.v0");
        assert_eq!(received.1, "note");
        assert_eq!(received.2.as_deref(), Some("epiphany-test"));
        Ok(())
    }

    #[test]
    fn node_projects_soa_tables_for_registered_documents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store_path = temp.path().join("cultmesh-soa.ccmp");
        let mut node = CultMesh::create_node(&store_path, TestDocuments, Default::default())?;
        node.put(
            "note/self",
            &Note {
                body: "private verse".to_string(),
                owner: "self".to_string(),
            },
        )?;
        node.put(
            "note/hands",
            &Note {
                body: "repo action".to_string(),
                owner: "hands".to_string(),
            },
        )?;
        node.flush()?;

        let table = node.soa::<Note>()?;
        assert_eq!(table.len(), 2);
        let owners = table.column::<String>("owner")?.values().to_vec();
        let bodies = table.column::<String>("body")?.values().to_vec();
        let mut rows = owners.into_iter().zip(bodies).collect::<Vec<_>>();
        rows.sort();
        assert_eq!(
            rows,
            vec![
                ("hands".to_string(), "repo action".to_string()),
                ("self".to_string(), "private verse".to_string()),
            ]
        );
        Ok(())
    }
}
