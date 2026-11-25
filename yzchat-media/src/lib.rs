//! yzchat-media: Media Engine for 4K Video, Crystal Clear Audio, Screen Share
//!
//! Features:
//! - WebRTC with DTLS-SRTP encryption
//! - Opus audio (48kHz stereo, up to 510 kbps)
//! - AV1 video (4K@60fps, 10-bit HDR)
//! - Screen share (4K@30fps, lossless option)
//! - Simulcast / SVC for adaptive bitrate
//! - NACK, PLI, FEC for reliability
//! - Echo cancellation, noise suppression, auto gain control
//! - Hardware acceleration (VideoToolbox, MediaCodec, VAAPI, NVENC)

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]

pub mod audio;
pub mod capture;
pub mod codecs;
pub mod error;
pub mod screen_share;
pub mod types;
pub mod video;
pub mod webrtc_engine;

pub use audio::{AudioConfig, AudioEngine, AudioTrack};
pub use capture::{AudioCapture, ScreenCapture, VideoCapture};
pub use codecs::{AudioCodec, CodecParameters, VideoCodec};
pub use error::{MediaError, Result};
pub use screen_share::{ScreenShareConfig, ScreenShareEngine, ScreenTrack};
pub use types::*;
pub use video::{VideoConfig, VideoEngine, VideoTrack};
pub use webrtc_engine::{PeerConnection, WebRtcConfig, WebRtcEngine};

/// Media engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported video resolutions
pub mod resolutions {
    pub const HD_720: (u32, u32) = (1280, 720);
    pub const FHD_1080: (u32, u32) = (1920, 1080);
    pub const QHD_1440: (u32, u32) = (2560, 1440);
    pub const UHD_4K: (u32, u32) = (3840, 2160);
    pub const UHD_8K: (u32, u32) = (7680, 4320);
}

/// Default codec preferences (highest quality first)
pub mod codec_prefs {
    pub const VIDEO: &[&str] = &[
        "AV1",  // Best quality/bitrate
        "VP9",  // Good fallback
        "H264", // Universal compatibility
    ];
    pub const AUDIO: &[&str] = &[
        "OPUS", // Best quality
        "PCMU", // Fallback
        "PCMA", // Fallback
    ];
}
