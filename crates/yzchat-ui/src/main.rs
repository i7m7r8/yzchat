//! Cross-platform Signal-like UI for yzchat (Slint, all-Rust).
//!
//! Stub entry point. The Slint `slint!` DSL triggers an edition warning under
//! CI's `-D warnings`, so the live UI is wired here once slint edition handling
//! is resolved. The crate compiles cleanly.

use yzchat_crypto::{hybrid_kem::HybridKem, Result};

fn generate_identity() -> Result<yzchat_crypto::hybrid_kem::HybridKemKeypair> {
    HybridKem::generate()
}

fn main() -> Result<()> {
    let _identity = generate_identity()?;
    Ok(())
}
