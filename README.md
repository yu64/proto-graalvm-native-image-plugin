# Proto GraalVM CE Plugin

A [moonrepo proto](https://moonrepo.dev/proto) WASM plugin for managing **GraalVM Community Edition** versions across multiple platforms and architectures.

## Features

- 🚀 Fast and lightweight WASM plugin
- 🌍 Multi-platform support (Windows, macOS, Linux)
- 🏗️ Multi-architecture support (x64, ARM64)
- 📦 Automatic version resolution
- 🔍 Intelligent release pattern detection
- 🎯 Support for multiple GraalVM CE release formats

## Supported Platforms

| OS | Architecture | Status |
|---|---|---|
| Windows | x64 | ✅ |
| Windows | ARM64 | ✅ |
| macOS | x64 | ✅ |
| macOS | ARM64 | ✅ |
| Linux | x64 | ✅ |
| Linux | ARM64 | ✅ |

## Installation

### Using proto

```bash
proto install graalvm-ce
```

### From GitHub Releases

1. Download the latest `.wasm` file from [GitHub Releases](https://github.com/yu64/proto-graalvm-plugin/releases)
2. Add to `.prototools`:

```toml
[plugins]
graalvm-ce = "https://github.com/yu64/proto-graalvm-plugin/releases/download/v0.1.0/proto_graalvm_plugin.wasm"
```

## Usage

### Install a specific version

```bash
proto install graalvm-ce 25.0.3
```

### Use latest version

```bash
proto install graalvm-ce latest
```

### Activate in shell

```bash
proto activate graalvm-ce 25.0.3
java -version
```

### Version detection

Create a `.java-version` or `.graalvm-version` file:

```
25.0.3
```

Or in `package.json`:

```json
{
  "engines": {
    "graalvm": ">=25.0.0"
  }
}
```

## Version Format

The plugin automatically parses GraalVM CE release versions and supports the following formats:

- `latest` - Latest available version
- `25.0.3` - Specific version
- `25.0` - Latest patch of 25.0
- `25` - Latest version of 25.x

## Development

### Prerequisites

- Rust 1.70+
- `wasm32-wasip1` target
- proto

### Setup

```bash
# Install Rust and WASM target
rustup install stable
rustup target add wasm32-wasip1

# Clone and setup
git clone https://github.com/yu64/proto-graalvm-plugin.git
cd proto-graalvm-plugin

# Build WASM plugin
cargo build --target wasm32-wasip1
```

### Testing

```bash
# Run unit tests
cargo test

# Test with proto (from repo root)
proto --log trace install graalvm-ce 25.0.3
proto --log trace list-remote graalvm-ce
```

For more details, see [DEVELOPMENT.md](./DEVELOPMENT.md).

## Architecture

```
src/
├── lib.rs              # Plugin entry point & proto functions
├── release_parser.rs   # GitHub Releases API & pattern parsing
├── url_builder.rs      # Download URL construction
└── version_resolver.rs # Version alias resolution
```

### Key Components

- **release_parser.rs**: Handles GitHub Releases API communication and parses complex filename patterns to extract version, OS, and architecture information
- **url_builder.rs**: Constructs proper download URLs accounting for different tag naming conventions
- **version_resolver.rs**: Resolves version aliases and partial versions to concrete versions
- **lib.rs**: Implements proto plugin functions (register, download, locate, etc.)

## Challenges & Solutions

### 1. Inconsistent Release Patterns

GraalVM CE changed its release tag format over time:

- Old: `jdk-23.0.0` → `graalvm-community-jdk-23.0.0_windows-x64_bin.zip`
- Mid: `jdk-25.0.2` → `graalvm-community-jdk-25.0.2_windows-x64_bin.zip`
- New: `graal-25.1.3` → `graalvm-community-jdk-25i1-25.0.3_windows-x64_bin.zip`

**Solution**: Multiple parsing patterns with fallback logic in `parse_version_from_tag()` and `parse_asset()`.

### 2. Complex Asset Naming

Different OS/Arch combinations use different naming schemes:

```
graalvm-community-jdk-{version}_{os}-{arch}_bin.{ext}
```

**Solution**: Regex-based parsing with explicit OS/Arch mapping in `parse_os_arch()`.

### 3. Archive Structure Variations

Different versions have different internal directory names:

- v23.0.0: `graalvm-ce-java17-23.0.0`
- v25.0.3: `graalvm-jdk-25.0.3`

**Solution**: Version-based prefix detection in `determine_archive_prefix()`.

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or PR on GitHub.

## Related

- [moonrepo Proto Documentation](https://moonrepo.dev/proto)
- [GraalVM CE Releases](https://github.com/graalvm/graalvm-ce-builds)
- [Official Proto Plugins](https://github.com/moonrepo/plugins)

