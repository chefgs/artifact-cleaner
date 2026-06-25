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
    HelperBundle(Vec<String>),
    SharedContainer(Vec<String>),
    NamedCache(String),
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

    if let Some(candidates) = helper_bundle_candidates(name) {
        return EntryKind::HelperBundle(candidates);
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
        return EntryKind::SharedContainer(shared_container_candidates(&bundle_id));
    }

    // Pattern: group.com.company.app
    if let Some(bundle_id) = strip_group_prefix(name) {
        return EntryKind::SharedContainer(shared_container_candidates(&bundle_id));
    }

    // Plain bundle ID with no prefix (e.g. com.company.app)
    if looks_like_bundle_id(name) {
        return EntryKind::SharedContainer(shared_container_candidates(name));
    }

    EntryKind::Excluded
}

fn classify_cache(name: &str) -> EntryKind {
    if name.starts_with("com.apple.") {
        return EntryKind::Excluded;
    }

    if let Some(candidates) = helper_bundle_candidates(name) {
        return EntryKind::HelperBundle(candidates);
    }

    if looks_like_bundle_id(name) {
        return EntryKind::BundleId(name.to_string());
    }
    // Generic cache folders are ambiguous; resolve conservatively later.
    EntryKind::NamedCache(name.to_string())
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
        && parts
            .iter()
            .all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
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

fn strip_shared_suffix(name: &str) -> Option<String> {
    name.strip_suffix(".shared").map(|s| s.to_string())
}

fn strip_private_suffix(name: &str) -> Option<String> {
    name.strip_suffix(".private").map(|s| s.to_string())
}

fn strip_family_suffix(name: &str) -> Option<String> {
    name.strip_suffix(".family").map(|s| s.to_string())
}

fn shared_container_candidates(name: &str) -> Vec<String> {
    let mut candidates = vec![name.to_string()];

    if let Some(base) = strip_shared_suffix(name) {
        candidates.push(base);
    }

    if let Some(base) = strip_private_suffix(name) {
        candidates.push(base);
    }

    if let Some(base) = strip_family_suffix(name) {
        candidates.push(base);
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

fn helper_bundle_candidates(name: &str) -> Option<Vec<String>> {
    let mut candidates = vec![name.to_string()];
    let mut normalized = false;

    if let Some(base) = strip_shipit_suffix(name) {
        candidates.push(base);
        normalized = true;
    }

    if let Some(base) = strip_dot_updater_suffix(name) {
        candidates.push(base);
        normalized = true;
    }

    if let Some(base) = strip_trailing_updater_token(name) {
        candidates.push(base);
        normalized = true;
    }

    if !normalized {
        return None;
    }

    candidates.sort();
    candidates.dedup();
    Some(candidates)
}

fn strip_shipit_suffix(name: &str) -> Option<String> {
    name.strip_suffix(".ShipIt").map(|s| s.to_string())
}

fn strip_dot_updater_suffix(name: &str) -> Option<String> {
    name.strip_suffix(".Updater").map(|s| s.to_string())
}

fn strip_trailing_updater_token(name: &str) -> Option<String> {
    if !name.ends_with("Updater") {
        return None;
    }

    let parts: Vec<&str> = name.split('.').collect();
    let last = parts.last()?;
    if *last == "Updater" || !last.ends_with("Updater") {
        return None;
    }

    let trimmed = last.strip_suffix("Updater")?;
    if trimmed.is_empty() || parts.len() < 2 {
        return None;
    }

    let mut rebuilt = parts[..parts.len() - 1].join(".");
    rebuilt.push('.');
    rebuilt.push_str(trimmed);
    Some(rebuilt)
}

fn looks_like_bundle_id(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2
        && matches!(
            parts[0],
            "com" | "net" | "io" | "org" | "app" | "dev" | "ai" | "co" | "uk" | "ru" | "us"
        )
}

#[cfg(test)]
mod tests {
    use super::{EntryKind, FolderSource, classify};

    #[test]
    fn group_container_with_shared_suffix_is_treated_as_shared() {
        let kind = classify(
            "group.net.whatsapp.WhatsApp.shared",
            &FolderSource::GroupContainers,
        );

        match kind {
            EntryKind::SharedContainer(candidates) => {
                assert!(candidates.contains(&"net.whatsapp.WhatsApp.shared".to_string()));
                assert!(candidates.contains(&"net.whatsapp.WhatsApp".to_string()));
            }
            _ => panic!("expected shared container"),
        }
    }

    #[test]
    fn group_container_with_private_suffix_is_treated_as_shared() {
        let kind = classify(
            "group.net.whatsapp.WhatsApp.private",
            &FolderSource::GroupContainers,
        );

        match kind {
            EntryKind::SharedContainer(candidates) => {
                assert!(candidates.contains(&"net.whatsapp.WhatsApp.private".to_string()));
                assert!(candidates.contains(&"net.whatsapp.WhatsApp".to_string()));
            }
            _ => panic!("expected shared container"),
        }
    }

    #[test]
    fn team_id_prefixed_group_container_stays_conservative() {
        let kind = classify(
            "UBF8T346G9.OneDriveSyncClientSuite",
            &FolderSource::GroupContainers,
        );

        match kind {
            EntryKind::SharedContainer(candidates) => {
                assert_eq!(candidates, vec!["OneDriveSyncClientSuite".to_string()]);
            }
            _ => panic!("expected shared container"),
        }
    }

    #[test]
    fn named_cache_is_not_treated_as_cli_ownership() {
        let kind = classify("Homebrew", &FolderSource::Caches);

        match kind {
            EntryKind::NamedCache(name) => assert_eq!(name, "Homebrew"),
            _ => panic!("expected named cache"),
        }
    }

    #[test]
    fn helper_bundle_shipit_produces_parent_candidate() {
        let kind = classify("com.github.GitHubClient.ShipIt", &FolderSource::Caches);

        match kind {
            EntryKind::HelperBundle(candidates) => {
                assert!(candidates.contains(&"com.github.GitHubClient.ShipIt".to_string()));
                assert!(candidates.contains(&"com.github.GitHubClient".to_string()));
            }
            _ => panic!("expected helper bundle"),
        }
    }

    #[test]
    fn helper_bundle_updater_produces_parent_candidate() {
        let kind = classify("com.microsoft.OneDriveUpdater", &FolderSource::Caches);

        match kind {
            EntryKind::HelperBundle(candidates) => {
                assert!(candidates.contains(&"com.microsoft.OneDriveUpdater".to_string()));
                assert!(candidates.contains(&"com.microsoft.OneDrive".to_string()));
            }
            _ => panic!("expected helper bundle"),
        }
    }
}
