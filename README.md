# artifact-cleaner

A fast, cross-platform CLI tool to find and delete stale build artifacts — `node_modules`, `.next`, `dist`, `build`, `.terraform` — from developer workspaces.

Built in Rust. Single binary. No dependencies.

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
