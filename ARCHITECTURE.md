# yzchat Architecture Specification

## Overview

yzchat is a **pure peer-to-peer messenger** with **quantum-resistant encryption**, **4K video calls**, **crystal-clear audio**, and **screen sharing** — all without any servers, domains, or hosting. Built entirely in **Rust** with a **Kotlin/Jetpack Compose** Android frontend.

### Core Principles

1. **Zero Infrastructure** — No servers, no central authority, no metadata collection
2. **Quantum Resistance** — Hybrid ML-KEM-768 + X25519 (KEM) and ML-DSA-65 + Ed25519 (signatures)
3. **Highest Quality Media** — AV1 4K60 video, Opus 510kbps audio, 4K30 screen share
4. **Telegram X UX** — Familiar, polished interface with Material 3 design
5. **Pure Rust Core** — All crypto, P2P, media logic in memory-safe Rust
6. **GitHub Actions CI/CD** — All builds on GitHub, zero local build required

---

## System Architecture

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

## Crate Structure

```
yzchat/
├── Cargo.toml                 # Workspace root
├── yzchat-crypto/             # Quantum-resistant cryptography
├── yzchat-p2p/                # P2P networking (Iroh-based)
├── yzchat-media/              # Media engine (WebRTC, AV1, Opus)
├── yzchat-ffi/                # UniFFI bindings for Kotlin
├── yzchat-android/            # Android app (Kotlin + Compose)
│   ├── app/                   # Main application module
│   └── gradle/                # Gradle wrapper
└── .github/workflows/         # CI/CD pipelines
```

---

## Cryptography (yzchat-crypto)

### Hybrid Key Encapsulation (KEM)
- **ML-KEM-768** (FIPS 203) + **X25519** (RFC 7748)
- Shared secret: `HKDF-SHA256(ML-KEM-SS || X25519-SS, salt, "yzchat/key-exchange/v1")`
- Key sizes: 1184 + 32 = 1216 bytes public, 2400 + 32 = 2432 bytes private

### Hybrid Signatures
- **ML-DSA-65** (FIPS 204) + **Ed25519** (RFC 8032)
- Both must verify for acceptance
- Signature size: 3309 + 64 = 3373 bytes

### Noise Protocol
- Pattern: `Noise_IK_ChaChaPoly_BLAKE3s_HybridKEM_HybridSig`
- Initiator knows responder's static key (IK)
- ChaCha20-Poly1305 AEAD, BLAKE3 hash

### Double Ratchet
- Forward secrecy + post-compromise security
- Symmetric ratchet per message, DH ratchet per key rotation
- Skipped message keys for out-of-order delivery (max 1000)

### Key Derivation
- HKDF-SHA256 with domain separation
- Domain tags: `key-exchange`, `message-encryption`, `file-encryption`, `call-signaling`, `identity-binding`, `group-handshake`

---

## P2P Networking (yzchat-p2p)

### Transport: Iroh (QUIC-based)
- **Noise_IK** handshake with Ed25519 static keys
- Multiplexed streams over single QUIC connection
- 0-RTT resumption for fast reconnection

### Discovery
| Method | Purpose | Scope |
|--------|---------|-------|
| **Kademlia DHT** | Global peer discovery | Internet-wide |
| **mDNS** | Local network discovery | LAN |
| **N0** | Relay-assisted discovery | NAT traversal |

### Protocols (ALPN)
| Protocol | Purpose |
|----------|---------|
| `yzchat/messaging/1` | Encrypted messages |
| `yzchat/voice/1` | Voice calls |
| `yzchat/video/1` | Video calls |
| `yzchat/screen/1` | Screen sharing |
| `yzchat/file/1` | File transfer |
| `yzchat/signaling/1` | WebRTC signaling |
| `yzchat/sync/1` | Document sync |
| `yzchat/presence/1` | Presence updates |
| `yzchat/typing/1` | Typing indicators |

