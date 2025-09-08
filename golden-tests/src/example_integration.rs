// This file shows how the golden tests integrate with rust-sitter grammars

#[cfg(any(feature = "python-grammar", feature = "javascript-grammar"))]
use rust_sitter::pure_parser::ParsedNode;

#[cfg(feature = "python-grammar")]
pub fn parse_python_to_sexp(source: &str) -> anyhow::Result<String> {
    // Register the external scanner required by the Python grammar
    rust_sitter_python::register_scanner();
    let parsed = rust_sitter_python::parse(source).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(tree_to_sexp(&parsed, source))
}

#[cfg(feature = "javascript-grammar")]
pub fn parse_javascript_to_sexp(source: &str) -> anyhow::Result<String> {
    let parsed =
        rust_sitter_javascript::parse(source).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(tree_to_sexp(&parsed, source))
}

#[cfg(any(feature = "python-grammar", feature = "javascript-grammar"))]
fn tree_to_sexp(node: &ParsedNode, source: &str) -> String {
    fn node_to_sexp(node: &ParsedNode, source: &str, indent: usize) -> String {
        let mut result = String::new();
        let spaces = " ".repeat(indent);

        if node.is_named() {
            result.push_str(&format!("{}({}", spaces, node.kind()));

            if node.child_count() == 0 {
                let text = node.utf8_text(source.as_bytes()).unwrap_or("");
                result.push_str(&format!(" \"{}\")", escape_string(text)));
            } else {
                result.push('\n');
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        result.push_str(&node_to_sexp(child, source, indent + 2));
                        result.push('\n');
                    }
                }
                result.push_str(&format!("{})", spaces));
            }
        } else {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            result.push_str(&format!("{}\"{}\"", spaces, escape_string(text)));
        }

        result
    }

    node_to_sexp(node, source, 0)
}

#[cfg(any(feature = "python-grammar", feature = "javascript-grammar"))]
fn escape_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            '\r' => vec!['\\', 'r'],
            '\t' => vec!['\\', 't'],
            c if c.is_control() => format!("\\u{{{:04x}}}", c as u32).chars().collect(),
            c => vec![c],
        })
        .collect()
}
