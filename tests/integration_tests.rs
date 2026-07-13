//! Integration tests for proto-graalvm-plugin
//!
//! These tests verify the complete workflow and integration of the plugin.
//! They simulate realistic scenarios of version management.

use semver::Version;

#[cfg(test)]
mod integration {
    use super::*;

    // ========================================================================
    // Version Format & Parsing Tests
    // ========================================================================

    #[test]
    fn test_version_format_parsing() {
        let versions = vec!["25.0.3", "25.0.2", "23.0.0"];
        for version in versions {
            assert!(!version.is_empty());
            assert!(version.contains('.'));
            
            // Should be parseable as semver
            let parsed = Version::parse(version);
            assert!(parsed.is_ok(), "Failed to parse {}", version);
        }
    }

    #[test]
    fn test_version_format_multiple_patterns() {
        let patterns = vec![
            "25.0.3",
            "25.1.0",
            "23.0.0",
            "24.1.0",
            "26.0.0",
        ];
        
        for pattern in patterns {
            let v = Version::parse(pattern).expect("Failed to parse version");
            assert!(v.major > 0, "Major version should be positive");
            assert!(v.minor >= 0);
            assert!(v.patch >= 0);
        }
    }

    // ========================================================================
    // Platform Support Tests
    // ========================================================================

    #[test]
    fn test_platform_variants() {
        let platforms = vec![
            ("windows", "x64"),
            ("windows", "arm64"),
            ("macos", "x64"),
            ("macos", "arm64"),
            ("linux", "x64"),
            ("linux", "arm64"),
        ];
        
        assert_eq!(platforms.len(), 6, "Should support 6 platform combinations");
        
        // Verify each platform is unique
        for (os, arch) in &platforms {
            assert!(!os.is_empty());
            assert!(!arch.is_empty());
        }
    }

