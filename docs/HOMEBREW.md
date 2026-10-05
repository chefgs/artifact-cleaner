# How the Homebrew tap works

`brew install cloudengine-labs/tap/artifact-cleaner` is served by three separate pieces. Homebrew does not compile or host anything.

| Piece | Where | Role |
|---|---|---|
| Release archives | GitHub Releases of `chefgs/artifact-cleaner` | `.tar.gz` per platform plus a `.sha256` file, built by `.github/workflows/release.yml` on a version tag |
| Formula | `Formula/artifact-cleaner.rb` in the tap | Says which URL to download, which checksum to verify, and which binaries to install |
| Tap | `cloudengine-labs/homebrew-tap` (public) | A repo whose name starts with `homebrew-`; `cloudengine-labs/tap` is its short name |

The tap also hosts the `sbom-sentinel` formula. Each tool gets its own file in `Formula/`.

## Why it is simple here

The source repo is public, so the formula points straight at this repo's release assets. (A private source repo would need the archives copied to the public tap's Releases, because Homebrew downloads anonymously.)

## What the formula does

- Picks the archive for the user's OS and CPU: macOS arm64/Intel and Linux arm64/x64 (static musl builds). Windows is not served by Homebrew.
- Verifies the download against the `sha256` in the formula.
- Installs `artifact-cleaner` and `afc` into Homebrew's `bin`. Nothing is compiled.
- `brew test` runs `afc --version`.

Archive layout is `artifact-cleaner-v<version>-<target>/{artifact-cleaner,afc,README.md,LICENSE}`. Homebrew strips the single top-level folder, so the formula installs `afc` by bare name.

## The generator

`packaging/homebrew/generate-formula.sh <version> <output-file>` reads the four `.sha256` files from the GitHub release and writes the formula, so checksums are never pasted by hand.

- Writes to `<output>.tmp` and moves it into place only on success. A shell redirect (`> file`) would leave an empty formula if the script failed.
- Logs go to stderr. Set `QUIET=1` to silence them.
- `BASE_URL=file:///path/to/assets` tests against local files.
- Each download has a 30 second timeout.
- Output is kept clean for `brew audit --strict`: description under 80 characters, no redundant `version` line.

If Homebrew's style rules complain, fix the generator, not just the generated file, so every future release is clean.

## Release flow

See [RELEASING.md](../RELEASING.md).

## Problems we hit

| Symptom | Cause | Fix |
|---|---|---|
| Pushing a tag skipped every release job | `validate-version` only runs on manual dispatch. Without `!cancelled()`, jobs depending on a skipped job are skipped too. Earlier releases only worked via dispatch. | Added `!cancelled()` to `prepare-release` and the three build jobs |
| `brew style`: description too long | Over 80 characters | Shortened it in the generator |
| `afc --version` shows an old version after install | An older copy earlier on `PATH` (`~/.cargo/bin`, `~/.local/bin`) | `which -a afc`, remove the stale copy, `hash -r` |
| "Refusing to load formula from untrusted tap" | Homebrew 7 trust setting on the user's machine | `brew trust cloudengine-labs/tap` |
| Generator hung once on a checksum download | Network stall with no timeout | Added `curl -m 30` |

## Not verified

- The Intel macOS and both Linux archives are published and in the formula, but have not been installed or run. Only Apple Silicon was tested.
- A Mac that has never trusted the tap may show a trust prompt.
- Tap updates are manual. Automating them needs a token with write access to the tap repo, stored as a repository secret.
