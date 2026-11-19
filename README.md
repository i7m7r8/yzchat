# yzchat — Pure P2P Messenger

> **Quantum-resistant • Zero infrastructure • 4K video • Crystal audio • Telegram X UX**

[![Build Status](https://github.com/yzchat/yzchat/workflows/CI/badge.svg)](https://github.com/yzchat/yzchat/actions)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0-or-later-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://rust-lang.org)
[![Android](https://img.shields.io/badge/Android-API%2024+-green.svg)](https://android.com)

---

## Features

### 🔐 Quantum-Resistant Encryption
- **Hybrid KEM**: ML-KEM-768 (FIPS 203) + X25519 (RFC 7748)
- **Hybrid Signatures**: ML-DSA-65 (FIPS 204) + Ed25519 (RFC 8032)
- **Double Ratchet**: Forward secrecy + post-compromise security
- **Noise Protocol**: `Noise_IK_ChaChaPoly_BLAKE3s_HybridKEM_HybridSig`

### 🌐 True Peer-to-Peer (Zero Infrastructure)
- **No servers, no domains, no hosting**
- **Iroh QUIC transport** with Noise_IK handshake
- **Kademlia DHT** + **mDNS** + **N0** discovery
- **NAT traversal** via relay-assisted connections

### 📹 Highest Quality Media
| Feature | Specification |
|---------|---------------|
| **Video** | AV1 4K@60fps, 10-bit HDR, simulcast (3 layers) |
| **Audio** | Opus 48kHz stereo, 510 kbps, DTX + FEC |
| **Screen Share** | 4K@30fps, 50 Mbps, lossless option |
| **Codecs** | AV1 (primary), VP9, H.264 fallback |
| **Latency** | <100ms end-to-end (low latency mode) |

### 💬 Telegram X Experience
- Dark theme default (true black for OLED)
- Fluid animations, spring physics
- Message bubbles, reactions, replies
- Voice messages, file sharing
- Groups up to 200,000 members
- Channels, bots, polls

### 🛡️ Security First
- **Open source** (GPL-3.0-or-later)
- **Memory-safe Rust** core
- **Local-first storage** (redb, encrypted)
- **Safety numbers** for contact verification
- **No metadata collection** — ever

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Android App (Kotlin)                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │   UI Layer  │ │  ViewModels │ │ Repository  │ │  DI/Hilt  │ │
│  │ (Compose)   │ │  (State)    │ │  (Data)     │ │           │ │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └─────┬─────┘ │
│         │               │               │             │       │
│         ▼               ▼               ▼             ▼       │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              yzchat-ffi (UniFFI Bindings)               │  │
│  │  CryptoManager │ P2PManager │ MediaManager │ Storage    │  │
│  └───────────────────────────┬──────────────────────────────┘  │
└────────────────────────────────┼────────────────────────────────┘
                                 │ JNI / FFI
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Rust Core (Cargo Workspace)                │
│  ┌──────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ │
│  │ yzchat-crypto│ │ yzchat-p2p │ │yzchat-media│ │ yzchat-ffi │ │
│  │              │ │            │ │            │ │            │ │
│  │ • Hybrid KEM │ │ • Iroh     │ │ • WebRTC   │ │ • UniFFI   │ │
│  │ • Hybrid Sig │ │ • DHT/MDNS │ │ • AV1/Opus │ │ • Kotlin   │ │
│  │ • Noise IK   │ │ • WebRTC   │ │ • GStreamer│ │   bindings │ │
│  │ • Double     │ │ • Blobs    │ │ • Screen   │ │            │ │
│  │   Ratchet    │ │ • Docs     │ │   Share    │ │            │ │
│  └──────────────┘ └────────────┘ └────────────┘ └────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## Quick Start (Development)

### Prerequisites
- **Rust 1.80+** with `aarch64-linux-android`, `x86_64-linux-android` targets
- **Android SDK 35** + **NDK 27**
- **JDK 17** (Temurin)
- **Gradle 8.5** (via wrapper)

### Build on GitHub (Recommended)
All builds run on GitHub Actions — **no local build required**.

1. Fork this repository
2. Push to your fork
3. GitHub Actions builds APKs automatically
4. Download from **Actions → Artifacts** or **Releases**

### Local Development (Optional)
```bash
# Clone
git clone https://github.com/yzchat/yzchat.git
cd yzchat

# Build Rust core for Android
cd yzchat-ffi
cargo ndk --target aarch64-linux-android --target x86_64-linux-android --platform 24 -- build --release --features android

# Generate Kotlin bindings
cargo install uniffi_bindgen --locked
cargo run --bin uniffi-bindgen generate --library target/aarch64-linux-android/release/libyzchat_ffi.so --language kotlin --out-dir ../yzchat-android/app/src/main/java

# Build Android app
cd ../yzchat-android
./gradlew assembleDebug
```

---

## Project Structure

```
yzchat/
├── .github/workflows/          # CI/CD pipelines
│   ├── ci.yml                  # Main CI (Rust + Android + Security)
│   └── nightly.yml             # Nightly builds
├── yzchat-crypto/              # Quantum-resistant cryptography
├── yzchat-p2p/                 # P2P networking (Iroh)
├── yzchat-media/               # Media engine (WebRTC, AV1, Opus)
├── yzchat-ffi/                 # UniFFI bindings (Rust ↔ Kotlin)
├── yzchat-android/             # Android app
│   ├── app/                    # Main module
│   └── gradle/                 # Gradle wrapper
├── Cargo.toml                  # Rust workspace
├── ARCHITECTURE.md             # Full architecture spec
├── LICENSE                     # GPL-3.0-or-later
└── README.md                   # This file
```

---

## CI/CD Pipeline

| Workflow | Trigger | Artifacts |
|----------|---------|-----------|
| **rust-check** | Push/PR | — |
| **android-build** | Push/PR | Debug APK (30 days) |
| **security-audit** | Push/PR | — |
| **docs** | Push | Rust docs (30 days) |
| **release** | Tag `v*` | Signed Release APKs (90 days) |
| **nightly** | Daily schedule | Nightly APK (7 days) |

### Release Process
```bash
git tag v1.0.0
git push origin v1.0.0
# GitHub Actions builds, signs, and creates Release
```

---

## Security

### Threat Model & Mitigations
| Threat | Mitigation |
|--------|------------|
| Passive eavesdropping | Hybrid KEM (ML-KEM-768 + X25519) |
| Active MITM | Hybrid signatures + Noise IK |
| Quantum computer | NIST PQC (FIPS 203/204) |
| Key compromise | Double Ratchet (FS + PCS) |
| Metadata collection | Zero infrastructure, P2P only |
| Device compromise | Android Keystore / Secure Enclave |

### Verification
- **Safety Numbers**: 60-char fingerprint (5 groups of 12)
- **QR Codes**: Compact in-person verification
- **Trust on First Use**: Optional with verification prompts

---

## License

**GPL-3.0-or-later** — Free software, copyleft, ensures freedom for all users.

See [LICENSE](LICENSE) for full text.

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt && cargo clippy` and `./gradlew lint`
5. Submit a PR

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## Community

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, design
- **Matrix**: `#yzchat:matrix.org` (coming soon)

---

## Acknowledgments

- **Iroh** — Pure Rust P2P networking
- **pqcrypto** — Post-quantum cryptography
- **webrtc-rs** — WebRTC in Rust
- **rav1e/dav1d** — AV1 codec
- **UniFFI** — Rust ↔ Kotlin FFI
- **Jetpack Compose** — Modern Android UI

---

**Built with ❤️ in Rust** — For privacy, for freedom, for everyone.