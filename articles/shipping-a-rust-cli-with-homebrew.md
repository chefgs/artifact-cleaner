# Shipping `artifact-cleaner` with `brew install`: one day, four releases, and the release bugs we found

*A recap of publishing the Rust CLI `afc` / `artifact-cleaner` through the CloudEngine Labs Homebrew tap. Everything below happened in one working session, including the mistakes. It follows the earlier `sbom-sentinel` tap write-up; read that first for the general mechanics.*

---

## TL;DR

```sh
brew install cloudengine-labs/tap/artifact-cleaner
afc --version        # afc 0.14.1
```

What it took:

1. A **formula generator script** that turns the release's `.sha256` files into a Homebrew formula.
2. **One new file** in the existing tap: `Formula/artifact-cleaner.rb`.
3. Two **release workflow bugs** that only showed up once we pushed a real tag.
4. A friendlier **`--help`**, and docs for the whole flow.

Releases: `0.12.0` (formula only, nothing rebuilt) → `0.13.0` → `0.14.0` → `0.14.1`.

---

## 1. Starting point

We already had a public tap, `cloudengine-labs/homebrew-tap`, with one formula (`sbom-sentinel`). The `artifact-cleaner` repo already built and published archives for six targets on every release:

| Platform | Archive |
|---|---|
| macOS Apple Silicon / Intel | `aarch64-apple-darwin`, `x86_64-apple-darwin` (`.tar.gz`) |
| Linux arm64 / x64 (static musl) | `aarch64-unknown-linux-musl`, `x86_64-unknown-linux-musl` (`.tar.gz`) |
| Windows arm64 / x64 | `aarch64-pc-windows-msvc`, `x86_64-pc-windows-msvc` (`.zip`) |

Each has a `.sha256` file next to it.

The `sbom-sentinel` guide assumed a **private** source repo, which forced it to copy archives onto the tap's own Releases. `artifact-cleaner` is public, so that whole step disappears: the formula points straight at the project's GitHub Releases.

Differences from the `sbom-sentinel` setup that mattered:

- Archive names include a `v`: `artifact-cleaner-v0.12.0-<target>.tar.gz`.
- Each archive wraps its files in a top-level folder, and ships **two** binaries (`artifact-cleaner` and the short alias `afc`).
- No external tools to depend on, so no `depends_on` lines.
- License was MIT in both `LICENSE` and `Cargo.toml`, so nothing to reconcile.

## 2. Ship the formula first, on the existing release

We deliberately did the formula **before** touching any code, using the already-published `v0.12.0`. That proved the whole pipeline (generate, style, audit, install, test) without a new release. If the formula had been wrong, there would have been nothing to undo in the repo.

### The generator

`packaging/homebrew/generate-formula.sh <version> <output-file>` fetches the four `.sha256` files (macOS and Linux, two CPUs each) and writes the formula. Design points carried over from the earlier guide:

- Writes to `<output>.tmp` and `mv`s on success. A plain `script > file` truncates the file first, so a failed run leaves a 0-byte formula.
- Logs on stderr, `QUIET=1` silences them.
- `BASE_URL` override for testing against local files.

Added here: a `curl -m 30` timeout, after the first run stalled on a download and never recovered.

### The formula

```ruby
class ArtifactCleaner < Formula
  desc "Find and remove stale build artifacts like node_modules, dist and .terraform"
  homepage "https://github.com/chefgs/artifact-cleaner"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/chefgs/artifact-cleaner/releases/download/v0.14.1/artifact-cleaner-v0.14.1-aarch64-apple-darwin.tar.gz"
      sha256 "<from the .sha256 file>"
    end
    on_intel do
      url "…/artifact-cleaner-v0.14.1-x86_64-apple-darwin.tar.gz"
      sha256 "…"
    end
  end

  on_linux do
    on_arm do
      url "…/artifact-cleaner-v0.14.1-aarch64-unknown-linux-musl.tar.gz"
      sha256 "…"
    end
    on_intel do
      url "…/artifact-cleaner-v0.14.1-x86_64-unknown-linux-musl.tar.gz"
      sha256 "…"
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
```

Notes:

- We added `on_linux` blocks because the static musl archives already existed. Homebrew runs on Linux, so it cost nothing.
- Homebrew strips the single top-level folder inside the archive, so `bin.install "afc"` works with a bare name.
- Homebrew reads the version from the file name (`…-v0.12.0-…`), so there is no `version` line. Adding one is flagged by `brew audit`.

