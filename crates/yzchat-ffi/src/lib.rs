//! yzchat-ffi: UniFFI bindings for yzchat (Kotlin <-> Rust)

use uniffi::export;
use yzchat_crypto::hybrid_kem::HybridKem;

uniffi::setup_scaffolding!();

/// Return the crate version.
#[export]
pub fn yzchat_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Generate a hybrid KEM (ML-KEM-768 + X25519) keypair and return the public key bytes.
#[export]
pub fn kem_generate_keypair() -> Vec<u8> {
    HybridKem::generate()
        .map(|keypair| keypair.public_key_bytes())
        .unwrap_or_default()
}
