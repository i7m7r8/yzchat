//! yzchat-ffi: UniFFI bindings for yzchat (Kotlin <-> Rust)

use uniffi::{export, Error};
use yzchat_crypto::hybrid_kem::HybridKem;

/// FFI-facing error surfaced to Kotlin.
#[derive(Debug, uniffi::Error)]
pub enum FfiError {
    #[uniffi::domain = "yzchat_ffi"]
    Crypto(String),
}

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FfiError::Crypto(s) => write!(f, "crypto error: {}", s),
        }
    }
}

impl From<yzchat_crypto::CryptoError> for FfiError {
    fn from(e: yzchat_crypto::CryptoError) -> Self {
        FfiError::Crypto(e.to_string())
    }
}

/// Return the crate version.
#[export]
pub fn yzchat_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Generate a hybrid KEM (ML-KEM-768 + X25519) keypair and return the public key bytes.
#[export]
pub fn kem_generate_keypair() -> Result<Vec<u8>, FfiError> {
    let keypair = HybridKem::generate().map_err(FfiError::from)?;
    Ok(keypair.public_key_bytes())
}
