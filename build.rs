// libxkbcommon is linked at build time (statically for musl targets: with
// `-static`, the linker resolves `-lxkbcommon` to `libxkbcommon.a`, which the
// release build images provide). Here we only help dynamic builds find the
// library, by asking pkg-config for its libdir; pkg-config being absent or
// having no xkbcommon .pc file is fine, the default search paths are tried.
fn main() {
    // Convention with Cross.toml's pre-build: the cross musl images fetch
    // Alpine's libxkbcommon-static into /musl-deps. A missing directory is
    // harmless; for local musl builds, pass -L native=<dir> via RUSTFLAGS.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("musl") {
        println!("cargo:rustc-link-search=native=/musl-deps");
    }
    if let Ok(output) = std::process::Command::new("pkg-config")
        .args(["--variable=libdir", "xkbcommon"])
        .output()
    {
        if output.status.success() {
            let dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !dir.is_empty() {
                println!("cargo:rustc-link-search=native={dir}");
            }
        }
    }
}
