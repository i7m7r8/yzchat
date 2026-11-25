//! WebRTC Engine: Peer connections, ICE, DTLS-SRTP, data channels

use crate::{codec_prefs, MediaError, Result};
use anyhow::Context;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use webrtc::api::{APIBuilder, API};
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::Configuration;
use webrtc::peer_connection::peer_connection_state::PeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::rtp_transceiver::RTCRtpTransceiver;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

/// WebRTC configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WebRtcConfig {
    /// ICE servers (STUN/TURN)
    pub ice_servers: Vec<IceServerConfig>,
    /// Enable DTLS-SRTP (required)
    pub enable_dtls_srtp: bool,
    /// Preferred video codecs
    pub video_codecs: Vec<String>,
    /// Preferred audio codecs
    pub audio_codecs: Vec<String>,
    /// Enable simulcast
    pub enable_simulcast: bool,
    /// Enable SVC
    pub enable_svc: bool,
    /// Bundle policy
    pub bundle_policy: BundlePolicy,
    /// RTCP mux policy
    pub rtcp_mux_policy: RtcpMuxPolicy,
    /// ICE candidate pool size
    pub ice_candidate_pool_size: u16,
    /// Audio bitrate (bps)
    pub audio_bitrate: u32,
    /// Video bitrate (bps)
    pub video_bitrate: u32,
    /// Screen share bitrate (bps)
    pub screen_bitrate: u32,
}

impl Default for WebRtcConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec![
                IceServerConfig {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    username: None,
                    credential: None,
                },
                IceServerConfig {
                    urls: vec!["stun:stun1.l.google.com:19302".to_string()],
                    username: None,
                    credential: None,
                },
            ],
            enable_dtls_srtp: true,
            video_codecs: codec_prefs::VIDEO.iter().map(|s| s.to_string()).collect(),
            audio_codecs: codec_prefs::AUDIO.iter().map(|s| s.to_string()).collect(),
            enable_simulcast: true,
            enable_svc: true,
            bundle_policy: BundlePolicy::MaxBundle,
            rtcp_mux_policy: RtcpMuxPolicy::Require,
            ice_candidate_pool_size: 10,
            audio_bitrate: 510_000,     // 510 kbps Opus
            video_bitrate: 25_000_000,  // 25 Mbps for 4K
            screen_bitrate: 50_000_000, // 50 Mbps for 4K screen
        }
    }
}

/// ICE server configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct IceServerConfig {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

/// Bundle policy
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum BundlePolicy {
    Balanced,
    MaxCompat,
    MaxBundle,
}

/// RTCP mux policy
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum RtcpMuxPolicy {
    Negotiate,
    Require,
}

/// WebRTC peer connection wrapper
pub struct PeerConnection {
    pc: Arc<RTCPeerConnection>,
    config: WebRtcConfig,
    local_description: Arc<RwLock<Option<RTCSessionDescription>>>,
    remote_description: Arc<RwLock<Option<RTCSessionDescription>>>,
    ice_candidates: Arc<Mutex<Vec<RTCIceCandidateInit>>>,
    data_channels: Arc<RwLock<HashMap<String, Arc<RTCDataChannel>>>>,
    tracks: Arc<RwLock<HashMap<String, Arc<dyn TrackLocal + Send + Sync>>>>,
    on_ice_candidate: Option<Box<dyn Fn(RTCIceCandidateInit) + Send + Sync>>,
    on_connection_state_change: Option<Box<dyn Fn(PeerConnectionState) + Send + Sync>>,
    on_data_channel: Option<Box<dyn Fn(Arc<RTCDataChannel>) + Send + Sync>>,
    on_track: Option<Box<dyn Fn(Arc<dyn TrackLocal + Send + Sync>) + Send + Sync>>,
}

