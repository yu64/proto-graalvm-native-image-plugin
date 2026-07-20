use proto_pdk_test_utils::*;

mod graalvm_plugin {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn locate_java_and_javac_executables() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .locate_executables(LocateExecutablesInput::default())
            .await;

        assert!(output.exes.contains_key("java"));
        assert!(output.exes.contains_key("javac"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn java_is_primary_executable() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .locate_executables(LocateExecutablesInput::default())
            .await;

        let java_config = output.exes.get("java").expect("java executable not found");
        assert!(java_config.primary, "java should be the primary executable");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn javac_is_not_primary() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .locate_executables(LocateExecutablesInput::default())
            .await;

        let javac_config = output.exes.get("javac").expect("javac executable not found");
        assert!(!javac_config.primary, "javac should not be the primary executable");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn has_executable_directories() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .locate_executables(LocateExecutablesInput::default())
            .await;

        assert!(!output.exes_dirs.is_empty(), "Should have executable directories");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn register_tool_metadata() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .register_tool(RegisterToolInput::default())
            .await;

        assert_eq!(output.name, "GraalVM");
        assert_eq!(output.type_of, PluginType::Language);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn detect_version_files() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin.detect_version_files(()).await;

        assert!(
            output.files.contains(&".graalvm-version".into()),
            "Should detect .graalvm-version files"
        );
        assert!(
            output.files.contains(&".java-version".into()),
            "Should detect .java-version files"
        );
    }
}
