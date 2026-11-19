//! yzchat-ffi: UniFFI bindings for Kotlin ↔ Rust

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]

use uniffi::Object;

mod crypto_manager;
mod p2p_manager;
mod media_manager;
mod storage_manager;
mod yzchat_core;
mod types;
mod callbacks;

pub use crypto_manager::CryptoManager;
pub use p2p_manager::P2PManager;
pub use media_manager::MediaManager;
pub use storage_manager::StorageManager;
pub use yzchat_core::YzchatCore;
pub use types::*;
pub use callbacks::*;

uniffi::setup_scaffolding!();