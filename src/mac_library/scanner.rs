// scanner.rs — walks Library subdirectories and classifies each entry

// ─── RUST LESSON — super:: imports ───────────────────────────────────────────
// `super::` refers to the parent module — here, `mac_library`.
// `super::checker` = mac_library::checker (sibling module).
// `crate::` would refer to the root of the whole project (main.rs).
// This mirrors filesystem paths: `super` = `..`, `crate` = project root.
// ─────────────────────────────────────────────────────────────────────────────
use super::checker;
use super::resolver::{self, EntryKind, FolderSource};
use humansize::{DECIMAL, format_size};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum EntryStatus {
    /// App not installed → safe to delete
    OrphanedApp,
    /// App IS installed but cache is oversized — shown as a caution item only
    ActiveOversized,
    /// Cannot determine ownership with high confidence → shown, not deleted
    Unknown,
}

#[derive(Debug, Clone)]
pub struct LibraryEntry {
    pub path: PathBuf,
    pub name: String,
    pub source: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub status: EntryStatus,
}

pub fn scan(min_size_bytes: u64, dirs: &[String]) -> Vec<LibraryEntry> {
    let home = match home_dir() {
        Some(h) => h,
        None => return vec![],
    };

    let mut results: Vec<LibraryEntry> = Vec::new();

    for dir in dirs {
        match dir.as_str() {
            "caches" => {
                let path = home.join("Library/Caches");
                results.extend(scan_dir(
                    &path,
                    FolderSource::Caches,
                    "Caches",
                    min_size_bytes,
                ));
            }
            "containers" => {
                let path = home.join("Library/Containers");
                results.extend(scan_dir(
                    &path,
                    FolderSource::Containers,
                    "Containers",
                    min_size_bytes,
                ));
            }
            "groups" => {
                let path = home.join("Library/Group Containers");
                results.extend(scan_dir(
                    &path,
                    FolderSource::GroupContainers,
                    "Group Containers",
                    min_size_bytes,
                ));
            }
            _ => {}
        }
    }

    // ─── RUST LESSON — Closures capturing the environment ────────────────────
    // `sort_by(|a, b| ...)` takes a closure that borrows `a` and `b` by reference.
    // Inside, `priority` is itself a closure — a closure defined inside a closure.
    // `priority` captures nothing from the outer scope, so it has zero overhead.
    // `.then()` on `Ordering` chains comparisons: if the first comparison is Equal,
    // it falls through to the second. This gives us a two-level sort in one expression.
    // ─────────────────────────────────────────────────────────────────────────
    results.sort_by(|a, b| {
        let priority = |s: &EntryStatus| match s {
            EntryStatus::OrphanedApp => 0,
            EntryStatus::Unknown => 1,
            EntryStatus::ActiveOversized => 2,
        };
        priority(&a.status)
            .cmp(&priority(&b.status))
            .then(b.size_bytes.cmp(&a.size_bytes))
    });

    results
}

fn scan_dir(
    dir: &Path,
    source: FolderSource,
    label: &str,
    min_size_bytes: u64,
) -> Vec<LibraryEntry> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut results = Vec::new();

    // ─── RUST LESSON — .flatten() on iterators of Results ────────────────────
    // `fs::read_dir` returns an iterator of `Result<DirEntry, Error>`.
    // `.flatten()` is shorthand for `.filter_map(|e| e.ok())` — it discards
    // any Err variants and unwraps Ok variants, giving a clean `DirEntry` stream.
    // This is idiomatic Rust: handle errors lazily in the iterator pipeline
    // rather than unwrapping and risking a panic.
    // ─────────────────────────────────────────────────────────────────────────
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let kind = resolver::classify(&name, &source);

        // ─── RUST LESSON — Pattern matching with guards ───────────────────────
        // `if let` binds the inner value only when the pattern matches.
        // This avoids a nested match and keeps the happy path flat.
        // ─────────────────────────────────────────────────────────────────────
        if let EntryKind::Excluded = kind {
            continue;
        }

        let size_bytes = compute_size(&path);
        if size_bytes < min_size_bytes {
            continue;
        }

        let status = resolve_status(&kind);
        if let Some(status) = status {
            results.push(LibraryEntry {
                path,
                name,
                source: label.to_string(),
                size_bytes,
                size_human: format_size(size_bytes, DECIMAL),
                status,
            });
        }
    }

    results
}

fn resolve_status(kind: &EntryKind) -> Option<EntryStatus> {
    match kind {
        EntryKind::BundleId(id) => {
            if checker::is_bundle_id_installed(id) {
                Some(EntryStatus::ActiveOversized)
            } else {
                Some(EntryStatus::OrphanedApp)
            }
        }
        EntryKind::HelperBundle(bundle_ids) => {
            if checker::is_helper_bundle_active(bundle_ids) {
                Some(EntryStatus::ActiveOversized)
            } else {
                Some(EntryStatus::Unknown)
            }
        }
        EntryKind::SharedContainer(bundle_ids) => {
            if checker::is_any_bundle_id_installed(bundle_ids) {
                Some(EntryStatus::ActiveOversized)
            } else {
                Some(EntryStatus::Unknown)
            }
        }
        EntryKind::NamedCache(name) => {
            if checker::is_named_cache_active(name) {
                Some(EntryStatus::ActiveOversized)
            } else {
                Some(EntryStatus::Unknown)
            }
        }
        EntryKind::Excluded => None,
    }
}

fn compute_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

// ─── RUST LESSON — Option chaining with .ok() and .map() ────────────────────
// `std::env::var("HOME")` returns `Result<String, VarError>`.
// `.ok()` converts Result → Option, turning Err into None.
// `.map(PathBuf::from)` transforms the inner String into a PathBuf —
// only if the Option is Some. If it is None, map is a no-op.
// This entire chain never panics — it returns None when HOME is unset.
// ─────────────────────────────────────────────────────────────────────────────
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
