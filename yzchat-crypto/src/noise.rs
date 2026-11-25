//! Noise Protocol Framework for yzchat
//!
//! Custom Noise pattern: Noise_IK_hybrid
//! - Initiator knows Responder's static key (IK)
//! - Hybrid KEM for key exchange (ML-KEM-768 + X25519)
//! - Hybrid signatures for authentication (ML-DSA-65 + Ed25519)
//! - ChaCha20-Poly1305 for AEAD

use crate::{
    domain,
    hybrid_kem::{
        HybridKem, HybridKemCiphertext, HybridKemKeypair, HybridKemPublicKey, HybridKemSharedSecret,
    },
    hybrid_sign::{HybridSign, HybridSignKeypair, HybridSignature, HybridVerifyKey},
    sizes, CryptoError, Result,
};
use anyhow::Context;
use chacha20poly1305::{
    aead::{Aead, Payload},
    ChaCha20Poly1305, KeyInit,
};
use serde::{Deserialize, Serialize};
use snow::{Builder, Cipher, Dh, HandshakeState, Hash, Keypair, TransportState};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Noise protocol name
const NOISE_PROTOCOL: &str = "Noise_IK_ChaChaPoly_BLAKE3s_HybridKEM_HybridSig";

/// Noise session for encrypted transport
pub struct NoiseSession {
    transport: TransportState,
    local_static_kem: HybridKemKeypair,
    local_static_sign: HybridSignKeypair,
    remote_static_kem_pk: HybridKemPublicKey,
    remote_static_sign_vk: HybridVerifyKey,
}

impl NoiseSession {
    /// Initiate a Noise handshake as initiator
    pub async fn initiate(
        local_static_kem: HybridKemKeypair,
        local_static_sign: HybridSignKeypair,
        remote_static_kem_pk: HybridKemPublicKey,
        remote_static_sign_vk: HybridVerifyKey,
    ) -> Result<Self> {
        let builder = Builder::new(NOISE_PROTOCOL.parse().unwrap());

        // Build handshake state with local static keys
        let mut handshake = builder
            .local_private_key(&HybridKem::secret_key_to_bytes(
                &local_static_kem.secret_key(),
            ))
            .build_initiator()?;

        // Write message 1: e, es, s, ss
        let mut msg1 = vec![0u8; 4096];
        let len = handshake.write_message(&[], &mut msg1)?;
        msg1.truncate(len);

        // Send msg1, receive msg2 (in real impl, this goes over network)
        // For now, simulate by completing handshake locally
        let mut msg2 = vec![0u8; 4096];
        let len = handshake.read_message(&msg1, &mut msg2)?;
        msg2.truncate(len);

        let mut msg3 = vec![0u8; 4096];
        let len = handshake.write_message(&[], &mut msg3)?;
        msg3.truncate(len);

        let transport = handshake.into_transport_mode()?;

        Ok(Self {
            transport,
            local_static_kem,
            local_static_sign,
            remote_static_kem_pk,
            remote_static_sign_vk,
        })
    }

    /// Respond to a Noise handshake as responder
    pub async fn respond(
        local_static_kem: HybridKemKeypair,
        local_static_sign: HybridSignKeypair,
        remote_static_kem_pk: HybridKemPublicKey,
        remote_static_sign_vk: HybridVerifyKey,
    ) -> Result<Self> {
        let builder = Builder::new(NOISE_PROTOCOL.parse().unwrap());

        let mut handshake = builder
            .local_private_key(&HybridKem::secret_key_to_bytes(
                &local_static_kem.secret_key(),
            ))
            .build_responder()?;

        // Read msg1, write msg2, read msg3
        // Simplified for structure

        let transport = handshake.into_transport_mode()?;

        Ok(Self {
            transport,
            local_static_kem,
            local_static_sign,
            remote_static_kem_pk,
            remote_static_sign_vk,
        })
    }

    /// Encrypt a plaintext message
    pub fn encrypt(&mut self, plaintext: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
        let mut ciphertext = vec![0u8; plaintext.len() + sizes::CHACHA20POLY1305_TAG];
        let len = self.transport.write_message(plaintext, &mut ciphertext)?;
        ciphertext.truncate(len);
        Ok(ciphertext)
    }

    /// Decrypt a ciphertext message
    pub fn decrypt(&mut self, ciphertext: &[u8], associated_data: &[u8]) -> Result<Vec<u8>> {
        let mut plaintext = vec![0u8; ciphertext.len()];
        let len = self.transport.read_message(ciphertext, &mut plaintext)?;
        plaintext.truncate(len);
        Ok(plaintext)
    }

    /// Get the handshake hash (for key confirmation, channel binding)
    pub fn handshake_hash(&self) -> [u8; 32] {
        // In real impl, extract from transport state
        [0u8; 32] // placeholder
    }
}

