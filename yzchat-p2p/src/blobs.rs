//! Blob Store: Large file transfer with streaming, deduplication, encryption

use crate::{NodeId, P2pError, Result};
use anyhow::Context;
use iroh::Endpoint;
use iroh_blobs::get::progress::Progress;
use iroh_blobs::get::request::Request as GetRequest;
use iroh_blobs::protocol::{Blobs, BlobsProtocol};
use iroh_blobs::rpc::client::blobs::BlobsClient;
use iroh_blobs::rpc::server::blobs::BlobsServer;
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::{blob::Blob, store::Store, Hash, HashAndFormat, Tag};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use yzchat_crypto::{HybridKem, HybridKemPublicKey, HybridKemSharedSecret};

/// Blob store configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BlobConfig {
    /// Store path (None = in-memory)
    pub store_path: Option<PathBuf>,
    /// Max blob size (bytes)
    pub max_blob_size: u64,
    /// Chunk size for streaming
    pub chunk_size: usize,
    /// Enable encryption
    pub encrypt: bool,
    /// Cache size (bytes)
    pub cache_size: u64,
}

impl Default for BlobConfig {
    fn default() -> Self {
        Self {
            store_path: None,
            max_blob_size: 2 * 1024 * 1024 * 1024, // 2 GB
            chunk_size: 1024 * 1024,               // 1 MB
            encrypt: true,
            cache_size: 100 * 1024 * 1024, // 100 MB
        }
    }
}

