use crate::graalvm_api;
use extism_pdk::*;
use proto_pdk::*;
use rustc_hash::FxHashMap;

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: "GraalVM".into(),
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
        if !content.is_empty() {
            version = Some(UnresolvedVersionSpec::parse(content)?);
        }
    }

    Ok(Json(ParseVersionFileOutput { version }))
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
    let download_url = graalvm_api::find_asset(&releases, &version_str, &env)
        .ok_or(PluginError::Message(format!(
            "No asset found for version {} on {}/{}",
            version_str, env.os, env.arch
        )))?;

    Ok(Json(DownloadPrebuiltOutput {
        download_url,
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;
    let mut exes: FxHashMap<String, ExecutableConfig> = FxHashMap::default();

    // GraalVM binaries location depends on OS
    let bin_dir = match env.os {
        HostOS::MacOS => "Contents/Home/bin",
        _ => "bin",
    };

    exes.insert("java".into(), ExecutableConfig::new_primary(format!("{}/java", bin_dir)));
    exes.insert("javac".into(), ExecutableConfig::new(format!("{}/javac", bin_dir)));

    let exes_dirs = vec![bin_dir.into()];

    Ok(Json(LocateExecutablesOutput {
        exes_dirs,
        exes,
        ..Default::default()
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

    Ok(Json(LoadVersionsOutput {
        versions,
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
}

