#!/usr/bin/env bash
# Print the Homebrew formula for a published artifact-cleaner release.
#
#   packaging/homebrew/generate-formula.sh 0.12.0 ../homebrew-tap/Formula/artifact-cleaner.rb
#
# Writes the file only on success (a shell redirect would truncate it first).
# Reads the .sha256 files attached to the GitHub release v<version> of this repo.
# BASE_URL can point elsewhere (e.g. file:///path/to/assets) for testing.
# Logs go to stderr; set QUIET=1 to silence them.
#
# Requires: curl, awk. Network access to github.com (each download times out at 30s).
# Before pushing the result, run `brew style <file>`; after pushing, see RELEASING.md.
# Background: docs/HOMEBREW.md
set -euo pipefail

log() { [[ "${QUIET:-0}" == "1" ]] || printf '[generate-formula] %s\n' "$*" >&2; }

version="${1:?usage: $0 <version, e.g. 0.12.0> [output-file]}"
out_file="${2:-}"
version="${version#v}"
repo="chefgs/artifact-cleaner"
base="${BASE_URL:-https://github.com/${repo}/releases/download/v${version}}"

sha_for() {
  local target="$1" file="artifact-cleaner-v${version}-${1}.tar.gz"
  local url="${base}/${file}.sha256" sum
  log "fetching checksum for ${target}: ${url}"
  sum="$(curl -fsSL -m 30 "${url}" 2>/dev/null | awk '{print $1}' || true)"
  if [[ -z "${sum}" ]]; then
    log "ERROR: could not read ${url}"
    log "       Check that release v${version} is published and the ${target} build succeeded."
    exit 1
  fi
  printf '%s' "${sum}"
}

arm_mac="$(sha_for aarch64-apple-darwin)"
intel_mac="$(sha_for x86_64-apple-darwin)"
arm_linux="$(sha_for aarch64-unknown-linux-musl)"
intel_linux="$(sha_for x86_64-unknown-linux-musl)"

if [[ -n "${out_file}" ]]; then
  exec 3>&1 1>"${out_file}.tmp"
fi

cat <<FORMULA
class ArtifactCleaner < Formula
  desc "Find and remove stale build artifacts like node_modules, dist and .terraform"
  homepage "https://github.com/${repo}"
  license "MIT"

  on_macos do
    on_arm do
      url "${base}/artifact-cleaner-v${version}-aarch64-apple-darwin.tar.gz"
      sha256 "${arm_mac}"
    end
    on_intel do
      url "${base}/artifact-cleaner-v${version}-x86_64-apple-darwin.tar.gz"
      sha256 "${intel_mac}"
    end
  end

  on_linux do
    on_arm do
      url "${base}/artifact-cleaner-v${version}-aarch64-unknown-linux-musl.tar.gz"
      sha256 "${arm_linux}"
    end
    on_intel do
      url "${base}/artifact-cleaner-v${version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "${intel_linux}"
    end
  end

  def install
    bin.install "artifact-cleaner"
    bin.install "afc"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/afc --version")
  end
end
FORMULA

if [[ -n "${out_file}" ]]; then
  exec 1>&3 3>&-
  mv "${out_file}.tmp" "${out_file}"
  log "wrote ${out_file}"
fi
