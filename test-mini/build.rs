use std::path::Path;

fn main() {
    // Enable pure-rust parser generation
    std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
    eprintln!("RUST_SITTER_USE_PURE_RUST = {:?}", std::env::var("RUST_SITTER_USE_PURE_RUST"));
    rust_sitter_tool::build_parsers(Path::new("src/lib.rs"));
}