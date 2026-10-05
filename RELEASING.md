# Releasing

How the Homebrew side works: [docs/HOMEBREW.md](./docs/HOMEBREW.md).

## Steps

1. Bump `version` in `Cargo.toml` (minor bump for any code change) and merge to `main` through a PR.
2. Tag and push:
   ```sh
   git checkout main && git pull
   git tag vX.Y.Z && git push origin vX.Y.Z
   ```
   The Release workflow builds all archives and `.sha256` files and creates the GitHub Release.
   If the tag push does not start it, run it manually:
   `gh workflow run release.yml --ref main -f version=vX.Y.Z`
3. Wait for it: `gh run watch` (13 assets expected: 6 archives, 6 checksums, `checksums.txt`).
4. Regenerate the formula and push the tap:
   ```sh
   git clone git@github.com:cloudengine-labs/homebrew-tap.git ../homebrew-tap   # first time only
   cd ../homebrew-tap && git pull && cd -
   packaging/homebrew/generate-formula.sh X.Y.Z ../homebrew-tap/Formula/artifact-cleaner.rb
   cd ../homebrew-tap
   brew style Formula/artifact-cleaner.rb
   git add . && git commit -m "feat(formula): artifact-cleaner X.Y.Z" && git push
   ```
5. Verify:
   ```sh
   brew update
   brew audit --strict --online cloudengine-labs/tap/artifact-cleaner
   brew upgrade cloudengine-labs/tap/artifact-cleaner
   afc --version      # must print X.Y.Z; if not, run `which -a afc` for a stale copy
   brew test cloudengine-labs/tap/artifact-cleaner
   ```

The git tag, the `Cargo.toml` version, the archive names and the formula URLs must all agree.

Tap repo: https://github.com/cloudengine-labs/homebrew-tap
