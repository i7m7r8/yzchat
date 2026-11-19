//! Codec definitions and parameters

use serde::{Serialize, Deserialize};

/// Audio codec types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AudioCodec {
    Opus,
    PCMU,   // G.711 μ-law
    PCMA,   // G.711 A-law
    G722,
    ISAC,
    ILBC,
}

impl AudioCodec {
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioCodec::Opus => "audio/opus",
            AudioCodec::PCMU => "audio/PCMU",
            AudioCodec::PCMA => "audio/PCMA",
            AudioCodec::G722 => "audio/G722",
            AudioCodec::ISAC => "audio/ISAC",
            AudioCodec::ILBC => "audio/ILBC",
        }
    }

    pub fn clock_rate(&self) -> u32 {
        match self {
            AudioCodec::Opus => 48000,
            AudioCodec::PCMU => 8000,
            AudioCodec::PCMA => 8000,
            AudioCodec::G722 => 8000,
            AudioCodec::ISAC => 16000,
            AudioCodec::ILBC => 8000,
        }
    }

    pub fn channels(&self) -> u8 {
        match self {
            AudioCodec::Opus => 2,
            AudioCodec::PCMU => 1,
            AudioCodec::PCMA => 1,
            AudioCodec::G722 => 1,
            AudioCodec::ISAC => 1,
            AudioCodec::ILBC => 1,
        }
    }
}

/// Video codec types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VideoCodec {
    AV1,
    VP9,
    VP8,
    H264,
    H265,
}

impl VideoCodec {
    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoCodec::AV1 => "video/AV1",
            VideoCodec::VP9 => "video/VP9",
            VideoCodec::VP8 => "video/VP8",
            VideoCodec::H264 => "video/H264",
            VideoCodec::H265 => "video/H265",
        }
    }

    pub fn clock_rate(&self) -> u32 {
        90000 // Standard for video
    }

    pub fn supports_hdr(&self) -> bool {
        matches!(self, VideoCodec::AV1 | VideoCodec::VP9 | VideoCodec::H265)
    }

    pub fn supports_10bit(&self) -> bool {
        matches!(self, VideoCodec::AV1 | VideoCodec::VP9 | VideoCodec::H265)
    }

    pub fn supports_svc(&self) -> bool {
        matches!(self, VideoCodec::AV1 | VideoCodec::VP9 | VideoCodec::H265)
    }
}

/// Codec parameters for SDP
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodecParameters {
    pub codec: VideoCodec,
    pub profile_id: Option<String>,
    pub level_id: Option<String>,
    pub packetization_mode: Option<String>,
    pub max_framerate: Option<u32>,
    pub max_bitrate: Option<u32>,
    pub max_resolution: Option<(u32, u32)>,
    pub color_space: Option<String>,
    pub transfer_characteristics: Option<String>,
    pub matrix_coefficients: Option<String>,
}

impl Default for CodecParameters {
    fn default() -> Self {
        Self {
            codec: VideoCodec::AV1,
            profile_id: None,
            level_id: None,
            packetization_mode: None,
            max_framerate: Some(60),
            max_bitrate: Some(25_000_000),
            max_resolution: Some((3840, 2160)),
            color_space: Some("bt2020".to_string()),
            transfer_characteristics: Some("pq".to_string()),
            matrix_coefficients: Some("bt2020nc".to_string()),
        }
    }
}

/// SDP media description builder
pub struct SdpBuilder;

impl SdpBuilder {
    /// Build audio media section
    pub fn audio(codec: AudioCodec, port: u16, rtcp_port: u16) -> String {
        format!(
            "m=audio {} RTP/SAVPF {}\n\
             a=rtpmap:{} {}/{}\n\
             a=fmtp:{} minptime=10;useinbandfec=1;stereo=1\n\
             a=rtcp-mux\n\
             a=rtcp-rsize\n\
             a=sendrecv\n",
            port,
            Self::payload_type(codec),
            Self::payload_type(codec),
            codec.mime_type(),
            codec.clock_rate(),
            Self::payload_type(codec)
        )
    }

    /// Build video media section
    pub fn video(codec: VideoCodec, params: &CodecParameters, port: u16, rtcp_port: u16) -> String {
        let mut fmtp = Vec::new();
        if let Some(p) = &params.profile_id { fmtp.push(format!("profile-id={}", p)); }
        if let Some(p) = &params.level_id { fmtp.push(format!("level-id={}", p)); }
        if let Some(p) = &params.packetization_mode { fmtp.push(format!("packetization-mode={}", p)); }
        if let Some(p) = params.max_framerate { fmtp.push(format!("max-framerate={}", p)); }
        if let Some(p) = params.max_bitrate { fmtp.push(format!("max-bitrate={}", p)); }
        if let Some((w, h)) = params.max_resolution { fmtp.push(format!("max-resolution={}x{}", w, h)); }

        format!(
            "m=video {} RTP/SAVPF {}\n\
             a=rtpmap:{} {}/{}\n\
             a=fmtp:{}: {}\n\
             a=rtcp-mux\n\
             a=rtcp-rsize\n\
             a=sendrecv\n",
            port,
            Self::payload_type_video(codec),
            Self::payload_type_video(codec),
            codec.mime_type(),
            codec.clock_rate(),
            Self::payload_type_video(codec),
            fmtp.join("; ")
        )
    }

    /// Build screen share media section
    pub fn screen(codec: VideoCodec, params: &CodecParameters, port: u16, rtcp_port: u16) -> String {
        let mut fmtp = Vec::new();
        if let Some(p) = &params.profile_id { fmtp.push(format!("profile-id={}", p)); }
        if let Some(p) = &params.level_id { fmtp.push(format!("level-id={}", p)); }

        format!(
            "m=video {} RTP/SAVPF {}\n\
             a=rtpmap:{} {}/{}\n\
             a=fmtp:{}: {}\n\
             a=content:slides\n\
             a=rtcp-mux\n\
             a=rtcp-rsize\n\
             a=sendrecv\n",
            port,
            Self::payload_type_video(codec),
            Self::payload_type_video(codec),
            codec.mime_type(),
            codec.clock_rate(),
            Self::payload_type_video(codec),
            fmtp.join("; ")
        )
    }

    fn payload_type(codec: AudioCodec) -> u8 {
        match codec {
            AudioCodec::Opus => 111,
            AudioCodec::PCMU => 0,
            AudioCodec::PCMA => 8,
            AudioCodec::G722 => 9,
            AudioCodec::ISAC => 103,
            AudioCodec::ILBC => 102,
        }
    }

    fn payload_type_video(codec: VideoCodec) -> u8 {
        match codec {
            VideoCodec::AV1 => 96,
            VideoCodec::VP9 => 97,
            VideoCodec::VP8 => 98,
            VideoCodec::H264 => 99,
            VideoCodec::H265 => 100,
        }
    }
}