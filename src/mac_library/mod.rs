// mac_library — scans ~/Library/{Caches,Containers,Group Containers}
// for orphaned or oversized data left by uninstalled apps and CLI tools.
//
// Only supported on macOS (uses mdfind / which).

#[cfg(target_os = "macos")]
mod checker;
#[cfg(target_os = "macos")]
mod display;
#[cfg(target_os = "macos")]
mod resolver;
#[cfg(target_os = "macos")]
mod scanner;

use crate::MacLibArgs;
use colored::Colorize;
#[cfg(target_os = "macos")]
use dialoguer::Confirm;
#[cfg(target_os = "macos")]
use humansize::{DECIMAL, format_size};
#[cfg(target_os = "macos")]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(target_os = "macos")]
use scanner::EntryStatus;
#[cfg(target_os = "macos")]
use std::fs;
#[cfg(target_os = "macos")]
use std::io::IsTerminal;

#[cfg(not(target_os = "macos"))]
pub fn run(_args: &MacLibArgs) {
    eprintln!(
        "{} mac-lib is only supported on macOS.",
        "Error:".red().bold()
    );
    std::process::exit(1);
}

// ─── RUST LESSON — #[cfg(...)] conditional compilation ───────────────────────
// `#[cfg(target_os = "macos")]` is evaluated at COMPILE time, not runtime.
// The function below is only compiled into macOS builds.
// This is different from an `if` statement — the excluded code is never compiled,
// so it can reference macOS-only APIs without causing linker errors on Linux.
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(target_os = "macos")]
pub fn run(args: &MacLibArgs) {
    let min_size_bytes = args.min_size * 1024 * 1024;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    spinner.set_message(format!(
        "Scanning ~/Library/{} (min size: {} MB)...",
        args.dirs.join(", "),
        args.min_size
    ));

    let entries = scanner::scan(min_size_bytes, &args.dirs);

    spinner.finish_and_clear();

    if entries.is_empty() {
        println!();
        println!(
            "  {} No items found above {} MB threshold.",
            "✓".green().bold(),
            args.min_size
        );
        println!();
        return;
    }

    // ─── RUST LESSON — matches! macro ────────────────────────────────────────
    // `matches!(expr, pattern)` returns true if `expr` matches `pattern`.
    // It is equivalent to writing `if let pattern = expr { true } else { false }`,
    // but far more readable inside closures and filter chains.
    // The `|` inside matches! is a pattern OR — not a bitwise OR.
    // ─────────────────────────────────────────────────────────────────────────
    let orphaned: Vec<_> = entries
        .iter()
        .filter(|e| matches!(e.status, EntryStatus::OrphanedApp))
        .collect();

    let active_count = entries
        .iter()
        .filter(|e| matches!(e.status, EntryStatus::ActiveOversized))
        .count();

    let unknown_count = entries
        .iter()
        .filter(|e| matches!(e.status, EntryStatus::Unknown))
        .count();

    let orphaned_bytes: u64 = orphaned.iter().map(|e| e.size_bytes).sum();

    display::print_header(entries.len(), orphaned.len(), orphaned_bytes, args.min_size);
    display::print_results(&entries);
    display::print_caution_note(active_count, unknown_count);

    if orphaned.is_empty() {
        println!("  {} No orphaned items to delete.", "✓".green().bold());
        println!();
        return;
    }

    if args.dry_run {
        display::print_delete_result(orphaned.len() as u32, orphaned_bytes, &[], true);
        return;
    }

    if !std::io::stdin().is_terminal() {
        println!(
            "  {} Cannot confirm deletion in a non-interactive terminal. Nothing was deleted.",
            "✗".yellow()
        );
        println!();
        return;
    }

    let confirmed = Confirm::new()
        .with_prompt(format!(
            "  Delete {} orphaned items and free {}?",
            orphaned.len(),
            format_size(orphaned_bytes, DECIMAL)
        ))
        .default(false)
        .interact()
        .unwrap_or(false);

    if !confirmed {
        println!("  {} Aborted — nothing deleted.", "✗".yellow());
        println!();
        return;
    }

    let mut deleted: u32 = 0;
    let mut freed_bytes: u64 = 0;
    let mut failed: Vec<String> = Vec::new();

    // ─── RUST LESSON — for entry in &orphaned ────────────────────────────────
    // `for entry in &orphaned` borrows each element as `&&LibraryEntry`.
    // We use `&orphaned` (not `orphaned`) because we still need `orphaned`
    // after this loop to count totals — consuming it here would move it.
    // In Rust, iterating a Vec by value moves and drops each element.
    // Iterating by reference (`&vec`) borrows — the Vec remains usable after.
    // ─────────────────────────────────────────────────────────────────────────
    for entry in &orphaned {
        match fs::remove_dir_all(&entry.path) {
            Ok(()) => {
                deleted += 1;
                freed_bytes += entry.size_bytes;
            }
            Err(e) => {
                failed.push(format!("{}: {}", entry.path.display(), e));
            }
        }
    }

    display::print_delete_result(deleted, freed_bytes, &failed, false);
}
