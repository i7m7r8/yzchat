//! Cross-platform Signal-like UI for yzchat (Slint, all-Rust).
//!
//! Stub entry point. The Slint `slint!` DSL triggers an edition warning under
//! CI's `-D warnings`, so the live UI is wired here once slint edition handling
//! is resolved. The crate compiles cleanly and exposes the backend types.

use yzchat_crypto::{hybrid_kem::HybridKem, CryptoError, Result};
use yzchat_media::MediaManager;

struct Backend {
    media: MediaManager,
}

impl Backend {
    fn new() -> Result<Self> {
        Ok(Self { media: MediaManager::new() })
    }

    fn generate_identity(&self) -> Result<yzchat_crypto::hybrid_kem::HybridKemKeypair> {
        HybridKem::generate()
    }
}

fn main() -> Result<()> {
    let backend = Backend::new()?;
    let _identity = backend.generate_identity()?;
    Ok(())
}
