# Rust Learning, Part 3: Production Patterns in a Real CLI

After you understand Rust's syntax, ownership, and error handling, the next step is learning how Rust code is organized in production. This article uses CLI development as the example because command-line tools expose many practical Rust patterns: argument parsing, filesystem traversal, formatting, error handling, dependency management, and testing.

The comparison with Go, Python, and Java matters because each ecosystem has a different instinct:

- Go favors small binaries, simple deployment, and explicit errors.
- Python favors fast iteration and rich libraries.
- Java favors structured architecture and mature operational tooling.
- Rust aims for small binaries, strong compile-time safety, explicit errors, and predictable performance.

## 1. Keep Modules Small and Purpose-Driven

A Rust CLI commonly starts with a simple module split:

```text
src/
|-- main.rs
|-- scanner.rs
|-- cleaner.rs
`-- display.rs
```

Each module should have one job:

| Module | Responsibility |
|---|---|
| `main.rs` | parse CLI arguments and coordinate the flow |
| `scanner.rs` | inspect the filesystem and produce structured results |
| `cleaner.rs` | delete selected artifacts and report deletion results |
| `display.rs` | format output for humans |

Comparison:

| Rust | Go | Python | Java |
|---|---|---|---|
| file modules declared with `mod` | packages by directory | modules by file | packages and classes |
| private by default | lowercase private | convention-based privacy | explicit visibility modifiers |
| `pub` exposes APIs | capitalized names expose APIs | `_name` suggests private | `public` exposes APIs |

Rust rewards narrow module boundaries. If a function does not need to be called from another module, keep it private.

## 2. Use Structs for Data Crossing Module Boundaries

When data leaves one module and enters another, make the shape explicit.

```rust
#[derive(Debug, Clone)]
pub struct ArtifactFolder {
    pub path: std::path::PathBuf,
    pub artifact_type: String,
    pub project: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub last_modified: String,
}
```

This is more maintainable than passing loosely related tuples or multiple parallel vectors.

In other languages:

- Go: use a struct.
- Python: use a dataclass or typed dict.
- Java: use a record or class.
- Rust: use a struct, derive traits for common behavior, and control field visibility.

Derives are important:

```rust
#[derive(Debug, Clone)]
```

`Debug` enables developer formatting. `Clone` enables explicit deep copying. Add derives only when needed.

## 3. Prefer Iterators for Data Pipelines

Rust iterators can express filesystem and collection processing clearly without allocating intermediate lists.

```rust
fn total_size(paths: &[std::path::PathBuf]) -> u64 {
    paths
        .iter()
        .map(|path| std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0))
        .sum()
}
```

Language comparison:

| Task | Rust | Go | Python | Java |
|---|---|---|---|---|
| Transform | `.map(...)` | loop | generator/map | stream `.map(...)` |
| Filter | `.filter(...)` | loop | comprehension/filter | stream `.filter(...)` |
| Sum | `.sum()` | loop accumulator | `sum(...)` | stream terminal op |
| Skip failed values | `.filter_map(...)` | `if err == nil` | try/except or guard | stream plus exception handling |

Rust iterator chains are lazy. Nothing executes until a terminal operation such as `.sum()`, `.collect()`, `.count()`, or `.for_each()`.

Use a loop when it is clearer. Production Rust does not require forcing every operation into an iterator chain.

## 4. Choose Dependencies Conservatively

Rust dependencies are called crates. They live in `Cargo.toml`.

```toml
[dependencies]
walkdir = "2"
clap = { version = "4", features = ["derive"] }
chrono = "0.4"
```

Good CLI crates:

| Crate | Use |
|---|---|
| `clap` | argument parsing |
| `walkdir` | recursive directory traversal |
| `colored` | terminal colors |
| `indicatif` | progress bars |
| `dialoguer` | interactive prompts |
| `humansize` | human-readable byte sizes |

Comparison:

| Rust | Go | Python | Java |
|---|---|---|---|
| crates.io | pkg.go.dev / modules | PyPI | Maven Central |
| Cargo features | build tags are different | extras | profiles/dependency scopes |
| lock file common for apps | `go.sum` | tool-specific | tool-specific |

Rust crates can expose compile-time features. This keeps optional functionality out of your binary unless enabled.

## 5. Design CLI Errors for Users and Developers

CLI tools have two audiences:

- Users need clear messages.
- Developers need enough context to debug.

For application code, returning `Result` keeps failures explicit.

```rust
fn delete_path(path: &std::path::Path) -> Result<(), std::io::Error> {
    std::fs::remove_dir_all(path)
}
```

At the CLI boundary, convert errors into user-facing messages:

```rust
match delete_path(path) {
    Ok(()) => println!("deleted {}", path.display()),
    Err(error) => eprintln!("failed to delete {}: {}", path.display(), error),
}
```

Comparison:

| Language | Common style |
|---|---|
| Rust | return `Result`, handle at boundary |
| Go | return `error`, handle explicitly |
| Python | raise exceptions, catch at boundary |
| Java | throw/catch exceptions or use result-like types |

Rust and Go feel similar here, but Rust makes success and failure part of the return type.

## 6. Treat Destructive Actions as Explicit Workflows

For a cleaner tool, deletion must be safe by default. A strong workflow is:

1. Scan and collect candidates.
2. Print what will be removed.
3. Require confirmation for destructive cleanup.
4. Support `--dry-run`.
5. Support `--yes` only for scripts and CI.

Rust helps because you can model each step with concrete types.

```rust
pub struct DeleteResult {
    pub deleted: usize,
    pub failed: Vec<String>,
    pub total_freed_bytes: u64,
}
```

This is better than returning only a boolean. It gives the caller enough information to print a useful summary and choose an exit code.

## 7. Keep Ownership Simple at API Boundaries

A practical API design rule:

| Function need | Rust parameter style |
|---|---|
| Read one path | `&Path` |
| Store a path | `PathBuf` |
| Read many items | `&[T]` |
| Consume many items | `Vec<T>` |
| Read text | `&str` |
| Store text | `String` |

Example:

```rust
use std::path::{Path, PathBuf};

