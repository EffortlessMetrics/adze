fn main() {
    let grammar = std::path::PathBuf::from("src/lib.rs");
    adze_tool::build_parsers(&grammar);
}
