# Contributing to yzchat

Thank you for your interest in contributing to yzchat! This document outlines the process for contributing to this pure P2P messenger project.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before contributing.

## Getting Started

### Prerequisites

- **Rust 1.80+** with Android targets:
  ```bash
  rustup target add aarch64-linux-android x86_64-linux-android
  ```
- **Android SDK 35** + **NDK 27**
- **JDK 17** (Eclipse Temurin recommended)
- **Git** with LFS support

### Development Setup

1. **Fork and clone:**
   ```bash
   git clone https://github.com/your-username/yzchat.git
   cd yzchat
   ```

2. **Install development tools:**
   ```bash
   # Rust tools
   cargo install cargo-ndk cargo-audit cargo-outdated uniffi_bindgen

   # Android: Use Android Studio or command line
   # Ensure ANDROID_HOME is set
   ```

3. **Build Rust core for Android:**
   ```bash
   cd yzchat-ffi
   cargo ndk \
     --target aarch64-linux-android \
     --target x86_64-linux-android \
     --platform 24 \
     -- build --release --features android
   ```

4. **Generate Kotlin bindings:**
   ```bash
   cargo run --bin uniffi-bindgen generate \
     --library target/aarch64-linux-android/release/libyzchat_ffi.so \
     --language kotlin \
     --out-dir ../yzchat-android/app/src/main/java
   ```

5. **Build Android app:**
   ```bash
   cd ../yzchat-android
   ./gradlew assembleDebug
   ```

## Contribution Workflow

### 1. Find or Create an Issue

- Check [existing issues](https://github.com/yzchat/yzchat/issues)
- Create a new issue if needed, using appropriate template
- Discuss the approach before implementing large changes

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

Use descriptive branch names:
- `feature/av1-hardware-acceleration`
- `fix/double-ratchet-oom`
- `docs/update-architecture`

### 3. Make Changes

Follow the coding standards:

#### Rust Code Style
```bash
# Format
cargo fmt --all

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Test
cargo test --workspace --all-targets --all-features
```

#### Kotlin/Android Code Style
```bash
# Lint
./gradlew lint

# Format (via ktlint)
./gradlew ktlintFormat
```

### 4. Commit Guidelines

Use conventional commits:

```
type(scope): short description

Longer description if needed.

Fixes #123
```

Types:
- `feat` — New feature
- `fix` — Bug fix
- `docs` — Documentation only
- `style` — Formatting, no code change
- `refactor` — Code restructuring
- `test` — Adding tests
- `chore` — Maintenance, build changes
- `perf` — Performance improvement
- `security` — Security fix

Example:
```
feat(crypto): add ML-KEM-1024 support for higher security

Implements hybrid KEM with ML-KEM-1024 + X25519 for
post-quantum security level 5.

Fixes #456
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then open a Pull Request on GitHub with:
- Clear title and description
- Link to related issue
- Screenshots for UI changes
- Test results if applicable

## Code Standards

### Rust

- **Edition**: 2021
- **Minimum MSRV**: 1.80
- **Formatting**: `rustfmt` (default config)
- **Linting**: `clippy` with `-D warnings`
- **Documentation**: All public items must have docs
- **Safety**: `forbid(unsafe_code)` — no unsafe allowed
- **Error handling**: `anyhow` for applications, `thiserror` for libraries
- **Async**: `tokio` with structured concurrency

### Kotlin

- **Version**: 2.0.20
- **Style**: [Kotlin Coding Conventions](https://kotlinlang.org/docs/coding-conventions.html)
- **Formatting**: `ktlint` with official rules
- **Coroutines**: Structured concurrency, `SupervisorJob` for UI
- **Flow**: `StateFlow`/`SharedFlow` for state, `Flow` for streams
- **Compose**: Material 3, stateless composables, hoisted state

### Architecture Principles

1. **Pure Rust Core** — All crypto, P2P, media logic in Rust
2. **FFI Boundary** — UniFFI for type-safe Kotlin interop
3. **No Unsafe** — Zero `unsafe` in Rust codebase
4. **Zero Infrastructure** — No server dependencies anywhere
5. **Security First** — Quantum-resistant by default
6. **Memory Safety** — Leverage Rust's ownership model

## Testing

### Rust Tests
```bash
# Unit tests
cargo test --workspace --lib

# Integration tests
cargo test --workspace --test '*'

# Specific crate
cargo test -p yzchat-crypto
```

### Android Tests
```bash
# Unit tests
./gradlew testDebugUnitTest

# Instrumented tests (requires emulator/device)
./gradlew connectedDebugAndroidTest
```

### Media Tests
```bash
# Test encoding/decoding
cargo test -p yzchat-media -- --test-threads=1
```

## Security Considerations

### Reporting Vulnerabilities

**Do not open public issues for security vulnerabilities.**

Email: `security@yzchat.org`

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Cryptography Changes

Any changes to cryptographic primitives require:
1. Security review by 2+ maintainers
2. Formal verification where possible
3. Test vectors against known implementations
4. Documentation of threat model changes

### FFI Boundary

- All FFI types must be `Send + Sync`
- No raw pointers across FFI
- Use UniFFI records/enums for complex types
- Validate all inputs from Kotlin side

## Documentation

### Required Documentation

- **Public APIs**: Rustdoc for all `pub` items
- **Architecture**: Update `ARCHITECTURE.md` for structural changes
- **README**: Update feature lists, badges
- **Changelog**: Add entry for user-facing changes

### Documentation Commands
```bash
# Build Rust docs
cargo doc --workspace --all-features --no-deps --document-private-items

# View docs
open target/doc/yzchat_crypto/index.html
```

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):
- **Major**: Breaking API changes, protocol changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, security patches

### Release Checklist

1. Update version in `Cargo.toml` (workspace) and `gradle.properties`
2. Update `CHANGELOG.md`
3. Create release branch: `git checkout -b release/v1.x.x`
4. Run full test suite
5. Create tag: `git tag v1.x.x && git push origin v1.x.x`
6. GitHub Actions builds and publishes release

## Community

### Communication Channels

- **GitHub Discussions**: Design discussions, questions
- **GitHub Issues**: Bug reports, feature requests
- **Matrix**: `#yzchat:matrix.org` (real-time chat)

### Maintainers

- @yzchat-core — Core team
- @yzchat-security — Security team
- @yzchat-android — Android team

## License

By contributing, you agree that your contributions will be licensed under **GPL-3.0-or-later** (same as the project).

---

## Quick Reference

| Task | Command |
|------|---------|
| Format Rust | `cargo fmt --all` |
| Lint Rust | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Test Rust | `cargo test --workspace --all-targets --all-features` |
| Build Android Debug | `cd yzchat-android && ./gradlew assembleDebug` |
| Build Android Release | `cd yzchat-android && ./gradlew assembleRelease` |
| Lint Android | `cd yzchat-android && ./gradlew lint` |
| Generate Bindings | `cd yzchat-ffi && cargo run --bin uniffi-bindgen generate ...` |
| Security Audit | `cargo audit --workspace` |
| Check Outdated | `cargo outdated --workspace --root-deps-only` |

---

**Thank you for contributing to yzchat!** 🚀

Together we're building a truly private, quantum-resistant, serverless messenger for everyone.