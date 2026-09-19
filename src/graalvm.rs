//! Pure GraalVM selection rules, kept independent from the WASM host so they
//! can be tested on the developer host.

use proto_pdk_api::{ExecutableConfig, LocateExecutablesOutput};
use rustc_hash::FxHashMap;

// #############################################################################
// MARK: Constants

pub const COMMUNITY_DISTRIBUTION: &str = "graalvm_community";
pub const LAUNCHER_DIR: &str = "native-image-bin";

// #############################################################################
// MARK: Package selection

/// Distribution selection is fixed by this plugin, not by a version scope.
pub fn accepts_unscoped_version(scope: Option<&str>) -> Result<(), &'static str> {
  if scope.is_some() {
    Err("Only an unscoped GraalVM version may be specified, for example 25.0.3.")
  } else {
    Ok(())
  }
}

/// Missing libc metadata is accepted; an explicit incompatible libc is rejected.
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

// #############################################################################
// MARK: Native Image launcher

/// Registers only the dedicated launcher, keeping the JDK bin directory off PATH.
pub fn executable_config(is_windows: bool) -> LocateExecutablesOutput {
  let exe_name = native_image_executable_name(is_windows);
  let exe_path = format!("{LAUNCHER_DIR}/{exe_name}");

  LocateExecutablesOutput {
    exes: FxHashMap::from_iter([(
      "native-image".into(),
      ExecutableConfig {
        // Relative launchers must run from their installation directory,
        // rather than a symlink in proto's shared bin directory.
        no_bin: true,
        update_perms: true,
        ..ExecutableConfig::new_primary(exe_path)
      },
    )]),
    ..Default::default()
  }
}

/// Resolves the real executable relative to the launcher so installations can move.
pub fn native_image_launcher(bin_dir: &str, is_windows: bool) -> String {
  if is_windows {
    format!(
      "@echo off\r\ncall \"%~dp0..\\{}\\native-image.cmd\" %*\r\nexit /b %errorlevel%\r\n",
      bin_dir.replace('/', "\\")
    )
  } else {
    format!("#!/bin/sh\nexec \"$(dirname \"$0\")/../{bin_dir}/native-image\" \"$@\"\n")
  }
}

pub fn native_image_executable_name(is_windows: bool) -> &'static str {
  if is_windows {
    "native-image.cmd"
  } else {
    "native-image"
  }
}

// #############################################################################
// MARK: Tests

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

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

  #[test]
  fn locate_executables_exposes_native_image_command() {
    for (is_windows, expected) in [
      (true, "native-image-bin/native-image.cmd"),
      (false, "native-image-bin/native-image"),
    ] {
      let output = executable_config(is_windows);

      assert_eq!(output.exes.len(), 1);

      let executable = &output.exes["native-image"];
      assert_eq!(executable.exe_path, Some(PathBuf::from(expected)));
      assert!(executable.primary);
      assert!(executable.no_bin);
      assert!(executable.update_perms);
      assert!(
        output.exes_dirs.is_empty(),
        "The JDK bin directory must not be added to PATH"
      );
    }
  }

  #[test]
  fn launcher_preserves_arguments_and_exit_status() {
    // Spaces in the installation path exercise quoting in both launchers.
    let root = std::env::temp_dir().join(format!(
      "graalvm launcher test {} {}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let windows = cfg!(windows);
    let name = native_image_executable_name(windows);
    let bin = if windows { "bin" } else { "Contents/Home/bin" };

    std::fs::create_dir_all(root.join(bin)).unwrap();
    std::fs::create_dir_all(root.join(LAUNCHER_DIR)).unwrap();

    // A stub executable makes argument forwarding and exit status observable.
    let target = root.join(bin).join(name);
    std::fs::write(
      &target,
      if windows {
        "@echo off\r\nif \"%~1\"==\"two words\" exit /b 37\r\nexit /b 1\r\n"
      } else {
        "#!/bin/sh\n[ \"$1\" = 'two words' ] && exit 37\nexit 1\n"
      },
    )
    .unwrap();

    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;

      std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let launcher = PathBuf::from(LAUNCHER_DIR).join(name);
    std::fs::write(root.join(&launcher), native_image_launcher(bin, windows)).unwrap();

    let mut command = std::process::Command::new(if windows { "cmd.exe" } else { "sh" });
    if windows {
      command.args(["/d", "/c"]);
    }

    let result = command
      .current_dir(&root)
      .arg(launcher)
      .arg("two words")
      .status();

    // Clean up before asserting so a failed child process leaves no fixture.
    std::fs::remove_dir_all(&root).unwrap();
    assert_eq!(result.unwrap().code(), Some(37));
  }
}
