//! MediaManager: FFI wrapper for yzchat-media

use crate::callbacks::*;
use crate::crypto_manager::CryptoManager;
use crate::types::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uniffi::Object;
use uuid::Uuid;
use yzchat_media::{
    audio::{AudioConfig, AudioEngine},
    screen_share::{ScreenShareConfig, ScreenShareEngine},
    types::*,
    video::{VideoConfig, VideoEngine},
    webrtc_engine::{PeerConnection, WebRtcConfig, WebRtcEngine},
};

/// Media manager for FFI
#[derive(Object)]
pub struct MediaManager {
    crypto: Arc<CryptoManager>,
    webrtc: Arc<WebRtcEngine>,
    audio_engine: Arc<Mutex<Option<AudioEngine>>>,
    video_engine: Arc<Mutex<Option<VideoEngine>>>,
    screen_engine: Arc<Mutex<Option<ScreenShareEngine>>>,
    active_calls: Arc<Mutex<HashMap<String, CallState>>>,
    call_callback: Arc<Mutex<Option<Box<dyn IncomingCallCallback>>>>,
    stats_callback: Arc<Mutex<Option<Box<dyn MediaStatsCallback>>>>,
}

#[derive(Clone, Debug)]
struct CallState {
    call_id: String,
    session_type: MediaSessionType,
    remote_node: Vec<u8>,
    peer_connection: Arc<PeerConnection>,
    state: CallStateEnum,
    quality: MediaQuality,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CallStateEnum {
    Connecting,
    Ringing,
    Connected,
    Ending,
    Ended,
}

impl MediaManager {
    pub async fn new(crypto: Arc<CryptoManager>) -> Result<Self> {
        let webrtc_config = WebRtcConfig::default();
        let webrtc = Arc::new(WebRtcEngine::new(webrtc_config));

        Ok(Self {
            crypto,
            webrtc,
            audio_engine: Arc::new(Mutex::new(None)),
            video_engine: Arc::new(Mutex::new(None)),
            screen_engine: Arc::new(Mutex::new(None)),
            active_calls: Arc::new(Mutex::new(HashMap::new())),
            call_callback: Arc::new(Mutex::new(None)),
            stats_callback: Arc::new(Mutex::new(None)),
        })
    }

    // Audio calls
    pub async fn start_audio_call(
        &self,
        remote_node: Vec<u8>,
        constraints: MediaConstraints,
    ) -> Result<String> {
        let call_id = Uuid::new_v4().to_string();

        // Initialize audio engine
        let audio_config = AudioConfig::default();
        let mut audio_engine = AudioEngine::new(audio_config);
        audio_engine.initialize().await?;
        audio_engine.start().await?;
        *self.audio_engine.lock().await = Some(audio_engine);

        // Create WebRTC peer connection
        let pc = self
            .webrtc
            .create_peer_connection(Uuid::parse_str(&call_id)?)
            .await?;

        // Add audio track
        let audio_track = audio_engine
            .lock()
            .await
            .as_ref()
            .unwrap()
            .track()
            .unwrap()
            .clone();
        pc.add_audio_track(audio_track).await?;

        // Set up call state
        let call_state = CallState {
            call_id: call_id.clone(),
            session_type: MediaSessionType::VoiceCall,
            remote_node,
            peer_connection: pc,
            state: CallStateEnum::Connecting,
            quality: MediaQuality::default(),
        };
        self.active_calls
            .lock()
            .await
            .insert(call_id.clone(), call_state);

        // Create offer
        let offer = pc.create_offer().await?;

        // Notify callback for signaling
        if let Some(cb) = self.call_callback.lock().await.as_ref() {
            let quality = MediaQuality::default();
            cb(
                call_id.clone(),
                remote_node,
                MediaSessionType::VoiceCall,
                quality,
            );
        }

        Ok(call_id)
    }

