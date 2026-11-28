//! Double Ratchet Implementation for yzchat
//!
//! Provides forward secrecy and post-compromise security for messaging.
//! Based on Signal's Double Ratchet algorithm, adapted for hybrid PQ crypto.

use crate::{
    domain, hybrid_kem::{HybridKem, HybridKemKeypair, HybridKemPublicKey, HybridKemSharedSecret},
    sizes, CryptoError, Result,
};
use chacha20poly1305::{
    aead::{Aead, Payload},
    ChaCha20Poly1305, KeyInit,
};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::VecDeque;
use std::fmt;

/// Double Ratchet state
#[derive(Clone, Serialize, Deserialize)]
pub struct RatchetState {
    // Root key chain
    pub root_key: [u8; sizes::CHACHA20POLY1305_KEY],

    // Sending chain
    pub send_chain_key: [u8; sizes::CHACHA20POLY1305_KEY],
    pub send_message_number: u32,
    pub send_chain_length: u32,

    // Receiving chain
    pub recv_chain_key: [u8; sizes::CHACHA20POLY1305_KEY],
    pub recv_message_number: u32,
    pub recv_chain_length: u32,

    // Next DH ratchet (for DH ratchet step)
    pub next_dh_ratchet: Option<HybridKemKeypair>,

    // Skipped message keys for out-of-order delivery
    pub skipped_keys: VecDeque<SkippedMessageKey>,

    // Maximum skipped keys to store
    pub max_skip: usize,
}

/// Skipped message key for out-of-order messages
#[derive(Clone, Serialize, Deserialize)]
pub struct SkippedMessageKey {
    pub dh_public: HybridKemPublicKey,
    pub message_number: u32,
    pub message_key: [u8; sizes::CHACHA20POLY1305_KEY],
}

impl fmt::Debug for RatchetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RatchetState")
            .field("send_msg_num", &self.send_message_number)
            .field("recv_msg_num", &self.recv_message_number)
            .field("skipped_keys", &self.skipped_keys.len())
            .finish_non_exhaustive()
    }
}

/// Double Ratchet operations
pub struct DoubleRatchet;

impl DoubleRatchet {
    /// Initialize a new Double Ratchet session from a shared secret
    pub fn initialize(
        shared_secret: &HybridKemSharedSecret,
        is_initiator: bool,
    ) -> Result<RatchetState> {
        // Derive root key and initial chain keys from shared secret
        let hkdf = Hkdf::<Sha256>::new(Some(domain::MESSAGE_ENCRYPTION), shared_secret.as_ref());
        let mut okm = [0u8; sizes::CHACHA20POLY1305_KEY * 3];
        hkdf.expand(b"yzchat-double-ratchet-init", &mut okm)
            .map_err(|_| CryptoError::Other(anyhow::anyhow!("hkdf expand failed")))?;

        let root_key = okm[..sizes::CHACHA20POLY1305_KEY].try_into().unwrap();
        let (send_chain_key, recv_chain_key) = if is_initiator {
            (
                okm[sizes::CHACHA20POLY1305_KEY..2 * sizes::CHACHA20POLY1305_KEY]
                    .try_into()
                    .unwrap(),
                okm[2 * sizes::CHACHA20POLY1305_KEY..].try_into().unwrap(),
            )
        } else {
            (
                okm[2 * sizes::CHACHA20POLY1305_KEY..].try_into().unwrap(),
                okm[sizes::CHACHA20POLY1305_KEY..2 * sizes::CHACHA20POLY1305_KEY]
                    .try_into()
                    .unwrap(),
            )
        };

        Ok(RatchetState {
            root_key,
            send_chain_key,
            send_message_number: 0,
            send_chain_length: 0,
            recv_chain_key,
            recv_message_number: 0,
            recv_chain_length: 0,
            next_dh_ratchet: None,
            skipped_keys: VecDeque::new(),
            max_skip: 1000,
        })
    }

    /// Perform a DH ratchet step (when receiving a new DH public key)
    pub fn ratchet_dh(state: &mut RatchetState, remote_dh_pk: &HybridKemPublicKey) -> Result<()> {
        // DH shared secret
        let (_dh_ct, dh_shared) = HybridKem::encapsulate(remote_dh_pk)?;

        // KDF root key
        let hkdf = Hkdf::<Sha256>::new(Some(&state.root_key), dh_shared.as_ref());
        let mut okm = [0u8; sizes::CHACHA20POLY1305_KEY * 2];
        hkdf.expand(b"yzchat-dh-ratchet", &mut okm)
            .map_err(|_| CryptoError::Other(anyhow::anyhow!("hkdf expand failed")))?;

        state.root_key = okm[..sizes::CHACHA20POLY1305_KEY].try_into().unwrap();
        state.recv_chain_key = okm[sizes::CHACHA20POLY1305_KEY..].try_into().unwrap();

        // Reset receive chain
        state.recv_message_number = 0;
        state.recv_chain_length = 0;

        // Set new sending chain key
        state.send_chain_key = state.root_key;
        state.send_message_number = 0;
        state.send_chain_length = 0;

        // Store new DH ratchet for next step
        state.next_dh_ratchet = Some(HybridKem::generate()?);

        Ok(())
    }

