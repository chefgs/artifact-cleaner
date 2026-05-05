# Installation Guide

Step-by-step instructions to download, install, and configure `artifact-cleaner`
on macOS, Linux, and Windows.

---

## One-liner install (recommended)

The install scripts automatically detect your OS and CPU architecture, download
the correct binary, **verify the SHA256 checksum**, and add the binary to your PATH.

### macOS and Linux

```bash
curl -fsSL https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.sh | bash
```

Install a specific version:
```bash
VERSION=v1.2.0 curl -fsSL https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.sh | bash
```

Install to a custom directory:
```bash
INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.ps1 | iex
```

Install a specific version:
```powershell
$env:VERSION = "v1.2.0"
irm https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.ps1 | iex
```

---

## What the install script does

```
1. Detects your OS and CPU architecture (x86_64 or arm64/aarch64)
2. Fetches the latest release version from the GitHub API
3. Downloads the correct binary archive (.tar.gz or .zip)
4. Downloads checksums.txt from the same release
5. Verifies the SHA256 checksum — aborts immediately if it does not match
6. Extracts the binary
7. Installs to /usr/local/bin or ~/.local/bin (macOS/Linux)
              or $HOME\bin (Windows)
8. Adds the install directory to PATH if needed
9. Removes macOS Gatekeeper quarantine flag (macOS only)
10. Verifies the binary runs correctly
```

---

## About SHA256 verification

Every release publishes a `checksums.txt` file alongside the binaries:

```
# checksums.txt — example contents
a3f1d8...  artifact-cleaner-v1.0.0-aarch64-apple-darwin.tar.gz
b9c2e4...  artifact-cleaner-v1.0.0-x86_64-apple-darwin.tar.gz
c7d3f1...  artifact-cleaner-v1.0.0-x86_64-unknown-linux-musl.tar.gz
d4e8a2...  artifact-cleaner-v1.0.0-aarch64-unknown-linux-musl.tar.gz
e5f9b3...  artifact-cleaner-v1.0.0-x86_64-pc-windows-msvc.zip
f6a0c4...  artifact-cleaner-v1.0.0-aarch64-pc-windows-msvc.zip
```

**Why checksums matter:**
- **Integrity** — confirms the file was not corrupted during download
- **Authenticity** — confirms nobody tampered with the binary between GitHub and your machine (man-in-the-middle protection)
- **Industry standard** — used by Go, Terraform, Homebrew, rustup, and every serious CLI tool

**Manually verify any download:**

```bash
# macOS
shasum -a 256 artifact-cleaner-v1.0.0-aarch64-apple-darwin.tar.gz

# Linux
sha256sum artifact-cleaner-v1.0.0-x86_64-unknown-linux-musl.tar.gz

# Compare output against the hash in checksums.txt — they must match exactly
```

```powershell
# Windows PowerShell
(Get-FileHash artifact-cleaner-v1.0.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256).Hash
# Compare against checksums.txt — must match (case-insensitive)
```

**Skip verification** (not recommended — only for testing):
```bash
NO_VERIFY=1 curl -fsSL .../install.sh | bash   # macOS/Linux
$env:NO_VERIFY="1"; irm .../install.ps1 | iex  # Windows
```

---

## Table of Contents

