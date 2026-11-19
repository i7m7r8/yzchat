//! Screen Share Engine: High-resolution screen capture and encoding

use crate::{MediaError, Result, VideoConfig, VideoCodec, VideoColorFormat, LatencyMode};
use anyhow::Context;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use rav1e::{Encoder as Av1Encoder, EncoderConfig as Av1Config};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, debug, warn};

/// Screen share configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScreenShareConfig {
    /// Video config (inherits from VideoConfig)
    pub video: VideoConfig,
    /// Capture cursor
    pub capture_cursor: bool,
    /// Capture specific window (None = full screen)
    pub window_id: Option<u32>,
    /// Capture region (x, y, width, height)
    pub region: Option<(i32, i32, u32, u32)>,
    /// Frame rate limit (fps)
    pub framerate_limit: u32,
    /// Enable lossless mode (for text clarity)
    pub lossless: bool,
    /// Downscale factor (1 = native, 2 = half resolution)
    pub downscale: u32,
}

impl Default for ScreenShareConfig {
    fn default() -> Self {
        let mut video = VideoConfig::default();
        video.width = 3840;
        video.height = 2160;
        video.framerate = 30;
        video.bitrate = 50_000_000;
        video.max_bitrate = 60_000_000;
        video.codec = VideoCodec::AV1;
        video.keyframe_interval = 30;
        video.latency_mode = LatencyMode::Low;
        video.simulcast_layers = 1;
        video.svc_layers = 1;

        Self {
            video,
            capture_cursor: true,
            window_id: None,
            region: None,
            framerate_limit: 30,
            lossless: false,
            downscale: 1,
        }
    }
}

/// Screen track for WebRTC
pub struct ScreenTrack {
    config: ScreenShareConfig,
    av1_encoder: Mutex<Option<Av1Encoder>>,
    frame_counter: Mutex<u64>,
}

impl ScreenTrack {
    /// Create new screen track
    pub fn new(config: ScreenShareConfig) -> Result<Self> {
        let mut av1_config = Av1Config::new();
        av1_config.width = config.video.width as usize / config.downscale as usize;
        av1_config.height = config.video.height as usize / config.downscale as usize;
        av1_config.bit_depth = 8;
        av1_config.chroma_sampling = rav1e::ChromaSampling::Cs420;
        av1_config.speed_settings = rav1e::SpeedSettings::from_preset(4); // Higher quality for screen
        av1_config.quantizer = if config.lossless { 0 } else { 20 };
        av1_config.min_quantizer = if config.lossless { 0 } else { 5 };
        av1_config.max_quantizer = if config.lossless { 0 } else { 40 };
        av1_config.bitrate = config.video.bitrate as usize;
        av1_config.max_bitrate = config.video.max_bitrate as usize;
        av1_config.key_frame_interval = config.video.keyframe_interval as usize;
        av1_config.low_latency = true;
        av1_config.tiles = 4; // Parallel encoding

        let av1_encoder = Av1Encoder::new(av1_config)
            .context("Failed to create AV1 encoder for screen")?;

        Ok(Self {
            config,
            av1_encoder: Mutex::new(Some(av1_encoder)),
            frame_counter: Mutex::new(0),
        })
    }

    /// Encode screen frame (RGB/RGBA) to AV1
    pub async fn encode(&self, rgb_data: &[u8], stride: usize) -> Result<Vec<u8>> {
        let mut encoder = self.av1_encoder.lock().await;
        let encoder = encoder.as_mut().ok_or(MediaError::EncoderNotInitialized)?;

        // Convert RGB to I420 (simplified - real impl uses videoconvert)
        let (y_plane, u_plane, v_plane) = Self::rgb_to_i420(
            rgb_data,
            config.video.width as usize / config.downscale as usize,
            config.video.height as usize / config.downscale as usize,
            stride,
        )?;

        let mut frame = Av1Frame::new(encoder.config().width, encoder.config().height);
        frame.planes[0].copy_from_slice(&y_plane);
        frame.planes[1].copy_from_slice(&u_plane);
        frame.planes[2].copy_from_slice(&v_plane);

        let mut frame_counter = self.frame_counter.lock().await;
        *frame_counter += 1;
        let frame_number = *frame_counter;

        // Force keyframe at interval
        let force_keyframe = frame_number % self.config.video.keyframe_interval as u64 == 0;

        encoder.send_frame(frame).context("Failed to send frame")?;

        let mut output = Vec::new();
        while let Ok(packet) = encoder.receive_packet() {
            output.extend_from_slice(&packet.data);
            if packet.frame_type == rav1e::FrameType::KEY && force_keyframe {
                break;
            }
        }

        Ok(output)
    }

    /// RGB to I420 conversion (simplified)
    fn rgb_to_i420(rgb: &[u8], width: usize, height: usize, stride: usize) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let y_size = width * height;
        let uv_size = (width / 2) * (height / 2);
        let mut y = vec![0u8; y_size];
        let mut u = vec![0u8; uv_size];
        let mut v = vec![0u8; uv_size];

        // Simple conversion (real impl uses libyuv or GStreamer videoconvert)
        for y_idx in 0..height {
            for x_idx in 0..width {
                let src_idx = y_idx * stride + x_idx * 4;
                if src_idx + 2 >= rgb.len() { continue; }
                let r = rgb[src_idx] as f32;
                let g = rgb[src_idx + 1] as f32;
                let b = rgb[src_idx + 2] as f32;

                let dst_idx = y_idx * width + x_idx;
                y[dst_idx] = (0.299 * r + 0.587 * g + 0.114 * b) as u8;

                if y_idx % 2 == 0 && x_idx % 2 == 0 {
                    let uv_idx = (y_idx / 2) * (width / 2) + (x_idx / 2);
                    u[uv_idx] = (-0.169 * r - 0.331 * g + 0.500 * b + 128.0) as u8;
                    v[uv_idx] = (0.500 * r - 0.419 * g - 0.081 * b + 128.0) as u8;
                }
            }
        }

