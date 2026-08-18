// main.rs — CLI entry point for artifact-cleaner
//
// ─── RUST LESSON — mod declarations ──────────────────────────────────────────
// Rust does not auto-discover files. You must declare each module here.
// `mod scanner;` tells the compiler: "find src/scanner.rs and compile it."
// `use` then imports specific items from those modules into scope.
// ─────────────────────────────────────────────────────────────────────────────
mod cleaner;
mod display;
mod mac_library;
mod scanner;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

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
    name = env!("CARGO_BIN_NAME"),
    about = "Reclaim disk space — remove stale build artifacts and unused macOS Library data",
    version = env!("CARGO_PKG_VERSION"),
    long_about = "
Scans a workspace directory for stale build artifact folders
(node_modules, .next, dist, build, .terraform) and removes them
from projects that haven't been updated within a configurable threshold.

Use `size` inside a project to inspect current artifact folder sizes
without stale filtering or deletion prompts.

Use `--types` with a comma-separated list to target additional directory
names, such as target, coverage, .gradle, or __pycache__.
`scan` and `size` match directories only; they do not select individual
files or filter project artifacts by a minimum size.

Python virtual environments (.venv, venv, env) are always excluded.

`artifact-cleaner` and `afc` are interchangeable. Run either name with
`<command> --help` for command-specific options and examples.
"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// ─── RUST LESSON — Subcommands ───────────────────────────────────────────────
// #[derive(Subcommand)] generates a clap subcommand enum.
// Each variant becomes a subcommand: `afc scan`, `afc mac-lib`.
// #[derive(Args)] on a separate struct lets each subcommand own its flags.
// This is cleaner than one giant struct as commands grow.
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Subcommand, Debug)]
enum Commands {
    /// Find and remove stale artifact directories in a workspace
    Scan(ScanArgs),
    /// Show artifact directory sizes directly under a project directory
    Size(SizeArgs),
    /// Scan macOS Library folders for orphaned app/tool data (macOS only)
    MacLib(MacLibArgs),
}

#[derive(Args, Debug)]
#[command(
    after_help = "EXAMPLES (full CLI name):\n  artifact-cleaner scan ~/projects --dry-run\n  artifact-cleaner scan ~/projects --months 6 --types target,coverage,.gradle\n\nSHORT ALIAS:\n  afc scan . --types node_modules,.next --yes\n\nNOTES:\n  artifact-cleaner and afc are interchangeable.\n  --types accepts comma-separated directory names only.\n  There is no minimum-size filter for workspace artifacts."
)]
struct ScanArgs {
    /// Workspace directory to scan (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Delete artifacts from projects not updated in this many months
    #[arg(short, long, default_value_t = 2)]
    months: u32,

    /// Comma-separated artifact directory names to target
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

#[derive(Args, Debug)]
#[command(
    after_help = "EXAMPLES (full CLI name):\n  artifact-cleaner size\n\nSHORT ALIAS:\n  afc size ~/projects/my-app --types target,node_modules,coverage\n\nNOTES:\n  artifact-cleaner and afc are interchangeable.\n  --types accepts directory names only; individual files are not reported."
)]
struct SizeArgs {
    /// Directory to inspect (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Comma-separated artifact directory names to size
    #[arg(
        short,
        long,
        value_delimiter = ',',
        default_values = scanner::DEFAULT_ARTIFACTS
    )]
    types: Vec<String>,
}

#[derive(Args, Debug)]
#[command(
    after_help = "EXAMPLES (full CLI name):\n  artifact-cleaner mac-lib --dry-run\n\nSHORT ALIAS:\n  afc mac-lib --min-size 500 --dirs caches,containers\n\nNOTE:\n  artifact-cleaner and afc are interchangeable.\n  --min-size is measured in MB and applies only to macOS Library entries,\n  not to the workspace artifacts selected by scan or size."
)]
pub struct MacLibArgs {
    /// Only flag items larger than this size in MB
    #[arg(long, default_value_t = 100)]
    pub min_size: u64,

    /// Directories to scan — any combination of: caches, containers, groups
    #[arg(
        long,
        value_delimiter = ',',
        default_values = &["caches", "containers", "groups"]
    )]
    pub dirs: Vec<String>,

    /// Preview what would be deleted without actually deleting
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    /// Accepted for compatibility; mac-lib still asks for confirmation before deleting
    #[arg(short, long, default_value_t = false)]
    pub yes: bool,
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

    match cli.command {
        Commands::Scan(args) => run_scan(args),
        Commands::Size(args) => run_size(args),
        Commands::MacLib(args) => mac_library::run(&args),
    }
}

// Resolve the workspace path to an absolute canonical path.
// canonicalize() follows symlinks and resolves relative paths.
// unwrap_or_else runs only if canonicalize() returns Err — graceful fallback.
fn run_scan(cli: ScanArgs) {
    let workspace = cli.path.canonicalize().unwrap_or_else(|_| {
        eprintln!(
            "{} Path not found: {}",
            "Error:".red().bold(),
            cli.path.display()
        );
        std::process::exit(1);
    });

    if !workspace.is_dir() {
        eprintln!(
            "{} Not a directory: {}",
            "Error:".red().bold(),
            workspace.display()
        );
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
        println!(
            "  {} No stale artifacts found in {}",
            "✓".green().bold(),
            workspace.display()
        );
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
    let confirmed = cli.yes
        || Confirm::new()
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

    let paths: Vec<PathBuf> = artifacts.into_iter().map(|a| a.path).collect();
    let result = cleaner::delete_artifacts(paths, false);
    display::print_delete_result(&result, false);
}

fn run_size(cli: SizeArgs) {
    let directory = cli.path.canonicalize().unwrap_or_else(|_| {
        eprintln!(
            "{} Path not found: {}",
            "Error:".red().bold(),
            cli.path.display()
        );
        std::process::exit(1);
    });

    if !directory.is_dir() {
        eprintln!(
            "{} Not a directory: {}",
            "Error:".red().bold(),
            directory.display()
        );
        std::process::exit(1);
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    spinner.set_message(format!("Sizing artifacts in {}...", directory.display()));

    let sizes = scanner::scan_current_directory(&directory, &cli.types);

    spinner.finish_and_clear();

    if sizes.is_empty() {
        println!();
        println!(
            "  {} No matching artifact folders found in {}",
            "✓".green().bold(),
            directory.display()
        );
        println!();
        return;
    }

    let total_bytes: u64 = sizes.iter().map(|s| s.size_bytes).sum();
    display::print_size_header(&directory.to_string_lossy(), sizes.len(), total_bytes);
    display::print_size_results(&sizes);
}