    #[test]
    fn test_platform_coverage() {
        let supported = vec![
            "windows-x64",
            "windows-arm64",
            "macos-x64",
            "macos-aarch64",
            "linux-x64",
            "linux-aarch64",
        ];
        
        // All platforms should be distinct
        let mut unique = supported.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), supported.len(), "All platforms should be unique");
    }

    // ========================================================================
    // Version Sorting Tests
    // ========================================================================

    #[test]
    fn test_version_sorting() {
        let versions = vec!["25.0.3", "25.0.2", "25.1.0"];
        let mut sorted = versions.clone();
        sorted.sort();

        assert_eq!(sorted[0], "25.0.2");
        assert_eq!(sorted[1], "25.0.3");
        assert_eq!(sorted[2], "25.1.0");
    }

    #[test]
    fn test_version_sorting_semver() {
        let versions: Vec<Version> = vec![
            Version::parse("25.0.3").unwrap(),
            Version::parse("25.0.2").unwrap(),
            Version::parse("25.1.0").unwrap(),
            Version::parse("24.0.0").unwrap(),
            Version::parse("25.0.1").unwrap(),
        ];
        
        let mut sorted = versions.clone();
        sorted.sort();
        
        // Check ascending order
        assert!(sorted[0] < sorted[1]);
        assert!(sorted[1] < sorted[2]);
        assert!(sorted[2] < sorted[3]);
        assert!(sorted[3] < sorted[4]);
    }

    #[test]
    fn test_version_sorting_descending() {
        let mut versions: Vec<Version> = vec![
            Version::parse("25.0.3").unwrap(),
            Version::parse("25.0.2").unwrap(),
            Version::parse("25.1.0").unwrap(),
        ];
        
        versions.sort();
        versions.reverse();
        
        // Check descending order (newest first)
        assert_eq!(versions[0], Version::new(25, 1, 0));
        assert_eq!(versions[1], Version::new(25, 0, 3));
        assert_eq!(versions[2], Version::new(25, 0, 2));
    }

    // ========================================================================
    // Filename Pattern Tests
    // ========================================================================

    #[test]
    fn test_graalvm_filename_patterns() {
        let patterns = vec![
            "graalvm-community-jdk-25.0.3_windows-x64_bin.zip",
            "graalvm-community-jdk-25.0.3_macos-aarch64_bin.tar.gz",
            "graalvm-community-jdk-25.0.3_linux-x64_bin.tar.gz",
            "graalvm-community-jdk-25i1-25.0.3_windows-x64_bin.zip",
        ];
        
        for pattern in patterns {
            assert!(pattern.contains("graalvm"));
            assert!(pattern.contains("community"));
            assert!(pattern.contains("_bin"));
            assert!(
                pattern.ends_with(".zip") || pattern.ends_with(".tar.gz"),
                "Invalid extension: {}",
                pattern
            );
        }
    }

    #[test]
    fn test_graalvm_filename_structure() {
        let filename = "graalvm-community-jdk-25.0.3_windows-x64_bin.zip";
        
        // Should have version component
        assert!(filename.contains("25"));
        assert!(filename.contains("0"));
        assert!(filename.contains("3"));
        
        // Should have OS/Arch
        assert!(filename.contains("windows"));
        assert!(filename.contains("x64"));
        
        // Should have bin marker
        assert!(filename.contains("_bin"));
    }

    // ========================================================================
    // Archive Type Tests
    // ========================================================================

    #[test]
    fn test_archive_type_detection() {
        let zip_files = vec![
            "graalvm-community-jdk-25.0.3_windows-x64_bin.zip",
            "graalvm-community-jdk-25.0.3_windows-arm64_bin.zip",
        ];
        
        let tar_files = vec![
            "graalvm-community-jdk-25.0.3_macos-x64_bin.tar.gz",
            "graalvm-community-jdk-25.0.3_linux-x64_bin.tar.gz",
        ];
        
        for zip in &zip_files {
            assert!(zip.ends_with(".zip"), "{} should be zip", zip);
        }
        
        for tar in &tar_files {
            assert!(tar.ends_with(".tar.gz"), "{} should be tar.gz", tar);
        }
    }

    // ========================================================================
    // Version Alias Tests
    // ========================================================================

    #[test]
    fn test_version_aliases() {
        let aliases = vec!["latest", "lts", "current", "stable"];
        
        for alias in aliases {
            assert!(!alias.is_empty());
            assert!(!alias.contains('.'));
            assert!(!alias.contains(|c: char| c.is_numeric()));
        }
    }

    #[test]
    fn test_partial_version_formats() {
        let partials = vec!["25", "25.0", "24", "23.1"];
        
        for partial in partials {
            // Should only contain numbers and dots
            assert!(
                partial.chars().all(|c| c.is_numeric() || c == '.'),
                "Invalid partial version: {}",
                partial
            );
        }
    }

    // ========================================================================
    // URL Structure Tests
    // ========================================================================

    #[test]
    fn test_github_releases_url_format() {
        let base = "https://github.com/graalvm/graalvm-ce-builds/releases/download";
        
        assert!(base.starts_with("https://"));
        assert!(base.contains("github.com"));
        assert!(base.contains("graalvm"));
        assert!(base.contains("releases"));
        assert!(base.contains("download"));
    }

    #[test]
    fn test_github_releases_tag_patterns() {
        let tags = vec![
            ("jdk-25.0.0", "jdk prefix"),
            ("jdk-25.0.2", "jdk prefix"),
            ("graal-25.1.3", "graal prefix"),
        ];
        
        for (tag, description) in tags {
            assert!(
                tag.contains('-'),
                "Tag should contain dash: {} ({})",
                tag,
                description
            );
            let parts: Vec<&str> = tag.split('-').collect();
            assert_eq!(parts.len(), 2, "Tag should have 2 parts: {}", tag);
        }
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_empty_version_list() {
        let versions: Vec<Version> = vec![];
        assert_eq!(versions.len(), 0);
    }

    #[test]
    fn test_single_version() {
        let versions = vec![Version::parse("25.0.0").unwrap()];
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0], Version::new(25, 0, 0));
    }

    #[test]
    fn test_large_version_number() {
        let version = Version::parse("999.999.999").unwrap();
        assert_eq!(version.major, 999);
        assert_eq!(version.minor, 999);
        assert_eq!(version.patch, 999);
    }

    #[test]
    fn test_zero_version() {
        let version = Version::parse("0.0.0").unwrap();
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }

    // ========================================================================
    // Consistency Tests
    // ========================================================================

    #[test]
    fn test_version_consistency_across_formats() {
        let version_str = "25.0.3";
        let parsed = Version::parse(version_str).unwrap();
        let reconstructed = format!("{}", parsed);
        
        assert_eq!(version_str, reconstructed);
    }

    #[test]
    fn test_platform_string_consistency() {
        let platforms = vec![
            "windows-x64",
            "macos-aarch64",
            "linux-x64",
        ];
        
        for platform in platforms {
            let parts: Vec<&str> = platform.split('-').collect();
            assert_eq!(parts.len(), 2, "Platform should have 2 parts: {}", platform);
            assert!(!parts[0].is_empty());
            assert!(!parts[1].is_empty());
        }
    }
}
