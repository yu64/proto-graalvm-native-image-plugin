use proto_pdk::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const GITHUB_RELEASES_API: &str =
    "https://api.github.com/repos/graalvm/graalvm-ce-builds/releases?per_page=100";
const GITHUB_RELEASES_API_ALL: &str =
    "https://api.github.com/repos/graalvm/graalvm-ce-builds/releases?per_page=100&page=";

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub prerelease: bool,
    pub draft: bool,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParsedAsset {
    pub version: Version,
    pub os: HostOS,
    pub arch: HostArch,
    pub file_type: FileType,
    pub filename: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
    Zip,
    TarGz,
}

// ============================================================================
// Main API Function
// ============================================================================

pub fn fetch_and_parse_releases() -> FnResult<Vec<Version>> {
    // GitHub Releases API から取得（キャッシングは proto が自動で行う）
    let releases: Vec<GitHubRelease> = fetch_json(GITHUB_RELEASES_API)?;

    let mut versions = BTreeMap::new();

    for release in releases {
        // Pre-release と draft は除外
        if release.prerelease || release.draft {
            continue;
        }

        // タグからバージョンを抽出
        if let Some(version) = parse_version_from_tag(&release.tag_name) {
            versions.insert(version.clone(), version);
        }
    }

    // バージョンを新しい順にソート
    let mut result: Vec<Version> = versions.keys().cloned().collect();
    result.sort();
    result.reverse();

    Ok(result)
}

/// 特定バージョンのアセット情報を取得
pub fn find_asset_for_version(
    version: &str,
    os: HostOS,
    arch: HostArch,
) -> FnResult<Option<ParsedAsset>> {
    let releases: Vec<GitHubRelease> = fetch_json(GITHUB_RELEASES_API)?;

    for release in releases {
        if release.prerelease || release.draft {
            continue;
        }

        if let Some(parsed_version) = parse_version_from_tag(&release.tag_name) {
            if parsed_version.to_string() == version {
                // このリリースから OS/Arch に合致するアセットを探す
                for asset in release.assets {
                    if let Some(parsed) = parse_asset(&asset.name, &asset.browser_download_url) {
                        if parsed.os == os && parsed.arch == arch {
                            return Ok(Some(parsed));
                        }
                    }
                }
                return Ok(None);
            }
        }
    }

    Ok(None)
}

// ============================================================================
// Parsing Functions
// ============================================================================

/// タグ名からバージョンを抽出
fn parse_version_from_tag(tag: &str) -> Option<Version> {
    // パターン 1: "jdk-25.0.0" → 25.0.0
    if let Some(version_str) = tag.strip_prefix("jdk-") {
        if let Ok(v) = Version::parse(version_str) {
            return Some(v);
        }
    }

    // パターン 2: "graal-25.1.3" → 25.1.3
    if let Some(version_str) = tag.strip_prefix("graal-") {
        if let Ok(v) = Version::parse(version_str) {
            return Some(v);
        }
    }

    // パターン 3: "vm-25.0.0" → 25.0.0
    if let Some(version_str) = tag.strip_prefix("vm-") {
        if let Ok(v) = Version::parse(version_str) {
            return Some(v);
        }
    }

    None
}

/// ファイル名をパース
///
/// GraalVM CE のアーカイブファイル名をパースしてメタデータを抽出します。
///
/// # パターン例
///
/// ```text
/// graalvm-community-jdk-25i1-25.0.3_windows-x64_bin.zip
///  └─ prefix ──┬───┘  └─ internal ┘ └─ os-arch ──┘ └ ext
///              └── ignored (build number)
///
/// graalvm-community-jdk-25.0.2_macos-aarch64_bin.tar.gz
/// graalvm-community-jdk-23.0.0_linux-x64_bin.tar.gz
/// ```
///
/// # Returns
///
/// Some(ParsedAsset) - パース成功
/// None - ファイル名が予期された形式でない
pub fn parse_asset(filename: &str, url: &str) -> Option<ParsedAsset> {
    // ファイルタイプを判定
    let file_type = if filename.ends_with(".zip") {
        FileType::Zip
    } else if filename.ends_with(".tar.gz") {
        FileType::TarGz
    } else {
        return None;
    };

    // "bin" の前のパートを抽出
    let before_bin = if let Some(idx) = filename.rfind("_bin.") {
        &filename[..idx]
    } else {
        return None;
    };

    // 最後の "_" で OS-Arch を分割
    let parts: Vec<&str> = before_bin.split('_').collect();
    if parts.len() < 2 {
        return None;
    }

    let os_arch = parts[parts.len() - 1]; // "windows-x64", "macos-aarch64"
    let (os, arch) = parse_os_arch(os_arch)?;

    // バージョン部分を抽出（最後の "_" の前）
    let version_part = parts[..parts.len() - 1].join("_"); // "graalvm-community-jdk-25i1-25.0.3"

    // バージョン番号を抽出（最後の "-" 以降）
    let version_str = if let Some(idx) = version_part.rfind('-') {
        let raw_version = &version_part[idx + 1..];
        // "25i1-25.0.3" → "25.0.3" の場合
        if raw_version.contains('-') {
            raw_version.split('-').last()?
        } else {
            raw_version
        }
    } else {
        return None;
    };

    // Version をパース
    let version = Version::parse(version_str).ok()?;

    Some(ParsedAsset {
        version,
        os,
        arch,
        file_type,
        filename: filename.to_string(),
        url: url.to_string(),
    })
}

