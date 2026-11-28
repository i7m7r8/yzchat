//! Hybrid KEM: ML-KEM-768 + X25519 (NIST SP 800-208 compliant)
//!
//! Combines classical X25519 ECDH with ML-KEM-768 for quantum resistance.
//! The shared secret is HKDF(ML-KEM-SS || X25519-SS, salt, info).

use crate::{domain, sizes, CryptoError, Result};
use hkdf::Hkdf;
use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SecretKey, SharedSecret};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fmt;
use x25519_dalek::{
    EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Hybrid KEM keypair (ML-KEM-768 || X25519) - stored as raw bytes.
#[derive(Clone, Serialize, Deserialize)]
pub struct HybridKemKeypair {
    pub kyber_pk: Vec<u8>,
    pub kyber_sk: Vec<u8>,
    pub x25519_pk: Vec<u8>,
    pub x25519_sk: Vec<u8>,
}

impl fmt::Debug for HybridKemKeypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridKemKeypair")
            .field("kyber_pk", &format_args!("{} bytes", self.kyber_pk.len()))
            .field("x25519_pk", &format_args!("{} bytes", self.x25519_pk.len()))
            .finish_non_exhaustive()
    }
}

/// Hybrid KEM ciphertext (ML-KEM-CT || X25519-ephemeral-PK)
#[derive(Clone, Serialize, Deserialize)]
pub struct HybridKemCiphertext {
    pub kyber_ct: Vec<u8>,
    pub x25519_ephemeral_pk: Vec<u8>,
}

impl fmt::Debug for HybridKemCiphertext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridKemCiphertext")
            .field("kyber_ct", &format_args!("{} bytes", self.kyber_ct.len()))
            .field(
                "x25519_ephemeral_pk",
                &format_args!("{} bytes", self.x25519_ephemeral_pk.len()),
            )
            .finish_non_exhaustive()
    }
}

/// Hybrid KEM shared secret (32 bytes)
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct HybridKemSharedSecret(pub [u8; sizes::CHACHA20POLY1305_KEY]);

impl fmt::Debug for HybridKemSharedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridKemSharedSecret")
            .field("len", &self.0.len())
            .finish_non_exhaustive()
    }
}

impl AsRef<[u8]> for HybridKemSharedSecret {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Hybrid KEM public key component (for encapsulation)
#[derive(Clone, Serialize, Deserialize)]
pub struct HybridKemPublicKey {
    pub kyber_pk: Vec<u8>,
    pub x25519_pk: Vec<u8>,
}

impl fmt::Debug for HybridKemPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridKemPublicKey")
            .field("kyber_pk", &format_args!("{} bytes", self.kyber_pk.len()))
            .field("x25519_pk", &format_args!("{} bytes", self.x25519_pk.len()))
            .finish_non_exhaustive()
    }
}

impl PartialEq for HybridKemPublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.kyber_pk == other.kyber_pk && self.x25519_pk == other.x25519_pk
    }
}

impl Eq for HybridKemPublicKey {}

/// Hybrid KEM secret key component (for decapsulation)
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct HybridKemSecretKey {
    pub kyber_sk: Vec<u8>,
    pub x25519_sk: Vec<u8>,
}

/// Hybrid KEM operations
pub struct HybridKem;

impl HybridKem {
    /// Generate a new hybrid keypair
    pub fn generate() -> Result<HybridKemKeypair> {
        // ML-KEM-768 keypair
        let (kyber_pk, kyber_sk) = kyber768::keypair();

        // X25519 keypair (OS RNG via getrandom)
        let x25519_sk = X25519StaticSecret::random();
        let x25519_pk = X25519PublicKey::from(&x25519_sk);

        Ok(HybridKemKeypair {
            kyber_pk: kyber_pk.as_bytes().to_vec(),
            kyber_sk: kyber_sk.as_bytes().to_vec(),
            x25519_pk: x25519_pk.to_bytes().to_vec(),
            x25519_sk: x25519_sk.to_bytes().to_vec(),
        })
    }

