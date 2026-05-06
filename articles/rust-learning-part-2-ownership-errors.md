# Rust Learning, Part 2: Ownership, Borrowing, and Errors

For Go, Python, and Java developers, ownership is the concept that makes Rust feel most different. Managed languages let you create object graphs freely and rely on a garbage collector. Rust has no garbage collector. Instead, the compiler proves when each value can be freed.

This article explains ownership, borrowing, lifetimes at a practical level, and Rust's approach to missing values and errors.

## 1. The Core Rule: Every Value Has One Owner

In Rust, each value has exactly one owner. When the owner goes out of scope, Rust drops the value.

```rust
fn example() {
    let name = String::from("artifact-cleaner");
} // name is dropped here
```

This is deterministic cleanup without a garbage collector.

Comparison:

| Language | Memory management model |
|---|---|
| Rust | ownership checked at compile time |
| Go | garbage collector |
| Python | reference counting plus garbage collector |
| Java | garbage collector |

Rust's model gives you predictable cleanup and no GC pauses, but it requires you to be precise about who owns data.

## 2. Moves Make Ownership Visible

Heap-backed values such as `String`, `Vec<T>`, and `PathBuf` move by default.

```rust
let a = String::from("target");
let b = a;

println!("{}", b);
// println!("{}", a); // compile error: a was moved
```

In Go, Python, and Java, both variables would remain usable references or values:

```go
a := "target"
b := a
fmt.Println(a, b)
```

```python
a = "target"
b = a
print(a, b)
```

```java
String a = "target";
String b = a;
System.out.println(a + b);
```

Rust prevents accidental shared ownership. If you want a deep copy, say so:

```rust
let a = String::from("target");
let b = a.clone();

println!("{} {}", a, b);
```

Cloning is explicit because it can be expensive.

## 3. Copy Types Behave Like Simple Values

Small fixed-size values implement `Copy`.

```rust
let x = 42_u64;
let y = x;

println!("{} {}", x, y);
```

Integers, floats, booleans, and some simple tuples copy because copying them is cheap and unambiguous.

Use this mental model:

- `u64`, `bool`, `char`: copied.
- `String`, `Vec<T>`, `PathBuf`: moved.
- `.clone()`: explicit deep copy when you need two owners.

## 4. Borrowing Lets Functions Read Without Taking Ownership

Most functions should not own their inputs. They should borrow them.

```rust
use std::path::Path;

fn is_target_folder(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "target")
        .unwrap_or(false)
}
```

`&Path` means "I need to inspect a path, but I will not own it."

Compare:

```go
func isTargetFolder(path string) bool {
    return filepath.Base(path) == "target"
}
```

```python
def is_target_folder(path: Path) -> bool:
    return path.name == "target"
```

```java
boolean isTargetFolder(Path path) {
    return path.getFileName().toString().equals("target");
}
```

The Rust signature carries an extra guarantee: the function cannot keep or destroy the borrowed path.

## 5. Mutable Borrowing Is Exclusive

Rust permits many immutable borrows or one mutable borrow.

```rust
let mut artifacts = Vec::new();

artifacts.push("target".to_string());
artifacts.push("node_modules".to_string());
```

If a function needs to modify a value, it takes `&mut`.

```rust
fn add_default_type(types: &mut Vec<String>) {
    types.push("target".to_string());
}

let mut types = Vec::new();
add_default_type(&mut types);
```

The key rule:

- Many readers are allowed.
- One writer is allowed.
- Readers and a writer cannot be active at the same time.

This eliminates data races in safe Rust.

## 6. Slices Are Borrowed Views Into Collections

When a function only needs to read a list, prefer a slice:

```rust
fn print_types(types: &[String]) {
    for artifact_type in types {
        println!("{}", artifact_type);
    }
}
```

`&[String]` can accept a borrowed `Vec<String>` or any compatible slice.

This is similar in spirit to accepting an interface or abstract view:

| Rust | Go | Python | Java |
|---|---|---|---|
| `&[T]` | `[]T` | sequence/list | `List<T>` |
| `&str` | string value | `str` | `String` |
| `&Path` | string/path value | `Path` | `Path` |

The Rust difference is that the borrow is checked by the compiler.

## 7. Lifetimes Are Usually Inferred

A lifetime is the compiler's name for how long a reference is valid.

