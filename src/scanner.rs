// scanner.rs — finds stale artifact folders in a workspace

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

// ─── RUST LESSON — Structs ────────────────────────────────────────────────────
// A struct groups related data — like a TypeScript interface but with ownership.
// #[derive(Debug, Clone)] auto-generates:
//   - Debug: lets you print it with {:?}
//   - Clone: lets you duplicate it with .clone()
// `pub` makes the field accessible outside this module.
// String = heap-allocated, owned string. &str = borrowed string reference.
// u64 = unsigned 64-bit integer (always positive — good for byte counts).
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct ArtifactFolder {
    pub path: PathBuf,
    pub artifact_type: String,
    pub project: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub last_modified: String,
}

// ─── RUST LESSON — Constants ──────────────────────────────────────────────────
// `const` is evaluated at compile time.
// `&[&str]` is a slice (a view into a fixed array) of borrowed string refs.
// ─────────────────────────────────────────────────────────────────────────────
pub const DEFAULT_ARTIFACTS: &[&str] = &["node_modules", ".next", "dist", "build", ".terraform"];

const EXCLUDED_PATHS: &[&str] = &[
    "/.venv/",
    "/venv/",
    "/env/",
    "/lib/python",
    "/site-packages/",
    "/.git/",
];

// ─── RUST LESSON — Functions ──────────────────────────────────────────────────
// `pub fn` = public function visible outside this module.
// `path: &Path` = borrowed reference — we read it, don't own it.
// `months: u32` = unsigned 32-bit int, passed by value (copied).
// `artifact_types: &[String]` = borrowed slice of owned Strings.
// `-> Vec<ArtifactFolder>` = returns a heap-allocated list of structs.
// The last expression in a function is the return value (no `return` keyword needed).
// ─────────────────────────────────────────────────────────────────────────────
pub fn scan_workspace(path: &Path, months: u32, artifact_types: &[String]) -> Vec<ArtifactFolder> {
    // SystemTime::now() - Duration = cutoff timestamp.
    // checked_sub returns Option<SystemTime> (None if it would underflow).
    // unwrap_or gives a safe fallback value when the Option is None.
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(months as u64 * 30 * 24 * 3600))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut results: Vec<ArtifactFolder> = Vec::new();

    // ─── RUST LESSON — Iterators ──────────────────────────────────────────────
    // Rust iterators are lazy — nothing runs until .collect() or .for_each().
    // .filter_map(|e| e.ok()) = transform + filter in one step:
    //   maps each Result<T,E> to Option<T>, then drops None (errors).
    // .collect::<Vec<_>>() gathers iterator results into a Vec.
    // ─────────────────────────────────────────────────────────────────────────
    let project_dirs: Vec<_> = WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .collect();

    for project_entry in project_dirs {
        let project_path = project_entry.path();

        // ─── RUST LESSON — match (pattern matching) ───────────────────────────
        // `match` is Rust's switch — but exhaustive (must handle ALL cases).
        // Result<T, E> has two variants: Ok(value) and Err(error).
        // `continue` skips to the next loop iteration (same as JS).
        // ─────────────────────────────────────────────────────────────────────
        let project_mtime = match fs::metadata(project_path) {
            Ok(meta) => match meta.modified() {
                Ok(t) => t,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if project_mtime >= cutoff {
            continue;
        }

        for entry in WalkDir::new(project_path)
            .min_depth(1)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let entry_path = entry.path();
            let path_str = entry_path.to_string_lossy();

            // .any() returns true if the closure is true for ANY element — like JS Array.some()
            if EXCLUDED_PATHS.iter().any(|ex| path_str.contains(ex)) {
                continue;
            }

            // file_name() returns Option<&OsStr> — and_then chains Option transforms
            let folder_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if !artifact_types.iter().any(|t| t == folder_name) {
                continue;
            }

            // Skip nested artifacts (e.g. node_modules/lodash/node_modules).
            // Check the PARENT path (not including the folder itself) — if any
            // artifact type name already appears above this folder, it is nested.
            let parent_str = entry_path
                .parent()
                .map(|p| p.to_string_lossy())
                .unwrap_or_default();
            if artifact_types
                .iter()
                .any(|t| parent_str.contains(t.as_str()))
            {
                continue;
            }

            let size_bytes = compute_size(entry_path);
            let size_human = format_size(size_bytes, DECIMAL);
            let last_modified = format_mtime(project_mtime);

            // ─── RUST LESSON — Closures ───────────────────────────────────────
            // unwrap_or_else takes a closure (anonymous function) as fallback.
            // || is Rust closure syntax — equivalent to () => in JavaScript.
            // ─────────────────────────────────────────────────────────────────
            let project_name = project_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // .push() appends to a Vec — like JS Array.push()
            results.push(ArtifactFolder {
                path: entry_path.to_path_buf(),
                artifact_type: folder_name.to_string(),
                project: project_name,
                size_bytes,
                size_human,
                last_modified,
            });
        }
    }

    // Sort by size descending — biggest space wasters first
    results.sort_by_key(|b| std::cmp::Reverse(b.size_bytes));
    results
}

// ─── RUST LESSON — Iterator chains as computation ─────────────────────────────
// This function sums file sizes with a single expression — no loops, no mutation.
// Each step transforms the iterator without allocating intermediate collections.
// .sum() works because u64 implements the `Sum` trait.
// ─────────────────────────────────────────────────────────────────────────────
pub fn compute_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

fn format_mtime(mtime: SystemTime) -> String {
    // SystemTime → chrono DateTime for human-readable formatting.
    // .into() converts between compatible types — Rust infers the target type.
    let datetime: DateTime<Local> = mtime.into();
    datetime.format("%Y-%m-%d").to_string()
}
