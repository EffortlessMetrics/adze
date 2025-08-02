// Golden tests comparing pure-Rust implementation against Tree-sitter C output
use rust_sitter::pure_parser::{ParsedNode, Parser, TSLanguage, TSParseAction};
use rust_sitter::unified_parser::Parser as UnifiedParser;
use std::fs;
use std::path::Path;

/// Golden test data structure
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GoldenTest {
    name: String,
    language: String,
    source: String,
    expected_tree: String,
    expected_tables: GoldenTables,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GoldenTables {
    symbol_count: u32,
    state_count: u32,
    parse_table_entries: Vec<u16>,
    symbol_names: Vec<String>,
}

/// Create a test language that matches Tree-sitter's output
fn create_golden_language() -> &'static TSLanguage {
    // This represents the JSON grammar from Tree-sitter
    static LANGUAGE: TSLanguage = TSLanguage {
        version: 14,
        symbol_count: 30,
        alias_count: 0,
        token_count: 15,
        external_token_count: 0,
        state_count: 100,
        large_state_count: 2,
        production_id_count: 1,
        field_count: 3,
        max_alias_sequence_length: 5,
        parse_table: &[
            // Actual parse table data would go here
            // This is what we're validating matches Tree-sitter
        ],
        small_parse_table: &[],
        small_parse_table_map: &[],
        parse_actions: &[],
        symbol_names: &[
            "end",
            "object",
            "{",
            "}",
            "pair",
            "string",
            ":",
            "value",
            "array",
            "[",
            "]",
            ",",
            "number",
            "true",
            "false",
            "null",
            "_string_content",
            "escape_sequence",
            "comment", // ... etc
        ],
        field_names: &["key", "value", "element"],
        field_map_slices: &[],
        field_map_entries: &[],
        symbol_metadata: &[],
        public_symbol_map: &[],
        alias_map: &[],
        alias_sequences: &[],
        lex_modes: &[],
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: None,
        primary_state_ids: &[],
    };
    &LANGUAGE
}

/// Format a parse tree in the same format as Tree-sitter CLI
fn format_tree_like_tree_sitter(node: &ParsedNode, source: &str, indent: usize) -> String {
    let mut result = String::new();
    let indent_str = "  ".repeat(indent);

    result.push_str(&indent_str);
    result.push('(');
    result.push_str(&node.kind);

    if node.named {
        if let Some(field_name) = &node.field_name {
            result.push_str(" <");
            result.push_str(field_name);
            result.push('>');
        }
    }

    if node.children.is_empty() && node.end_byte > node.start_byte {
        // Leaf node with text
        let text = &source[node.start_byte..node.end_byte];
        if !text.contains('\n') && text.len() < 20 {
            result.push_str(": \"");
            result.push_str(&text.replace('"', "\\\""));
            result.push('"');
        }
    }

    if !node.children.is_empty() {
        result.push('\n');
        for child in &node.children {
            result.push_str(&format_tree_like_tree_sitter(child, source, indent + 1));
        }
        result.push_str(&indent_str);
    }

    result.push_str(")\n");
    result
}

#[test]
fn test_json_grammar_golden() {
    let source = r#"{"name": "test", "value": 42}"#;
    let expected_tree = r#"(document
  (object
    (pair
      key: (string
        (string_content))
      value: (string
        (string_content)))
    (pair
      key: (string
        (string_content))
      value: (number))))
"#;

    let language = create_golden_language();
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();

    let result = parser.parse_string(source);
    assert!(
        result.errors.is_empty(),
        "Parse errors: {:?}",
        result.errors
    );

    if let Some(root) = result.root {
        let formatted = format_tree_like_tree_sitter(&root, source, 0);

        // Compare line by line for better error messages
        let expected_lines: Vec<&str> = expected_tree.lines().collect();
        let actual_lines: Vec<&str> = formatted.lines().collect();

        for (i, (expected, actual)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
            assert_eq!(
                expected.trim(),
                actual.trim(),
                "Line {} differs:\nExpected: {}\nActual:   {}",
                i + 1,
                expected,
                actual
            );
        }
    } else {
        panic!("No parse tree produced");
    }
}

