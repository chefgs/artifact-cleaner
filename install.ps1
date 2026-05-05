# install.ps1 — artifact-cleaner installer for Windows (PowerShell)
#
# Usage (run in PowerShell):
#   irm https://raw.githubusercontent.com/chefgs/artifact-cleaner/main/install.ps1 | iex
#
# Options (set as variables before running):
#   $env:VERSION    = "v1.2.0"           # install a specific version (default: latest)
#   $env:INSTALL_DIR = "C:\tools\bin"    # custom install directory
#   $env:NO_VERIFY  = "1"                # skip SHA256 verification (not recommended)
#
# What this script does:
#   1. Detects your CPU architecture (x64 or ARM64)
#   2. Picks the correct binary for your machine
#   3. Downloads the binary .zip and checksums.txt from GitHub Releases
#   4. Verifies the SHA256 checksum — aborts if it does not match
#   5. Extracts artifact-cleaner.exe and installs it to your PATH
#   6. Verifies the installed binary runs correctly

# Stop on any error — equivalent to `set -e` in bash
$ErrorActionPreference = "Stop"

# ── Configuration ─────────────────────────────────────────────────────────────
$REPO    = "chefgs/artifact-cleaner"
$BINARY  = "artifact-cleaner"
$VERSION   = $env:VERSION    # empty = fetch latest
$NO_VERIFY = $env:NO_VERIFY  # "1" = skip checksum

# ── Colour helper functions ────────────────────────────────────────────────────
# Write-Host supports -ForegroundColor natively in PowerShell
function Info    { param($msg) Write-Host "info  $msg" -ForegroundColor Cyan }
function Success { param($msg) Write-Host " ok   $msg" -ForegroundColor Green }
function Warn    { param($msg) Write-Host "warn  $msg" -ForegroundColor Yellow }
function Err     { param($msg) Write-Host "err   $msg" -ForegroundColor Red; exit 1 }

# ── Step 1 — Detect CPU architecture ──────────────────────────────────────────
# $env:PROCESSOR_ARCHITECTURE is set by Windows for the current process.
# AMD64 = x86_64 (standard Intel/AMD 64-bit)
# ARM64 = aarch64 (Surface Pro X, Snapdragon laptops)
function Detect-Arch {
  switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { return "x86_64" }
    "ARM64" { return "aarch64" }
    default { Err "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE. Download manually from https://github.com/$REPO/releases" }
  }
}

# ── Step 2 — Resolve version to install ───────────────────────────────────────
# If VERSION is not set, query GitHub API for the latest release.
# Invoke-RestMethod parses JSON automatically — no grep/sed needed in PowerShell.
function Resolve-Version {
  if ($VERSION) { return $VERSION }

  Info "Fetching latest release version..."
  try {
    $release = Invoke-RestMethod "https://api.github.com/repos/$REPO/releases/latest"
    return $release.tag_name
  } catch {
    Err "Could not fetch latest version: $_`nSet `$env:VERSION='vX.X.X' to install a specific version."
  }
}

# ── Step 3 — Download a file ──────────────────────────────────────────────────
# Invoke-WebRequest is PowerShell's equivalent of curl/wget.
# -UseBasicParsing avoids dependency on Internet Explorer engine (deprecated).
function Download-File {
  param([string]$Url, [string]$Dest)
  Info "Downloading $(Split-Path $Url -Leaf)..."
  Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing
}

# ── Step 4 — Verify SHA256 checksum ──────────────────────────────────────────
# checksums.txt format:  <hash>  <filename>
#
# Get-FileHash is PowerShell's built-in SHA256 tool.
# No external tools required — works on all modern Windows systems.
#
# WHY verify?
# - Confirms no corruption during download
# - Confirms no tampering (man-in-the-middle)
# - Same reason you verify downloads of any software
function Verify-Checksum {
  param([string]$Archive, [string]$ChecksumsFile)

  if ($NO_VERIFY -eq "1") {
    Warn "Skipping checksum verification (NO_VERIFY=1)"
    return
  }

  Info "Verifying SHA256 checksum..."

  $filename = Split-Path $Archive -Leaf

  # Read checksums.txt and find the line for our archive
  $checksums = Get-Content $ChecksumsFile
  $expected_line = $checksums | Where-Object { $_ -match "  $([regex]::Escape($filename))$" }

  if (-not $expected_line) {
    Err "Could not find checksum for '$filename' in checksums.txt"
  }

  # Extract just the hash (first field, before the two spaces)
  $expected_hash = ($expected_line -split "\s+")[0].ToLower()

  # Compute actual hash of downloaded file
  $actual_hash = (Get-FileHash $Archive -Algorithm SHA256).Hash.ToLower()

  if ($actual_hash -ne $expected_hash) {
    Err "Checksum mismatch for $filename!`n  Expected : $expected_hash`n  Got      : $actual_hash`n  The file may be corrupted or tampered with. Aborting."
  }

  Success "Checksum verified: $actual_hash"
}

