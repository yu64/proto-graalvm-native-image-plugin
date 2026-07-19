#[cfg(feature = "wasm")]
mod graalvm_api;

#[cfg(feature = "wasm")]
mod proto;

#[cfg(feature = "wasm")]
pub use proto::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
