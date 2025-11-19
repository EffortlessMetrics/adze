use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("rust-sitter").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust-sitter CLI"));
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("rust-sitter").unwrap();

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
        #[rust_sitter::grammar("test")]
        mod grammar {
            #[rust_sitter::language]
            pub struct Test {
                #[rust_sitter::leaf(text = "test")]
                _test: (),
                #[rust_sitter::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    "#;

    fs::write(&grammar_file, grammar).unwrap();

    let mut cmd = Command::cargo_bin("rust-sitter").unwrap();
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
        #[rust_sitter::grammar("test")]
        mod grammar {
            #[rust_sitter::language]
            pub struct Test {
                #[rust_sitter::leaf(text = "test")]
                _test: (),
                #[rust_sitter::repeat]
                items: Vec<Item>,
            }
            
            #[rust_sitter::language]
            pub struct Item {
                #[rust_sitter::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    "#;

    fs::write(&grammar_file, grammar).unwrap();

    let mut cmd = Command::cargo_bin("rust-sitter").unwrap();
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
        #[rust_sitter::grammar("test")]
        mod grammar {
            /// This is a test grammar
            /// It demonstrates documentation
            #[rust_sitter::language]
            pub struct Test {
                /// A test field
                #[rust_sitter::leaf(text = "test")]
                _test: (),
            }
        }
    "#;

    fs::write(&grammar_file, grammar).unwrap();

    let mut cmd = Command::cargo_bin("rust-sitter").unwrap();
    cmd.arg("doc")
        .arg(&grammar_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("This is a test grammar"))
        .stdout(predicate::str::contains("It demonstrates documentation"));
}
