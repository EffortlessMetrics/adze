use tree_sitter_json;

fn main() {
    let language = tree_sitter_json::LANGUAGE;
    let symbol_count = language.symbol_count();
    let state_count = language.state_count();
    println!("Symbol count: {}", symbol_count);
    println!("State count: {}", state_count);
}
