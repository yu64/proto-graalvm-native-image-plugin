//! A proto tool plugin that exposes only GraalVM Community Native Image.

mod graalvm;
#[cfg(feature = "wasm")]
mod proto;
mod version;

#[cfg(feature = "wasm")]
pub use proto::*;
