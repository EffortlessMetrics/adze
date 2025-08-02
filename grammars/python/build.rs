use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/scanner.rs");

    // Build the parser
    rust_sitter_tool::build_parsers(&PathBuf::from("src/lib.rs"));

    // Register the external scanner
    println!("cargo:rustc-env=RUST_SITTER_EXTERNAL_SCANNER=python");
}
