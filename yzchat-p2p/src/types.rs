//! Common types for yzchat-p2p

use iroh::NodeId;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Peer information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: NodeId,
    pub addresses: Vec<std::net::SocketAddr>,
    pub aliases: Vec<String>,
    pub public_key: Vec<u8>, // Identity public key
    pub last_seen: DateTime<Utc>,
    pub is_online: bool,
    pub capabilities: PeerCapabilities,
}

/// Peer capabilities
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PeerCapabilities {
    pub supports_voice: bool,
    pub supports_video: bool,
    pub supports_screen_share: bool,
    pub supports_file_transfer: bool,
    pub supports_groups: bool,
    pub max_video_resolution: VideoResolution,
    pub supported_codecs: Vec<String>,
}

/// Video resolution
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum VideoResolution {
    #[default]
    P720,
    P1080,
    P1440,
    P4K,
    P8K,
}

/// Connection quality
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionQuality {
    pub rtt_ms: u32,
    pub jitter_ms: u32,
    pub packet_loss_percent: f32,
    pub bandwidth_kbps: u32,
    pub quality_score: f32, // 0.0 - 1.0
}

/// Network statistics
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub retransmissions: u64,
}

/// Protocol identifiers
pub mod protocols {
    pub const MESSAGING: &str = "yzchat/messaging/1";
    pub const VOICE_CALL: &str = "yzchat/voice/1";
    pub const VIDEO_CALL: &str = "yzchat/video/1";
    pub const SCREEN_SHARE: &str = "yzchat/screen/1";
    pub const FILE_TRANSFER: &str = "yzchat/file/1";
    pub const SIGNALING: &str = "yzchat/signaling/1";
    pub const SYNC: &str = "yzchat/sync/1";
    pub const PRESENCE: &str = "yzchat/presence/1";
    pub const TYPING: &str = "yzchat/typing/1";
}

/// Message priority
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Delivery receipt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub message_id: Uuid,
    pub recipient: NodeId,
    pub status: DeliveryStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Read,
    Failed,
}

/// Presence status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
    Invisible,
}

/// Typing indicator
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypingIndicator {
    pub conversation_id: Uuid,
    pub user_id: NodeId,
    pub is_typing: bool,
    pub timestamp: DateTime<Utc>,
}