impl PeerConnection {
    /// Create a new peer connection
    pub async fn new(config: WebRtcConfig) -> Result<Self> {
        let api = APIBuilder::new().build();

        let rtc_config = Configuration {
            ice_servers: config
                .ice_servers
                .iter()
                .map(|s| RTCIceServer {
                    urls: s.urls.clone(),
                    username: s.username.clone(),
                    credential: s.credential.clone(),
                    ..Default::default()
                })
                .collect(),
            ice_transport_policy:
                webrtc::peer_connection::configuration::RTCIceTransportPolicy::All,
            bundle_policy: match config.bundle_policy {
                BundlePolicy::Balanced => {
                    webrtc::peer_connection::configuration::RTCBundlePolicy::Balanced
                }
                BundlePolicy::MaxCompat => {
                    webrtc::peer_connection::configuration::RTCBundlePolicy::MaxCompat
                }
                BundlePolicy::MaxBundle => {
                    webrtc::peer_connection::configuration::RTCBundlePolicy::MaxBundle
                }
            },
            rtcp_mux_policy: match config.rtcp_mux_policy {
                RtcpMuxPolicy::Negotiate => {
                    webrtc::peer_connection::configuration::RTCRtcpMuxPolicy::Negotiate
                }
                RtcpMuxPolicy::Require => {
                    webrtc::peer_connection::configuration::RTCRtcpMuxPolicy::Require
                }
            },
            ice_candidate_pool_size: config.ice_candidate_pool_size,
            ..Default::default()
        };

        let pc = api
            .new_peer_connection(rtc_config)
            .await
            .context("Failed to create peer connection")?;

        let pc = Arc::new(pc);

        let conn = Self {
            pc: pc.clone(),
            config,
            local_description: Arc::new(RwLock::new(None)),
            remote_description: Arc::new(RwLock::new(None)),
            ice_candidates: Arc::new(Mutex::new(Vec::new())),
            data_channels: Arc::new(RwLock::new(HashMap::new())),
            tracks: Arc::new(RwLock::new(HashMap::new())),
            on_ice_candidate: None,
            on_connection_state_change: None,
            on_data_channel: None,
            on_track: None,
        };

        // Set up event handlers
        conn.setup_handlers().await?;

        Ok(conn)
    }

    /// Set up event handlers
    async fn setup_handlers(&self) -> Result<()> {
        let pc = self.pc.clone();
        let ice_candidates = self.ice_candidates.clone();
        let on_ice_candidate = self.on_ice_candidate.clone();

        // ICE candidate handler
        pc.on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidateInit>| {
            if let Some(c) = candidate {
                let ice_candidates = ice_candidates.clone();
                let on_ice_candidate = on_ice_candidate.clone();
                tokio::spawn(async move {
                    ice_candidates.lock().await.push(c.clone());
                    if let Some(cb) = on_ice_candidate {
                        cb(c);
                    }
                });
            }
            Box::pin(async {})
        }));

        // Connection state change
        let on_state_change = self.on_connection_state_change.clone();
        pc.on_connection_state_change(Box::new(move |state: PeerConnectionState| {
            if let Some(cb) = on_state_change {
                cb(state);
            }
            Box::pin(async {})
        }));

        // Data channel
        let data_channels = self.data_channels.clone();
        let on_data_channel = self.on_data_channel.clone();
        pc.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
            let label = dc.label().to_string();
            let data_channels = data_channels.clone();
            let on_data_channel = on_data_channel.clone();
            tokio::spawn(async move {
                data_channels
                    .write()
                    .await
                    .insert(label.clone(), dc.clone());
                if let Some(cb) = on_data_channel {
                    cb(dc);
                }
            });
            Box::pin(async {})
        }));

        // Track
        let tracks = self.tracks.clone();
        let on_track = self.on_track.clone();
        pc.on_track(Box::new(move |track, _receiver, _transceiver| {
            let tracks = tracks.clone();
            let on_track = on_track.clone();
            tokio::spawn(async move {
                let id = track.id();
                tracks.write().await.insert(id, track.clone());
                if let Some(cb) = on_track {
                    cb(track);
                }
            });
            Box::pin(async {})
        }));

        Ok(())
    }

    /// Create offer
    pub async fn create_offer(&self) -> Result<RTCSessionDescription> {
        let offer = self.pc.create_offer(None).await?;
        self.pc.set_local_description(offer.clone()).await?;
        *self.local_description.write().await = Some(offer.clone());
        Ok(offer)
    }

    /// Create answer
    pub async fn create_answer(&self) -> Result<RTCSessionDescription> {
        let answer = self.pc.create_answer(None).await?;
        self.pc.set_local_description(answer.clone()).await?;
        *self.local_description.write().await = Some(answer.clone());
        Ok(answer)
    }

    /// Set remote description
    pub async fn set_remote_description(&self, desc: RTCSessionDescription) -> Result<()> {
        self.pc.set_remote_description(desc.clone()).await?;
        *self.remote_description.write().await = Some(desc);
        Ok(())
    }

    /// Add ICE candidate
    pub async fn add_ice_candidate(&self, candidate: RTCIceCandidateInit) -> Result<()> {
        self.pc.add_ice_candidate(candidate).await?;
        Ok(())
    }

    /// Get pending ICE candidates
    pub async fn take_ice_candidates(&self) -> Vec<RTCIceCandidateInit> {
        let mut candidates = self.ice_candidates.lock().await;
        std::mem::take(&mut *candidates)
    }

    /// Add audio track
    pub async fn add_audio_track(
        &self,
        track: Arc<TrackLocalStaticSample>,
    ) -> Result<Arc<RTCRtpTransceiver>> {
        let transceiver = self.pc.add_track(track.clone()).await?;
        self.tracks.write().await.insert(track.id(), track);
        Ok(transceiver)
    }

    /// Add video track
    pub async fn add_video_track(
        &self,
        track: Arc<TrackLocalStaticSample>,
    ) -> Result<Arc<RTCRtpTransceiver>> {
        let transceiver = self.pc.add_track(track.clone()).await?;
        self.tracks.write().await.insert(track.id(), track);
        Ok(transceiver)
    }

    /// Add screen track
    pub async fn add_screen_track(
        &self,
        track: Arc<TrackLocalStaticSample>,
    ) -> Result<Arc<RTCRtpTransceiver>> {
        let transceiver = self.pc.add_track(track.clone()).await?;
        self.tracks.write().await.insert(track.id(), track);
        Ok(transceiver)
    }

    /// Create data channel
    pub async fn create_data_channel(
        &self,
        label: &str,
        ordered: bool,
        max_retransmits: Option<u16>,
    ) -> Result<Arc<RTCDataChannel>> {
        let dc = self
            .pc
            .create_data_channel(
                label,
                Some(webrtc::data_channel::RTCDataChannelInit {
                    ordered: Some(ordered),
                    max_retransmits,
                    ..Default::default()
                }),
            )
            .await?;
        self.data_channels
            .write()
            .await
            .insert(label.to_string(), dc.clone());
        Ok(dc)
    }

    /// Get connection state
    pub fn connection_state(&self) -> PeerConnectionState {
        self.pc.connection_state()
    }

    /// Close connection
    pub async fn close(&self) -> Result<()> {
        self.pc.close().await?;
        Ok(())
    }

    /// Set ICE candidate callback
    pub fn on_ice_candidate<F>(&mut self, f: F)
    where
        F: Fn(RTCIceCandidateInit) + Send + Sync + 'static,
    {
        self.on_ice_candidate = Some(Box::new(f));
    }

    /// Set connection state change callback
    pub fn on_connection_state_change<F>(&mut self, f: F)
    where
        F: Fn(PeerConnectionState) + Send + Sync + 'static,
    {
        self.on_connection_state_change = Some(Box::new(f));
    }

    /// Set data channel callback
    pub fn on_data_channel<F>(&mut self, f: F)
    where
        F: Fn(Arc<RTCDataChannel>) + Send + Sync + 'static,
    {
        self.on_data_channel = Some(Box::new(f));
    }

    /// Set track callback
    pub fn on_track<F>(&mut self, f: F)
    where
        F: Fn(Arc<dyn TrackLocal + Send + Sync>) + Send + Sync + 'static,
    {
        self.on_track = Some(Box::new(f));
    }

    /// Get local description
    pub async fn local_description(&self) -> Option<RTCSessionDescription> {
        self.local_description.read().await.clone()
    }

    /// Get remote description
    pub async fn remote_description(&self) -> Option<RTCSessionDescription> {
        self.remote_description.read().await.clone()
    }

    /// Get stats
    pub async fn get_stats(&self) -> Result<webrtc::stats::StatsReport> {
        Ok(self.pc.get_stats().await?)
    }
}

