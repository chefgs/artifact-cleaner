# Artifact Cleaner

[![CI](https://github.com/chefgs/artifact-cleaner/actions/workflows/ci.yml/badge.svg)](https://github.com/chefgs/artifact-cleaner/actions/workflows/ci.yml)
[![Release](https://github.com/chefgs/artifact-cleaner/actions/workflows/release.yml/badge.svg)](https://github.com/chefgs/artifact-cleaner/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Contributions welcome](https://img.shields.io/badge/contributions-welcome-brightgreen.svg)](CONTRIBUTING.md)

A Rust-based CLI tool for scanning and cleaning common development artifacts such as `node_modules`, `.next`, `dist`, `build`, `target`, `.terraform`, and cache folders.

This project is also used as a practical Rust learning project for Go, Python, and Java developers.

Built in Rust. Single binary. No dependencies.

![Artifact Cleaner CLI cleaning development artifacts and reclaiming storage](assets/repo-image.png)

## Safe by default

The tool always shows what it found before deleting anything. By default, destructive cleanup requires an interactive confirmation prompt, and `--dry-run` previews what would be deleted without removing files.

For scripts and CI, deletion without a prompt requires the explicit `--yes` flag.

## Install

### One-liner (recommended)

**macOS and Linux** — auto-detects your architecture, verifies SHA256, installs to PATH:
```bash
curl -fsSL https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.sh | bash
```

**Windows** (PowerShell) — same, auto-detects x64 or ARM64:
```powershell
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

**Linux (x64 — static binary, works on any distro):**
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-x86_64-unknown-linux-musl.tar.gz
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

**Build from source (all platforms):**
```bash
cargo install --path .
```

**Not sure which binary to pick?** See the [platform guide in INSTALL.md](./INSTALL.md#before-you-start--find-the-right-binary).

## Usage

```bash
# Scan current directory, 2-month threshold (default)
artifact-cleaner

# Scan a specific workspace
artifact-cleaner ~/Documents/github

# Use a 3-month threshold
artifact-cleaner ~/Documents/github --months 3

# Dry run — preview without deleting
artifact-cleaner ~/Documents/github --dry-run

# Skip confirmation prompt (for scripts/CI)
artifact-cleaner ~/Documents/github --yes

# Target specific artifact types only
artifact-cleaner ~/Documents/github --types node_modules,.next
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

This repo is designed to help Go, Python, and Java developers understand Rust through a real DevOps CLI project.

Start here:

1. Read `src/main.rs`
2. Read `src/scanner.rs`
3. Read `src/cleaner.rs`
4. Read `src/display.rs`
5. Then open `RUST_LEARNING.md`

For a shorter article-style path, read the three-part series:

1. [Part 1: Foundations for Go, Python, and Java Developers](articles/rust-learning-part-1-foundations.md)
2. [Part 2: Ownership, Borrowing, and Errors](articles/rust-learning-part-2-ownership-errors.md)
3. [Part 3: Production Patterns in a Real CLI](articles/rust-learning-part-3-production-patterns.md)

For an even simpler backend-focused introduction, read [Rust for Backend Developers: Simple Examples You Can Relate To](articles/rust-for-backend-developers.md).

## Options

```
Usage: artifact-cleaner [OPTIONS] [PATH]

Arguments:
  [PATH]  Workspace directory to scan [default: .]

Options:
  -m, --months <MONTHS>  Stale threshold in months [default: 2]
  -t, --types <TYPES>    Artifact types to target [default: node_modules,.next,dist,build,.terraform]
  -d, --dry-run          Preview without deleting
  -y, --yes              Skip confirmation prompt
      --no-interactive   Non-interactive output only
  -h, --help             Print help
  -V, --version          Print version
```

## What it skips (always safe)

- Python virtual environments: `.venv`, `venv`, `env`
- Python packages: `lib/python*`, `site-packages`
- Git metadata: `.git`
- Projects modified within the threshold window

## License

MIT
