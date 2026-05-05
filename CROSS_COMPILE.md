# Cross-Compilation & GitHub Actions
### How artifact-cleaner ships to macOS, Linux, and Windows

---

## What is cross-compilation?

**Native compilation** = compile ON the target machine FOR that machine.
**Cross-compilation** = compile ON machine A FOR machine B.

```
Your Mac (aarch64-apple-darwin)
  │
  ├── compiles FOR → aarch64-apple-darwin   ← native (same machine)
  ├── compiles FOR → x86_64-apple-darwin    ← cross (Apple SDK handles this)
  ├── compiles FOR → x86_64-linux-musl      ← cross (needs musl linker)
  └── compiles FOR → x86_64-windows-msvc   ← cannot — needs Windows SDK
```

Some targets you can cross-compile freely. Others require the target OS's toolchain.
This is why the GitHub Actions pipeline uses three different runners.

---

## Target Triples — the Rust naming system

Every compilation target in Rust is identified by a **target triple**:

```
aarch64   -   apple   -   darwin
  │               │           │
  CPU arch    Vendor      OS / ABI

x86_64   -   unknown   -   linux   -   musl
  │               │           │          │
  CPU arch    Vendor        OS       C library
```

### CPU architectures

| Name | Means | Common hardware |
|------|-------|----------------|
| `x86_64` | 64-bit Intel/AMD | Most laptops/desktops/servers |
| `aarch64` | 64-bit ARM | Apple Silicon, AWS Graviton, Raspberry Pi 4 |
| `i686` | 32-bit Intel | Old machines (rare now) |
| `armv7` | 32-bit ARM | Raspberry Pi 2/3, IoT devices |

### Operating systems / ABIs

| Target suffix | OS | C library | Notes |
|--------------|----|-----------|-------|
| `apple-darwin` | macOS | libSystem (Apple) | Requires macOS runner |
| `unknown-linux-gnu` | Linux | glibc | Dynamic — version-tied to build host |
| `unknown-linux-musl` | Linux | musl | **Fully static — runs on any Linux** |
| `pc-windows-msvc` | Windows | MSVC CRT | Requires Windows runner + MSVC |
| `pc-windows-gnu` | Windows | MinGW | Can cross-compile from Linux |

---

## Why each OS needs its own runner

### macOS — Apple's closed toolchain

Apple's linker (`ld64`) and SDK are **only available on macOS**.
They cannot be legally redistributed or run on other platforms.

```
macOS runner (M1)
├── Apple Clang compiler  ← installed by default
├── Apple ld64 linker     ← macOS only
├── macOS SDK headers     ← needed for system calls
└── Xcode command-line tools
```

The macOS runner can compile BOTH `aarch64-apple-darwin` (native M1) and
`x86_64-apple-darwin` (Intel) because Apple's toolchain supports both — this
is how Universal binaries (`lipo`) work.

```yaml
# In release.yml:
runs-on: macos-14              # Apple Silicon M1 runner
targets: aarch64-apple-darwin  # native
         x86_64-apple-darwin   # cross — Apple's toolchain handles it
```

### Linux — glibc vs musl

The Linux runner uses **musl** instead of glibc. Here's why:

```
glibc binary compiled on Ubuntu 24:
  └── dynamically links to /lib/x86_64-linux-gnu/libc.so.6 (version 2.39)
      └── FAILS on Ubuntu 20 (has glibc 2.31) — version mismatch!
      └── FAILS on Alpine Linux (uses musl, not glibc)
      └── FAILS on scratch Docker containers

musl binary:
  └── statically linked — contains everything it needs
      └── runs on Ubuntu 20, Ubuntu 24, Alpine, Debian, RHEL, any Linux
      └── runs in scratch Docker containers (zero base image)
```

```yaml
# In release.yml:
runs-on: ubuntu-22.04
target: x86_64-unknown-linux-musl    # static binary — runs everywhere
```

```bash
# Verify a binary is static:
file target/x86_64-unknown-linux-musl/release/artifact-cleaner
# → ELF 64-bit LSB executable, statically linked   ← what we want

file target/x86_64-unknown-linux-gnu/release/artifact-cleaner
# → ELF 64-bit LSB executable, dynamically linked  ← avoid for distribution
```

### ARM64 Linux — why we need `cross`

GitHub has no native ARM64 Linux runner (as of 2025).
To compile `aarch64-unknown-linux-musl` from an x86_64 runner, we use `cross`.

```
cross = cargo build wrapper
      + Docker container with the right toolchain
      + QEMU to emulate ARM64 if needed for build scripts

x86_64 Ubuntu runner
  └── runs Docker container (ghcr.io/cross-rs/aarch64-unknown-linux-musl)
        └── contains ARM64 musl-gcc cross-compiler
              └── produces aarch64 ELF binary
```

```yaml
# In release.yml:
- name: Install cross
  run: cargo install cross --git https://github.com/cross-rs/cross

- name: Build
  run: cross build --release --target aarch64-unknown-linux-musl
  # ^^^^^ drop-in replacement for `cargo build`
```

