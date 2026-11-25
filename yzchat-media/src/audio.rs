//! Audio Engine: Capture, encode, decode, playback

use crate::{AudioCodec, CodecParameters, MediaError, Result};
use anyhow::Context;
use bytes::BytesMut;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
use opus::{Application, Channels, Decoder, Encoder};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};

/// Audio configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioConfig {
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Channels
    pub channels: u8,
    /// Bitrate (bps)
    pub bitrate: u32,
    /// Frame duration (ms)
    pub frame_duration_ms: u32,
    /// Enable DTX (discontinuous transmission)
    pub enable_dtx: bool,
    /// Enable FEC (forward error correction)
    pub enable_fec: bool,
    /// Complexity (0-10)
    pub complexity: i32,
    /// Application type
    pub application: AudioApplication,
    /// Echo cancellation
    pub echo_cancellation: bool,
    /// Noise suppression
    pub noise_suppression: bool,
    /// Auto gain control
    pub auto_gain_control: bool,
    /// Highpass filter
    pub highpass_filter: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bitrate: 510_000,
            frame_duration_ms: 20,
            enable_dtx: true,
            enable_fec: true,
            complexity: 10,
            application: AudioApplication::Voip,
            echo_cancellation: true,
            noise_suppression: true,
            auto_gain_control: true,
            highpass_filter: true,
        }
    }
}

/// Audio application type
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum AudioApplication {
    Voip,
    Audio,
    RestrictedLowdelay,
}

/// Audio track for WebRTC
pub struct AudioTrack {
    config: AudioConfig,
    encoder: Mutex<Option<Encoder>>,
    decoder: Mutex<Option<Decoder>>,
    sample_buffer: Mutex<Vec<f32>>,
}

impl AudioTrack {
    /// Create new audio track
    pub fn new(config: AudioConfig) -> Result<Self> {
        let channels = if config.channels == 1 {
            Channels::Mono
        } else {
            Channels::Stereo
        };

        let application = match config.application {
            AudioApplication::Voip => Application::Voip,
            AudioApplication::Audio => Application::Audio,
            AudioApplication::RestrictedLowdelay => Application::RestrictedLowdelay,
        };

        let encoder = Encoder::new(config.sample_rate, channels, application)
            .context("Failed to create Opus encoder")?;
        encoder.set_bitrate(config.bitrate)?;
        encoder.set_dtx(config.enable_dtx)?;
        encoder.set_fec(config.enable_fec)?;
        encoder.set_complexity(config.complexity)?;

        let decoder =
            Decoder::new(config.sample_rate, channels).context("Failed to create Opus decoder")?;

        Ok(Self {
            config,
            encoder: Mutex::new(Some(encoder)),
            decoder: Mutex::new(Some(decoder)),
            sample_buffer: Mutex::new(Vec::new()),
        })
    }

    /// Encode audio samples to Opus
    pub async fn encode(&self, samples: &[f32]) -> Result<Vec<u8>> {
        let mut encoder = self.encoder.lock().await;
        let encoder = encoder.as_mut().ok_or(MediaError::EncoderNotInitialized)?;

        // Convert f32 to i16
        let mut pcm = Vec::with_capacity(samples.len());
        for &s in samples {
            let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
            pcm.push(v);
        }

        let mut output = vec![0u8; 4000];
        let len = encoder.encode(&pcm, &mut output)?;
        output.truncate(len);
        Ok(output)
    }

    /// Decode Opus to audio samples
    pub async fn decode(&self, data: &[u8]) -> Result<Vec<f32>> {
        let mut decoder = self.decoder.lock().await;
        let decoder = decoder.as_mut().ok_or(MediaError::DecoderNotInitialized)?;

        let frame_size =
            (self.config.sample_rate as usize * self.config.frame_duration_ms as usize) / 1000;
        let mut pcm = vec![0i16; frame_size * self.config.channels as usize];

        let len = decoder.decode(data, &mut pcm, false)?;
        pcm.truncate(len * self.config.channels as usize);

        let samples: Vec<f32> = pcm.iter().map(|&v| v as f32 / 32767.0).collect();
        Ok(samples)
    }

    /// Get frame size in samples
    pub fn frame_size(&self) -> usize {
        (self.config.sample_rate as usize * self.config.frame_duration_ms as usize) / 1000
    }

    /// Get config
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }
}

/// Audio capture from microphone
pub struct AudioCapture {
    config: AudioConfig,
    pipeline: Option<gst::Pipeline>,
    appsink: Option<gst_app::AppSink>,
    sample_tx: mpsc::Sender<Vec<f32>>,
}

impl AudioCapture {
    /// Create new audio capture
    pub async fn new(config: AudioConfig) -> Result<(Self, mpsc::Receiver<Vec<f32>>)> {
        let (sample_tx, sample_rx) = mpsc::channel(100);

        let capture = Self {
            config,
            pipeline: None,
            appsink: None,
            sample_tx,
        };

        Ok((capture, sample_rx))
    }

