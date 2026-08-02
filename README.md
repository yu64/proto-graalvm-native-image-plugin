# GraalVM Native Image proto plugin

A proto WASM plugin that installs **GraalVM Community** and exposes only its
`native-image` executable. It deliberately does not expose `java`, `javac`, or
`jar`.

## Installation

Register the plugin by adding the following to your `.prototools` file:

```toml
[plugins]
graalvm-native-image = "github://yu64/proto-graalvm-plugin"

[tools]
graalvm-native-image = "25.0.3"
```

Only unscoped GraalVM versions are accepted. Java distribution scopes such as
`temurin-25` and `graalvm-community-25` are rejected.

The plugin resolves archives through the Foojay Disco API, restricted to the
`graalvm_community` distribution and supported archive formats. On activation,
it sets `GRAALVM_HOME`; it does not modify `JAVA_HOME`.

## Usage

### With Java Plugin (Recommended)

When using the `java` plugin with a GraalVM distribution, specify `graalvm-native-image` with the same version or `"bundled"`. The native-image binary will be installed directly to java's bin directory automatically.

```toml
[plugins.tools]
graalvm-native-image = 'github://yu64/proto-graalvm-plugin'

[tools]
java = "graalvm-community-25.0.1"
graalvm-native-image = "bundled"
# OR
# graalvm-native-image = "25.0.1"  # Same version as java
```

Both java and native-image will use the same GraalVM installation with no disk duplication.

Use native-image:
```bash
native-image --version
```

### Independent GraalVM Native Image

For standalone GraalVM Community installations:

```toml
[tools]
graalvm-native-image = "25.0.1"
```

Use native-image:
```bash
proto use graalvm-native-image
native-image --version
```

## Development

```powershell
cargo test
cargo build --target wasm32-wasip1 --release
```
