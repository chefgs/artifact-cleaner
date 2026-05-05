#!/usr/bin/env bash
# install.sh — artifact-cleaner installer for macOS and Linux
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.sh | bash
#
# Options (set via environment variables before piping):
#   VERSION=v1.2.0   install a specific version (default: latest)
#   INSTALL_DIR=/custom/path  install to a custom directory (default: /usr/local/bin or ~/.local/bin)
#   NO_VERIFY=1      skip SHA256 checksum verification (not recommended)
#
# What this script does:
#   1. Detects your OS (macOS or Linux) and CPU architecture (x86_64 or arm64)
#   2. Picks the correct binary for your machine
#   3. Downloads the binary archive and checksums.txt from GitHub Releases
#   4. Verifies the SHA256 checksum — aborts if it does not match
#   5. Extracts the binary and installs it to your PATH
#   6. Verifies the installed binary runs correctly

set -euo pipefail
# set -e  → exit immediately if any command fails
# set -u  → treat unset variables as errors
# set -o pipefail → if any command in a pipe fails, the whole pipe fails

# ── Configuration ─────────────────────────────────────────────────────────────
REPO="chefgs/artifact-cleaner"
BINARY="artifact-cleaner"
VERSION="${VERSION:-}"        # empty = fetch latest from GitHub API
NO_VERIFY="${NO_VERIFY:-0}"  # set to 1 to skip checksum (not recommended)
ARTIFACT_CLEANER_INSTALL_TMP_DIR=""

# ── Colours for terminal output ───────────────────────────────────────────────
# tput checks if the terminal supports colour — falls back to no colour if not
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
  RED=$(tput setaf 1); GREEN=$(tput setaf 2); YELLOW=$(tput setaf 3)
  CYAN=$(tput setaf 6); BOLD=$(tput bold); RESET=$(tput sgr0)
else
  RED=""; GREEN=""; YELLOW=""; CYAN=""; BOLD=""; RESET=""
fi

# ── Helper functions ──────────────────────────────────────────────────────────
info()    { echo "${CYAN}${BOLD}info${RESET}  $*" >&2; }
success() { echo "${GREEN}${BOLD} ok ${RESET}  $*" >&2; }
warn()    { echo "${YELLOW}${BOLD}warn${RESET}  $*" >&2; }
error()   { echo "${RED}${BOLD}err ${RESET}  $*" >&2; exit 1; }

# ── Step 1 — Detect OS ────────────────────────────────────────────────────────
# uname -s returns: Darwin (macOS) or Linux
detect_os() {
  local os
  os="$(uname -s)"
  case "$os" in
    Darwin) echo "apple-darwin" ;;
    Linux)  echo "unknown-linux-musl" ;;
    *)      error "Unsupported OS: $os. Please download manually from https://github.com/$REPO/releases" ;;
  esac
}

# ── Step 2 — Detect CPU architecture ─────────────────────────────────────────
# uname -m returns: x86_64, arm64 (macOS), or aarch64 (Linux)
detect_arch() {
  local arch
  arch="$(uname -m)"
  case "$arch" in
    x86_64)         echo "x86_64" ;;
    arm64|aarch64)  echo "aarch64" ;;
    *)              error "Unsupported architecture: $arch. Please download manually from https://github.com/$REPO/releases" ;;
  esac
}

