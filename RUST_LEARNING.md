# Rust for Go, Python & Java Developers
### Lessons from building `artifact-cleaner` and `ac`

> Every concept is explained using real code from this project — not toy examples.
> Read it alongside the source files in `src/`.
>
> **Quick reference:** See [LESSONS.md](./LESSONS.md) for an indexed map of all lesson blocks in the source.

---

## Table of Contents

0. [Before You Start — complete beginners start here](#0-before-you-start)
1. [The Big Picture — Why Rust is different](#1-the-big-picture)
2. [Project layout & the build system](#2-project-layout--the-build-system)
3. [Variables, types, and mutability](#3-variables-types-and-mutability)
4. [Ownership — Rust's core idea](#4-ownership--rusts-core-idea)
5. [Borrowing & references](#5-borrowing--references)
6. [Structs — Rust's version of classes](#6-structs)
7. [Enums — far more powerful than Go/Python enums](#7-enums)
8. [Option\<T\> — no null, no None surprises](#8-optiont--no-null-no-none-surprises)
9. [Result\<T, E\> — error handling without exceptions](#9-resultt-e--error-handling-without-exceptions)
10. [Pattern matching with `match`](#10-pattern-matching-with-match)
11. [Functions and implicit return](#11-functions-and-implicit-return)
12. [Closures](#12-closures)
13. [Iterators — functional data pipelines](#13-iterators--functional-data-pipelines)
14. [Modules and visibility](#14-modules-and-visibility)
15. [Traits — Rust's interfaces](#15-traits--rusts-interfaces)
16. [Macros — code that writes code](#16-macros--code-that-writes-code)
17. [The standard library you'll use daily](#17-the-standard-library-youll-use-daily)
18. [External crates (dependencies)](#18-external-crates-dependencies)
19. [Memory model — stack vs heap](#19-memory-model--stack-vs-heap)
20. [Common beginner mistakes](#20-common-beginner-mistakes)
21. [Cheat sheet — Go/Python vs Rust](#21-cheat-sheet--gopython-vs-rust)

---

## 0. Before You Start

> **This section is for complete beginners.** If you already know Go, Python, or another compiled language, skip to Section 1.

### What is a compiled language?

Python and JavaScript run code by interpreting it line by line. Rust is different — before your program can run, it must be **compiled**: translated from source code into a native binary your CPU executes directly.

```
Python:   source.py  →  interpreter reads + runs it (slow startup, no binary)
Go:       main.go    →  go build  →  binary  →  runs
Rust:     main.rs    →  cargo build  →  binary  →  runs (with more checks)
```

The Rust compiler (`rustc`) checks your entire program for type errors, memory errors, and logic problems **before** it produces a binary. If the code compiles, it almost certainly won't crash at runtime due to those categories of errors. This is the trade-off: longer compile times, faster and safer programs.

### Installing Rust

The official installer is `rustup`. It installs the compiler, the standard library, and Cargo (Rust's build tool) in one step.

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows — download rustup-init.exe from https://rustup.rs
```

After installation, open a new terminal and verify:

```bash
rustc --version    # e.g. rustc 1.82.0 (f6e511eec 2024-10-15)
cargo --version    # e.g. cargo 1.82.0 (8f40fc59f 2024-08-21)
```

### Running this project for the first time

```bash
# 1. Clone the repo
git clone https://github.com/chefgs/artifact-cleaner.git
cd artifact-cleaner

# 2. Build (debug mode — fast compile, slower binary)
cargo build

# 3. Run with --help to see what was built
cargo run -- --help
# or, after build:
./target/debug/artifact-cleaner --help

# 4. Try a dry run in your home directory
cargo run -- scan ~ --dry-run

# 5. Build optimised binary (slow compile, fast runtime)
cargo build --release
./target/release/ac mac-lib --dry-run
```

### What to expect from the compiler

The Rust compiler's error messages are the best of any language. When something is wrong, it tells you exactly what, why, and often how to fix it:

```
error[E0382]: borrow of moved value: `paths`
  --> src/main.rs:42:20
   |
40 |     let paths = get_paths();
   |         ----- move occurs because `paths` has type `Vec<PathBuf>`
41 |     delete(paths);    // paths moved here
   |            ----- value moved here
42 |     println!("{:?}", paths);  // ERROR
   |                      ^^^^^ value borrowed here after move
   |
help: consider cloning the value if the performance cost is acceptable
   |
41 |     delete(paths.clone());
   |                 ++++++++
```

Read every error message — they almost always tell you the fix. The `help:` and `note:` lines are especially useful.

### Compiler warnings are not optional

Unlike Python or Go, ignoring Rust warnings is bad practice. The compiler warns about unused variables, dead code, and patterns that could cause bugs. Fix warnings as they appear — they often point to real issues.

```bash
cargo check   # check for errors and warnings without producing a binary (fastest)
cargo clippy  # additional lint checks (install once with: rustup component add clippy)
```

Now continue to Section 1 to understand why Rust is fundamentally different from Python and Go.

---

## 1. The Big Picture

### Why does Rust exist?

Rust solves one problem: **memory safety without a garbage collector**.

| Language | Memory management | Runtime cost |
|----------|------------------|-------------|
| Python | Garbage collector (GC) | Pause unpredictable, high memory |
| Go | Garbage collector (GC) | Small pauses, good for servers |
| C/C++ | Manual (`malloc`/`free`) | Zero cost, but crashes and security bugs |
| **Rust** | **Compiler enforces ownership rules** | **Zero cost, no crashes** |

The Rust compiler is your pair programmer. It refuses to compile unsafe code.
This means if it compiles — it almost certainly won't crash at runtime due to memory issues.

### What does "ownership" mean in one sentence?
Every value in Rust has exactly one owner. When the owner goes out of scope, the value is freed. No GC needed.

### How does this compare?

```python
# Python — GC decides when to free memory, you never think about it
def scan():
    results = []          # GC tracks this list
    results.append(item)  # GC tracks item
    return results        # GC keeps it alive as long as needed
```

```go
// Go — GC also manages memory, escape analysis decides stack vs heap
func scan() []Result {
    results := []Result{}
    results = append(results, item)
    return results   // GC keeps alive
}
```

```rust
// Rust — compiler tracks ownership, no GC needed
fn scan() -> Vec<ArtifactFolder> {
    let mut results: Vec<ArtifactFolder> = Vec::new();
    results.push(item);
    results  // ownership transferred to caller — no GC, no copy
}
```

---

## 2. Project Layout & the Build System

### Cargo — Rust's all-in-one build tool

Cargo does everything: build, test, run, add dependencies, publish.
It is the best build tool of any language — no Makefile, no CMake, no setup.py.

```
artifact-cleaner/
├── Cargo.toml           ← package manifest (like package.json / go.mod)
├── Cargo.lock           ← exact locked versions (like package-lock.json / go.sum)
└── src/
    ├── main.rs          ← binary entry point; declares all top-level modules
    ├── scanner.rs       ← workspace scanner module
    ├── cleaner.rs       ← artifact deletion module
    ├── display.rs       ← terminal output module
    └── mac_library/     ← module directory (see below)
        ├── mod.rs       ← entry point for the mac_library module
        ├── resolver.rs  ← folder name classification
        ├── checker.rs   ← app/CLI install detection
        ├── scanner.rs   ← Library directory walker
        └── display.rs   ← output formatting for mac-lib
```

### Module directories — `src/module/mod.rs`

When a module grows beyond a single file, Rust lets you turn it into a **directory**. The directory's entry point is always `mod.rs`:

```
src/mac_library/mod.rs     ← Rust loads this when it sees `mod mac_library;`
src/mac_library/scanner.rs ← sub-module, declared inside mod.rs as `mod scanner;`
```

From `main.rs`, you declare the directory module exactly like a single-file module:
```rust
mod mac_library;  // Rust checks src/mac_library.rs, then src/mac_library/mod.rs
```

Inside `mod.rs`, you then declare its sub-modules:
```rust
mod checker;    // resolves to src/mac_library/checker.rs
mod display;    // resolves to src/mac_library/display.rs
mod resolver;   // resolves to src/mac_library/resolver.rs
mod scanner;    // resolves to src/mac_library/scanner.rs
```

Sub-modules navigate to each other with `super::` (one level up) instead of `crate::` (project root):
```rust
// Inside src/mac_library/scanner.rs — sibling modules
use super::checker;              // mac_library::checker
use super::resolver::EntryKind;  // mac_library::resolver::EntryKind
```

### Two binaries from one source file

This project builds two CLI commands (`artifact-cleaner` and `ac`) from a single `src/main.rs`. The trick is two `[[bin]]` entries in `Cargo.toml`:

```toml
[[bin]]
name = "artifact-cleaner"
path = "src/main.rs"

[[bin]]
name = "ac"
path = "src/main.rs"
```

Inside the code, `env!("CARGO_BIN_NAME")` resolves to whichever binary is being built, so each binary's `--help` self-identifies correctly:

```rust
#[command(name = env!("CARGO_BIN_NAME"), ...)]
struct Cli { ... }
// When built as "ac":       name = "ac"
// When built as "artifact-cleaner": name = "artifact-cleaner"
```

### Cargo commands

```bash
cargo build           # compile (debug, fast compile, large binary)
cargo build --release # compile (optimised, slow compile, small fast binary)
cargo run             # compile + run
cargo run -- --help   # compile + run with arguments (-- separates cargo args from program args)
cargo check           # type-check only — fastest way to catch errors
cargo test            # run tests
cargo add walkdir     # add a dependency (updates Cargo.toml automatically)
```

### Cargo.toml vs go.mod vs requirements.txt

```toml
# Cargo.toml
[package]
name = "artifact-cleaner"
version = "0.1.0"
edition = "2024"          # Rust edition — like Go's go directive

[dependencies]
walkdir = "2"             # semver — any 2.x.x
clap = { version = "4", features = ["derive"] }  # with feature flags
chrono = "0.4"
```

```go
// go.mod equivalent
module artifact-cleaner
go 1.21
require github.com/some/lib v2.0.0
```

```python
# requirements.txt equivalent
walkdir==2.5.0
clap==4.6.1
```

Key difference: Cargo has **features** — optional compile-time capabilities within a crate.
`clap = { features = ["derive"] }` enables the macro-based argument parsing we use in `main.rs`.

---

## 3. Variables, Types, and Mutability

### `let` — immutable by default

In Rust, variables are **immutable by default**. This is the opposite of Go/Python where everything is mutable.

```rust
// From scanner.rs
let cutoff = SystemTime::now()...;  // immutable — cannot reassign

let mut results: Vec<ArtifactFolder> = Vec::new();  // mut = mutable
results.push(item);  // allowed because results is mut
```

```go
// Go — everything mutable by default
cutoff := time.Now()  // can reassign anytime
results := []Result{}
```

```python
# Python — everything mutable by default
cutoff = datetime.now()
results = []
```

**Why immutable by default?** It prevents accidental mutation, makes code easier to reason about, and enables compiler optimisations.

### Type inference

Rust infers types like Go, but you can always annotate explicitly:

```rust
let size_bytes: u64 = 0;           // explicit
let size_bytes = compute_size(&p); // inferred from return type of compute_size()
let mut results: Vec<ArtifactFolder> = Vec::new();  // explicit (needed here — compiler can't infer T)
```

### Primitive types

| Rust | Go | Python | Notes |
|------|----|--------|-------|
| `i32` | `int32` | `int` | signed 32-bit |
| `u32` | `uint32` | `int` | unsigned 32-bit |
| `i64` | `int64` | `int` | signed 64-bit |
| `u64` | `uint64` | `int` | unsigned 64-bit — used for byte counts |
| `usize` | `int` | `int` | pointer-sized int — used for array indices |
| `f64` | `float64` | `float` | 64-bit float |
| `bool` | `bool` | `bool` | true/false |
| `&str` | `string` | `str` | borrowed string slice (read-only view) |
| `String` | `string` | `str` | owned, heap-allocated string |

### String vs &str — the most confusing part for beginners

```rust
let s1: &str = "hello";           // string literal — lives in compiled binary, borrowed
let s2: String = String::from("hello");  // heap-allocated, owned
let s3: String = "hello".to_string();    // same — converts &str to String
let s4: &str = &s2;              // borrow a String as &str — always works

// In function signatures:
fn takes_str(s: &str) {}         // accepts both &str and &String (flexible)
fn takes_string(s: String) {}    // takes ownership — caller loses it
fn returns_str() -> &str {}      // borrows from somewhere — lifetime needed
fn returns_string() -> String {} // returns owned — caller gets it
```

Rule of thumb from this project:
- Function **parameters** → use `&str` or `&Path` (borrow, don't own)
- Function **return values** → use `String` or `PathBuf` (return owned data)
- **Struct fields** → use `String`, `PathBuf` (own the data)

---

## 4. Ownership — Rust's Core Idea

This is the hardest concept for Go/Python developers. Read this carefully.

### The rule: every value has exactly one owner

```rust
let a = String::from("hello");  // a owns the string
let b = a;                       // ownership MOVED to b — a no longer valid!

println!("{}", a);  // COMPILE ERROR: value moved
println!("{}", b);  // OK
```

```go
// Go — no such concept. Both a and b are valid.
a := "hello"
b := a
fmt.Println(a)  // fine
fmt.Println(b)  // fine
```

```python
# Python — reference counting. Both point to same object.
a = "hello"
b = a
print(a)  # fine
print(b)  # fine
```

### Move vs Copy

**Primitives are Copied** (i32, u64, bool, f64 — cheap to duplicate):
```rust
let x: u64 = 42;
let y = x;        // copied — x is still valid
println!("{}", x); // fine
```

**Heap types are Moved** (String, Vec, PathBuf — expensive to duplicate):
```rust
let paths: Vec<PathBuf> = vec![...];
let moved = paths;    // ownership moved
// paths is now gone — cannot use it
```

**To keep the original, clone it explicitly:**
```rust
let paths: Vec<PathBuf> = vec![...];
let also_paths = paths.clone();  // explicit deep copy — intentional cost
// both paths and also_paths are valid now
```

### How this showed up in our project

In `main.rs`:
```rust
// dry_run path — we need artifacts later too, so we borrow with .iter()
let paths: Vec<PathBuf> = artifacts.iter().map(|a| a.path.clone()).collect();

// final delete path — we won't need artifacts after this, so we move with into_iter()
let paths: Vec<PathBuf> = artifacts.into_iter().map(|a| a.path).collect();
//                                  ^^^^^^^^^^^
//                                  consumes artifacts — moves each a.path out
```

`.iter()` = borrow each element (keep the original Vec intact)
`.into_iter()` = move each element out (consume the Vec)

---

## 5. Borrowing & References

Since moving transfers ownership, Rust provides **borrowing** — temporary access without ownership transfer.

### Two types of borrows

```rust
let s = String::from("hello");

let r1 = &s;      // immutable borrow — read-only access
let r2 = &s;      // another immutable borrow — multiple OK
println!("{} {}", r1, r2);

let r3 = &mut s;  // ERROR — s itself must be `mut` for a mutable borrow
```

```rust
let mut s = String::from("hello");
let r3 = &mut s;  // mutable borrow — exclusive write access
// while r3 exists, no other borrows allowed
r3.push_str(" world");
```

### The borrow checker rules (memorise these)

1. You can have **any number of immutable borrows** at the same time
2. You can have **exactly one mutable borrow** — and nothing else
3. Borrows must not **outlive** the thing they borrow

These rules are checked at **compile time** — zero runtime cost.

### How this showed up in our project

```rust
// scanner.rs — path: &Path means "borrow a path, don't own it"
pub fn scan_workspace(path: &Path, months: u32, artifact_types: &[String]) -> Vec<ArtifactFolder>
//                          ^^^^^ borrow        ^^^^^^^^^^^^^^^^^ borrow slice

// cleaner.rs — Vec<PathBuf> means "take ownership of the paths"
pub fn delete_artifacts(paths: Vec<PathBuf>, dry_run: bool) -> DeleteResult
//                             ^^^^^^^^^^^^ owned — caller gives it up

// display.rs — &[ArtifactFolder] means "borrow a slice to read from"
pub fn print_results(artifacts: &[ArtifactFolder])
//                              ^^^^^^^^^^^^^^^^^^ borrow slice, read-only
```

**Rule of thumb:** If a function only needs to READ → borrow (`&`). If it needs to CONSUME or OWN → take ownership.

---

## 6. Structs

Rust has no classes. It has **structs** (data) and **impl blocks** (methods). Composition over inheritance.

### Defining a struct

```rust
// From scanner.rs
#[derive(Debug, Clone)]
pub struct ArtifactFolder {
    pub path: PathBuf,
    pub artifact_type: String,
    pub project: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub last_modified: String,
}
```

```go
// Go equivalent
type ArtifactFolder struct {
    Path         string
    ArtifactType string
    Project      string
    SizeBytes    uint64
    SizeHuman    string
    LastModified string
}
```

```python
# Python equivalent
@dataclass
class ArtifactFolder:
    path: str
    artifact_type: str
    project: str
    size_bytes: int
    size_human: str
    last_modified: str
```

### `#[derive(...)]` — auto-generated trait implementations

```rust
#[derive(Debug, Clone)]
```

| Derive | What it gives you | Go equivalent |
|--------|------------------|---------------|
| `Debug` | `{:?}` formatting, `dbg!()` macro | `%+v` in fmt |
| `Clone` | `.clone()` for deep copy | manual copy |
| `PartialEq` | `==` comparison | `==` |
| `Serialize` | JSON/YAML output (with serde crate) | `json.Marshal` |

### Creating a struct instance

```rust
// struct literal — all fields required
results.push(ArtifactFolder {
    path: entry_path.to_path_buf(),
    artifact_type: folder_name.to_string(),
    project: project_name,    // shorthand: when variable name == field name
    size_bytes,               // shorthand for size_bytes: size_bytes
    size_human,
    last_modified,
});
```

### impl blocks — adding methods to structs

We didn't write `impl` blocks in this project (all functions were standalone), but here's the pattern:

```rust
impl ArtifactFolder {
    // Constructor pattern — Rust has no `new` keyword
    pub fn new(path: PathBuf, artifact_type: String) -> Self {
        ArtifactFolder {
            path,
            artifact_type,
            project: String::new(),
            size_bytes: 0,
            size_human: "0 B".to_string(),
            last_modified: String::new(),
        }
    }

    // Method — &self means read-only access to the struct
    pub fn is_large(&self) -> bool {
        self.size_bytes > 100 * 1024 * 1024  // > 100 MB
    }

    // Mutable method — &mut self means can modify the struct
    pub fn set_size(&mut self, bytes: u64) {
        self.size_bytes = bytes;
    }
}
```

```go
// Go equivalent
func NewArtifactFolder(path, artifactType string) *ArtifactFolder {
    return &ArtifactFolder{Path: path, ArtifactType: artifactType}
}
func (a *ArtifactFolder) IsLarge() bool { return a.SizeBytes > 100*1024*1024 }
```

---

## 7. Enums — Far More Powerful Than Go/Python Enums

Rust enums can hold data. They are the foundation of `Option` and `Result`.

```rust
// Simple enum (like Go/Python)
enum ArtifactType {
    NodeModules,
    NextJs,
    Dist,
    Build,
    Terraform,
}

// Enum with data — each variant can hold different types
enum ScanResult {
    Found(ArtifactFolder),   // holds a struct
    Empty,                    // holds nothing
    Error(String),            // holds an error message
}
```

```go
// Go — enums are just constants, can't hold data
type ArtifactType int
const (
    NodeModules ArtifactType = iota
    NextJs
)
```

You work with enums using `match` — covered in section 10.

---

## 8. Option\<T\> — No null, No None Surprises

Python has `None`. Go has `nil`. Both can cause runtime panics if you forget to check.

Rust has `Option<T>` — a type that **forces you to handle the missing case at compile time**.

```rust
// Option<T> is defined as:
enum Option<T> {
    Some(T),   // there is a value
    None,      // there is no value
}
```

### How we used it in scanner.rs

```rust
// file_name() returns Option<&OsStr> — the path might have no filename
let folder_name = entry_path
    .file_name()              // Option<&OsStr>
    .and_then(|n| n.to_str()) // Option<&str>  — chain another Option operation
    .unwrap_or("");           // &str          — provide a default if None
```

```python
# Python equivalent — but no compiler help if you forget the check
folder_name = os.path.basename(entry_path) or ""
```

### Option methods you'll use constantly

```rust
let opt: Option<String> = Some("hello".to_string());

opt.unwrap()                    // get value or PANIC (avoid in production)
opt.unwrap_or("default".to_string())  // get value or default
opt.unwrap_or_else(|| compute_default()) // get value or run closure
opt.is_some()                   // true if Some
opt.is_none()                   // true if None
opt.map(|v| v.len())            // Option<String> → Option<usize>
opt.and_then(|v| parse(v))      // Option<String> → Option<T> (chain)
opt.filter(|v| v.len() > 3)     // Some → None if predicate false
```

---

## 9. Result\<T, E\> — Error Handling Without Exceptions

Python uses exceptions (`try/except`). Go uses multiple return values (`val, err := ...`). Rust uses `Result<T, E>`.

```rust
// Result<T, E> is defined as:
enum Result<T, E> {
    Ok(T),   // success — holds the value
    Err(E),  // failure — holds the error
}
```

### How we used it in cleaner.rs

```rust
// fs::remove_dir_all returns Result<(), std::io::Error>
match fs::remove_dir_all(&path) {
    Ok(()) => {
        deleted += 1;
        total_freed_bytes += size;
    }
    Err(e) => {
        failed.push(format!("{}: {}", path.display(), e));
    }
}
```

```python
# Python equivalent
try:
    shutil.rmtree(path)
    deleted += 1
except OSError as e:
    failed.append(f"{path}: {e}")
```

```go
// Go equivalent
if err := os.RemoveAll(path); err != nil {
    failed = append(failed, fmt.Sprintf("%s: %v", path, err))
} else {
    deleted++
}
```

### The `?` operator — propagate errors upward

Rust has a shorthand for "return the error if there is one":

```rust
fn read_file(path: &Path) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string(path)?;  // ? = return Err if Err, unwrap if Ok
    Ok(contents)
}

// Without ? — more verbose
fn read_file(path: &Path) -> Result<String, std::io::Error> {
    let contents = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };
    Ok(contents)
}
```

`?` is equivalent to Go's `if err != nil { return err }` — but as a single character.

---

## 10. Pattern Matching with `match`

`match` is Rust's most powerful statement. It's like `switch` but exhaustive — the compiler forces you to handle every case.

### Basic match

```rust
// From display.rs
fn color_artifact_type(t: &str) -> colored::ColoredString {
    match t {
        "node_modules" => t.yellow(),
        ".next"        => t.cyan(),
        ".terraform"   => t.magenta(),
        "dist"         => t.blue(),
        "build"        => t.bright_blue(),
        _              => t.normal(),   // _ is the catch-all — required if not exhaustive
    }
}
```

```python
# Python equivalent (match added in 3.10)
match t:
    case "node_modules": return colored(t, "yellow")
    case ".next":        return colored(t, "cyan")
    case _:              return t
```

```go
// Go equivalent
switch t {
case "node_modules": return yellow(t)
case ".next":        return cyan(t)
default:             return t
}
```

### Matching on Result and Option

```rust
// From scanner.rs — nested match on Result
let project_mtime = match fs::metadata(project_path) {
    Ok(meta) => match meta.modified() {
        Ok(t) => t,
        Err(_) => continue,  // _ ignores the error value
    },
    Err(_) => continue,
};
```

### `if let` — match when you only care about one variant

```rust
// Instead of full match when you only want Some:
if let Some(name) = entry_path.file_name() {
    println!("{}", name.to_string_lossy());
}

// Instead of:
match entry_path.file_name() {
    Some(name) => println!("{}", name.to_string_lossy()),
    None => {}  // do nothing
}
```

---

## 11. Functions and Implicit Return

```rust
// Full signature breakdown from scanner.rs:
pub fn scan_workspace(
//  ^^^                ← pub = public (visible outside module)
//      ^^             ← fn = function keyword
//         ^^^^^^^^^^^^← function name (snake_case by convention)
    path: &Path,       // parameter: name: type
    months: u32,
    artifact_types: &[String],
) -> Vec<ArtifactFolder> {   // return type after ->
//                       ^ no semicolon — this is the return expression
    // ...
    results  // NO semicolon = implicit return of this value
             // Adding ; here would make it return () (nothing) — common mistake!
}
```

### Semicolons matter

```rust
fn returns_five() -> i32 {
    5        // no semicolon — returns 5
}

fn returns_nothing() -> () {
    5;       // semicolon — expression becomes statement, returns ()
}

fn early_return(x: i32) -> i32 {
    if x < 0 {
        return 0;  // explicit return — semicolon is fine here
    }
    x * 2    // implicit return
}
```

---

## 12. Closures

Closures are anonymous functions that can capture their environment.

```rust
// Rust closure syntax — || is the parameter list
let double = |x| x * 2;
let add = |x, y| x + y;
let greet = |name: &str| format!("Hello, {}!", name);

// Capture from environment
let threshold = 100_u64;
let is_large = |bytes: u64| bytes > threshold;  // captures threshold
```

```python
# Python equivalent
double = lambda x: x * 2
is_large = lambda bytes: bytes > threshold
```

```go
// Go equivalent
double := func(x int) int { return x * 2 }
isLarge := func(bytes uint64) bool { return bytes > threshold }
```

### How we used closures in this project

```rust
// scanner.rs — closure passed to sort_by
results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
//              ^^^^^^^ closure with two params
//                      compare b to a (reversed = descending)

// scanner.rs — closure in filter_map (returns Option)
.filter_map(|e| e.ok())   // |e| is the closure param, e.ok() converts Result to Option

// scanner.rs — closure in any()
EXCLUDED_PATHS.iter().any(|ex| path_str.contains(ex))
//                        ^^^^ closure: for each excluded path, check if it's in path_str

// main.rs — closure in unwrap_or_else (only runs if Err)
.unwrap_or_else(|_| {
    eprintln!("Error: path not found");
    std::process::exit(1);
})
```

### Move closures — capturing ownership

```rust
let name = String::from("world");
let greet = move || println!("Hello, {}", name);
//          ^^^^ name is MOVED into the closure — closure owns it
greet(); // works
// name is no longer accessible here
```

Use `move` when the closure will outlive the current scope (e.g. threads).

---

## 13. Iterators — Functional Data Pipelines

Rust iterators are lazy chains — nothing runs until `.collect()`, `.sum()`, `.for_each()`, etc.

```rust
// From scanner.rs — the cleanest example in the project
pub fn compute_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()              // create iterator over directory entries
        .filter_map(|e| e.ok())  // skip errors, unwrap Ok values
        .filter(|e| e.file_type().is_file())  // only files (not dirs)
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))  // entry → byte size
        .sum()                    // add all u64 values
}
```

```python
# Python equivalent
def compute_size(path):
    return sum(
        f.stat().st_size
        for f in Path(path).rglob("*")
        if f.is_file()
    )
```

```go
// Go — no built-in functional pipeline, use a loop
func computeSize(path string) uint64 {
    var total uint64
    filepath.Walk(path, func(p string, info os.FileInfo, err error) error {
        if err == nil && !info.IsDir() {
            total += uint64(info.Size())
        }
        return nil
    })
    return total
}
```

### Common iterator methods

| Method | What it does | Python equivalent |
|--------|-------------|-------------------|
| `.map(f)` | Transform each element | `map(f, iter)` |
| `.filter(f)` | Keep elements where f returns true | `filter(f, iter)` |
| `.filter_map(f)` | Transform + filter None/Err in one step | — |
| `.any(f)` | True if any element satisfies f | `any(f(x) for x in iter)` |
| `.all(f)` | True if all elements satisfy f | `all(f(x) for x in iter)` |
| `.count()` | Count elements | `len(list(iter))` |
| `.sum()` | Sum numeric elements | `sum(iter)` |
| `.collect()` | Materialise into Vec, HashMap etc. | `list(iter)` |
| `.for_each(f)` | Run f on each element | `[f(x) for x in iter]` |
| `.enumerate()` | Add index: `(0, val), (1, val)...` | `enumerate(iter)` |
| `.zip(other)` | Pair two iterators | `zip(a, b)` |
| `.flat_map(f)` | Map then flatten | `itertools.chain.from_iterable(map(f, iter))` |
| `.take(n)` | First n elements | `itertools.islice(iter, n)` |
| `.skip(n)` | Skip first n elements | `itertools.islice(iter, n, None)` |

### `.iter()` vs `.into_iter()` vs `.iter_mut()`

```rust
let v = vec![1, 2, 3];

v.iter()       // borrows — yields &i32. v still usable after.
v.into_iter()  // moves — yields i32. v consumed after.
v.iter_mut()   // mutable borrow — yields &mut i32. v still usable after.
```

---

## 14. Modules and Visibility

Rust does **not** auto-discover files. You declare modules explicitly.

```rust
// main.rs — declare modules
mod scanner;   // compiler looks for src/scanner.rs
mod cleaner;   // compiler looks for src/cleaner.rs
mod display;   // compiler looks for src/display.rs
```

```rust
// Importing from modules
use crate::scanner::ArtifactFolder;  // from our own project
use std::path::PathBuf;              // from standard library
use walkdir::WalkDir;                // from external crate
```

### Visibility rules

```rust
pub struct Foo { }           // public struct
pub fn bar() { }             // public function
struct Hidden { }            // private — only this module can use it
fn also_hidden() { }         // private function

pub struct Mixed {
    pub name: String,        // public field
    secret: String,          // private field — only this module sees it
}
```

```go
// Go — capital letter = public, lowercase = private
type Foo struct { }         // exported
type hidden struct { }      // unexported
func Bar() {}               // exported
func baz() {}               // unexported
```

### Module tree

```
crate (src/main.rs)
├── mod scanner       (src/scanner.rs)
├── mod cleaner       (src/cleaner.rs)
├── mod display       (src/display.rs)
└── mod mac_library   (src/mac_library/mod.rs)
    ├── mod checker   (src/mac_library/checker.rs)
    ├── mod resolver  (src/mac_library/resolver.rs)
    ├── mod scanner   (src/mac_library/scanner.rs)
    └── mod display   (src/mac_library/display.rs)
```

Refer to items across modules:
```rust
crate::scanner::compute_size(&path)          // full path from crate root
mac_library::run(&args)                      // from main.rs (direct child)
use super::resolver::EntryKind;              // from mac_library/scanner.rs (sibling)
```

### `#[cfg(...)]` — conditional compilation

`#[cfg]` is evaluated at **compile time**, not runtime. Code inside a `#[cfg]` block is completely excluded from the binary on platforms where the condition is false.

```rust
// src/mac_library/mod.rs — this entire block is omitted on non-macOS builds
#[cfg(not(target_os = "macos"))]
pub fn run(_args: &crate::MacLibArgs) {
    eprintln!("mac-lib is only supported on macOS.");
}

#[cfg(target_os = "macos")]
pub fn run(args: &crate::MacLibArgs) {
    // actual implementation
}
```

This is different from a runtime `if`:
```rust
// Runtime check — both branches exist in the binary, condition checked at runtime
if std::env::consts::OS == "macos" { ... }

// Compile-time check — the non-matching branch is never compiled in
#[cfg(target_os = "macos")]
fn mac_only() { ... }
```

Use `#[cfg]` when you need platform-specific code, OS-specific APIs, or optional feature flags.

---

## 15. Traits — Rust's Interfaces

A trait defines behaviour a type must implement. Similar to Go interfaces, but more explicit.

```rust
// Traits you used via #[derive] without knowing it:

// Debug trait — enables {:?} formatting
#[derive(Debug)]
struct ArtifactFolder { ... }
println!("{:?}", artifact);   // works because of Debug

// Clone trait — enables .clone()
#[derive(Clone)]
struct ArtifactFolder { ... }
let copy = artifact.clone();
```

### Using traits from external crates

```rust
use colored::Colorize;  // import the Colorize trait

// Now &str has .red(), .bold(), .cyan() methods — added by the trait
"hello".red().bold()
```

This is called a **trait extension** — adding methods to existing types via traits.
Go has no equivalent. Python does this informally via monkey-patching.

### Common standard library traits

| Trait | What it enables | Derive? |
|-------|----------------|---------|
| `Debug` | `{:?}` formatting | Yes |
| `Display` | `{}` formatting, `to_string()` | Manual |
| `Clone` | `.clone()` deep copy | Yes |
| `Copy` | implicit copy (primitives) | Yes |
| `PartialEq` | `==` comparison | Yes |
| `PartialOrd` | `<`, `>` comparison | Yes |
| `Iterator` | iterator protocol | Manual |
| `From`/`Into` | type conversion | Yes/Auto |

---

## 16. Macros — Code That Writes Code

Macros end with `!` in Rust. They are not functions — they generate code at compile time.

```rust
println!("hello {}", name);   // formatted print to stdout
eprintln!("error: {}", e);    // formatted print to stderr (like sys.stderr in Python)
format!("hello {}", name);    // returns a String (like Python's f-string)
vec![1, 2, 3];                // creates a Vec<i32>
dbg!(value);                  // prints file:line:value to stderr — great for debugging
todo!();                      // compile fine, panic at runtime — placeholder
unreachable!();               // marks a code path that should never be reached
```

### `matches!` — readable boolean pattern checks

`matches!(expr, pattern)` returns `true` if `expr` matches the pattern. It's shorthand for a `match` that returns a bool, and is especially useful inside `.filter()` chains:

```rust
// From src/mac_library/mod.rs — filter entries by status
let orphaned: Vec<_> = results
    .iter()
    .filter(|e| matches!(e.status, EntryStatus::OrphanedApp | EntryStatus::OrphanedCli))
    .collect();

// The verbose equivalent — same result, more noise:
let orphaned: Vec<_> = results
    .iter()
    .filter(|e| match e.status {
        EntryStatus::OrphanedApp | EntryStatus::OrphanedCli => true,
        _ => false,
    })
    .collect();
```

```python
# Python equivalent
orphaned = [e for e in results if e.status in ("OrphanedApp", "OrphanedCli")]
```

### Format strings

```rust
println!("{}", value);          // Display trait — human-readable
println!("{:?}", value);        // Debug trait — developer representation
println!("{:#?}", value);       // Debug, pretty-printed with indentation
println!("{:>10}", value);      // right-align in 10 chars
println!("{:<10}", value);      // left-align in 10 chars
println!("{:0>5}", 42);         // "00042" — pad with zeros
println!("{:.2}", 3.14159);     // "3.14" — 2 decimal places
```

```python
# Python equivalents
print(f"{value}")
print(f"{value!r}")
print(f"{value:>10}")
print(f"{42:05d}")
print(f"{3.14159:.2f}")
```

---

## 17. The Standard Library You'll Use Daily

```rust
// Filesystem
use std::fs;
fs::read_to_string(path)?;        // read file to String
fs::write(path, contents)?;       // write String to file
fs::remove_dir_all(path)?;        // recursive delete (our cleaner.rs)
fs::metadata(path)?;              // file info (size, mtime)
fs::create_dir_all(path)?;        // mkdir -p

// Paths
use std::path::{Path, PathBuf};
Path::new("/some/path")           // &Path from &str
PathBuf::from("/some/path")       // owned path
path.join("subdir")               // append component
path.file_name()                  // Option<&OsStr>
path.extension()                  // Option<&OsStr>
path.parent()                     // Option<&Path>
path.exists()                     // bool
path.is_dir()                     // bool
path.is_file()                    // bool
path.to_string_lossy()            // Cow<str> — safe string conversion

// Collections
use std::collections::HashMap;
let mut map: HashMap<String, u64> = HashMap::new();
map.insert("key".to_string(), 42);
map.get("key")                    // Option<&u64>

// Time
use std::time::{SystemTime, Duration};
SystemTime::now()
Duration::from_secs(60 * 60 * 24)

// Environment
std::env::args()                  // command-line arguments iterator
std::env::var("HOME")             // Result<String, VarError>
std::process::exit(1);            // exit with code
```

---

## 18. External Crates (Dependencies)

### Finding crates

- **[crates.io](https://crates.io)** — the official package registry (like PyPI / pkg.go.dev)
- **[docs.rs](https://docs.rs)** — auto-generated documentation for every crate
- **[lib.rs](https://lib.rs)** — curated, ranked crate search

### Crates used in this project

| Crate | Purpose | Python equiv | Go equiv |
|-------|---------|-------------|---------|
| `walkdir` | Recursive dir traversal | `os.walk()` | `filepath.Walk()` |
| `clap` | CLI argument parsing | `argparse` / `click` | `cobra` / `flag` |
| `chrono` | Date & time | `datetime` | `time` |
| `humansize` | Bytes → "1.2 GB" | — | — |
| `colored` | Terminal colours | `colorama` | — |
| `indicatif` | Progress bars/spinners | `tqdm` | — |
| `dialoguer` | Interactive prompts | `inquirer` | — |

### Subcommands with clap

This project uses clap's derive API to build a multi-command CLI (`ac scan` and `ac mac-lib`). The pattern has three parts:

```rust
// 1. Top-level parser — holds the subcommand
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_BIN_NAME"), about = "...", version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// 2. Enum of subcommands — each variant holds its own Args struct
#[derive(Subcommand, Debug)]
enum Commands {
    Scan(ScanArgs),    // `ac scan [OPTIONS] [PATH]`
    MacLib(MacLibArgs), // `ac mac-lib [OPTIONS]`
}

// 3. Args struct per subcommand
#[derive(Args, Debug)]
struct ScanArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short, long, default_value = "2")]
    months: u32,
    #[arg(short = 'd', long)]
    dry_run: bool,
}
```

Dispatching is a simple `match` on the enum:
```rust
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan(args) => run_scan(args),
        Commands::MacLib(args) => mac_library::run(&args),
    }
}
```

clap auto-generates `--help`, usage strings, and error messages for each subcommand. `env!("CARGO_BIN_NAME")` makes each binary self-identify when built as both `artifact-cleaner` and `ac`.

### Adding a crate

```bash
cargo add serde --features derive   # adds to Cargo.toml automatically
```

Or manually in `Cargo.toml`:
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
```

---

## 19. Memory Model — Stack vs Heap

Understanding this explains why Rust's ownership rules exist.

| | Stack | Heap |
|-|-------|------|
| Size | Fixed at compile time | Dynamic |
| Speed | Very fast (pointer move) | Slower (allocation) |
| Examples | `i32`, `u64`, `bool`, `f64` | `String`, `Vec<T>`, `Box<T>` |
| Freed when | Scope ends (automatic) | Owner goes out of scope |

```rust
fn example() {
    let x: i32 = 42;           // stack — freed when function returns
    let s = String::from("hi"); // heap — String data on heap, pointer on stack
                                 // freed when s goes out of scope (end of fn)
}   // ← x and s both freed here automatically
```

```python
# Python — everything on heap, GC frees when ref count = 0
def example():
    x = 42          # int object on heap
    s = "hi"        # str object on heap
# GC might free x and s now, or later — you don't know when
```

**This is why Rust is fast:** no GC pauses, no reference counting overhead. The compiler statically proves when each value is freed and inserts the free at exactly the right point.

---

## 20. Common Beginner Mistakes

### 1. Forgot `mut`
```rust
let results = Vec::new();
results.push(item);  // ERROR: cannot borrow `results` as mutable
// Fix:
let mut results = Vec::new();
```

### 2. Used value after move
```rust
let paths = vec![PathBuf::from("/tmp")];
let result = delete(paths);     // paths moved into delete()
println!("{:?}", paths);        // ERROR: value moved
// Fix: clone if you need it in both places
let result = delete(paths.clone());
```

### 3. Semicolon on return value
```rust
fn get_name() -> String {
    "hello".to_string();   // ERROR: returns () not String (semicolon!)
}
// Fix:
fn get_name() -> String {
    "hello".to_string()    // no semicolon = implicit return
}
```

### 4. &String vs &str confusion
```rust
fn greet(name: &String) { }  // works but too restrictive
fn greet(name: &str) { }     // better — accepts both &str and &String
```

### 5. unwrap() in production code
```rust
let val = some_option.unwrap();         // panics if None
// Fix:
let val = some_option.unwrap_or(default);
let val = some_option.expect("meaningful message");  // panics with better message
```

### 6. Iterating and modifying
```rust
for item in &mut results {  // mutable borrow
    item.size_bytes = 0;    // OK — modifying each element
}
// Cannot also call results.push() inside — only one mutable borrow at a time
```

---

## 21. Cheat Sheet — Go/Python vs Rust

### Variables
| Concept | Python | Go | Rust |
|---------|--------|-----|------|
| Declare | `x = 5` | `x := 5` | `let x = 5;` |
| Mutable declare | `x = 5` | `x := 5` | `let mut x = 5;` |
| Typed declare | — | `var x int = 5` | `let x: i32 = 5;` |
| Constant | `X = 5` | `const X = 5` | `const X: i32 = 5;` |

### Functions
| Concept | Python | Go | Rust |
|---------|--------|-----|------|
| Define | `def f(x): ...` | `func f(x int) int` | `fn f(x: i32) -> i32` |
| Public | (no concept) | Capital letter | `pub fn` |
| Return | `return x` | `return x` | `x` (last expr) or `return x;` |
| Multiple return | `return a, b` | `return a, b` | `(a, b)` tuple |
| Error return | `raise Exception` | `return val, err` | `Result<T, E>` |

### Collections
| Concept | Python | Go | Rust |
|---------|--------|-----|------|
| Dynamic array | `list` | `slice` | `Vec<T>` |
| Key-value map | `dict` | `map` | `HashMap<K, V>` |
| Fixed array | `tuple` | `[N]T` | `[T; N]` |
| Set | `set` | `map[T]struct{}` | `HashSet<T>` |
| Create list | `[1, 2, 3]` | `[]int{1,2,3}` | `vec![1, 2, 3]` |
| Append | `l.append(x)` | `append(l, x)` | `v.push(x)` |
| Length | `len(l)` | `len(l)` | `v.len()` |

### Error handling
| Concept | Python | Go | Rust |
|---------|--------|-----|------|
| Throw/return error | `raise ValueError("msg")` | `return fmt.Errorf("msg")` | `return Err("msg".into())` |
| Handle error | `try/except` | `if err != nil` | `match` / `?` |
| Ignore error | (bad practice) | `_` | `.ok()` / `.unwrap_or_default()` |
| Propagate error | `raise` | `return err` | `?` |

### Null safety
| Concept | Python | Go | Rust |
|---------|--------|-----|------|
| Nullable value | `None` | `nil` | `Option<T>` = `None` |
| Has value | `x is not None` | `x != nil` | `x.is_some()` |
| Get value | `x` | `*x` | `x.unwrap()` |
| Safe get | `x or default` | `if x != nil { use x }` | `x.unwrap_or(default)` |

---

## What to learn next

1. **Lifetimes** — when borrowing gets complex, the compiler needs hints about how long references live
2. **Traits in depth** — implementing your own traits, trait objects (`dyn Trait`)
3. **Generics** — writing functions that work over many types `fn largest<T>(list: &[T]) -> T`
4. **Async/await** — concurrent programming with `tokio` (Rust's most popular async runtime)
5. **Error handling patterns** — `thiserror` crate for custom error types, `anyhow` for applications
6. **Testing** — `#[test]`, `#[cfg(test)]`, integration tests in `tests/` directory
7. **Closures and Fn traits** — `Fn`, `FnMut`, `FnOnce` — why they're different
8. **Smart pointers** — `Box<T>`, `Rc<T>`, `Arc<T>` for cases where ownership gets complex

### Recommended resources

- **The Rust Book** — [doc.rust-lang.org/book](https://doc.rust-lang.org/book) — free, official, comprehensive
- **Rust by Example** — [doc.rust-lang.org/rust-by-example](https://doc.rust-lang.org/rust-by-example) — learn by reading code
- **Rustlings** — [github.com/rust-lang/rustlings](https://github.com/rust-lang/rustlings) — small exercises
- **This project** — read `src/scanner.rs` → `src/cleaner.rs` → `src/display.rs` → `src/main.rs`

---

*Written from hands-on experience building `artifact-cleaner` — a real Rust CLI tool.*
*Every concept in this document maps to actual code in `src/`.*