### Windows — MSVC is non-negotiable

Windows system APIs (Win32, WinRT) require Microsoft's headers and linker.

```
Windows runner
├── Visual Studio Build Tools  ← pre-installed on GitHub Windows runners
├── MSVC linker (link.exe)     ← needed for windows-msvc targets
├── Windows SDK headers        ← needed for windows system calls
└── Windows CRT (runtime)      ← C runtime library for Windows
```

```yaml
# In release.yml:
runs-on: windows-latest
target: x86_64-pc-windows-msvc
# Output: artifact-cleaner.exe — 64-bit Windows executable
```

---

## The 6 release binaries — what each one is for

| Binary | Who needs it |
|--------|-------------|
| `aarch64-apple-darwin.tar.gz` | Mac with M1/M2/M3 chip (2020+) |
| `x86_64-apple-darwin.tar.gz` | Mac with Intel chip (pre-2020) |
| `x86_64-unknown-linux-musl.tar.gz` | Linux servers, CI runners, Docker (x64) |
| `aarch64-unknown-linux-musl.tar.gz` | AWS Graviton, Raspberry Pi 4, Linux ARM |
| `x86_64-pc-windows-msvc.zip` | Windows PC/laptop (x64 — most Windows machines) |
| `aarch64-pc-windows-msvc.zip` | Surface Pro X, Snapdragon Windows laptops |

---

## How to trigger a release

```bash
# 1. Update version in Cargo.toml
#    version = "1.0.0"

# 2. Commit the change
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 1.0.0"

# 3. Create and push a tag — this triggers the workflow
git tag v1.0.0
git push origin main --tags

# GitHub Actions fires within seconds.
# All 6 binaries appear on the Releases page in ~5 minutes.
```

---

## How to build locally for your platform

```bash
# Build for YOUR current machine (fastest)
cargo build --release
./target/release/artifact-cleaner --help

# Install into your PATH
cargo install --path .
artifact-cleaner --help
```

## How to cross-compile locally (macOS → Linux)

```bash
# Install the target
rustup target add x86_64-unknown-linux-musl

# Install musl cross-compiler (macOS via brew)
brew install FiloSottile/musl-cross/musl-cross

# Set the linker for this target
# Add to .cargo/config.toml in the project:
# [target.x86_64-unknown-linux-musl]
# linker = "x86_64-linux-musl-gcc"

# Build
cargo build --release --target x86_64-unknown-linux-musl
```

---

## CI workflow (ci.yml) — runs on every PR

```
Every push/PR to main
  │
  ├── check    → cargo check      (type errors, ~10s)
  ├── clippy   → cargo clippy     (linting, ~15s)
  ├── fmt      → cargo fmt --check (formatting, ~5s)
  ├── test     → cargo test       (unit tests, ~15s)
  └── build    → cargo build      (debug build on macOS + Linux + Windows, ~45s each)
```

The CI catches problems early. The release workflow only runs when you push a tag.

---

## Release workflow (release.yml) — runs on git tags

```
git push tag v1.0.0
  │
  ├── build-macos    (macos-14 runner)
  │     ├── aarch64-apple-darwin   → .tar.gz → GitHub Release
  │     └── x86_64-apple-darwin    → .tar.gz → GitHub Release
  │
  ├── build-linux    (ubuntu-22.04 runner)
  │     ├── x86_64-unknown-linux-musl   → .tar.gz → GitHub Release
  │     └── aarch64-unknown-linux-musl  → .tar.gz → GitHub Release (via cross)
  │
  ├── build-windows  (windows-latest runner)
  │     ├── x86_64-pc-windows-msvc   → .zip → GitHub Release
  │     └── aarch64-pc-windows-msvc  → .zip → GitHub Release
  │
  └── release-complete (gate job — confirms all succeeded)
```

All jobs run **in parallel** — total time ≈ longest single job (~5 min).

---

## Cargo.toml for releases — optimisation settings

Add this to `Cargo.toml` to produce smaller, faster release binaries:

```toml
[profile.release]
opt-level = 3        # maximum optimisation (default is already 3)
lto = true           # link-time optimisation — cross-module inlining
codegen-units = 1    # single codegen unit — slower compile, smaller binary
strip = true         # strip debug symbols from binary (reduces size 3-5x)
panic = "abort"      # smaller binary — no stack unwinding on panic
```

With these settings, the release binary on macOS typically shrinks from ~3MB to ~600KB.

---

## Useful commands

```bash
# List all targets Rust supports
rustup target list

# Add a new target to your local toolchain
rustup target add x86_64-unknown-linux-musl

# Show what targets are installed
rustup target list --installed

# Check what a binary links against (Linux)
ldd target/release/artifact-cleaner

# Check what a binary links against (macOS)
otool -L target/release/artifact-cleaner

# Confirm a Linux musl binary is static
file target/x86_64-unknown-linux-musl/release/artifact-cleaner
# → statically linked ✓
```