fn scan_workspace(path: &Path) -> Vec<PathBuf> {
    vec![path.join("target")]
}

fn delete_artifacts(paths: Vec<PathBuf>) {
    for path in paths {
        println!("delete {}", path.display());
    }
}
```

The scanner borrows the workspace path and returns owned artifact paths. The cleaner consumes those paths.

That is idiomatic Rust because ownership follows the workflow.

## 8. Test Pure Logic First

Rust tests can live beside the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_target_folder() {
        assert!(is_artifact_type("target", &["target".to_string()]));
    }
}
```

Testing comparison:

| Rust | Go | Python | Java |
|---|---|---|---|
| `#[test]` | `func TestXxx` | `pytest` / `unittest` | JUnit |
| `cargo test` | `go test` | `pytest` | Maven/Gradle test |
| tests can live in same file | tests usually separate `_test.go` | flexible | usually separate test tree |

For CLI tools, start with tests for pure functions:

- artifact type matching
- path exclusion rules
- size formatting
- stale threshold calculation
- argument parsing edge cases

Then add integration tests for filesystem behavior.

## 9. Build Release Binaries Intentionally

Rust debug builds are optimized for fast compilation. Release builds are optimized for speed and binary size.

```bash
cargo build
cargo build --release
```

Release profile settings live in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

This is one reason Rust is strong for CLI distribution: you can ship a single binary with no runtime dependency.

Comparison:

| Language | Typical CLI distribution |
|---|---|
| Rust | single native binary |
| Go | single native binary |
| Python | script plus interpreter/environment, or packaged executable |
| Java | JAR plus JVM, or native image with extra tooling |

## 10. Production Rust Habits

Use these habits when writing real Rust:

1. Borrow first; own only when necessary.
2. Keep `clone()` visible and intentional.
3. Avoid `unwrap()` in normal runtime paths.
4. Return `Result` from fallible functions.
5. Keep module APIs small.
6. Use structs for data crossing boundaries.
7. Prefer standard library types before adding crates.
8. Add tests for logic before testing terminal output.
9. Run `cargo fmt`, `cargo clippy`, and `cargo test` before release.

These habits map well from other languages:

- From Go, keep explicit error handling and simple deployment.
- From Python, keep readable transformations and fast feedback loops.
- From Java, keep clear boundaries and stable data models.
- From Rust, add ownership discipline and compile-time safety.

## Practice Tasks

Use `artifact-cleaner` as the practice codebase:

1. Read `src/main.rs` and write the high-level CLI workflow in five steps.
2. Read `src/scanner.rs` and identify which functions are pure enough to test easily.
3. Read `src/cleaner.rs` and find where filesystem errors are collected.
4. Read `src/display.rs` and identify which functions are formatting-only.
5. Pick one function signature and explain why each parameter is owned or borrowed.

## Series Summary

Across the three parts, the learning path is:

1. Learn Rust's foundations: Cargo, variables, functions, structs, enums, and modules.
2. Learn the ownership model: moves, borrowing, lifetimes, `Option`, and `Result`.
3. Learn production patterns: module boundaries, iterators, dependencies, tests, CLI safety, and release builds.

The biggest shift from Go, Python, and Java is that Rust moves many runtime questions into compile-time checks. Once that becomes familiar, Rust stops feeling like a restrictive language and starts feeling like a precise one.
