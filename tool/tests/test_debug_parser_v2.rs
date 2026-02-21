use std::fs;
use tempfile::TempDir;

/// Test to debug why the JavaScript grammar fails to parse
#[test]
fn test_debug_javascript_parsing() {
    // Download JavaScript grammar
    let temp_dir = TempDir::new().unwrap();
    let grammar_path = temp_dir.path().join("javascript.grammar.js");

    std::process::Command::new("curl")
        .args([
            "-s",
            "-o",
            grammar_path.to_str().unwrap(),
            "https://raw.githubusercontent.com/tree-sitter/tree-sitter-javascript/master/grammar.js"
        ])
        .output()
        .expect("Failed to download");

    let content = fs::read_to_string(&grammar_path).unwrap();

    println!("File size: {} bytes", content.len());

    // Check the regex directly
    let exports_regex =
        regex::Regex::new(r"module\.exports\s*=\s*grammar\s*\(([\s\S]*)\)").unwrap();
    println!("Direct regex match: {}", exports_regex.is_match(&content));

    if let Some(caps) = exports_regex.captures(&content) {
        println!("Captured content length: {}", caps[1].len());
        // Show first and last 100 chars of captured content
        let captured = &caps[1];
        println!(
            "First 100 chars: {}",
            &captured[..100.min(captured.len())].replace('\n', "\\n")
        );
        println!(
            "Last 100 chars: {}",
            &captured[captured.len().saturating_sub(100)..].replace('\n', "\\n")
        );
    }

    // Now try with the actual parser
    use adze_tool::grammar_js::parse_grammar_js_v2;

    match parse_grammar_js_v2(&content) {
        Ok(grammar) => {
            println!("\nParser succeeded!");
            println!("Grammar name: {}", grammar.name);
            println!("Rules count: {}", grammar.rules.len());
        }
        Err(e) => {
            println!("\nParser failed: {}", e);

            // Let's check if it's the regex in the parser
            // Try with a simpler version that doesn't rely on balanced parens
            let simple_regex = regex::Regex::new(r"module\.exports\s*=\s*grammar\s*\(").unwrap();
            println!("Simple regex match: {}", simple_regex.is_match(&content));
        }
    }
}