### Components
- **NodeEndpoint** — Identity + QUIC listener + RPC server
- **Discovery** — DHT + mDNS + N0 with peer metadata
- **Router** — Connection pooling, message routing, protocol multiplexing
- **Signaling** — SDP offer/answer + ICE candidates over Iroh RPC
- **Blobs** — Large file transfer with streaming, encryption, resume
- **Docs** — Syncable data structures (contacts, messages, groups)

---

## Media Engine (yzchat-media)

### Video: AV1 (Primary) / VP9 / H.264 (Fallback)
| Resolution | Bitrate | Framerate | Codec |
|------------|---------|-----------|-------|
| 4K (3840×2160) | 25 Mbps | 60 fps | AV1 Main 10-bit |
| 1440p | 12 Mbps | 60 fps | AV1 |
| 1080p | 6 Mbps | 60 fps | AV1/VP9 |
| 720p | 3 Mbps | 30 fps | VP9/H.264 |

- **Hardware acceleration**: MediaCodec (Android), VideoToolbox (iOS), VAAPI/NVENC (Desktop)
- **Simulcast**: 3 layers (4K, 1080p, 720p) for adaptive bitrate
- **SVC**: 3 temporal layers for graceful degradation
- **Keyframe interval**: 1 second (60 frames)

### Audio: Opus
- **Sample rate**: 48 kHz
- **Channels**: Stereo (2)
- **Bitrate**: 510 kbps (max quality)
- **Frame duration**: 20 ms
- **Features**: DTX, FEC, complexity 10
- **Processing**: AEC (echo cancellation), NS (noise suppression), AGC (auto gain)

### Screen Share
- **Resolution**: Up to 4K (3840×2160)
- **Framerate**: 30 fps
- **Bitrate**: 50 Mbps (lossless option for text)
- **Codec**: AV1 with tiles (parallel encoding)
- **Downscale**: Configurable (1x, 2x)

### WebRTC Stack
- **DTLS-SRTP** for media encryption
- **ICE** with STUN/TURN (via Iroh relay)
- **Data channels** for signaling + file transfer
- **NACK/PLI/FEC** for reliability
- **Congestion control**: GCC (Google Congestion Control)

---

## Android App (yzchat-android)

### Architecture: MVVM + Clean Architecture
```
UI (Compose) → ViewModels → Repository → Data Sources
                    ↓
              yzchat-ffi (UniFFI)
                    ↓
              Rust Core (JNI)
```

