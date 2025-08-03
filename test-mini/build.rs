use std::path::Path;

fn main() {
    eprintln!("RUST_SITTER_USE_PURE_RUST = {:?}", std::env::var("RUST_SITTER_USE_PURE_RUST"));
    rust_sitter_tool::build_parsers(Path::new("src/lib.rs"));
}