### Style fix

`brew style` failed on the first generated file: **"Description is too long. It should be less than 80 characters"** (ours was 84). We fixed the description in the *generator*, not the generated file, so every future release is clean.

### Verify

```sh
brew style Formula/artifact-cleaner.rb
brew install cloudengine-labs/tap/artifact-cleaner
brew test cloudengine-labs/tap/artifact-cleaner
brew audit --strict --online cloudengine-labs/tap/artifact-cleaner
```

All passed on `0.12.0`. We then updated the tap README with the install line and the `brew trust` note.

## 3. Release 0.13.0, and the first release bug

The repo-side changes went in through a PR (branch `feature/homebrew-tap`, merged by hand):

- Added `repository` and `homepage` to `Cargo.toml`.
- Bundled `LICENSE` in every release archive (macOS/Linux and Windows).
- Version bump to `0.13.0`, the repo's rule for any change.
- The generator, a `RELEASING.md` checklist, and a Homebrew section in the README.

Then we pushed the tag `v0.13.0`. **Every job in the run was skipped.**

### Root cause

`validate-version` only runs on manual dispatch, so on a tag push it is *skipped*. Every job that listed it in `needs:` had an `if:` like:

```yaml
if: needs.validate-version.result == 'success' || needs.validate-version.result == 'skipped'
```

That looks right, but without `!cancelled()` (or `always()`), GitHub adds an implicit `success()` check, and a skipped dependency fails it. So the job was skipped even though the condition spells out "skipped is fine".

Earlier releases (`v0.11.0`, `v0.12.0`) only worked because they had been started by manual dispatch, where `validate-version` actually runs. The tag-push path had been broken since an earlier hardening change.

### Workaround and fix

We released `v0.13.0` through manual dispatch, since the tag pointed at the same commit as `main`:

```sh
gh workflow run release.yml --ref main -f version=v0.13.0
```

Then fixed the workflow in a separate PR: prefix the `if:` of `prepare-release` and the three build jobs with `${{ !cancelled() && … }}`.

## 4. Release 0.14.0: a friendlier `--help`

The `sbom-sentinel` CLI has a quick-start block at the bottom of `--help`. We applied the same pattern to `afc`:

- `afc --help` now ends with **QUICK START**, **COMMON TASKS** and **GOOD TO KNOW** sections.
- `scan`, `size` and `mac-lib` each get **WHAT IT DOES / EXAMPLES / NOTES**.
- Bare `afc` prints help instead of an error (`arg_required_else_help`).
- The examples lead with `--dry-run`, because `scan` deletes after a y/N prompt.

The text is plain `const` strings in `src/main.rs`, wired in with `after_help` and `after_long_help`. No behaviour changed; we ran the examples against a scratch directory to check they match the real flags. We checked the `mac-lib` description against its source and added that oversized caches of *installed* apps are shown only as caution items.

## 5. The second release bug: `checksums.txt`

Pushing `v0.14.0` now started the release on its own (the first fix worked), but the release had **12 assets instead of 13**. The `Generate checksums.txt` and `Release complete` jobs were skipped. They sit one level further down the dependency chain and were not covered by the first fix.

What did that break?

| Installer | Checks | Impact |
|---|---|---|
| Homebrew formula | per-archive `.sha256` | none |
| `install.sh` (macOS/Linux) | `checksums.txt`, falls back to `.sha256` | none |
| `install.ps1` (Windows) | `checksums.txt` only | **fails** until the file exists |

Two fixes:

1. **Fix the workflow** (`!cancelled()` plus explicit `needs.*.result == 'success'` checks on the last two jobs).
2. **Add the missing file to `v0.14.0` without rebuilding.** Re-running the full workflow would rebuild and re-upload the archives. Rebuilds are rarely byte-identical, and the formula's checksums would then not match what users download. The job only runs `cat *.sha256 | sort`, so we reproduced it from the files already on the release:

```sh
gh release download v0.14.0 --pattern '*.sha256' --dir chk
cat chk/*.sha256 | sort > chk/checksums.txt
gh release upload v0.14.0 chk/checksums.txt --clobber
```

Then we confirmed the public download matched the local file. (The first download right after the upload returned 404; a retry a few seconds later succeeded.)

## 6. Release 0.14.1: the real test

