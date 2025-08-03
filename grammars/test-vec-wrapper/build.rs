use std::path::Path;

fn main() {
    // Use pure Rust parser generation
    std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
    rust_sitter_tool::build_parsers(Path::new("src/lib.rs"));
}