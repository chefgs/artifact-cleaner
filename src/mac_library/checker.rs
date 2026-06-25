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

pub fn is_any_bundle_id_installed(bundle_ids: &[String]) -> bool {
    bundle_ids.iter().any(|id| is_bundle_id_installed(id))
}

pub fn is_helper_bundle_active(bundle_ids: &[String]) -> bool {
    bundle_ids.iter().any(|id| is_bundle_id_installed(id))
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

pub fn is_named_cache_active(cache_name: &str) -> bool {
    cli_candidates_for_cache(cache_name)
        .iter()
        .any(|candidate| is_cli_installed(candidate))
}

fn cli_candidates_for_cache(cache_name: &str) -> Vec<&str> {
    match cache_name {
        "Homebrew" => vec!["brew"],
        "Yarn" => vec!["yarn"],
        "go-build" => vec!["go"],
        "ms-playwright" | "ms-playwright-go" => vec!["playwright"],
        "node-gyp" => vec!["node-gyp", "npm", "node"],
        "podman-desktop-updater" => vec!["podman"],
        "pnpm" => vec!["pnpm"],
        "typescript" => vec!["tsc", "typescript"],
        "pip" => vec!["pip", "pip3"],
        _ => vec![cache_name],
    }
}
