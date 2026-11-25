//! P2PManager: FFI wrapper for yzchat-p2p

use crate::callbacks::*;
use crate::storage_manager::StorageManager;
use crate::types::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use uniffi::Object;
use uuid::Uuid;
use yzchat_crypto::types::IdentityPublicKey as CryptoIdentityPublicKey;
use yzchat_p2p::{
    blobs::{BlobConfig, BlobStore},
    discovery::{Discovery, DiscoveryConfig},
    docs::{DocConfig, DocStore},
    endpoint::{EndpointBuilder, EndpointConfig, NodeEndpoint},
    router::{IncomingMessage, OutgoingMessage, Router, RouterConfig},
    signaling::{Signaling, SignalingConfig},
    types::*,
    P2pError, Result as P2pResult,
};

/// P2P manager for FFI
#[derive(Object)]
pub struct P2PManager {
    endpoint: Arc<NodeEndpoint>,
    discovery: Arc<Discovery>,
    router: Arc<Mutex<Option<Router>>>,
    signaling: Arc<Mutex<Option<Signaling>>>,
    blobs: Arc<BlobStore>,
    docs: Arc<DocStore>,
    crypto: Arc<crate::crypto_manager::CryptoManager>,
    storage: Arc<StorageManager>,
    message_callback: Arc<Mutex<Option<Box<dyn MessageCallback>>>>,
    signaling_callback: Arc<Mutex<Option<Box<dyn SignalingCallback>>>>,
    connection_callback: Arc<Mutex<Option<Box<dyn ConnectionChangeCallback>>>>,
    pending_sessions: Arc<Mutex<HashMap<String, Uuid>>>, // session_id -> signaling session_id
}

impl P2PManager {
    pub async fn new(
        endpoint: Arc<NodeEndpoint>,
        crypto: Arc<crate::crypto_manager::CryptoManager>,
        storage: Arc<StorageManager>,
    ) -> Result<Self> {
        // Create discovery
        let discovery_config = DiscoveryConfig::default();
        let discovery = Arc::new(Discovery::new(endpoint.clone(), discovery_config).await?);

        // Create router
        let (router, outbox_rx) = Router::new(endpoint.clone(), RouterConfig::default());

        // Create signaling
        let signaling_config = SignalingConfig::default();
        let mut signaling = Signaling::new(endpoint.clone(), signaling_config);
        signaling.start().await?;

        // Create blobs
        let blob_config = BlobConfig::default();
        let blobs = Arc::new(BlobStore::new(endpoint.clone(), blob_config).await?);

        // Create docs
        let doc_config = DocConfig::default();
        let docs = Arc::new(DocStore::new(endpoint.clone(), doc_config).await?);

        let manager = Self {
            endpoint: endpoint.clone(),
            discovery,
            router: Arc::new(Mutex::new(Some(router))),
            signaling: Arc::new(Mutex::new(Some(signaling))),
            blobs,
            docs,
            crypto,
            storage,
            message_callback: Arc::new(Mutex::new(None)),
            signaling_callback: Arc::new(Mutex::new(None)),
            connection_callback: Arc::new(Mutex::new(None)),
            pending_sessions: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start background tasks
        manager.start_background_tasks().await?;

        Ok(manager)
    }

    async fn start_background_tasks(&self) -> Result<()> {
        // Start discovery
        let discovery_handle = self.discovery.start();

        // Start router message processing
        let router = self.router.clone();
        let message_callback = self.message_callback.clone();
        let crypto = self.crypto.clone();
        let storage = self.storage.clone();

        tokio::spawn(async move {
            if let Some(mut router) = router.lock().await.take() {
                // Process outbox (handled by router internally)
                // Process inbox
                while let Some(msg) = router.recv().await {
                    // Decrypt message
                    if let Ok(decrypted) =
                        Self::process_incoming_message(&crypto, &storage, msg).await
                    {
                        // Call callback
                        if let Some(cb) = message_callback.lock().await.as_ref() {
                            cb(decrypted.from, decrypted.protocol, decrypted.payload);
                        }
                    }
                }
            }
        });

        // Start connection monitoring
        let endpoint = self.endpoint.clone();
        let connection_callback = self.connection_callback.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            let mut last_connected = false;
            loop {
                interval.tick().await;
                let connected = endpoint.endpoint().is_connected();
                let peer_count = 0; // TODO: get actual count
                if connected != last_connected {
                    if let Some(cb) = connection_callback.lock().await.as_ref() {
                        cb(connected, peer_count);
                    }
                    last_connected = connected;
                }
            }
        });

