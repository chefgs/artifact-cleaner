# Contributing

Thanks for helping improve `artifact-cleaner`.

## Development setup

Install Rust using `rustup`, then verify the project:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo check --all-targets
```

## Making changes

- Keep changes focused and small.
- Update documentation when behavior changes.
- For code changes, bump the minor version in `Cargo.toml` and keep `Cargo.lock` in sync.
- Run the checks above before opening a pull request.

## Releases

Releases are created from tags like `v0.7.0`. The release workflow builds and uploads platform binaries for macOS, Linux, and Windows.
