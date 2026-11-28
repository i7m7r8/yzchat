//! Noise Protocol Framework for yzchat
//!
//! Standard Noise pattern: `Noise_IK_25519_ChaChaPoly_BLAKE2s` for the
//! transport handshake, layered with the hybrid KEM/signatures at the
//! application level for quantum resistance.

use crate::{sizes, CryptoError, Result};
use anyhow::Context;
use snow::{Builder, HandshakeState, TransportState};
use std::fmt;

/// Noise protocol name (standard, recognized by snow)
const NOISE_PROTOCOL: &str = "Noise_IK_25519_ChaChaPoly_BLAKE2s";

/// Noise session for encrypted transport
pub struct NoiseSession {
    transport: TransportState,
}

impl NoiseSession {
    /// Initiate a Noise handshake as initiator using a 32-byte X25519 static private key.
    pub async fn initiate(local_static_sk: &[u8; 32]) -> Result<Self> {
        let builder = Builder::new(NOISE_PROTOCOL.parse().map_err(CryptoError::NoiseError)?);

        let mut handshake = builder
            .local_private_key(local_static_sk)
            .build_initiator()?;

        // Write message 1
        let mut msg1 = vec![0u8; 4096];
        let len = handshake.write_message(&[], &mut msg1)?;
        msg1.truncate(len);

        // Read message 2 (simulated loopback for structure; real impl uses network)
        let mut msg2 = vec![0u8; 4096];
        let len = handshake.read_message(&msg1, &mut msg2)?;
        msg2.truncate(len);

        // Write message 3
        let mut msg3 = vec![0u8; 4096];
        let len = handshake.write_message(&[], &mut msg3)?;
        msg3.truncate(len);

        let transport = handshake.into_transport_mode()?;

        Ok(Self { transport })
    }

    /// Respond to a Noise handshake as responder using a 32-byte X25519 static private key.
    pub async fn respond(local_static_sk: &[u8; 32]) -> Result<Self> {
        let builder = Builder::new(NOISE_PROTOCOL.parse().map_err(CryptoError::NoiseError)?);

        let mut handshake = builder
            .local_private_key(local_static_sk)
            .build_responder()?;

        // Read message 1, write message 2, read message 3
        let msg1 = vec![0u8; 4096];
        let len = handshake.read_message(&msg1, &mut [])?; // placeholder read
        let _ = len;

        let transport = handshake.into_transport_mode()?;

        Ok(Self { transport })
    }

    /// Encrypt a plaintext message
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut ciphertext = vec![0u8; plaintext.len() + sizes::CHACHA20POLY1305_TAG];
        let len = self
            .transport
            .write_message(plaintext, &mut ciphertext)
            .context("Noise encrypt failed")?;
        ciphertext.truncate(len);
        Ok(ciphertext)
    }

    /// Decrypt a ciphertext message
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let mut plaintext = vec![0u8; ciphertext.len()];
        let len = self
            .transport
            .read_message(ciphertext, &mut plaintext)
            .context("Noise decrypt failed")?;
        plaintext.truncate(len);
        Ok(plaintext)
    }
}

/// Noise handshake state machine
pub struct NoiseHandshakeState {
    handshake: HandshakeState,
    role: HandshakeRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeRole {
    Initiator,
    Responder,
}

impl NoiseHandshakeState {
    /// Create new handshake as initiator
    pub fn new_initiator(local_static_sk: &[u8; 32]) -> Result<Self> {
        let builder = Builder::new(NOISE_PROTOCOL.parse().map_err(CryptoError::NoiseError)?);
        let handshake = builder.local_private_key(local_static_sk).build_initiator()?;

        Ok(Self {
            handshake,
            role: HandshakeRole::Initiator,
        })
    }

    /// Create new handshake as responder
    pub fn new_responder(local_static_sk: &[u8; 32]) -> Result<Self> {
        let builder = Builder::new(NOISE_PROTOCOL.parse().map_err(CryptoError::NoiseError)?);
        let handshake = builder.local_private_key(local_static_sk).build_responder()?;

        Ok(Self {
            handshake,
            role: HandshakeRole::Responder,
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
}

/// Noise transport for encrypted messaging
pub struct NoiseTransport {
    transport: TransportState,
}

impl NoiseTransport {
    /// Encrypt a message
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut ciphertext = vec![0u8; plaintext.len() + sizes::CHACHA20POLY1305_TAG];
        let len = self
            .transport
            .write_message(plaintext, &mut ciphertext)
            .context("Noise encrypt failed")?;
        ciphertext.truncate(len);
        Ok(ciphertext)
    }

    /// Decrypt a message
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let mut plaintext = vec![0u8; ciphertext.len()];
        let len = self
            .transport
            .read_message(ciphertext, &mut plaintext)
            .context("Noise decrypt failed")?;
        plaintext.truncate(len);
        Ok(plaintext)
    }
}

impl fmt::Debug for NoiseSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoiseSession").finish_non_exhaustive()
    }
}

impl fmt::Debug for NoiseHandshakeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoiseHandshakeState")
            .field("role", &self.role)
            .finish_non_exhaustive()
    }
}
