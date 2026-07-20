use proto_pdk_test_utils::*;

mod graalvm_plugin {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn activate_environment_sets_java_home() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let context = ToolContext {
            tool_dir: VirtualPath::from("/home/user/.proto/tools/graalvm/25.0.0"),
            ..Default::default()
        };

        let output = plugin
            .activate_environment(ActivateEnvironmentInput {
                context,
                ..Default::default()
            })
            .await;

        assert!(
            output.env.contains_key("JAVA_HOME"),
            "Should set JAVA_HOME environment variable"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn java_home_contains_tool_path() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let tool_path = VirtualPath::from("/home/user/.proto/tools/graalvm/25.0.0");
        let context = ToolContext {
            tool_dir: tool_path.clone(),
            ..Default::default()
        };

        let output = plugin
            .activate_environment(ActivateEnvironmentInput {
                context,
                ..Default::default()
            })
            .await;

        if let Some(java_home) = output.env.get("JAVA_HOME") {
            assert!(
                java_home.contains("graalvm/25.0.0"),
                "JAVA_HOME should contain tool path"
            );
        } else {
            panic!("JAVA_HOME not set");
        }
    }
}
