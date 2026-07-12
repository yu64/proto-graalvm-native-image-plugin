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

    // Java メインコマンド
    exes.insert(
        "java".into(),
        ExecutableConfig::new_primary(env.os.for_native("bin/java", "bin/java.exe")),
    );

    // Java コンパイラ
    exes.insert(
        "javac".into(),
        ExecutableConfig::new(env.os.for_native("bin/javac", "bin/javac.exe")),
    );

    // その他の重要なコマンド
    exes.insert(
        "jshell".into(),
        ExecutableConfig::new(env.os.for_native("bin/jshell", "bin/jshell.exe")),
    );

    exes.insert(
        "jar".into(),
        ExecutableConfig::new(env.os.for_native("bin/jar", "bin/jar.exe")),
    );

    exes.insert(
        "javadoc".into(),
        ExecutableConfig::new(env.os.for_native("bin/javadoc", "bin/javadoc.exe")),
    );

    // native-image コマンド（GraalVM 特有）
    exes.insert(
        "native-image".into(),
        ExecutableConfig::new(env.os.for_native("bin/native-image", "bin/native-image.exe")),
    );

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
        let candidate = match alias.as_str() {
            "latest" => "latest",
            "lts" => "lts",
            _ => return Ok(output),
        };

        output.candidate = Some(UnresolvedVersionSpec::Alias(candidate.to_owned()));
    }

    Ok(Json(output))
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
