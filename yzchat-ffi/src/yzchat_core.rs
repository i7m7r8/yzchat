//! YzchatCore: Main entry point for the FFI layer

use crate::{
    callbacks::*, crypto_manager::CryptoManager, media_manager::MediaManager,
    p2p_manager::P2PManager, storage_manager::StorageManager, types::*,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uniffi::Object;
use uuid::Uuid;

/// Core configuration
#[derive(uniffi::Record, Clone, Debug)]
pub struct CoreConfig {
    pub data_dir: String,
    pub enable_logging: bool,
    pub log_level: String,
    pub auto_accept_calls: bool,
    pub auto_download_media: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            data_dir: "".to_string(),
            enable_logging: true,
            log_level: "info".to_string(),
            auto_accept_calls: false,
            auto_download_media: true,
        }
    }
}

/// Main yzchat core object
#[derive(Object)]
pub struct YzchatCore {
    config: CoreConfig,
    crypto: Arc<CryptoManager>,
    p2p: Arc<Mutex<Option<P2PManager>>>,
    media: Arc<Mutex<Option<MediaManager>>>,
    storage: Arc<StorageManager>,
    initialized: Arc<Mutex<bool>>,
}

impl YzchatCore {
    /// Create new core instance
    #[uniffi::constructor]
    pub fn new(config: CoreConfig) -> Arc<Self> {
        // Initialize logging if enabled
        if config.enable_logging {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(&config.log_level)
                .with_ansi(false)
                .try_init();
        }

        // Resolve data directory
        let data_dir = if config.data_dir.is_empty() {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("yzchat")
        } else {
            PathBuf::from(&config.data_dir)
        };

        // Create storage
        let storage = Arc::new(StorageManager::new(data_dir.clone()));

        // Create crypto manager
        let crypto = Arc::new(CryptoManager::new(storage.clone()));

        Arc::new(Self {
            config,
            crypto,
            p2p: Arc::new(Mutex::new(None)),
            media: Arc::new(Mutex::new(None)),
            storage,
            initialized: Arc::new(Mutex::new(false)),
        })
    }

    /// Initialize all subsystems
    pub async fn initialize(&self) -> Result<()> {
        let mut initialized = self.initialized.lock().await;
        if *initialized {
            return Ok(());
        }

        // Initialize crypto
        self.crypto.initialize().await?;

        // Initialize P2P
        let p2p_config = yzchat_p2p::endpoint::EndpointConfig::default();
        let endpoint = Arc::new(yzchat_p2p::endpoint::NodeEndpoint::new(p2p_config).await?);
        let p2p_manager =
            P2PManager::new(endpoint, self.crypto.clone(), self.storage.clone()).await?;
        *self.p2p.lock().await = Some(p2p_manager);

        // Initialize media
        let media_manager = MediaManager::new(self.crypto.clone()).await?;
        *self.media.lock().await = Some(media_manager);

        *initialized = true;
        Ok(())
    }

    /// Shutdown all subsystems
    pub async fn shutdown(&self) -> Result<()> {
        let mut initialized = self.initialized.lock().await;
        if !*initialized {
            return Ok(());
        }

        // Shutdown media
        if let Some(media) = self.media.lock().await.take() {
            media.shutdown().await?;
        }

        // Shutdown P2P
        if let Some(p2p) = self.p2p.lock().await.take() {
            p2p.shutdown().await?;
        }

        *initialized = false;
        Ok(())
    }

    /// Get crypto manager
    pub fn crypto(&self) -> Arc<CryptoManager> {
        self.crypto.clone()
    }

    /// Get P2P manager
    pub async fn p2p(&self) -> Arc<P2PManager> {
        self.p2p
            .lock()
            .await
            .as_ref()
            .expect("P2P not initialized")
            .clone()
    }

    /// Get media manager
    pub async fn media(&self) -> Arc<MediaManager> {
        self.media
            .lock()
            .await
            .as_ref()
            .expect("Media not initialized")
            .clone()
    }

    /// Get storage manager
    pub fn storage(&self) -> Arc<StorageManager> {
        self.storage.clone()
    }

    // ============================================
    // High-level operations
    // ============================================

    /// Register a new user (generate identity)
    pub async fn register_user(&self, display_name: String) -> Result<IdentityPublicKey> {
        let (public, secret) = self.crypto.generate_identity().await?;
        self.storage.save_identity(secret).await?;
        self.storage
            .save_setting("display_name".to_string(), display_name)
            .await?;
        Ok(public)
    }

    /// Login (load identity)
    pub async fn login(&self) -> Result<bool> {
        Ok(self.storage.load_identity().await?.is_some())
    }

    /// Logout (clear identity)
    pub async fn logout(&self) -> Result<()> {
        self.storage.delete_identity().await?;
        Ok(())
    }

    /// Check if logged in
    pub async fn is_logged_in(&self) -> bool {
        self.storage
            .load_identity()
            .await
            .is_ok_and(|k| k.is_some())
    }

    // Contacts
    pub async fn add_contact(&self, identity: IdentityPublicKey, alias: String) -> Result<()> {
        self.storage.save_contact(identity, alias).await
    }

