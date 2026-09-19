//! A proto tool plugin that exposes only GraalVM Community Native Image.

// #############################################################################
// MARK: Modules

// Pure helpers are compiled for unit tests; exported hooks require a WASM host.
#[cfg(any(test, all(feature = "wasm", target_arch = "wasm32")))]
mod graalvm;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod proto;

#[cfg(any(test, all(feature = "wasm", target_arch = "wasm32")))]
mod version;

// #############################################################################
// MARK: WASM exports

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use proto::*;
