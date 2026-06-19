// resolver.rs — classifies a Library subfolder name into something we can act on

// ─── RUST LESSON — Enums as rich types ───────────────────────────────────────
// Rust enums can carry data in their variants (algebraic data types).
// EntryKind::BundleId(String) means "this is a bundle ID, here is its value".
// This is far safer than using a stringly-typed status code.
// The compiler forces every match arm to handle all variants.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FolderSource {
    Caches,
    Containers,
    GroupContainers,
}

#[derive(Debug, Clone)]
pub enum EntryKind {
    BundleId(String),
    CliTool(String),
    Excluded,
}

pub fn classify(name: &str, source: &FolderSource) -> EntryKind {
    match source {
        FolderSource::Containers => classify_container(name),
        FolderSource::GroupContainers => classify_group_container(name),
        FolderSource::Caches => classify_cache(name),
    }
}

fn classify_container(name: &str) -> EntryKind {
    if is_uuid(name) {
        return EntryKind::Excluded;
    }
    // System containers — deleting these can break macOS
    if name.starts_with("com.apple.") {
        return EntryKind::Excluded;
    }
    EntryKind::BundleId(name.to_string())
}

fn classify_group_container(name: &str) -> EntryKind {
    // System group containers
    if name.starts_with("com.apple.") || name.starts_with("group.com.apple.") {
        return EntryKind::Excluded;
    }

    // Pattern: XXXXXXXXXX.bundle.id  (10-char alphanumeric Team ID prefix)
    if let Some(bundle_id) = strip_team_id(name) {
        if bundle_id.starts_with("com.apple.") {
            return EntryKind::Excluded;
        }
        return EntryKind::BundleId(bundle_id);
    }

    // Pattern: group.com.company.app
    if let Some(bundle_id) = strip_group_prefix(name) {
        return EntryKind::BundleId(bundle_id);
    }

    // Plain bundle ID with no prefix (e.g. com.company.app)
    if looks_like_bundle_id(name) {
        return EntryKind::BundleId(name.to_string());
    }

    EntryKind::Excluded
}

fn classify_cache(name: &str) -> EntryKind {
    if name.starts_with("com.apple.") {
        return EntryKind::Excluded;
    }
    if looks_like_bundle_id(name) {
        return EntryKind::BundleId(name.to_string());
    }
    // Everything else is treated as a CLI tool / named cache
    EntryKind::CliTool(name.to_string())
}

// ─── RUST LESSON — Pure helper functions ─────────────────────────────────────
// No `pub` = private to this module. Callers only see `classify`.
// Each helper has a single, testable responsibility.
// ─────────────────────────────────────────────────────────────────────────────

fn is_uuid(name: &str) -> bool {
    let parts: Vec<&str> = name.split('-').collect();
    parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[3].len() == 4
        && parts[4].len() == 12
        && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
}

// Strip a 10-char Team ID prefix: "6N38VWS5BX.ru.keepcoder.Telegram" → "ru.keepcoder.Telegram"
fn strip_team_id(name: &str) -> Option<String> {
    if name.len() <= 11 {
        return None;
    }
    let (prefix, rest) = name.split_at(10);
    if prefix.chars().all(|c| c.is_ascii_alphanumeric()) && rest.starts_with('.') {
        let bundle_id = &rest[1..];
        if !bundle_id.is_empty() {
            return Some(bundle_id.to_string());
        }
    }
    None
}

// Strip "group." prefix: "group.net.whatsapp.WhatsApp.shared" → "net.whatsapp.WhatsApp.shared"
fn strip_group_prefix(name: &str) -> Option<String> {
    name.strip_prefix("group.").map(|s| s.to_string())
}

fn looks_like_bundle_id(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2
        && matches!(
            parts[0],
            "com" | "net" | "io" | "org" | "app" | "dev" | "ai" | "co" | "uk" | "ru" | "us"
        )
}
