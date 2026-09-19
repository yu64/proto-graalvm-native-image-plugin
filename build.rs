// #############################################################################
// MARK: Host linker configuration

fn main() {
  println!("cargo:rerun-if-changed=build.rs");

  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
    && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
  {
    // The host DLL is not consumed by native callers. Avoid generating an
    // unused import library/export file and their linker stdout messages.
    println!("cargo:rustc-cdylib-link-arg=/NOIMPLIB");
    println!("cargo:rustc-cdylib-link-arg=/NOEXP");
  }
}