    pub async fn stop_audio_call(&self, call_id: String) -> Result<()> {
        if let Some(mut call) = self.active_calls.lock().await.remove(&call_id) {
            call.peer_connection.close().await?;
            call.state = CallStateEnum::Ended;
        }
        if let Some(mut engine) = self.audio_engine.lock().await.take() {
            engine.stop().await?;
        }
        Ok(())
    }

    pub async fn mute_audio(&self, call_id: String, muted: bool) -> Result<()> {
        if let Some(call) = self.active_calls.lock().await.get_mut(&call_id) {
            // Mute audio track
            // TODO: implement track muting
        }
        Ok(())
    }

    pub async fn set_audio_device(&self, call_id: String, device_id: String) -> Result<()> {
        // TODO: implement device switching
        Ok(())
    }

    // Video calls
    pub async fn start_video_call(
        &self,
        remote_node: Vec<u8>,
        constraints: MediaConstraints,
    ) -> Result<String> {
        let call_id = Uuid::new_v4().to_string();

        // Initialize video engine
        let video_config = VideoConfig::default();
        let mut video_engine = VideoEngine::new(video_config);
        video_engine.initialize().await?;
        video_engine.start().await?;
        *self.video_engine.lock().await = Some(video_engine);

        // Create WebRTC peer connection
        let pc = self
            .webrtc
            .create_peer_connection(Uuid::parse_str(&call_id)?)
            .await?;

        // Add video track
        let video_track = video_engine
            .lock()
            .await
            .as_ref()
            .unwrap()
            .track()
            .unwrap()
            .clone();
        pc.add_video_track(video_track).await?;

        // Set up call state
        let call_state = CallState {
            call_id: call_id.clone(),
            session_type: MediaSessionType::VideoCall,
            remote_node,
            peer_connection: pc,
            state: CallStateEnum::Connecting,
            quality: MediaQuality::default(),
        };
        self.active_calls
            .lock()
            .await
            .insert(call_id.clone(), call_state);

        // Notify callback
        if let Some(cb) = self.call_callback.lock().await.as_ref() {
            let quality = MediaQuality::default();
            cb(
                call_id.clone(),
                remote_node,
                MediaSessionType::VideoCall,
                quality,
            );
        }

        Ok(call_id)
    }

    pub async fn stop_video_call(&self, call_id: String) -> Result<()> {
        if let Some(mut call) = self.active_calls.lock().await.remove(&call_id) {
            call.peer_connection.close().await?;
            call.state = CallStateEnum::Ended;
        }
        if let Some(mut engine) = self.video_engine.lock().await.take() {
            engine.stop().await?;
        }
        Ok(())
    }

    pub async fn mute_video(&self, call_id: String, muted: bool) -> Result<()> {
        if let Some(call) = self.active_calls.lock().await.get_mut(&call_id) {
            // Mute video track
        }
        Ok(())
    }

    pub async fn set_video_device(&self, call_id: String, device_id: String) -> Result<()> {
        Ok(())
    }

    pub async fn switch_camera(&self, call_id: String, facing_mode: String) -> Result<()> {
        Ok(())
    }

    // Screen share
    pub async fn start_screen_share(
        &self,
        remote_node: Vec<u8>,
        constraints: MediaConstraints,
    ) -> Result<String> {
        let call_id = Uuid::new_v4().to_string();

        // Initialize screen share engine
        let screen_config = ScreenShareConfig::default();
        let mut screen_engine = ScreenShareEngine::new(screen_config);
        screen_engine.initialize().await?;
        screen_engine.start().await?;
        *self.screen_engine.lock().await = Some(screen_engine);

        // Create WebRTC peer connection
        let pc = self
            .webrtc
            .create_peer_connection(Uuid::parse_str(&call_id)?)
            .await?;

        // Add screen track
        let screen_track = screen_engine
            .lock()
            .await
            .as_ref()
            .unwrap()
            .track()
            .unwrap()
            .clone();
        pc.add_screen_track(screen_track).await?;

        // Set up call state
        let call_state = CallState {
            call_id: call_id.clone(),
            session_type: MediaSessionType::ScreenShare,
            remote_node,
            peer_connection: pc,
            state: CallStateEnum::Connecting,
            quality: MediaQuality::default(),
        };
        self.active_calls
            .lock()
            .await
            .insert(call_id.clone(), call_state);

        Ok(call_id)
    }

