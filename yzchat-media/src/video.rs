//! Video Engine: Capture, encode (AV1/VP9/H264), decode, render

use crate::{MediaError, Result, VideoCodec, CodecParameters};
use anyhow::Context;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use rav1e::{Encoder as Av1Encoder, EncoderConfig as Av1Config, Frame as Av1Frame, Packet as Av1Packet};
use dav1d::{Decoder as Av1Decoder, DecoderSettings as Av1DecoderSettings, Picture as Av1Picture};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, debug, warn};

/// Video configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VideoConfig {
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Frame rate (fps)
    pub framerate: u32,
    /// Bitrate (bps)
    pub bitrate: u32,
    /// Max bitrate (bps)
    pub max_bitrate: u32,
    /// Codec
    pub codec: VideoCodec,
    /// Keyframe interval (frames)
    pub keyframe_interval: u32,
    /// Enable hardware acceleration
    pub hardware_accel: bool,
    /// Hardware decoder preference
    pub hw_decoder: Option<String>,
    /// Color format
    pub color_format: VideoColorFormat,
    /// HDR support
    pub hdr: bool,
    /// Latency mode
    pub latency_mode: LatencyMode,
    /// Simulcast layers
    pub simulcast_layers: u8,
    /// SVC layers
    pub svc_layers: u8,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
            framerate: 60,
            bitrate: 25_000_000,
            max_bitrate: 30_000_000,
            codec: VideoCodec::AV1,
            keyframe_interval: 60,
            hardware_accel: true,
            hw_decoder: None,
            color_format: VideoColorFormat::I420,
            hdr: false,
            latency_mode: LatencyMode::Low,
            simulcast_layers: 3,
            svc_layers: 3,
        }
    }
}

/// Video color format
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum VideoColorFormat {
    I420,
    NV12,
    YUY2,
    RGB,
    RGBA,
    P010, // 10-bit
}

/// Latency mode
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum LatencyMode {
    UltraLow,  // < 50ms
    Low,       // < 100ms
    Normal,    // < 200ms
    High,      // < 500ms
}

/// Video track for WebRTC
pub struct VideoTrack {
    config: VideoConfig,
    av1_encoder: Mutex<Option<Av1Encoder>>,
    av1_decoder: Mutex<Option<Av1Decoder>>,
    frame_counter: Mutex<u64>,
}

impl VideoTrack {
    /// Create new video track
    pub fn new(config: VideoConfig) -> Result<Self> {
        // Initialize AV1 encoder
        let mut av1_config = Av1Config::new();
        av1_config.width = config.width as usize;
        av1_config.height = config.height as usize;
        av1_config.bit_depth = if matches!(config.color_format, VideoColorFormat::P010) { 10 } else { 8 };
        av1_config.chroma_sampling = rav1e::ChromaSampling::Cs420;
        av1_config.speed_settings = rav1e::SpeedSettings::from_preset(6); // Balanced
        av1_config.quantizer = 30;
        av1_config.min_quantizer = 10;
        av1_config.max_quantizer = 50;
        av1_config.bitrate = config.bitrate as usize;
        av1_config.max_bitrate = config.max_bitrate as usize;
        av1_config.key_frame_interval = config.keyframe_interval as usize;
        av1_config.low_latency = matches!(config.latency_mode, LatencyMode::UltraLow | LatencyMode::Low);

        let av1_encoder = Av1Encoder::new(av1_config)
            .context("Failed to create AV1 encoder")?;

        // Initialize AV1 decoder
        let mut decoder_settings = Av1DecoderSettings::default();
        decoder_settings.max_frame_delay = 0; // Low latency
        decoder_settings.n_threads = num_cpus::get();
        let av1_decoder = Av1Decoder::new(decoder_settings)
            .context("Failed to create AV1 decoder")?;

        Ok(Self {
            config,
            av1_encoder: Mutex::new(Some(av1_encoder)),
            av1_decoder: Mutex::new(Some(av1_decoder)),
            frame_counter: Mutex::new(0),
        })
    }

