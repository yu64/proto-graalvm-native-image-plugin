use crate::graalvm_api;
use extism_pdk::*;
use proto_pdk::*;
use rustc_hash::FxHashMap;

static NAME: &str = "GraalVM";

// Main GraalVM binaries
static GRAALVM_BINS: [&str; 2] = ["java", "javac"];

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(DetectVersionOutput {
        files: vec![".graalvm-version".into(), ".java-version".into()],
        ignore: vec![],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let mut version = None;

    if input.file == ".graalvm-version" || input.file == ".java-version" {
        let content = input.content.trim();

        // Strip any prefix and find the first numeric digit
        let version_str = if let Some(idx) = content.find(|c: char| c.is_ascii_digit()) {
            &content[idx..]
        } else {
            content
        };

        if !version_str.is_empty() {
            version = Some(UnresolvedVersionSpec::parse(version_str)?);
        }
    }

    Ok(Json(ParseVersionFileOutput { version }))
}

#[plugin_fn]
pub fn activate_environment(
    Json(input): Json<ActivateEnvironmentInput>,
) -> FnResult<Json<ActivateEnvironmentOutput>> {
    let tool_dir = input
        .context
        .tool_dir
        .real_path_string()
        .ok_or(PluginError::Message(
            "Could not determine real tool directory".into(),
        ))?;
    let env = get_host_environment()?;

    let java_home = match env.os {
        HostOS::MacOS => format!("{tool_dir}/Contents/Home"),
        _ => tool_dir,
    };

    Ok(Json(ActivateEnvironmentOutput {
        env: [("JAVA_HOME".into(), java_home)].into_iter().collect(),
        ..ActivateEnvironmentOutput::default()
    }))
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let version = input
        .context
        .version
        .as_version()
        .ok_or(PluginError::Message("Unsupported version type.".into()))?;

    let env = get_host_environment()?;
    let releases = graalvm_api::fetch_releases()?;

    let version_str = format!("{}.{}.{}", version.major, version.minor, version.patch);
    let download_url = graalvm_api::find_asset(&releases, &version_str, &env).ok_or(
        PluginError::Message(format!(
            "No asset found for version {} on {}/{}",
            version_str, env.os, env.arch
        )),
    )?;

    // Archive prefix is the directory name inside the archive
    let archive_prefix = format!(
        "graalvm-ce-{}-{}",
        version_str,
        graalvm_api::get_os_string(env.os)
    );

    Ok(Json(DownloadPrebuiltOutput {
        download_url,
        archive_prefix: Some(archive_prefix),
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;

    let exes: FxHashMap<String, _> = GRAALVM_BINS
        .into_iter()
        .map(|bin| {
            let exe_name = env.os.get_exe_name(bin);
            let exe_path = match env.os {
                HostOS::MacOS => format!("Contents/Home/bin/{exe_name}"),
                _ => format!("bin/{exe_name}"),
            };

            let config = match bin {
                "java" => ExecutableConfig::new_primary(exe_path),
                _ => ExecutableConfig::new(exe_path),
            };

            (String::from(bin), config)
        })
        .collect();

    let exes_dirs = match env.os {
        HostOS::MacOS => vec!["Contents/Home/bin".into()],
        _ => vec!["bin".into()],
    };

    Ok(Json(LocateExecutablesOutput {
        exes_dirs,
        exes,
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn load_versions(_: ()) -> FnResult<Json<LoadVersionsOutput>> {
    let releases = graalvm_api::fetch_releases()?;

    let mut versions: Vec<VersionSpec> = releases
        .iter()
        .filter_map(|release| {
            graalvm_api::parse_version_from_tag(&release.tag_name).map(|(major, minor, patch)| {
                VersionSpec::parse(&format!("{}.{}.{}", major, minor, patch))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    versions.sort();
    versions.reverse();

    let latest = versions
        .first()
        .and_then(|v| v.as_version())
        .cloned()
        .unwrap_or(Version::new(0, 0, 0));

    let mut aliases = FxHashMap::default();
    aliases.insert(
        "latest".into(),
        UnresolvedVersionSpec::Semantic(SemVer(latest.clone())),
    );

    Ok(Json(LoadVersionsOutput {
        versions,
        aliases,
        latest: Some(UnresolvedVersionSpec::Semantic(SemVer(latest))),
        ..Default::default()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_tool() {
        let result = register_tool(Json(RegisterToolInput::default()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_version_files() {
        let result = detect_version_files(());
        assert!(result.is_ok());
        let output = result.unwrap().0;
        assert!(output.files.contains(&".graalvm-version".into()));
        assert!(output.files.contains(&".java-version".into()));
    }

    #[test]
    fn test_parse_version_file_graalvm() {
        let input = ParseVersionFileInput {
            file: ".graalvm-version".into(),
            content: "25.0.0".into(),
        };
        let result = parse_version_file(Json(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_version_file_java() {
        let input = ParseVersionFileInput {
            file: ".java-version".into(),
            content: "21.0.1".into(),
        };
        let result = parse_version_file(Json(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_version_file_with_prefix() {
        let input = ParseVersionFileInput {
            file: ".java-version".into(),
            content: "graalvm-17.0.8".into(),
        };
        let result = parse_version_file(Json(input));
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_tool_name() {
        let result = register_tool(Json(RegisterToolInput::default()));
        assert!(result.is_ok());
        let output = result.unwrap().0;
        assert_eq!(output.name, NAME);
    }

    #[test]
    fn test_locate_executables() {
        let result = locate_executables(Json(LocateExecutablesInput::default()));
        assert!(result.is_ok());
        let output = result.unwrap().0;
        assert!(output.exes.contains_key("java"));
        assert!(output.exes.contains_key("javac"));
    }
}
