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

When using the `java` plugin with a GraalVM distribution, specify `graalvm-native-image` with the same version or `"bundled"`. The native-image binary will be found in java's directory automatically.

```toml
[plugins.tools]
graalvm-native-image = 'github://yu64/proto-graalvm-plugin'

[tools]
java = "graalvm-community-25.0.1"

# Option 1: Use "bundled" alias
graalvm-native-image = "bundled"

# Option 2: Use same version as java
# graalvm-native-image = "25.0.1"
```

**Behavior:**
- `resolve_version` resolves "bundled" to java's GraalVM version (extracted from PROTO_JAVA_VERSION)
- `download_prebuilt` detects version match with java and **skips download** entirely
- `locate_executables` finds native-image in java's JAVA_HOME/bin directory
- **Single JDK installation** - no disk duplication

### Independent GraalVM Native Image

For standalone GraalVM Community installations (different version than java):

```toml
[tools]
java = "temurin-21"  # No GraalVM
graalvm-native-image = "25.0.1"  # Separate GraalVM
```

In this case:
- JDK is downloaded separately
- native-image found in graalvm-native-image's directory

## Development

```powershell
cargo test
cargo build --target wasm32-wasip1 --release
```
