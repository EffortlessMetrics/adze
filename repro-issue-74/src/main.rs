#[adze::grammar("repro")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug)]
    pub struct Program(
        #[adze::leaf(pattern = r"\d+", transform = |s| {
            println!("TRANSFORM CALLED WITH: {:?}", s);
            format!("TRANSFORMED_{}", s)
        })]
        pub String,
    );
}

fn main() {
    println!("--- AUTOMATIC PARSE ---");
    let result = grammar::parse("42");
    match result {
        Ok(p) => {
            println!("Parsed AST: {:?}", p);
        }
        Err(e) => {
            println!("Parse failed: {:?}", e);
        }
    }

    println!("\n--- MANUAL PARSE ---");
    use adze::pure_parser::{ParsedNode, Parser};
    let mut parser = Parser::new();
    parser.set_language(grammar::LANGUAGE_REF).unwrap();
    let res = parser.parse_string("42");
    if let Some(root) = res.root {
        fn dump(node: &ParsedNode, indent: usize, source: &[u8]) {
            let text = &source[node.start_byte..node.end_byte];
            println!(
                "{}{} symbol={}, range={}..{}, text={:?}",
                "  ".repeat(indent),
                node.kind(),
                node.symbol,
                node.start_byte,
                node.end_byte,
                std::str::from_utf8(text).unwrap()
            );
            for child in &node.children {
                dump(child, indent + 1, source);
            }
        }
        dump(&root, 0, "42".as_bytes());
    } else {
        println!("Manual parse failed: {:?}", res.errors);
    }
}