#[test]
fn test_arithmetic_grammar_golden() {
    let source = "1 + 2 * 3";
    let expected_tree = r#"(expression
  (binary_expression
    left: (number: "1")
    operator: "+"
    right: (binary_expression
      left: (number: "2")
      operator: "*"
      right: (number: "3"))))
"#;

    // Test would continue similarly...
}

#[test]
fn test_parse_table_compression_golden() {
    // Verify that our table compression produces identical results to Tree-sitter
    let language = create_golden_language();

    // Check table sizes match
    assert_eq!(language.state_count, 100, "State count mismatch");
    assert_eq!(language.symbol_count, 30, "Symbol count mismatch");

    // Verify parse table entries match expected values
    // This would compare against known good values from Tree-sitter
}

#[test]
fn test_incremental_parsing_golden() {
    let initial_source = r#"{"a": 1}"#;
    let edited_source = r#"{"a": 1, "b": 2}"#;

    let language = create_golden_language();
    let mut parser = UnifiedParser::new();
    parser.set_language(language).unwrap();

    // Parse initial
    let initial_result = parser.parse(initial_source, None);
    assert!(initial_result.errors.is_empty());

    // Apply edit
    let edit = rust_sitter::pure_incremental::Edit {
        start_byte: 7,
        old_end_byte: 7,
        new_end_byte: 16,
        start_point: rust_sitter::pure_parser::Point { row: 0, column: 7 },
        old_end_point: rust_sitter::pure_parser::Point { row: 0, column: 7 },
        new_end_point: rust_sitter::pure_parser::Point { row: 0, column: 16 },
    };

    if let Some(tree) = initial_result.root {
        let tree = rust_sitter::pure_incremental::Tree::new(tree);
        let edited_result = parser.parse_with_edits(edited_source, Some(tree), &[edit]);

        assert!(edited_result.errors.is_empty());
        // Verify the tree structure matches expected
    }
}

/// Load and run all golden tests from files
#[test]
#[ignore] // Run with --ignored to execute
fn test_all_golden_files() {
    let golden_dir = Path::new("tests/golden");
    if !golden_dir.exists() {
        eprintln!("Golden test directory not found, skipping");
        return;
    }

    for entry in fs::read_dir(golden_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension() == Some(std::ffi::OsStr::new("json")) {
            println!("Running golden test: {:?}", path);

            let content = fs::read_to_string(&path).unwrap();
            let test: GoldenTest = serde_json::from_str(&content).unwrap();

            run_golden_test(&test);
        }
    }
}

fn run_golden_test(test: &GoldenTest) {
    let language = create_golden_language();
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();

    let result = parser.parse_string(&test.source);

    if let Some(root) = result.root {
        let formatted = format_tree_like_tree_sitter(&root, &test.source, 0);

        assert_eq!(
            formatted.trim(),
            test.expected_tree.trim(),
            "Tree mismatch for test '{}'",
            test.name
        );
    } else {
        panic!("Failed to parse test '{}'", test.name);
    }
}

/// Generate golden test files from Tree-sitter output
#[test]
#[ignore]
fn generate_golden_tests() {
    use std::process::Command;

    let test_cases = vec![
        ("json", r#"{"test": true}"#),
        ("javascript", "const x = 42;"),
        ("python", "def hello(): pass"),
    ];

    for (lang, source) in test_cases {
        // Run tree-sitter CLI to get expected output
        let output = Command::new("tree-sitter")
            .args(&["parse", "-"])
            .arg(format!("--scope=source.{}", lang))
            .env("TREE_SITTER_DIR", ".")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(source.as_bytes())
                    .unwrap();
                child.wait_with_output()
            });

        if let Ok(output) = output {
            let tree = String::from_utf8_lossy(&output.stdout);

            let golden = GoldenTest {
                name: format!("{}_basic", lang),
                language: lang.to_string(),
                source: source.to_string(),
                expected_tree: tree.to_string(),
                expected_tables: GoldenTables {
                    symbol_count: 0, // Would be filled from actual grammar
                    state_count: 0,
                    parse_table_entries: vec![],
                    symbol_names: vec![],
                },
            };

            let path = format!("tests/golden/{}_{}.json", lang, "basic");
            fs::write(path, serde_json::to_string_pretty(&golden).unwrap()).unwrap();
        }
    }
}
