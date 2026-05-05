// main.rs — CLI entry point for artifact-cleaner
//
// ─── RUST LESSON — mod declarations ──────────────────────────────────────────
// Rust does not auto-discover files. You must declare each module here.
// `mod scanner;` tells the compiler: "find src/scanner.rs and compile it."
// `use` then imports specific items from those modules into scope.
// ─────────────────────────────────────────────────────────────────────────────
mod scanner;
mod cleaner;
mod display;

use std::path::PathBuf;
use clap::Parser;
use colored::Colorize;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};

// ─── RUST LESSON — clap #[derive(Parser)] ────────────────────────────────────
// clap uses Rust's derive macro system to generate CLI argument parsing
// from a plain struct. No manual argument parsing code needed.
//
// #[derive(Parser)] generates a full CLI parser from this struct at compile time.
// #[command(...)] sets metadata shown in --help output.
// #[arg(...)] configures each field as a CLI flag or positional argument.
//
// Option<T> fields = optional arguments (not required).
// bool fields with `default_value_t = false` = --flag switches.
// Vec<String> = accepts multiple values: --types node_modules .next dist
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Parser, Debug)]
#[command(
    name = "artifact-cleaner",
    about = "Find and remove stale build artifacts to reclaim disk space",
    version = "0.1.0",
    long_about = "
Scans a workspace directory for stale build artifact folders
(node_modules, .next, dist, build, .terraform) and removes them
from projects that haven't been updated within a configurable threshold.

Python virtual environments (.venv, venv, env) are always excluded.
"
)]
struct Cli {
    /// Workspace directory to scan (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Delete artifacts from projects not updated in this many months
    #[arg(short, long, default_value_t = 2)]
    months: u32,

    /// Artifact folder types to target
    #[arg(
        short,
        long,
        value_delimiter = ',',
        default_values = scanner::DEFAULT_ARTIFACTS
    )]
    types: Vec<String>,

    /// Preview what would be deleted without actually deleting
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,

    /// Delete without asking for confirmation
    #[arg(short, long, default_value_t = false)]
    yes: bool,

    /// Show all found artifacts without prompting (non-interactive)
    #[arg(long, default_value_t = false)]
    no_interactive: bool,
}

// ─── RUST LESSON — fn main() ─────────────────────────────────────────────────
// Every Rust binary starts at fn main().
// Unlike many languages, main() returns () (unit — nothing).
// If we need to propagate errors from main, we'd use `-> Result<(), Box<dyn Error>>`.
// std::process::exit(1) terminates with a non-zero exit code (signals failure
// to the shell — same convention as bash scripts).
// ─────────────────────────────────────────────────────────────────────────────
fn main() {
    // Cli::parse() reads argv, validates it against our struct, and returns
    // a populated Cli instance — or prints help/error and exits automatically.
    let cli = Cli::parse();

    // Resolve the workspace path to an absolute canonical path.
    // canonicalize() follows symlinks and resolves relative paths.
    // unwrap_or_else runs only if canonicalize() returns Err — graceful fallback.
    let workspace = cli.path.canonicalize().unwrap_or_else(|_| {
        eprintln!("{} Path not found: {}", "Error:".red().bold(), cli.path.display());
        std::process::exit(1);
    });

    if !workspace.is_dir() {
        eprintln!("{} Not a directory: {}", "Error:".red().bold(), workspace.display());
        std::process::exit(1);
    }

    // ─── RUST LESSON — Progress spinner ──────────────────────────────────────
    // indicatif::ProgressBar renders a live spinner in the terminal.
    // ProgressStyle::with_template sets the format string.
    // .tick_strings() defines animation frames for the spinner.
    // .enable_steady_tick() starts the animation on a background thread.
    // ─────────────────────────────────────────────────────────────────────────
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    spinner.set_message(format!(
        "Scanning {} (threshold: {} months)...",
        workspace.display(),
        cli.months
    ));

    // Run the scan — this is where scanner.rs does the heavy lifting
    let artifacts = scanner::scan_workspace(&workspace, cli.months, &cli.types);

    spinner.finish_and_clear();

    if artifacts.is_empty() {
        println!();
        println!("  {} No stale artifacts found in {}", "✓".green().bold(), workspace.display());
        println!();
        return;
    }

    // Compute totals for the summary header
    // .iter().map().sum() = functional sum over the Vec
    let total_bytes: u64 = artifacts.iter().map(|a| a.size_bytes).sum();

    display::print_header(
        &workspace.to_string_lossy(),
        cli.months,
        artifacts.len(),
        total_bytes,
    );
    display::print_results(&artifacts);

    if cli.dry_run {
        // Dry run: show what would be deleted, then exit
        let paths: Vec<PathBuf> = artifacts.iter().map(|a| a.path.clone()).collect();
        let result = cleaner::delete_artifacts(paths, true);
        display::print_delete_result(&result, true);
        return;
    }

    if cli.no_interactive {
        return;
    }

    // ─── RUST LESSON — Conditional confirmation prompt ────────────────────────
    // `cli.yes` short-circuits the prompt — useful for scripts/CI.
    // Confirm::new() from dialoguer renders an interactive [y/N] prompt.
    // .interact() blocks until the user answers — returns Result<bool>.
    // unwrap_or(false) treats any error (e.g. non-interactive terminal) as No.
    // ─────────────────────────────────────────────────────────────────────────
    let confirmed = cli.yes || Confirm::new()
        .with_prompt(format!(
            "  Delete {} folders and free {}?",
            artifacts.len(),
            humansize::format_size(total_bytes, humansize::DECIMAL)
        ))
        .default(false)
        .interact()
        .unwrap_or(false);

    if !confirmed {
        println!("  {} Aborted — nothing deleted.", "✗".yellow());
        println!();
        return;
    }

    // Collect paths to delete — .map() transforms, .collect() gathers into Vec
    let paths: Vec<PathBuf> = artifacts.into_iter().map(|a| a.path).collect();
    let result = cleaner::delete_artifacts(paths, false);
    display::print_delete_result(&result, false);
}
