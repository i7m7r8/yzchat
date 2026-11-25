//! Document Store: Syncable data structures for contacts, messages, groups

use crate::{NodeId, P2pError, Result};
use anyhow::Context;
use chrono::{DateTime, Utc};
use iroh::Endpoint;
use iroh_docs::protocol::DocsProtocol;
use iroh_docs::rpc::client::docs::DocsClient;
use iroh_docs::rpc::server::docs::DocsServer;
use iroh_docs::store::fs::FsStore;
use iroh_docs::store::mem::MemStore;
use iroh_docs::sync::SyncMode;
use iroh_docs::{AuthorId, Doc, DocId, DocTicket, Engine, Store as DocsStore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Document store configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DocConfig {
    /// Store path (None = in-memory)
    pub store_path: Option<PathBuf>,
    /// Enable sync with peers
    pub enable_sync: bool,
    /// Sync interval (seconds)
    pub sync_interval_secs: u64,
    /// Max document size
    pub max_doc_size: usize,
}

impl Default for DocConfig {
    fn default() -> Self {
        Self {
            store_path: None,
            enable_sync: true,
            sync_interval_secs: 60,
            max_doc_size: 1024 * 1024, // 1 MB
        }
    }
}

/// Document types for yzchat
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DocType {
    /// User profile
    Profile,
    /// Contact list
    Contacts,
    /// Message history (per conversation)
    Messages { conversation_id: Uuid },
    /// Group metadata
    Group { group_id: Uuid },
    /// Group members
    GroupMembers { group_id: Uuid },
    /// Settings
    Settings,
    /// Pre-keys
    PreKeys,
    /// Device list
    Devices,
}

/// Document key
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocKey {
    pub doc_type: DocType,
    pub owner: AuthorId,
}

impl DocKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// Document store service
pub struct DocStore {
    endpoint: Arc<Endpoint>,
    config: DocConfig,
    engine: Arc<Engine>,
    docs_protocol: Option<DocsProtocol>,
    local_author: AuthorId,
    doc_handles: Arc<RwLock<HashMap<DocType, Doc>>>,
}

impl DocStore {
    /// Create new document store
    pub async fn new(endpoint: Arc<Endpoint>, config: DocConfig) -> Result<Self> {
        // Create author identity
        let author = AuthorId::generate(rand::rngs::OsRng);

        // Create store
        let store: Arc<dyn DocsStore> = if let Some(path) = &config.store_path {
            Arc::new(FsStore::new(path.clone()).await?)
        } else {
            Arc::new(MemStore::new())
        };

        // Create engine
        let engine = Engine::new(store.clone(), author);

        // Create docs protocol
        let docs = iroh_docs::Docs::new(endpoint.clone(), engine.clone());
        let docs_protocol = Some(docs.protocol());

        Ok(Self {
            endpoint,
            config,
            engine: Arc::new(engine),
            docs_protocol,
            local_author: author,
            doc_handles: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get the docs protocol for router registration
    pub fn protocol(&self) -> Option<&DocsProtocol> {
        self.docs_protocol.as_ref()
    }

    /// Get local author ID
    pub fn author_id(&self) -> AuthorId {
        self.local_author
    }

    /// Open or create a document
    pub async fn open_doc(&self, doc_type: DocType) -> Result<Doc> {
        let mut handles = self.doc_handles.write().await;

        if let Some(doc) = handles.get(&doc_type) {
            return Ok(doc.clone());
        }

        // Create document ID from type and author
        let doc_id = DocId::from_bytes(&Self::doc_id_bytes(&doc_type, self.local_author))?;
        let doc = self.engine.create(doc_id).await?;

        handles.insert(doc_type.clone(), doc.clone());
        Ok(doc)
    }

    /// Get document ID bytes for a type
    fn doc_id_bytes(doc_type: &DocType, author: AuthorId) -> [u8; 32] {
        use blake3;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"yzchat/doc/");
        hasher.update(&serde_json::to_vec(doc_type).unwrap());
        hasher.update(author.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// Set a key-value in a document
    pub async fn set(&self, doc_type: DocType, key: &str, value: &[u8]) -> Result<()> {
        let doc = self.open_doc(doc_type).await?;
        doc.set_bytes(self.local_author, key.as_bytes(), value)
            .await?;
        Ok(())
    }

    /// Get a value from a document
    pub async fn get(&self, doc_type: DocType, key: &str) -> Result<Option<Vec<u8>>> {
        let doc = self.open_doc(doc_type).await?;
        Ok(doc.get(key.as_bytes()).await?)
    }

    /// Delete a key from a document
    pub async fn delete(&self, doc_type: DocType, key: &str) -> Result<()> {
        let doc = self.open_doc(doc_type).await?;
        doc.del(self.local_author, key.as_bytes()).await?;
        Ok(())
    }

    /// Get all keys in a document
    pub async fn keys(&self, doc_type: DocType) -> Result<Vec<String>> {
        let doc = self.open_doc(doc_type).await?;
        let entries = doc.get_many(vec![]).await?;
        Ok(entries
            .into_iter()
            .map(|e| String::from_utf8_lossy(e.key()).to_string())
            .collect())
    }

    /// Sync a document with a peer
    pub async fn sync_with_peer(&self, peer: NodeId, doc_type: DocType) -> Result<()> {
        if !self.config.enable_sync {
            return Ok(());
        }

        let doc = self.open_doc(doc_type).await?;
        let ticket = doc.share(SyncMode::Full).await?;

        // Send ticket to peer via RPC
        let client = DocsClient::new(self.endpoint.clone());
        client.receive(peer, ticket).await?;

        Ok(())
    }

    /// Start periodic sync
    pub fn start_sync_task(&self) -> tokio::task::JoinHandle<()> {
        let engine = self.engine.clone();
        let endpoint = self.endpoint.clone();
        let config = self.config.clone();
        let doc_handles = self.doc_handles.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(config.sync_interval_secs));

            loop {
                interval.tick().await;
                // Sync all open docs with connected peers
                // Real impl would track peers and sync selectively
                debug!("Periodic doc sync tick");
            }
        })
    }

    /// Export document to JSON
    pub async fn export_doc(&self, doc_type: DocType) -> Result<serde_json::Value> {
        let doc = self.open_doc(doc_type).await?;
        let entries = doc.get_many(vec![]).await?;
        let mut map = serde_json::Map::new();
        for entry in entries {
            let key = String::from_utf8_lossy(entry.key()).to_string();
            let value = serde_json::from_slice(entry.value()).unwrap_or(serde_json::Value::String(
                String::from_utf8_lossy(entry.value()).to_string(),
            ));
            map.insert(key, value);
        }
        Ok(serde_json::Value::Object(map))
    }

    /// Import document from JSON
    pub async fn import_doc(&self, doc_type: DocType, data: serde_json::Value) -> Result<()> {
        let doc = self.open_doc(doc_type).await?;
        if let serde_json::Value::Object(map) = data {
            for (key, value) in map {
                let bytes = serde_json::to_vec(&value)?;
                doc.set_bytes(self.local_author, key.as_bytes(), &bytes)
                    .await?;
            }
        }
        Ok(())
    }
}
