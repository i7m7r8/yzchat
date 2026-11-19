//! Peer Discovery: DHT + mDNS + N0

use crate::{P2pError, Result, NodeId};
use anyhow::Context;
use iroh::Endpoint;
use iroh_net::discovery::{dht::DhtDiscovery, mdns::MdnsDiscovery, n0::N0Discovery};
use iroh_net::discovery::Discovery as DiscoveryTrait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, debug, warn};

/// Discovery configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryConfig {
    /// Enable Kademlia DHT
    pub enable_dht: bool,
    /// Enable mDNS (local network)
    pub enable_mdns: bool,
    /// Enable N0 (global discovery)
    pub enable_n0: bool,
    /// DHT bootstrap nodes
    pub bootstrap_nodes: Vec<(NodeId, SocketAddr)>,
    /// Discovery interval (seconds)
    pub interval_secs: u64,
    /// Max peers to track
    pub max_peers: usize,
    /// Peer TTL (seconds)
    pub peer_ttl_secs: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_dht: true,
            enable_mdns: true,
            enable_n0: true,
            bootstrap_nodes: vec![],
            interval_secs: 30,
            max_peers: 10000,
            peer_ttl_secs: 300,
        }
    }
}

/// Discovered peer information
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredPeer {
    pub node_id: NodeId,
    pub addresses: Vec<SocketAddr>,
    pub source: DiscoverySource,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum DiscoverySource {
    Dht,
    Mdns,
    N0,
    Bootstrap,
    Manual,
}

/// Discovery service
pub struct Discovery {
    endpoint: Arc<Endpoint>,
    config: DiscoveryConfig,
    peers: Arc<RwLock<HashMap<NodeId, DiscoveredPeer>>>,
    dht: Option<Arc<DhtDiscovery>>,
    mdns: Option<Arc<MdnsDiscovery>>,
    n0: Option<Arc<N0Discovery>>,
}

impl Discovery {
    /// Create new discovery service
    pub async fn new(endpoint: Arc<Endpoint>, config: DiscoveryConfig) -> Result<Self> {
        let peers = Arc::new(RwLock::new(HashMap::new()));

        // Initialize DHT
        let dht = if config.enable_dht {
            let dht = DhtDiscovery::new(endpoint.clone()).await?;
            // Add bootstrap nodes
            for (node_id, addr) in &config.bootstrap_nodes {
                dht.add_node_addr(*node_id, *addr).await?;
            }
            Some(Arc::new(dht))
        } else { None };

        // Initialize mDNS
        let mdns = if config.enable_mdns {
            Some(Arc::new(MdnsDiscovery::new(endpoint.clone()).await?))
        } else { None };

        // Initialize N0
        let n0 = if config.enable_n0 {
            Some(Arc::new(N0Discovery::new(endpoint.clone()).await?))
        } else { None };

        Ok(Self {
            endpoint,
            config,
            peers,
            dht,
            mdns,
            n0,
        })
    }

    /// Start discovery background task
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let peers = self.peers.clone();
        let dht = self.dht.clone();
        let mdns = self.mdns.clone();
        let n0 = self.n0.clone();
        let config = self.config.clone();
        let endpoint = self.endpoint.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.interval_secs));
            loop {
                interval.tick().await;
                if let Err(e) = Self::discover_peers(
                    &peers, &dht, &mdns, &n0, &endpoint, &config
                ).await {
                    warn!("Discovery error: {}", e);
                }
                // Cleanup stale peers
                Self::cleanup_peers(&peers, &config).await;
            }
        })
    }

    /// Discover peers from all sources
    async fn discover_peers(
        peers: &Arc<RwLock<HashMap<NodeId, DiscoveredPeer>>>,
        dht: &Option<Arc<DhtDiscovery>>,
        mdns: &Option<Arc<MdnsDiscovery>>,
        n0: &Option<Arc<N0Discovery>>,
        endpoint: &Arc<Endpoint>,
        config: &DiscoveryConfig,
    ) -> Result<()> {
        let mut new_peers = Vec::new();

        // DHT discovery
        if let Some(dht) = dht {
            let nodes = dht.get_closest_nodes(endpoint.node_id()).await?;
            for node in nodes {
                let addrs = dht.get_node_addrs(node).await?;
                if !addrs.is_empty() {
                    new_peers.push(DiscoveredPeer {
                        node_id: node,
                        addresses: addrs,
                        source: DiscoverySource::Dht,
                        last_seen: chrono::Utc::now(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        // mDNS discovery
        if let Some(mdns) = mdns {
            let nodes = mdns.get_peers().await?;
            for (node_id, addrs) in nodes {
                new_peers.push(DiscoveredPeer {
                    node_id,
                    addresses: addrs,
                    source: DiscoverySource::Mdns,
                    last_seen: chrono::Utc::now(),
                    metadata: HashMap::new(),
                });
            }
        }

        // N0 discovery
        if let Some(n0) = n0 {
            let nodes = n0.get_peers().await?;
            for (node_id, addrs) in nodes {
                new_peers.push(DiscoveredPeer {
                    node_id,
                    addresses: addrs,
                    source: DiscoverySource::N0,
                    last_seen: chrono::Utc::now(),
                    metadata: HashMap::new(),
                });
            }
        }

        // Merge new peers
        let mut peers_map = peers.write().await;
        for peer in new_peers {
            if peers_map.len() >= config.max_peers {
                break;
            }
            peers_map.entry(peer.node_id)
                .and_modify(|p| {
                    p.addresses.extend(peer.addresses.clone());
                    p.addresses.sort();
                    p.addresses.dedup();
                    p.last_seen = peer.last_seen;
                    // Prefer more reliable sources
                    if matches!(peer.source, DiscoverySource::Mdns | DiscoverySource::Dht) {
                        p.source = peer.source;
                    }
                })
                .or_insert(peer);
        }

        debug!("Discovered {} peers", peers_map.len());
        Ok(())
    }

    /// Clean up stale peers
    async fn cleanup_peers(
        peers: &Arc<RwLock<HashMap<NodeId, DiscoveredPeer>>>,
        config: &DiscoveryConfig,
    ) {
        let mut peers_map = peers.write().await;
        let now = chrono::Utc::now();
        let ttl = chrono::Duration::seconds(config.peer_ttl_secs as i64);
        peers_map.retain(|_, peer| now - peer.last_seen < ttl);
    }

    /// Get all known peers
    pub async fn get_peers(&self) -> Vec<DiscoveredPeer> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Get peer by node ID
    pub async fn get_peer(&self, node_id: NodeId) -> Option<DiscoveredPeer> {
        self.peers.read().await.get(&node_id).cloned()
    }

    /// Add a peer manually (e.g., from QR code, invite link)
    pub async fn add_peer(&self, node_id: NodeId, addresses: Vec<SocketAddr>) -> Result<()> {
        let mut peers = self.peers.write().await;
        if peers.len() >= self.config.max_peers {
            return Err(P2pError::MaxPeersReached);
        }
        peers.insert(node_id, DiscoveredPeer {
            node_id,
            addresses,
            source: DiscoverySource::Manual,
            last_seen: chrono::Utc::now(),
            metadata: HashMap::new(),
        });
        Ok(())
    }

    /// Announce ourselves to the network
    pub async fn announce(&self) -> Result<()> {
        if let Some(dht) = &self.dht {
            let addrs = self.endpoint.node_addr().await?
                .direct_addresses()
                .iter()
                .filter_map(|a| a.to_socket_addr())
                .collect();
            for addr in addrs {
                dht.add_node_addr(self.endpoint.node_id(), addr).await?;
            }
        }
        Ok(())
    }

    /// Find closest nodes to a target (for DHT routing)
    pub async fn find_closest(&self, target: NodeId) -> Result<Vec<NodeId>> {
        if let Some(dht) = &self.dht {
            dht.get_closest_nodes(target).await
        } else {
            Ok(vec![])
        }
    }
}