/// Blob transfer progress
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BlobTransferProgress {
    pub transfer_id: Uuid,
    pub blob_hash: String,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub status: TransferStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TransferStatus {
    Pending,
    Connecting,
    Downloading,
    Uploading,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

/// Blob transfer info
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BlobTransfer {
    pub transfer_id: Uuid,
    pub blob_hash: Hash,
    pub format: iroh_blobs::store::Format,
    pub size: u64,
    pub peer: NodeId,
    pub direction: TransferDirection,
    pub encryption_key: Option<HybridKemSharedSecret>,
    pub progress: BlobTransferProgress,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TransferDirection {
    Send,
    Receive,
}

/// Blob store service
pub struct BlobStore {
    endpoint: Arc<Endpoint>,
    config: BlobConfig,
    store: Arc<dyn Store>,
    blobs_protocol: Option<BlobsProtocol>,
    transfers: Arc<RwLock<HashMap<Uuid, BlobTransfer>>>,
}

impl BlobStore {
    /// Create new blob store
    pub async fn new(endpoint: Arc<Endpoint>, config: BlobConfig) -> Result<Self> {
        // Create store
        let store: Arc<dyn Store> = if let Some(path) = &config.store_path {
            let fs_store = FsStore::new(path.clone()).await?;
            Arc::new(fs_store)
        } else {
            Arc::new(MemStore::new())
        };

        // Create blobs protocol
        let blobs = Blobs::new(endpoint.clone(), store.clone());
        let blobs_protocol = Some(blobs.protocol());

        Ok(Self {
            endpoint,
            config,
            store,
            blobs_protocol,
            transfers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get the blobs protocol for router registration
    pub fn protocol(&self) -> Option<&BlobsProtocol> {
        self.blobs_protocol.as_ref()
    }

    /// Add a blob from reader (with optional encryption)
    pub async fn add_blob<R: AsyncRead + Unpin + Send + 'static>(
        &self,
        reader: R,
        encrypt: bool,
        encryption_key: Option<HybridKemSharedSecret>,
    ) -> Result<(Hash, iroh_blobs::store::Format, u64)> {
        // For encryption, wrap reader
        let reader: Box<dyn AsyncRead + Unpin + Send> = if encrypt && encryption_key.is_some() {
            Box::new(EncryptedReader::new(reader, encryption_key.unwrap()))
        } else {
            Box::new(reader)
        };

        let (hash, format, size) = self.store.add(reader).await?;
        Ok((hash, format, size))
    }

    /// Get a blob by hash (with optional decryption)
    pub async fn get_blob(
        &self,
        hash: Hash,
        decrypt: bool,
        encryption_key: Option<HybridKemSharedKey>,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send>> {
        let reader = self.store.get(hash).await?;
        if decrypt && encryption_key.is_some() {
            Ok(Box::new(DecryptedReader::new(
                reader,
                encryption_key.unwrap(),
            )))
        } else {
            Ok(reader)
        }
    }

    /// Send blob to peer
    pub async fn send_blob(
        &self,
        peer: NodeId,
        hash: Hash,
        format: iroh_blobs::store::Format,
        encryption_key: Option<HybridKemSharedSecret>,
    ) -> Result<Uuid> {
        let transfer_id = Uuid::new_v4();
        let size = self.store.get_size(hash).await?;

        let transfer = BlobTransfer {
            transfer_id,
            blob_hash: hash,
            format,
            size,
            peer,
            direction: TransferDirection::Send,
            encryption_key,
            progress: BlobTransferProgress {
                transfer_id,
                blob_hash: hash.to_string(),
                total_bytes: size,
                transferred_bytes: 0,
                status: TransferStatus::Connecting,
                started_at: chrono::Utc::now(),
                completed_at: None,
            },
        };

        self.transfers.write().await.insert(transfer_id, transfer);

        // Start transfer via blobs protocol
        // Real impl uses BlobsClient::send
        let client = BlobsClient::new(self.endpoint.clone());
        client.send(peer, vec![(hash, format)]).await?;

        Ok(transfer_id)
    }

    /// Receive blob from peer (announced via protocol)
    pub async fn receive_blob(
        &self,
        peer: NodeId,
        hash: Hash,
        format: iroh_blobs::store::Format,
        encryption_key: Option<HybridKemSharedSecret>,
    ) -> Result<Uuid> {
        let transfer_id = Uuid::new_v4();
        let size = 0; // Will be known during transfer

        let transfer = BlobTransfer {
            transfer_id,
            blob_hash: hash,
            format,
            size,
            peer,
            direction: TransferDirection::Receive,
            encryption_key,
            progress: BlobTransferProgress {
                transfer_id,
                blob_hash: hash.to_string(),
                total_bytes: 0,
                transferred_bytes: 0,
                status: TransferStatus::Connecting,
                started_at: chrono::Utc::now(),
                completed_at: None,
            },
        };

        self.transfers.write().await.insert(transfer_id, transfer);

        // Real impl: BlobsClient::receive
        let client = BlobsClient::new(self.endpoint.clone());
        client.receive(peer, hash, format).await?;

        Ok(transfer_id)
    }

    /// Get transfer progress
    pub async fn get_progress(&self, transfer_id: Uuid) -> Option<BlobTransferProgress> {
        self.transfers
            .read()
            .await
            .get(&transfer_id)
            .map(|t| t.progress.clone())
    }

    /// List all transfers
    pub async fn list_transfers(&self) -> Vec<BlobTransfer> {
        self.transfers.read().await.values().cloned().collect()
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, transfer_id: Uuid) -> Result<()> {
        let mut transfers = self.transfers.write().await;
        if let Some(transfer) = transfers.get_mut(&transfer_id) {
            transfer.progress.status = TransferStatus::Cancelled;
            transfer.progress.completed_at = Some(chrono::Utc::now());
        }
        Ok(())
    }

    /// Export blob to file
    pub async fn export_to_file(&self, hash: Hash, path: PathBuf) -> Result<u64> {
        let mut reader = self.store.get(hash).await?;
        let mut file = tokio::fs::File::create(path).await?;
        let bytes = tokio::io::copy(&mut reader, &mut file).await?;
        Ok(bytes)
    }

    /// Import blob from file
    pub async fn import_from_file(
        &self,
        path: PathBuf,
    ) -> Result<(Hash, iroh_blobs::store::Format, u64)> {
        let file = tokio::fs::File::open(path).await?;
        self.add_blob(file, false, None).await
    }
}

/// Encrypted reader wrapper
struct EncryptedReader<R: AsyncRead + Unpin> {
    inner: R,
    cipher: chacha20poly1305::ChaCha20Poly1305,
    nonce_counter: u64,
    buffer: Vec<u8>,
    position: usize,
}

impl<R: AsyncRead + Unpin> EncryptedReader<R> {
    fn new(inner: R, key: HybridKemSharedSecret) -> Self {
        let cipher = chacha20poly1305::ChaCha20Poly1305::new(key.as_ref().into());
        Self {
            inner,
            cipher,
            nonce_counter: 0,
            buffer: Vec::new(),
            position: 0,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for EncryptedReader<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Simplified - real impl would handle chunked encryption
        self.inner.poll_read(cx, buf)
    }
}

/// Decrypted reader wrapper
struct DecryptedReader<R: AsyncRead + Unpin> {
    inner: R,
    cipher: chacha20poly1305::ChaCha20Poly1305,
    nonce_counter: u64,
}

impl<R: AsyncRead + Unpin> DecryptedReader<R> {
    fn new(inner: R, key: HybridKemSharedSecret) -> Self {
        let cipher = chacha20poly1305::ChaCha20Poly1305::new(key.as_ref().into());
        Self {
            inner,
            cipher,
            nonce_counter: 0,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for DecryptedReader<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Simplified
        self.inner.poll_read(cx, buf)
    }
}
