use std::path::PathBuf;

fn main() {
    // Tell rustc this cfg is intentional so it doesn't warn
    println!("cargo::rustc-check-cfg=cfg(adze_unsafe_attrs)");
    println!("cargo:rerun-if-changed=src/lib.rs");
    // Enable pure-rust parser generation
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("ADZE_USE_PURE_RUST", "1");
    }
    adze_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
