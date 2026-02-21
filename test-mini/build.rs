use std::path::Path;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(adze_unsafe_attrs)");
    // Enable pure-rust parser generation
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("ADZE_USE_PURE_RUST", "1");
    }
    eprintln!(
        "ADZE_USE_PURE_RUST = {:?}",
        std::env::var("ADZE_USE_PURE_RUST")
    );
    adze_tool::build_parsers(Path::new("src/lib.rs"));
}