/// WebRTC Engine: manages multiple peer connections
pub struct WebRtcEngine {
    config: WebRtcConfig,
    connections: Arc<RwLock<HashMap<Uuid, Arc<PeerConnection>>>>,
    api: Arc<API>,
}

impl WebRtcEngine {
    /// Create new WebRTC engine
    pub fn new(config: WebRtcConfig) -> Self {
        let api = Arc::new(APIBuilder::new().build());
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            api,
        }
    }

    /// Create a new peer connection
    pub async fn create_peer_connection(&self, connection_id: Uuid) -> Result<Arc<PeerConnection>> {
        let pc = PeerConnection::new(self.config.clone()).await?;
        let pc = Arc::new(pc);
        self.connections
            .write()
            .await
            .insert(connection_id, pc.clone());
        Ok(pc)
    }

    /// Get peer connection
    pub async fn get_peer_connection(&self, connection_id: Uuid) -> Option<Arc<PeerConnection>> {
        self.connections.read().await.get(&connection_id).cloned()
    }

    /// Remove peer connection
    pub async fn remove_peer_connection(&self, connection_id: Uuid) -> Result<()> {
        if let Some(pc) = self.connections.write().await.remove(&connection_id) {
            pc.close().await?;
        }
        Ok(())
    }

    /// List all connections
    pub async fn list_connections(&self) -> Vec<Uuid> {
        self.connections.read().await.keys().cloned().collect()
    }

    /// Get engine config
    pub fn config(&self) -> &WebRtcConfig {
        &self.config
    }
}
