//! CryptoManager: FFI wrapper for yzchat-crypto

use crate::types::*;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use uniffi::Object;
use yzchat_crypto::{
    hybrid_kem::{HybridKem, HybridKemKeypair},
    hybrid_sign::{HybridSign, HybridSignKeypair},
    ratchet::{DoubleRatchet, RatchetState},
    types::*,
    IdentityPublicKey as CryptoIdentityPublicKey,
    IdentitySecretKey as CryptoIdentitySecretKey,
    EncryptionPublicKey as CryptoEncryptionPublicKey,
    EncryptionSecretKey as CryptoEncryptionSecretKey,
    PreKey as CryptoPreKey,
    SignedPreKey as CryptoSignedPreKey,
    OneTimePreKey as CryptoOneTimePreKey,
    KeyBundle as CryptoKeyBundle,
    EncryptedMessage as CryptoEncryptedMessage,
};
use crate::storage_manager::StorageManager;

/// Crypto manager for FFI
#[derive(Object)]
pub struct CryptoManager {
    storage: Arc<StorageManager>,
    identity: Arc<Mutex<Option<HybridSignKeypair>>>,
    encryption_keypair: Arc<Mutex<Option<HybridKemKeypair>>>,
    ratchet_states: Arc<Mutex<std::collections::HashMap<String, RatchetState>>>,
}

impl CryptoManager {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self {
            storage,
            identity: Arc::new(Mutex::new(None)),
            encryption_keypair: Arc::new(Mutex::new(None)),
            ratchet_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Load identity from storage
        if let Some(identity_bytes) = self.storage.load_identity_bytes().await? {
            let identity = HybridSignKeypair::from_bytes(&identity_bytes)?;
            *self.identity.lock().await = Some(identity);
        }

        // Load encryption keypair
        if let Some(enc_bytes) = self.storage.load_encryption_keypair_bytes().await? {
            let (pk_bytes, sk_bytes) = split_keypair_bytes(&enc_bytes)?;
            let keypair = HybridKemKeypair::from_bytes(&pk_bytes, &sk_bytes)?;
            *self.encryption_keypair.lock().await = Some(keypair);
        } else {
            // Generate new encryption keypair
            let keypair = HybridKem::generate()?;
            self.storage.save_encryption_keypair_bytes(&keypair.secret_key_bytes()).await?;
            *self.encryption_keypair.lock().await = Some(keypair);
        }

        Ok(())
    }

    // Identity management
    pub async fn generate_identity(&self) -> Result<(IdentityPublicKey, IdentitySecretKey)> {
        let keypair = HybridSign::generate()?;
        let public = IdentityPublicKey { bytes: keypair.verify_key_bytes() };
        let secret = IdentitySecretKey { bytes: keypair.secret_key_bytes() };
        Ok((public, secret))
    }

    pub fn export_identity_public(&self, key: IdentitySecretKey) -> IdentityPublicKey {
        let keypair = HybridSignKeypair::from_bytes(&key.bytes).unwrap();
        IdentityPublicKey { bytes: keypair.verify_key_bytes() }
    }

    pub fn export_identity_secret(&self, key: IdentitySecretKey) -> IdentitySecretKey {
        key
    }

    pub fn import_identity(&self, public: IdentityPublicKey, secret: IdentitySecretKey) -> IdentitySecretKey {
        // Verify they match
        let keypair = HybridSignKeypair::from_bytes(&secret.bytes).unwrap();
        assert_eq!(keypair.verify_key_bytes(), public.bytes);
        secret
    }

