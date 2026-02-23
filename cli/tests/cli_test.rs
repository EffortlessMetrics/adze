use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Adze CLI - Tools for grammar development",
        ));
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("adze");

    cmd.arg("init")
        .arg("test-grammar")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Creating new grammar project"));

    // Check that files were created
    assert!(temp_dir.path().join("test-grammar").exists());
    assert!(temp_dir.path().join("test-grammar/Cargo.toml").exists());
    assert!(temp_dir.path().join("test-grammar/src/grammar.rs").exists());
}

#[test]
#[ignore = "Check command needs OUT_DIR environment variable - requires CLI to set temp OUT_DIR"]
fn test_check_command() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("test.rs");

    // Write a valid grammar (using a complete pattern)
    let grammar = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Test {
                #[adze::leaf(text = "test")]
                _test: (),
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    "#;

    fs::write(&grammar_file, grammar).unwrap();

    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("check")
        .arg(&grammar_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Grammar syntax is valid"));
}

#[test]
fn test_stats_command() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("test.rs");

    let grammar = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Test {
                #[adze::leaf(text = "test")]
                _test: (),
                #[adze::repeat]
                items: Vec<Item>,
            }
            
            #[adze::language]
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    "#;

    fs::write(&grammar_file, grammar).unwrap();

    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("stats")
        .arg(&grammar_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Rules: 2"))
        .stdout(predicate::str::contains("Leaf rules: 2"))
        .stdout(predicate::str::contains("Repeat rules: 1"));
}

#[test]
fn test_doc_command() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("test.rs");

    let grammar = r#"
        #[adze::grammar("test")]
        mod grammar {
            /// This is a test grammar
            /// It demonstrates documentation
            #[adze::language]
            pub struct Test {
                /// A test field
                #[adze::leaf(text = "test")]
                _test: (),
            }
        }
    "#;

    fs::write(&grammar_file, grammar).unwrap();

    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("doc")
        .arg(&grammar_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("This is a test grammar"))
        .stdout(predicate::str::contains("It demonstrates documentation"));
}

// Tests for dynamic loading functionality
#[cfg(feature = "dynamic")]
mod dynamic_tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_library_path() -> Option<PathBuf> {
        // Look for a test tree-sitter library in common locations
        let possible_paths = vec![
            "/usr/lib/libtree-sitter-json.so",
            "/usr/local/lib/libtree-sitter-json.so",
            "/opt/homebrew/lib/libtree-sitter-json.so", // macOS
            "libtree-sitter-json.so",
            "tree-sitter-json.dll", // Windows
        ];

