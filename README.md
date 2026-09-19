# GraalVM Native Image proto plugin

A proto WASM plugin that installs **GraalVM Community** and exposes only
`native-image`, without adding `java`, `javac`, or `jar` to PATH.
On activation, it sets `GRAALVM_HOME` and leaves `JAVA_HOME` unchanged.

## Install from GitHub Releases

Once a WASM plugin asset is published to
[GitHub Releases](https://github.com/yu64/proto-graalvm-native-image-plugin/releases),
add the following to your project's `.prototools`:

```toml
graalvm-native-image = "25.0.1"

[plugins.tools]
graalvm-native-image = "github://yu64/proto-graalvm-native-image-plugin"
```

This selects the latest release. To pin a plugin release, append `@<release-tag>`
to the locator, replacing `<release-tag>` with a published tag.

Run these commands from the directory containing `.prototools`:

```sh
proto use
proto run graalvm-native-image -- --version
```

## Build and try locally

Tested with proto `0.59.0` and GraalVM Community `25.0.1`.
From the repository root:

```sh
rustup target add wasm32-wasip1
cargo build --locked --target wasm32-wasip1 --release
```

The included `proto_test/.prototools` registers the locally built plugin:

```toml
graalvm-native-image = "25.0.1"

[plugins.tools]
graalvm-native-image = "file://../target/wasm32-wasip1/release/proto_graalvm_native_image_plugin.wasm"
```

Specify the GraalVM version without a distribution prefix such as `temurin-`.
Install and run it from `proto_test`:

```sh
cd proto_test
proto use
proto run graalvm-native-image -- --version
```

## Tests

From the repository root:

```sh
cargo fmt --check
cargo test --locked
```

For an isolated Linux test environment:

```sh
docker build -f proto_test/Dockerfile --build-arg PROTO_VERSION=0.59.0 -t graalvm-plugin-test:0.59.0 .
docker run --rm graalvm-plugin-test:0.59.0
```

Change `PROTO_VERSION` and the image tag to test another proto release.
With Podman, replace `docker` with `podman`.

The container checks installation, `native-image --version`, and shell
activation. It does not test compiling an application to a native executable.

## Contributing

This is a personal project and is not open to external contributions.

*Note: If the official proto (moonrepo) team provides a mechanism to expose tools like `jlink` without redownloading the entire JDK separately from the one managed by the official java plugin, this plugin might evolve into a more proper tool.*
