use std::fs;
use tempfile::TempDir;

/// Test to understand JavaScript grammar structure
#[test]
fn test_js_grammar_structure() {
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

    // Look at the end of the file
    let lines: Vec<&str> = content.lines().collect();
    println!("Total lines: {}", lines.len());

    // Show last 20 lines
    println!("\nLast 20 lines:");
    for line in lines.iter().rev().take(20).rev() {
        println!("{}", line);
    }

    // Check for the module.exports pattern with more detail
    let exports_regex = regex::Regex::new(r"module\.exports\s*=\s*grammar\s*\(").unwrap();
    if let Some(mat) = exports_regex.find(&content) {
        println!("\nFound module.exports at byte {}", mat.start());

        // Count parentheses to find the matching closing paren
        let start = mat.end();
        let chars: Vec<char> = content[start..].chars().collect();
        let mut depth = 1;
        let mut pos = 0;

        while depth > 0 && pos < chars.len() {
            match chars[pos] {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            pos += 1;
        }

        if depth == 0 {
            println!("Matching ) found at offset {} from start", pos);
            println!("That's byte {} in the file", start + pos);

            // Check what comes after
            let after_close = &content[start + pos..];
            println!(
                "\nAfter closing paren: {:?}",
                &after_close[..20.min(after_close.len())]
            );
        } else {
            println!("No matching ) found!");
        }
    } else {
        println!("\nNo module.exports pattern found!");
    }
}
