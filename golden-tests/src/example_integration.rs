// This file shows how the golden tests will integrate with rust-sitter grammars
// once they are available. This is not functional yet but shows the pattern.

#![allow(dead_code)]

// TODO: Uncomment when grammars are properly integrated
/*
#[cfg(any(feature = "python-grammar", feature = "javascript-grammar"))]
mod example_integration {
    use rust_sitter::Parse;

    // Example of what the Python parser integration would look like
    #[cfg(feature = "python-grammar")]
    pub fn parse_python_to_sexp(source: &str) -> anyhow::Result<String> {
        // This would use the generated Python parser
        let parsed = rust_sitter_python::parse(source)?;

        // Convert to S-expression format
        Ok(tree_to_sexp(&parsed))
    }

    // Example of what the JavaScript parser integration would look like
    #[cfg(feature = "javascript-grammar")]
    pub fn parse_javascript_to_sexp(source: &str) -> anyhow::Result<String> {
        // This would use the generated JavaScript parser
        let parsed = rust_sitter_javascript::parse(source)?;

        // Convert to S-expression format
        Ok(tree_to_sexp(&parsed))
    }

    // Convert parse tree to S-expression (matching Tree-sitter's format)
    fn tree_to_sexp<T: rust_sitter::Node>(tree: &T) -> String {
        fn node_to_sexp<N: rust_sitter::Node>(node: &N, source: &str, indent: usize) -> String {
            let mut result = String::new();
            let spaces = " ".repeat(indent);

            if node.is_named() {
                result.push_str(&format!("{}({}", spaces, node.kind()));

                if node.child_count() == 0 {
                    // Leaf node - include text
                    let text = &source[node.byte_range()];
                    result.push_str(&format!(" \"{}\")", escape_string(text)));
                } else {
                    // Internal node - recurse on children
                    result.push('\n');

                    for i in 0..node.child_count() {
                        if let Some(child) = node.child(i) {
                            result.push_str(&node_to_sexp(&child, source, indent + 2));
                            result.push('\n');
                        }
                    }

                    result.push_str(&format!("{})", spaces));
                }
            } else {
                // Anonymous node - just include the text
                let text = &source[node.byte_range()];
                result.push_str(&format!("{}\"{}\"", spaces, escape_string(text)));
            }

            result
        }

        node_to_sexp(tree.root_node(), tree.source(), 0)
    }

    fn escape_string(s: &str) -> String {
        s.chars()
            .flat_map(|c| match c {
                '"' => vec!['\\', '"'],
                '\\' => vec!['\\', '\\'],
                '\n' => vec!['\\', 'n'],
                '\r' => vec!['\\', 'r'],
                '\t' => vec!['\\', 't'],
                c if c.is_control() => {
                    format!("\\u{{{:04x}}}", c as u32).chars().collect()
                }
                c => vec![c],
            })
            .collect()
    }
}
*/