    /// Encode video frame (raw YUV) to AV1
    pub async fn encode(&self, y_plane: &[u8], u_plane: &[u8], v_plane: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = self.av1_encoder.lock().await;
        let encoder = encoder.as_mut().ok_or(MediaError::EncoderNotInitialized)?;

        let mut frame = Av1Frame::new(encoder.config().width, encoder.config().height);
        frame.planes[0].copy_from_slice(y_plane);
        frame.planes[1].copy_from_slice(u_plane);
        frame.planes[2].copy_from_slice(v_plane);

        let mut frame_counter = self.frame_counter.lock().await;
        *frame_counter += 1;
        let frame_number = *frame_counter;

        // Force keyframe at interval
        let force_keyframe = frame_number % self.config.keyframe_interval as u64 == 0;

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

    /// Decode AV1 packet to YUV
    pub async fn decode(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let mut decoder = self.av1_decoder.lock().await;
        let decoder = decoder.as_mut().ok_or(MediaError::DecoderNotInitialized)?;

        decoder.send_data(data).context("Failed to send data")?;

        let picture = decoder.get_picture().context("Failed to get picture")?;

        let y_plane = picture.planes[0].to_vec();
        let u_plane = picture.planes[1].to_vec();
        let v_plane = picture.planes[2].to_vec();

        Ok((y_plane, u_plane, v_plane))
    }

    /// Get config
    pub fn config(&self) -> &VideoConfig {
        &self.config
    }
}

/// Video capture from camera
pub struct VideoCapture {
    config: VideoConfig,
    pipeline: Option<gst::Pipeline>,
    appsink: Option<gst_app::AppSink>,
    frame_tx: mpsc::Sender<(Vec<u8>, Vec<u8>, Vec<u8>)>,
}

impl VideoCapture {
    /// Create new video capture
    pub async fn new(config: VideoConfig) -> Result<(Self, mpsc::Receiver<(Vec<u8>, Vec<u8>, Vec<u8>)>)> {
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

        // Use v4l2src for Linux, avfvideosrc for macOS, ksvideosrc for Windows
        // For Android: use camerabin or custom source
        let source = if cfg!(target_os = "linux") {
            "v4l2src device=/dev/video0"
        } else if cfg!(target_os = "macos") {
            "avfvideosrc"
        } else if cfg!(target_os = "windows") {
            "ksvideosrc"
        } else if cfg!(target_os = "android") {
            // Android camera source would use JNI bridge
            "androidcamerasrc"
        } else {
            "videotestsrc" // Fallback
        };

        let pipeline_str = format!(
            "{} ! \
             video/x-raw,format=I420,width={},height={},framerate={}/1 ! \
             videoconvert ! \
             video/x-raw,format=I420 ! \
             appsink name=sink emit-signals=true max-buffers=5 drop=true",
            source, self.config.width, self.config.height, self.config.framerate
        );

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

                    // Parse I420 planes
                    let info = gst_video::VideoInfo::from_caps(sample.caps().unwrap())?;
                    let frame = gst_video::VideoFrameRef::from_buffer_ref_readonly(buffer, &info)
                        .map_err(|_| gst::FlowError::Error)?;

                    let y_plane = frame.plane_data(0).unwrap_or_default().to_vec();
                    let u_plane = frame.plane_data(1).unwrap_or_default().to_vec();
                    let v_plane = frame.plane_data(2).unwrap_or_default().to_vec();

                    let _ = frame_tx.try_send((y_plane, u_plane, v_plane));

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

/// Video rendering to screen
pub struct VideoRender {
    config: VideoConfig,
    pipeline: Option<gst::Pipeline>,
    appsrc: Option<gst_app::AppSrc>,
}

impl VideoRender {
    /// Create new video render
    pub async fn new(config: VideoConfig) -> Result<Self> {
        Ok(Self {
            config,
            pipeline: None,
            appsrc: None,
        })
    }

    /// Start render
    pub async fn start(&mut self) -> Result<()> {
        gst::init().context("Failed to initialize GStreamer")?;

        // Use glimagesink for hardware accelerated, or autovideosink
        let sink = if cfg!(target_os = "android") {
            "androidvideosink"
        } else if cfg!(target_os = "linux") {
            "glimagesink"
        } else if cfg!(target_os = "macos") {
            "vtdec ! videoconvert ! osxvideosink"
        } else if cfg!(target_os = "windows") {
            "d3d11videosink"
        } else {
            "autovideosink"
        };

        let pipeline_str = format!(
            "appsrc name=src format=time is-live=true ! \
             video/x-raw,format=I420,width={},height={},framerate={}/1 ! \
             videoconvert ! \
             {}",
            self.config.width, self.config.height, self.config.framerate, sink
        );

        let pipeline = gst::parse::launch(&pipeline_str)
            .context("Failed to create pipeline")?
            .downcast::<gst::Pipeline>()
            .map_err(|_| MediaError::PipelineCreationFailed)?;

        let appsrc = pipeline.by_name("src")
            .ok_or(MediaError::ElementNotFound("appsrc".into()))?
            .downcast::<gst_app::AppSrc>()
            .map_err(|_| MediaError::ElementCastFailed)?;

        appsrc.set_caps(&gst_video::VideoCapsBuilder::new()
            .format(gst_video::VideoFormat::I420)
            .width(self.config.width as i32)
            .height(self.config.height as i32)
            .framerate(gst::Fraction::new(self.config.framerate as i32, 1))
            .build());

        pipeline.set_state(gst::State::Playing)
            .context("Failed to start pipeline")?;

        self.pipeline = Some(pipeline);
        self.appsrc = Some(appsrc);

        Ok(())
    }

    /// Push YUV frame
    pub async fn push_frame(&self, y_plane: &[u8], u_plane: &[u8], v_plane: &[u8]) -> Result<()> {
        let appsrc = self.appsrc.as_ref().ok_or(MediaError::NotStarted)?;

        // Create buffer with three planes
        let mut buffer = gst::Buffer::with_size(y_plane.len() + u_plane.len() + v_plane.len())
            .map_err(|_| MediaError::BufferCreationFailed)?;

        {
            let mut map = buffer.map_writable().map_err(|_| MediaError::BufferMapFailed)?;
            let mut offset = 0;
            map.as_mut_slice()[offset..offset + y_plane.len()].copy_from_slice(y_plane);
            offset += y_plane.len();
            map.as_mut_slice()[offset..offset + u_plane.len()].copy_from_slice(u_plane);
            offset += u_plane.len();
            map.as_mut_slice()[offset..offset + v_plane.len()].copy_from_slice(v_plane);
        }

        // Set buffer metadata
        {
            let mut buffer = buffer.make_mut();
            buffer.set_pts(gst::ClockTime::NONE);
            buffer.set_dts(gst::ClockTime::NONE);
            buffer.set_duration(gst::ClockTime::from_nseconds(
                (1_000_000_000 / self.config.framerate as u64) as u64
            ));
        }

        appsrc.push_buffer(buffer).map_err(|_| MediaError::PushBufferFailed)?;
        Ok(())
    }

    /// Stop render
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null)?;
        }
        self.pipeline = None;
        self.appsrc = None;
        Ok(())
    }
}

