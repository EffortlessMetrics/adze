use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/scanner.rs");

    // Build the parser
    // Enable pure-rust parser generation
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
    }
    rust_sitter_tool::build_parsers(&PathBuf::from("src/lib.rs"));

    // Register the external scanner
    println!("cargo:rustc-env=RUST_SITTER_EXTERNAL_SCANNER=python");
}
