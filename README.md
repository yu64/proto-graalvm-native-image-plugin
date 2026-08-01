# GraalVM Native Image proto plugin

A proto WASM plugin that installs **GraalVM Community** and exposes only its
`native-image` executable. It deliberately does not expose `java`, `javac`, or
`jar`.

```toml
[tools]
graalvm-native-image = "25.0.3"
```

Only unscoped GraalVM versions are accepted. Java distribution scopes such as
`temurin-25` and `graalvm-community-25` are rejected.

The plugin resolves archives through the Foojay Disco API, restricted to the
`graalvm_community` distribution and supported archive formats. On activation,
it sets `GRAALVM_HOME`; it does not modify `JAVA_HOME`.

## Development

```powershell
cargo test
cargo build --target wasm32-wasip1 --release
```