### Key Modules
- **ui/** — Compose screens (Chats, Calls, Contacts, Settings, Chat, Call)
- **navigation/** — Navigation graph with type-safe routes
- **di/** — Hilt modules for dependency injection
- **data/** — Room database, DataStore, Repository implementations
- **ffi/** — JNI bridge to Rust core

### UI Design (Telegram X Inspired)
- **Dark theme default** (true black for OLED)
- **Primary color**: `#0088CC` (Telegram Blue)
- **Typography**: System font with custom scale
- **Shapes**: Rounded corners (4-24dp), full-round for avatars
- **Animations**: Shared element transitions, spring physics

### Screens
1. **Onboarding** — 6-page feature tour
2. **Welcome** — Get started with key features
3. **Main** — Bottom nav: Chats, Calls, Contacts, Settings
4. **Chat** — Message bubbles, media, voice messages, reply
5. **Call** — Full-screen video/voice with PiP, controls
6. **Settings** — Sectioned list with icons, destructive actions

---

## FFI Layer (yzchat-ffi)

### UniFFI Interface (yzchat.udl)
Generates:
- **Rust**: `libyzchat_ffi.so` (cdylib)
- **Kotlin**: `YzchatCore`, `CryptoManager`, `P2PManager`, `MediaManager`, `StorageManager`

### Key Interfaces
| Interface | Responsibilities |
|-----------|------------------|
| `YzchatCore` | Main entry point, high-level operations |
| `CryptoManager` | Identity, keys, encryption, signing |
| `P2PManager` | Discovery, connections, messaging, signaling, file transfer |
| `MediaManager` | Audio/video calls, screen share, devices, quality |
| `StorageManager` | Persistent storage (redb), backup/restore |

### Callbacks (Kotlin → Rust)
- `MessageEventCallback` — New messages
- `CallEventCallback` — Call state changes
- `ContactEventCallback` — Contact updates
- `ConnectionChangeCallback` — Network status
- `IncomingCallCallback` — Incoming call notifications
- `MediaStatsCallback` — Real-time call statistics

---

## CI/CD Pipeline (GitHub Actions)

### Workflows
1. **rust-check** — fmt, clippy, build, test (all targets)
2. **android-build** — Build Rust for Android (arm64/x86_64), generate UniFFI bindings, assemble APK
3. **security-audit** — cargo-audit, cargo-outdated
4. **docs** — cargo doc for all crates
5. **release** — Tag-based release with signed APKs
6. **nightly** — Scheduled nightly builds with nightly Rust

### Android Build Process
```yaml
# 1. Install Rust + Android targets
# 2. cargo ndk --target aarch64-linux-android --target x86_64-linux-android --platform 24 -- build --release --features android
# 3. Copy .so files to app/src/main/jniLibs/{arm64-v8a,x86_64}/
# 4. cargo install uniffi_bindgen && cargo run --bin uniffi-bindgen generate --library target/.../libyzchat_ffi.so --language kotlin --out-dir app/src/main/java
# 5. ./gradlew assembleDebug/Release
```

### Artifacts
- **Debug APK**: Every push/PR (30-day retention)
- **Release APK**: Tag pushes (90-day retention, signed)
- **Nightly APK**: Daily schedule (7-day retention)

---

## Security Model

### Threat Model
| Threat | Mitigation |
|--------|------------|
| Passive eavesdropping | Hybrid KEM (ML-KEM-768 + X25519) |
| Active MITM | Hybrid signatures (ML-DSA-65 + Ed25519), Noise IK |
| Quantum computer | NIST PQC standards (FIPS 203/204) |
| Key compromise | Double Ratchet (forward secrecy + PCS) |
| Metadata collection | Zero infrastructure, P2P only |
| Device compromise | Local encryption, secure enclave for keys |
| Downgrade attack | Protocol version negotiation, mandatory PQC |

### Key Lifecycle
1. **Identity Key** — Generated once, stored encrypted (Android Keystore / Secure Enclave)
2. **Encryption Keypair** — Generated once, used for key agreement
3. **Signed Pre-Key** — Rotated weekly, signed by identity
4. **One-Time Pre-Keys** — Generated in batches (100), consumed on use
5. **Session Keys** — Derived per conversation via X3DH + Double Ratchet

### Verification
- **Safety Numbers** — SHA-256 fingerprint of identity keys (60 chars, 5 groups)
- **QR Code** — Compact representation for in-person verification
- **Trust on First Use** — Optional, with verification prompts

---

## Data Storage

### Database: redb (embedded, ACID)
| Table | Purpose |
|-------|---------|
| `identity` | Current user's identity secret key |
| `encryption_keypair` | Long-term encryption keypair |
| `pre_keys` | One-time pre-keys (multimap by key_id) |
| `contacts` | Contact list with aliases, capabilities |
| `conversations` | Conversation metadata |
| `messages` | Encrypted messages (multimap by conversation_id) |
| `groups` | Group metadata |
| `group_members` | Group membership (multimap by group_id) |
| `settings` | User preferences |
| `key_bundles` | Cached peer key bundles |

### Backup/Restore
- **Encrypted export** — Age (X25519 + ChaCha20-Poly1305) with user password
- **Selective restore** — Choose data to restore
- **Cross-platform** — Compatible with desktop/iOS versions

---

## Deployment

### GitHub Repository Structure
```
yzchat/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml           # Main CI/CD
│   │   ├── nightly.yml      # Nightly builds
│   │   └── release.yml      # Release automation
│   └── dependabot.yml       # Dependency updates
├── yzchat-crypto/
├── yzchat-p2p/
├── yzchat-media/
├── yzchat-ffi/
├── yzchat-android/
│   ├── app/
│   ├── gradle/
│   ├── gradlew
│   ├── gradle.properties
│   └── build.gradle.kts
├── Cargo.toml
├── Cargo.lock
├── ARCHITECTURE.md
├── README.md
├── LICENSE (GPL-3.0-or-later)
└── .gitignore
```

### Release Process
1. Create tag: `git tag v1.0.0 && git push origin v1.0.0`
2. GitHub Actions builds release APKs (arm64 + x86_64)
3. Signs with keystore (secrets in GitHub)
4. Creates GitHub Release with APKs + SHA256 checksums
5. Auto-generates changelog from commits

### Distribution
- **GitHub Releases** — Primary distribution
- **F-Droid** — Optional, reproducible builds
- **Website** — Direct APK downloads with verification
- **No Play Store** — True to "zero infrastructure" principle

---

## Development Roadmap

### Phase 1: Foundation (Months 1-3)
- [x] Architecture specification
- [ ] Rust workspace with crypto + P2P crates
- [ ] Basic Iroh endpoint + DHT discovery
- [ ] Hybrid KEM + Signatures implementation
- [ ] UniFFI bindings generation
- [ ] Android project with Gradle + Compose

### Phase 2: Core Messaging (Months 3-6)
- [ ] X3DH key agreement + Double Ratchet
- [ ] Encrypted messaging protocol
- [ ] Contact management + key bundles
- [ ] Message storage + sync (iroh-docs)
- [ ] Basic UI: chats, contacts, settings

### Phase 3: Media (Months 6-9)
- [ ] WebRTC integration (webrtc-rs)
- [ ] AV1 encoding (rav1e) + decoding (dav1d)
- [ ] Opus audio pipeline
- [ ] Screen capture + encoding
- [ ] Call UI: incoming/outgoing, controls, PiP

### Phase 4: Advanced Features (Months 9-12)
- [ ] Groups (MLS-style ratchet tree)
- [ ] File transfer (iroh-blobs)
- [ ] Push notifications (UnifiedPush/WebPush)
- [ ] Multi-device sync
- [ ] Backup/restore
- [ ] Performance optimization

### Phase 5: Polish & Audit (Months 12-15)
- [ ] Security audit (internal + external)
- [ ] F-Droid reproducible builds
- [ ] Accessibility (TalkBack, dynamic text)
- [ ] Localization (i18n)
- [ ] Documentation + user guides

---

## Dependencies

### Rust (Core)
| Crate | Version | Purpose |
|-------|---------|---------|
| `iroh` | 0.43 | P2P networking |
| `pqcrypto-kyber` | 0.4 | ML-KEM-768 |
| `pqcrypto-dilithium` | 0.4 | ML-DSA-65 |
| `x25519-dalek` | 2.1 | X25519 ECDH |
| `ed25519-dalek` | 2.1 | Ed25519 signatures |
| `snow` | 0.9 | Noise protocol |
| `webrtc` | 0.12 | WebRTC |
| `gstreamer` | 0.21 | Media pipelines |
| `rav1e` | 0.7 | AV1 encoder |
| `dav1d` | 0.9 | AV1 decoder |
| `opus` | 0.5 | Opus codec |
| `uniffi` | 0.28 | FFI bindings |
| `redb` | 0.1 | Embedded database |
| `tokio` | 1.40 | Async runtime |

### Android
| Dependency | Version |
|------------|---------|
| Android Gradle Plugin | 8.5.0 |
| Kotlin | 2.0.20 |
| Compose BOM | 2024.06.00 |
| Hilt | 2.48 |
| Room | 2.6.1 |
| Navigation | 2.7.7 |
| Coil | 2.6.0 |
| Gradle | 8.5 |
| NDK | 27.0.12077973 |

---

## License

**GPL-3.0-or-later** — Free software, copyleft, ensures freedom for all users.

---

## Conclusion

This architecture provides a **complete, production-ready blueprint** for building yzchat. Every component is specified with:
- Exact crate dependencies and versions
- Protocol designs with security rationale
- Android UI matching Telegram X quality
- GitHub Actions CI/CD for zero-local-build deployment
- Clear milestones for team execution

The pure Rust core ensures memory safety and auditability. The UniFFI boundary provides type-safe Kotlin integration. Iroh provides battle-tested P2P networking. The result: a messenger that is **truly serverless, quantum-resistant, and highest quality** — built the right way.