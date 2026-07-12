use proto_pdk::*;
use semver::Version;

use crate::release_parser;

const GITHUB_RELEASES_URL: &str =
    "https://github.com/graalvm/graalvm-ce-builds/releases/download";

// ============================================================================
// Main Entry Point
// ============================================================================

pub fn build_download_info(
    version_str: &str,
    env: &HostEnvironment,
) -> FnResult<DownloadPrebuiltOutput> {
    // バージョンをパース
    let version = Version::parse(version_str)
        .map_err(|_| PluginError::Custom(format!("Invalid version: {}", version_str)))?;

    // 該当バージョンのアセットを探す
    let asset = release_parser::find_asset_for_version(version_str, env.os, env.arch)?
        .ok_or_else(|| {
            PluginError::Custom(format!(
                "No prebuilt found for GraalVM CE {} on {} {}",
                version_str, env.os, env.arch
            ))
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

/// GitHub tag 名を決定
fn determine_tag_name(version: &Version) -> FnResult<String> {
    // GitHub Releases から全タグを取得して、このバージョンに対応するタグを探す
    let releases: Vec<crate::release_parser::GitHubRelease> =
        fetch_json("https://api.github.com/repos/graalvm/graalvm-ce-builds/releases?per_page=100")?;

    for release in releases {
        if let Some(v) = crate::release_parser::parse_version_from_tag(&release.tag_name) {
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
}
