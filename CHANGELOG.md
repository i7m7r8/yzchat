# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with Rust workspace
- yzchat-crypto: Hybrid ML-KEM-768 + X25519 KEM and ML-DSA-65 + Ed25519 signatures
- yzchat-p2p: Iroh-based P2P networking with DHT, mDNS, N0 discovery
- yzchat-media: WebRTC engine with AV1, Opus, GStreamer pipelines
- yzchat-ffi: UniFFI bindings for Kotlin ↔ Rust interop
- yzchat-android: Jetpack Compose UI with Telegram X inspired design
- GitHub Actions CI/CD pipeline for Android APK builds
- Comprehensive architecture documentation

### Security
- Quantum-resistant cryptography (NIST FIPS 203/204)
- Double Ratchet for forward secrecy and post-compromise security
- Noise_IK protocol for authenticated key exchange

## [1.0.0] - TBD

### Added
- First stable release
- End-to-end encrypted messaging
- 4K video calls with AV1
- Crystal-clear audio with Opus 510kbps
- Screen sharing up to 4K30
- File transfer with streaming
- Groups up to 200,000 members
- Contact verification with safety numbers
- Local-first encrypted storage

---

## Release Checklist

For each release:
- [ ] Update version in Cargo.toml (workspace) and gradle.properties
- [ ] Update CHANGELOG.md with release notes
- [ ] Run full test suite: `cargo test --workspace && ./gradlew test`
- [ ] Run security audit: `cargo audit --workspace`
- [ ] Create release branch: `git checkout -b release/vX.Y.Z`
- [ ] Create tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
- [ ] Verify GitHub Actions builds release APKs
- [ ] Verify release artifacts on GitHub Releases page
- [ ] Announce release

---

## Version History Template

### [X.Y.Z] - YYYY-MM-DD

#### Added
- New features

#### Changed
- Changes in existing functionality

#### Deprecated
- Soon-to-be removed features

#### Removed
- Removed features

#### Fixed
- Bug fixes

#### Security
- Security improvements