    /// Encrypt a message (symmetric ratchet step)
    pub fn encrypt(
        state: &mut RatchetState,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Result<(Vec<u8>, u32, HybridKemPublicKey)> {
        // Derive message key from sending chain
        let (message_key, next_chain_key) = Self::kdf_chain(&state.send_chain_key)?;
        state.send_chain_key = next_chain_key;
        let msg_num = state.send_message_number;
        state.send_message_number += 1;
        state.send_chain_length += 1;

        // Encrypt with ChaCha20-Poly1305
        let cipher = ChaCha20Poly1305::new(&message_key.into());
        let nonce = Self::nonce_from_counter(msg_num);
        let ciphertext = cipher
            .encrypt(
                &nonce.into(),
                Payload {
                    msg: plaintext,
                    aad: associated_data,
                },
            )
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Get current DH public key for header
        let dh_pk = state
            .next_dh_ratchet
            .as_ref()
            .ok_or(CryptoError::MissingDhRatchet)?
            .public_key();

        Ok((ciphertext, msg_num, dh_pk))
    }

    /// Decrypt a message (symmetric ratchet step)
    pub fn decrypt(
        state: &mut RatchetState,
        ciphertext: &[u8],
        associated_data: &[u8],
        msg_num: u32,
        dh_pk: &HybridKemPublicKey,
    ) -> Result<Vec<u8>> {
        // Check if we need to do a DH ratchet
        if state.recv_chain_length > 0
            && Some(dh_pk)
                != state
                    .next_dh_ratchet
                    .as_ref()
                    .map(|k| k.public_key())
                    .as_ref()
        {
            // Skip keys up to msg_num
            Self::skip_message_keys(state, msg_num)?;
            // Perform DH ratchet
            Self::ratchet_dh(state, dh_pk)?;
        }

        // Try to find skipped key
        if let Some(idx) = state
            .skipped_keys
            .iter()
            .position(|k| k.message_number == msg_num && k.dh_public == *dh_pk)
        {
            let skipped = state.skipped_keys.remove(idx).unwrap();
            let cipher = ChaCha20Poly1305::new(&skipped.message_key.into());
            let nonce = Self::nonce_from_counter(msg_num);
            return cipher
                .decrypt(
                    &nonce.into(),
                    Payload {
                        msg: ciphertext,
                        aad: associated_data,
                    },
                )
                .map_err(|_| CryptoError::DecryptionFailed);
        }

        // Derive message key from receiving chain
        if msg_num >= state.recv_message_number {
            Self::skip_message_keys(state, msg_num + 1)?;
        }

        let (message_key, next_chain_key) = Self::kdf_chain(&state.recv_chain_key)?;
        state.recv_chain_key = next_chain_key;
        state.recv_message_number = msg_num + 1;
        state.recv_chain_length += 1;

        let cipher = ChaCha20Poly1305::new(&message_key.into());
        let nonce = Self::nonce_from_counter(msg_num);
        cipher
            .decrypt(
                &nonce.into(),
                Payload {
                    msg: ciphertext,
                    aad: associated_data,
                },
            )
            .map_err(|_| CryptoError::DecryptionFailed)
    }

    /// KDF for chain key: CK' = HMAC-SHA256(CK, 0x01), MK = HMAC-SHA256(CK, 0x02)
    fn kdf_chain(
        chain_key: &[u8; sizes::CHACHA20POLY1305_KEY],
    ) -> Result<(
        [u8; sizes::CHACHA20POLY1305_KEY],
        [u8; sizes::CHACHA20POLY1305_KEY],
    )> {
        let hkdf = Hkdf::<Sha256>::new(None, chain_key);
        let mut okm = [0u8; sizes::CHACHA20POLY1305_KEY * 2];
        hkdf.expand(b"yzchat-chain-kdf", &mut okm)
            .map_err(|_| CryptoError::Other(anyhow::anyhow!("hkdf expand failed")))?;

        let message_key = okm[..sizes::CHACHA20POLY1305_KEY].try_into().unwrap();
        let next_chain_key = okm[sizes::CHACHA20POLY1305_KEY..].try_into().unwrap();

        Ok((message_key, next_chain_key))
    }

    /// Skip message keys up to a given message number
    fn skip_message_keys(state: &mut RatchetState, until: u32) -> Result<()> {
        while state.recv_message_number < until && state.skipped_keys.len() < state.max_skip {
            let (message_key, next_chain_key) = Self::kdf_chain(&state.recv_chain_key)?;
            state.recv_chain_key = next_chain_key;

            let dh_pk = state
                .next_dh_ratchet
                .as_ref()
                .ok_or(CryptoError::MissingDhRatchet)?
                .public_key();

            state.skipped_keys.push_back(SkippedMessageKey {
                dh_public: dh_pk,
                message_number: state.recv_message_number,
                message_key,
            });
            state.recv_message_number += 1;
        }
        Ok(())
    }

    /// Generate nonce from message counter (96-bit nonce for ChaCha20-Poly1305)
    fn nonce_from_counter(counter: u32) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        nonce[8..].copy_from_slice(&counter.to_be_bytes());
        nonce
    }
}

/// Message header for Double Ratchet
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RatchetHeader {
    pub dh_public: HybridKemPublicKey,
    pub message_number: u32,
    pub previous_chain_length: u32,
}

/// Encrypted message with header
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RatchetMessage {
    pub header: RatchetHeader,
    pub ciphertext: Vec<u8>,
}
