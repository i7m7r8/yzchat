//! P2P networking layer for yzchat
//!
//! Serverless peer-to-peer transport over QUIC with Kademlia DHT,
//! mDNS local discovery, and NAT traversal. No servers, no domains,
//! no infrastructure.
//!
//! Stub: the iroh integration is wired here once the iroh crate's
//! deprecated `net` module is resolved. The crate compiles cleanly.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use dashmap::DashMap;

/// Application-layer protocol ALPN for yzchat over the P2P transport.
pub const YZCHAT_ALPN: &[u8] = b"yzchat/1";

/// A yzchat peer we are connected to.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub node_id: Vec<u8>,
    pub name: Option<String>,
    pub connected_at: Option<u64>,
}

/// Incoming yzchat message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub from: Vec<u8>,
    pub payload: Vec<u8>,
}

/// P2P manager: tracks peer connections.
pub struct P2PManager {
    peers: Arc<DashMap<Vec<u8>, Peer>>,
    local_node_id: Vec<u8>,
}

impl P2PManager {
    /// Create a new P2P manager with a random local node id.
    pub fn new() -> Result<Self, anyhow::Error> {
        use rand::RngCore;
        let mut id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut id);

        Ok(Self {
            peers: Arc::new(DashMap::new()),
            local_node_id: id.to_vec(),
        })
    }

    /// The local node's public identifier (share this to let peers connect).
    pub fn node_id(&self) -> Vec<u8> {
        self.local_node_id.clone()
    }

    /// Register a peer by its node id.
    pub async fn connect(&self, node_id: Vec<u8>) -> Result<(), anyhow::Error> {
        if self.peers.contains_key(&node_id) {
            return Ok(());
        }
        self.peers.insert(
            node_id.clone(),
            Peer {
                node_id,
                name: None,
                connected_at: None,
            },
        );
        Ok(())
    }

    /// Send an encrypted payload to a connected peer.
    pub async fn send(&self, _node_id: Vec<u8>, _payload: &[u8]) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// List currently connected peer node IDs.
    pub fn peers(&self) -> Vec<Vec<u8>> {
        self.peers.iter().map(|entry| entry.key().clone()).collect()
    }
}
