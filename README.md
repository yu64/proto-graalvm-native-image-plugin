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

### Independent GraalVM Native Image

```bash
# Specify version explicitly
proto install graalvm-native-image 25.0.1

# Activate and run
proto use graalvm-native-image
native-image --version
```

### Integrated with Java Plugin

If the `java` plugin is configured with a GraalVM distribution, this plugin can use that GraalVM instead of downloading a separate version.

#### Configuration

```toml
[tools.java]
# Specify a GraalVM distribution
distribution = "graalvm-community"

[tools]
java = "25.0.1"

# Option 1: Use bundled GraalVM from java plugin
graalvm-native-image = "bundled"

# Option 2: Match java version (use java's GraalVM)
graalvm-native-image = "25.0.1"

# Option 3: Use different version (download separately)
graalvm-native-image = "24.0.1"
```

#### Behavior

- **`bundled` version**: Automatically resolves to the GraalVM version managed by the `java` plugin. If java doesn't have GraalVM, falls back to the latest version
- **Same version as java**: Uses the GraalVM from the `java` plugin (no separate download needed)
- **Different version**: Downloads a separate GraalVM Community instance
- **java without GraalVM**: Downloads GraalVM independently

## Development

```powershell
cargo test
cargo build --target wasm32-wasip1 --release
```
