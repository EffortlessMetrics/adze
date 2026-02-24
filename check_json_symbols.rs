use tree_sitter_json;

fn main() {
    let language = tree_sitter_json::LANGUAGE;
    let symbol_count = language.symbol_count();
    for i in 0..symbol_count {
        println!("{}: {}", i, language.symbol_name(i as u16));
    }
}
