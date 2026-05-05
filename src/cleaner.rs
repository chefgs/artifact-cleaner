// cleaner.rs — deletes artifact folders and reports results

use std::path::PathBuf;
use std::fs;
use humansize::{format_size, DECIMAL};

// ─── RUST LESSON — Structs with owned data ───────────────────────────────────
// DeleteResult owns all its data — the Vec<String> for failed paths.
// When DeleteResult goes out of scope, Rust automatically frees the memory.
// No garbage collector — the compiler tracks ownership at compile time.
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
#[allow(dead_code)]
pub struct DeleteResult {
    pub deleted: u32,
    pub failed: Vec<String>,
    pub total_freed_bytes: u64,  // kept for programmatic use (future GUI/JSON output)
    pub total_freed_human: String,
}

// ─── RUST LESSON — Taking ownership via Vec<PathBuf> ─────────────────────────
// We take `paths: Vec<PathBuf>` (owned) not `&Vec<PathBuf>` (borrowed).
// This means the caller transfers ownership to us — they can't use it after.
// It's fine here because after deletion, the caller has no use for the list.
//
// `dry_run: bool` — if true, simulate deletion without actually deleting.
// ─────────────────────────────────────────────────────────────────────────────
pub fn delete_artifacts(paths: Vec<PathBuf>, dry_run: bool) -> DeleteResult {
    let mut deleted: u32 = 0;
    let mut failed: Vec<String> = Vec::new();
    let mut total_freed_bytes: u64 = 0;

    for path in paths {
        // Compute size BEFORE deleting (after deletion, metadata is gone)
        let size = crate::scanner::compute_size(&path);

        if dry_run {
            // In dry-run mode, just count what would be freed
            deleted += 1;
            total_freed_bytes += size;
            continue;
        }

        // ─── RUST LESSON — Error handling with match ──────────────────────────
        // fs::remove_dir_all returns Result<(), std::io::Error>.
        // Ok(()) means success with no value (unit type — like void).
        // Err(e) means failure — we capture the error `e` and record it.
        // This forces us to handle the error — Rust won't let us ignore it.
        // ─────────────────────────────────────────────────────────────────────
        match fs::remove_dir_all(&path) {
            Ok(()) => {
                deleted += 1;
                total_freed_bytes += size;
            }
            Err(e) => {
                // .display() converts PathBuf to a printable string
                // format!() is Rust's string interpolation — like JS template literals
                failed.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    let total_freed_human = format_size(total_freed_bytes, DECIMAL);

    // ─── RUST LESSON — Struct literal return (implicit return) ───────────────
    // No `return` keyword — the last expression is the return value.
    // This is idiomatic Rust. The struct fields use shorthand when the
    // variable name matches the field name (same as JS object shorthand).
    // ─────────────────────────────────────────────────────────────────────────
    DeleteResult {
        deleted,
        failed,
        total_freed_bytes,
        total_freed_human,
    }
}
