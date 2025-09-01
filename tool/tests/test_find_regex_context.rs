#![allow(clippy::needless_range_loop)]

use std::fs;
use tempfile::TempDir;

/// Test to find the context around the problematic regex
#[test]
fn test_find_regex_context() {
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

    // Find the problematic regex
    let pattern = r"/[\s\p{Zs}\uFEFF\u2028\u2029\u2060\u200B";

    if let Some(idx) = content.find(pattern) {
        println!("Found problematic pattern at index {}", idx);

        // Show context before and after
        let start = idx.saturating_sub(200);
        let end = (idx + 200).min(content.len());

        let context = &content[start..end];
        println!("\nContext:\n{}", context);

        // Look for the complete regex
        let regex_start = idx;
        let mut regex_end = idx + pattern.len();
        let chars: Vec<char> = content.chars().collect();
        let mut char_idx = 0;
        let mut byte_idx = 0;

        // Find char index for regex start
        for (i, ch) in chars.iter().enumerate() {
            if byte_idx >= regex_start {
                char_idx = i;
                break;
            }
            byte_idx += ch.len_utf8();
        }

        // Find the closing /
        let mut escaped = false;
        for i in (char_idx + pattern.chars().count())..chars.len() {
            if escaped {
                escaped = false;
            } else if chars[i] == '\\' {
                escaped = true;
            } else if chars[i] == '/' {
                // Found the end
                regex_end = byte_idx;
                break;
            }
            byte_idx += chars[i].len_utf8();
        }

        let full_regex = &content[regex_start..regex_end + 1];
        println!("\nFull regex: {}", full_regex);
    } else {
        println!("Pattern not found!");
    }
}
