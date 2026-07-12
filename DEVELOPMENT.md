# Development Guide

This guide covers setting up your development environment and working with the proto GraalVM CE plugin.

## Prerequisites

### Required

- **Rust 1.70+** ([Install](https://rustup.rs/))
- **wasm32-wasip1 target**
- **proto** ([Install](https://moonrepo.dev/proto/install))

### Optional but Recommended

- **wasm-pack** - for advanced WASM debugging
- **cargo-watch** - for auto-rebuilding on file changes
- **just** - command runner

## Setup

### 1. Install Rust and WASM target

```bash
# If you haven't installed Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-wasip1
```

### 2. Clone the repository

```bash
git clone https://github.com/yu64/proto-graalvm-plugin.git
cd proto-graalvm-plugin
```

### 3. Verify setup

```bash
# Check Rust version
rustc --version  # Should be 1.70+

# Check WASM target
rustup target list | grep wasm32-wasip1  # Should show "installed"

# Check proto installation
proto --version
```

## Building

### Development build (fast, with debug symbols)

```bash
cargo build --target wasm32-wasip1
```

This creates `target/wasm32-wasip1/debug/proto_graalvm_plugin.wasm` (~2-5 MB).

### Release build (optimized, smaller)

```bash
cargo build --target wasm32-wasip1 --release
```

This creates `target/wasm32-wasip1/release/proto_graalvm_plugin.wasm` (~500 KB).

### Watch and rebuild (with cargo-watch)

```bash
cargo install cargo-watch
cargo watch -x "build --target wasm32-wasip1"
```

## Testing

### Unit tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parse_asset_windows_x64

# Run with output
cargo test -- --nocapture
```

### Integration tests with proto

```bash
# From the repository root (where .prototools is located)

# List available versions
proto --log trace list-remote graalvm-ce

# Install a specific version (uses debug WASM)
proto --log trace install graalvm-ce 25.0.3

# Check installed version
java -version

# View installed tool
ls ~/.proto/tools/graalvm-ce/
```

### Full workflow test

```bash
# 1. Build the plugin
cargo build --target wasm32-wasip1

# 2. Clear any cached version
rm -rf ~/.proto/tools/graalvm-ce

# 3. Test installation
proto --log trace install graalvm-ce latest

# 4. Activate and verify
proto activate graalvm-ce latest
java -version
```

## File Structure

```
proto-graalvm-plugin/
├── Cargo.toml                 # Project manifest
├── Cargo.lock                 # Dependency lockfile
├── src/
│   ├── lib.rs                 # Main plugin entry point
│   ├── release_parser.rs      # GitHub Releases API
│   ├── url_builder.rs         # Download URL generation
│   └── version_resolver.rs    # Version resolution logic
├── tests/                     # Integration tests (future)
├── .prototools                # Proto config for local testing
├── README.md                  # User documentation
├── DEVELOPMENT.md             # This file
├── IMPLEMENTATION_PLAN.md     # Implementation roadmap
└── .github/
    └── workflows/             # CI/CD workflows (future)
```

## Code Style

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

Both commands should pass before committing.

## Common Development Tasks

### Adding a new test case

1. Add test to the relevant module (e.g., `release_parser.rs`)
2. Use the `#[test]` attribute
3. Run `cargo test` to verify

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Arrange
        let input = "example";

        // Act
        let result = some_function(input);

        // Assert
        assert_eq!(result, "expected");
    }
}
```

### Debugging

Use `dbg!()` or `println!()` for debugging:

```rust
dbg!(version);  // Prints debug info
println!("Value: {}", var);  // Prints to stdout
```

When testing with proto, enable logging:

```bash
proto --log debug install graalvm-ce 25.0.3
proto --log trace install graalvm-ce 25.0.3  # Most verbose
```

### Testing specific OS/Arch combinations

The plugin detects your current OS/Arch and downloads accordingly. To test cross-platform:

1. **Manual URL testing**: Extract the URL that would be generated and test manually
2. **Mock releases**: Modify `GITHUB_RELEASES_API` temporarily to point to test data
3. **CI testing**: Use GitHub Actions with matrix builds (see `.github/workflows/`)

## Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update all dependencies
cargo update

# Update specific dependency
cargo update --package proto_pdk
```

## Release Process

### Local testing before release

```bash
# 1. Update version in Cargo.toml
# 2. Build release binary
cargo build --target wasm32-wasip1 --release

# 3. Test thoroughly
proto --log trace install graalvm-ce 25.0.3
java -version

# 4. Verify file size
ls -lh target/wasm32-wasip1/release/proto_graalvm_plugin.wasm
```

### Publishing

1. Tag the commit:
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

2. GitHub Actions automatically builds and publishes to Releases

3. Update proto registry (if registered)

## Troubleshooting

### "No prebuilt found for version X"

- Check if the version exists on [GitHub Releases](https://github.com/graalvm/graalvm-ce-builds/releases)
- Verify `parse_version_from_tag()` correctly parses the tag
- Check `parse_asset()` for filename matching issues
- Enable trace logging: `proto --log trace`

### "wasm32-wasip1 target not found"

```bash
rustup target add wasm32-wasip1
rustup update
```

### WASM file too large (>1 MB)

The release build should be ~500 KB. If larger:

1. Check `Cargo.toml` profile settings
2. Ensure optimizations are enabled:
   ```toml
   [profile.release]
   strip = true
   opt-level = "z"
   lto = true
   ```

3. Use `cargo build-sbom` or `cargo bloat` to analyze size

### Proto can't find the plugin

Ensure `.prototools` points to correct WASM file:

```toml
[plugins]
graalvm-ce = "file://./target/wasm32-wasip1/debug/proto_graalvm_plugin.wasm"
```

The path is relative to `.prototools` location.

## Performance Tips

### Build cache

Cache is stored in `target/`. To clean:

```bash
cargo clean
```

### Incremental compilation

Rust supports incremental compilation (enabled by default). For CI, disable it:

```bash
CARGO_INCREMENTAL=0 cargo build --target wasm32-wasip1 --release
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [proto_pdk Documentation](https://docs.rs/proto_pdk/)
- [WASM/WASI Specification](https://webassembly.org/)
- [moonrepo WASM Guide](https://moonrepo.dev/docs/guides/wasm-plugins)
- [GraalVM CE Documentation](https://www.graalvm.org/latest/docs/)

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/yu64/proto-graalvm-plugin/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yu64/proto-graalvm-plugin/discussions)
- **Proto Support**: [moonrepo Discord](https://discord.gg/qCh9MEynv2)