- [One-liner install](#one-liner-install-recommended)
- [About SHA256 verification](#about-sha256-verification)
- [Before you start — find the right binary](#before-you-start--find-the-right-binary)
- [macOS — manual install](#macos)
- [Linux — manual install](#linux)
- [Windows — manual install](#windows)
- [Build from source (all platforms)](#build-from-source-all-platforms)
- [Verify the installation](#verify-the-installation)
- [Uninstall](#uninstall)

---

## Before you start — find the right binary

Every release ships 6 binaries. Pick the one that matches your machine:

| Your machine | Binary to download |
|-------------|-------------------|
| Mac with M1 / M2 / M3 chip (2020 or later) | `artifact-cleaner-vX.X.X-aarch64-apple-darwin.tar.gz` |
| Mac with Intel chip (2019 or earlier) | `artifact-cleaner-vX.X.X-x86_64-apple-darwin.tar.gz` |
| Linux PC / server (most common) | `artifact-cleaner-vX.X.X-x86_64-unknown-linux-musl.tar.gz` |
| Linux on ARM (AWS Graviton, Raspberry Pi 4) | `artifact-cleaner-vX.X.X-aarch64-unknown-linux-musl.tar.gz` |
| Windows PC / laptop (most common) | `artifact-cleaner-vX.X.X-x86_64-pc-windows-msvc.zip` |
| Windows on ARM (Surface Pro X, Snapdragon laptop) | `artifact-cleaner-vX.X.X-aarch64-pc-windows-msvc.zip` |

**Not sure which Mac chip you have?**
→ Click the Apple menu () → About This Mac → look for "Chip" (M1/M2/M3 = Apple Silicon) or "Processor" (Intel = x86_64).

**Not sure which Linux architecture you have?**
```bash
uname -m
# x86_64  → use the x86_64 binary
# aarch64 → use the aarch64 binary
```

**Not sure which Windows architecture you have?**
→ Settings → System → About → look for "System type".
→ "64-bit operating system, x64-based processor" = use the x86_64 binary.

---

## macOS

### Step 1 — Detect your chip

```bash
uname -m
```

- `arm64` → you have Apple Silicon (M1/M2/M3) → use `aarch64-apple-darwin`
- `x86_64` → you have Intel → use `x86_64-apple-darwin`

### Step 2 — Download and extract

**Option A — Terminal (fastest, one command)**

Apple Silicon:
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-aarch64-apple-darwin.tar.gz
```

Intel:
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-x86_64-apple-darwin.tar.gz
```

Then extract:
```bash
tar -xzf artifact-cleaner.tar.gz
```

What this produces:
```
artifact-cleaner-v1.0.0-aarch64-apple-darwin/
└── artifact-cleaner        ← the binary
```

**Option B — Browser**
1. Go to the [Releases page](https://github.com/chefgs/artifact-cleaner/releases/latest)
2. Download the correct `.tar.gz` file
3. Double-click it in Finder — macOS extracts it automatically

### Step 3 — Move the binary to your PATH

The most common install location on macOS is `/usr/local/bin` — it is already in your PATH by default.

```bash
# Move the binary (sudo required to write to /usr/local/bin)
sudo mv artifact-cleaner-*/artifact-cleaner /usr/local/bin/

# Make sure it is executable (should already be, but just in case)
sudo chmod +x /usr/local/bin/artifact-cleaner
```

### Step 4 — Handle Gatekeeper (macOS security prompt)

macOS Gatekeeper blocks binaries that are not signed by an Apple Developer certificate.
On first run you may see: *"artifact-cleaner cannot be opened because it is from an unidentified developer"*

**Fix — one-time approval via Terminal:**
```bash
xattr -dr com.apple.quarantine /usr/local/bin/artifact-cleaner
```

This removes the quarantine flag that Gatekeeper adds when you download a file.
You only need to do this once.

**Alternative fix via System Settings:**
1. Try running `artifact-cleaner` — macOS blocks it and shows the error
2. Open **System Settings → Privacy & Security**
3. Scroll down to the Security section — you will see "artifact-cleaner was blocked"
4. Click **Allow Anyway**
5. Run `artifact-cleaner` again and click **Open** in the confirmation dialog

### Step 5 — Verify

```bash
artifact-cleaner --version
# artifact-cleaner 1.0.0
```

### Optional — install to user bin (no sudo needed)

If you do not want to use `sudo`, install to `~/.local/bin` instead:

```bash
mkdir -p ~/.local/bin
mv artifact-cleaner-*/artifact-cleaner ~/.local/bin/
```

Then add `~/.local/bin` to your PATH. Add this line to `~/.zshrc` (zsh, default on macOS) or `~/.bashrc` (bash):

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Reload your shell:
```bash
source ~/.zshrc   # or source ~/.bashrc
```

---

## Linux

### Step 1 — Detect your architecture

```bash
uname -m
# x86_64  → use x86_64-unknown-linux-musl
# aarch64 → use aarch64-unknown-linux-musl
```

### Step 2 — Download and extract

**x86_64 (most Linux machines):**
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-x86_64-unknown-linux-musl.tar.gz
```

**ARM64 (AWS Graviton, Raspberry Pi 4+):**
```bash
curl -Lo artifact-cleaner.tar.gz \
  https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-aarch64-unknown-linux-musl.tar.gz
```

Extract:
```bash
tar -xzf artifact-cleaner.tar.gz
```

> **Why musl?** The Linux binaries are statically linked using the musl C library.
> This means the binary has **zero runtime dependencies** — it runs on any Linux
> distribution (Ubuntu, Debian, Fedora, Alpine, RHEL, Arch) without installing
> anything else. No glibc version conflicts. No "missing library" errors.

### Step 3 — Install the binary

**System-wide install** (available to all users, requires sudo):
```bash
sudo mv artifact-cleaner-*/artifact-cleaner /usr/local/bin/
sudo chmod +x /usr/local/bin/artifact-cleaner
```

**User install** (no sudo required, only for your user):
```bash
mkdir -p ~/.local/bin
mv artifact-cleaner-*/artifact-cleaner ~/.local/bin/
chmod +x ~/.local/bin/artifact-cleaner
```

If you chose the user install, add `~/.local/bin` to your PATH.
Add this to `~/.bashrc` or `~/.zshrc`:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

Reload your shell:
```bash
source ~/.bashrc    # or source ~/.zshrc
```

### Step 4 — Verify it is a static binary (optional but educational)

```bash
file /usr/local/bin/artifact-cleaner
# artifact-cleaner: ELF 64-bit LSB executable, x86-64,
#   statically linked, stripped
#                   ^^^^^^^^^^^^^^^^
#                   confirms no external dependencies
```

### Step 5 — Verify the install

```bash
artifact-cleaner --version
# artifact-cleaner 1.0.0
```

### Package manager support (roadmap)

For now, installation is via the one-liner script or manual download.
The following package managers are planned for future releases:

#### Homebrew (macOS and Linux)

Homebrew is the most popular package manager on macOS and works on Linux too.
Supporting it requires publishing a **Homebrew tap** — a separate GitHub repo
(`chefgs/homebrew-tap`) containing a Ruby `Formula` file.

Once published, users would install with:
```bash
brew tap chefgs/tap
brew install artifact-cleaner
```

And upgrade with:
```bash
brew upgrade artifact-cleaner
```

**What needs to be built:**
1. Create a repo named `chefgs/homebrew-tap`
2. Add `Formula/artifact-cleaner.rb` with the download URL, SHA256, and install instructions
3. Add a GitHub Actions workflow to auto-update the formula SHA256 on each release

#### APT (Debian / Ubuntu)

APT requires building a `.deb` package and hosting a signed APT repository.
Once published, users would install with:
```bash
curl -fsSL https://chefgs.github.io/artifact-cleaner/apt/gpg.key | sudo apt-key add -
echo "deb https://chefgs.github.io/artifact-cleaner/apt stable main" | sudo tee /etc/apt/sources.list.d/artifact-cleaner.list
sudo apt update && sudo apt install artifact-cleaner
```

**What needs to be built:** `.deb` packaging in the release workflow + a GitHub Pages-hosted APT repo.

#### RPM (Fedora / RHEL / CentOS)

Similar to APT but for Red Hat-based distros:
```bash
sudo dnf install https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner.x86_64.rpm
```

**What needs to be built:** `.rpm` packaging using `cargo-generate-rpm` in the release workflow.

#### Scoop (Windows)

Scoop is a popular Windows package manager for developers:
```powershell
scoop bucket add chefgs https://github.com/chefgs/scoop-bucket
scoop install artifact-cleaner
```

**What needs to be built:** A Scoop bucket repo with a `artifact-cleaner.json` manifest.

#### Cargo (all platforms — already works)

Since this is a Rust project, anyone with Rust installed can already use:
```bash
cargo install artifact-cleaner   # once published to crates.io
```

**What needs to be done:** `cargo publish` to [crates.io](https://crates.io).

---

## Windows

### Step 1 — Detect your architecture

1. Press `Win + I` to open Settings
2. Go to **System → About**
3. Look at "System type":
   - `64-bit operating system, x64-based processor` → use `x86_64-pc-windows-msvc`
   - `64-bit operating system, ARM-based processor` → use `aarch64-pc-windows-msvc`

Or from PowerShell:
```powershell
$env:PROCESSOR_ARCHITECTURE
# AMD64  → x86_64
# ARM64  → aarch64
```

### Step 2 — Download the .zip file

**Option A — PowerShell (recommended)**

x86_64:
```powershell
Invoke-WebRequest `
  -Uri "https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-x86_64-pc-windows-msvc.zip" `
  -OutFile "artifact-cleaner.zip"
```

ARM64:
```powershell
Invoke-WebRequest `
  -Uri "https://github.com/chefgs/artifact-cleaner/releases/latest/download/artifact-cleaner-latest-aarch64-pc-windows-msvc.zip" `
  -OutFile "artifact-cleaner.zip"
```

**Option B — Browser**
1. Go to the [Releases page](https://github.com/chefgs/artifact-cleaner/releases/latest)
2. Click the correct `.zip` file to download it

### Step 3 — Extract the .zip

**PowerShell:**
```powershell
Expand-Archive -Path artifact-cleaner.zip -DestinationPath .
```

**File Explorer:**
1. Right-click the `.zip` file
2. Select **Extract All...**
3. Choose a destination folder and click **Extract**

The extracted folder contains:
```
artifact-cleaner-v1.0.0-x86_64-pc-windows-msvc\
└── artifact-cleaner.exe        ← the binary
```

### Step 4 — Add the binary to your PATH

There are two ways — pick one.

---

**Method A — Install to a dedicated tools folder (recommended)**

1. Create a permanent folder for CLI tools (if you do not have one already):
```powershell
New-Item -ItemType Directory -Path "$HOME\bin" -Force
```

2. Move the binary into it:
```powershell
Move-Item "artifact-cleaner-*\artifact-cleaner.exe" "$HOME\bin\"
```

3. Add `$HOME\bin` to your PATH permanently:
```powershell
# This updates your user PATH in the Windows Registry — no admin required
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
[Environment]::SetEnvironmentVariable(
    "PATH",
    "$currentPath;$HOME\bin",
    "User"
)
```

4. Restart your terminal (PowerShell, Command Prompt, or Windows Terminal) for the PATH change to take effect.

---

**Method B — System-wide install (requires Administrator)**

1. Move the binary to `C:\Windows\System32` (already in PATH for all users):
```powershell
# Run PowerShell as Administrator
Move-Item "artifact-cleaner-*\artifact-cleaner.exe" "C:\Windows\System32\"
```

> Note: writing to `System32` requires Administrator rights. Right-click PowerShell → "Run as administrator".

---

**Method C — Add the extracted folder directly to PATH (quickest)**

1. Extract the zip to a permanent location, e.g. `C:\tools\artifact-cleaner\`
2. Open **System Properties → Advanced → Environment Variables**
3. Under "User variables", select `Path` and click **Edit**
4. Click **New** and add the full path to the folder (e.g. `C:\tools\artifact-cleaner`)
5. Click **OK** on all dialogs
6. Restart your terminal

### Step 5 — Handle Windows SmartScreen (security prompt)

Windows Defender SmartScreen may warn: *"Windows protected your PC — artifact-cleaner.exe is from an unknown publisher"*

**Fix:**
1. Click **More info** in the SmartScreen dialog
2. Click **Run anyway**

This happens because the binary is not yet code-signed with a Windows certificate.
You only need to do this once — subsequent runs are not blocked.

Alternatively, disable the check for this specific file via PowerShell:
```powershell
Unblock-File -Path "$HOME\bin\artifact-cleaner.exe"
```

### Step 6 — Verify

Open a new PowerShell or Command Prompt window and run:
```powershell
artifact-cleaner --version
# artifact-cleaner 1.0.0

artifact-cleaner --help
```

---

## Build from source (all platforms)

If you have Rust installed, you can build directly from source.
This always produces the correct binary for your current machine.

### Install Rust (if not already installed)

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Windows — download and run:
# https://win.rustup.rs/x86_64
```

### Build and install

```bash
# Clone the repository
git clone https://github.com/chefgs/artifact-cleaner.git
cd artifact-cleaner

# Build a release binary and install it to ~/.cargo/bin/ (already in PATH)
cargo install --path .

# Verify
artifact-cleaner --version
```

`~/.cargo/bin` is added to your PATH automatically by the Rust installer.
On Windows it is `%USERPROFILE%\.cargo\bin`.

---

## Verify the installation

Run these commands after installing to confirm everything works:

```bash
# Check the version
artifact-cleaner --version

# Check the full help
artifact-cleaner --help

# Safe test — dry run on your home directory, no files deleted
artifact-cleaner ~ --months 3 --dry-run
```

Expected output:
```
  Artifact Cleaner
  ──────────────────────────────────────────────────────────────────────
  Workspace  : /Users/yourname
  Threshold  : 3 months
  Found      : N folders · X.X GB
  ──────────────────────────────────────────────────────────────────────
  ...
  ◉ Dry run — N folders would be deleted, freeing X.X GB
```

---

## Uninstall

### macOS / Linux

```bash
# If installed to /usr/local/bin
sudo rm /usr/local/bin/artifact-cleaner

# If installed to ~/.local/bin
rm ~/.local/bin/artifact-cleaner
```

### Windows (PowerShell)

```powershell
# If installed to $HOME\bin
Remove-Item "$HOME\bin\artifact-cleaner.exe"

# If installed to System32 (requires Administrator)
Remove-Item "C:\Windows\System32\artifact-cleaner.exe"
```

Then remove the PATH entry you added:

**PowerShell (user PATH):**
```powershell
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$newPath = ($currentPath -split ";" | Where-Object { $_ -notlike "*artifact-cleaner*" }) -join ";"
[Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
```

**System Properties (GUI):**
1. Open **System Properties → Advanced → Environment Variables**
2. Select the `Path` entry you added and click **Delete**
3. Click **OK**

### Built from source (all platforms)

```bash
cargo uninstall artifact-cleaner
```

---

## Troubleshooting

| Problem | Cause | Fix |
|---------|-------|-----|
| `command not found` | Binary not in PATH | Check the PATH step for your OS |
| macOS: "cannot be opened" | Gatekeeper quarantine | Run `xattr -dr com.apple.quarantine $(which artifact-cleaner)` |
| Windows: SmartScreen warning | Binary not code-signed | Click "More info" → "Run anyway", or run `Unblock-File` |
| Linux: `Permission denied` | Binary not executable | Run `chmod +x /path/to/artifact-cleaner` |
| Wrong architecture | Downloaded wrong binary | Run `uname -m` (Linux/macOS) or check System Settings (Windows) and download the matching binary |
| `Illegal instruction` on Linux | Compiled for wrong CPU | Make sure you have the correct arch — x86_64 vs aarch64 |
