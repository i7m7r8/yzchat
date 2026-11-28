//! yzchat-crypto: Hybrid Post-Quantum + Classical Cryptography
//!
//! Implements NIST-standardized hybrid key exchange (ML-KEM-768 + X25519)
//! and hybrid signatures (ML-DSA-65 + Ed25519) for quantum-resistant E2EE.

#![forbid(unsafe_code)]
#![allow(missing_docs)]

pub mod error;
pub mod hybrid_kem;
pub mod hybrid_sign;
pub mod noise;
pub mod ratchet;
pub mod types;

pub use error::{CryptoError, Result};
pub use hybrid_kem::{
    HybridKem, HybridKemCiphertext, HybridKemKeypair, HybridKemPublicKey,
    HybridKemSecretKey, HybridKemSharedSecret,
};
pub use hybrid_sign::{HybridSign, HybridSignKeypair, HybridSignature, HybridVerifyKey};
pub use noise::{NoiseHandshakeState, NoiseSession, NoiseTransport};
pub use ratchet::{DoubleRatchet, RatchetState};
pub use types::*;

/// Protocol version for crypto negotiation
pub const PROTOCOL_VERSION: u8 = 1;

/// Domain separation tags
pub mod domain {
    pub const KEY_EXCHANGE: &[u8] = b"yzchat/key-exchange/v1";
    pub const MESSAGE_ENCRYPTION: &[u8] = b"yzchat/message-encryption/v1";
    pub const FILE_ENCRYPTION: &[u8] = b"yzchat/file-encryption/v1";
    pub const CALL_SIGNALING: &[u8] = b"yzchat/call-signaling/v1";
    pub const IDENTITY_BINDING: &[u8] = b"yzchat/identity-binding/v1";
    pub const GROUP_HANDSHAKE: &[u8] = b"yzchat/group-handshake/v1";
}

/// Key sizes
pub mod sizes {
    pub const X25519_PUBLIC: usize = 32;
    pub const X25519_PRIVATE: usize = 32;
    pub const X25519_SHARED: usize = 32;
    pub const ED25519_PUBLIC: usize = 32;
    pub const ED25519_PRIVATE: usize = 64;
    pub const ED25519_SIGNATURE: usize = 64;
    pub const KYBER768_PUBLIC: usize = 1184;
    pub const KYBER768_PRIVATE: usize = 2400;
    pub const KYBER768_CIPHERTEXT: usize = 1088;
    pub const KYBER768_SHARED: usize = 32;
    pub const DILITHIUM3_PUBLIC: usize = 1952;
    pub const DILITHIUM3_PRIVATE: usize = 4000;
    pub const DILITHIUM3_SIGNATURE: usize = 3309;
    pub const HYBRID_KEM_PUBLIC: usize = KYBER768_PUBLIC + X25519_PUBLIC;
    pub const HYBRID_KEM_PRIVATE: usize = KYBER768_PRIVATE + X25519_PRIVATE;
    pub const HYBRID_KEM_CIPHERTEXT: usize = KYBER768_CIPHERTEXT + X25519_PUBLIC;
    pub const HYBRID_SIGN_PUBLIC: usize = DILITHIUM3_PUBLIC + ED25519_PUBLIC;
    pub const HYBRID_SIGN_PRIVATE: usize = DILITHIUM3_PRIVATE + ED25519_PRIVATE;
    pub const HYBRID_SIGNATURE: usize = DILITHIUM3_SIGNATURE + ED25519_SIGNATURE;
    pub const CHACHA20POLY1305_KEY: usize = 32;
    pub const CHACHA20POLY1305_NONCE: usize = 12;
    pub const CHACHA20POLY1305_TAG: usize = 16;
    pub const AES256GCM_KEY: usize = 32;
    pub const AES256GCM_NONCE: usize = 12;
    pub const AES256GCM_TAG: usize = 16;
}

/// Generate secure random bytes using the OS RNG.
pub fn random_bytes(buf: &mut [u8]) {
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(buf);
}

/// Constant-time equality check.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}