# ── Step 3 — Resolve the version to install ───────────────────────────────────
# If VERSION is not set, query the GitHub API for the latest release tag.
# The API returns JSON — we extract the tag_name with grep + sed (no jq needed).
resolve_version() {
  if [ -n "$VERSION" ]; then
    echo "$VERSION"
    return
  fi

  info "Fetching latest release version..."

  local api_url="https://api.github.com/repos/$REPO/releases/latest"
  local version

  if command -v curl >/dev/null 2>&1; then
    version=$(curl -fsSL "$api_url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
  elif command -v wget >/dev/null 2>&1; then
    version=$(wget -qO- "$api_url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
  else
    error "Neither curl nor wget found. Please install one and retry."
  fi

  if [ -z "$version" ]; then
    error "Could not determine latest version — no releases found for $REPO.
  Create a release first (push a git tag like 'v0.1.0', or trigger the Release workflow manually from GitHub Actions),
  or set VERSION=vX.X.X to install a specific version once a release exists."
  fi

  echo "$version"
}

# ── Step 4 — Download a file ──────────────────────────────────────────────────
# Tries curl first, falls back to wget.
# -fsSL = fail on HTTP errors, silent, follow redirects, show errors
download() {
  local url="$1"
  local dest="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --progress-bar "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -q --show-progress "$url" -O "$dest"
  else
    error "Neither curl nor wget found. Please install one and retry."
  fi
}

# ── Step 5 — Verify SHA256 checksum ──────────────────────────────────────────
# checksums.txt format (same as sha256sum output):
#   <hash>  <filename>
#
# We grep for our archive filename, extract the expected hash,
# compute the actual hash of the downloaded file, and compare them.
#
# WHY verify?
# - Confirms the file was not corrupted during download
# - Confirms the file was not tampered with (man-in-the-middle attack)
# - Industry standard — every serious CLI tool ships checksums
verify_checksum() {
  local archive="$1"
  local checksums_file="$2"

  if [ "$NO_VERIFY" = "1" ]; then
    warn "Skipping checksum verification (NO_VERIFY=1)"
    return
  fi

  info "Verifying SHA256 checksum..."

  local filename
  filename="$(basename "$archive")"

  # Extract the expected hash from checksums.txt for our specific file
  local expected_hash
  expected_hash=$(grep "  $filename$" "$checksums_file" | awk '{print $1}')

  if [ -z "$expected_hash" ]; then
    error "Could not find checksum for $filename in checksums.txt"
  fi

  # Compute the actual hash of the downloaded file
  # macOS uses `shasum -a 256`, Linux uses `sha256sum`
  local actual_hash
  if command -v sha256sum >/dev/null 2>&1; then
    actual_hash=$(sha256sum "$archive" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual_hash=$(shasum -a 256 "$archive" | awk '{print $1}')
  else
    warn "Neither sha256sum nor shasum found — skipping verification"
    return
  fi

  # Compare — abort immediately if they differ
  if [ "$actual_hash" != "$expected_hash" ]; then
    error "Checksum mismatch for $filename!
  Expected : $expected_hash
  Got      : $actual_hash
  The file may be corrupted or tampered with. Aborting."
  fi

  success "Checksum verified: $actual_hash"
}

# ── Step 6 — Determine install directory ─────────────────────────────────────
# Preference order:
#   1. INSTALL_DIR env var (user override)
#   2. /usr/local/bin if writable (system-wide, no sudo prompt in most cases)
#   3. ~/.local/bin  (user-local, no sudo needed — created if missing)
resolve_install_dir() {
  if [ -n "${INSTALL_DIR:-}" ]; then
    echo "$INSTALL_DIR"
    return
  fi

  if [ -w "/usr/local/bin" ]; then
    echo "/usr/local/bin"
  else
    echo "$HOME/.local/bin"
  fi
}

# ── Step 7 — Remove macOS Gatekeeper quarantine flag ─────────────────────────
# macOS tags downloaded files with a quarantine extended attribute.
# This causes the "unidentified developer" popup on first run.
# xattr -d removes the flag — standard practice for non-App-Store CLI tools.
remove_quarantine() {
  local binary_path="$1"
  if [ "$(uname -s)" = "Darwin" ] && command -v xattr >/dev/null 2>&1; then
    xattr -d com.apple.quarantine "$binary_path" 2>/dev/null || true
    # 2>/dev/null || true — silently ignore if the attribute doesn't exist
  fi
}

# ── Main ──────────────────────────────────────────────────────────────────────
main() {
  echo ""
  echo "${BOLD}  artifact-cleaner installer${RESET}"
  echo "  ─────────────────────────────────────────"
  echo ""

  local os arch version target archive_name base_url
  local tmp_dir archive_path checksums_path install_dir binary_path

  os=$(detect_os)
  arch=$(detect_arch)
  version=$(resolve_version)

  # Compose the target triple and archive filename
  target="${arch}-${os}"
  archive_name="${BINARY}-${version}-${target}.tar.gz"
  base_url="https://github.com/$REPO/releases/download/$version"

  info "Version      : $version"
  info "Platform     : $target"
  info "Archive      : $archive_name"
  echo ""

  # Create a temporary working directory — cleaned up automatically on exit
  # trap ensures cleanup even if the script fails mid-way
  tmp_dir=$(mktemp -d)
  ARTIFACT_CLEANER_INSTALL_TMP_DIR="$tmp_dir"
  trap 'rm -rf "$ARTIFACT_CLEANER_INSTALL_TMP_DIR"' EXIT

  archive_path="$tmp_dir/$archive_name"
  checksums_path="$tmp_dir/checksums.txt"

  # Download the binary archive
  info "Downloading $archive_name..."
  download "$base_url/$archive_name" "$archive_path"

  # Download the checksums file
  info "Downloading checksums.txt..."
  download "$base_url/checksums.txt" "$checksums_path"

  # Verify integrity
  verify_checksum "$archive_path" "$checksums_path"

  # Extract — tar -xzf extracts .tar.gz into tmp_dir
  info "Extracting..."
  tar -xzf "$archive_path" -C "$tmp_dir"

  # Find the extracted binary
  local extracted_binary
  extracted_binary=$(find "$tmp_dir" -type f -name "$BINARY" | head -1)

  if [ -z "$extracted_binary" ]; then
    error "Could not find binary '$BINARY' in extracted archive"
  fi

  # Resolve install directory and create it if needed
  install_dir=$(resolve_install_dir)
  mkdir -p "$install_dir"

  binary_path="$install_dir/$BINARY"

  # Install — move binary to target location
  # If /usr/local/bin is not writable without sudo, try sudo
  if [ -w "$install_dir" ]; then
    cp "$extracted_binary" "$binary_path"
    chmod +x "$binary_path"
  else
    info "Installing to $install_dir (sudo required)..."
    sudo cp "$extracted_binary" "$binary_path"
    sudo chmod +x "$binary_path"
  fi

  # Remove Gatekeeper quarantine on macOS
  remove_quarantine "$binary_path"

  echo ""
  success "Installed to $binary_path"

  # ── PATH check ───────────────────────────────────────────────────────────────
  # Warn the user if the install directory is not in their current PATH.
  # This is a common gotcha — the binary is installed but the shell can't find it.
  if ! echo "$PATH" | grep -q "$install_dir"; then
    echo ""
    warn "$install_dir is not in your PATH."
    echo ""
    echo "  Add this line to your shell config:"
    echo ""
    if [ -n "${ZSH_VERSION:-}" ] || [ "$(basename "${SHELL:-}")" = "zsh" ]; then
      echo "    echo 'export PATH=\"$install_dir:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
    else
      echo "    echo 'export PATH=\"$install_dir:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
    fi
    echo ""
  fi

  # ── Verify the binary works ───────────────────────────────────────────────────
  info "Verifying installation..."
  if "$binary_path" --version >/dev/null 2>&1; then
    local installed_version
    installed_version=$("$binary_path" --version)
    success "$installed_version is ready."
  else
    warn "Binary installed but could not run. Check the PATH and permissions above."
  fi

  echo ""
  echo "  ${BOLD}Quick start:${RESET}"
  echo "    $BINARY --help"
  echo "    $BINARY ~/Documents/github --dry-run"
  echo ""
}

main "$@"