    pub async fn stop_screen_share(&self, call_id: String) -> Result<()> {
        if let Some(mut call) = self.active_calls.lock().await.remove(&call_id) {
            call.peer_connection.close().await?;
        }
        if let Some(mut engine) = self.screen_engine.lock().await.take() {
            engine.stop().await?;
        }
        Ok(())
    }

    // Incoming calls
    pub fn on_incoming_call(&self, callback: Box<dyn IncomingCallCallback>) {
        *self.call_callback.lock().await = Some(callback);
    }

    pub async fn answer_call(&self, call_id: String) -> Result<()> {
        if let Some(call) = self.active_calls.lock().await.get_mut(&call_id) {
            call.state = CallStateEnum::Connected;
            // Create answer
            let answer = call.peer_connection.create_answer().await?;
            // Signal answer to remote
        }
        Ok(())
    }

    pub async fn reject_call(&self, call_id: String, reason: String) -> Result<()> {
        if let Some(mut call) = self.active_calls.lock().await.remove(&call_id) {
            call.peer_connection.close().await?;
        }
        Ok(())
    }

    // Devices
    pub async fn enumerate_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        // TODO: implement device enumeration via GStreamer
        Ok(vec![AudioDevice {
            id: "default".to_string(),
            name: "Default".to_string(),
            is_input: true,
            is_output: true,
            is_default: true,
            sample_rates: vec![48000, 44100, 16000],
            channel_counts: vec![1, 2],
        }])
    }

    pub async fn enumerate_video_devices(&self) -> Result<Vec<VideoDevice>> {
        Ok(vec![VideoDevice {
            id: "default".to_string(),
            name: "Default Camera".to_string(),
            formats: vec![
                VideoFormat {
                    width: 1920,
                    height: 1080,
                    framerate: 30,
                    format: "I420".to_string(),
                },
                VideoFormat {
                    width: 3840,
                    height: 2160,
                    framerate: 30,
                    format: "I420".to_string(),
                },
            ],
            is_default: true,
        }])
    }

    // Quality
    pub async fn set_media_quality(&self, call_id: String, quality: MediaQuality) -> Result<()> {
        if let Some(call) = self.active_calls.lock().await.get_mut(&call_id) {
            call.quality = quality;
            // Apply quality settings to encoders
        }
        Ok(())
    }

    pub async fn get_media_stats(&self, call_id: String) -> Result<MediaStats> {
        if let Some(call) = self.active_calls.lock().await.get(&call_id) {
            let stats = call.peer_connection.get_stats().await?;
            // Convert WebRTC stats to MediaStats
            Ok(MediaStats::default())
        } else {
            Err(anyhow::anyhow!("Call not found"))
        }
    }

    pub fn on_media_stats(&self, callback: Box<dyn MediaStatsCallback>) {
        *self.stats_callback.lock().await = Some(callback);
    }

    // Shutdown
    pub async fn shutdown(&self) -> Result<()> {
        // Stop all calls
        let call_ids: Vec<String> = self.active_calls.lock().await.keys().cloned().collect();
        for call_id in call_ids {
            self.stop_audio_call(call_id).await.ok();
            self.stop_video_call(call_id).await.ok();
            self.stop_screen_share(call_id).await.ok();
        }
        Ok(())
    }

    pub fn set_call_callback(&self, callback: Box<dyn IncomingCallCallback>) {
        *self.call_callback.lock().await = Some(callback);
    }
}

// Default implementation for MediaQuality
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

// Default implementation for MediaStats
impl Default for MediaStats {
    fn default() -> Self {
        Self {
            audio: AudioStats::default(),
            video: VideoStats::default(),
            screen: VideoStats::default(),
            connection: ConnectionStats::default(),
        }
    }
}
