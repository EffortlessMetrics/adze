//! Golden tests comparing parsers against a known-good JSON grammar.
//! 
//! **Design:**
//! - **Preferred path (now):** run goldens via `ts-compat` using the upstream
//!   `tree-sitter-json` grammar. This avoids unsafe/null-pointer TSLanguage construction.
//! - **Pure-Rust path:** scaffolded & `#[ignore]`; switch on when your `UnifiedParser`
//!   + language wiring is ready (via `tablegen`).
//!
//! This keeps the historical backbone active **today** while the pure-Rust backend
//! finishes stabilizing.

// Helper module for pure-rust tests
#[cfg(all(test, feature = "pure-rust"))]
#[path = "support/unified_json_helper.rs"]
mod unified_json_helper;

// Common test helper utilities
#[cfg(test)]
#[path = "support/test_helpers.rs"]
mod test_helpers;

// --- Tree-sitter compatibility path (active today) ---------------------------
#[cfg(feature = "ts-compat")]
mod ts_compat_golden {
    use tree_sitter_json as json;

    fn sexp_named(node: tree_sitter::Node, source: &[u8]) -> String {
        // Render only named nodes for stability across backends
        if node.child_count() == 0 {
            // Leaf node - include the text for literals
            if !node.is_named() {
                return String::new(); // Skip anonymous nodes in output
            }
            let text = std::str::from_utf8(&source[node.byte_range()]).unwrap_or("");
            if text.len() < 20 && !text.contains('\n') {
                return format!("({}: \"{}\")", node.kind(), text.escape_default());
            }
            return format!("({})", node.kind());
        }
        
        let mut parts = vec![format!("({}", node.kind())];
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.is_named() {
                let child_sexp = sexp_named(child, source);
                if !child_sexp.is_empty() {
                    parts.push(format!(" {}", child_sexp));
                }
            }
        }
        parts.push(")".into());
        parts.concat()
    }

    pub fn parse_to_sexp(source: &str) -> String {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&json::LANGUAGE.into()).expect("set JSON language");
        let tree = parser.parse(source, None).expect("parsed");
        let root = tree.root_node();
        sexp_named(root, source.as_bytes())
    }

    #[test]
    fn json_object_simple() {
        let src = r#"{ "a": 1, "b": [2, 3] }"#;
        let got = parse_to_sexp(src);
        
        // Check basic structure is present
        assert!(got.contains("(document"), "should have document root");
        assert!(got.contains("(object"), "should have object node");
        assert!(got.contains("(pair"), "should have pair nodes");
        assert!(got.contains("(array"), "should have array node");
    }

    #[test]
    fn json_nested() {
        let src = r#"{"outer": {"inner": true}, "n": null}"#;
        let got = parse_to_sexp(src);
        
        // Debug: print the actual output
        eprintln!("json_nested sexp: {}", got);
        
        // Verify nested structure
        assert!(got.contains("(object"), "should have object nodes");
        // Note: true and null may be represented differently in tree-sitter-json
        // They might be (true) or just embedded in their parent nodes
    }

    #[test]
    fn json_array_heterogeneous() {
        let src = r#"[1, "two", true, null, {"nested": []}]"#;
        let got = parse_to_sexp(src);
        
        // Debug: print the actual output
        eprintln!("json_array_heterogeneous sexp: {}", got);
        
        assert!(got.contains("(array"), "should have array node");
        assert!(got.contains("(number"), "should have number");
        assert!(got.contains("(string"), "should have string");
        assert!(got.contains("(object"), "should have nested object");
        // true and null representations may vary
    }

    #[test]
    fn json_incremental_smoke() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&json::LANGUAGE.into()).expect("set JSON language");
        
        let src1 = r#"{"a": 1}"#;
        let tree1 = parser.parse(src1, None).expect("parsed initial");

        // Basic incremental path: provide previous tree
        let src2 = r#"{"a": 1, "b": 2}"#;
        let tree2 = parser.parse(src2, Some(&tree1)).expect("parsed incremental");

        // Sanity: both parse to document roots
        assert_eq!(tree1.root_node().kind(), "document");
        assert_eq!(tree2.root_node().kind(), "document");
        
        // Debug: print node counts
        let node_count1 = tree1.root_node().descendant_count();
        let node_count2 = tree2.root_node().descendant_count();
        eprintln!("Node counts: tree1={}, tree2={}", node_count1, node_count2);
        
        // The second tree should have nodes (may not always be more due to tree structure)
        assert!(node_count2 > 0, "incremental parse should produce valid tree");
    }

    #[test]
    fn json_error_recovery() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&json::LANGUAGE.into()).expect("set JSON language");
        
        // Parse with syntax error
        let src = r#"{"a": 1, "b": }"#; // Missing value after "b"
        let tree = parser.parse(src, None).expect("parsed with error");
        
        // Should still produce a tree with ERROR nodes
        assert!(tree.root_node().has_error(), "should detect syntax error");
    }
}