    // Pre-keys
    pub async fn generate_pre_keys(&self, count: u32) -> Result<Vec<PreKey>> {
        let mut keys = Vec::new();
        for i in 0..count {
            let kem_keypair = HybridKem::generate()?;
            let identity = self.identity.lock().await;
            let identity = identity.as_ref().ok_or_else(|| anyhow::anyhow!("No identity"))?;

            let signed_pre_key = HybridSign::sign(
                identity,
                &kem_keypair.public_key_bytes(),
                yzchat_crypto::domain::IDENTITY_BINDING,
            )?;

            keys.push(PreKey {
                key_id: i,
                public_key: EncryptionPublicKey { bytes: kem_keypair.public_key_bytes() },
                signature: yzchat_crypto::hybrid_sign::HybridSign::signature_to_bytes(&signed_pre_key),
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
        }
        Ok(keys)
    }

    pub async fn generate_signed_pre_key(&self, identity: IdentitySecretKey) -> Result<SignedPreKey> {
        let kem_keypair = HybridKem::generate()?;
        let identity_keypair = HybridSignKeypair::from_bytes(&identity.bytes)?;

        let signed = HybridSign::sign(
            &identity_keypair,
            &kem_keypair.public_key_bytes(),
            yzchat_crypto::domain::IDENTITY_BINDING,
        )?;

        Ok(SignedPreKey {
            key_id: rand::random(),
            public_key: EncryptionPublicKey { bytes: kem_keypair.public_key_bytes() },
            signature: yzchat_crypto::hybrid_sign::HybridSign::signature_to_bytes(&signed),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub async fn generate_one_time_pre_keys(&self, count: u32) -> Result<Vec<PreKey>> {
        // Same as generate_pre_keys but marked as one-time
        self.generate_pre_keys(count).await
    }

    pub async fn create_key_bundle(
        &self,
        identity: IdentityPublicKey,
        signed_pre_key: SignedPreKey,
        one_time_keys: Vec<PreKey>,
    ) -> Result<KeyBundle> {
        Ok(KeyBundle {
            identity_key: identity,
            signed_pre_key,
            one_time_pre_keys: one_time_keys,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    // Encryption
    pub async fn encrypt_message(
        &self,
        sender_identity: IdentitySecretKey,
        recipient_bundle: KeyBundle,
        plaintext: Vec<u8>,
        message_type: MessageType,
    ) -> Result<EncryptedMessage> {
        // Get or create ratchet state for this recipient
        let recipient_id = hex::encode(&recipient_bundle.identity_key.bytes);
        let mut ratchet_states = self.ratchet_states.lock().await;

        let ratchet_state = if let Some(state) = ratchet_states.get_mut(&recipient_id) {
            state.clone()
        } else {
            // Initialize new ratchet
            let sender_enc = self.encryption_keypair.lock().await.as_ref()
                .ok_or_else(|| anyhow::anyhow!("No encryption keypair"))?.public_key();
            let recipient_enc = recipient_bundle.signed_pre_key.public_key;

            // Perform X3DH key agreement (simplified)
            let (_, shared_secret) = HybridKem::encapsulate(&recipient_enc.into())?;
            let mut state = DoubleRatchet::initialize(&shared_secret, true)?;
            ratchet_states.insert(recipient_id.clone(), state.clone());
            state
        };

        // Encrypt with double ratchet
        let associated_data = b"yzchat-message";
        let (ciphertext, msg_num, dh_pk) = DoubleRatchet::encrypt(&mut ratchet_state, &plaintext, associated_data)?;

        // Update stored state
        ratchet_states.insert(recipient_id, ratchet_state);

        // Build ratchet header
        let ratchet_header = RatchetHeader {
            dh_public: EncryptionPublicKey { bytes: HybridKem::public_key_to_bytes(&dh_pk) },
            message_number: msg_num,
            previous_chain_length: 0, // TODO
        };

        Ok(EncryptedMessage {
            message_type,
            sender_identity: IdentityPublicKey { bytes: sender_identity.bytes }, // TODO: derive from secret
            recipient_identity: recipient_bundle.identity_key,
            timestamp: chrono::Utc::now().timestamp_millis(),
            nonce: vec![], // Included in ciphertext
            ciphertext,
            ratchet_header: Some(ratchet_header),
        })
    }

    pub async fn decrypt_message(
        &self,
        recipient_identity: IdentitySecretKey,
        message: EncryptedMessage,
    ) -> Result<Vec<u8>> {
        let sender_id = hex::encode(&message.sender_identity.bytes);
        let mut ratchet_states = self.ratchet_states.lock().await;

        let ratchet_state = ratchet_states.get_mut(&sender_id)
            .ok_or_else(|| anyhow::anyhow!("No ratchet state for sender"))?;

        let header = message.ratchet_header.ok_or_else(|| anyhow::anyhow!("Missing ratchet header"))?;
        let dh_pk = HybridKem::public_key_from_bytes(&header.dh_public.bytes)?;

        let plaintext = DoubleRatchet::decrypt(
            ratchet_state,
            &message.ciphertext,
            b"yzchat-message",
            header.message_number,
            &dh_pk,
        )?;

        Ok(plaintext)
    }

    // Signing
    pub async fn sign_data(&self, identity: IdentitySecretKey, data: Vec<u8>) -> Result<Vec<u8>> {
        let keypair = HybridSignKeypair::from_bytes(&identity.bytes)?;
        let signature = HybridSign::sign(&keypair, &data, yzchat_crypto::domain::IDENTITY_BINDING)?;
        Ok(yzchat_crypto::hybrid_sign::HybridSign::signature_to_bytes(&signature))
    }

    pub async fn verify_signature(&self, public: IdentityPublicKey, data: Vec<u8>, signature: Vec<u8>) -> Result<bool> {
        let vk = yzchat_crypto::hybrid_sign::HybridVerifyKey::from_bytes(&public.bytes)?;
        let sig = yzchat_crypto::hybrid_sign::HybridSignature::from_bytes(&signature)?;
        HybridSign::verify(&vk, &data, &sig, yzchat_crypto::domain::IDENTITY_BINDING).map(|_| true).map_err(|_| false)
    }

    // Internal helper
    pub fn get_encryption_keypair(&self) -> Arc<Mutex<Option<HybridKemKeypair>>> {
        self.encryption_keypair.clone()
    }

    pub async fn get_key_bundle(&self, identity: &IdentityPublicKey) -> Result<KeyBundle> {
        self.storage.load_key_bundle(identity).await
    }
}

use std::collections::HashMap;

fn split_keypair_bytes(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    // Assuming format: [public_key][secret_key]
    // This is simplified - real impl uses proper serialization
    let pk_len = yzchat_crypto::sizes::HYBRID_KEM_PUBLIC;
    if bytes.len() < pk_len {
        return Err(anyhow::anyhow!("Invalid keypair bytes"));
    }
    Ok((bytes[..pk_len].to_vec(), bytes[pk_len..].to_vec()))
}

// Extension traits for FFI types
impl From<IdentityPublicKey> for yzchat_crypto::types::IdentityPublicKey {
    fn from(key: IdentityPublicKey) -> Self {
        Self(key.bytes)
    }
}

impl From<IdentitySecretKey> for yzchat_crypto::types::IdentitySecretKey {
    fn from(key: IdentitySecretKey) -> Self {
        Self(key.bytes)
    }
}

impl From<EncryptionPublicKey> for yzchat_crypto::types::EncryptionPublicKey {
    fn from(key: EncryptionPublicKey) -> Self {
        Self(key.bytes)
    }
}

impl From<&PreKey> for yzchat_crypto::types::PreKey {
    fn from(key: &PreKey) -> Self {
        Self {
            key_id: key.key_id,
            public_key: key.public_key.clone().into(),
            signature: yzchat_crypto::hybrid_sign::HybridSignature::from_bytes(&key.signature).unwrap(),
            timestamp: key.timestamp,
        }
    }
}