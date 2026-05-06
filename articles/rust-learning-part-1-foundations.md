# Rust Learning, Part 1: Foundations for Go, Python, and Java Developers

Rust feels unfamiliar at first because it combines ideas that are usually split across other languages: the low-level control of C, the package discipline of Go, the expressive types of functional languages, and the safety expectations of managed runtimes.

If you already know Go, Python, or Java, the fastest way to learn Rust is not to translate syntax line by line. The better path is to notice where Rust asks a different question.

- Go asks: is this simple enough to maintain?
- Python asks: is this clear enough to express quickly?
- Java asks: is this structured enough to scale across teams?
- Rust asks: can the compiler prove this is safe and efficient?

This article covers the foundation: project layout, variables, types, functions, structs, enums, and modules.

## 1. Cargo Is Rust's Project System

Rust projects are built around Cargo. Cargo is similar to `go` tooling, Maven/Gradle, and Python packaging tools, but in one standard workflow.

```text
artifact-cleaner/
|-- Cargo.toml
|-- Cargo.lock
`-- src/
    |-- main.rs
    |-- scanner.rs
    |-- cleaner.rs
    `-- display.rs
```

Language comparison:

| Concept | Rust | Go | Python | Java |
|---|---|---|---|---|
| Package manifest | `Cargo.toml` | `go.mod` | `pyproject.toml` / `requirements.txt` | `pom.xml` / `build.gradle` |
| Lock file | `Cargo.lock` | `go.sum` | lock files vary by tool | Gradle/Maven lock support varies |
| Build command | `cargo build` | `go build` | tool-specific | `mvn package` / `gradle build` |
| Run tests | `cargo test` | `go test` | `pytest` / `unittest` | JUnit via Maven/Gradle |
| Entry point | `src/main.rs` | `main.go` | script/module entry point | `public static void main` |

Cargo gives Rust a strong convention: most Rust projects look familiar once you know the standard layout.

## 2. Variables Are Immutable by Default

In Go, Python, and Java, local variables are mutable unless you opt into restrictions.

In Rust, local bindings are immutable unless you write `mut`.

```rust
let months = 2;
let mut total_bytes = 0_u64;

total_bytes += 1024;
```

Compare that with the other languages:

```go
months := 2
totalBytes := uint64(0)
totalBytes += 1024
```

```python
months = 2
total_bytes = 0
total_bytes += 1024
```

```java
int months = 2;
long totalBytes = 0L;
totalBytes += 1024;
```

Rust's default immutability changes how you design code. You make mutation visible. When a Rust function has `let mut`, it is a clear signal that state changes inside that scope.

Use this rule:

- Start with `let`.
- Add `mut` only when the variable must change.
- Prefer creating a new value when mutation makes the code harder to follow.

## 3. Types Are Static, but Usually Inferred

Rust is statically typed like Go and Java. It often feels lighter because the compiler infers local types.

```rust
let path = std::path::PathBuf::from(".");
let months = 2_u32;
let artifact_types = vec!["node_modules".to_string(), "target".to_string()];
```

You can add explicit annotations when they improve clarity or when the compiler needs help:

```rust
let mut results: Vec<ArtifactFolder> = Vec::new();
let size_bytes: u64 = 0;
```

Type comparison:

| Concept | Rust | Go | Python | Java |
|---|---|---|---|---|
| Signed integer | `i32`, `i64` | `int32`, `int64` | `int` | `int`, `long` |
| Unsigned integer | `u32`, `u64` | `uint32`, `uint64` | no fixed unsigned int | no native unsigned general-purpose int |
| Boolean | `bool` | `bool` | `bool` | `boolean` |
| Owned string | `String` | `string` | `str` | `String` |
| Borrowed string view | `&str` | no direct equivalent | no direct equivalent | no direct equivalent |
| Dynamic array | `Vec<T>` | `[]T` | `list` | `ArrayList<T>` |
| Map | `HashMap<K, V>` | `map[K]V` | `dict` | `HashMap<K, V>` |

The important Rust distinction is owned versus borrowed data. `String` owns text. `&str` is a view into text owned somewhere else.

```rust
fn print_label(label: &str) {
    println!("{}", label);
}

let owned = String::from("target");
print_label(&owned);
print_label("node_modules");
```