        Ok((y, u, v))
    }

    /// Get config
    pub fn config(&self) -> &ScreenShareConfig {
        &self.config
    }
}

/// Screen capture
pub struct ScreenCapture {
    config: ScreenShareConfig,
    pipeline: Option<gst::Pipeline>,
    appsink: Option<gst_app::AppSink>,
    frame_tx: mpsc::Sender<(Vec<u8>, usize)>, // (RGB data, stride)
}

impl ScreenCapture {
    /// Create new screen capture
    pub async fn new(config: ScreenShareConfig) -> Result<(Self, mpsc::Receiver<(Vec<u8>, usize)>)> {
        let (frame_tx, frame_rx) = mpsc::channel(30);

        Ok((Self {
            config,
            pipeline: None,
            appsink: None,
            frame_tx,
        }, frame_rx))
    }

    /// Start capture
    pub async fn start(&mut self) -> Result<()> {
        gst::init().context("Failed to initialize GStreamer")?;

        // Platform-specific screen capture
        let source = if cfg!(target_os = "linux") {
            // Use pipewiresrc for Wayland, ximagesrc for X11
            "pipewiresrc"
        } else if cfg!(target_os = "macos") {
            "avfvideosrc capture-screen=true"
        } else if cfg!(target_os = "windows") {
            "dxgiscreencapturesrc"
        } else if cfg!(target_os = "android") {
            // Android: use MediaProjection via JNI
            "androidscreensrc"
        } else {
            "videotestsrc pattern=ball"
        };

        let mut pipeline_str = format!(
            "{} ! \
             video/x-raw,format=RGBA,width={},height={},framerate={}/1 ! \
             videoconvert ! \
             video/x-raw,format=RGBA ! \
             appsink name=sink emit-signals=true max-buffers=5 drop=true",
            source,
            self.config.video.width / self.config.downscale,
            self.config.video.height / self.config.downscale,
            self.config.framerate_limit
        );

        if let Some((x, y, w, h)) = self.config.region {
            pipeline_str = format!(
                "{} ! \
                 video/x-raw,format=RGBA ! \
                 videocrop left={} right={} top={} bottom={} ! \
                 videoscale ! \
                 video/x-raw,width={},height={} ! \
                 videoconvert ! \
                 video/x-raw,format=RGBA ! \
                 appsink name=sink emit-signals=true max-buffers=5 drop=true",
                source, x, x + w as i32, y, y + h as i32,
                self.config.video.width / self.config.downscale,
                self.config.video.height / self.config.downscale
            );
        }

        let pipeline = gst::parse::launch(&pipeline_str)
            .context("Failed to create pipeline")?
            .downcast::<gst::Pipeline>()
            .map_err(|_| MediaError::PipelineCreationFailed)?;

        let appsink = pipeline.by_name("sink")
            .ok_or(MediaError::ElementNotFound("appsink".into()))?
            .downcast::<gst_app::AppSink>()
            .map_err(|_| MediaError::ElementCastFailed)?;

        let frame_tx = self.frame_tx.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;

                    let info = gst_video::VideoInfo::from_caps(sample.caps().unwrap())?;
                    let stride = info.stride()[0] as usize;
                    let data = map.as_slice().to_vec();

                    let _ = frame_tx.try_send((data, stride));

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline.set_state(gst::State::Playing)
            .context("Failed to start pipeline")?;

        self.pipeline = Some(pipeline);
        self.appsink = Some(appsink);

        Ok(())
    }

    /// Stop capture
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null)?;
        }
        self.pipeline = None;
        self.appsink = None;
        Ok(())
    }
}

/// Screen share engine
pub struct ScreenShareEngine {
    config: ScreenShareConfig,
    track: Option<ScreenTrack>,
    capture: Option<ScreenCapture>,
    capture_rx: Option<mpsc::Receiver<(Vec<u8>, usize)>>,
    running: bool,
}

impl ScreenShareEngine {
    /// Create new screen share engine
    pub fn new(config: ScreenShareConfig) -> Self {
        Self {
            config,
            track: None,
            capture: None,
            capture_rx: None,
            running: false,
        }
    }

    /// Initialize
    pub async fn initialize(&mut self) -> Result<()> {
        self.track = Some(ScreenTrack::new(self.config.clone())?);
        let (capture, rx) = ScreenCapture::new(self.config.clone()).await?;
        self.capture = Some(capture);
        self.capture_rx = Some(rx);
        Ok(())
    }

    /// Start
    pub async fn start(&mut self) -> Result<()> {
        if self.running { return Ok(()); }

        self.capture.as_mut().unwrap().start().await?;
        self.running = true;

        let mut capture_rx = self.capture_rx.take().unwrap();
        let track = self.track.as_ref().unwrap();

        tokio::spawn(async move {
            while let Some((rgb_data, stride)) = capture_rx.recv().await {
                // Encode for network
                let encoded = track.encode(&rgb_data, stride).await.ok();
                // In real impl: send encoded to network via WebRTC
            }
        });

        Ok(())
    }

    /// Stop
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running { return Ok(()); }
        self.capture.as_mut().unwrap().stop().await?;
        self.running = false;
        Ok(())
    }

    /// Get track for WebRTC
    pub fn track(&self) -> Option<&ScreenTrack> {
        self.track.as_ref()
    }
}