    pub async fn remove_contact(&self, identity: IdentityPublicKey) -> Result<()> {
        self.storage.delete_contact(identity).await
    }

    pub async fn get_contacts(&self) -> Result<Vec<Contact>> {
        self.storage.load_contacts().await
    }

    pub async fn update_contact_alias(
        &self,
        identity: IdentityPublicKey,
        alias: String,
    ) -> Result<()> {
        self.storage.update_contact_alias(identity, alias).await
    }

    // Conversations
    pub async fn create_conversation(
        &self,
        participants: Vec<IdentityPublicKey>,
    ) -> Result<String> {
        let conversation_id = Uuid::new_v4().to_string();
        self.storage
            .create_conversation(conversation_id.clone(), participants)
            .await?;
        Ok(conversation_id)
    }

    pub async fn send_message(&self, conversation_id: String, text: String) -> Result<()> {
        let identity = self
            .storage
            .load_identity()
            .await?
            .ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        let conversation = self
            .storage
            .get_conversation(&conversation_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Conversation not found"))?;

        // Encrypt for each participant
        for participant in conversation.participants {
            if participant != identity.public_key() {
                let bundle = self.crypto.get_key_bundle(&participant).await?;
                let message = self
                    .crypto
                    .encrypt_message(&identity, bundle, text.as_bytes(), MessageType::Message)
                    .await?;
                // Send via P2P
                self.p2p()
                    .await
                    .send_message(
                        participant.node_id(),
                        yzchat_p2p::protocols::MESSAGING,
                        serde_json::to_vec(&message)?,
                    )
                    .await?;
            }
        }

        // Save locally
        self.storage.save_message(conversation_id, message).await?;
        Ok(())
    }

    pub async fn get_messages(&self, conversation_id: String, limit: u32) -> Result<Vec<Message>> {
        self.storage.load_messages(conversation_id, limit).await
    }

    pub async fn mark_read(&self, conversation_id: String, message_id: String) -> Result<()> {
        self.storage
            .mark_message_read(conversation_id, message_id)
            .await
    }

    // Groups
    pub async fn create_group(
        &self,
        name: String,
        members: Vec<IdentityPublicKey>,
    ) -> Result<String> {
        let group_id = Uuid::new_v4().to_string();
        let identity = self
            .storage
            .load_identity()
            .await?
            .ok_or_else(|| anyhow::anyhow!("Not logged in"))?;
        self.storage
            .create_group(group_id.clone(), name, identity.public_key(), members)
            .await?;
        Ok(group_id)
    }

    pub async fn join_group(&self, invite_link: String) -> Result<()> {
        // Parse invite link and join
        todo!("Implement group join")
    }

    pub async fn leave_group(&self, group_id: String) -> Result<()> {
        self.storage.leave_group(group_id).await
    }

    pub async fn get_groups(&self) -> Result<Vec<Group>> {
        self.storage.load_groups().await
    }

    pub async fn send_group_message(&self, group_id: String, text: String) -> Result<()> {
        todo!("Implement group messaging")
    }

    // Calls
    pub async fn start_call(
        &self,
        remote_identity: IdentityPublicKey,
        session_type: MediaSessionType,
    ) -> Result<String> {
        let call_id = Uuid::new_v4().to_string();
        let p2p = self.p2p().await;
        let media = self.media().await;

        // Get remote node ID from identity
        let remote_node = self
            .storage
            .get_node_id_for_identity(&remote_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Unknown contact"))?;

        // Create signaling session
        let session_id = p2p.create_call_session(remote_node).await?;

        // Start media
        let constraints = MediaConstraints::default();
        match session_type {
            MediaSessionType::VoiceCall => {
                media.start_audio_call(remote_node, constraints).await?;
            }
            MediaSessionType::VideoCall => {
                media.start_video_call(remote_node, constraints).await?;
            }
            MediaSessionType::ScreenShare => {
                media.start_screen_share(remote_node, constraints).await?;
            }
            MediaSessionType::GroupCall => {
                todo!("Group calls");
            }
        }

        Ok(call_id)
    }

    pub async fn answer_call(&self, call_id: String) -> Result<()> {
        self.media().await.answer_call(call_id).await
    }

    pub async fn end_call(&self, call_id: String) -> Result<()> {
        self.media().await.end_call(call_id).await
    }

    // Event callbacks
    pub fn on_message(&self, callback: Box<dyn MessageEventCallback>) {
        self.storage.set_message_callback(callback);
    }

    pub fn on_call(&self, callback: Box<dyn CallEventCallback>) {
        self.media
            .lock()
            .await
            .as_ref()
            .map(|m| m.set_call_callback(callback));
    }

    pub fn on_contact(&self, callback: Box<dyn ContactEventCallback>) {
        self.storage.set_contact_callback(callback);
    }

    pub fn on_connection_change(&self, callback: Box<dyn ConnectionChangeCallback>) {
        self.p2p
            .lock()
            .await
            .as_ref()
            .map(|p| p.set_connection_callback(callback));
    }
}
