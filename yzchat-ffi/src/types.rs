//! FFI types for yzchat

use serde::{Deserialize, Serialize};
use uniffi::Object;

/// Message type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum MessageType {
    KeyExchange,
    Message,
    GroupMessage,
    CallSignaling,
    FileTransfer,
    Receipt,
    Typing,
    Presence,
}

/// Video resolution
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum VideoResolution {
    P720,
    P1080,
    P1440,
    P4K,
    P8K,
}

/// Peer capabilities
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Record)]
pub struct PeerCapabilities {
    pub supports_voice: bool,
    pub supports_video: bool,
    pub supports_screen_share: bool,
    pub supports_file_transfer: bool,
    pub supports_groups: bool,
    pub max_video_resolution: VideoResolution,
    pub supported_codecs: Vec<String>,
}

impl Default for PeerCapabilities {
    fn default() -> Self {
        Self {
            supports_voice: true,
            supports_video: true,
            supports_screen_share: true,
            supports_file_transfer: true,
            supports_groups: true,
            max_video_resolution: VideoResolution::P4K,
            supported_codecs: vec!["AV1".to_string(), "VP9".to_string(), "OPUS".to_string()],
        }
    }
}

/// Identity public key
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Record)]
pub struct IdentityPublicKey {
    pub bytes: Vec<u8>,
}

impl IdentityPublicKey {
    pub fn from_hex_string(hex: &str) -> Self {
        let bytes = hex.chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|c| u8::from_str_radix(&c.iter().collect::<String>(), 16).unwrap())
            .collect();
        Self { bytes }
    }

    pub fn to_hex_string(&self) -> String {
        self.bytes.iter().map(|b| format!("{:02X}", b)).collect()
    }
}

/// Identity secret key
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct IdentitySecretKey {
    pub bytes: Vec<u8>,
}

/// Encryption public key
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Record)]
pub struct EncryptionPublicKey {
    pub bytes: Vec<u8>,
}

/// Encryption secret key
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct EncryptionSecretKey {
    pub bytes: Vec<u8>,
}

/// PreKey for async key exchange
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct PreKey {
    pub key_id: u32,
    pub public_key: EncryptionPublicKey,
    pub signature: Vec<u8>,
    pub timestamp: i64,
}

/// Signed pre-key
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct SignedPreKey {
    pub key_id: u32,
    pub public_key: EncryptionPublicKey,
    pub signature: Vec<u8>,
    pub timestamp: i64,
}

/// Key bundle for session initiation
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct KeyBundle {
    pub identity_key: IdentityPublicKey,
    pub signed_pre_key: SignedPreKey,
    pub one_time_pre_keys: Vec<PreKey>,
    pub timestamp: i64,
}

/// Encrypted message
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct EncryptedMessage {
    pub message_type: MessageType,
    pub sender_identity: IdentityPublicKey,
    pub recipient_identity: IdentityPublicKey,
    pub timestamp: i64,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub ratchet_header: Option<RatchetHeader>,
}

/// Ratchet header
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct RatchetHeader {
    pub dh_public: EncryptionPublicKey,
    pub message_number: u32,
    pub previous_chain_length: u32,
}

/// Transfer status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum TransferStatus {
    Pending,
    Connecting,
    Downloading,
    Uploading,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

/// Transfer progress
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct TransferProgress {
    pub transfer_id: String,
    pub blob_hash: String,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub status: TransferStatus,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

/// Media session type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum MediaSessionType {
    VoiceCall,
    VideoCall,
    ScreenShare,
    GroupCall,
}

/// Media direction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum MediaDirection {
    SendOnly,
    RecvOnly,
    SendRecv,
    Inactive,
}

/// Video quality
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct VideoQuality {
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub bitrate: u32,
    pub codec: VideoCodec,
    pub hardware_accel: bool,
}

/// Audio quality
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct AudioQuality {
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate: u32,
    pub codec: AudioCodec,
    pub echo_cancellation: bool,
    pub noise_suppression: bool,
    pub auto_gain_control: bool,
}

/// Screen quality
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct ScreenQuality {
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub bitrate: u32,
    pub codec: VideoCodec,
    pub lossless: bool,
}

/// Media quality
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct MediaQuality {
    pub video: VideoQuality,
    pub audio: AudioQuality,
    pub screen: ScreenQuality,
}

impl Default for MediaQuality {
    fn default() -> Self {
        Self {
            video: VideoQuality {
                resolution: (3840, 2160),
                framerate: 60,
                bitrate: 25_000_000,
                codec: VideoCodec::AV1,
                hardware_accel: true,
            },
            audio: AudioQuality {
                sample_rate: 48000,
                channels: 2,
                bitrate: 510_000,
                codec: AudioCodec::Opus,
                echo_cancellation: true,
                noise_suppression: true,
                auto_gain_control: true,
            },
            screen: ScreenQuality {
                resolution: (3840, 2160),
                framerate: 30,
                bitrate: 50_000_000,
                codec: VideoCodec::AV1,
                lossless: false,
            },
        }
    }
}

/// Video codec
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum VideoCodec {
    AV1,
    VP9,
    VP8,
    H264,
    H265,
}

/// Audio codec
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum AudioCodec {
    Opus,
    PCMU,
    PCMA,
    G722,
    ISAC,
    ILBC,
}

/// Call state
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum CallState {
    Idle,
    Connecting,
    Ringing,
    Answering,
    Connected,
    Holding,
    Ending,
    Ended,
    Failed,
}

