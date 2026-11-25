//! yzchat-ffi: UniFFI bindings for Kotlin ↔ Rust

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]

use uniffi::Object;

mod callbacks;
mod crypto_manager;
mod media_manager;
mod p2p_manager;
mod storage_manager;
mod types;
mod yzchat_core;

pub use callbacks::*;
pub use crypto_manager::CryptoManager;
pub use media_manager::MediaManager;
pub use p2p_manager::P2PManager;
pub use storage_manager::StorageManager;
pub use types::*;
pub use yzchat_core::YzchatCore;

uniffi::setup_scaffolding!();
