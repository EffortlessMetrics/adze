use std::fs;
use tempfile::TempDir;

/// Test to debug brace balancing in JavaScript grammar
#[test]
fn test_debug_brace_balance() {
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

    // Find module.exports
    let exports_regex = regex::Regex::new(r"module\.exports\s*=\s*grammar\s*\(").unwrap();
    if let Some(mat) = exports_regex.find(&content) {
        println!("Found module.exports at position {}", mat.start());

        // Try to find the matching closing paren
        let start = mat.end();
        let mut depth = 1;
        let mut pos = 0;
        let chars: Vec<char> = content[start..].chars().collect();
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_regex = false;
        let mut escape_next = false;

        println!("Starting brace/paren balance check from position {}", start);

        while depth > 0 && pos < chars.len() {
            let ch = chars[pos];

            if escape_next {
                escape_next = false;
            } else if ch == '\\' {
                escape_next = true;
            } else if !in_regex && !in_string && (ch == '\'' || ch == '"' || ch == '`') {
                in_string = true;
                string_char = ch;
                println!("  [{}] Entering string with {}", start + pos, ch);
            } else if in_string && ch == string_char && !escape_next {
                in_string = false;
                println!("  [{}] Exiting string", start + pos);
            } else if !in_string && !in_regex && ch == '/' && pos > 0 {
                // Check context for regex
                let prev = if pos > 0 { chars[pos - 1] } else { ' ' };
                let next = if pos + 1 < chars.len() {
                    chars[pos + 1]
                } else {
                    ' '
                };

                if (prev.is_whitespace() || "[,({:;=".contains(prev)) && next != '/' && next != '*'
                {
                    in_regex = true;
                    println!("  [{}] Entering regex", start + pos);
                }
            } else if in_regex && ch == '/' && !escape_next {
                in_regex = false;
                println!("  [{}] Exiting regex", start + pos);
            } else if !in_string && !in_regex {
                match ch {
                    '(' => {
                        depth += 1;
                        println!("  [{}] Found '(' - depth now {}", start + pos, depth);
                    }
                    ')' => {
                        depth -= 1;
                        println!("  [{}] Found ')' - depth now {}", start + pos, depth);
                        if depth == 0 {
                            println!("Found matching paren at position {}", start + pos);
                            break;
                        }
                    }
                    '{' => println!("  [{}] Found '{{' (ignored in paren matching)", start + pos),
                    '}' => println!("  [{}] Found '}}' (ignored in paren matching)", start + pos),
                    _ => {}
                }
            }

            pos += 1;
        }

        if depth > 0 {
            println!("ERROR: Unbalanced parentheses! Depth remaining: {}", depth);
            println!("Position reached: {}", start + pos);

            // Show context around where we stopped
            let context_start = (start + pos).saturating_sub(100);
            let context_end = (start + pos + 100).min(content.len());
            println!("\nContext around position {}:", start + pos);
            println!("{}", &content[context_start..context_end]);
        }
    }
}