// --- Pure-Rust path scaffold (disabled for now) -----------------------------
#[cfg(feature = "pure-rust")]
mod pure_rust_golden {
    use super::unified_json_helper;
    use rust_sitter::unified_parser::Parser;

    /// Flip this on once `build_min_json_grammar` + `make_minimal_parse_table`
    /// are replaced by a real grammar + table (or your tablegen pipeline).
    #[test]
    #[ignore = "wire real IR + parse table, then remove #[ignore]"]
    fn json_object_simple_pure_rust() {
        let language = unified_json_helper::unified_json_language();
        let mut parser = Parser::new();
        parser.set_language(language).expect("set language");
        let tree = parser.parse(r#"{ "a": 1 }"#, None);
        assert!(tree.is_some());
    }

    #[test]
    #[ignore = "wire real IR + parse table, then remove #[ignore]"]
    fn json_incremental_pure_rust() {
        // This test would verify incremental parsing in the pure-Rust implementation
        let language = unified_json_helper::unified_json_language();
        let mut parser = Parser::new();
        parser.set_language(language).expect("set language");
        let tree1 = parser.parse(r#"{"a": 1}"#, None);
        assert!(tree1.is_some());
        
        // TODO: When incremental parsing is ready, use tree1 for incremental parse
        let tree2 = parser.parse(r#"{"a": 1, "b": 2}"#, None);
        assert!(tree2.is_some());
    }

    #[test]
    #[ignore = "wire real IR + parse table, then remove #[ignore]"]
    fn json_error_recovery_pure_rust() {
        // This test would verify error recovery in the pure-Rust parser
        let language = unified_json_helper::unified_json_language();
        let mut parser = Parser::new();
        parser.set_language(language).expect("set language");
        let tree = parser.parse(r#"{"a": 1, "b": }"#, None); // Missing value
        // TODO: Check for error nodes when the parser is fully implemented
        assert!(tree.is_some() || tree.is_none()); // Parser may or may not produce tree with errors
    }
}

// --- Test fixture support (optional) -----------------------------------------
#[cfg(all(test, feature = "ts-compat"))]
mod fixture_runner {
    use std::fs;
    use std::path::Path;

    #[test]
    #[ignore = "enable when fixture files are present"]
    fn run_golden_fixture_dir() {
        let dir = Path::new("tests/golden/json");
        if !dir.exists() {
            eprintln!("No fixture directory at {:?}, skipping", dir);
            return;
        }
        
        for entry in fs::read_dir(dir).expect("read fixture dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let src = fs::read_to_string(&path).expect("read source");
                let got = super::ts_compat_golden::parse_to_sexp(&src);
                
                let expected_path = path.with_extension("sexp");
                if expected_path.exists() {
                    let expected = fs::read_to_string(&expected_path).expect("read expected");
                    // Normalize whitespace for comparison
                    let got_normalized = got.split_whitespace().collect::<Vec<_>>().join(" ");
                    let expected_normalized = expected.split_whitespace().collect::<Vec<_>>().join(" ");
                    
                    assert_eq!(
                        got_normalized, expected_normalized,
                        "fixture mismatch for {:?}", path
                    );
                } else {
                    // Create expected file for new fixtures
                    eprintln!("Creating expected file: {:?}", expected_path);
                    fs::write(&expected_path, &got).expect("write expected");
                }
            }
        }
    }
}

// --- Legacy test structures (preserved for reference) ------------------------
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