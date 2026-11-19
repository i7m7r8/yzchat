//! Node Endpoint: Iroh identity + QUIC listener + RPC server

use crate::{ALPN_YZCHAT, MAX_MESSAGE_SIZE, P2pError, Result};
use anyhow::Context;
use iroh::{Endpoint, NodeId, SecretKey, protocol::Router};
use iroh_net::key::SecretKey as NetSecretKey;
use iroh_net::{discovery::dht::DhtDiscovery, NetConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn};

/// Endpoint configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EndpointConfig {
    /// Secret key (generated if None)
    pub secret_key: Option<Vec<u8>>,
    /// QUIC listen addresses
    pub listen_addrs: Vec<SocketAddr>,
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Enable DHT discovery
    pub enable_dht: bool,
    /// Enable N0 discovery
    pub enable_n0: bool,
    /// Bootstrap nodes for DHT
    pub bootstrap_nodes: Vec<(NodeId, SocketAddr)>,
    /// ALPN protocols
    pub alpns: Vec<Vec<u8>>,
    /// Max concurrent connections
    pub max_connections: usize,
    /// Connection timeout (seconds)
    pub connection_timeout: u64,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            secret_key: None,
            listen_addrs: vec!["[::]:0".parse().unwrap(), "0.0.0.0:0".parse().unwrap()],
            enable_mdns: true,
            enable_dht: true,
            enable_n0: true,
            bootstrap_nodes: vec![],
            alpns: vec![ALPN_YZCHAT.to_vec()],
            max_connections: 1000,
            connection_timeout: 30,
        }
    }
}

/// Node endpoint wrapper
pub struct NodeEndpoint {
    endpoint: Endpoint,
    node_id: NodeId,
    router: Arc<RwLock<Option<Router>>>,
    config: EndpointConfig,
}

impl NodeEndpoint {
    /// Create a new endpoint from config
    pub async fn new(config: EndpointConfig) -> Result<Self> {
        // Generate or load secret key
        let secret_key = if let Some(bytes) = config.secret_key {
            SecretKey::from_bytes(&bytes).context("Invalid secret key")?
        } else {
            SecretKey::generate(rand::rngs::OsRng)
        };

        let node_id = secret_key.public();

        // Build net config
        let mut net_config = NetConfig::default();
        net_config.discovery_dht = config.enable_dht;
        net_config.discovery_mdns = config.enable_mdns;
        net_config.discovery_n0 = config.enable_n0;
        net_config.bootstrap_nodes = config.bootstrap_nodes.clone();

        // Create endpoint
        let endpoint = Endpoint::builder()
            .secret_key(secret_key)
            .alpns(config.alpns.clone())
            .discovery(net_config.discovery)
            .bind()
            .await
            .context("Failed to bind endpoint")?;

        info!("Node endpoint created: {}", node_id);
        debug!("Listening on: {:?}", endpoint.node_addr().await?);

        Ok(Self {
            endpoint,
            node_id,
            router: Arc::new(RwLock::new(None)),
            config,
        })
    }

    /// Get the node ID
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get the underlying Iroh endpoint
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Get the router (for protocol registration)
    pub async fn router(&self) -> Option<Router> {
        self.router.read().await.clone()
    }

    /// Set the router
    pub async fn set_router(&self, router: Router) {
        *self.router.write().await = Some(router);
    }

    /// Get listening addresses
    pub async fn listening_addrs(&self) -> Result<Vec<SocketAddr>> {
        let node_addr = self.endpoint.node_addr().await?;
        Ok(node_addr.direct_addresses().iter().map(|a| a.to_socket_addr().unwrap()).collect())
    }

    /// Get home relay URL (for NAT traversal)
    pub async fn home_relay(&self) -> Result<Option<String>> {
        Ok(self.endpoint.home_relay().await.map(|r| r.to_string()))
    }

    /// Connect to a remote node
    pub async fn connect(&self, node_id: NodeId, addr: Option<SocketAddr>) -> Result<iroh::Connection> {
        let conn = if let Some(addr) = addr {
            self.endpoint.connect(addr, ALPN_YZCHAT).await?
        } else {
            self.endpoint.connect(node_id, ALPN_YZCHAT).await?
        };
        Ok(conn)
    }

    /// Accept incoming connections (for custom protocols)
    pub async fn accept(&self) -> Result<Option<iroh::Incoming>> {
        Ok(self.endpoint.accept().await)
    }

    /// Close the endpoint
    pub async fn close(&self) -> Result<()> {
        self.endpoint.close().await;
        Ok(())
    }

    /// Export secret key for backup
    pub fn secret_key_bytes(&self) -> Vec<u8> {
        self.endpoint.secret_key().to_bytes().to_vec()
    }
}

/// Builder for NodeEndpoint with fluent API
pub struct EndpointBuilder {
    config: EndpointConfig,
}

impl EndpointBuilder {
    pub fn new() -> Self {
        Self { config: EndpointConfig::default() }
    }

    pub fn secret_key(mut self, key: Vec<u8>) -> Self {
        self.config.secret_key = Some(key);
        self
    }

    pub fn listen_addr(mut self, addr: SocketAddr) -> Self {
        self.config.listen_addrs.push(addr);
        self
    }

    pub fn enable_mdns(mut self, enable: bool) -> Self {
        self.config.enable_mdns = enable;
        self
    }

    pub fn enable_dht(mut self, enable: bool) -> Self {
        self.config.enable_dht = enable;
        self
    }

    pub fn enable_n0(mut self, enable: bool) -> Self {
        self.config.enable_n0 = enable;
        self
    }

    pub fn bootstrap_node(mut self, node_id: NodeId, addr: SocketAddr) -> Self {
        self.config.bootstrap_nodes.push((node_id, addr));
        self
    }

    pub fn alpns(mut self, alpns: Vec<Vec<u8>>) -> Self {
        self.config.alpns = alpns;
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    pub fn connection_timeout(mut self, secs: u64) -> Self {
        self.config.connection_timeout = secs;
        self
    }

    pub async fn build(self) -> Result<NodeEndpoint> {
        NodeEndpoint::new(self.config).await
    }
}

impl Default for EndpointBuilder {
    fn default() -> Self {
        Self::new()
    }
}