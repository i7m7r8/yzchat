//! Message Router: Connection management, message routing, protocol multiplexing

use crate::{P2pError, Result, NodeId, ALPN_YZCHAT, MAX_MESSAGE_SIZE};
use anyhow::Context;
use iroh::{Endpoint, Connection, NodeAddr};
use iroh_net::protocol::Router as IrohRouter;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::timeout;
use tracing::{info, debug, warn, error};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Router configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RouterConfig {
    /// Max concurrent connections per peer
    pub max_connections_per_peer: usize,
    /// Connection idle timeout (seconds)
    pub idle_timeout_secs: u64,
    /// Message queue size per connection
    pub message_queue_size: usize,
    /// Enable connection pooling
    pub enable_pooling: bool,
    /// Max message size
    pub max_message_size: usize,
    /// Handshake timeout (seconds)
    pub handshake_timeout_secs: u64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_connections_per_peer: 3,
            idle_timeout_secs: 300,
            message_queue_size: 1000,
            enable_pooling: true,
            max_message_size: MAX_MESSAGE_SIZE,
            handshake_timeout_secs: 10,
        }
    }
}

/// Incoming message from network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncomingMessage {
    pub message_id: Uuid,
    pub from: NodeId,
    pub to: NodeId,
    pub protocol: String,
    pub payload: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reply_tx: Option<mpsc::Sender<OutgoingMessage>>,
}

/// Outgoing message to network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutgoingMessage {
    pub message_id: Uuid,
    pub from: NodeId,
    pub to: NodeId,
    pub protocol: String,
    pub payload: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub require_ack: bool,
}

/// Protocol handler trait
pub trait ProtocolHandler: Send + Sync {
    fn protocol_name(&self) -> &'static str;
    fn handle_message(&self, msg: IncomingMessage) -> Result<Vec<OutgoingMessage>>;
    fn on_connect(&self, peer: NodeId) -> Result<()>;
    fn on_disconnect(&self, peer: NodeId) -> Result<()>;
}

/// Connection info
#[derive(Clone, Debug)]
struct ConnectionInfo {
    connection: Connection,
    established_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
    protocol: String,
}

/// Message router
pub struct Router {
    endpoint: Arc<Endpoint>,
    config: RouterConfig,
    connections: Arc<RwLock<HashMap<NodeId, Vec<ConnectionInfo>>>>,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn ProtocolHandler>>>>,
    outbox: Arc<Mutex<mpsc::Sender<OutgoingMessage>>>,
    inbox: mpsc::Receiver<IncomingMessage>,
    inbox_tx: mpsc::Sender<IncomingMessage>,
    local_node_id: NodeId,
}

impl Router {
    /// Create new router
    pub fn new(endpoint: Arc<Endpoint>, config: RouterConfig) -> (Self, mpsc::Receiver<OutgoingMessage>) {
        let (outbox_tx, outbox_rx) = mpsc::channel(config.message_queue_size);
        let (inbox_tx, inbox_rx) = mpsc::channel(config.message_queue_size);
        let local_node_id = endpoint.node_id();

        let router = Self {
            endpoint,
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            outbox: Arc::new(Mutex::new(outbox_tx)),
            inbox: inbox_rx,
            inbox_tx,
            local_node_id,
        };

        (router, outbox_rx)
    }

    /// Register a protocol handler
    pub async fn register_handler(&self, handler: Arc<dyn ProtocolHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(handler.protocol_name().to_string(), handler);
    }

    /// Start the router (accept loop + outbox processor)
    pub async fn start(&self) -> Result<()> {
        // Start accept loop
        let accept_handle = self.start_accept_loop();
        // Start outbox processor
        let outbox_handle = self.start_outbox_processor();
        // Start connection cleanup
        let cleanup_handle = self.start_connection_cleanup();

        // Wait for all (they run forever)
        tokio::try_join!(accept_handle, outbox_handle, cleanup_handle)?;
        Ok(())
    }

