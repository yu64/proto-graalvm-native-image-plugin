use proto_pdk_test_utils::*;

mod graalvm_plugin {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn registers_metadata() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let metadata = plugin.register_tool(RegisterToolInput::default()).await;

        assert_eq!(metadata.name, "GraalVM");
        assert_eq!(metadata.type_of, PluginType::Language);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn has_minimum_proto_version() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let metadata = plugin.register_tool(RegisterToolInput::default()).await;

        assert!(
            metadata.minimum_proto_version.is_some(),
            "Should have minimum proto version"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn has_plugin_version() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let metadata = plugin.register_tool(RegisterToolInput::default()).await;

        assert!(
            metadata.plugin_version.is_some(),
            "Should have plugin version"
        );
    }
}
