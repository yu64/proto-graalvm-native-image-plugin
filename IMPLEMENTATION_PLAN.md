# GraalVM CE Proto WASM Plugin - 実装計画

## 📋 プロジェクト概要

GraalVM Community Edition の複数バージョン・複数プラットフォームに対応した proto WASM プラグイン。

**課題**: GraalVM CE のリリースパターンが時系列で複数存在

---

## 🔍 GraalVM CE リリースパターン分析

### パターン 1: 古いバージョン（v23.0.0 など）
```
Tag:      jdk-25.0.0
Files:    graalvm-community-jdk-23.0.0_windows-x64_bin.zip
          graalvm-community-jdk-23.0.0_macos-x64_bin.tar.gz
          ...
```

### パターン 2: 中期バージョン（v25.0.1, 25.0.2）
```
Tag:      jdk-25.0.2
Files:    graalvm-community-jdk-25.0.2_windows-x64_bin.zip
          graalvm-community-jdk-25.0.2_macos-aarch64_bin.tar.gz
          ...
```

### パターン 3: 新しいバージョン（v25.0.3+）
```
Tag:      graal-25.1.3
Files:    graalvm-community-jdk-25i1-25.0.3_windows-x64_bin.zip
          graalvm-community-jdk-25i1-25.0.3_macos-aarch64_bin.tar.gz
          graalvm-community-jdk-25i1-25.0.3_linux-aarch64_bin.tar.gz
          graalvm-community-jdk-25i1-25.0.3_linux-x64_bin.tar.gz
```

**プラットフォームマトリクス**:
| OS | Architecture | 対応 |
|---|---|---|
| Windows | x64 | ✓ |
| Windows | ARM64 | ✓ (新パターンのみ) |
| macOS | x64 | ✓ |
| macOS | ARM64 (Apple Silicon) | ✓ |
| Linux | x64 | ✓ |
| Linux | ARM64 | ✓ (新パターンのみ) |

---

## 🏗️ 実装方針

### **推奨: Rust による実装**（moonrepo 標準）

proto WASM プラグインは Rust + `proto_pdk` で実装するのが標準。
- 最小サイズの WASM バイナリ
- moonrepo コミュニティとの互換性
- テストユーティリティが豊富

### **代替: AssemblyScript（TypeScript ライク）**

TypeScript ライクな構文で WASM を書きたい場合：
- AssemblyScript で開発
- WASM に型安全にコンパイル
- 学習曲線が緩い

**推奨**: 最初は **Rust** で実装し、後に TypeScript プラグインは別プロジェクトで検討

---

## 📦 プロジェクト構成

```
proto-graalvm-plugin/
├── Cargo.toml                    # Rust プロジェクト定義
├── Cargo.lock
├── src/
│   ├── lib.rs                    # メインプラグイン
│   ├── release_parser.rs         # GitHub Releases パターン解析
│   ├── version_resolver.rs       # バージョン解決ロジック
│   └── url_builder.rs            # ダウンロード URL 生成
├── tests/
│   ├── download_install.rs       # インストールテスト
│   ├── version_resolution.rs     # バージョン解決テスト
│   └── url_generation.rs         # URL 生成テスト
├── .prototools                   # デバッグ用 proto 設定
├── .github/
│   └── workflows/
│       └── release.yml           # CI/CD: リリース自動化
├── build.rs                      # ビルドスクリプト
├── DEVELOPMENT.md                # 開発手順書
├── IMPLEMENTATION_PLAN.md        # このファイル
└── README.md                     # プラグイン説明

```

---

## 🎯 実装フェーズ

### **Phase 1: プロジェクト初期化** (Day 1)

**タスク**:
- [ ] Cargo.toml 初期化
- [ ] 依存関係設定（proto_pdk, serde, reqwest など）
- [ ] .gitignore, LICENSE, README.md 作成
- [ ] `.prototools` デバッグ設定

**成果物**:
```bash
cargo new --lib proto-graalvm-plugin
# 基本構造と依存関係
```

---

### **Phase 2: コア解析モジュール** (Day 2-3)

**実装項目**:

