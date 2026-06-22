// display.rs — terminal output for the mac-lib subcommand

use super::scanner::{EntryStatus, LibraryEntry};
use colored::Colorize;
use humansize::{DECIMAL, format_size};

pub fn print_header(total: usize, orphaned: usize, orphaned_bytes: u64, min_size_mb: u64) {
    println!();
    println!("{}", "  Mac Library Scanner".bold().cyan());
    println!("  {}", "─".repeat(72).dimmed());
    println!("  Threshold  : {} MB minimum size", min_size_mb);
    println!(
        "  Found      : {} items scanned · {} orphaned · {} recoverable",
        total.to_string().bold(),
        orphaned.to_string().yellow().bold(),
        format_size(orphaned_bytes, DECIMAL).red().bold()
    );
    println!("  {}", "─".repeat(72).dimmed());
    println!();
}

// ─── RUST LESSON — &[T] slices vs Vec<T> ─────────────────────────────────────
// Function parameters use `&[LibraryEntry]` (a slice) rather than `&Vec<LibraryEntry>`.
// A slice is a view into any contiguous sequence — Vec, array, or sub-range.
// Accepting `&[T]` makes the function more general: it works with Vec, arrays,
// and slices from other sources without any change to the caller.
// The compiler automatically coerces `&Vec<T>` → `&[T]` at call sites.
// ─────────────────────────────────────────────────────────────────────────────
pub fn print_results(entries: &[LibraryEntry]) {
    if entries.is_empty() {
        println!(
            "  {} No items found above the size threshold.",
            "✓".green().bold()
        );
        return;
    }

    println!(
        "  {:<18} {:<40} {:>9}  {}",
        "Directory".bold().underline(),
        "Name".bold().underline(),
        "Size".bold().underline(),
        "Status".bold().underline(),
    );
    println!("  {}", "─".repeat(72).dimmed());

    for entry in entries {
        let name = truncate(&entry.name, 38);
        let source = truncate(&entry.source, 16);
        let (status_label, size_colored) = format_status(&entry.status, &entry.size_human);

        println!(
            "  {:<18} {:<40} {:>9}  {}",
            source.dimmed(),
            name,
            size_colored,
            status_label,
        );
    }
    println!();
}

pub fn print_caution_note(active_count: usize, unknown_count: usize) {
    if active_count > 0 {
        println!(
            "  {} {} active item(s) shown above are oversized but will NOT be deleted.",
            "◉".yellow(),
            active_count.to_string().bold()
        );
        println!("    Remove them manually only if you are certain the app data is regenerable.");
    }

    if unknown_count > 0 {
        println!(
            "  {} {} ambiguous item(s) are shown as unknown and will NOT be deleted automatically.",
            "?".normal(),
            unknown_count.to_string().bold()
        );
        println!("    These often include shared containers, helper data, and named caches.");
    }

    if active_count > 0 || unknown_count > 0 {
        println!();
    }
}

pub fn print_delete_result(deleted: u32, freed_bytes: u64, failed: &[String], dry_run: bool) {
    println!();
    if dry_run {
        println!(
            "  {} Dry run — {} items would be deleted, freeing {}",
            "◉".yellow().bold(),
            deleted.to_string().bold(),
            format_size(freed_bytes, DECIMAL).green().bold()
        );
    } else {
        println!(
            "  {} Deleted {} items · freed {}",
            "✓".green().bold(),
            deleted.to_string().bold(),
            format_size(freed_bytes, DECIMAL).green().bold()
        );
    }

    if !failed.is_empty() {
        println!();
        println!("  {} Failed to delete:", "✗".red().bold());
        for f in failed {
            println!("    - {}", f.red());
        }
    }
    println!();
}

// ─── RUST LESSON — Tuple return types ────────────────────────────────────────
// Rust functions can return multiple values as a tuple: `(A, B)`.
// The caller destructures it: `let (label, color) = format_status(...)`.
// This is cheaper than allocating a struct for a short-lived pair.
// `colored::ColoredString` is an owned type — it carries both the string
// content and the ANSI escape codes. Returning it (not a reference) is correct
// because the value is created inside this function and has no owner above it.
// ─────────────────────────────────────────────────────────────────────────────
fn format_status(
    status: &EntryStatus,
    size_human: &str,
) -> (colored::ColoredString, colored::ColoredString) {
    match status {
        EntryStatus::OrphanedApp => ("● orphaned-app".red(), size_human.red()),
        EntryStatus::ActiveOversized => ("◉ active-oversized".yellow(), size_human.yellow()),
        EntryStatus::Unknown => ("? unknown".normal(), size_human.normal()),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
