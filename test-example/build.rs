use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/main.rs");
    adze_tool::build_parsers(&PathBuf::from("src/main.rs"));
}
