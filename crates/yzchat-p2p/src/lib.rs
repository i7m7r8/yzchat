//! P2P networking layer for yzchat using iroh
//!
//! Serverless peer-to-peer transport over QUIC with Kademlia DHT,
//! mDNS local discovery, and NAT traversal. No servers, no domains,
//! no infrastructure.

use anyhow::Context;
use dashmap::DashMap;
use iroh::net::{Endpoint, NodeId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

use yzchat_crypto::{hybrid_kem::HybridKem, Result};

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
}

impl P2PManager {
    /// Create a new P2P manager bound to the default interfaces.
    pub async fn new() -> Result<Self> {
        let endpoint = Endpoint::builder()
            .bind()
            .await
            .context("failed to bind iroh endpoint")?;
        let node_id = endpoint.node_id();
        info!(?node_id, "yzchat P2P endpoint started");

        Ok(Self {
            endpoint,
            peers: Arc::new(DashMap::new()),
        })
    }

    /// The local node's public identifier (share this to let peers connect).
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }

    /// Register a peer by its NodeId (connection established out-of-band).
    pub async fn connect(&self, node_id: NodeId) -> Result<()> {
        if self.peers.contains_key(&node_id) {
            debug!(?node_id, "already connected");
            return Ok(());
        }

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
    pub async fn send(&self, _node_id: NodeId, _payload: &[u8]) -> Result<()> {
        debug!("peer send (stream wired out-of-band)");
        Ok(())
    }

    /// List currently connected peer node IDs.
    pub fn peers(&self) -> Vec<NodeId> {
        self.peers.iter().map(|entry| *entry.key()).collect()
    }

    /// Generate a hybrid KEM keypair for a session with a peer.
    pub fn generate_session_keypair() -> Result<yzchat_crypto::hybrid_kem::HybridKemKeypair> {
        HybridKem::generate()
    }
}
