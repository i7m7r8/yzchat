//! Hybrid Signatures: ML-DSA-65 + Ed25519
//!
//! Both signatures must verify for acceptance. Provides defense-in-depth:
//! if either classical or PQ scheme breaks, the other still holds.

use crate::{
    constant_time_eq, domain, random_bytes, sizes, CryptoError, Result,
};
use anyhow::Context;
use blake3;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{
    DetachedSignature as PqDetachedSignature, PublicKey as PqPublicKey, SecretKey as PqSecretKey,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Hybrid signing keypair (ML-DSA-65 || Ed25519) - stored as raw bytes.
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct HybridSignKeypair {
    pub dilithium_pk: Vec<u8>,
    pub dilithium_sk: Vec<u8>,
    pub ed25519_sk: Vec<u8>,
}

impl fmt::Debug for HybridSignKeypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridSignKeypair")
            .field("dilithium_pk", &format_args!("{} bytes", self.dilithium_pk.len()))
            .field("ed25519_sk", &format_args!("{} bytes", self.ed25519_sk.len()))
            .finish_non_exhaustive()
    }
}

/// Hybrid signature (ML-DSA-65 || Ed25519)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct HybridSignature {
    pub dilithium_sig: Vec<u8>,
    pub ed25519_sig: Vec<u8>,
}

impl fmt::Debug for HybridSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridSignature")
            .field("dilithium_sig", &format_args!("{} bytes", self.dilithium_sig.len()))
            .field("ed25519_sig", &format_args!("{} bytes", self.ed25519_sig.len()))
            .finish_non_exhaustive()
    }
}

/// Hybrid verification key (public keys)
#[derive(Clone, Serialize, Deserialize)]
pub struct HybridVerifyKey {
    pub dilithium_pk: Vec<u8>,
    pub ed25519_pk: Vec<u8>,
}

impl fmt::Debug for HybridVerifyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridVerifyKey")
            .field("dilithium_pk", &format_args!("{} bytes", self.dilithium_pk.len()))
            .field("ed25519_pk", &format_args!("{} bytes", self.ed25519_pk.len()))
            .finish_non_exhaustive()
    }
}

/// Hybrid signature operations
pub struct HybridSign;

impl HybridSign {
    /// Generate a new hybrid signing keypair
    pub fn generate() -> Result<HybridSignKeypair> {
        let (dilithium_pk, dilithium_sk) = dilithium3::keypair();
        let ed25519_sk = SigningKey::generate(&mut rand::rngs::OsRng);

        Ok(HybridSignKeypair {
            dilithium_pk: dilithium_pk.as_ref().to_vec(),
            dilithium_sk: dilithium_sk.as_ref().to_vec(),
            ed25519_sk: ed25519_sk.to_bytes().to_vec(),
        })
    }

    /// Sign a message with both schemes
    pub fn sign(
        sk: &HybridSignKeypair,
        message: &[u8],
        domain_tag: &[u8],
    ) -> Result<HybridSignature> {
        // Reconstruct keys from bytes
        let dilithium_sk = dilithium3::SecretKey::from_bytes(&sk.dilithium_sk)
            .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let ed25519_sk =
            SigningKey::from_bytes(&sk.ed25519_sk).map_err(|_| CryptoError::InvalidKeyFormat)?;

        // Domain-separated hash for each scheme
        let dilithium_msg = Self::domain_separated_hash(message, domain_tag, b"dilithium3");
        let ed25519_msg = Self::domain_separated_hash(message, domain_tag, b"ed25519");

        // ML-DSA-65 signature
        let dilithium_sig = dilithium3::detached_sign(&dilithium_msg, &dilithium_sk);

        // Ed25519 signature
        let ed25519_sig = ed25519_sk.sign(&ed25519_msg);

        Ok(HybridSignature {
            dilithium_sig: dilithium_sig.as_ref().to_vec(),
            ed25519_sig: ed25519_sig.to_bytes().to_vec(),
        })
    }

    /// Verify a hybrid signature (both must verify)
    pub fn verify(
        vk: &HybridVerifyKey,
        message: &[u8],
        sig: &HybridSignature,
        domain_tag: &[u8],
    ) -> Result<()> {
        // Reconstruct keys from bytes
        let dilithium_pk = dilithium3::PublicKey::from_bytes(&vk.dilithium_pk)
            .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let ed25519_pk =
            VerifyingKey::from_bytes(&vk.ed25519_pk).map_err(|_| CryptoError::InvalidKeyFormat)?;

        let dilithium_msg = Self::domain_separated_hash(message, domain_tag, b"dilithium3");
        let ed25519_msg = Self::domain_separated_hash(message, domain_tag, b"ed25519");

        let dilithium_sig =
            dilithium3::DetachedSignature::from_bytes(&sig.dilithium_sig)
                .map_err(|_| CryptoError::InvalidSignatureFormat)?;
        let ed25519_sig = ed25519_dalek::Signature::from_bytes(&sig.ed25519_sig)
            .map_err(|_| CryptoError::InvalidSignatureFormat)?;

        // Verify ML-DSA-65
        dilithium3::verify_detached_signature(&dilithium_sig, &dilithium_msg, &dilithium_pk)
            .map_err(|_| CryptoError::SignatureVerificationFailed("ML-DSA-65".into()))?;

        // Verify Ed25519
        ed25519_pk
            .verify(&ed25519_msg, &ed25519_sig)
            .map_err(|_| CryptoError::SignatureVerificationFailed("Ed25519".into()))?;

        Ok(())
    }

