// checker.rs — determines whether an app or CLI tool is currently installed

use std::process::Command;

// ─── RUST LESSON — std::process::Command ─────────────────────────────────────
// Command::new("mdfind") spawns a child process synchronously.
// .arg() appends one argument (safer than shell strings — no injection risk).
// .output() blocks until the process exits, returning stdout/stderr/status.
// We use mdfind (macOS Spotlight) to query installed apps by bundle ID.
// This is more reliable than scanning /Applications manually because it
// catches apps installed in ~/Applications, /Applications, and via other paths.
// ─────────────────────────────────────────────────────────────────────────────

/// Returns true if an app with the given bundle ID is installed.
/// Uses Spotlight (mdfind) — only available on macOS.
pub fn is_bundle_id_installed(bundle_id: &str) -> bool {
    Command::new("mdfind")
        .arg(format!("kMDItemCFBundleIdentifier == '{bundle_id}'"))
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

/// Returns true if a CLI tool with the given name is found in PATH.
pub fn is_cli_installed(tool_name: &str) -> bool {
    // `which` exits 0 if found, non-zero if not
    Command::new("which")
        .arg(tool_name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