    /// Accept incoming connections
    async fn start_accept_loop(&self) -> Result<()> {
        let endpoint = self.endpoint.clone();
        let handlers = self.handlers.clone();
        let connections = self.connections.clone();
        let inbox_tx = self.inbox_tx.clone();
        let config = self.config.clone();
        let local_node_id = self.local_node_id;

        tokio::spawn(async move {
            loop {
                match endpoint.accept().await {
                    Ok(Some(incoming)) => {
                        let handlers = handlers.clone();
                        let connections = connections.clone();
                        let inbox_tx = inbox_tx.clone();
                        let config = config.clone();
                        let local_node_id = local_node_id;

                        tokio::spawn(async move {
                            match Self::handle_incoming_connection(
                                incoming, handlers, connections, inbox_tx, config, local_node_id
                            ).await {
                                Ok(_) => debug!("Connection handled successfully"),
                                Err(e) => warn!("Connection error: {}", e),
                            }
                        });
                    }
                    Ok(None) => {
                        debug!("Endpoint closed");
                        break;
                    }
                    Err(e) => {
                        error!("Accept error: {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle a single incoming connection
    async fn handle_incoming_connection(
        incoming: iroh::Incoming,
        handlers: Arc<RwLock<HashMap<String, Arc<dyn ProtocolHandler>>>>,
        connections: Arc<RwLock<HashMap<NodeId, Vec<ConnectionInfo>>>>,
        inbox_tx: mpsc::Sender<IncomingMessage>,
        config: RouterConfig,
        local_node_id: NodeId,
    ) -> Result<()> {
        let connection = incoming.accept().await?;
        let remote_node_id = connection.remote_node_id()?;
        let protocol = connection.protocol().to_string();

        info!("Incoming connection from {} on protocol {}", remote_node_id, protocol);

        // Check connection limit
        {
            let conns = connections.read().await;
            if let Some(peer_conns) = conns.get(&remote_node_id) {
                if peer_conns.len() >= config.max_connections_per_peer {
                    warn!("Max connections reached for {}", remote_node_id);
                    return Err(P2pError::MaxConnectionsReached);
                }
            }
        }

        // Register connection
        let conn_info = ConnectionInfo {
            connection: connection.clone(),
            established_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            protocol: protocol.clone(),
        };
        connections.write().await.entry(remote_node_id).or_default().push(conn_info);

        // Notify handler
        if let Some(handler) = handlers.read().await.get(&protocol) {
            handler.on_connect(remote_node_id)?;
        }

        // Read loop
        let mut reader = connection.reader();
        loop {
            tokio::select! {
                read_result = reader.read_message() => {
                    match read_result {
                        Ok(Some(bytes)) => {
                            if bytes.len() > config.max_message_size {
                                warn!("Message too large from {}", remote_node_id);
                                continue;
                            }
                            // Update activity
                            connections.write().await
                                .entry(remote_node_id)
                                .and_modify(|v| {
                                    if let Some(c) = v.iter_mut().find(|c| c.connection.stable_id() == connection.stable_id()) {
                                        c.last_activity = chrono::Utc::now();
                                    }
                                });

                            // Parse message (simplified - real impl uses framed protocol)
                            let msg = IncomingMessage {
                                message_id: Uuid::new_v4(),
                                from: remote_node_id,
                                to: local_node_id,
                                protocol: protocol.clone(),
                                payload: bytes,
                                timestamp: chrono::Utc::now(),
                                reply_tx: None,
                            };
                            if inbox_tx.send(msg).await.is_err() {
                                break; // Channel closed
                            }
                        }
                        Ok(None) => break, // Connection closed
                        Err(e) => {
                            warn!("Read error from {}: {}", remote_node_id, e);
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(config.idle_timeout_secs)) => {
                    // Check idle
                    let is_idle = connections.read().await
                        .get(&remote_node_id)
                        .map(|v| v.iter().find(|c| c.connection.stable_id() == connection.stable_id()))
                        .flatten()
                        .map(|c| chrono::Utc::now() - c.last_activity > chrono::Duration::seconds(config.idle_timeout_secs as i64))
                        .unwrap_or(false);
                    if is_idle {
                        debug!("Connection idle, closing");
                        break;
                    }
                }
            }
        }

        // Cleanup
        connections.write().await.entry(remote_node_id).and_modify(|v| {
            v.retain(|c| c.connection.stable_id() != connection.stable_id());
        });
        if let Some(handler) = handlers.read().await.get(&protocol) {
            handler.on_disconnect(remote_node_id)?;
        }

        Ok(())
    }

    /// Process outbox messages
    async fn start_outbox_processor(&self) -> Result<()> {
        let outbox = self.outbox.clone();
        let connections = self.connections.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut rx = outbox.lock().await.subscribe();
            while let Some(msg) = rx.recv().await {
                if let Err(e) = Self::send_message(&connections, &msg, &config).await {
                    warn!("Failed to send message to {}: {}", msg.to, e);
                }
            }
        });

        Ok(())
    }

    /// Send a message to a peer
    async fn send_message(
        connections: &Arc<RwLock<HashMap<NodeId, Vec<ConnectionInfo>>>>,
        msg: &OutgoingMessage,
        config: &RouterConfig,
    ) -> Result<()> {
        let conns = connections.read().await;
        let peer_conns = conns.get(&msg.to)
            .ok_or(P2pError::PeerNotConnected)?;

        // Find connection with matching protocol
        let conn = peer_conns.iter()
            .find(|c| c.protocol == msg.protocol)
            .or_else(|| peer_conns.first())
            .ok_or(P2pError::NoSuitableConnection)?;

        // Send with timeout
        let mut writer = conn.connection.writer();
        timeout(
            Duration::from_secs(config.handshake_timeout_secs),
            writer.write_message(&msg.payload)
        ).await??;

        Ok(())
    }

    /// Send a message (public API)
    pub async fn send(&self, msg: OutgoingMessage) -> Result<()> {
        self.outbox.lock().await.send(msg).await
            .map_err(|_| P2pError::ChannelClosed)?;
        Ok(())
    }

    /// Receive next incoming message
    pub async fn recv(&mut self) -> Option<IncomingMessage> {
        self.inbox.recv().await
    }

    /// Connect to a peer and send a message
    pub async fn connect_and_send(&self, node_id: NodeId, protocol: &str, payload: Vec<u8>) -> Result<()> {
        let connection = self.endpoint.connect(node_id, protocol.as_bytes()).await?;
        let mut writer = connection.writer();
        writer.write_message(&payload).await?;
        Ok(())
    }

    /// Get active connections
    pub async fn connections(&self) -> HashMap<NodeId, usize> {
        let conns = self.connections.read().await;
        conns.iter().map(|(k, v)| (*k, v.len())).collect()
    }

    /// Close all connections to a peer
    pub async fn disconnect_peer(&self, node_id: NodeId) -> Result<()> {
        let mut conns = self.connections.write().await;
        if let Some(peer_conns) = conns.remove(&node_id) {
            for conn_info in peer_conns {
                conn_info.connection.close().await;
            }
        }
        Ok(())
    }

    /// Shutdown router
    pub async fn shutdown(&self) -> Result<()> {
        let conns = self.connections.write().await;
        for (_, peer_conns) in conns.iter() {
            for conn_info in peer_conns {
                conn_info.connection.close().await;
            }
        }
        Ok(())
    }
}