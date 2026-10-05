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
// ─── User-facing help text ───────────────────────────────────────────────────
// Shown at the bottom of `afc --help`. Plain consts keep the layout readable
// and make it easy to keep examples in sync with the real flags.
// ─────────────────────────────────────────────────────────────────────────────
const QUICKSTART: &str = "\
QUICK START (safe, nothing is deleted until you confirm):
  afc size .                    # 1. how big are the artifacts in this project?
  afc scan ~/projects --dry-run # 2. preview stale artifacts across a workspace
  afc scan ~/projects           # 3. review the list, then answer y/N to delete
  afc mac-lib --dry-run         # (macOS) preview orphaned Library data

COMMON TASKS:
  Free space across all projects      afc scan ~/projects
  Only look, never delete             afc scan ~/projects --dry-run
  Be more or less aggressive          afc scan ~/projects --months 6
  Clean other folders too             afc scan . --types target,coverage,.gradle
  Script / CI use (no prompt)         afc scan . --yes
  Report only, no prompt              afc scan . --no-interactive
  Large macOS Library leftovers       afc mac-lib --min-size 500

GOOD TO KNOW:
  - A project is \"stale\" when it has not changed in --months months (default 2).
  - Default targets: node_modules, .next, dist, build, .terraform.
  - Python virtual environments (.venv, venv, env) are always skipped.
  - `afc` and `artifact-cleaner` are the same program.

Run `afc <command> --help` for details and examples.";

const SCAN_HELP: &str = "\
WHAT IT DOES:
  Walks the workspace, finds artifact folders (node_modules, dist, ...) inside
  projects that have not been updated recently, shows their sizes, and asks
  before deleting.

EXAMPLES:
  afc scan ~/projects --dry-run                  preview only, deletes nothing
  afc scan ~/projects                            review, then confirm [y/N]
  afc scan ~/projects --months 6                 only projects idle for 6+ months
  afc scan . --types target,coverage,.gradle     choose which folders to target
  afc scan . --types node_modules,.next --yes    no prompt (scripts, CI)
  afc scan . --no-interactive                    list results, never prompt

NOTES:
  Run with --dry-run first. Deletion cannot be undone.
  --types takes directory names only, comma-separated. There is no size filter.";

const SIZE_HELP: &str = "\
WHAT IT DOES:
  Reports the size of artifact folders directly under one project. Read-only:
  it never deletes and ignores how recently the project changed.

EXAMPLES:
  afc size                                       current directory
  afc size ~/projects/my-app                     a specific project
  afc size . --types target,node_modules,coverage

NOTES:
  --types takes directory names only; individual files are not reported.";

const MAC_LIB_HELP: &str = "\
WHAT IT DOES:
  (macOS only) Looks in ~/Library for caches, containers and group data left
  behind by apps that are no longer installed, and offers to remove them.
  Oversized caches of apps that ARE installed are shown as caution items only.

EXAMPLES:
  afc mac-lib --dry-run                          preview only, deletes nothing
  afc mac-lib                                    review, then confirm
  afc mac-lib --min-size 500                     only items larger than 500 MB
  afc mac-lib --dirs caches,containers           limit which folders are checked

NOTES:
  --min-size is in MB and applies only to Library entries, not to scan/size.
  It always asks for confirmation before deleting; --yes is accepted for
  compatibility but does not skip the prompt.";

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_BIN_NAME"),
    about = "Reclaim disk space: remove stale build artifacts and unused macOS Library data",
    version = env!("CARGO_PKG_VERSION"),
    arg_required_else_help = true,
    after_help = QUICKSTART,
    after_long_help = QUICKSTART
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
#[command(after_long_help = SCAN_HELP, after_help = SCAN_HELP)]
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
#[command(after_long_help = SIZE_HELP, after_help = SIZE_HELP)]
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
#[command(after_long_help = MAC_LIB_HELP, after_help = MAC_LIB_HELP)]
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