#### 2.1 GitHub Releases API 連携
```rust
// リリース情報の取得
pub async fn fetch_graalvm_releases() -> Result<Vec<Release>>

// リリース情報の構造体
pub struct Release {
    pub tag: String,           // "jdk-25.0.0", "graal-25.1.3"
    pub version: SemVer,       // 25.0.0, 25.0.3
    pub published_at: DateTime,
    pub assets: Vec<Asset>,
}

pub struct Asset {
    pub name: String,          // "graalvm-community-jdk-..."
    pub download_url: String,  // GitHub release asset URL
}
```

#### 2.2 リリースパターン解析器
```rust
// ファイル名パターンのマッチング
pub fn parse_asset_filename(filename: &str) -> Result<AssetInfo>

pub struct AssetInfo {
    pub version: SemVer,
    pub os: HostOS,
    pub arch: HostArch,
    pub file_type: FileType,   // Zip, TarGz
}
```

**例**:
```
"graalvm-community-jdk-25i1-25.0.3_windows-x64_bin.zip"
→ {
    version: 25.0.3,
    os: Windows,
    arch: X64,
    file_type: Zip
  }
```

#### 2.3 バージョン解決エンジン
```rust
// バージョン候補から最適なものを選択
pub fn resolve_version(
    initial: &str,
    available_versions: &[SemVer],
) -> Result<SemVer>

// サポート例：
// "latest" → 最新版
// "25.0" → 25.0.x の最新
// "25" → 25.x.x の最新
```

---

### **Phase 3: Proto Plugin Functions** (Day 3-4)

**必須実装**:

#### 3.1 `register_tool()`
```rust
#[plugin_fn]
pub fn register_tool(Json(input): Json<RegisterToolInput>) 
    -> FnResult<Json<RegisterToolOutput>>
```

**出力**:
```json
{
  "name": "GraalVM CE",
  "type_of": "Runtime",
  "minimum_proto_version": "0.42.0"
}
```

#### 3.2 `download_prebuilt()` ⭐ 最も複雑
```rust
#[plugin_fn]
pub fn download_prebuilt(Json(input): Json<DownloadPrebuiltInput>)
    -> FnResult<Json<DownloadPrebuiltOutput>>
```

**ロジック**:
1. `input.context.version` を SemVer に解決
2. GitHub Releases API からメタデータ取得
3. OS/Arch に合致するアセットを検索
   - Windows x64: `_windows-x64_bin.zip`
   - macOS ARM64: `_macos-aarch64_bin.tar.gz`
   - Linux x64: `_linux-x64_bin.tar.gz`
4. `DownloadPrebuiltOutput` を返す

**出力例**:
```json
{
  "download_url": "https://github.com/graalvm/graalvm-ce-builds/releases/download/graal-25.1.3/graalvm-community-jdk-25i1-25.0.3_windows-x64_bin.zip",
  "download_name": "graalvm-community-jdk-25i1-25.0.3_windows-x64_bin.zip",
  "archive_prefix": "graalvm-jdk-25.0.3",
  "checksum_url": null  // GraalVM CE は checksum を提供しない
}
```

#### 3.3 `locate_executables()`
```rust
#[plugin_fn]
pub fn locate_executables(Json(_): Json<LocateExecutablesInput>)
    -> FnResult<Json<LocateExecutablesOutput>>
```

**出力**:
```json
{
  "exes": {
    "java": {
      "path": "bin/java",           // Unix
      "windows_path": "bin/java.exe"
    },
    "javac": { "path": "bin/javac" },
    "jshell": { "path": "bin/jshell" },
    // ...
  }
}
```

#### 3.4 `load_versions()`
```rust
#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>)
    -> FnResult<Json<LoadVersionsOutput>>
```

**ロジック**:
1. GitHub Releases API から全リリース取得
2. タグをパースしてバージョンリスト化
3. `latest` エイリアスを設定

#### 3.5 `resolve_version()` （オプション）
バージョンエイリアス対応：
- `lts` → LTS 最新
- `latest` → 絶対最新
- `25` → 25.x.x の最新

#### 3.6 `detect_version_files()`
GraalVM のバージョン検出ファイル：
- `.java-version`
- `.graalvm-version`
- `package.json` (engines.graalvm)

---

### **Phase 4: テスト実装** (Day 4-5)