    /// Encapsulate to a recipient's public key, producing ciphertext and shared secret
    pub fn encapsulate(
        pk: &HybridKemPublicKey,
    ) -> Result<(HybridKemCiphertext, HybridKemSharedSecret)> {
        // Reconstruct kyber public key from bytes
        let kyber_pk =
            kyber768::PublicKey::from_bytes(&pk.kyber_pk)
                .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let x25519_arr: [u8; 32] = pk
            .x25519_pk
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let x25519_pk = X25519PublicKey::from(x25519_arr);

        // ML-KEM encapsulation
        let (kyber_ct, kyber_ss) = kyber768::encapsulate(&kyber_pk);

        // X25519 ephemeral key generation + DH
        let x25519_eph_sk = EphemeralSecret::random();
        let x25519_eph_pk = X25519PublicKey::from(&x25519_eph_sk);
        let x25519_ss = x25519_eph_sk.diffie_hellman(&x25519_pk);

        // Combine shared secrets via HKDF
        let combined_ss = HybridKem::combine_shared_secrets(
            kyber_ss.as_bytes(),
            x25519_ss.as_bytes(),
            domain::KEY_EXCHANGE,
        )?;

        Ok((
            HybridKemCiphertext {
                kyber_ct: kyber_ct.as_bytes().to_vec(),
                x25519_ephemeral_pk: x25519_eph_pk.to_bytes().to_vec(),
            },
            combined_ss,
        ))
    }

    /// Decapsulate using private key, recovering shared secret
    pub fn decapsulate(
        sk: &HybridKemSecretKey,
        ct: &HybridKemCiphertext,
    ) -> Result<HybridKemSharedSecret> {
        // Reconstruct keys from bytes
        let kyber_sk =
            kyber768::SecretKey::from_bytes(&sk.kyber_sk)
                .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let kyber_ct =
            kyber768::Ciphertext::from_bytes(&ct.kyber_ct)
                .map_err(|_| CryptoError::InvalidCiphertextFormat)?;
        let x25519_arr: [u8; 32] = sk
            .x25519_sk
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat)?;
        let x25519_sk = X25519StaticSecret::from(x25519_arr);
        let x25519_arr: [u8; 32] = ct
            .x25519_ephemeral_pk
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidCiphertextFormat)?;
        let x25519_ephemeral_pk = X25519PublicKey::from(x25519_arr);

        // ML-KEM decapsulation
        let kyber_ss = kyber768::decapsulate(&kyber_ct, &kyber_sk);

        // X25519 DH
        let x25519_ss = x25519_sk.diffie_hellman(&x25519_ephemeral_pk);

        // Combine
        HybridKem::combine_shared_secrets(
            kyber_ss.as_bytes(),
            x25519_ss.as_bytes(),
            domain::KEY_EXCHANGE,
        )
    }

    /// Combine two shared secrets using HKDF-SHA256
    fn combine_shared_secrets(
        kyber_ss: &[u8],
        x25519_ss: &[u8],
        domain: &[u8],
    ) -> Result<HybridKemSharedSecret> {
        let mut combined = Vec::with_capacity(kyber_ss.len() + x25519_ss.len());
        combined.extend_from_slice(kyber_ss);
        combined.extend_from_slice(x25519_ss);

        let hkdf = Hkdf::<Sha256>::new(Some(domain), &combined);
        let mut okm = [0u8; sizes::CHACHA20POLY1305_KEY];
        hkdf.expand(b"yzchat-hybrid-kem-v1", &mut okm)
            .map_err(|_| CryptoError::Other(anyhow::anyhow!("hkdf expand failed")))?;

        Ok(HybridKemSharedSecret(okm))
    }

    /// Serialize public key to bytes (ML-KEM || X25519)
    pub fn public_key_to_bytes(pk: &HybridKemPublicKey) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(sizes::HYBRID_KEM_PUBLIC);
        bytes.extend_from_slice(&pk.kyber_pk);
        bytes.extend_from_slice(&pk.x25519_pk);
        bytes
    }

    /// Deserialize public key from bytes
    pub fn public_key_from_bytes(bytes: &[u8]) -> Result<HybridKemPublicKey> {
        if bytes.len() != sizes::HYBRID_KEM_PUBLIC {
            return Err(CryptoError::InvalidKeySize {
                expected: sizes::HYBRID_KEM_PUBLIC,
                actual: bytes.len(),
            });
        }
        Ok(HybridKemPublicKey {
            kyber_pk: bytes[..sizes::KYBER768_PUBLIC].to_vec(),
            x25519_pk: bytes[sizes::KYBER768_PUBLIC..].to_vec(),
        })
    }

    /// Serialize secret key to bytes
    pub fn secret_key_to_bytes(sk: &HybridKemSecretKey) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(sizes::HYBRID_KEM_PRIVATE);
        bytes.extend_from_slice(&sk.kyber_sk);
        bytes.extend_from_slice(&sk.x25519_sk);
        bytes
    }

    /// Deserialize secret key from bytes
    pub fn secret_key_from_bytes(bytes: &[u8]) -> Result<HybridKemSecretKey> {
        if bytes.len() != sizes::HYBRID_KEM_PRIVATE {
            return Err(CryptoError::InvalidKeySize {
                expected: sizes::HYBRID_KEM_PRIVATE,
                actual: bytes.len(),
            });
        }
        Ok(HybridKemSecretKey {
            kyber_sk: bytes[..sizes::KYBER768_PRIVATE].to_vec(),
            x25519_sk: bytes[sizes::KYBER768_PRIVATE..].to_vec(),
        })
    }

    /// Serialize ciphertext to bytes
    pub fn ciphertext_to_bytes(ct: &HybridKemCiphertext) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(sizes::HYBRID_KEM_CIPHERTEXT);
        bytes.extend_from_slice(&ct.kyber_ct);
        bytes.extend_from_slice(&ct.x25519_ephemeral_pk);
        bytes
    }

    /// Deserialize ciphertext from bytes
    pub fn ciphertext_from_bytes(bytes: &[u8]) -> Result<HybridKemCiphertext> {
        if bytes.len() != sizes::HYBRID_KEM_CIPHERTEXT {
            return Err(CryptoError::InvalidCiphertextSize {
                expected: sizes::HYBRID_KEM_CIPHERTEXT,
                actual: bytes.len(),
            });
        }
        Ok(HybridKemCiphertext {
            kyber_ct: bytes[..sizes::KYBER768_CIPHERTEXT].to_vec(),
            x25519_ephemeral_pk: bytes[sizes::KYBER768_CIPHERTEXT..].to_vec(),
        })
    }
}

