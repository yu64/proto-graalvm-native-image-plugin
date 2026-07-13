use proto_pdk::*;
use semver::Version;

use crate::release_parser::{self, ParsedAsset};

const GITHUB_RELEASES_URL: &str =
    "https://github.com/graalvm/graalvm-ce-builds/releases/download";

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/graalvm/graalvm-ce-builds/releases?per_page=100";

// ============================================================================
// Main Entry Point
// ============================================================================

/// GraalVM CE のダウンロード情報を構築
///
/// # Arguments
/// * `version_str` - バージョン文字列（例："25.0.3"）
/// * `env` - ホスト環境情報（OS/Architecture）
///
/// # Returns
/// DownloadPrebuiltOutput - ダウンロード URL とメタデータ
///
/// # Errors
/// - 無効なバージョン形式
/// - 該当バージョン/OS/Arch の組み合わせが見つからない
pub fn build_download_info(
    version_str: &str,
    env: &HostEnvironment,
) -> AnyResult<DownloadPrebuiltOutput> {
    // バージョンをパース
    let version = Version::parse(version_str)
        .map_err(|_| anyhow::anyhow!("Invalid version: {}", version_str))?;

    // 該当バージョンのアセットを探す
    let asset = find_asset_for_platform(version_str, env.os, env.arch)?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No prebuilt found for GraalVM CE {} on {} {}",
                version_str,
                env.os,
                env.arch
            )
        })?;

    // GitHub Releases URL を構築
    let tag = determine_tag_name(&version)?;
    let download_url = format!(
        "{}/{}/{}",
        GITHUB_RELEASES_URL, tag, asset.filename
    );

    // Archive prefix を決定（アーカイブ内のトップレベルディレクトリ名）
    let archive_prefix = determine_archive_prefix(&version);

    Ok(DownloadPrebuiltOutput {
        download_url,
        download_name: Some(asset.filename),
        archive_prefix: Some(archive_prefix),
        checksum_url: None, // GraalVM CE は公式チェックサムを提供していない
        ..DownloadPrebuiltOutput::default()
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// プラットフォーム用のアセットを探す
fn find_asset_for_platform(
    version_str: &str,
    os: HostOS,
    arch: HostArch,
) -> AnyResult<Option<ParsedAsset>> {
    release_parser::find_asset_for_version(version_str, os, arch)
}

/// GitHub tag 名を決定
fn determine_tag_name(version: &Version) -> AnyResult<String> {
    let releases: Vec<release_parser::GitHubRelease> = release_parser::fetch_releases(GITHUB_API_URL)?;

    for release in releases {
        if let Some(v) = release_parser::parse_version_from_tag(&release.tag_name) {
            if v == *version {
                return Ok(release.tag_name);
            }
        }
    }

    // フォールバック: パターンから推測
    Ok(guess_tag_name(version))
}

/// Tag 名を推測（API から取得できない場合のフォールバック）
fn guess_tag_name(version: &Version) -> String {
    // 新しいバージョン（25.1.0 以上）は "graal-" プレフィックス
    if version.major > 25 || (version.major == 25 && version.minor >= 1) {
        format!("graal-{}", version)
    } else {
        // 古いバージョンは "jdk-" プレフィックス
        format!("jdk-{}", version)
    }
}

/// Archive 内のトップレベルディレクトリ名を決定
fn determine_archive_prefix(version: &Version) -> String {
    // GraalVM CE のアーカイブ内ディレクトリ名パターン:
    // - graalvm-ce-java17-23.0.0
    // - graalvm-ce-java17-25.0.0
    // - graalvm-jdk-25.0.3

    if version.major >= 25 {
        format!("graalvm-jdk-{}", version)
    } else {
        format!("graalvm-ce-java17-{}", version)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_tag_name_old_version() {
        let version = Version::new(23, 0, 0);
        assert_eq!(guess_tag_name(&version), "jdk-23.0.0");
    }

    #[test]
    fn test_guess_tag_name_new_version() {
        let version = Version::new(25, 1, 3);
        assert_eq!(guess_tag_name(&version), "graal-25.1.3");
    }

    #[test]
    fn test_determine_archive_prefix_old() {
        let version = Version::new(23, 0, 0);
        assert_eq!(
            determine_archive_prefix(&version),
            "graalvm-ce-java17-23.0.0"
        );
    }

    #[test]
    fn test_determine_archive_prefix_new() {
        let version = Version::new(25, 0, 3);
        assert_eq!(determine_archive_prefix(&version), "graalvm-jdk-25.0.3");
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_guess_tag_name_major_24() {
        // バージョン 24.x
        let version = Version::new(24, 1, 0);
        assert_eq!(guess_tag_name(&version), "jdk-24.1.0");
    }

    #[test]
    fn test_guess_tag_name_major_26() {
        // バージョン 26.0.0 以上（仮定）
        let version = Version::new(26, 0, 0);
        assert_eq!(guess_tag_name(&version), "graal-26.0.0");
    }

    #[test]
    fn test_determine_archive_prefix_boundary() {
        // 境界值: 25.0.0
        let v1 = Version::new(24, 9, 9);
        let v2 = Version::new(25, 0, 0);
        let v3 = Version::new(25, 0, 1);

        assert!(determine_archive_prefix(&v1).contains("ce-java17"));
        assert!(determine_archive_prefix(&v2).contains("graalvm-jdk"));
        assert!(determine_archive_prefix(&v3).contains("graalvm-jdk"));
    }

    #[test]
    fn test_determine_archive_prefix_format() {
        // フォーマットチェック
        let version = Version::new(25, 1, 3);
        let prefix = determine_archive_prefix(&version);

        assert!(prefix.starts_with("graalvm-"));
        assert!(prefix.contains(&version.to_string()));
    }

    #[test]
    fn test_tag_name_consistency() {
        // タグ名の一貫性チェック
        let versions = vec![
            (23, 0, 0, "jdk"),
            (24, 0, 0, "jdk"),
            (25, 0, 0, "jdk"),
            (25, 1, 0, "graal"),
            (26, 0, 0, "graal"),
        ];

        for (major, minor, patch, expected_prefix) in versions {
            let version = Version::new(major, minor, patch);
            let tag = guess_tag_name(&version);
            assert!(
                tag.starts_with(expected_prefix),
                "Expected {} prefix for {}.{}.{}, got {}",
                expected_prefix,
                major,
                minor,
                patch,
                tag
            );
        }
    }

    #[test]
    fn test_download_url_structure() {
        // ダウンロード URL の構造チェック
        let tag = "graal-25.0.3";
        let filename = "graalvm-community-jdk-25.0.3_windows-x64_bin.zip";
        let url = format!("{}/{}/{}", GITHUB_RELEASES_URL, tag, filename);

        assert!(url.contains("github.com"));
        assert!(url.contains("graalvm-ce-builds"));
        assert!(url.contains("releases"));
        assert!(url.contains("download"));
    }
}