/// Video engine
pub struct VideoEngine {
    config: VideoConfig,
    track: Option<VideoTrack>,
    capture: Option<VideoCapture>,
    render: Option<VideoRender>,
    capture_rx: Option<mpsc::Receiver<(Vec<u8>, Vec<u8>, Vec<u8>)>>,
    running: bool,
}

impl VideoEngine {
    /// Create new video engine
    pub fn new(config: VideoConfig) -> Self {
        Self {
            config,
            track: None,
            capture: None,
            render: None,
            capture_rx: None,
            running: false,
        }
    }

    /// Initialize
    pub async fn initialize(&mut self) -> Result<()> {
        self.track = Some(VideoTrack::new(self.config.clone())?);
        let (capture, rx) = VideoCapture::new(self.config.clone()).await?;
        self.capture = Some(capture);
        self.capture_rx = Some(rx);
        self.render = Some(VideoRender::new(self.config.clone()).await?);
        Ok(())
    }

    /// Start
    pub async fn start(&mut self) -> Result<()> {
        if self.running { return Ok(()); }

        self.capture.as_mut().unwrap().start().await?;
        self.render.as_mut().unwrap().start().await?;
        self.running = true;

        let mut capture_rx = self.capture_rx.take().unwrap();
        let track = self.track.as_ref().unwrap();
        let render = self.render.as_ref().unwrap();

        tokio::spawn(async move {
            while let Some((y, u, v)) = capture_rx.recv().await {
                // Encode for network
                let encoded = track.encode(&y, &u, &v).await.ok();
                // In real impl: send encoded to network

                // Render locally
                render.push_frame(&y, &u, &v).await.ok();
            }
        });

        Ok(())
    }

    /// Stop
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running { return Ok(()); }
        self.capture.as_mut().unwrap().stop().await?;
        self.render.as_mut().unwrap().stop().await?;
        self.running = false;
        Ok(())
    }

    /// Process incoming encoded video
    pub async fn process_incoming(&self, data: &[u8]) -> Result<()> {
        let track = self.track.as_ref().ok_or(MediaError::NotInitialized)?;
        let (y, u, v) = track.decode(data).await?;
        self.render.as_ref().unwrap().push_frame(&y, &u, &v).await
    }

    /// Get track for WebRTC
    pub fn track(&self) -> Option<&VideoTrack> {
        self.track.as_ref()
    }
}