impl HybridKemKeypair {
    /// Get public key for encapsulation
    pub fn public_key(&self) -> HybridKemPublicKey {
        HybridKemPublicKey {
            kyber_pk: self.kyber_pk.clone(),
            x25519_pk: self.x25519_pk.clone(),
        }
    }

    /// Get secret key for decapsulation
    pub fn secret_key(&self) -> HybridKemSecretKey {
        HybridKemSecretKey {
            kyber_sk: self.kyber_sk.clone(),
            x25519_sk: self.x25519_sk.clone(),
        }
    }

    /// Export public key as bytes
    pub fn public_key_bytes(&self) -> Vec<u8> {
        HybridKem::public_key_to_bytes(&self.public_key())
    }

    /// Export secret key as bytes (sensitive!)
    pub fn secret_key_bytes(&self) -> Vec<u8> {
        HybridKem::secret_key_to_bytes(&self.secret_key())
    }

    /// Import from bytes
    pub fn from_bytes(pk_bytes: &[u8], sk_bytes: &[u8]) -> Result<Self> {
        let pk = HybridKem::public_key_from_bytes(pk_bytes)?;
        let sk = HybridKem::secret_key_from_bytes(sk_bytes)?;

        // `sk` implements Drop (ZeroizeOnDrop), so clone its fields rather
        // than moving them out.
        Ok(Self {
            kyber_pk: pk.kyber_pk,
            kyber_sk: sk.kyber_sk.clone(),
            x25519_pk: pk.x25519_pk,
            x25519_sk: sk.x25519_sk.clone(),
        })
    }
}
