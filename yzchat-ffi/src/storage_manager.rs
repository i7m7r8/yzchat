//! StorageManager: FFI wrapper for persistent storage

use crate::types::*;
use anyhow::{Context, Result};
use redb::{Database, TableDefinition, ReadableTable, MultimapTableDefinition};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uniffi::Object;
use uuid::Uuid;
use yzchat_crypto::types::*;

/// Storage manager for FFI
#[derive(Object)]
pub struct StorageManager {
    db: Arc<Database>,
    message_callback: Arc<Mutex<Option<Box<dyn crate::callbacks::MessageEventCallback>>>>,
    contact_callback: Arc<Mutex<Option<Box<dyn crate::callbacks::ContactEventCallback>>>>,
}

impl StorageManager {
    pub fn new(data_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&data_dir).unwrap();
        let db_path = data_dir.join("yzchat.redb");
        let db = Arc::new(Database::create(db_path).unwrap());
        Self::init_tables(&db);
        Self {
            db,
            message_callback: Arc::new(Mutex::new(None)),
            contact_callback: Arc::new(Mutex::new(None)),
        }
    }

    fn init_tables(db: &Database) {
        let write_txn = db.begin_write().unwrap();
        {
            // Identity table
            let _: TableDefinition<&str, &[u8]> = TableDefinition::new("identity");
            // Encryption keypair table
            let _: TableDefinition<&str, &[u8]> = TableDefinition::new("encryption_keypair");
            // Pre-keys table
            let _: MultimapTableDefinition<u32, &[u8]> = MultimapTableDefinition::new("pre_keys");
            // Contacts table
            let _: TableDefinition<&[u8], &[u8]> = TableDefinition::new("contacts");
            // Conversations table
            let _: TableDefinition<&str, &[u8]> = TableDefinition::new("conversations");
            // Messages table (multimap by conversation_id)
            let _: MultimapTableDefinition<&str, &[u8]> = MultimapTableDefinition::new("messages");
            // Groups table
            let _: TableDefinition<&str, &[u8]> = TableDefinition::new("groups");
            // Group members table
            let _: MultimapTableDefinition<&str, &[u8]> = TableDefinition::new("group_members");
            // Settings table
            let _: TableDefinition<&str, &[u8]> = TableDefinition::new("settings");
            // Key bundles table
            let _: TableDefinition<&[u8], &[u8]> = TableDefinition::new("key_bundles");
        }
        write_txn.commit().unwrap();
    }

    // Identity
    pub async fn save_identity(&self, key: IdentitySecretKey) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("identity"))?;
            table.insert("current", &key.bytes)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_identity(&self) -> Result<Option<IdentitySecretKey>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TableDefinition::new("identity"))?;
        Ok(table.get("current")?.map(|v| IdentitySecretKey { bytes: v.value().to_vec() }))
    }

    pub async fn load_identity_bytes(&self) -> Result<Option<Vec<u8>>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TableDefinition::new("identity"))?;
        Ok(table.get("current")?.map(|v| v.value().to_vec()))
    }

    pub async fn delete_identity(&self) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("identity"))?;
            table.remove("current")?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // Encryption keypair
    pub async fn save_encryption_keypair_bytes(&self, bytes: &[u8]) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("encryption_keypair"))?;
            table.insert("current", bytes)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_encryption_keypair_bytes(&self) -> Result<Option<Vec<u8>>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TableDefinition::new("encryption_keypair"))?;
        Ok(table.get("current")?.map(|v| v.value().to_vec()))
    }

    // Pre-keys
    pub async fn save_pre_keys(&self, keys: Vec<PreKey>) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_multimap_table(MultimapTableDefinition::new("pre_keys"))?;
            for key in keys {
                let bytes = serde_json::to_vec(&key)?;
                table.insert(key.key_id, &bytes)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_pre_keys(&self) -> Result<Vec<PreKey>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_multimap_table(MultimapTableDefinition::new("pre_keys"))?;
        let mut keys = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let key: PreKey = serde_json::from_slice(value.value())?;
            keys.push(key);
        }
        Ok(keys)
    }

    pub async fn consume_one_time_pre_key(&self, key_id: u32) -> Result<Option<PreKey>> {
        let write_txn = self.db.begin_write()?;
        let key = {
            let table = write_txn.open_multimap_table(MultimapTableDefinition::new("pre_keys"))?;
            if let Some(value) = table.get(key_id)?.next() {
                let key: PreKey = serde_json::from_slice(value.value())?;
                table.remove(key_id, value.value())?;
                Some(key)
            } else {
                None
            }
        };
        write_txn.commit()?;
        Ok(key)
    }

    // Contacts
    pub async fn save_contact(&self, identity: IdentityPublicKey, alias: String) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("contacts"))?;
            let contact = Contact {
                identity: identity.clone(),
                alias,
                is_online: false,
                last_seen: None,
                capabilities: PeerCapabilities::default(),
            };
            table.insert(&identity.bytes, &serde_json::to_vec(&contact)?)?;
        }
        write_txn.commit()?;
        self.notify_contact_event(contact, ContactEvent::Added).await;
        Ok(())
    }

    pub async fn delete_contact(&self, identity: IdentityPublicKey) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("contacts"))?;
            table.remove(&identity.bytes)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_contacts(&self) -> Result<Vec<Contact>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TableDefinition::new("contacts"))?;
        let mut contacts = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let contact: Contact = serde_json::from_slice(value.value())?;
            contacts.push(contact);
        }
        Ok(contacts)
    }

    pub async fn update_contact_alias(&self, identity: IdentityPublicKey, alias: String) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("contacts"))?;
            if let Some(value) = table.get(&identity.bytes)? {
                let mut contact: Contact = serde_json::from_slice(value.value())?;
                contact.alias = alias;
                table.insert(&identity.bytes, &serde_json::to_vec(&contact)?)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    // Conversations
    pub async fn create_conversation(&self, conversation_id: String, participants: Vec<IdentityPublicKey>) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("conversations"))?;
            let conv = Conversation {
                id: conversation_id.clone(),
                participants: participants.iter().map(|p| p.bytes.clone()).collect(),
                created_at: chrono::Utc::now().timestamp_millis(),
                updated_at: chrono::Utc::now().timestamp_millis(),
            };
            table.insert(&conversation_id, &serde_json::to_vec(&conv)?)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Option<Conversation>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TableDefinition::new("conversations"))?;
        Ok(table.get(conversation_id)?.map(|v| serde_json::from_slice(v.value()).unwrap()))
    }

    // Messages
    pub async fn save_message(&self, conversation_id: String, message: EncryptedMessage) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_multimap_table(MultimapTableDefinition::new("messages"))?;
            let msg = StoredMessage {
                id: Uuid::new_v4().to_string(),
                conversation_id: conversation_id.clone(),
                message,
                timestamp: chrono::Utc::now().timestamp_millis(),
                is_read: false,
                is_delivered: false,
            };
            table.insert(&conversation_id, &serde_json::to_vec(&msg)?)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_messages(&self, conversation_id: String, limit: u32) -> Result<Vec<Message>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_multimap_table(MultimapTableDefinition::new("messages"))?;
        let mut messages = Vec::new();
        for entry in table.get(&conversation_id)?.take(limit as usize) {
            let (_, value) = entry?;
            let stored: StoredMessage = serde_json::from_slice(value.value())?;
            // Decrypt message (simplified)
            messages.push(Message {
                id: stored.id,
                conversation_id: stored.conversation_id,
                sender: stored.message.sender_identity,
                content: String::from_utf8_lossy(&stored.message.ciphertext).to_string(), // TODO: decrypt
                message_type: stored.message.message_type,
                timestamp: stored.timestamp,
                is_read: stored.is_read,
                is_delivered: stored.is_delivered,
                attachments: vec![],
            });
        }
        Ok(messages)
    }

    pub async fn mark_message_read(&self, conversation_id: String, message_id: String) -> Result<()> {
        // TODO: implement read receipt tracking
        Ok(())
    }

    // Groups
    pub async fn create_group(&self, group_id: String, name: String, creator: IdentityPublicKey, members: Vec<IdentityPublicKey>) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("groups"))?;
            let group = Group {
                id: group_id.clone(),
                name,
                creator: creator.clone(),
                members: members.iter().map(|m| GroupMember {
                    identity: m.clone(),
                    alias: "".to_string(),
                    role: if m == creator { GroupRole::Owner } else { GroupRole::Member },
                    joined_at: chrono::Utc::now().timestamp_millis(),
                }).collect(),
                created_at: chrono::Utc::now().timestamp_millis(),
                updated_at: chrono::Utc::now().timestamp_millis(),
            };
            table.insert(&group_id, &serde_json::to_vec(&group)?)?;

            // Add members to multimap
            let mut member_table = write_txn.open_multimap_table(MultimapTableDefinition::new("group_members"))?;
            for member in &group.members {
                member_table.insert(&group_id, &serde_json::to_vec(member)?)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn leave_group(&self, group_id: String) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("groups"))?;
            table.remove(&group_id)?;
            let mut member_table = write_txn.open_multimap_table(MultimapTableDefinition::new("group_members"))?;
            member_table.remove(&group_id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_groups(&self) -> Result<Vec<Group>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TableDefinition::new("groups"))?;
        let mut groups = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let group: Group = serde_json::from_slice(value.value())?;
            groups.push(group);
        }
        Ok(groups)
    }

    // Key bundles
    pub async fn save_key_bundle(&self, identity: IdentityPublicKey, bundle: KeyBundle) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("key_bundles"))?;
            table.insert(&identity.bytes, &serde_json::to_vec(&bundle)?)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_key_bundle(&self, identity: &IdentityPublicKey) -> Result<KeyBundle> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TableDefinition::new("key_bundles"))?;
        Ok(table.get(&identity.bytes)?
            .map(|v| serde_json::from_slice(v.value()).unwrap())
            .ok_or_else(|| anyhow::anyhow!("Key bundle not found"))?)
    }

    pub async fn get_node_id_for_identity(&self, identity: &IdentityPublicKey) -> Result<Option<Vec<u8>>> {
        // TODO: map identity to node ID
        Ok(None)
    }

    // Settings
    pub async fn save_setting(&self, key: String, value: String) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("settings"))?;
            table.insert(&key, &value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub async fn load_setting(&self, key: String) -> Result<Option<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TableDefinition::new("settings"))?;
        Ok(table.get(&key)?.map(|v| v.value().to_string()))
    }

    pub async fn delete_setting(&self, key: String) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TableDefinition::new("settings"))?;
            table.remove(&key)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // Backup/Restore
    pub async fn export_backup(&self, password: String) -> Result<Vec<u8>> {
        // Export all tables to encrypted archive
        // Simplified: just return empty for now
        Ok(vec![])
    }

    pub async fn import_backup(&self, data: Vec<u8>, password: String) -> Result<()> {
        // Decrypt and import
        Ok(())
    }

    // Callbacks
    pub fn set_message_callback(&self, callback: Box<dyn crate::callbacks::MessageEventCallback>) {
        *self.message_callback.blocking_lock() = Some(callback);
    }

    pub fn set_contact_callback(&self, callback: Box<dyn crate::callbacks::ContactEventCallback>) {
        *self.contact_callback.blocking_lock() = Some(callback);
    }

    async fn notify_message_event(&self, message: Message) {
        if let Some(cb) = self.message_callback.lock().await.as_ref() {
            cb(message);
        }
    }

    async fn notify_contact_event(&self, contact: Contact, event: ContactEvent) {
        if let Some(cb) = self.contact_callback.lock().await.as_ref() {
            cb(contact, event);
        }
    }
}

// Internal types
#[derive(serde::Serialize, serde::Deserialize)]
struct Conversation {
    id: String,
    participants: Vec<Vec<u8>>,
    created_at: i64,
    updated_at: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StoredMessage {
    id: String,
    conversation_id: String,
    message: EncryptedMessage,
    timestamp: i64,
    is_read: bool,
    is_delivered: bool,
}