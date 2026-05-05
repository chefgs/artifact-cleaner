// display.rs — terminal output formatting

use crate::cleaner::DeleteResult;
use crate::scanner::ArtifactFolder;
use colored::Colorize;
use humansize::{DECIMAL, format_size};

// ─── RUST LESSON — `use` imports ─────────────────────────────────────────────
// `use crate::scanner::ArtifactFolder` imports from our own scanner module.
// `crate` refers to the root of the current project (like `./` in JS imports).
// External crate items (colored, humansize) are available after adding to Cargo.toml.
// ─────────────────────────────────────────────────────────────────────────────

pub fn print_header(workspace: &str, months: u32, total: usize, total_bytes: u64) {
    println!();
    // ─── RUST LESSON — String formatting ─────────────────────────────────────
    // println! is a macro (note the !). Macros are identified by ! in Rust.
    // {} is the default format placeholder — calls the Display trait on the value.
    // {:>10} = right-align in 10 characters. {:<30} = left-align in 30 chars.
    // .bold(), .cyan(), .green() are methods from the `colored` crate,
    // added to &str via Rust's trait extension system.
    // ─────────────────────────────────────────────────────────────────────────
    println!("{}", "  Artifact Cleaner".bold().cyan());
    println!("  {}", "─".repeat(70).dimmed());
    println!("  Workspace  : {}", workspace.yellow());
    println!("  Threshold  : {} months", months);
    println!(
        "  Found      : {} folders · {}",
        total.to_string().bold(),
        format_size(total_bytes, DECIMAL).red().bold()
    );
    println!("  {}", "─".repeat(70).dimmed());
    println!();
}

pub fn print_results(artifacts: &[ArtifactFolder]) {
    if artifacts.is_empty() {
        println!("  {} No stale artifacts found.", "✓".green().bold());
        return;
    }

    // Column headers
    println!(
        "  {:<35} {:<14} {:>10}  {}",
        "Project".bold().underline(),
        "Type".bold().underline(),
        "Size".bold().underline(),
        "Last Modified".bold().underline(),
    );
    println!("  {}", "─".repeat(70).dimmed());

    // ─── RUST LESSON — Iterating with references ──────────────────────────────
    // `for artifact in artifacts` would move (take ownership of) each item.
    // `for artifact in artifacts.iter()` borrows each item as `&ArtifactFolder`.
    // Since we just want to read and print, borrowing is the right choice.
    // ─────────────────────────────────────────────────────────────────────────
    for artifact in artifacts.iter() {
        // Truncate long project names to fit the column
        let project = truncate(&artifact.project, 33);
        let artifact_type = color_artifact_type(&artifact.artifact_type);

        println!(
            "  {:<35} {:<14} {:>10}  {}",
            project,
            artifact_type,
            artifact.size_human.red(),
            artifact.last_modified.dimmed(),
        );
    }
    println!();
}

pub fn print_delete_result(result: &DeleteResult, dry_run: bool) {
    println!();
    if dry_run {
        println!(
            "  {} Dry run — {} folders would be deleted, freeing {}",
            "◉".yellow().bold(),
            result.deleted.to_string().bold(),
            result.total_freed_human.green().bold()
        );
    } else {
        println!(
            "  {} Deleted {} folders · freed {}",
            "✓".green().bold(),
            result.deleted.to_string().bold(),
            result.total_freed_human.green().bold()
        );
    }

    // ─── RUST LESSON — if !vec.is_empty() ────────────────────────────────────
    // .is_empty() is idiomatic Rust — preferred over .len() == 0.
    // `!` is the boolean NOT operator (same as JS).
    // ─────────────────────────────────────────────────────────────────────────
    if !result.failed.is_empty() {
        println!();
        println!("  {} Failed to delete:", "✗".red().bold());
        for f in &result.failed {
            println!("    - {}", f.red());
        }
    }
    println!();
}

// ─── RUST LESSON — Private helper functions ───────────────────────────────────
// No `pub` = private to this module. Clean API: callers only see print_* functions.
// `&str` return borrows from the input — no allocation needed when not truncating.
// But since we may build a new String, we return owned String for simplicity.
// ─────────────────────────────────────────────────────────────────────────────
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        // &s[..max-1] slices the string — Rust strings are UTF-8 byte arrays
        format!("{}…", &s[..max - 1])
    }
}

fn color_artifact_type(t: &str) -> colored::ColoredString {
    match t {
        "node_modules" => t.yellow(),
        ".next" => t.cyan(),
        ".terraform" => t.magenta(),
        "dist" => t.blue(),
        "build" => t.bright_blue(),
        _ => t.normal(),
    }
}
