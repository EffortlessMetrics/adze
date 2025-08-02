use std::fs;
use tempfile::TempDir;

/// Test to understand string context issue
#[test]
fn test_string_context() {
    // Download JavaScript grammar
    let temp_dir = TempDir::new().unwrap();
    let grammar_path = temp_dir.path().join("javascript.grammar.js");

    println!("Downloading JavaScript grammar from tree-sitter repository...");
    let grammar_content = std::process::Command::new("curl")
        .arg("-s")
        .arg("https://raw.githubusercontent.com/tree-sitter/tree-sitter-javascript/master/grammar.js")
        .output()
        .expect("Failed to download grammar");

    fs::write(&grammar_path, &grammar_content.stdout).unwrap();
    let content = String::from_utf8(grammar_content.stdout).unwrap();

    // Find module.exports and extract grammar content
    let exports_regex = regex::Regex::new(r"module\.exports\s*=\s*grammar\s*\(").unwrap();
    if let Some(mat) = exports_regex.find(&content) {
        let start = mat.end();

        // Look for rules: section
        let grammar_section = &content[start..];
        if let Some(rules_pos) = grammar_section.find("rules:") {
            let after_rules = &grammar_section[rules_pos + 6..];
            let trimmed = after_rules.trim_start();

            if trimmed.starts_with('{') {
                let content_after_brace = &trimmed[1..];
                let chars: Vec<char> = content_after_brace.chars().collect();

                // Look around position 5000
                println!("Context around position 5000:");
                for i in 4990..5010.min(chars.len()) {
                    println!("  [{}] char='{}' (U+{:04X})", i, chars[i], chars[i] as u32);
                }

                // Find all quote characters
                println!("\nQuote characters found:");
                let mut quote_count = 0;
                for (i, ch) in chars.iter().enumerate() {
                    if *ch == '\'' || *ch == '"' || *ch == '`' {
                        if quote_count < 20 {
                            let context_start = i.saturating_sub(10);
                            let context_end = (i + 10).min(chars.len());
                            let context: String =
                                chars[context_start..context_end].iter().collect();
                            println!(
                                "  [{}] {} in context: ...{}...",
                                i,
                                ch,
                                context.replace('\n', "\\n")
                            );
                        }
                        quote_count += 1;
                    }
                }
                println!("Total quote characters: {}", quote_count);
            }
        }
    }
}
