//! yzchat-p2p: Pure P2P Networking Layer
//!
//! Built on Iroh for:
//! - QUIC-based encrypted transport (Noise_IK)
//! - Kademlia DHT for peer discovery
//! - mDNS for local discovery
//! - N0 discovery for global connectivity
//! - Blobs for file transfer
//! - Docs for syncable data structures
//!
//! Architecture:
//! - Endpoint: Iroh node identity + QUIC listener
//! - Router: Message routing, connection management
//! - Discovery: DHT + mDNS + N0
//! - Signaling: WebRTC ICE candidate exchange via Iroh RPC
//! - Blobs: Large file transfer with streaming

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]

pub mod blobs;
pub mod discovery;
pub mod docs;
pub mod endpoint;
pub mod error;
pub mod router;
pub mod signaling;
pub mod types;

pub use blobs::{BlobConfig, BlobStore, BlobTransfer};
pub use discovery::{DiscoveredPeer, Discovery, DiscoveryConfig};
pub use docs::{DocConfig, DocStore, Document};
pub use endpoint::{EndpointConfig, NodeEndpoint};
pub use error::{P2pError, Result};
pub use router::{IncomingMessage, OutgoingMessage, Router, RouterConfig};
pub use signaling::{IceCandidate, SdpAnswer, SdpOffer, Signaling, SignalingConfig};
pub use types::*;

/// Protocol ALPN for yzchat
pub const ALPN_YZCHAT: &[u8] = b"yzchat/1";

/// Maximum message size for direct RPC
pub const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16 MB

/// Default ports
pub mod ports {
    pub const QUIC_LISTEN: u16 = 0; // OS assigned
    pub const MDNS: u16 = 5353;
}
