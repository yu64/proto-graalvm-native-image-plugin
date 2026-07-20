use proto_pdk_test_utils::*;

mod graalvm_plugin {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn load_versions_succeeds() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin.load_versions(LoadVersionsInput::default()).await;

        assert!(!output.versions.is_empty(), "Should return some versions");
        assert!(output.latest.is_some(), "Should have a latest version");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parse_graalvm_version_file() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: "25.0.0".into(),
                file: ".graalvm-version".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parse_java_version_file() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: "21.0.1".into(),
                file: ".java-version".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parse_java_version_with_prefix() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        // Test with graalvm prefix
        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: "graalvm-17.0.8".into(),
                file: ".java-version".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_some());
        assert_eq!(output.version.unwrap().to_string(), "17.0.8");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parse_version_file_with_whitespace() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: "  25.1.0  \n".into(),
                file: ".graalvm-version".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_some());
        assert_eq!(output.version.unwrap().to_string(), "25.1.0");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parse_empty_version_file() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: "".into(),
                file: ".graalvm-version".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ignore_non_version_files() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("graalvm-test").await;

        let output = plugin
            .parse_version_file(ParseVersionFileInput {
                content: "25.0.0".into(),
                file: ".some-other-file".into(),
                ..Default::default()
            })
            .await;

        assert!(output.version.is_none());
    }
}
