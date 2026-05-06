# Rust for Backend Developers: Simple Examples You Can Relate To

Rust can look strict when you first see it. The syntax is not the real challenge. The real shift is that Rust wants you to be clear about data, failure, and ownership before the service runs in production.

If you build APIs, workers, CLIs, cron jobs, or internal tools, Rust is easier to understand when you connect it to backend problems you already know:

- reading config
- validating input
- returning errors
- sharing application state
- handling optional database fields
- processing jobs
- cleaning old files

This article explains Rust through those familiar examples.

## 1. Think of Rust as a Strict Backend Teammate

In backend work, many bugs come from unclear assumptions:

- Is this value allowed to be missing?
- Can this function fail?
- Who is allowed to modify this data?
- Is this string still valid after this request ends?
- Did we handle the error or accidentally ignore it?

Rust pushes those questions into the code.

Instead of finding mistakes from logs later, the compiler catches many of them before the service starts.

## 2. A Simple Request Type

Backend developers often start with request and response models.

```rust
#[derive(Debug)]
struct CreateUserRequest {
    email: String,
    display_name: Option<String>,
    age: u8,
}
```

This says:

- `email` is required.
- `display_name` is optional.
- `age` is an unsigned 8-bit number.
- `Debug` lets us print the struct while debugging.

In JSON terms, this might represent:

```json
{
  "email": "dev@example.com",
  "display_name": "Dev",
  "age": 29
}
```

The important part is `Option<String>`. Rust does not use `null` casually. If a value can be missing, the type says so.

```rust
fn greeting(request: &CreateUserRequest) -> String {
    match &request.display_name {
        Some(name) => format!("Hello, {}!", name),
        None => format!("Hello, {}!", request.email),
    }
}
```

This is useful in real backend code because the compiler forces you to handle both cases.

## 3. Required Data vs Optional Data

A common backend mistake is assuming a database column, HTTP header, or config value exists.

Rust makes the missing case visible:

```rust
fn get_request_id(headers: &std::collections::HashMap<String, String>) -> Option<&String> {
    headers.get("x-request-id")
}
```

The return type is `Option<&String>`, which means:

- `Some(value)` if the header exists.
- `None` if it does not.

Using it:

```rust
let request_id = get_request_id(&headers)
    .map(String::as_str)
    .unwrap_or("unknown");
```

For production code, if you need to store the request ID beyond this scope, keep the fallback as an owned `String`:

```rust
let request_id = headers
    .get("x-request-id")
    .cloned()
    .unwrap_or_else(|| "unknown".to_string());
```

That looks slightly longer, but the behavior is explicit.

## 4. Errors Are Part of the Function Signature

Backend code fails all the time:

- config file missing
- database unavailable
- invalid request body
- permission denied
- network timeout

Rust uses `Result<T, E>` for this.

```rust
use std::fs;

fn read_config() -> Result<String, std::io::Error> {
    fs::read_to_string("config.toml")
}
```

This function does not hide failure. The signature says:

- success returns `String`
- failure returns `std::io::Error`

Handling it:

```rust
match read_config() {
    Ok(config) => println!("Loaded config: {}", config),
    Err(error) => eprintln!("Could not read config: {}", error),
}
```

This is similar to Go's explicit error handling, but the success and failure paths are wrapped in one type.

## 5. The Question Mark Operator Is Backend-Friendly

Backend functions often call several fallible operations in a row.

```rust
use std::fs;

fn load_template_and_config() -> Result<String, std::io::Error> {
    let config = fs::read_to_string("config.toml")?;
    let template = fs::read_to_string("email-template.html")?;

    Ok(format!("{}\n{}", config, template))
}
```

The `?` means:

- if the operation succeeds, give me the value
- if it fails, return the error from this function

It is a compact version of "check error and return early."

## 6. Borrowing Is Like Lending Data to a Function

In backend services, most functions do not need to own data. They just need to read it.

```rust
fn is_company_email(email: &str) -> bool {
    email.ends_with("@example.com")
}
```

`&str` means the function borrows the string. It can read the email, but it does not own it.

Using it:

```rust
let email = String::from("dev@example.com");

if is_company_email(&email) {
    println!("internal user");
}

println!("{}", email);
```

The original `email` is still usable after the function call.

This is one of the most important Rust habits:

- Use `&str` when reading text.
- Use `String` when storing or returning owned text.
- Use `&Path` when reading a path.
- Use `PathBuf` when storing or returning an owned path.

