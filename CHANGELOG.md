# Changelog

## 0.2.0

- Require proto 0.60.0 or newer. For proto 0.59.x, use plugin 0.1.1.
- Update `proto_pdk` to 0.34 and `proto_pdk_api` to 0.33 for the
  [proto 0.60 WASM API changes](https://moonrepo.dev/blog/proto-v0.60).
- Convert `GRAALVM_HOME` from a virtual path to a host path with the new
  `VirtualPathExt::to_real_path` API.
- Update the integration test's default proto version to 0.60.0 and document
  how to pin the older plugin for proto 0.59.x.