/// Call info
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct CallInfo {
    pub call_id: String,
    pub session_type: MediaSessionType,
    pub state: CallState,
    pub local_user: String,
    pub remote_user: String,
    pub started_at: i64,
    pub connected_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub quality: MediaQuality,
}

/// Audio stats
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct AudioStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub jitter_ms: f32,
    pub rtt_ms: f32,
    pub bitrate_bps: u32,
    pub codec: String,
    pub concealment_events: u32,
}

/// Video stats
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct VideoStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_lost: u64,
    pub jitter_ms: f32,
    pub rtt_ms: f32,
    pub bitrate_bps: u32,
    pub framerate: f32,
    pub resolution: (u32, u32),
    pub codec: String,
    pub frames_encoded: u64,
    pub frames_decoded: u64,
    pub frames_dropped: u64,
    pub encode_time_ms: f32,
    pub decode_time_ms: f32,
}

/// Connection stats
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct ConnectionStats {
    pub local_candidate_type: String,
    pub remote_candidate_type: String,
    pub transport: String,
    pub dtls_cipher: String,
    pub srtp_cipher: String,
    pub selected_candidate_pair: String,
}

/// Media stats
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct MediaStats {
    pub audio: AudioStats,
    pub video: VideoStats,
    pub screen: VideoStats,
    pub connection: ConnectionStats,
}

/// Audio device
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_default: bool,
    pub sample_rates: Vec<u32>,
    pub channel_counts: Vec<u8>,
}

/// Video device
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct VideoDevice {
    pub id: String,
    pub name: String,
    pub formats: Vec<VideoFormat>,
    pub is_default: bool,
}

/// Video format
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct VideoFormat {
    pub width: u32,
    pub height: u32,
    pub framerate: u32,
    pub format: String,
}

/// Media constraints
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct MediaConstraints {
    pub audio: Option<AudioConstraints>,
    pub video: Option<VideoConstraints>,
    pub screen: Option<ScreenConstraints>,
}

/// Audio constraints
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct AudioConstraints {
    pub device_id: Option<String>,
    pub echo_cancellation: Option<bool>,
    pub noise_suppression: Option<bool>,
    pub auto_gain_control: Option<bool>,
    pub sample_rate: Option<u32>,
    pub channel_count: Option<u8>,
    pub bitrate: Option<u32>,
}

/// Video constraints
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct VideoConstraints {
    pub device_id: Option<String>,
    pub width: Option<(u32, u32)>,
    pub height: Option<(u32, u32)>,
    pub framerate: Option<(u32, u32)>,
    pub facing_mode: Option<String>,
    pub bitrate: Option<u32>,
    pub codec: Option<VideoCodec>,
}

/// Screen constraints
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct ScreenConstraints {
    pub width: Option<(u32, u32)>,
    pub height: Option<(u32, u32)>,
    pub framerate: Option<(u32, u32)>,
    pub capture_cursor: Option<bool>,
    pub window_id: Option<u32>,
    pub region: Option<(i32, i32, u32, u32)>,
    pub bitrate: Option<u32>,
    pub lossless: Option<bool>,
}

/// Contact
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct Contact {
    pub identity: IdentityPublicKey,
    pub alias: String,
    pub is_online: bool,
    pub last_seen: Option<i64>,
    pub capabilities: PeerCapabilities,
}

/// Message
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub sender: IdentityPublicKey,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: i64,
    pub is_read: bool,
    pub is_delivered: bool,
    pub attachments: Vec<Attachment>,
}

/// Attachment
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub thumbnail: Option<Vec<u8>>,
    pub blob_hash: String,
}

/// Group
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub creator: IdentityPublicKey,
    pub members: Vec<GroupMember>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Group member
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct GroupMember {
    pub identity: IdentityPublicKey,
    pub alias: String,
    pub role: GroupRole,
    pub joined_at: i64,
}

/// Group role
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum GroupRole {
    Owner,
    Admin,
    Member,
}

/// Core config
#[derive(Clone, Debug, Serialize, Deserialize, Default, uniffi::Record)]
pub struct CoreConfig {
    pub data_dir: String,
    pub enable_logging: bool,
    pub log_level: String,
    pub auto_accept_calls: bool,
    pub auto_download_media: bool,
}

/// Call event
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum CallEvent {
    Incoming,
    Ringing,
    Connected,
    Ended,
    Failed,
    Muted,
    Unmuted,
}

/// Contact event
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, uniffi::Enum)]
pub enum ContactEvent {
    Added,
    Removed,
    Updated,
    Online,
    Offline,
}

/// Signaling message
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Enum)]
pub enum SignalingMessage {
    Offer(SdpOffer),
    Answer(SdpAnswer),
    IceCandidate(IceCandidateMessage),
    End(SessionEnd),
    Reject(SessionReject),
}

/// SDP offer
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct SdpOffer {
    pub session_id: String,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub sdp: String,
    pub timestamp: i64,
}

/// SDP answer
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct SdpAnswer {
    pub session_id: String,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub sdp: String,
    pub timestamp: i64,
}

/// ICE candidate
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: String,
    pub sdp_mline_index: u32,
    pub username_fragment: Option<String>,
}

/// ICE candidate message
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct IceCandidateMessage {
    pub session_id: String,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub candidate: IceCandidate,
    pub timestamp: i64,
}

/// Session end
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct SessionEnd {
    pub session_id: String,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub reason: String,
}

/// Session reject
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Record)]
pub struct SessionReject {
    pub session_id: String,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub reason: String,
}