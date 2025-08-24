#[cfg(test)]
mod debug_tests {
    use crate::{language, parse};

    fn print_tree(node: &rust_sitter::pure_parser::ParsedNode, depth: usize, source: &[u8]) {
        let indent = "  ".repeat(depth);
        let text =
            std::str::from_utf8(&source[node.start_byte..node.end_byte]).unwrap_or("<invalid>");
        eprintln!(
            "{}Node: kind='{}', symbol={}, text='{}', children={}",
            indent,
            node.kind(),
            node.symbol,
            text,
            node.children.len()
        );
        for child in &node.children {
            print_tree(child, depth + 1, source);
        }
    }

    #[test]
    fn debug_parse_tree() {
        let source = "42";
        let mut parser = rust_sitter::tree_sitter::Parser::new();
        parser.set_language(language()).unwrap();

        let parse_result = parser.parse_bytes_with_tree(source.as_bytes(), None);
        eprintln!("\n=== Full parse tree for '42' ===");
        if let Some(root) = &parse_result.root {
            print_tree(root, 0, source.as_bytes());
        } else {
            eprintln!("No root node in parse result!");
        }

        // Now test extraction
        eprintln!("\n=== Testing extraction ===");
        let result = parse("42");
        match result {
            Ok(module) => {
                eprintln!("Parse succeeded! Module body length: {}", module.body.len());
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }
}