To harden Windows too, `install.ps1` now falls back to the archive's own `.sha256` when `checksums.txt` is missing, the same way `install.sh` does. That is a real change worth a release, and it needs a new version anyway: **never move a published tag.** We bumped to `0.14.1`, merged, and pushed `v0.14.1`.

Result: every job ran, including `Generate checksums.txt` and `Release complete`, with no manual dispatch and **13 assets**. Both workflow fixes are confirmed.

Then the tap update, with the generator:

```sh
packaging/homebrew/generate-formula.sh 0.14.1 ../homebrew-tap/Formula/artifact-cleaner.rb
cd ../homebrew-tap
brew style Formula/artifact-cleaner.rb
git add . && git commit -m "feat(formula): artifact-cleaner 0.14.1" && git push
```

Verified on this Mac: `brew audit --strict --online` clean, `brew test` passed, upgrade from 0.14.0, a clean uninstall/untap/reinstall, and `install.sh` into a temporary directory. `install.ps1`, the Intel macOS build and the Linux builds were not run.

## 7. Troubleshooting table

| Symptom | Cause | Fix |
|---|---|---|
| Tag push skipped every release job | `validate-version` skipped on tag pushes; dependents lack `!cancelled()` | `${{ !cancelled() && … }}` on every job downstream of a conditional job |
| `checksums.txt` missing, run still "success" | Last two jobs skipped for the same reason; skipped jobs don't fail a run | Same fix; always count assets (expect 13) |
| Windows install fails | `install.ps1` required `checksums.txt` | `.sha256` fallback, as `install.sh` |
| `brew style`: description too long | 84 characters; the limit is 80 | Fix it in the generator |
| Generator hung on a download | No timeout on `curl` | `curl -m 30` |
| `afc --version` shows an old version after install | An older copy earlier on `PATH` (`~/.cargo/bin` from `cargo install`) | `which -a afc`, `cargo uninstall artifact-cleaner`, `hash -r` |
| Brew warns the tap is untrusted | Homebrew 7 tap trust | `brew trust cloudengine-labs/tap` |

## 8. Lessons

- **Prove the packaging on an existing release first.** It separates "is the formula right?" from "does the release pipeline work?".
- **Fix the generator, not the output.** Style fixes belong in the script so they stick.
- **A conditional job upstream affects every job downstream.** If a job can be skipped, every dependent needs `!cancelled()` and an explicit result check.
- **"Success" is not "complete".** Skipped jobs don't fail a workflow run. Check the asset count.
- **Don't rebuild a published release to add one file.** If archive checksums are in a formula, regenerate the missing file from what's already published.
- **Never move a published tag.** Add a new version instead.
- **Check `which -a`.** A local dev install can shadow the Homebrew one and look like a failed upgrade.
- **Keep install paths symmetric.** `install.sh` and `install.ps1` should accept the same checksum sources.

## 9. What is still not verified

- The **Intel macOS** archive and both **Linux** archives are published and in the formula, but were never installed or run. Only Apple Silicon was tested.
- `install.ps1` has not been run on Windows.
- A Mac that has never trusted the tap may show a trust prompt.
- Tap updates are manual. Automating them needs a token with write access to the tap repo, stored as a repository secret.

## 10. Release checklist (current)

```sh
# 1. Bump version in Cargo.toml, merge via PR, then:
git checkout main && git pull
git tag vX.Y.Z && git push origin vX.Y.Z      # should start the Release workflow
gh run watch                                   # expect 13 assets (6 archives, 6 .sha256, checksums.txt)

# 2. Update the tap
packaging/homebrew/generate-formula.sh X.Y.Z ../homebrew-tap/Formula/artifact-cleaner.rb
cd ../homebrew-tap && brew style Formula/artifact-cleaner.rb
git add . && git commit -m "feat(formula): artifact-cleaner X.Y.Z" && git push

# 3. Verify
brew update
brew audit --strict --online cloudengine-labs/tap/artifact-cleaner
brew upgrade cloudengine-labs/tap/artifact-cleaner
afc --version && brew test cloudengine-labs/tap/artifact-cleaner
```

The four things that must always agree: **the git tag, the `Cargo.toml` version, the archive names, and the formula URLs.**

---

*The repo-side docs for all of this live in [`docs/HOMEBREW.md`](../docs/HOMEBREW.md) and [`RELEASING.md`](../RELEASING.md).*
