use crate::{
    graalvm::{
        COMMUNITY_DISTRIBUTION, accepts_unscoped_version, is_supported_archive,
        native_image_executable_name,
    },
    version::{from_java_version, to_java_version},
};
use extism_pdk::*;
use proto_pdk::*;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;

const FOOJAY_API_URL: &str = "https://api.foojay.io/disco/v3.0";

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FoojayResponse<T> {
    result: Vec<T>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Package {
    architecture: String,
    archive_type: String,
    distribution: String,
    id: String,
    java_version: String,
    lib_c_type: Option<String>,
    operating_system: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PackageInfo {
    checksum: String,
    checksum_type: String,
    checksum_uri: String,
    direct_download_uri: String,
    filename: String,
}

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: "GraalVM Native Image".into(),
        type_of: PluginType::Language,
        requires: vec!["java".into()],
        minimum_proto_version: Some(Version::new(0, 59, 0)),
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(DetectVersionOutput {
        files: vec![".graalvm-version".into()],
        ignore: vec![],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let version = if input.file == ".graalvm-version" {
        let value = input.content.trim();
        if value.is_empty() {
            None
        } else {
            let spec = UnresolvedVersionSpec::parse(value)?;
            validate_unresolved_version(&spec)?;
            Some(spec)
        }
    } else {
        None
    };
    Ok(Json(ParseVersionFileOutput { version }))
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    validate_unresolved_version(&input.initial)?;
    Ok(Json(ResolveVersionOutput::default()))
}

#[plugin_fn]
pub fn load_versions(_: Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let env = get_host_environment()?;
    let packages = fetch_packages(&env, None)?;
    let versions = packages
        .iter()
        .filter(|package| package.distribution == COMMUNITY_DISTRIBUTION)
        .map(|package| from_java_version(&package.java_version))
        .collect::<Vec<_>>();
    Ok(Json(LoadVersionsOutput::from(versions)?))
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    if input.context.version.is_canary() {
        return Err(plugin_err!(
            "Canary releases are not supported for GraalVM Native Image."
        ));
    }
    validate_resolved_version(&input.context.version)?;

    // Check if java plugin manages GraalVM
    if let Ok(Some(java_version)) = get_host_env_var("PROTO_JAVA_VERSION") {
        if java_version.starts_with("graalvm") {
            let requested_version = input.context.version.to_string();
            
            // If "bundled" is specified, use java's GraalVM
            if requested_version == "bundled" {
                return Err(plugin_err!(
                    "Version \"bundled\" requires java plugin with GraalVM. Please install a GraalVM version with the java plugin."
                ));
            }
            
            // Extract version from java_version (e.g., "graalvm-community-25.0.1" -> "25.0.1")
            if let Some(java_graalvm_version) = java_version.split('-').last() {
                // If versions match, use java's GraalVM
                if requested_version == java_graalvm_version {
                    return Err(plugin_err!(
                        "Version <hash>{requested_version}</hash> is managed by the java plugin. The GraalVM from java will be used."
                    ));
                }
            }
        }
    }

    let env = get_host_environment()?;
    let requested = to_java_version(&input.context.version);
    let package = fetch_packages(&env, Some(&requested))?
        .into_iter()
        .find(|package| package.java_version == requested)
        .ok_or_else(|| plugin_err!("No GraalVM Community package is available for version <hash>{requested}</hash> on {} {}.", env.os, env.arch))?;
    let info = fetch_package_info(&package.id)?;

    let checksum = if matches!(info.checksum_type.as_str(), "sha256" | "sha512")
        && !info.checksum.is_empty()
    {
        Some(Checksum::from_str(&format!(
            "{}:{}",
            info.checksum_type, info.checksum
        ))?)
    } else {
        None
    };

    Ok(Json(DownloadPrebuiltOutput {
        archive_prefix: Some("*".into()),
        checksum,
        checksum_url: (!info.checksum_uri.is_empty()).then_some(info.checksum_uri),
        download_name: Some(info.filename),
        download_url: info.direct_download_uri,
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(input): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;
    let requested_version = input.context.version.to_string();
    let exe_name = native_image_executable_name(env.os.is_windows());
    
    // Check if java plugin manages GraalVM
    if let Ok(Some(java_version)) = get_host_env_var("PROTO_JAVA_VERSION") {
        if java_version.starts_with("graalvm") {
            // Extract version from java_version (e.g., "graalvm-community-25.0.1" -> "25.0.1")
            if let Some(java_graalvm_version) = java_version.split('-').last() {
                // If "bundled" or versions match, use java's GraalVM
                if requested_version == "bundled" || requested_version == java_graalvm_version {
                    if let Ok(Some(java_home)) = get_host_env_var("JAVA_HOME") {
                        let bin_subdir = if env.os.is_windows() { "bin" } else { "bin" };
                        let bin_dir = PathBuf::from(java_home).join(bin_subdir);
                        let exe_path = format!(
                            "{}/{}",
                            bin_dir.to_string_lossy().replace('\\', "/"),
                            exe_name
                        );
                        
                        return Ok(Json(LocateExecutablesOutput {
                            exes: FxHashMap::from_iter([(
                                "native-image".into(),
                                ExecutableConfig::new_primary(exe_path),
                            )]),
                            exes_dirs: vec![bin_dir],
                            ..Default::default()
                        }));
                    }
                }
            }
        }
    }
    
    // Otherwise, use the independently installed GraalVM
    let bin_dir = native_image_bin_dir(&input.context.tool_dir);
    let exe_path = format!(
        "{}/{}",
        bin_dir.to_string_lossy().replace('\\', "/"),
        exe_name
    );

    Ok(Json(LocateExecutablesOutput {
        exes: FxHashMap::from_iter([(
            "native-image".into(),
            ExecutableConfig::new_primary(exe_path),
        )]),
        exes_dirs: vec![bin_dir],
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn activate_environment(
    Json(input): Json<ActivateEnvironmentInput>,
) -> FnResult<Json<ActivateEnvironmentOutput>> {
    let mut output = ActivateEnvironmentOutput::default();
    let requested_version = input.context.version.to_string();
    
    // Check if java plugin manages GraalVM
    if let Ok(Some(java_version)) = get_host_env_var("PROTO_JAVA_VERSION") {
        if java_version.starts_with("graalvm") {
            // Extract version from java_version (e.g., "graalvm-community-25.0.1" -> "25.0.1")
            if let Some(java_graalvm_version) = java_version.split('-').last() {
                // If "bundled" or versions match, use java's GraalVM
                if requested_version == "bundled" || requested_version == java_graalvm_version {
                    if let Ok(Some(java_home)) = get_host_env_var("JAVA_HOME") {
                        output.env.insert("GRAALVM_HOME".into(), java_home);
                        return Ok(Json(output));
                    }
                }
            }
        }
    }
    
    // Otherwise, use the independently installed GraalVM
    let home = graalvm_home(&input.context.tool_dir);
    if let Some(path) = home.real_path_string() {
        output.env.insert("GRAALVM_HOME".into(), path);
    }
    Ok(Json(output))
}

fn validate_unresolved_version(spec: &UnresolvedVersionSpec) -> FnResult<()> {
    accepts_unscoped_version(spec.get_scope()).map_err(|message| plugin_err!("{message}"))
}

fn validate_resolved_version(spec: &VersionSpec) -> FnResult<()> {
    accepts_unscoped_version(spec.get_scope())
        .map_err(|_| plugin_err!("Only GraalVM Community versions are supported by this plugin."))
}

fn native_image_bin_dir(tool_dir: &VirtualPath) -> PathBuf {
    if tool_dir.join("Contents").join("Home").exists() {
        PathBuf::from("Contents/Home/bin")
    } else {
        PathBuf::from("bin")
    }
}

fn graalvm_home(tool_dir: &VirtualPath) -> VirtualPath {
    let macos_home = tool_dir.join("Contents").join("Home");
    if macos_home.exists() {
        macos_home
    } else {
        tool_dir.to_owned()
    }
}

fn fetch_packages(env: &HostEnvironment, version: Option<&str>) -> FnResult<Vec<Package>> {
    let mut url = format!(
        "{FOOJAY_API_URL}/packages?distro={COMMUNITY_DISTRIBUTION}&package_type=jdk&release_status=ga&javafx_bundled=false&archive_type=tar&archive_type=tar.gz&archive_type=tar.xz&archive_type=zip&operating_system={}&architecture={}",
        foojay_os(env)?,
        foojay_arch(env)?
    );
    if let Some(version) = version {
        url.push_str("&version=");
        url.push_str(&version.replace('+', "%2B"));
    }
    let response: FoojayResponse<Package> = fetch_json(&url)?;
    Ok(response
        .result
        .into_iter()
        .filter(|package| is_supported_package(package, env))
        .collect())
}

fn fetch_package_info(id: &str) -> FnResult<PackageInfo> {
    let response: FoojayResponse<PackageInfo> = fetch_json(&format!("{FOOJAY_API_URL}/ids/{id}"))?;
    response
        .result
        .into_iter()
        .next()
        .ok_or_else(|| plugin_err!("No download metadata exists for GraalVM package {id}."))
}

fn is_supported_package(package: &Package, env: &HostEnvironment) -> bool {
    is_supported_archive(
        &package.distribution,
        &package.archive_type,
        package.lib_c_type.as_deref(),
        expected_libc(env),
    )
}

fn expected_libc(env: &HostEnvironment) -> &'static str {
    let expected = if env.os.is_linux() {
        if env.libc == HostLibc::Musl {
            "musl"
        } else {
            "glibc"
        }
    } else if env.os.is_mac() {
        "libc"
    } else {
        "c_std_lib"
    };
    expected
}

fn foojay_os(env: &HostEnvironment) -> FnResult<&'static str> {
    match env.os {
        HostOS::Linux => Ok("linux"),
        HostOS::MacOS => Ok("macos"),
        HostOS::Windows => Ok("windows"),
        _ => Err(plugin_err!(
            "GraalVM Native Image does not support {}.",
            env.os
        )),
    }
}

fn foojay_arch(env: &HostEnvironment) -> FnResult<&'static str> {
    match env.arch {
        HostArch::X64 => Ok("x64"),
        HostArch::Arm64 => Ok("aarch64"),
        _ => Err(plugin_err!(
            "GraalVM Native Image does not support {}.",
            env.arch
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(os: HostOS, libc: HostLibc) -> HostEnvironment {
        HostEnvironment {
            os,
            libc,
            ..HostEnvironment::default()
        }
    }

    #[test]
    fn only_supports_target_platforms_and_architectures() {
        let linux = env(HostOS::Linux, HostLibc::Gnu);
        assert_eq!(foojay_os(&linux).unwrap(), "linux");
        assert_eq!(foojay_arch(&linux).unwrap(), "x64");
        let arm = HostEnvironment {
            arch: HostArch::Arm,
            ..linux
        };
        assert!(foojay_arch(&arm).is_err());
    }

    #[test]
    fn locate_executables_exposes_native_image_command() {
        let input = LocateExecutablesInput {
            context: ToolContext {
                tool_dir: VirtualPath::from("/path/to/graalvm"),
                ..ToolContext::default()
            },
        };
        let result = locate_executables(Json(input)).unwrap();
        let output = result.0;
        assert!(
            output.exes.contains_key("native-image"),
            "Expected 'native-image' command in executables"
        );
    }

    #[test]
    fn register_tool_requires_java_plugin() {
        let result = register_tool(Json(RegisterToolInput::default())).unwrap();
        let output = result.0;
        assert!(
            output.requires.contains(&"java".to_string()),
            "Expected graalvm-native-image to require 'java' plugin"
        );
    }
}
