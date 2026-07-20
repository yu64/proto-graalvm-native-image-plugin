use extism_pdk::FnResult;
use proto_pdk::{fetch_json, HostArch, HostEnvironment, HostOS, PluginError};
use serde::Deserialize;

const GITHUB_API_URL: &str = "https://api.github.com/repos/oracle/graalvm-ce-builds/releases";

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// Parse version from GitHub release tag (e.g., "jdk-25.0.0", "graal-25.1.3", "vm-24.0.0")
pub fn parse_version_from_tag(tag: &str) -> Option<(u64, u64, u64)> {
    let version_str = tag
        .strip_prefix("jdk-")
        .or_else(|| tag.strip_prefix("graal-"))
        .or_else(|| tag.strip_prefix("vm-"))?;

    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse::<u64>().ok()?;
        let minor = parts[1].parse::<u64>().ok()?;
        let patch = parts[2].parse::<u64>().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}

/// Determine OS string for GraalVM download URLs
pub fn get_os_string(os: HostOS) -> &'static str {
    match os {
        HostOS::Linux => "linux",
        HostOS::MacOS => "macos",
        HostOS::Windows => "windows",
        _ => "linux",
    }
}

/// Determine architecture string for GraalVM download URLs
fn get_arch_string(arch: HostArch) -> &'static str {
    match arch {
        HostArch::X64 => "x64",
        HostArch::Arm64 => "aarch64",
        HostArch::X86 => "x86",
        HostArch::Arm => "arm",
        _ => "x64",
    }
}

/// Fetch release information from GitHub API
pub fn fetch_releases() -> FnResult<Vec<GitHubRelease>> {
    let url = format!("{}?per_page=100", GITHUB_API_URL);
    Ok(fetch_json(url.as_str())?)
}

/// Find asset matching the platform and version
pub fn find_asset(
    releases: &[GitHubRelease],
    version: &str,
    env: &HostEnvironment,
) -> Option<String> {
    let os = get_os_string(env.os);
    let arch = get_arch_string(env.arch);

    for release in releases {
        if release.tag_name.contains(version) {
            for asset in &release.assets {
                let name = asset.name.to_lowercase();
                // Exclude .sha256 checksum files
                if !name.ends_with(".sha256") && name.contains(os) && name.contains(arch) {
                    return Some(asset.browser_download_url.clone());
                }
            }
        }
    }

    None
}

/// Fetch SHA256 checksum for a binary from GitHub release
pub fn fetch_checksum(download_url: &str) -> FnResult<String> {
    // Construct the .sha256 file URL by appending .sha256 to the download URL
    let checksum_url = format!("{}.sha256", download_url);

    // Fetch the checksum file content
    let checksum_content: String = fetch_json(checksum_url.as_str())?;

    // Extract just the hash (first 64 characters for SHA256)
    let hash = checksum_content
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();

    if hash.is_empty() || hash.len() < 64 {
        return Err(PluginError::Message("Invalid checksum format".into()).into());
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_jdk_prefix() {
        let result = parse_version_from_tag("jdk-25.0.0");
        assert_eq!(result, Some((25, 0, 0)));
    }

    #[test]
    fn test_parse_version_jdk_with_patch() {
        let result = parse_version_from_tag("jdk-21.0.1");
        assert_eq!(result, Some((21, 0, 1)));
    }

    #[test]
    fn test_parse_version_graal_prefix() {
        let result = parse_version_from_tag("graal-25.1.3");
        assert_eq!(result, Some((25, 1, 3)));
    }

    #[test]
    fn test_parse_version_graal_lts() {
        let result = parse_version_from_tag("graal-23.1.0");
        assert_eq!(result, Some((23, 1, 0)));
    }

    #[test]
    fn test_parse_version_vm_prefix() {
        let result = parse_version_from_tag("vm-24.0.0");
        assert_eq!(result, Some((24, 0, 0)));
    }

    #[test]
    fn test_parse_version_invalid() {
        let result = parse_version_from_tag("invalid");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_version_no_numbers() {
        let result = parse_version_from_tag("jdk-abc");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_version_incomplete() {
        let result = parse_version_from_tag("jdk-25.0");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_os_string_linux() {
        assert_eq!(get_os_string(HostOS::Linux), "linux");
    }

    #[test]
    fn test_get_os_string_macos() {
        assert_eq!(get_os_string(HostOS::MacOS), "macos");
    }

    #[test]
    fn test_get_os_string_windows() {
        assert_eq!(get_os_string(HostOS::Windows), "windows");
    }

    #[test]
    fn test_get_arch_string_x64() {
        assert_eq!(get_arch_string(HostArch::X64), "x64");
    }

    #[test]
    fn test_get_arch_string_arm64() {
        assert_eq!(get_arch_string(HostArch::Arm64), "aarch64");
    }

    #[test]
    fn test_get_arch_string_x86() {
        assert_eq!(get_arch_string(HostArch::X86), "x86");
    }

    #[test]
    fn test_get_arch_string_arm() {
        assert_eq!(get_arch_string(HostArch::Arm), "arm");
    }

    #[test]
    fn test_github_release_deserialization() {
        let json = r#"{
            "tag_name": "graal-25.0.0",
            "assets": [
                {
                    "name": "graalvm-ce-java21-windows-x64-25.0.0.zip",
                    "browser_download_url": "https://example.com/download"
                }
            ]
        }"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();
        assert_eq!(release.tag_name, "graal-25.0.0");
        assert_eq!(release.assets.len(), 1);
        assert_eq!(
            release.assets[0].name,
            "graalvm-ce-java21-windows-x64-25.0.0.zip"
        );
    }

    #[test]
    fn test_find_asset_excludes_checksum_files() {
        let releases = vec![GitHubRelease {
            tag_name: "jdk-25.0.0".to_string(),
            assets: vec![
                GitHubAsset {
                    name: "graalvm-community-jdk-25.0.0_linux-x64_bin.tar.gz".to_string(),
                    browser_download_url: "https://example.com/binary.tar.gz".to_string(),
                },
                GitHubAsset {
                    name: "graalvm-community-jdk-25.0.0_linux-x64_bin.tar.gz.sha256".to_string(),
                    browser_download_url: "https://example.com/binary.tar.gz.sha256".to_string(),
                },
            ],
        }];

        let env = HostEnvironment {
            os: HostOS::Linux,
            arch: HostArch::X64,
            ..Default::default()
        };

        let result = find_asset(&releases, "25.0.0", &env);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "https://example.com/binary.tar.gz");
    }

    #[test]
    fn test_checksum_url_generation() {
        let download_url = "https://github.com/oracle/graalvm-ce-builds/releases/download/jdk-25.0.0/graalvm-community-jdk-25.0.0_linux-x64_bin.tar.gz";
        let checksum_url = format!("{}.sha256", download_url);
        assert!(checksum_url.ends_with(".sha256"));
        assert!(checksum_url.contains("graalvm-community-jdk-25.0.0_linux-x64_bin.tar.gz.sha256"));
    }
}
