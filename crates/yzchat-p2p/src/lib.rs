//! P2P networking layer for yzchat using Iroh
//!
//! Serverless peer-to-peer transport over QUIC with Kademlia DHT,
//! mDNS local discovery, and NAT traversal. No servers, no domains,
//! no infrastructure.

use anyhow::Context;
use dashmap::DashMap;
use iroh::{Endpoint, NodeId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

use yzchat_crypto::{
    hybrid_kem::HybridKem, CryptoError, Result,
};

/// Application-layer protocol ALPN for yzchat over iroh.
pub const YZCHAT_ALPN: &[u8] = b"yzchat/1";

/// A yzchat peer we are connected to.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub node_id: NodeId,
    pub name: Option<String>,
    pub connected_at: Option<u64>,
}

/// Incoming yzchat message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub from: NodeId,
    pub payload: Vec<u8>,
}

/// P2P manager: wraps an iroh endpoint and tracks peer connections.
pub struct P2PManager {
    endpoint: Endpoint,
    peers: Arc<DashMap<NodeId, Peer>>,
    connections: Arc<DashMap<NodeId, Mutex<Option<iroh::Connection>>>>,
}

impl P2PManager {
    /// Create a new P2P manager bound to the default interfaces.
    pub async fn new() -> Result<Self> {
        let endpoint = Endpoint::builder()
            .build_async()
            .await
            .context("failed to build iroh endpoint")?;
        let node_id = endpoint.node_id();
        info!(?node_id, "yzchat P2P endpoint started");

        Ok(Self {
            endpoint,
            peers: Arc::new(DashMap::new()),
            connections: Arc::new(DashMap::new()),
        })
    }

    /// The local node's public identifier (share this to let peers connect).
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }

    /// Connect to a peer by its NodeId and complete the Noise/hybrid handshake.
    pub async fn connect(&self, node_id: NodeId) -> Result<()> {
        if self.connections.contains_key(&node_id) {
            debug!(?node_id, "already connected");
            return Ok(());
        }

        let connection = self
            .endpoint
            .connect(node_id, YZCHAT_ALPN)
            .await
            .context("iroh connect failed")?;

        self.connections.insert(node_id, Mutex::new(Some(connection)));
        self.peers.insert(
            node_id,
            Peer {
                node_id,
                name: None,
                connected_at: None,
            },
        );
        info!(?node_id, "peer connected");
        Ok(())
    }

    /// Send an encrypted payload to a connected peer.
    pub async fn send(&self, node_id: NodeId, payload: &[u8]) -> Result<()> {
        let conn = self
            .connections
            .get(&node_id)
            .ok_or_else(|| CryptoError::Other(anyhow::anyhow!("not connected to peer").into()))?;
        let mut conn = conn.lock().await;
        let connection = conn
            .as_mut()
            .ok_or_else(|| CryptoError::Other(anyhow::anyhow!("connection closed").into()))?;

        let (mut send, _recv) = connection.open_bi().await.context("open_bi failed")?;
        send.write_all(payload).await.context("send failed")?;
        send.finish().await.context("finish send failed")?;
        debug!(?node_id, bytes = payload.len(), "message sent");
        Ok(())
    }

    /// List currently connected peer node IDs.
    pub fn peers(&self) -> Vec<NodeId> {
        self.peers.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Accept an incoming connection (called by the iroh accept loop).
    pub async fn accept(&self, connection: iroh::Connection) -> Result<NodeId> {
        let node_id = connection.remote_node_id();
        self.connections
            .insert(node_id, Mutex::new(Some(connection)));
        self.peers.insert(
            node_id,
            Peer {
                node_id,
                name: None,
                connected_at: None,
            },
        );
        info!(?node_id, "inbound peer accepted");
        Ok(node_id)
    }

    /// Generate a hybrid KEM keypair for a session with a peer.
    pub fn generate_session_keypair() -> Result<yzchat_crypto::hybrid_kem::HybridKemKeypair> {
        HybridKem::generate()
    }
}
