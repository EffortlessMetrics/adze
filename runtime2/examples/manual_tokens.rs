use rust_sitter_runtime::{Language, Parser, Token};

fn main() {
    // TEMP: no parse table yet; Parser::parse will error on the driver.
    // But we can at least validate the tokenizer hookup and error path.
    let lang = Language::new_stub()
        .with_static_tokens(vec![
            Token { kind: 1, start: 0, end: 1 },
            Token { kind: 2, start: 1, end: 2 },
        ]);

    let mut p = Parser::new();
    p.set_language(lang.clone());
    match p.parse("ab", None) {
        Ok(tree) => println!("Parsed: {:?}", tree.root_kind()),
        Err(e) => eprintln!("As expected (no tables yet): {e}"),
    }
}