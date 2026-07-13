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
    if partial.is_empty() {
        return None;
    }

    let parts: Vec<&str> = partial.split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }

    if parts.iter().any(|part| part.is_empty() || part.chars().any(|ch| !ch.is_ascii_digit())) {
        return None;
    }

    if parts.iter().any(|part| part.starts_with('0') && part.len() > 1) {
        return None;
    }

    match parts.len() {
        1 => {
            let major = parts[0].parse::<u64>().ok()?;
            available_versions
                .iter()
                .filter(|v| v.major == major)
                .max_by(|a, b| a.cmp(b))
                .cloned()
        }

        2 => {
            let major = parts[0].parse::<u64>().ok()?;
            let minor = parts[1].parse::<u64>().ok()?;
            available_versions
                .iter()
                .filter(|v| v.major == major && v.minor == minor)
                .max_by(|a, b| a.cmp(b))
                .cloned()
        }

        _ => {
            Version::parse(partial).ok().filter(|version| {
                available_versions.iter().any(|candidate| candidate == version)
            })
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

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_resolve_empty_version_list() {
        // 空のバージョンリスト
        let empty: Vec<Version> = vec![];
        assert_eq!(resolve_alias("latest", &empty), None);
        assert_eq!(resolve_alias("lts", &empty), None);
    }

    #[test]
    fn test_resolve_single_version() {
        // 1 つのバージョンのみ
        let versions = vec![Version::new(25, 0, 0)];
        assert_eq!(resolve_alias("latest", &versions), Some(Version::new(25, 0, 0)));
        assert_eq!(resolve_alias("lts", &versions), Some(Version::new(25, 0, 0)));
    }

    #[test]
    fn test_resolve_partial_major_no_match() {
        let versions = make_versions();
        assert_eq!(resolve_alias("99", &versions), None);
    }

    #[test]
    fn test_resolve_partial_major_minor_no_match() {
        let versions = make_versions();
        assert_eq!(resolve_alias("25.9", &versions), None);
    }

    #[test]
    fn test_resolve_multiple_minor_versions() {
        // 同じ major.minor で複数の patch がある場合
        let versions = vec![
            Version::new(25, 0, 5),
            Version::new(25, 0, 4),
            Version::new(25, 0, 3),
        ];
        assert_eq!(resolve_alias("25.0", &versions), Some(Version::new(25, 0, 5)));
    }

    #[test]
    fn test_resolve_version_comparison_order() {
        // バージョン比較順序の確認
        let v1 = Version::new(25, 0, 3);
        let v2 = Version::new(25, 1, 0);
        
        assert!(v2 > v1);
        assert!(v1 < v2);
    }

    #[test]
    fn test_resolve_lts_with_non_lts_versions() {
        // LTS とそれ以外が混在
        let versions = vec![
            Version::new(25, 1, 0), // not LTS
            Version::new(25, 0, 0), // LTS (x.0.0)
            Version::new(24, 1, 0), // not LTS
            Version::new(24, 0, 0), // LTS
        ];
        
        let lts = resolve_alias("lts", &versions);
        assert!(lts.is_some());
        let resolved = lts.unwrap();
        assert_eq!(resolved.patch, 0); // LTS は xx.0.0 の形式
    }

    #[test]
    fn test_resolve_partial_with_leading_zeros() {
        // Leading zero は semver では許可されない（エラーになる）
        let versions = make_versions();
        assert_eq!(resolve_alias("025", &versions), None);
    }

    #[test]
    fn test_resolve_partial_with_special_chars() {
        // 特殊文字は許可されない
        let versions = make_versions();
        assert_eq!(resolve_alias("25.0-beta", &versions), None);
        assert_eq!(resolve_alias("25.0+build", &versions), None);
    }

    #[test]
    fn test_resolve_version_sorted_correctly() {
        // バージョンソート順の確認
        let mut versions = vec![
            Version::new(23, 0, 0),
            Version::new(25, 1, 0),
            Version::new(25, 0, 3),
            Version::new(24, 0, 0),
            Version::new(25, 0, 2),
        ];
        versions.sort();
        versions.reverse(); // 新しい順

        assert_eq!(versions[0], Version::new(25, 1, 0));
        assert_eq!(versions[1], Version::new(25, 0, 3));
    }

    #[test]
    fn test_lts_priority_order() {
        // LTS 候補の優先順位確認
        let versions = vec![
            Version::new(25, 1, 0),
            Version::new(25, 0, 0), // ← これが LTS
            Version::new(24, 0, 0),
            Version::new(23, 0, 0),
        ];

        let lts = resolve_alias("lts", &versions).unwrap();
        // 最初に見つかった LTS を返す
        assert_eq!(lts, Version::new(25, 0, 0));
    }
}
