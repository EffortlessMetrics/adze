use std::fs;
use tempfile::TempDir;

/// Test to understand rules extraction issue
#[test]
fn test_rules_extraction() {
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
        
        // Use the same paren matching logic as in the parser
        let mut depth = 1;
        let mut pos = 0;
        let chars: Vec<char> = content[start..].chars().collect();
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_regex = false;
        let mut escape_next = false;
        
        while depth > 0 && pos < chars.len() {
            let ch = chars[pos];
            
            if escape_next {
                escape_next = false;
            } else if ch == '\\' {
                escape_next = true;
            } else if !in_regex && !in_string && (ch == '\'' || ch == '"' || ch == '`') {
                in_string = true;
                string_char = ch;
            } else if in_string && ch == string_char && !escape_next {
                in_string = false;
            } else if !in_string && !in_regex && ch == '/' && pos > 0 {
                let prev = if pos > 0 { chars[pos - 1] } else { ' ' };
                let next = if pos + 1 < chars.len() { chars[pos + 1] } else { ' ' };
                
                if (prev.is_whitespace() || "[,({:;=".contains(prev)) && next != '/' && next != '*' {
                    in_regex = true;
                }
            } else if in_regex && ch == '/' && !escape_next {
                in_regex = false;
            } else if !in_string && !in_regex {
                match ch {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
            }
            
            pos += 1;
        }
        
        if depth == 0 {
            let grammar_content = &content[start..start + pos - 1];
            println!("Successfully extracted grammar content ({} chars)", grammar_content.len());
            
            // Now look for rules:
            if let Some(rules_pos) = grammar_content.find("rules:") {
                println!("Found 'rules:' at position {}", rules_pos);
                
                let after_rules = &grammar_content[rules_pos + 6..];
                let trimmed = after_rules.trim_start();
                
                println!("After 'rules:' trimmed, first 50 chars: {}", 
                         &trimmed[..50.min(trimmed.len())].replace('\n', "\\n"));
                
                if trimmed.starts_with('{') {
                    println!("Found opening brace after rules:");
                    
                    // Now let's check what happens with brace matching
                    let content_after_brace = &trimmed[1..];
                    println!("Content after opening brace (first 100 chars):");
                    println!("{}", &content_after_brace[..100.min(content_after_brace.len())].replace('\n', "\\n"));
                    
                    // Try to find some rule definitions
                    let rule_regex = regex::Regex::new(r#"(\w+):\s*\$\s*=>"#).unwrap();
                    let mut count = 0;
                    for mat in rule_regex.find_iter(content_after_brace) {
                        if count < 5 {
                            println!("Found rule definition: {}", mat.as_str());
                        }
                        count += 1;
                    }
                    println!("Total rule definitions found: {}", count);
                }
            }
        }
    }
}