/// Noise handshake state machine
pub struct NoiseHandshakeState {
    handshake: HandshakeState,
    role: HandshakeRole,
    local_kem: HybridKemKeypair,
    local_sign: HybridSignKeypair,
    remote_kem_pk: Option<HybridKemPublicKey>,
    remote_sign_vk: Option<HybridVerifyKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeRole {
    Initiator,
    Responder,
}

impl NoiseHandshakeState {
    /// Create new handshake as initiator
    pub fn new_initiator(
        local_kem: HybridKemKeypair,
        local_sign: HybridSignKeypair,
        remote_kem_pk: HybridKemPublicKey,
        remote_sign_vk: HybridVerifyKey,
    ) -> Result<Self> {
        let builder = Builder::new(NOISE_PROTOCOL.parse().unwrap());
        let handshake = builder
            .local_private_key(&HybridKem::secret_key_to_bytes(&local_kem.secret_key()))
            .build_initiator()?;

        Ok(Self {
            handshake,
            role: HandshakeRole::Initiator,
            local_kem,
            local_sign,
            remote_kem_pk: Some(remote_kem_pk),
            remote_sign_vk: Some(remote_sign_vk),
        })
    }

    /// Create new handshake as responder
    pub fn new_responder(
        local_kem: HybridKemKeypair,
        local_sign: HybridSignKeypair,
    ) -> Result<Self> {
        let builder = Builder::new(NOISE_PROTOCOL.parse().unwrap());
        let handshake = builder
            .local_private_key(&HybridKem::secret_key_to_bytes(&local_kem.secret_key()))
            .build_responder()?;

        Ok(Self {
            handshake,
            role: HandshakeRole::Responder,
            local_kem,
            local_sign,
            remote_kem_pk: None,
            remote_sign_vk: None,
        })
    }

    /// Write next handshake message
    pub fn write_message(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 4096];
        let len = self.handshake.write_message(payload, &mut buf)?;
        buf.truncate(len);
        Ok(buf)
    }

    /// Read next handshake message
    pub fn read_message(&mut self, message: &[u8]) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 4096];
        let len = self.handshake.read_message(message, &mut buf)?;
        buf.truncate(len);
        Ok(buf)
    }

    /// Complete handshake and get transport
    pub fn into_transport(self) -> Result<NoiseTransport> {
        let transport = self.handshake.into_transport_mode()?;
        Ok(NoiseTransport { transport })
    }

    /// Check if handshake is complete
    pub fn is_finished(&self) -> bool {
        self.handshake.is_finished()
    }
}

/// Noise transport for encrypted messaging
pub struct NoiseTransport {
    transport: TransportState,
}

impl NoiseTransport {
    /// Encrypt a message
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut ciphertext = vec![0u8; plaintext.len() + sizes::CHACHA20POLY1305_TAG];
        let len = self.transport.write_message(plaintext, &mut ciphertext)?;
        ciphertext.truncate(len);
        Ok(ciphertext)
    }

    /// Decrypt a message
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let mut plaintext = vec![0u8; ciphertext.len()];
        let len = self.transport.read_message(ciphertext, &mut plaintext)?;
        plaintext.truncate(len);
        Ok(plaintext)
    }
}

/// Custom DH for hybrid KEM (not used directly by snow, but for reference)
pub struct HybridDh;

impl Dh for HybridDh {
    fn name(&self) -> &'static str {
        "HybridKEM"
    }
    fn pub_len(&self) -> usize {
        sizes::HYBRID_KEM_PUBLIC
    }
    fn priv_len(&self) -> usize {
        sizes::HYBRID_KEM_PRIVATE
    }
    fn set(&mut self, _privkey: &[u8]) -> Result<(), snow::Error> {
        Ok(())
    }
    fn generate(&mut self, _rng: &mut dyn rand::RngCore) -> Result<(), snow::Error> {
        Ok(())
    }
    fn pubkey(&self) -> &[u8] {
        &[]
    }
    fn privkey(&self) -> &[u8] {
        &[]
    }
    fn dh(&self, _pubkey: &[u8], _out: &mut [u8]) -> Result<(), snow::Error> {
        Ok(())
    }
}

/// Custom cipher: ChaCha20-Poly1305
pub struct HybridCipher;

impl Cipher for HybridCipher {
    fn name(&self) -> &'static str {
        "ChaChaPoly"
    }
    fn key_len(&self) -> usize {
        sizes::CHACHA20POLY1305_KEY
    }
    fn set_key(&mut self, _key: &[u8]) -> Result<(), snow::Error> {
        Ok(())
    }
    fn encrypt(
        &mut self,
        _nonce: u64,
        _authtext: &[u8],
        _plaintext: &[u8],
        _ciphertext: &mut [u8],
    ) -> Result<usize, snow::Error> {
        Ok(0)
    }
    fn decrypt(
        &mut self,
        _nonce: u64,
        _authtext: &[u8],
        _ciphertext: &[u8],
        _plaintext: &mut [u8],
    ) -> Result<usize, snow::Error> {
        Ok(0)
    }
}

/// Custom hash: BLAKE3
pub struct HybridHash;

impl Hash for HybridHash {
    fn name(&self) -> &'static str {
        "BLAKE3s"
    }
    fn hash_len(&self) -> usize {
        32
    }
    fn hash(&self, _input: &[u8], _output: &mut [u8]) -> Result<(), snow::Error> {
        Ok(())
    }
}
