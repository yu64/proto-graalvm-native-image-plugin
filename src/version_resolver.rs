use semver::Version;

use crate::release_parser;

// ============================================================================
// Version Resolution
// ============================================================================

/// バージョンエイリアスをリアルバージョンに解決
pub fn resolve_alias(alias: &str, available_versions: &[Version]) -> Option<Version> {
    match alias {
        "latest" => available_versions.first().cloned(),

        "lts" => resolve_lts_version(available_versions),

        // "25.0" のような部分バージョンマッチ
        partial => resolve_partial_version(partial, available_versions),
    }
}

/// LTS（Long Term Support）バージョンを解決
fn resolve_lts_version(available_versions: &[Version]) -> Option<Version> {
    // GraalVM CE のリリースサイクルから見て LTS は通常 3 年ごと
    // 25.0.0 の次の LTS は 28.0.0 などになる
    // ここでは、最新の XX.0.0 メジャーバージョンをハードコード

    // 既知の LTS バージョン（あくまで推定）
    let known_lts = [
        Version::new(25, 0, 0),
        Version::new(23, 0, 0),
        Version::new(21, 0, 0),
        Version::new(19, 0, 0),
        Version::new(17, 0, 0),
        Version::new(16, 0, 0),
        Version::new(11, 0, 0),
    ];

    for lts in &known_lts {
        if available_versions.contains(lts) {
            return Some(lts.clone());
        }
    }

    // 既知の LTS がなければ、最新メジャーバージョンを返す
    available_versions.iter().find(|v| v.patch == 0).cloned()
}

/// 部分バージョン（"25.0", "25" など）をマッチ
fn resolve_partial_version(partial: &str, available_versions: &[Version]) -> Option<Version> {
    let parts: Vec<&str> = partial.split('.').collect();

    if parts.is_empty() {
        return None;
    }

    match parts.len() {
        1 => {
            // "25" → 25.x.x の最新
            let major = parts[0].parse::<u64>().ok()?;
            available_versions
                .iter()
                .find(|v| v.major == major)
                .cloned()
        }

        2 => {
            // "25.0" → 25.0.x の最新
            let major = parts[0].parse::<u64>().ok()?;
            let minor = parts[1].parse::<u64>().ok()?;
            available_versions
                .iter()
                .find(|v| v.major == major && v.minor == minor)
                .cloned()
        }

        _ => {
            // "25.0.3" → 完全マッチ
            Version::parse(partial).ok()
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_versions() -> Vec<Version> {
        vec![
            Version::new(25, 1, 3),
            Version::new(25, 1, 2),
            Version::new(25, 1, 1),
            Version::new(25, 1, 0),
            Version::new(25, 0, 3),
            Version::new(25, 0, 2),
            Version::new(25, 0, 1),
            Version::new(25, 0, 0),
            Version::new(24, 1, 0),
            Version::new(24, 0, 0),
            Version::new(23, 1, 0),
            Version::new(23, 0, 0),
        ]
    }

    #[test]
    fn test_resolve_latest() {
        let versions = make_versions();
        assert_eq!(resolve_alias("latest", &versions), Some(Version::new(25, 1, 3)));
    }

    #[test]
    fn test_resolve_lts() {
        let versions = make_versions();
        let lts = resolve_alias("lts", &versions);
        assert!(lts.is_some());
        // LTS は 25.0.0 が候補
        assert_eq!(lts, Some(Version::new(25, 0, 0)));
    }

    #[test]
    fn test_resolve_partial_major() {
        let versions = make_versions();
        assert_eq!(resolve_alias("25", &versions), Some(Version::new(25, 1, 3)));
    }

    #[test]
    fn test_resolve_partial_major_minor() {
        let versions = make_versions();
        assert_eq!(resolve_alias("25.0", &versions), Some(Version::new(25, 0, 3)));
    }

    #[test]
    fn test_resolve_partial_full_version() {
        let versions = make_versions();
        assert_eq!(
            resolve_alias("25.0.2", &versions),
            Some(Version::new(25, 0, 2))
        );
    }

    #[test]
    fn test_resolve_partial_non_existent() {
        let versions = make_versions();
        assert_eq!(resolve_alias("99.0.0", &versions), None);
    }
}