# ── Step 5 — Resolve install directory ────────────────────────────────────────
# Preference order:
#   1. $env:INSTALL_DIR (user override)
#   2. $HOME\bin  (user-local, no admin needed — created if missing)
#
# We avoid system directories (C:\Windows\System32) as they need admin rights
# and are not the right place for user tools.
function Resolve-InstallDir {
  if ($env:INSTALL_DIR) { return $env:INSTALL_DIR }
  return Join-Path $HOME "bin"
}

# ── Step 6 — Add directory to user PATH ───────────────────────────────────────
# Updates the Windows Registry for the current user's PATH.
# Does NOT require Administrator privileges.
# Change takes effect in new terminal windows (not the current one).
function Add-ToPath {
  param([string]$Dir)

  $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

  # Check if already in PATH — avoid duplicates
  if ($currentPath -split ";" | Where-Object { $_ -eq $Dir }) {
    return $false   # already present
  }

  $newPath = "$currentPath;$Dir"
  [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")

  # Also update the current session's PATH so the binary works immediately
  $env:PATH = "$env:PATH;$Dir"

  return $true   # added
}

# ── Main ──────────────────────────────────────────────────────────────────────
function Main {
  Write-Host ""
  Write-Host "  artifact-cleaner installer" -ForegroundColor Cyan
  Write-Host "  ─────────────────────────────────────────"
  Write-Host ""

  $arch    = Detect-Arch
  $version = Resolve-Version
  $target  = "${arch}-pc-windows-msvc"
  $archive_name = "${BINARY}-${version}-${target}.zip"
  $base_url = "https://github.com/$REPO/releases/download/$version"

  Info "Version      : $version"
  Info "Platform     : $target"
  Info "Archive      : $archive_name"
  Write-Host ""

  # Create a temp directory — cleaned up at end of script
  $tmp_dir = Join-Path $env:TEMP "artifact-cleaner-install-$(Get-Random)"
  New-Item -ItemType Directory -Path $tmp_dir | Out-Null

  try {
    $archive_path   = Join-Path $tmp_dir $archive_name
    $checksums_path = Join-Path $tmp_dir "checksums.txt"

    # Download archive and checksums
    Download-File "$base_url/$archive_name" $archive_path
    Download-File "$base_url/checksums.txt" $checksums_path

    # Verify integrity
    Verify-Checksum $archive_path $checksums_path

    # Extract .zip
    Info "Extracting..."
    Expand-Archive -Path $archive_path -DestinationPath $tmp_dir -Force

    # Find the extracted binary
    $extracted_binary = Get-ChildItem -Path $tmp_dir -Recurse -Filter "${BINARY}.exe" | Select-Object -First 1

    if (-not $extracted_binary) {
      Err "Could not find ${BINARY}.exe in extracted archive"
    }

    # Resolve install directory and create it if needed
    $install_dir = Resolve-InstallDir
    New-Item -ItemType Directory -Path $install_dir -Force | Out-Null

    $binary_path = Join-Path $install_dir "${BINARY}.exe"

    # Install the binary
    Copy-Item $extracted_binary.FullName $binary_path -Force

    Write-Host ""
    Success "Installed to $binary_path"

    # Add install dir to user PATH if needed
    $added = Add-ToPath $install_dir
    if ($added) {
      Write-Host ""
      Warn "Added $install_dir to your user PATH."
      Write-Host "  Restart your terminal for the PATH change to take effect." -ForegroundColor Yellow
      Write-Host ""
    }

    # Unblock the file — removes the "downloaded from internet" mark
    # This prevents the SmartScreen "unknown publisher" dialog on first run
    Info "Removing SmartScreen download mark (Unblock-File)..."
    Unblock-File $binary_path

    # Verify the binary works
    Info "Verifying installation..."
    $installed_version = & $binary_path --version 2>&1
    if ($LASTEXITCODE -eq 0) {
      Success "$installed_version is ready."
    } else {
      Warn "Binary installed but could not run. Check the PATH above."
    }

    Write-Host ""
    Write-Host "  Quick start:" -ForegroundColor White
    Write-Host "    $BINARY --help"
    Write-Host "    $BINARY `$HOME\Documents\github --dry-run"
    Write-Host ""

  } finally {
    # Always clean up temp directory, even if something failed
    Remove-Item -Recurse -Force $tmp_dir -ErrorAction SilentlyContinue
  }
}

Main
