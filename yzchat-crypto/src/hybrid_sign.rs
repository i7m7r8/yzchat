//! Hybrid Signatures: ML-DSA-65 + Ed25519
//!
//! Both signatures must verify for acceptance. Provides defense-in-depth:
//! if either classical or PQ scheme breaks, the other still holds.

use crate::{
    domain, sizes, random_bytes, constant_time_eq,
    CryptoError, Result,
};
use anyhow::Context;
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{PublicKey as PqPublicKey, SecretKey as PqSecretKey, Signature as PqSignature, DetachedSignature as PqDetachedSignature};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature as Ed25519Signature, Signer, Verifier};
use blake3;
use zeroize::{Zeroize, ZeroizeOnDrop};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Hybrid signing keypair (ML-DSA-65 || Ed25519)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct HybridSignKeypair {
    dilithium_pk: dilithium3::PublicKey,
    dilithium_sk: dilithium3::SecretKey,
    ed25519_sk: SigningKey,
}

impl fmt::Debug for HybridSignKeypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridSignKeypair")
            .field("dilithium_pk", &format_args!("{} bytes", self.dilithium_pk.as_bytes().len()))
            .field("ed25519_pk", &self.ed25519_sk.verifying_key().to_bytes())
            .finish_non_exhaustive()
    }
}

/// Hybrid signature (ML-DSA-65 || Ed25519)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct HybridSignature {
    dilithium_sig: dilithium3::DetachedSignature,
    ed25519_sig: Ed25519Signature,
}

impl fmt::Debug for HybridSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridSignature")
            .field("dilithium_sig", &format_args!("{} bytes", self.dilithium_sig.as_bytes().len()))
            .field("ed25519_sig", &self.ed25519_sig.to_bytes())
            .finish_non_exhaustive()
    }
}

/// Hybrid verification key (public keys)
#[derive(Clone, Serialize, Deserialize)]
pub struct HybridVerifyKey {
    dilithium_pk: dilithium3::PublicKey,
    ed25519_pk: VerifyingKey,
}

impl fmt::Debug for HybridVerifyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridVerifyKey")
            .field("dilithium_pk", &format_args!("{} bytes", self.dilithium_pk.as_bytes().len()))
            .field("ed25519_pk", &self.ed25519_pk.to_bytes())
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
            dilithium_pk,
            dilithium_sk,
            ed25519_sk,
        })
    }

    /// Sign a message with both schemes
    pub fn sign(sk: &HybridSignKeypair, message: &[u8], domain: &[u8]) -> Result<HybridSignature> {
        // Domain-separated hash for each scheme
        let dilithium_msg = Self::domain_separated_hash(message, domain, b"dilithium3");
        let ed25519_msg = Self::domain_separated_hash(message, domain, b"ed25519");

        // ML-DSA-65 signature
        let dilithium_sig = dilithium3::detached_sign(&dilithium_msg, &sk.dilithium_sk);

        // Ed25519 signature
        let ed25519_sig = sk.ed25519_sk.sign(&ed25519_msg);

        Ok(HybridSignature {
            dilithium_sig,
            ed25519_sig,
        })
    }

    /// Verify a hybrid signature (both must verify)
    pub fn verify(vk: &HybridVerifyKey, message: &[u8], sig: &HybridSignature, domain: &[u8]) -> Result<()> {
        let dilithium_msg = Self::domain_separated_hash(message, domain, b"dilithium3");
        let ed25519_msg = Self::domain_separated_hash(message, domain, b"ed25519");

        // Verify ML-DSA-65
        dilithium3::verify_detached_signature(&sig.dilithium_sig, &dilithium_msg, &vk.dilithium_pk)
            .map_err(|_| CryptoError::SignatureVerificationFailed("ML-DSA-65".into()))?;

        // Verify Ed25519
        vk.ed25519_pk.verify(&ed25519_msg, &sig.ed25519_sig)
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
        bytes.extend_from_slice(vk.dilithium_pk.as_bytes());
        bytes.extend_from_slice(vk.ed25519_pk.as_bytes());
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
        let dilithium_pk = dilithium3::PublicKey::from_bytes(&bytes[..sizes::DILITHIUM3_PUBLIC])
            .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let ed25519_pk = VerifyingKey::from_bytes(bytes[sizes::DILITHIUM3_PUBLIC..].try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat)?)?;
        Ok(HybridVerifyKey { dilithium_pk, ed25519_pk })
    }

    /// Serialize signature to bytes
    pub fn signature_to_bytes(sig: &HybridSignature) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(sizes::HYBRID_SIGNATURE);
        bytes.extend_from_slice(sig.dilithium_sig.as_bytes());
        bytes.extend_from_slice(sig.ed25519_sig.as_bytes());
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
        let dilithium_sig = dilithium3::DetachedSignature::from_bytes(&bytes[..sizes::DILITHIUM3_SIGNATURE])
            .map_err(|_| CryptoError::InvalidSignatureFormat)?;
        let ed25519_sig = Ed25519Signature::from_bytes(bytes[sizes::DILITHIUM3_SIGNATURE..].try_into()
            .map_err(|_| CryptoError::InvalidSignatureFormat)?)?;
        Ok(HybridSignature { dilithium_sig, ed25519_sig })
    }
}

impl HybridSignKeypair {
    /// Get verification key
    pub fn verify_key(&self) -> HybridVerifyKey {
        HybridVerifyKey {
            dilithium_pk: self.dilithium_pk.clone(),
            ed25519_pk: self.ed25519_sk.verifying_key(),
        }
    }

    /// Export verification key as bytes
    pub fn verify_key_bytes(&self) -> Vec<u8> {
        HybridSign::verify_key_to_bytes(&self.verify_key())
    }

    /// Export secret key as bytes (sensitive!)
    pub fn secret_key_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(sizes::HYBRID_SIGN_PRIVATE);
        bytes.extend_from_slice(self.dilithium_sk.as_bytes());
        bytes.extend_from_slice(self.ed25519_sk.to_bytes().as_slice());
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
        let dilithium_sk = dilithium3::SecretKey::from_bytes(&sk_bytes[..sizes::DILITHIUM3_PRIVATE])
            .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let ed25519_sk = SigningKey::from_bytes(sk_bytes[sizes::DILITHIUM3_PRIVATE..].try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat)?);

        // Verify they match by deriving PKs
        let dilithium_pk = dilithium3::PublicKey::from_secret_key(&dilithium_sk);
        let ed25519_pk = ed25519_sk.verifying_key();

        // Quick self-test
        let test_msg = b"yzchat-self-test";
        let sig = HybridSign::sign(&Self {
            dilithium_pk: dilithium_pk.clone(),
            dilithium_sk: dilithium_sk.clone(),
            ed25519_sk: ed25519_sk.clone(),
        }, test_msg, domain::IDENTITY_BINDING)?;
        let vk = HybridVerifyKey { dilithium_pk, ed25519_pk };
        HybridSign::verify(&vk, test_msg, &sig, domain::IDENTITY_BINDING)?;

        Ok(Self {
            dilithium_pk,
            dilithium_sk,
            ed25519_sk,
        })
    }
}