use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    // Enable pure-rust parser generation
    std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
    rust_sitter_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