        for path_str in possible_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }

        // Also check if we have any test libraries in target directory
        let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
        let debug_dir = PathBuf::from(target_dir).join("debug");

        if let Ok(entries) = fs::read_dir(&debug_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        if name_str.contains("tree_sitter")
                            && (name_str.ends_with(".so")
                                || name_str.ends_with(".dll")
                                || name_str.ends_with(".dylib"))
                        {
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    #[test]
    fn test_parse_dynamic_missing_library() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.txt");
        fs::write(&input_file, "test input").unwrap();

        let nonexistent_lib = temp_dir.path().join("nonexistent.so");

        let mut cmd = cargo_bin_cmd!("adze");
        cmd.arg("parse")
            .arg("--dynamic")
            .arg(&nonexistent_lib)
            .arg(&input_file)
            .arg("--symbol")
            .arg("tree_sitter_test")
            .assert()
            .failure()
            .stderr(predicate::str::contains("dynamic grammar not found"));
    }

    #[test]
    fn test_parse_dynamic_invalid_symbol() {
        // Skip this test if no test library is available
        let lib_path = match get_test_library_path() {
            Some(path) => path,
            None => {
                eprintln!("Skipping dynamic symbol test - no test library found");
                return;
            }
        };

        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.txt");
        fs::write(&input_file, r#"{"key": "value"}"#).unwrap();

        let mut cmd = cargo_bin_cmd!("adze");
        cmd.arg("parse")
            .arg("--dynamic")
            .arg(&lib_path)
            .arg(&input_file)
            .arg("--symbol")
            .arg("nonexistent_symbol")
            .assert()
            .failure(); // Should fail due to missing symbol
    }

    #[test]
    fn test_parse_dynamic_json_output() {
        let lib_path = match get_test_library_path() {
            Some(path) => path,
            None => {
                eprintln!("Skipping dynamic JSON output test - no test library found");
                return;
            }
        };

        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.json");
        fs::write(&input_file, r#"{"test": true}"#).unwrap();

        let mut cmd = cargo_bin_cmd!("adze");
        cmd.arg("parse")
            .arg("--dynamic")
            .arg(&lib_path)
            .arg(&input_file)
            .arg("--symbol")
            .arg("tree_sitter_json") // Common symbol name
            .arg("--format")
            .arg("json")
            .assert()
            .success()
            .stdout(predicate::str::contains(r#""status":"#));
    }

    #[test]
    fn test_parse_dynamic_empty_input() {
        let lib_path = match get_test_library_path() {
            Some(path) => path,
            None => {
                eprintln!("Skipping dynamic empty input test - no test library found");
                return;
            }
        };

        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("empty.txt");
        fs::write(&input_file, "").unwrap();

        let mut cmd = cargo_bin_cmd!("adze");
        cmd.arg("parse")
            .arg("--dynamic")
            .arg(&lib_path)
            .arg(&input_file)
            .arg("--symbol")
            .arg("tree_sitter_json")
            .assert()
            .success()
            .stdout(predicate::str::contains("Input size: 0 bytes"));
    }

    #[test]
    fn test_parse_dynamic_large_input() {
        let lib_path = match get_test_library_path() {
            Some(path) => path,
            None => {
                eprintln!("Skipping dynamic large input test - no test library found");
                return;
            }
        };

        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("large.json");

        // Create a reasonably large JSON file
        let mut large_json = String::from("{\n");
        for i in 0..100 {
            large_json.push_str(&format!(r#"  "key_{i}": "value_{i}","#));
            large_json.push('\n');
        }
        large_json.push_str(r#"  "final": "value""#);
        large_json.push_str("\n}");

        fs::write(&input_file, &large_json).unwrap();

        let mut cmd = cargo_bin_cmd!("adze");
        cmd.arg("parse")
            .arg("--dynamic")
            .arg(&lib_path)
            .arg(&input_file)
            .arg("--symbol")
            .arg("tree_sitter_json")
            .timeout(std::time::Duration::from_secs(10)) // Reasonable timeout
            .assert()
            .success()
            .stdout(predicate::str::contains("Input size:"));
    }

    #[test]
    fn test_parse_dynamic_malformed_input() {
        let lib_path = match get_test_library_path() {
            Some(path) => path,
            None => {
                eprintln!("Skipping dynamic malformed input test - no test library found");
                return;
            }
        };

        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("malformed.json");
        fs::write(&input_file, r#"{"incomplete": json,,,}"#).unwrap();

        let mut cmd = cargo_bin_cmd!("adze");
        cmd.arg("parse")
            .arg("--dynamic")
            .arg(&lib_path)
            .arg(&input_file)
            .arg("--symbol")
            .arg("tree_sitter_json")
            .assert()
            .success() // Should succeed but may report errors
            .stdout(predicate::str::contains("Input size:"));
    }

    #[test]
    fn test_parse_dynamic_unicode_input() {
        let lib_path = match get_test_library_path() {
            Some(path) => path,
            None => {
                eprintln!("Skipping dynamic unicode input test - no test library found");
                return;
            }
        };

        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("unicode.json");
        fs::write(
            &input_file,
            r#"{"emoji": "🦀🚀", "chinese": "你好", "arabic": "مرحبا"}"#,
        )
        .unwrap();

        let mut cmd = cargo_bin_cmd!("adze");
        cmd.arg("parse")
            .arg("--dynamic")
            .arg(&lib_path)
            .arg(&input_file)
            .arg("--symbol")
            .arg("tree_sitter_json")
            .assert()
            .success()
            .stdout(predicate::str::contains("Input size:"));
    }

    #[test]
    fn test_parse_dynamic_verbose_output() {
        let lib_path = match get_test_library_path() {
            Some(path) => path,
            None => {
                eprintln!("Skipping dynamic verbose output test - no test library found");
                return;
            }
        };

        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("test.json");
        fs::write(&input_file, r#"{"simple": "test"}"#).unwrap();

        let mut cmd = cargo_bin_cmd!("adze");
        cmd.arg("--verbose")
            .arg("parse")
            .arg("--dynamic")
            .arg(&lib_path)
            .arg(&input_file)
            .arg("--symbol")
            .arg("tree_sitter_json")
            .assert()
            .success()
            .stdout(predicate::str::contains("Loading dynamic grammar:"))
            .stdout(predicate::str::contains("Loaded language from:"));
    }
}

#[test]
fn test_parse_static_missing_input() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_grammar = temp_dir.path().join("nonexistent_grammar.rs");
    let nonexistent_input = temp_dir.path().join("nonexistent_input.txt");

    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("parse")
        .arg(&nonexistent_grammar)
        .arg(&nonexistent_input)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No such file or directory")
                .or(predicate::str::contains("cannot find the file")) // Windows
                .or(predicate::str::contains("No static grammars enabled")), // Expected for static grammar builds
        );
}

#[test]
fn test_parse_missing_grammar_arg() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("input.txt");
    fs::write(&input_file, "test content").unwrap();

    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("parse")
        .arg(&input_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ));
}
