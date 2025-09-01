use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test that we can identify different grammar.js patterns
#[test]
fn test_grammar_js_patterns() {
    // Test pattern 1: module.exports = grammar(...)
    let pattern1 = r#"
module.exports = grammar({
    name: 'test1',
    rules: {
        source_file: $ => 'hello'
    }
});
"#;

    // Test pattern 2: export default grammar(...)
    let pattern2 = r#"
export default grammar({
    name: 'test2', 
    rules: {
        source_file: $ => 'world'
    }
});
"#;

    // Test pattern 3: const grammar = require(...)
    let pattern3 = r#"
const {grammar} = require('tree-sitter-grammar');

module.exports = grammar({
    name: 'test3',
    rules: {
        source_file: $ => 'foo'
    }
});
"#;

    use rust_sitter_tool::grammar_js::parse_grammar_js_v2;

    println!("Testing pattern 1 (module.exports)...");
    match parse_grammar_js_v2(pattern1) {
        Ok(g) => println!("  Success: parsed grammar '{}'", g.name),
        Err(e) => println!("  Failed: {}", e),
    }

    println!("\nTesting pattern 2 (export default)...");
    match parse_grammar_js_v2(pattern2) {
        Ok(g) => println!("  Success: parsed grammar '{}'", g.name),
        Err(e) => println!("  Failed: {}", e),
    }

    println!("\nTesting pattern 3 (with require)...");
    match parse_grammar_js_v2(pattern3) {
        Ok(g) => println!("  Success: parsed grammar '{}'", g.name),
        Err(e) => println!("  Failed: {}", e),
    }
}

/// Test downloading and examining real grammar files
#[test]
fn test_real_grammar_patterns() {
    let grammars = vec![
        (
            "JSON",
            "https://raw.githubusercontent.com/tree-sitter/tree-sitter-json/master/grammar.js",
        ),
        (
            "JavaScript",
            "https://raw.githubusercontent.com/tree-sitter/tree-sitter-javascript/master/grammar.js",
        ),
        (
            "Python",
            "https://raw.githubusercontent.com/tree-sitter/tree-sitter-python/master/grammar.js",
        ),
    ];

    let temp_dir = TempDir::new().unwrap();
    let grammar_regex =
        regex::Regex::new(r"module\.exports\s*=\s*grammar\s*\(([\s\S]*)\)").unwrap();

    for (name, url) in grammars {
        println!("\nExamining {} grammar...", name);
        let path = temp_dir
            .path()
            .join(format!("{}.grammar.js", name.to_lowercase()));

        let output = Command::new("curl")
            .args(["-s", "-o", path.to_str().unwrap(), url])
            .output()
            .expect("Failed to run curl");

        if !output.status.success() {
            println!(
                "  Failed to download: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            continue;
        }

        if let Ok(content) = fs::read_to_string(&path) {
            println!("  File size: {} bytes", content.len());

            // Check for common patterns
            if content.contains("module.exports") {
                println!("  Uses: module.exports pattern");
                let idx = content.find("module.exports").unwrap();
                let snippet = &content[idx..idx.min(content.len()).min(idx + 100)];
                println!("  Snippet: {}", snippet.replace('\n', " "));

                // Check if it matches the expected pattern
                let exports_regex = &grammar_regex;
                if exports_regex.is_match(&content) {
                    println!("  Regex match: YES");
                } else {
                    println!("  Regex match: NO (this is the problem!)");
                }
            } else if content.contains("export default") {
                println!("  Uses: export default pattern");
            } else {
                println!("  Uses: unknown pattern");
                // Show first 200 chars
                println!(
                    "  First 200 chars: {}",
                    &content[..200.min(content.len())].replace('\n', " ")
                );
            }
        }
    }
}