## 7. Ownership Is About Who Cleans Up

Think about a request body in a web server.

If a function only validates the body, it should borrow it:

```rust
fn validate_body(body: &str) -> bool {
    !body.trim().is_empty()
}
```

If a function stores the body for later, it should own it:

```rust
struct Job {
    payload: String,
}

fn create_job(payload: String) -> Job {
    Job { payload }
}
```

After `payload` moves into `Job`, the job owns it.

That is Rust's ownership model in backend terms: data should live exactly as long as the thing responsible for it.

## 8. Mutability Is Explicit

Backend state changes should be easy to spot.

```rust
let mut retry_count = 0;

retry_count += 1;
```

Without `mut`, Rust will not let you change the value.

This makes mutation visible during code review. If a variable changes, the code says so.

## 9. A Simple Job Processor Example

Here is a small example that feels like backend worker code.

```rust
#[derive(Debug)]
struct Job {
    id: u64,
    payload: String,
    attempts: u8,
}

fn process_job(job: &mut Job) -> Result<(), String> {
    if job.payload.trim().is_empty() {
        job.attempts += 1;
        return Err("empty payload".to_string());
    }

    println!("processed job {}", job.id);
    Ok(())
}
```

Notice the function signature:

```rust
fn process_job(job: &mut Job) -> Result<(), String>
```

It tells us:

- the function can modify the job
- the function can fail
- success returns nothing meaningful, just `()`
- failure returns an error message

That is a lot of useful backend behavior captured in one line.

## 10. Iterators Feel Like Data Pipelines

Backend developers often transform lists:

- filter active users
- collect expired sessions
- sum invoice totals
- map database rows into API responses

Rust iterators are good for this.

```rust
#[derive(Debug)]
struct Session {
    user_id: u64,
    expired: bool,
}

fn expired_user_ids(sessions: &[Session]) -> Vec<u64> {
    sessions
        .iter()
        .filter(|session| session.expired)
        .map(|session| session.user_id)
        .collect()
}
```

Read this as:

1. borrow the sessions
2. keep only expired sessions
3. take each `user_id`
4. collect the result into a `Vec<u64>`

This is similar to stream processing in Java or list comprehensions in Python, but Rust keeps ownership and borrowing rules checked.

## 11. Match Is Great for Business Rules

Backend code often has state machines:

- order status
- payment status
- job status
- user role

Rust enums and `match` make those rules explicit.

```rust
enum PaymentStatus {
    Pending,
    Paid,
    Failed(String),
}

fn message(status: PaymentStatus) -> String {
    match status {
        PaymentStatus::Pending => "payment is pending".to_string(),
        PaymentStatus::Paid => "payment complete".to_string(),
        PaymentStatus::Failed(reason) => format!("payment failed: {}", reason),
    }
}
```

If you add a new status later, Rust will force you to update every match that needs to handle it.

That is useful for backend systems where missing a state can become a production bug.

## 12. How This Connects to `artifact-cleaner`

This repository is a CLI, but the patterns are backend-friendly:

- scan a workspace like a backend service scans records
- collect artifacts like collecting rows from a database
- filter old folders like filtering expired sessions
- show a dry run like previewing a destructive migration
- require confirmation like protecting a production operation
- return results from cleanup like returning a job report

For example, a cleanup result is similar to a worker summary:

```rust
struct DeleteResult {
    deleted: usize,
    failed: Vec<String>,
    total_freed_bytes: u64,
}
```

This is the same idea as returning:

- number of processed jobs
- failed job IDs
- total records changed

Rust encourages you to model that result directly.

## 13. What Backend Developers Should Learn First

Do not start with the hardest Rust topics. Start with the parts that pay off immediately:

1. `struct` for request, response, config, and job models.
2. `Option<T>` for values that may be missing.
3. `Result<T, E>` for operations that can fail.
4. Borrowing with `&str`, `&Path`, and `&[T]`.
5. `Vec<T>` and iterator chains for list processing.
6. `match` for status and business rules.
7. `cargo test` for fast feedback.

You can learn advanced lifetimes, async runtimes, traits, and macros after these basics feel normal.

## Final Thought

Rust is not just a systems language. It is also a practical backend language when you care about correctness, performance, and clear failure handling.

For backend developers, the simplest way to understand Rust is this:

Rust makes hidden production assumptions visible in code.

That is why it feels strict at first, and why it becomes valuable as the codebase grows.
