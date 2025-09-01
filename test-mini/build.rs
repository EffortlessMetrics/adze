use std::path::Path;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(rust_sitter_unsafe_attrs)");
    // Enable pure-rust parser generation
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
    }
    eprintln!(
        "RUST_SITTER_USE_PURE_RUST = {:?}",
        std::env::var("RUST_SITTER_USE_PURE_RUST")
    );
    rust_sitter_tool::build_parsers(Path::new("src/lib.rs"));
}