        Ok(())
    }

    async fn process_incoming_message(
        crypto: &Arc<crate::crypto_manager::CryptoManager>,
        storage: &Arc<StorageManager>,
        msg: IncomingMessage,
    ) -> Result<IncomingMessage> {
        // Try to decrypt if it's a messaging protocol message
        if msg.protocol == yzchat_p2p::protocols::MESSAGING {
            // Decrypt using crypto manager
            // For now, return as-is
        }
        Ok(msg)
    }

    // Identity
    pub fn get_node_id(&self) -> Vec<u8> {
        self.endpoint.node_id().as_bytes().to_vec()
    }

    pub async fn get_node_addresses(&self) -> Result<Vec<String>> {
        let addrs = self.endpoint.listening_addrs().await?;
        Ok(addrs.iter().map(|a| a.to_string()).collect())
    }

    // Discovery
    pub async fn discover_peers(&self) -> Result<Vec<PeerInfo>> {
        let peers = self.discovery.get_peers().await;
        Ok(peers.into_iter().map(Self::peer_to_info).collect())
    }

    pub async fn add_peer(&self, node_id: Vec<u8>, addresses: Vec<String>) -> Result<()> {
        let node_id = yzchat_p2p::types::NodeId::from_bytes(&node_id).context("Invalid node ID")?;
        let addrs: Result<Vec<std::net::SocketAddr>> = addresses
            .iter()
            .map(|a| a.parse().context("Invalid address"))
            .collect();
        self.discovery.add_peer(node_id, addrs?).await
    }

    pub async fn get_peer(&self, node_id: Vec<u8>) -> Result<Option<PeerInfo>> {
        let node_id = yzchat_p2p::types::NodeId::from_bytes(&node_id).context("Invalid node ID")?;
        Ok(self
            .discovery
            .get_peer(node_id)
            .await
            .map(Self::peer_to_info))
    }

    fn peer_to_info(peer: yzchat_p2p::discovery::DiscoveredPeer) -> PeerInfo {
        PeerInfo {
            node_id: peer.node_id.as_bytes().to_vec(),
            addresses: peer.addresses.iter().map(|a| a.to_string()).collect(),
            aliases: vec![],
            public_key: vec![],
            last_seen: peer.last_seen.timestamp_millis(),
            is_online: true,
            capabilities: PeerCapabilities {
                supports_voice: true,
                supports_video: true,
                supports_screen_share: true,
                supports_file_transfer: true,
                supports_groups: true,
                max_video_resolution: VideoResolution::P4K,
                supported_codecs: vec!["AV1".to_string(), "VP9".to_string(), "OPUS".to_string()],
            },
        }
    }

    // Connections
    pub async fn connect(&self, node_id: Vec<u8>) -> Result<()> {
        let node_id = yzchat_p2p::types::NodeId::from_bytes(&node_id).context("Invalid node ID")?;
        self.endpoint
            .connect(node_id, yzchat_p2p::ALPN_YZCHAT)
            .await?;
        Ok(())
    }

    pub async fn disconnect(&self, node_id: Vec<u8>) -> Result<()> {
        let node_id = yzchat_p2p::types::NodeId::from_bytes(&node_id).context("Invalid node ID")?;
        if let Some(router) = self.router.lock().await.as_ref() {
            router.disconnect_peer(node_id).await?;
        }
        Ok(())
    }

    pub async fn get_connections(&self) -> Result<Vec<Vec<u8>>> {
        if let Some(router) = self.router.lock().await.as_ref() {
            let conns = router.connections().await;
            Ok(conns.keys().map(|k| k.as_bytes().to_vec()).collect())
        } else {
            Ok(vec![])
        }
    }

    // Messaging
    pub async fn send_message(
        &self,
        to: Vec<u8>,
        protocol: String,
        payload: Vec<u8>,
    ) -> Result<()> {
        let node_id = yzchat_p2p::types::NodeId::from_bytes(&to).context("Invalid node ID")?;
        if let Some(router) = self.router.lock().await.as_ref() {
            router
                .send(OutgoingMessage {
                    message_id: Uuid::new_v4(),
                    from: self.endpoint.node_id(),
                    to: node_id,
                    protocol,
                    payload,
                    timestamp: chrono::Utc::now(),
                    require_ack: true,
                })
                .await?;
        }
        Ok(())
    }

    pub fn on_message(&self, callback: Box<dyn MessageCallback>) {
        *self.message_callback.lock().await = Some(callback);
    }

    // Signaling
    pub async fn create_call_session(&self, remote_node: Vec<u8>) -> Result<String> {
        let node_id =
            yzchat_p2p::types::NodeId::from_bytes(&remote_node).context("Invalid node ID")?;
        let session_id = Uuid::new_v4();
        self.pending_sessions
            .lock()
            .await
            .insert(session_id.to_string(), session_id);
        Ok(session_id.to_string())
    }

    pub async fn send_offer(&self, session_id: String, offer: SdpOffer) -> Result<()> {
        let session_uuid = Uuid::parse_str(&session_id).context("Invalid session ID")?;
        let remote_node =
            yzchat_p2p::types::NodeId::from_bytes(&offer.from).context("Invalid node ID")?;
        let sdp_offer = yzchat_p2p::signaling::SdpOffer {
            session_id: session_uuid,
            from: self.endpoint.node_id(),
            to: remote_node,
            sdp: offer.sdp,
            timestamp: chrono::DateTime::from_timestamp_millis(offer.timestamp).unwrap_or_default(),
        };
        if let Some(signaling) = self.signaling.lock().await.as_ref() {
            // Set local offer and send
            signaling
                .set_local_offer(session_uuid, sdp_offer.sdp.clone())
                .await?;
            signaling.send_offer(session_uuid).await?;
        }
        Ok(())
    }

    pub async fn send_answer(&self, session_id: String, answer: SdpAnswer) -> Result<()> {
        let session_uuid = Uuid::parse_str(&session_id).context("Invalid session ID")?;
        let remote_node =
            yzchat_p2p::types::NodeId::from_bytes(&answer.from).context("Invalid node ID")?;
        let sdp_answer = yzchat_p2p::signaling::SdpAnswer {
            session_id: session_uuid,
            from: self.endpoint.node_id(),
            to: remote_node,
            sdp: answer.sdp,
            timestamp: chrono::DateTime::from_timestamp_millis(answer.timestamp)
                .unwrap_or_default(),
        };
        if let Some(signaling) = self.signaling.lock().await.as_ref() {
            signaling
                .set_local_answer(session_uuid, sdp_answer.sdp.clone())
                .await?;
            signaling.send_answer(session_uuid).await?;
        }
        Ok(())
    }

    pub async fn send_ice_candidate(
        &self,
        session_id: String,
        candidate: IceCandidate,
    ) -> Result<()> {
        let session_uuid = Uuid::parse_str(&session_id).context("Invalid session ID")?;
        let remote_node = yzchat_p2p::types::NodeId::from_bytes(&candidate.sdp_mid.as_bytes()) // This is wrong, need proper mapping
            .context("Invalid node ID")?;
        let ice = yzchat_p2p::signaling::IceCandidate {
            candidate: candidate.candidate,
            sdp_mid: candidate.sdp_mid,
            sdp_mline_index: candidate.sdp_mline_index,
            username_fragment: candidate.username_fragment,
        };
        if let Some(signaling) = self.signaling.lock().await.as_ref() {
            signaling.add_local_candidate(session_uuid, ice).await?;
            signaling.send_ice_candidate(session_uuid, ice).await?;
        }
        Ok(())
    }

    pub async fn end_call_session(&self, session_id: String, reason: String) -> Result<()> {
        let session_uuid = Uuid::parse_str(&session_id).context("Invalid session ID")?;
        if let Some(signaling) = self.signaling.lock().await.as_ref() {
            signaling.end_session(session_uuid, reason).await?;
        }
        self.pending_sessions.lock().await.remove(&session_id);
        Ok(())
    }

    pub fn on_signaling_message(&self, callback: Box<dyn SignalingCallback>) {
        *self.signaling_callback.lock().await = Some(callback);
    }

    // File transfer
    pub async fn send_file(
        &self,
        peer: Vec<u8>,
        file_path: String,
        encryption_key: Vec<u8>,
    ) -> Result<String> {
        let node_id = yzchat_p2p::types::NodeId::from_bytes(&peer).context("Invalid node ID")?;
        let transfer_id = Uuid::new_v4();

        // Import file to blob store
        let path = std::path::PathBuf::from(&file_path);
        let (hash, format, size) = self.blobs.import_from_file(path).await?;

        // Send to peer
        let key = if !encryption_key.is_empty() {
            // Convert to HybridKemSharedSecret
            None // TODO
        } else {
            None
        };

        self.blobs.send_blob(node_id, hash, format, key).await?;

        Ok(transfer_id.to_string())
    }

    pub async fn receive_file(
        &self,
        peer: Vec<u8>,
        blob_hash: String,
        encryption_key: Vec<u8>,
    ) -> Result<String> {
        let node_id = yzchat_p2p::types::NodeId::from_bytes(&peer).context("Invalid node ID")?;
        let hash = yzchat_p2p::blobs::Hash::from_hex(&blob_hash).context("Invalid hash")?;
        let format = iroh_blobs::store::Format::Raw; // TODO: determine format
        let transfer_id = Uuid::new_v4();

        let key = if !encryption_key.is_empty() {
            None // TODO
        } else {
            None
        };

        self.blobs.receive_blob(node_id, hash, format, key).await?;

        Ok(transfer_id.to_string())
    }

    pub async fn get_transfer_progress(&self, transfer_id: String) -> Result<TransferProgress> {
        let uuid = Uuid::parse_str(&transfer_id).context("Invalid transfer ID")?;
        if let Some(progress) = self.blobs.get_progress(uuid).await {
            Ok(TransferProgress {
                transfer_id: progress.transfer_id.to_string(),
                blob_hash: progress.blob_hash,
                total_bytes: progress.total_bytes,
                transferred_bytes: progress.transferred_bytes,
                status: match progress.status {
                    yzchat_p2p::blobs::TransferStatus::Pending => TransferStatus::Pending,
                    yzchat_p2p::blobs::TransferStatus::Connecting => TransferStatus::Connecting,
                    yzchat_p2p::blobs::TransferStatus::Downloading => TransferStatus::Downloading,
                    yzchat_p2p::blobs::TransferStatus::Uploading => TransferStatus::Uploading,
                    yzchat_p2p::blobs::TransferStatus::Verifying => TransferStatus::Verifying,
                    yzchat_p2p::blobs::TransferStatus::Completed => TransferStatus::Completed,
                    yzchat_p2p::blobs::TransferStatus::Failed => TransferStatus::Failed,
                    yzchat_p2p::blobs::TransferStatus::Cancelled => TransferStatus::Cancelled,
                },
                started_at: progress.started_at.timestamp_millis(),
                completed_at: progress.completed_at.map(|t| t.timestamp_millis()),
            })
        } else {
            Err(anyhow::anyhow!("Transfer not found"))
        }
    }

    pub async fn cancel_transfer(&self, transfer_id: String) -> Result<()> {
        let uuid = Uuid::parse_str(&transfer_id).context("Invalid transfer ID")?;
        self.blobs.cancel_transfer(uuid).await
    }

    // Shutdown
    pub async fn shutdown(&self) -> Result<()> {
        if let Some(router) = self.router.lock().await.take() {
            router.shutdown().await?;
        }
        self.endpoint.close().await?;
        Ok(())
    }

    pub fn set_connection_callback(&self, callback: Box<dyn ConnectionChangeCallback>) {
        *self.connection_callback.lock().await = Some(callback);
    }
}