/// OS と Architecture をパース
fn parse_os_arch(os_arch: &str) -> Option<(HostOS, HostArch)> {
    match os_arch {
        "windows-x64" => Some((HostOS::Windows, HostArch::X64)),
        "windows-x86" => Some((HostOS::Windows, HostArch::X86)),
        "windows-aarch64" | "windows-arm64" => Some((HostOS::Windows, HostArch::Arm64)),

        "macos-x64" | "darwin-x64" => Some((HostOS::MacOS, HostArch::X64)),
        "macos-aarch64" | "macos-arm64" | "darwin-aarch64" => Some((HostOS::MacOS, HostArch::Arm64)),

        "linux-x64" | "linux-x86_64" => Some((HostOS::Linux, HostArch::X64)),
        "linux-aarch64" | "linux-arm64" => Some((HostOS::Linux, HostArch::Arm64)),
        "linux-ppc64le" => Some((HostOS::Linux, HostArch::Powerpc64)),
        "linux-s390x" => Some((HostOS::Linux, HostArch::S390x)),

        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_from_tag_jdk_pattern() {
        assert_eq!(
            parse_version_from_tag("jdk-25.0.0"),
            Some(Version::new(25, 0, 0))
        );
        assert_eq!(
            parse_version_from_tag("jdk-25.0.2"),
            Some(Version::new(25, 0, 2))
        );
    }

    #[test]
    fn test_parse_version_from_tag_graal_pattern() {
        assert_eq!(
            parse_version_from_tag("graal-25.1.3"),
            Some(Version::new(25, 1, 3))
        );
    }

    #[test]
    fn test_parse_asset_windows_x64() {
        let filename = "graalvm-community-jdk-25i1-25.0.3_windows-x64_bin.zip";
        let url = "https://example.com/file.zip";

        let asset = parse_asset(filename, url).unwrap();
        assert_eq!(asset.version, Version::new(25, 0, 3));
        assert_eq!(asset.os, HostOS::Windows);
        assert_eq!(asset.arch, HostArch::X64);
        assert_eq!(asset.file_type, FileType::Zip);
    }

    #[test]
    fn test_parse_asset_macos_aarch64() {
        let filename = "graalvm-community-jdk-25i1-25.0.3_macos-aarch64_bin.tar.gz";
        let url = "https://example.com/file.tar.gz";

        let asset = parse_asset(filename, url).unwrap();
        assert_eq!(asset.version, Version::new(25, 0, 3));
        assert_eq!(asset.os, HostOS::MacOS);
        assert_eq!(asset.arch, HostArch::Arm64);
        assert_eq!(asset.file_type, FileType::TarGz);
    }

    #[test]
    fn test_parse_asset_linux_x64() {
        let filename = "graalvm-community-jdk-25.0.2_linux-x64_bin.tar.gz";
        let url = "https://example.com/file.tar.gz";

        let asset = parse_asset(filename, url).unwrap();
        assert_eq!(asset.version, Version::new(25, 0, 2));
        assert_eq!(asset.os, HostOS::Linux);
        assert_eq!(asset.arch, HostArch::X64);
    }

    #[test]
    fn test_parse_os_arch_windows() {
        assert_eq!(
            parse_os_arch("windows-x64"),
            Some((HostOS::Windows, HostArch::X64))
        );
    }

    #[test]
    fn test_parse_os_arch_macos() {
        assert_eq!(
            parse_os_arch("macos-aarch64"),
            Some((HostOS::MacOS, HostArch::Arm64))
        );
    }

    #[test]
    fn test_parse_os_arch_linux() {
        assert_eq!(
            parse_os_arch("linux-x64"),
            Some((HostOS::Linux, HostArch::X64))
        );
    }

    #[test]
    fn test_parse_os_arch_linux_aarch64() {
        assert_eq!(
            parse_os_arch("linux-aarch64"),
            Some((HostOS::Linux, HostArch::Arm64))
        );
    }

    #[test]
    fn test_parse_asset_invalid_extension() {
        assert_eq!(parse_asset("graalvm.jar", "https://example.com"), None);
    }

    #[test]
    fn test_parse_asset_missing_bin_suffix() {
        assert_eq!(
            parse_asset("graalvm-community-jdk-25.0.3_windows-x64.zip", "https://example.com"),
            None
        );
    }

    #[test]
    fn test_parse_version_from_tag_fallback() {
        // 実際には存在しないタグ
        assert_eq!(parse_version_from_tag("invalid-tag"), None);
    }
}
