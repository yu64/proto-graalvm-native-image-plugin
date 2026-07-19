use extism_pdk::FnResult;
use proto_pdk::{fetch_json, HostArch, HostEnvironment, HostOS};
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
fn get_os_string(os: HostOS) -> &'static str {
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
    let releases = fetch_json::<Vec<GitHubRelease>>(url.as_str())?;
    Ok(releases)
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
                if name.contains(os) && name.contains(arch) {
                    return Some(asset.browser_download_url.clone());
                }
            }
        }
    }

    None
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
    fn test_parse_version_graal_prefix() {
        let result = parse_version_from_tag("graal-25.1.3");
        assert_eq!(result, Some((25, 1, 3)));
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
    fn test_get_os_string() {
        assert_eq!(get_os_string(HostOS::Linux), "linux");
        assert_eq!(get_os_string(HostOS::MacOS), "macos");
        assert_eq!(get_os_string(HostOS::Windows), "windows");
    }

    #[test]
    fn test_get_arch_string() {
        assert_eq!(get_arch_string(HostArch::X64), "x64");
        assert_eq!(get_arch_string(HostArch::Arm64), "aarch64");
        assert_eq!(get_arch_string(HostArch::X86), "x86");
    }
}
