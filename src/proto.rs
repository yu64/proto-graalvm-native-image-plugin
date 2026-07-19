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
        files: vec![".graalvm-version".into()],
        ignore: vec![],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let mut version = None;

    if input.file == ".graalvm-version" {
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
    let _version = input
        .context
        .version
        .as_version()
        .ok_or(PluginError::Message("Unsupported version type.".into()))?;

    // Placeholder: return empty for now
    Ok(Json(DownloadPrebuiltOutput::default()))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let mut exes: FxHashMap<String, ExecutableConfig> = FxHashMap::default();

    exes.insert("java".into(), ExecutableConfig::new_primary("bin/java"));
    exes.insert("javac".into(), ExecutableConfig::new("bin/javac"));

    Ok(Json(LocateExecutablesOutput {
        exes_dirs: vec!["bin".into()],
        exes,
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn load_versions(_: ()) -> FnResult<Json<LoadVersionsOutput>> {
    Ok(Json(LoadVersionsOutput::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_tool() {
        // Basic test to ensure function compiles
        let result = register_tool(Json(RegisterToolInput::default()));
        assert!(result.is_ok());
    }
}