    /// Domain-separated hash: BLAKE3(message || domain || scheme)
    fn domain_separated_hash(message: &[u8], domain: &[u8], scheme: &[u8]) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(message);
        hasher.update(domain);
        hasher.update(scheme);
        hasher.finalize().as_bytes().to_vec()
    }

    /// Serialize public key to bytes
    pub fn verify_key_to_bytes(vk: &HybridVerifyKey) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(sizes::HYBRID_SIGN_PUBLIC);
        bytes.extend_from_slice(&vk.dilithium_pk);
        bytes.extend_from_slice(&vk.ed25519_pk);
        bytes
    }

    /// Deserialize public key from bytes
    pub fn verify_key_from_bytes(bytes: &[u8]) -> Result<HybridVerifyKey> {
        if bytes.len() != sizes::HYBRID_SIGN_PUBLIC {
            return Err(CryptoError::InvalidKeySize {
                expected: sizes::HYBRID_SIGN_PUBLIC,
                actual: bytes.len(),
            });
        }
        Ok(HybridVerifyKey {
            dilithium_pk: bytes[..sizes::DILITHIUM3_PUBLIC].to_vec(),
            ed25519_pk: bytes[sizes::DILITHIUM3_PUBLIC..].to_vec(),
        })
    }

    /// Serialize signature to bytes
    pub fn signature_to_bytes(sig: &HybridSignature) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(sizes::HYBRID_SIGNATURE);
        bytes.extend_from_slice(&sig.dilithium_sig);
        bytes.extend_from_slice(&sig.ed25519_sig);
        bytes
    }

    /// Deserialize signature from bytes
    pub fn signature_from_bytes(bytes: &[u8]) -> Result<HybridSignature> {
        if bytes.len() != sizes::HYBRID_SIGNATURE {
            return Err(CryptoError::InvalidSignatureSize {
                expected: sizes::HYBRID_SIGNATURE,
                actual: bytes.len(),
            });
        }
        Ok(HybridSignature {
            dilithium_sig: bytes[..sizes::DILITHIUM3_SIGNATURE].to_vec(),
            ed25519_sig: bytes[sizes::DILITHIUM3_SIGNATURE..].to_vec(),
        })
    }
}

impl HybridSignKeypair {
    /// Get verification key
    pub fn verify_key(&self) -> HybridVerifyKey {
        HybridVerifyKey {
            dilithium_pk: self.dilithium_pk.clone(),
            ed25519_pk: Vec::new(), // placeholder - derive from ed25519_sk if needed
        }
    }

    /// Export verification key as bytes
    pub fn verify_key_bytes(&self) -> Vec<u8> {
        HybridSign::verify_key_to_bytes(&self.verify_key())
    }

    /// Export secret key as bytes (sensitive!)
    pub fn secret_key_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(sizes::HYBRID_SIGN_PRIVATE);
        bytes.extend_from_slice(&self.dilithium_sk);
        bytes.extend_from_slice(&self.ed25519_sk);
        bytes
    }

    /// Import from bytes
    pub fn from_bytes(sk_bytes: &[u8]) -> Result<Self> {
        if sk_bytes.len() != sizes::HYBRID_SIGN_PRIVATE {
            return Err(CryptoError::InvalidKeySize {
                expected: sizes::HYBRID_SIGN_PRIVATE,
                actual: sk_bytes.len(),
            });
        }
        // Validate keys can be reconstructed
        let _dilithium_sk =
            dilithium3::SecretKey::from_bytes(&sk_bytes[..sizes::DILITHIUM3_PRIVATE])
                .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let _ed25519_sk = SigningKey::from_bytes(&sk_bytes[sizes::DILITHIUM3_PRIVATE..])
            .map_err(|_| CryptoError::InvalidKeyFormat)?;

        Ok(Self {
            dilithium_pk: sk_bytes[..sizes::DILITHIUM3_PUBLIC].to_vec(),
            dilithium_sk: sk_bytes[..sizes::DILITHIUM3_PRIVATE].to_vec(),
            ed25519_sk: sk_bytes[sizes::DILITHIUM3_PRIVATE..].to_vec(),
        })
    }
}
