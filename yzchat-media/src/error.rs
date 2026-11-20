//! Error types for yzchat-media

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("GStreamer error: {0}")]
    GStreamer(#[from] gstreamer::Error),

    #[error("GStreamer boolean error: {0}")]
    GStreamerBool(#[from] gstreamer::BoolError),

    #[error("Pipeline creation failed: {0}")]
    PipelineCreationFailed(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Element cast failed: {0}")]
    ElementCastFailed,

    #[error("Buffer creation failed")]
    BufferCreationFailed,

    #[error("Buffer map failed")]
    BufferMapFailed,

    #[error("Push buffer failed")]
    PushBufferFailed,

    #[error("Encoder not initialized")]
    EncoderNotInitialized,

    #[error("Decoder not initialized")]
    DecoderNotInitialized,

    #[error("Not initialized")]
    NotInitialized,

    #[error("Not started")]
    NotStarted,

    #[error("Opus error: {0}")]
    Opus(#[from] opus::Error),

    #[error("RAV1E error: {0}")]
    Rav1e(#[from] rav1e::EncoderStatus),

    #[error("WebRTC error: {0}")]
    WebRtc(#[from] webrtc::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other media error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, MediaError>;
