//! Pure GraalVM selection rules, kept independent from the WASM host so they
//! can be tested on the developer host.

pub const COMMUNITY_DISTRIBUTION: &str = "graalvm_community";

pub fn accepts_unscoped_version(scope: Option<&str>) -> Result<(), &'static str> {
    if scope.is_some() {
        Err("Only an unscoped GraalVM version may be specified, for example 25.0.3.")
    } else {
        Ok(())
    }
}

pub fn is_supported_archive(
    distribution: &str,
    archive_type: &str,
    package_libc: Option<&str>,
    expected_libc: &str,
) -> bool {
    distribution == COMMUNITY_DISTRIBUTION
        && matches!(archive_type, "tar" | "tar.gz" | "tar.xz" | "zip")
        && package_libc.is_none_or(|libc| libc == expected_libc)
}

pub fn native_image_executable_name(is_windows: bool) -> &'static str {
    if is_windows {
        "native-image.cmd"
    } else {
        "native-image"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_java_distribution_scopes() {
        assert!(accepts_unscoped_version(None).is_ok());
        assert!(accepts_unscoped_version(Some("graalvm-community")).is_err());
        assert!(accepts_unscoped_version(Some("temurin")).is_err());
    }

    #[test]
    fn accepts_only_graalvm_community_archives() {
        assert!(is_supported_archive(
            COMMUNITY_DISTRIBUTION,
            "tar.gz",
            Some("glibc"),
            "glibc"
        ));
        assert!(!is_supported_archive(
            "temurin",
            "tar.gz",
            Some("glibc"),
            "glibc"
        ));
        assert!(!is_supported_archive(
            COMMUNITY_DISTRIBUTION,
            "dmg",
            Some("libc"),
            "libc"
        ));
    }

    #[test]
    fn selects_only_the_host_libc_variant() {
        assert!(is_supported_archive(
            COMMUNITY_DISTRIBUTION,
            "tar.gz",
            Some("musl"),
            "musl"
        ));
        assert!(!is_supported_archive(
            COMMUNITY_DISTRIBUTION,
            "tar.gz",
            Some("musl"),
            "glibc"
        ));
        assert!(is_supported_archive(
            COMMUNITY_DISTRIBUTION,
            "zip",
            None,
            "c_std_lib"
        ));
    }

    #[test]
    fn uses_the_windows_command_launcher() {
        assert_eq!(native_image_executable_name(true), "native-image.cmd");
        assert_eq!(native_image_executable_name(false), "native-image");
    }
}