Most Rust code does not write lifetime annotations because the compiler infers them.

```rust
fn first_type(types: &[String]) -> Option<&String> {
    types.first()
}
```

The returned reference must come from `types`. Rust understands this without extra syntax.

You need explicit lifetime annotations only when relationships are ambiguous:

```rust
fn longer<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}
```

Read this as: the returned string slice is valid as long as both inputs are valid.

For beginners, the practical rule is simple:

- Return owned values like `String` or `PathBuf` when data is created inside the function.
- Return references only when the data comes from an input reference.

## 8. Option Replaces Null

Rust does not have `null`.

```rust
let maybe_name: Option<&str> = Some("target");
let missing_name: Option<&str> = None;
```

To use an `Option<T>`, you must handle both cases.

```rust
match maybe_name {
    Some(name) => println!("found {}", name),
    None => println!("missing name"),
}
```

Comparison:

| Concept | Rust | Go | Python | Java |
|---|---|---|---|---|
| Missing value | `Option<T>` | `nil` | `None` | `null` / `Optional<T>` |
| Compiler forces handling | yes | no | no | only with `Optional<T>` conventions |
| Common safe fallback | `unwrap_or` | manual `if` | `or` / conditional | `orElse` |

Useful methods:

```rust
let display_name = maybe_name.unwrap_or("unknown");
let length = maybe_name.map(|name| name.len()).unwrap_or(0);
```

Avoid `unwrap()` in production paths unless a missing value is truly impossible and a panic is acceptable.

## 9. Result Replaces Exceptions and Error Return Pairs

Rust uses `Result<T, E>` for operations that can fail.

```rust
use std::fs;
use std::path::Path;

fn read_config(path: &Path) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}
```

Handling it:

```rust
match read_config(Path::new("config.toml")) {
    Ok(contents) => println!("{}", contents),
    Err(error) => eprintln!("failed to read config: {}", error),
}
```

Comparison:

```go
contents, err := os.ReadFile("config.toml")
if err != nil {
    return err
}
```

```python
try:
    contents = Path("config.toml").read_text()
except OSError as error:
    print(f"failed to read config: {error}")
```

```java
try {
    String contents = Files.readString(Path.of("config.toml"));
} catch (IOException error) {
    System.err.println("failed to read config: " + error);
}
```

Rust's `Result` is explicit in the function signature. A caller can see the failure path without reading the implementation.

## 10. The Question Mark Operator Propagates Errors

The `?` operator means: if this is `Err`, return the error from the current function; otherwise unwrap the `Ok` value.

```rust
use std::fs;
use std::path::Path;

fn read_two_files(left: &Path, right: &Path) -> Result<String, std::io::Error> {
    let left_contents = fs::read_to_string(left)?;
    let right_contents = fs::read_to_string(right)?;

    Ok(format!("{}\n{}", left_contents, right_contents))
}
```

This is the Rust equivalent of Go's repeated `if err != nil { return err }`, but checked through the type system.

## 11. Pattern Matching Is Exhaustive

Rust's `match` is like `switch`, but exhaustive.

```rust
fn describe_result(result: Result<u64, std::io::Error>) -> String {
    match result {
        Ok(bytes) => format!("{} bytes", bytes),
        Err(error) => format!("error: {}", error),
    }
}
```

If you forget a branch, the compiler rejects the program.

For one-case handling, use `if let`:

```rust
if let Some(name) = maybe_name {
    println!("{}", name);
}
```

## Practice Tasks

Use the project source code:

1. Find one function that takes ownership of a `Vec`.
2. Find one function that borrows a slice with `&[T]`.
3. Find a call to `.clone()` and explain why ownership requires it.
4. Find one `Option` chain and rewrite it as a `match`.
5. Find one `Result` handling block and compare it with Go's `if err != nil`.

## Part 2 Summary

Ownership is Rust's central tradeoff. You spend more time satisfying the compiler, but the result is code with fewer hidden runtime assumptions.

The most important habits:

- Borrow inputs when you only need to read.
- Take ownership when you need to store or consume.
- Clone only when you really need another owner.
- Use `Option<T>` instead of nullable values.
- Use `Result<T, E>` instead of exceptions or informal error conventions.

Part 3 applies these ideas to real CLI code: modules, traits, iterators, dependency choices, testing, and production habits.
