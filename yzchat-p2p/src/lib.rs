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

pub mod endpoint;
pub mod discovery;
pub mod router;
pub mod signaling;
pub mod blobs;
pub mod docs;
pub mod error;
pub mod types;

pub use error::{P2pError, Result};
pub use endpoint::{NodeEndpoint, EndpointConfig};
pub use discovery::{Discovery, DiscoveryConfig, DiscoveredPeer};
pub use router::{Router, RouterConfig, IncomingMessage, OutgoingMessage};
pub use signaling::{Signaling, SignalingConfig, IceCandidate, SdpOffer, SdpAnswer};
pub use blobs::{BlobStore, BlobConfig, BlobTransfer};
pub use docs::{DocStore, DocConfig, Document};
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