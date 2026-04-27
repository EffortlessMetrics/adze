use std::path::Path;

fn main() {
    adze_tool::build_parsers(Path::new("src/lib.rs"));
}
