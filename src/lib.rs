//! GraalVM Community Edition Proto WASM Plugin
//!
//! This plugin provides integration between moonrepo's proto version manager
//! and GraalVM Community Edition, enabling seamless version management across
//! multiple platforms and architectures.
//!
//! # Architecture
//!
//! - `release_parser`: Fetches and parses GitHub Releases API
//! - `url_builder`: Constructs download URLs for prebuilt binaries
//! - `version_resolver`: Resolves version aliases to concrete versions

use proto_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod release_parser;
mod url_builder;
mod version_resolver;

use release_parser::*;
use url_builder::*;
use version_resolver::*;

const NAME: &str = "GraalVM CE";
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

// Supported GraalVM CE executables
const GRAALVM_EXECUTABLES: &[&str] = &[
    "java",
    "javac",
    "jar",
    "javadoc",
    "jshell",
    "native-image",
    "jps",
    "jstat",
    "jstatd",
    "jrunscript",
    "jdb",
    "jcmd",
    "jhsdb",
    "keytool",
    "rmid",
    "rmiregistry",
];

// ============================================================================
// PLUGIN REGISTRATION
// ============================================================================

#[plugin_fn]
pub fn register_tool(Json(input): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::Runtime,
        minimum_proto_version: Some(Version::new(0, 42, 0)),
        plugin_version: Version::parse(PLUGIN_VERSION).ok(),
        ..RegisterToolOutput::default()
    }))
}

// ============================================================================
// DOWNLOAD & INSTALLATION
// ============================================================================

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    // 環境情報を取得
    let env = get_host_environment()?;

    // サポートされているプラットフォームを確認
    check_supported_os_and_arch(
        NAME,
        &env,
        permutations![
            HostOS::Windows => [HostArch::X64, HostArch::Arm64],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
            HostOS::Linux => [HostArch::X64, HostArch::Arm64],
        ],
    )?;

    // バージョンを取得
    let version = &input.context.version;

    // GitHub Releases から該当バージョンのアセットを取得
    let download_info = build_download_url(version, &env)?;

    Ok(Json(download_info))
}

#[plugin_fn]
pub fn unpack_archive(Json(input): Json<UnpackArchiveInput>) -> FnResult<()> {
    // GraalVM のアーカイブ構造に対応
    if input.input_file.ends_with(".zip") {
        unzip(input.input_file, input.output_dir)?;
    } else {
        untar(input.input_file, input.output_dir)?;
    }

    Ok(())
}

// ============================================================================
// EXECUTABLE LOCATIONS
// ============================================================================

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;

    let mut exes = HashMap::new();

    // Primary executable
    exes.insert(
        "java".into(),
        ExecutableConfig::new_primary(env.os.for_native("bin/java", "bin/java.exe")),
    );

    // Compiler and tools
    let tools = vec![
        ("javac", "Java compiler"),
        ("jar", "Java archive tool"),
        ("javadoc", "Documentation generator"),
        ("jshell", "Interactive Java shell"),
        ("jps", "Process status"),
        ("jstat", "Statistics monitor"),
        ("jdb", "Debugger"),
        ("jcmd", "Diagnostic command"),
        ("keytool", "Key management"),
        ("native-image", "GraalVM native-image compiler"),
    ];

    for (tool, _description) in tools {
        exes.insert(
            tool.into(),
            ExecutableConfig::new(env.os.for_native(
                &format!("bin/{}", tool),
                &format!("bin/{}.exe", tool),
            )),
        );
    }

    Ok(Json(LocateExecutablesOutput {
        exes,
        ..LocateExecutablesOutput::default()
    }))
}

// ============================================================================
// VERSION MANAGEMENT
// ============================================================================

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    // GitHub Releases API から GraalVM CE のバージョンリストを取得
    let versions = load_graalvm_versions()?;

    Ok(Json(versions))
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    let mut output = ResolveVersionOutput::default();

    if let UnresolvedVersionSpec::Alias(alias) = input.initial {
        // Normalize common aliases
        let normalized = match alias.as_str() {
            "latest" | "current" | "stable" => "latest",
            "lts" | "long-term-support" => "lts",
            // Partial version matching (e.g., "25", "25.0")
            partial if is_partial_version(partial) => partial,
            _ => return Ok(output),
        };

        output.candidate = Some(UnresolvedVersionSpec::Alias(normalized.to_owned()));
    }

    Ok(Json(output))
}

/// Check if input looks like a partial version (e.g., "25", "25.0")
fn is_partial_version(s: &str) -> bool {
    s.chars().all(|c| c.is_numeric() || c == '.')
}

// ============================================================================
// VERSION DETECTION
// ============================================================================

#[plugin_fn]
pub fn detect_version_files(
    Json(_): Json<DetectVersionInput>,
) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(DetectVersionOutput {
        files: vec![
            ".java-version".into(),
            ".graalvm-version".into(),
            "package.json".into(),
        ],
        ignore: vec!["node_modules".into(), "target".into()],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let mut version = None;

    match input.file.as_str() {
        "package.json" => {
            if let Ok(json) = serde_json::from_str::<PackageJsonEngines>(&input.content) {
                if let Some(engines) = json.engines {
                    if let Some(graalvm_constraint) = engines.graalvm {
                        version = Some(UnresolvedVersionSpec::parse(&graalvm_constraint)?);
                    }
                }
            }
        }
        _ => {
            // その他のファイルは直接パース
            version = Some(UnresolvedVersionSpec::parse(input.content.trim())?);
        }
    }

    Ok(Json(ParseVersionFileOutput { version }))
}

// ============================================================================
// Helper Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct PackageJsonEngines {
    engines: Option<Engines>,
}

#[derive(Debug, Deserialize)]
struct Engines {
    graalvm: Option<String>,
}

// ============================================================================
// Module Functions
// ============================================================================

fn load_graalvm_versions() -> FnResult<LoadVersionsOutput> {
    // release_parser モジュールから GraalVM バージョンを取得
    let versions = release_parser::fetch_and_parse_releases()?;

    let mut output = LoadVersionsOutput::default();

    if !versions.is_empty() {
        output.latest = Some(versions[0].clone());
        output.versions = versions;
    }

    Ok(output)
}

fn build_download_url(
    version: &str,
    env: &HostEnvironment,
) -> FnResult<DownloadPrebuiltOutput> {
    // url_builder モジュールから ダウンロード URL を構築
    url_builder::build_download_info(version, env)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_partial_version_major() {
        assert!(is_partial_version("25"));
    }

    #[test]
    fn test_is_partial_version_major_minor() {
        assert!(is_partial_version("25.0"));
    }

    #[test]
    fn test_is_partial_version_full() {
        assert!(is_partial_version("25.0.3"));
    }

    #[test]
    fn test_is_partial_version_invalid() {
        assert!(!is_partial_version("latest"));
        assert!(!is_partial_version("v25.0.0"));
        assert!(!is_partial_version("25.0.0-beta"));
    }
}