    /// Start capture
    pub async fn start(&mut self) -> Result<()> {
        gst::init().context("Failed to initialize GStreamer")?;

        let pipeline_str = format!(
            "pulsesrc device=default ! \
             audio/x-raw,format=F32LE,rate={},channels={},layout=interleaved ! \
             appsink name=sink emit-signals=true max-buffers=10 drop=true",
            self.config.sample_rate, self.config.channels
        );

        let pipeline = gst::parse::launch(&pipeline_str)
            .context("Failed to create pipeline")?
            .downcast::<gst::Pipeline>()
            .map_err(|_| MediaError::PipelineCreationFailed)?;

        let appsink = pipeline
            .by_name("sink")
            .ok_or(MediaError::ElementNotFound("appsink".into()))?
            .downcast::<gst_app::AppSink>()
            .map_err(|_| MediaError::ElementCastFailed)?;

        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;

                    // Convert to f32
                    let samples: Vec<f32> = map
                        .as_slice()
                        .chunks_exact(4)
                        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .collect();

                    // Send to channel (non-blocking)
                    // In real impl, use try_send or bounded channel

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        pipeline
            .set_state(gst::State::Playing)
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

/// Audio playback to speakers
pub struct AudioPlayback {
    config: AudioConfig,
    pipeline: Option<gst::Pipeline>,
    appsrc: Option<gst_app::AppSrc>,
}

impl AudioPlayback {
    /// Create new audio playback
    pub async fn new(config: AudioConfig) -> Result<Self> {
        Ok(Self {
            config,
            pipeline: None,
            appsrc: None,
        })
    }

    /// Start playback
    pub async fn start(&mut self) -> Result<()> {
        gst::init().context("Failed to initialize GStreamer")?;

        let pipeline_str = format!(
            "appsrc name=src format=time is-live=true ! \
             audio/x-raw,format=F32LE,rate={},channels={},layout=interleaved ! \
             pulsesink device=default",
            self.config.sample_rate, self.config.channels
        );

        let pipeline = gst::parse::launch(&pipeline_str)
            .context("Failed to create pipeline")?
            .downcast::<gst::Pipeline>()
            .map_err(|_| MediaError::PipelineCreationFailed)?;

        let appsrc = pipeline
            .by_name("src")
            .ok_or(MediaError::ElementNotFound("appsrc".into()))?
            .downcast::<gst_app::AppSrc>()
            .map_err(|_| MediaError::ElementCastFailed)?;

        appsrc.set_caps(
            &gst_audio::AudioCapsBuilder::new()
                .format(gst_audio::AUDIO_FORMAT_F32)
                .rate(self.config.sample_rate as i32)
                .channels(self.config.channels as i32)
                .layout(gst_audio::AudioLayout::Interleaved)
                .build(),
        );

        pipeline
            .set_state(gst::State::Playing)
            .context("Failed to start pipeline")?;

        self.pipeline = Some(pipeline);
        self.appsrc = Some(appsrc);

        Ok(())
    }

    /// Push audio samples
    pub async fn push_samples(&self, samples: &[f32]) -> Result<()> {
        let appsrc = self.appsrc.as_ref().ok_or(MediaError::NotStarted)?;

        let mut buffer = gst::Buffer::with_size(samples.len() * 4)
            .map_err(|_| MediaError::BufferCreationFailed)?;
        {
            let mut map = buffer
                .map_writable()
                .map_err(|_| MediaError::BufferMapFailed)?;
            for &s in samples {
                let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
                map.as_mut_slice()[map.as_mut_slice().len() - 4..]
                    .copy_from_slice(&v.to_le_bytes());
            }
        }

        appsrc
            .push_buffer(buffer)
            .map_err(|_| MediaError::PushBufferFailed)?;
        Ok(())
    }

    /// Stop playback
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null)?;
        }
        self.pipeline = None;
        self.appsrc = None;
        Ok(())
    }
}

/// Audio engine: coordinates capture, encoding, decoding, playback
pub struct AudioEngine {
    config: AudioConfig,
    track: Option<AudioTrack>,
    capture: Option<AudioCapture>,
    playback: Option<AudioPlayback>,
    capture_rx: Option<mpsc::Receiver<Vec<f32>>>,
    running: bool,
}

impl AudioEngine {
    /// Create new audio engine
    pub fn new(config: AudioConfig) -> Self {
        Self {
            config,
            track: None,
            capture: None,
            playback: None,
            capture_rx: None,
            running: false,
        }
    }

    /// Initialize engine
    pub async fn initialize(&mut self) -> Result<()> {
        self.track = Some(AudioTrack::new(self.config.clone())?);
        let (capture, rx) = AudioCapture::new(self.config.clone()).await?;
        self.capture = Some(capture);
        self.capture_rx = Some(rx);
        self.playback = Some(AudioPlayback::new(self.config.clone()).await?);
        Ok(())
    }

    /// Start engine
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }

        self.capture.as_mut().unwrap().start().await?;
        self.playback.as_mut().unwrap().start().await?;
        self.running = true;

        // Start processing loop
        let mut capture_rx = self.capture_rx.take().unwrap();
        let track = self.track.as_ref().unwrap();
        let playback = self.playback.as_ref().unwrap();

        tokio::spawn(async move {
            while let Some(samples) = capture_rx.recv().await {
                // Encode for network
                let encoded = track.encode(&samples).await.ok();
                // In real impl: send encoded to network

                // Playback locally (monitor)
                playback.push_samples(&samples).await.ok();
            }
        });

        Ok(())
    }

    /// Stop engine
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        self.capture.as_mut().unwrap().stop().await?;
        self.playback.as_mut().unwrap().stop().await?;
        self.running = false;
        Ok(())
    }

    /// Process incoming encoded audio
    pub async fn process_incoming(&self, data: &[u8]) -> Result<()> {
        let track = self.track.as_ref().ok_or(MediaError::NotInitialized)?;
        let samples = track.decode(data).await?;
        self.playback.as_ref().unwrap().push_samples(&samples).await
    }

    /// Get track for WebRTC
    pub fn track(&self) -> Option<&AudioTrack> {
        self.track.as_ref()
    }
}
