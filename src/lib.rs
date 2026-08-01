//! A proto tool plugin that exposes only GraalVM Community Native Image.

mod graalvm;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod proto;
mod version;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use proto::*;
