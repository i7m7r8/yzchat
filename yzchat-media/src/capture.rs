//! Media capture abstractions

use crate::{MediaError, Result};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Audio capture trait
#[async_trait]
pub trait AudioCapture: Send + Sync {
    type Sample;

    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn sample_receiver(&self) -> Option<mpsc::Receiver<Self::Sample>>;
    fn is_running(&self) -> bool;
}

/// Video capture trait
#[async_trait]
pub trait VideoCapture: Send + Sync {
    type Frame;

    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn frame_receiver(&self) -> Option<mpsc::Receiver<Self::Frame>>;
    fn is_running(&self) -> bool;
}

/// Screen capture trait
#[async_trait]
pub trait ScreenCapture: Send + Sync {
    type Frame;

    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn frame_receiver(&self) -> Option<mpsc::Receiver<Self::Frame>>;
    fn is_running(&self) -> bool;
}