**テストタイプ**:

#### 4.1 ユニットテスト
```rust
#[cfg(test)]
mod tests {
    // ファイル名パース テスト
    #[test]
    fn test_parse_asset_filename_windows_x64() { }
    
    #[test]
    fn test_parse_asset_filename_macos_aarch64() { }
    
    // バージョン解決テスト
    #[test]
    fn test_resolve_version_latest() { }
    
    // URL 生成テスト
    #[test]
    fn test_download_url_generation() { }
}
```

#### 4.2 統合テスト（マクロ使用）
```rust
// Proto test utils
generate_download_install_tests!("graalvm-ce", "25.0.3");
generate_resolve_versions_tests!("graalvm-ce", {
    "latest" => "25.1.3",
    "25.0" => "25.0.3",
    "lts" => "25.0.0",
});
```

#### 4.3 手動テスト
```bash
# ローカルテスト
cargo build --target wasm32-wasip1

proto --log trace install graalvm-ce
proto --log trace list-remote graalvm-ce
proto activate graalvm-ce 25.0.3
java -version
```

---

### **Phase 5: CI/CD & ドキュメント** (Day 5)

#### 5.1 GitHub Actions ワークフロー
```yaml
# .github/workflows/test.yml
- 自動テスト実行
- WASM ビルド
- リリース時の自動パブリッシュ

# .github/workflows/release.yml
- GitHub Releases へのアップロード
- proto registry への登録
```

#### 5.2 ドキュメント
- `DEVELOPMENT.md` - ローカル開発環境セットアップ
- `TROUBLESHOOTING.md` - よくある問題
- `CONTRIBUTING.md` - プルリクエストガイド

---

## 🔧 技術スタック

| 用途 | ツール | バージョン |
|---|---|---|
| 言語 | Rust | 1.70+ |
| WASM Target | wasm32-wasip1 | - |
| Plugin SDK | proto_pdk | 0.46+ |
| HTTP | reqwest | 0.11+ (async) |
| JSON | serde_json | - |
| 日時 | chrono | - |
| テスト | tokio + proto_pdk_test_utils | - |

---

## 🚀 開発環境セットアップ

```bash
# 必須
rustup install stable
rustup target add wasm32-wasip1

# オプション（デバッグ用）
cargo install wasm-pack
cargo install wasmtime

# proto 自体のインストール
curl -fsSL https://moonrepo.dev/install/proto.sh | bash
```

---

## 📊 複雑さ予想

| コンポーネント | 難易度 | 工数 |
|---|---|---|
| リリースパターン解析 | ⭐⭐⭐ | 4-6h |
| download_prebuilt() | ⭐⭐⭐⭐ | 8-10h |
| バージョン解決 | ⭐⭐ | 2-3h |
| その他プラグイン関数 | ⭐ | 2-3h |
| テスト実装 | ⭐⭐⭐ | 6-8h |
| **合計** | - | **24-32h** |

---

## ⚠️ 既知の課題と対策

| 課題 | 影響度 | 対策 |
|---|---|---|
| リリースタグパターンが複数 | 🔴 高 | 正規表現 + フォールバック |
| GitHub API レート制限 | 🟡 中 | キャッシング、ETag 利用 |
| arm64 の古いバージョン非対応 | 🟢 低 | サポート対象外と明記 |
| Checksum 不提供 | 🟢 低 | 検証スキップ（GitHub 信頼） |
| ダウンロード時間（大きいファイル） | 🟡 中 | プログレス表示、タイムアウト設定 |

---

## 🎬 次のステップ

1. **Phase 1** から順に実装開始
2. 各フェーズの完了後にテスト
3. 定期的に `cargo build --target wasm32-wasip1` でビルド確認
4. proto コマンドでの実行テスト
5. 最終的に GitHub Releases でパブリッシュ

---

## 📚 参考リソース

- [moonrepo Proto WASM Plugin Docs](https://moonrepo.dev/docs/proto/wasm-plugin)
- [proto_pdk Rust docs](https://docs.rs/proto_pdk/)
- [GraalVM CE Releases](https://github.com/graalvm/graalvm-ce-builds/releases)
- [Official proto Plugins](https://github.com/moonrepo/plugins)

