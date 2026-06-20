# Rust Lesson Index

Every `RUST LESSON` block in this project's source code, indexed by topic.
Use this as a quick-reference map — click any link to jump straight to the lesson in context.

> **New here?** Start with [RUST_LEARNING.md](./RUST_LEARNING.md) for the full guide,
> or jump into `src/main.rs` and follow the lesson blocks in order.

---

## Core Language

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| `mod` declarations | [`src/main.rs`](src/main.rs#L3) | 3 | How Rust discovers files — it doesn't auto-scan, you declare every module |
| `#[derive(Parser)]` with clap | [`src/main.rs`](src/main.rs#L19) | 19 | Generating a full CLI parser from a struct using derive macros |
| Subcommands with clap | [`src/main.rs`](src/main.rs#L49) | 49 | Structuring multi-command CLIs with `#[derive(Subcommand)]` and `Args` |
| `fn main()` | [`src/main.rs`](src/main.rs#L118) | 118 | Entry point, exit codes, and when to use `-> Result<>` |

---

## Structs and Data

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| Structs | [`src/scanner.rs`](src/scanner.rs#L10) | 10 | Defining data types; `pub`, `#[derive]`, owned vs borrowed fields |
| Structs with owned data | [`src/cleaner.rs`](src/cleaner.rs#L7) | 7 | Why struct fields use `String`/`Vec` (owned) not `&str`/`&[]` (borrowed) |
| Struct literal return (implicit return) | [`src/cleaner.rs`](src/cleaner.rs#L65) | 65 | Returning a struct without `return`; field shorthand syntax |
| Constants | [`src/scanner.rs`](src/scanner.rs#L29) | 29 | `const` vs `let`; `&[&str]` slice of string refs |

---

## Enums and Pattern Matching

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| Enums as rich types | [`src/mac_library/resolver.rs`](src/mac_library/resolver.rs#L3) | 3 | Enums that carry data (algebraic data types); why this beats string status codes |
| `match` (pattern matching) | [`src/scanner.rs`](src/scanner.rs#L79) | 79 | Exhaustive matching on `Result`; nested match; `continue` inside match |
| Pattern matching with guards | [`src/mac_library/scanner.rs`](src/mac_library/scanner.rs#L128) | 128 | `if let` — match on one variant without a full match block |
| `matches!` macro | [`src/mac_library/mod.rs`](src/mac_library/mod.rs#L62) | 62 | `matches!(expr, pattern \| pattern)` — readable boolean pattern checks in filter chains |

---

## Ownership and Borrowing

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| Taking ownership via `Vec<PathBuf>` | [`src/cleaner.rs`](src/cleaner.rs#L21) | 21 | When to take `Vec<T>` (owned) vs `&[T]` (borrowed); move semantics |
| `for entry in &orphaned` | [`src/mac_library/mod.rs`](src/mac_library/mod.rs#L116) | 116 | Iterating by reference vs by value — why `&vec` keeps the Vec alive after the loop |
| `&[T]` slices vs `Vec<T>` | [`src/mac_library/display.rs`](src/mac_library/display.rs#L22) | 22 | Why function parameters use slices; automatic coercion from `&Vec<T>` |

---

## Functions and Closures

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| Functions | [`src/scanner.rs`](src/scanner.rs#L44) | 44 | Signature anatomy: `pub fn`, parameters, return types, borrowed vs owned |
| Private helper functions | [`src/display.rs`](src/display.rs#L105) | 105 | No `pub` = private; clean module APIs |
| Closures | [`src/scanner.rs`](src/scanner.rs#L139) | 139 | `||` syntax; `unwrap_or_else` with a closure fallback |
| Closures capturing the environment | [`src/mac_library/scanner.rs`](src/mac_library/scanner.rs#L74) | 74 | Closures inside closures; `.then()` on `Ordering` for chained sorts |
| Tuple return types | [`src/mac_library/display.rs`](src/mac_library/display.rs#L101) | 101 | Returning multiple values as `(A, B)`; when to use tuples vs structs |

---

## Iterators and Collections

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| Iterators | [`src/scanner.rs`](src/scanner.rs#L62) | 62 | Lazy iterator chains; `.filter_map()`; `.collect::<Vec<_>>()` |
| Iterator chains as computation | [`src/scanner.rs`](src/scanner.rs#L165) | 165 | Summing file sizes with a single expression; `.sum()` via the `Sum` trait |
| `.flatten()` on iterators of Results | [`src/mac_library/scanner.rs`](src/mac_library/scanner.rs#L108) | 108 | `.flatten()` as shorthand for `.filter_map(\|e\| e.ok())` — idiomatic error skipping |
| Iterating with references | [`src/display.rs`](src/display.rs#L52) | 52 | `.iter()` borrows; when to use `.iter()` vs `.into_iter()` |

---

## Error Handling and Options

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| Error handling with match | [`src/cleaner.rs`](src/cleaner.rs#L44) | 44 | `fs::remove_dir_all` returns `Result`; `Ok(())` unit success; capturing `Err(e)` |
| Option chaining with `.ok()` and `.map()` | [`src/mac_library/scanner.rs`](src/mac_library/scanner.rs#L186) | 186 | `Result → Option → T` without panicking; chained transforms on `Option` |
| `if !vec.is_empty()` | [`src/display.rs`](src/display.rs#L91) | 91 | Idiomatic emptiness check; `!` boolean NOT |

---

## Modules and Visibility

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| `use` imports | [`src/display.rs`](src/display.rs#L8) | 8 | Importing from sibling modules (`crate::`) and external crates |
| `super::` imports | [`src/mac_library/scanner.rs`](src/mac_library/scanner.rs#L3) | 3 | `super::` = parent module; `crate::` = project root; module path analogy |
| Pure helper functions | [`src/mac_library/resolver.rs`](src/mac_library/resolver.rs#L81) | 81 | Private helpers with single responsibilities; keeping the public API minimal |

---

## Standard Library and Platform

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| `std::process::Command` | [`src/mac_library/checker.rs`](src/mac_library/checker.rs#L5) | 5 | Spawning subprocesses; `.arg()` vs shell strings (no injection risk); `.output()` |
| `#[cfg(...)]` conditional compilation | [`src/mac_library/mod.rs`](src/mac_library/mod.rs#L19) | 19 | Compile-time platform guards; why `#[cfg]` differs from a runtime `if` |
| Progress spinner | [`src/main.rs`](src/main.rs#L158) | 158 | `indicatif` spinner; `ProgressStyle`; `.enable_steady_tick()` on a background thread |
| Conditional confirmation prompt | [`src/main.rs`](src/main.rs#L217) | 217 | `dialoguer::Confirm`; `unwrap_or(false)` for non-interactive terminals |

---

## String Formatting

| Topic | File | Line | What You'll Learn |
|---|---|---|---|
| String formatting | [`src/display.rs`](src/display.rs#L16) | 16 | `println!` format specifiers; `{:>10}`, `{:<30}`; `.bold()`, `.cyan()` from `colored` |

---

## Reading Order for Beginners

If you are reading the source code for the first time, follow this order:

```
src/scanner.rs      ← structs, constants, functions, iterators, match
src/cleaner.rs      ← ownership, error handling, implicit return
src/display.rs      ← imports, formatting, slices, private helpers
src/main.rs         ← mod declarations, clap, subcommands, closures, spinner
src/mac_library/
  resolver.rs       ← enums as data, private helpers
  checker.rs        ← subprocess calls, Command API
  scanner.rs        ← super::, flatten, closures, Option chaining
  mod.rs            ← cfg, matches!, borrowing in loops
  display.rs        ← slices, tuple returns
```

Each file builds on concepts from the previous one.
For the full written guide, see [RUST_LEARNING.md](./RUST_LEARNING.md).
