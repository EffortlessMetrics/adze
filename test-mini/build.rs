use std::path::Path;

fn main() {
    rust_sitter_tool::build_parsers(Path::new("src/lib.rs"));
}