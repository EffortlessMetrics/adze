use std::path::PathBuf;

fn main() {
    adze_tool::build_parsers(&PathBuf::from("src/main.rs"));
}
