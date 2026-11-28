//! Media engine for yzchat: audio/video calls and screenshare
//!
//! Opus audio, AV1/rav1e video, and end-to-end encrypted media frames.
//! Transport is provided by yzchat-p2p (Iroh QUIC streams).

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// A single Opus-encoded audio frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioFrame {
    pub sequence: u64,
    pub timestamp: u64,
    pub payload: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// A single video frame (AV1 encoded via rav1e).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoFrame {
    pub sequence: u64,
    pub timestamp: u64,
    pub payload: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub keyframe: bool,
}

/// A screenshare frame (video with a region hint).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScreenShareFrame {
    pub frame: VideoFrame,
    pub region: Option<(u32, u32, u32, u32)>, // x, y, w, h
}

/// Opus audio encoder/decoder pair for a call.
pub struct AudioCodec {
    encoder: opus::Encoder,
    decoder: opus::Decoder,
    channels: usize,
}

impl AudioCodec {
    /// Create a stereo Opus encoder/decoder at 48 kHz.
    pub fn new(sample_rate: u32, channels_count: u16) -> anyhow::Result<Self> {
        let channels_count = channels_count as usize;
        let opus_channels = if channels_count == 1 {
            opus::Channels::Mono
        } else {
            opus::Channels::Stereo
        };
        let encoder = opus::Encoder::new(sample_rate as i32, opus_channels, opus::Application::Audio)
            .context("opus encoder creation failed")?;
        let decoder =
            opus::Decoder::new(sample_rate as i32, opus_channels)
                .context("opus decoder creation failed")?;
        info!(sample_rate, channels = channels_count, "audio codec initialized");
        Ok(Self { encoder, decoder, channels: channels_count })
    }

    /// Encode PCM16 samples into an Opus frame.
    pub fn encode(&mut self, samples: &[i16]) -> anyhow::Result<Vec<u8>> {
        let max_size = 4000;
        let mut out = vec![0u8; max_size];
        let len = self
            .encoder
            .encode(samples, &mut out)
            .context("opus encode failed")?;
        out.truncate(len);
        debug!(bytes = len, "audio frame encoded");
        Ok(out)
    }

    /// Decode an Opus frame into PCM16 samples.
    pub fn decode(&mut self, payload: &[u8]) -> anyhow::Result<Vec<i16>> {
        let max_frames = 960; // 20ms @48k
        let mut out = vec![0i16; max_frames * self.channels];
        let len = self
            .decoder
            .decode(payload, &mut out, false)
            .context("opus decode failed")?;
        out.truncate(len * self.channels);
        Ok(out)
    }
}

/// Rav1e AV1 video encoder configuration.
pub struct VideoEncoder {
    pub width: u32,
    pub height: u32,
}

impl VideoEncoder {
    pub fn new(width: u32, height: u32) -> Self {
        info!(width, height, "video encoder initialized");
        Self { width, height }
    }
}

/// Media manager: owns audio/video codecs for an active call.
pub struct MediaManager {
    audio: Option<AudioCodec>,
    video: Option<VideoEncoder>,
}

impl MediaManager {
    pub fn new() -> Self {
        Self {
            audio: None,
            video: None,
        }
    }

    pub fn init_audio(&mut self, sample_rate: u32, channels: u16) -> anyhow::Result<()> {
        self.audio = Some(AudioCodec::new(sample_rate, channels)?);
        Ok(())
    }

    pub fn init_video(&mut self, width: u32, height: u32) {
        self.video = Some(VideoEncoder::new(width, height));
    }

    pub fn audio(&mut self) -> Option<&mut AudioCodec> {
        self.audio.as_mut()
    }
}

impl Default for MediaManager {
    fn default() -> Self {
        Self::new()
    }
}
