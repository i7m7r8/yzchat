//! Common types for yzchat-media

use crate::{VideoCodec, AudioCodec, VideoColorFormat, LatencyMode};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Media session types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaSessionType {
    VoiceCall,
    VideoCall,
    ScreenShare,
    GroupCall,
}

/// Media direction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaDirection {
    SendOnly,
    RecvOnly,
    SendRecv,
    Inactive,
}

/// Media quality settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaQuality {
    pub video: VideoQuality,
    pub audio: AudioQuality,
    pub screen: ScreenQuality,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoQuality {
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub bitrate: u32,
    pub codec: VideoCodec,
    pub hardware_accel: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioQuality {
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate: u32,
    pub codec: AudioCodec,
    pub echo_cancellation: bool,
    pub noise_suppression: bool,
    pub auto_gain_control: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScreenQuality {
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub bitrate: u32,
    pub codec: VideoCodec,
    pub lossless: bool,
}

/// Media statistics
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MediaStats {
    pub audio: AudioStats,
    pub video: VideoStats,
    pub screen: VideoStats,
    pub connection: ConnectionStats,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ConnectionStats {
    pub local_candidate_type: String,
    pub remote_candidate_type: String,
    pub transport: String,
    pub dtls_cipher: String,
    pub srtp_cipher: String,
    pub selected_candidate_pair: String,
}

/// Call state
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallInfo {
    pub call_id: Uuid,
    pub session_type: MediaSessionType,
    pub state: CallState,
    pub local_user: String,
    pub remote_user: String,
    pub started_at: DateTime<Utc>,
    pub connected_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub quality: MediaQuality,
    pub stats: MediaStats,
}

/// Device info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_default: bool,
    pub sample_rates: Vec<u32>,
    pub channel_counts: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoDevice {
    pub id: String,
    pub name: String,
    pub formats: Vec<VideoFormat>,
    pub is_default: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoFormat {
    pub width: u32,
    pub height: u32,
    pub framerate: u32,
    pub format: VideoColorFormat,
}

/// Media constraints for getUserMedia-style API
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MediaConstraints {
    pub audio: Option<AudioConstraints>,
    pub video: Option<VideoConstraints>,
    pub screen: Option<ScreenConstraints>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioConstraints {
    pub device_id: Option<String>,
    pub echo_cancellation: Option<bool>,
    pub noise_suppression: Option<bool>,
    pub auto_gain_control: Option<bool>,
    pub sample_rate: Option<u32>,
    pub channel_count: Option<u8>,
    pub bitrate: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoConstraints {
    pub device_id: Option<String>,
    pub width: Option<(u32, u32)>, // (min, max) or exact
    pub height: Option<(u32, u32)>,
    pub framerate: Option<(u32, u32)>,
    pub facing_mode: Option<String>, // "user" | "environment"
    pub bitrate: Option<u32>,
    pub codec: Option<VideoCodec>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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