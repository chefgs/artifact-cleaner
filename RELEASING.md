# Releasing

1. Bump `version` in `Cargo.toml`, merge to `main`.
2. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z` — the Release workflow builds all archives and `.sha256` files.
3. Regenerate the Homebrew formula and push the tap:
   ```sh
   packaging/homebrew/generate-formula.sh X.Y.Z ../homebrew-tap/Formula/artifact-cleaner.rb
   cd ../homebrew-tap && git add . && git commit -m "feat(formula): artifact-cleaner X.Y.Z" && git push
   ```
4. Verify: `brew update && brew audit --strict --online cloudengine-labs/tap/artifact-cleaner && brew upgrade artifact-cleaner && afc --version`

The git tag, `Cargo.toml` version, archive names and formula URLs must agree.
Tap repo: https://github.com/cloudengine-labs/homebrew-tap
