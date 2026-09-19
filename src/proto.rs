//! WASM hooks for version discovery, installation, and shell activation.

use crate::{
  graalvm::{
    COMMUNITY_DISTRIBUTION, LAUNCHER_DIR, accepts_unscoped_version, executable_config,
    is_supported_archive, native_image_executable_name, native_image_launcher,
  },
  version::{from_java_version, to_java_version},
};
use extism_pdk::*;
use proto_pdk::*;
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;

const FOOJAY_API_URL: &str = "https://api.foojay.io/disco/v3.0";

// #############################################################################
// MARK: Foojay API models

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

// Download URLs and checksums are fetched separately using the package ID.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PackageInfo {
  checksum: String,
  checksum_type: String,
  checksum_uri: String,
  direct_download_uri: String,
  filename: String,
}

// #############################################################################
// MARK: Tool registration

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
  Ok(Json(RegisterToolOutput {
    name: "GraalVM Native Image".into(),
    type_of: PluginType::Language,
    minimum_proto_version: Some(Version::new(0, 59, 0)),
    plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
    ..Default::default()
  }))
}

// #############################################################################
// MARK: Version discovery and resolution

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

  // Let proto perform version matching after the scope has been validated.
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

// #############################################################################
// MARK: Package download

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

  let env = get_host_environment()?;
  let requested = to_java_version(&input.context.version);
  let package = fetch_packages(&env, Some(&requested))?
    .into_iter()
    .find(|package| package.java_version == requested)
    .ok_or_else(|| {
      plugin_err!(
        "No GraalVM Community package is available for version <hash>{requested}</hash> on {} {}.",
        env.os,
        env.arch
      )
    })?;
  let info = fetch_package_info(&package.id)?;

  // Use inline digests when supported; keep the checksum URL as a fallback.
  let checksum =
    if matches!(info.checksum_type.as_str(), "sha256" | "sha512") && !info.checksum.is_empty() {
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

// #############################################################################
// MARK: Executables and activation

#[plugin_fn]
pub fn locate_executables(
  Json(input): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
  let env = get_host_environment()?;
  let bin_dir = native_image_bin_dir(&input.install_dir);
  let is_windows = env.os.is_windows();

  // proto adds the primary executable's parent to PATH even when exes_dirs
  // is empty. Keep the launcher separate from the rest of the JDK tools.
  let launcher_dir = input.install_dir.join(LAUNCHER_DIR);
  std::fs::create_dir_all(&launcher_dir)?;

  let launcher_path = launcher_dir.join(native_image_executable_name(is_windows));
  let launcher = native_image_launcher(&bin_dir.to_string_lossy(), is_windows);

  // Locating executables also runs on activation, so avoid unnecessary writes.
  if std::fs::read_to_string(&launcher_path).ok().as_deref() != Some(&launcher) {
    std::fs::write(&launcher_path, launcher)?;
  }

  Ok(Json(executable_config(is_windows)))
}

#[plugin_fn]
pub fn activate_environment(
  Json(input): Json<ActivateEnvironmentInput>,
) -> FnResult<Json<ActivateEnvironmentOutput>> {
  let mut output = ActivateEnvironmentOutput::default();
  let home = graalvm_home(&input.context.tool_dir);

  // Export the host path, not the path used inside the WASM filesystem.
  if let Some(path) = home.real_path_string() {
    output.env.insert("GRAALVM_HOME".into(), path);
  }

  Ok(Json(output))
}

// #############################################################################
// MARK: Version validation

fn validate_unresolved_version(spec: &UnresolvedVersionSpec) -> FnResult<()> {
  accepts_unscoped_version(spec.get_scope()).map_err(|message| plugin_err!("{message}"))
}

fn validate_resolved_version(spec: &VersionSpec) -> FnResult<()> {
  accepts_unscoped_version(spec.get_scope())
    .map_err(|_| plugin_err!("Only GraalVM Community versions are supported by this plugin."))
}

// #############################################################################
// MARK: Installation layout

// macOS archives wrap the JDK in an application bundle.
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

// #############################################################################
// MARK: Foojay API requests

fn fetch_packages(env: &HostEnvironment, version: Option<&str>) -> FnResult<Vec<Package>> {
  let mut url = format!(
    "{FOOJAY_API_URL}/packages?distro={COMMUNITY_DISTRIBUTION}&package_type=jdk&release_status=ga&javafx_bundled=false&archive_type=tar&archive_type=tar.gz&archive_type=tar.xz&archive_type=zip&operating_system={}&architecture={}",
    foojay_os(env)?,
    foojay_arch(env)?
  );

  if let Some(version) = version {
    url.push_str("&version=");
    // A literal plus in build metadata must not become a query-string space.
    url.push_str(&version.replace('+', "%2B"));
  }

  let response: FoojayResponse<Package> = fetch_json(&url)?;

  // Apply distribution, archive, and libc rules to the returned candidates too.
  Ok(
    response
      .result
      .into_iter()
      .filter(|package| is_supported_package(package, env))
      .collect(),
  )
}

fn fetch_package_info(id: &str) -> FnResult<PackageInfo> {
  let response: FoojayResponse<PackageInfo> = fetch_json(&format!("{FOOJAY_API_URL}/ids/{id}"))?;

  response
    .result
    .into_iter()
    .next()
    .ok_or_else(|| plugin_err!("No download metadata exists for GraalVM package {id}."))
}

// #############################################################################
// MARK: Host platform matching

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

// #############################################################################
// MARK: Tests

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
}
