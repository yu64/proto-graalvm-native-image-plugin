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

```bash
# Activate the tool and use native-image
proto use graalvm-native-image

# Run native-image command
native-image --version
```

## Development

```powershell
cargo test
cargo build --target wasm32-wasip1 --release
```

## Contributing

This is a personal project and is not open to external contributions. 

*Note: If the official proto (moonrepo) team provides a mechanism to expose tools like `jlink` without redownloading the entire JDK separately from the one managed by the official java plugin, this plugin might evolve into a more proper tool.*

