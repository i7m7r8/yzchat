//! Error types for yzchat-crypto

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },

    #[error("Invalid key format")]
    InvalidKeyFormat,

    #[error("Invalid ciphertext size: expected {expected}, got {actual}")]
    InvalidCiphertextSize { expected: usize, actual: usize },

    #[error("Invalid ciphertext format")]
    InvalidCiphertextFormat,

    #[error("Invalid signature size: expected {expected}, got {actual}")]
    InvalidSignatureSize { expected: usize, actual: usize },

    #[error("Invalid signature format")]
    InvalidSignatureFormat,

    #[error("Decapsulation failed")]
    DecapsulationFailed,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Missing DH ratchet for key derivation")]
    MissingDhRatchet,

    #[error("Ratchet state corrupted")]
    RatchetStateCorrupted,

    #[error("Maximum skip limit exceeded")]
    MaxSkipExceeded,

    #[error("Noise protocol error: {0}")]
    NoiseError(#[from] snow::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other crypto error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, CryptoError>;