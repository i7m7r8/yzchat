//! Common types for yzchat-crypto

use crate::sizes;
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Identity public key (hybrid sign verify key)
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Zeroize, ZeroizeOnDrop)]
pub struct IdentityPublicKey(pub Vec<u8>);

impl fmt::Debug for IdentityPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IdentityPublicKey")
            .field(&format_args!("{} bytes", self.0.len()))
            .finish()
    }
}

impl IdentityPublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        Ok(Self(hex::decode(hex)?))
    }
}

/// Identity secret key (hybrid sign keypair)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct IdentitySecretKey(pub Vec<u8>);

impl fmt::Debug for IdentitySecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IdentitySecretKey")
            .field(&format_args!("{} bytes", self.0.len()))
            .finish()
    }
}

/// Encryption public key (hybrid KEM public key)
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Zeroize, ZeroizeOnDrop)]
pub struct EncryptionPublicKey(pub Vec<u8>);

impl fmt::Debug for EncryptionPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EncryptionPublicKey")
            .field(&format_args!("{} bytes", self.0.len()))
            .finish()
    }
}

/// Encryption secret key (hybrid KEM secret key)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct EncryptionSecretKey(pub Vec<u8>);

impl fmt::Debug for EncryptionSecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EncryptionSecretKey")
            .field(&format_args!("{} bytes", self.0.len()))
            .finish()
    }
}

/// Pre-key for asynchronous key exchange (signed encryption public key)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PreKey {
    pub key_id: u32,
    pub public_key: EncryptionPublicKey,
    pub signature: crate::hybrid_sign::HybridSignature,
    pub timestamp: i64,
}

/// Signed pre-key (longer-lived, signed by identity)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SignedPreKey {
    pub key_id: u32,
    pub public_key: EncryptionPublicKey,
    pub signature: crate::hybrid_sign::HybridSignature,
    pub timestamp: i64,
}

/// One-time pre-key (for forward secrecy)
#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub struct OneTimePreKey {
    pub key_id: u32,
    pub public_key: EncryptionPublicKey,
    pub secret_key: EncryptionSecretKey,
}

/// Key bundle for session initiation
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct KeyBundle {
    pub identity_key: IdentityPublicKey,
    pub signed_pre_key: SignedPreKey,
    pub one_time_pre_keys: Vec<PreKey>,
    pub timestamp: i64,
}

/// Message types
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum MessageType {
    /// Initial key exchange message
    KeyExchange,
    /// Regular encrypted message
    Message,
    /// Group message
    GroupMessage,
    /// Call signaling
    CallSignaling,
    /// File transfer
    FileTransfer,
    /// Receipt acknowledgment
    Receipt,
    /// Typing indicator
    Typing,
    /// Presence update
    Presence,
}

/// Encrypted message envelope
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EncryptedMessage {
    pub message_type: MessageType,
    pub sender_identity: IdentityPublicKey,
    pub recipient_identity: IdentityPublicKey,
    pub timestamp: i64,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub ratchet_header: Option<crate::ratchet::RatchetHeader>,
}

/// File encryption metadata
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FileEncryptionInfo {
    pub file_id: uuid::Uuid,
    pub file_name: String,
    pub mime_type: String,
    pub size: u64,
    pub encryption_key: Vec<u8>, // Encrypted with recipient's key
    pub chunk_size: u32,
    pub total_chunks: u32,
    pub sha256_hash: Vec<u8>,
}

/// Group encryption state
#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub struct GroupEncryptionState {
    pub group_id: uuid::Uuid,
    pub epoch: u64,
    pub sender_key: [u8; sizes::CHACHA20POLY1305_KEY],
    pub sender_chain: u32,
    pub member_keys: Vec<GroupMemberKey>,
}

/// Group member key
#[derive(Clone, Serialize, Deserialize, Debug, Zeroize, ZeroizeOnDrop)]
pub struct GroupMemberKey {
    pub identity: IdentityPublicKey,
    pub encryption_key: EncryptionPublicKey,
    pub signature: crate::hybrid_sign::HybridSignature,
}
