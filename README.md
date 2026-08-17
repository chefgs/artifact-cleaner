# Artifact Cleaner

[![CI](https://github.com/chefgs/artifact-cleaner/actions/workflows/ci.yml/badge.svg)](https://github.com/chefgs/artifact-cleaner/actions/workflows/ci.yml)
[![Release](https://github.com/chefgs/artifact-cleaner/actions/workflows/release.yml/badge.svg)](https://github.com/chefgs/artifact-cleaner/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Contributions welcome](https://img.shields.io/badge/contributions-welcome-brightgreen.svg)](CONTRIBUTING.md)

A Rust-based CLI tool for scanning and cleaning stale project artifacts. By default it targets `node_modules`, `.next`, `dist`, `build`, and `.terraform`; use `--types` to add other directory names such as `target`.

This project is also used as a practical Rust learning project for Go, Python, and Java developers.

Built in Rust. Single binary. No dependencies.

## Platform support

The project began on macOS, but the workspace cleanup commands are supported on macOS, Linux, and Windows. This is why releases include native installers and binaries for all three operating systems.

| Command | macOS | Linux | Windows |
|---|---:|---:|---:|
| `scan` — find and remove stale project artifacts | Yes | Yes | Yes |
| `size` — report artifact directory sizes | Yes | Yes | Yes |
| `mac-lib` — inspect orphaned `~/Library` app data | Yes | No | No |

`scan` and `size` use portable filesystem APIs and match directory names inside the workspace supplied by the user, so their cleanup behavior is not tied to macOS. `mac-lib` deliberately uses macOS Library locations and macOS tools; on Linux and Windows it is present in the CLI for consistency but exits without scanning or deleting anything.

Windows and Linux currently do not have equivalents of `mac-lib` for system or user cache cleanup. Their installers are for the cross-platform project-artifact cleanup commands above.

![Artifact Cleaner CLI cleaning development artifacts and reclaiming storage](assets/repo-image.png)

## Recent updates

- Added `afc size` for checking current project artifact sizes like `node_modules` and `.next` without stale filtering.
- Added `afc mac-lib` for scanning `~/Library/Caches`, `~/Library/Containers`, and `~/Library/Group Containers` for orphaned app and CLI data on macOS.
- Added release install scripts for macOS, Linux, and Windows with automatic platform detection and SHA256 verification.
- Kept CLI version output tied to Cargo package metadata so release binaries and source builds report the same version.

## Safe by default

The tool always shows what it found before deleting anything. By default, destructive cleanup requires an interactive confirmation prompt, and `--dry-run` previews what would be deleted without removing files.

For workspace scans, deletion without a prompt requires the explicit `--yes` flag. For `mac-lib`, the orphaned list is always shown and deletion still requires interactive confirmation.

## Install

### One-liner (recommended)

The install scripts detect your OS and CPU architecture, download the matching release artifact, verify SHA256 checksums, install the binary into your `PATH`, and verify the binary runs.

**macOS and Linux**:
```bash
curl -fsSL https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.sh | bash
```

Install a specific release:
```bash
VERSION=v0.9.0 curl -fsSL https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.sh | bash
```

Install to a custom directory:
```bash
INSTALL_DIR="$HOME/.local/bin" curl -fsSL https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.sh | bash
```

**Windows** (PowerShell):
```powershell
irm https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.ps1 | iex
```

Install a specific release:
```powershell
$env:VERSION = "v0.9.0"
irm https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.ps1 | iex
```

> Full install guide (manual steps, PATH setup, troubleshooting, SHA256 explanation):
> → **[INSTALL.md](./INSTALL.md)**

### Manual quick install

**macOS (Apple Silicon — M1/M2/M3):**
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-aarch64-apple-darwin.tar.gz
tar -xzf artifact-cleaner.tar.gz
sudo mv artifact-cleaner-*/artifact-cleaner /usr/local/bin/
# First run: xattr -dr com.apple.quarantine /usr/local/bin/artifact-cleaner
```

**macOS (Intel):**
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-x86_64-apple-darwin.tar.gz
tar -xzf artifact-cleaner.tar.gz
sudo mv artifact-cleaner-*/artifact-cleaner /usr/local/bin/
```

**Linux (x64 — static binary):**
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-x86_64-unknown-linux-musl.tar.gz
tar -xzf artifact-cleaner.tar.gz
sudo mv artifact-cleaner-*/artifact-cleaner /usr/local/bin/
```

**Linux (ARM64 — static binary):**
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-aarch64-unknown-linux-musl.tar.gz
tar -xzf artifact-cleaner.tar.gz
sudo mv artifact-cleaner-*/artifact-cleaner /usr/local/bin/
```

**Windows (x64 — PowerShell):**
```powershell
Invoke-WebRequest `
  -Uri "https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-x86_64-pc-windows-msvc.zip" `
  -OutFile "artifact-cleaner.zip"
Expand-Archive -Path artifact-cleaner.zip -DestinationPath .
# Move artifact-cleaner.exe to a folder in your PATH — see INSTALL.md
```

**Windows (ARM64 — PowerShell):**
```powershell
Invoke-WebRequest `
  -Uri "https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-aarch64-pc-windows-msvc.zip" `
  -OutFile "artifact-cleaner.zip"
Expand-Archive -Path artifact-cleaner.zip -DestinationPath .
# Move artifact-cleaner.exe to a folder in your PATH — see INSTALL.md
```

**Build from source (all platforms):**
```bash
cargo install --path .
```

Requires a current Rust toolchain with Cargo. The CLI version is sourced from `Cargo.toml`, so `artifact-cleaner --version` matches the package version for local builds too.

### Verify the install

```bash
artifact-cleaner --version
afc --version
artifact-cleaner --help
```

## Build locally

Use a debug build while developing:

```bash
cargo build
cargo run --bin afc -- --help
./target/debug/afc --help
```

Build an optimized release binary when you want the compact distributable build:

```bash
cargo build --release
./target/release/afc --help
./target/release/artifact-cleaner --help
```

`cargo build` compiles faster and is better for local iteration. `cargo build --release` applies the release profile in `Cargo.toml`, so the binary is smaller and faster. For this project, Rust ships as a compact native binary with no separate runtime dependency.

**Not sure which binary to pick?** See the [platform guide in INSTALL.md](./INSTALL.md#before-you-start--find-the-right-binary).

## Usage

Both `artifact-cleaner` (full name) and `afc` (short alias) are installed and identical.

### Scan workspace for stale build artifacts

`scan` checks projects whose top-level directory has not been modified within the selected age threshold. It finds matching **directories**, not individual files. The default directory names are `node_modules`, `.next`, `dist`, `build`, and `.terraform`.

```bash
# Scan current directory, 2-month threshold (default)
afc scan

# Scan a specific workspace
afc scan ~/Documents/github

# Use a 3-month threshold
afc scan ~/Documents/github --months 3

# Dry run — preview without deleting
afc scan ~/Documents/github --dry-run

# Skip confirmation prompt (for scripts/CI)
afc scan ~/Documents/github --yes

# Target specific artifact types only
afc scan ~/Documents/github --types node_modules,.next

# Include Rust, Python, Java, and test/build output directories
afc scan ~/Documents/github --types target,__pycache__,.gradle,coverage

# Preview a more conservative cleanup: only projects stale for 6 months
afc scan ~/Documents/github --months 6 --types target,coverage --dry-run
```

`--types` accepts comma-separated directory names. It does not match individual files, and `scan` does not currently offer a minimum-size filter for workspace artifacts. Use `--dry-run` before `--yes` when adding a new directory type.

### Check current project artifact sizes

```bash
# Size matching artifact folders directly under the current directory
afc size

# Size a specific project directory
afc size ~/Documents/github/my-next-app

# Check only JavaScript-heavy artifacts
afc size --types node_modules,.next

# Check Rust, Java, and test artifacts in one project
afc size . --types target,.gradle,coverage
```

Like `scan`, `size` reports directories only. It cannot currently list files larger than a chosen size.

### Scan macOS Library for orphaned data (macOS only)

`mac-lib` only auto-deletes high-confidence orphaned app containers. Shared containers and ambiguous named caches are shown as cautionary items and are not deleted automatically.

```bash
# Scan Caches, Containers, and Group Containers (default: items > 100 MB)
afc mac-lib

# Preview without deleting
afc mac-lib --dry-run

# Set a custom size threshold (200 MB)
afc mac-lib --min-size 200

# Show only macOS Library entries of 500 MB or more
afc mac-lib --min-size 500 --dry-run

# Scan only Caches
afc mac-lib --dirs caches

# Scan Caches and Containers only
afc mac-lib --dirs caches,containers

# `-y` is accepted for compatibility, but mac-lib still asks before deleting
afc mac-lib --yes
```

`--min-size` is measured in MB and applies only to `mac-lib`; it does not filter workspace artifacts in `scan` or `size`.

### Discover commands and options

```bash
# General command reference
afc --help

# Command-specific options and examples
afc scan --help
afc size --help
afc mac-lib --help
```

## Benchmark

Build an optimized binary first:

```bash
cargo build --release
```

Then benchmark a dry run with `hyperfine`:

```bash
hyperfine \
  './target/release/artifact-cleaner --path ~/projects --dry-run'
```

## Learn Rust with this project

This repo is designed to help Go, Python, and Java developers understand Rust through a real DevOps CLI project. The source code contains 34 inline `RUST LESSON` blocks explaining each language construct in context.

### Choose your learning path

**New to programming or Rust?**
→ Start with [RUST_LEARNING.md §0 — Before You Start](./RUST_LEARNING.md#0-before-you-start) then read the source files below.

**Already know Go, Python, or Java?**
→ Jump straight into the source files, then use [RUST_LEARNING.md](./RUST_LEARNING.md) as a reference.

**Prefer articles?**
→ Read the three-part series:
1. [Part 1: Foundations](articles/rust-learning-part-1-foundations.md)
2. [Part 2: Ownership, Borrowing, and Errors](articles/rust-learning-part-2-ownership-errors.md)
3. [Part 3: Production Patterns in a Real CLI](articles/rust-learning-part-3-production-patterns.md)

Or start even simpler: [Rust for Backend Developers](articles/rust-for-backend-developers.md).

### Suggested source reading order

```
src/scanner.rs              ← structs, constants, functions, iterators, match
src/cleaner.rs              ← ownership, error handling, implicit return
src/display.rs              ← imports, formatting, slices, private helpers
src/main.rs                 ← mod declarations, clap, subcommands, closures
src/mac_library/
  resolver.rs               ← enums as data, private helpers
  checker.rs                ← subprocess calls, std::process::Command
  scanner.rs                ← super::, flatten, closures, Option chaining
  mod.rs                    ← cfg, matches!, borrowing in loops
  display.rs                ← slices, tuple returns
```

**Quick reference:** [LESSONS.md](./LESSONS.md) — index of all 34 lesson blocks by topic with direct links to source lines.

## Commands

```
afc <COMMAND>

Commands:
  scan     Scan a workspace directory for stale build artifacts
  size     Show artifact folder sizes directly under the current directory
  mac-lib  Scan macOS Library folders for orphaned app/tool data
  help     Print help for any command

afc scan [OPTIONS] [PATH]
  [PATH]                   Workspace directory to scan [default: .]
  -m, --months <MONTHS>    Stale threshold in months [default: 2]
  -t, --types <TYPES>      Artifact types [default: node_modules,.next,dist,build,.terraform]
  -d, --dry-run            Preview without deleting
  -y, --yes                Skip confirmation prompt
      --no-interactive     Non-interactive output only

afc size [OPTIONS] [PATH]
  [PATH]                   Directory to inspect [default: .]
  -t, --types <TYPES>      Artifact types [default: node_modules,.next,dist,build,.terraform]

afc mac-lib [OPTIONS]
      --min-size <MB>      Minimum item size to flag in MB [default: 100]
      --dirs <DIRS>        Directories to scan: caches,containers,groups [default: all]
  -d, --dry-run            Preview without deleting
  -y, --yes                Accepted for compatibility; mac-lib still asks before deleting
```

## What it skips (always safe)

- Python virtual environments: `.venv`, `venv`, `env`
- Python packages: `lib/python*`, `site-packages`
- Git metadata: `.git`
- Projects modified within the threshold window

## License

MIT
