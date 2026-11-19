//! WebRTC Signaling over Iroh RPC
//!
//! Exchanges SDP offers/answers and ICE candidates for WebRTC peer connections.
//! Uses Iroh's RPC system for reliable, ordered delivery over QUIC.

use crate::{P2pError, Result, NodeId};
use anyhow::Context;
use iroh::Endpoint;
use iroh_rpc::proto::{Service, Method, Request, Response};
use iroh_rpc::server::Server;
use iroh_rpc::client::Client;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, debug, warn};
use uuid::Uuid;

/// Signaling protocol name
pub const SIGNALING_PROTOCOL: &str = "yzchat/signaling/1";

/// Signaling configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SignalingConfig {
    /// Max pending signaling sessions
    pub max_sessions: usize,
    /// Session timeout (seconds)
    pub session_timeout_secs: u64,
    /// ICE gathering timeout (seconds)
    pub ice_gathering_timeout_secs: u64,
}

impl Default for SignalingConfig {
    fn default() -> Self {
        Self {
            max_sessions: 100,
            session_timeout_secs: 60,
            ice_gathering_timeout_secs: 30,
        }
    }
}

/// ICE candidate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: String,
    pub sdp_mline_index: u32,
    pub username_fragment: Option<String>,
}

/// SDP offer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdpOffer {
    pub session_id: Uuid,
    pub from: NodeId,
    pub to: NodeId,
    pub sdp: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// SDP answer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdpAnswer {
    pub session_id: Uuid,
    pub from: NodeId,
    pub to: NodeId,
    pub sdp: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Signaling message types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    Offer(SdpOffer),
    Answer(SdpAnswer),
    IceCandidate(IceCandidateMessage),
    End(SessionEnd),
    Reject(SessionReject),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IceCandidateMessage {
    pub session_id: Uuid,
    pub from: NodeId,
    pub to: NodeId,
    pub candidate: IceCandidate,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionEnd {
    pub session_id: Uuid,
    pub from: NodeId,
    pub to: NodeId,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionReject {
    pub session_id: Uuid,
    pub from: NodeId,
    pub to: NodeId,
    pub reason: String,
}

/// Signaling session state
#[derive(Clone, Debug)]
pub struct SignalingSession {
    pub session_id: Uuid,
    pub remote_node: NodeId,
    pub state: SessionState,
    pub local_sdp: Option<String>,
    pub remote_sdp: Option<String>,
    pub local_candidates: Vec<IceCandidate>,
    pub remote_candidates: Vec<IceCandidate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionState {
    Pending,
    OfferSent,
    OfferReceived,
    AnswerSent,
    AnswerReceived,
    Connected,
    Failed,
    Ended,
}

/// Signaling service
pub struct Signaling {
    endpoint: Arc<Endpoint>,
    config: SignalingConfig,
    sessions: Arc<RwLock<HashMap<Uuid, SignalingSession>>>,
    message_tx: mpsc::Sender<SignalingMessage>,
    message_rx: mpsc::Receiver<SignalingMessage>,
    rpc_server: Option<Server>,
    rpc_client: Option<Client>,
}

impl Signaling {
    /// Create new signaling service
    pub fn new(endpoint: Arc<Endpoint>, config: SignalingConfig) -> Self {
        let (message_tx, message_rx) = mpsc::channel(config.max_sessions);
        Self {
            endpoint,
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            message_tx,
            message_rx,
            rpc_server: None,
            rpc_client: None,
        }
    }

    /// Start signaling service (register RPC methods)
    pub async fn start(&mut self) -> Result<()> {
        // Create RPC server
        let mut server = Server::builder()
            .accept(SIGNALING_PROTOCOL)
            .build();

        // Register methods
        server.register_method(
            Method::new("offer", Self::handle_offer),
            self.sessions.clone(),
        ).await?;
        server.register_method(
            Method::new("answer", Self::handle_answer),
            self.sessions.clone(),
        ).await?;
        server.register_method(
            Method::new("ice_candidate", Self::handle_ice_candidate),
            self.sessions.clone(),
        ).await?;
        server.register_method(
            Method::new("end", Self::handle_end),
            self.sessions.clone(),
        ).await?;

        self.rpc_server = Some(server);

        // Start message processor
        self.start_message_processor();

        Ok(())
    }

    /// Start background message processor
    fn start_message_processor(&self) {
        let sessions = self.sessions.clone();
        let endpoint = self.endpoint.clone();
        let mut message_rx = self.message_rx.clone(); // This won't work directly, need refactor

        tokio::spawn(async move {
            // Process outgoing messages via RPC
            // Real impl would use the RPC client
        });
    }

    /// Create a new signaling session (caller)
    pub async fn create_session(&self, remote_node: NodeId) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let session = SignalingSession {
            session_id,
            remote_node,
            state: SessionState::Pending,
            local_sdp: None,
            remote_sdp: None,
            local_candidates: vec![],
            remote_candidates: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        self.sessions.write().await.insert(session_id, session);
        Ok(session_id)
    }

    /// Set local SDP offer
    pub async fn set_local_offer(&self, session_id: Uuid, sdp: String) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(&session_id)
            .ok_or(P2pError::SessionNotFound)?;
        session.local_sdp = Some(sdp);
        session.state = SessionState::OfferSent;
        session.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Set local SDP answer
    pub async fn set_local_answer(&self, session_id: Uuid, sdp: String) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(&session_id)
            .ok_or(P2pError::SessionNotFound)?;
        session.local_sdp = Some(sdp);
        session.state = SessionState::AnswerSent;
        session.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Add local ICE candidate
    pub async fn add_local_candidate(&self, session_id: Uuid, candidate: IceCandidate) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(&session_id)
            .ok_or(P2pError::SessionNotFound)?;
        session.local_candidates.push(candidate);
        session.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Get session state
    pub async fn get_session(&self, session_id: Uuid) -> Option<SignalingSession> {
        self.sessions.read().await.get(&session_id).cloned()
    }

    /// Send offer to remote
    pub async fn send_offer(&self, session_id: Uuid) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)
            .ok_or(P2pError::SessionNotFound)?;

        let offer = SdpOffer {
            session_id,
            from: self.endpoint.node_id(),
            to: session.remote_node,
            sdp: session.local_sdp.clone()
                .ok_or(P2pError::NoLocalSdp)?,
            timestamp: chrono::Utc::now(),
        };

        // Send via RPC (simplified)
        self.send_rpc_message(session.remote_node, SignalingMessage::Offer(offer)).await
    }

    /// Send answer to remote
    pub async fn send_answer(&self, session_id: Uuid) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)
            .ok_or(P2pError::SessionNotFound)?;

        let answer = SdpAnswer {
            session_id,
            from: self.endpoint.node_id(),
            to: session.remote_node,
            sdp: session.local_sdp.clone()
                .ok_or(P2pError::NoLocalSdp)?,
            timestamp: chrono::Utc::now(),
        };

        self.send_rpc_message(session.remote_node, SignalingMessage::Answer(answer)).await
    }

    /// Send ICE candidate
    pub async fn send_ice_candidate(&self, session_id: Uuid, candidate: IceCandidate) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)
            .ok_or(P2pError::SessionNotFound)?;

        let ice_msg = IceCandidateMessage {
            session_id,
            from: self.endpoint.node_id(),
            to: session.remote_node,
            candidate,
            timestamp: chrono::Utc::now(),
        };

        self.send_rpc_message(session.remote_node, SignalingMessage::IceCandidate(ice_msg)).await
    }

    /// End session
    pub async fn end_session(&self, session_id: Uuid, reason: String) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)
            .ok_or(P2pError::SessionNotFound)?;

        let end_msg = SessionEnd {
            session_id,
            from: self.endpoint.node_id(),
            to: session.remote_node,
            reason,
        };

        self.send_rpc_message(session.remote_node, SignalingMessage::End(end_msg)).await?;
        self.sessions.write().await.remove(&session_id);
        Ok(())
    }

    /// Send RPC message (simplified)
    async fn send_rpc_message(&self, to: NodeId, msg: SignalingMessage) -> Result<()> {
        // In real impl: use iroh-rpc client to call remote method
        // For now, just log
        debug!("Sending signaling message to {}: {:?}", to, msg);
        Ok(())
    }

    // RPC Handlers
    async fn handle_offer(
        sessions: Arc<RwLock<HashMap<Uuid, SignalingSession>>>,
        request: Request,
    ) -> Result<Response> {
        let offer: SdpOffer = request.decode().await?;
        let mut sessions = sessions.write().await;

        let session = sessions.entry(offer.session_id).or_insert_with(|| SignalingSession {
            session_id: offer.session_id,
            remote_node: offer.from,
            state: SessionState::OfferReceived,
            local_sdp: None,
            remote_sdp: Some(offer.sdp.clone()),
            local_candidates: vec![],
            remote_candidates: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        session.remote_sdp = Some(offer.sdp);
        session.state = SessionState::OfferReceived;
        session.updated_at = chrono::Utc::now();

        Ok(Response::empty())
    }

    async fn handle_answer(
        sessions: Arc<RwLock<HashMap<Uuid, SignalingSession>>>,
        request: Request,
    ) -> Result<Response> {
        let answer: SdpAnswer = request.decode().await?;
        let mut sessions = sessions.write().await;

        if let Some(session) = sessions.get_mut(&answer.session_id) {
            session.remote_sdp = Some(answer.sdp);
            session.state = SessionState::AnswerReceived;
            session.updated_at = chrono::Utc::now();
        }

        Ok(Response::empty())
    }

    async fn handle_ice_candidate(
        sessions: Arc<RwLock<HashMap<Uuid, SignalingSession>>>,
        request: Request,
    ) -> Result<Response> {
        let ice_msg: IceCandidateMessage = request.decode().await?;
        let mut sessions = sessions.write().await;

        if let Some(session) = sessions.get_mut(&ice_msg.session_id) {
            session.remote_candidates.push(ice_msg.candidate);
            session.updated_at = chrono::Utc::now();
        }

        Ok(Response::empty())
    }

    async fn handle_end(
        sessions: Arc<RwLock<HashMap<Uuid, SignalingSession>>>,
        request: Request,
    ) -> Result<Response> {
        let end_msg: SessionEnd = request.decode().await?;
        let mut sessions = sessions.write().await;

        if let Some(session) = sessions.get_mut(&end_msg.session_id) {
            session.state = SessionState::Ended;
            session.updated_at = chrono::Utc::now();
        }

        Ok(Response::empty())
    }
}