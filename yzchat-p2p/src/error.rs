//! Error types for yzchat-p2p

use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2pError {
    #[error("Iroh error: {0}")]
    Iroh(#[from] iroh::Error),

    #[error("Iroh net error: {0}")]
    IrohNet(#[from] iroh_net::error::Error),

    #[error("RPC error: {0}")]
    Rpc(#[from] iroh_rpc::Error),

    #[error("Blobs error: {0}")]
    Blobs(#[from] iroh_blobs::Error),

    #[error("Docs error: {0}")]
    Docs(#[from] iroh_docs::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Peer not connected: {0}")]
    PeerNotConnected(NodeId),

    #[error("Max connections reached")]
    MaxConnectionsReached,

    #[error("Max peers reached")]
    MaxPeersReached,

    #[error("No suitable connection for protocol")]
    NoSuitableConnection,

    #[error("Session not found: {0}")]
    SessionNotFound(uuid::Uuid),

    #[error("No local SDP available")]
    NoLocalSdp,

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Timeout")]
    Timeout,

    #[error("Invalid address")]
    InvalidAddress,

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Signaling error: {0}")]
    Signaling(String),

    #[error("Blob error: {0}")]
    Blob(String),

    #[error("Document error: {0}")]
    Document(String),

    #[error("Other P2P error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, P2pError>;