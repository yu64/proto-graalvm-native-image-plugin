use extism_pdk::*;
use proto_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GraalVMConfig {
    pub version: String,
}

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: String) -> String;
}

/// Register the tool with the proto plugin
#[plugin_fn]
pub fn register_tool(Json(_): Json<ToolMetadataInput>) -> FnResult<Json<ToolMetadata>> {
    Ok(Json(ToolMetadata {
        name: "graalvm".into(),
        type_of: ToolType::Jvm,
        plugin_version: Some(env!("CARGO_PKG_VERSION").into()),
        ..Default::default()
    }))
}

/// Download a prebuilt binary
#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    // Placeholder implementation
    Ok(Json(DownloadPrebuiltOutput {
        download_url: format!(
            "https://github.com/oracle/graalvm-ce-builds/releases/download/vm-{}/graalvm-ce-java11-{}-{}.tar.gz",
            input.version, input.version, input.host_env.os
        ),
        ..Default::default()
    }))
}

/// Unpack an archive
#[plugin_fn]
pub fn unpack_archive(
    Json(_input): Json<UnpackArchiveInput>,
) -> FnResult<Json<UnpackArchiveOutput>> {
    Ok(Json(UnpackArchiveOutput::default()))
}

/// Locate executables
#[plugin_fn]
pub fn locate_executables(
    Json(_input): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    Ok(Json(LocateExecutablesOutput {
        bin_path: Some("bin".into()),
        executables: vec!["java".into(), "javac".into()],
        ..Default::default()
    }))
}