For function parameters, prefer borrowed views such as `&str`, `&Path`, and `&[T]` when the function only needs to read.

## 4. Functions Return Values by Expression

Rust functions name parameter types and return types explicitly:

```rust
fn is_stale(age_months: u32, threshold_months: u32) -> bool {
    age_months >= threshold_months
}
```

The final expression is returned when it has no semicolon.

```rust
fn double(x: i32) -> i32 {
    x * 2
}
```

A semicolon turns an expression into a statement:

```rust
fn broken_double(x: i32) -> i32 {
    x * 2;
}
```

That does not return `i32`; it returns `()`, Rust's unit type. This is one of the first errors beginners hit.

Comparison:

| Concept | Rust | Go | Python | Java |
|---|---|---|---|---|
| Function keyword | `fn` | `func` | `def` | method declaration |
| Return type position | after `->` | after parameter list | optional annotation | before method name |
| No-value return | `()` | no return values | `None` | `void` |
| Public function | `pub fn` | capitalized name | convention/module export | `public` |

## 5. Structs Replace Classes for Data

Rust does not have classes or inheritance. It has structs for data and `impl` blocks for methods.

```rust
#[derive(Debug, Clone)]
pub struct ArtifactFolder {
    pub path: std::path::PathBuf,
    pub artifact_type: String,
    pub project: String,
    pub size_bytes: u64,
}
```

That maps roughly to:

```go
type ArtifactFolder struct {
    Path         string
    ArtifactType string
    Project      string
    SizeBytes    uint64
}
```

```python
from dataclasses import dataclass

@dataclass
class ArtifactFolder:
    path: str
    artifact_type: str
    project: str
    size_bytes: int
```

```java
public record ArtifactFolder(
    Path path,
    String artifactType,
    String project,
    long sizeBytes
) {}
```

Methods live in `impl` blocks:

```rust
impl ArtifactFolder {
    pub fn is_large(&self) -> bool {
        self.size_bytes > 100 * 1024 * 1024
    }
}
```

`&self` means the method borrows the struct immutably. `&mut self` means it needs mutable access. `self` means it consumes the value.

## 6. Enums Carry Data

Rust enums are much more powerful than enum constants in Go or Java.

```rust
enum ScanOutcome {
    Found(ArtifactFolder),
    Skipped(String),
    Failed(std::io::Error),
}
```

Each variant can hold different data. This is the foundation for Rust's two most important standard types:

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

In Python you may use `None` or exceptions. In Go you may return `(value, error)`. In Java you may use `null`, `Optional<T>`, or exceptions. Rust makes absence and failure explicit in the type system.

## 7. Modules Are Explicit

Rust does not auto-discover source files. You declare modules from the crate root.

```rust
mod scanner;
mod cleaner;
mod display;
```

Then import items:

```rust
use crate::scanner::ArtifactFolder;
use std::path::PathBuf;
```

Visibility is also explicit:

```rust
pub fn scan_workspace() {}
fn helper_only_for_this_module() {}
```

Comparison:

| Concept | Rust | Go | Python | Java |
|---|---|---|---|---|
| Module/file declaration | `mod scanner;` | package by directory | import by file/module | package declaration |
| Public item | `pub` | capitalized name | convention / `__all__` | `public` |
| Private item | default | lowercase name | convention with `_` | `private` / package-private |

Rust's default is private. You expose only what other modules need.

## Practice Tasks

Use the `artifact-cleaner` source code while practicing:

1. Open `src/scanner.rs` and identify every `let mut`.
2. Find a function that accepts `&Path` and explain why it does not take `PathBuf`.
3. Find a struct and map it to a Go struct, Python dataclass, and Java record.
4. Find a `pub` item and explain which module needs to call it.

## Part 1 Summary

Rust's surface syntax is not the hard part. The hard part is its defaults:

- Values are immutable unless marked `mut`.
- Types are static, but local inference keeps code concise.
- Structs and methods replace class-based design.
- Enums model alternatives directly.
- Modules and visibility are explicit.

Part 2 builds on this foundation with Rust's central idea: ownership, borrowing, lifetimes, `Option`, and `Result`.
