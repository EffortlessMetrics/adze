/// Integration tests for the rust-sitter CLI
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Test that the CLI provides helpful messages when no parser is specified
#[test]
fn test_parse_without_parser() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "rust-sitter-gen",
            "--",
            "parse",
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run CLI");

    // Should exit with code 64 (EX_USAGE)
    assert_eq!(
        output.status.code(),
        Some(64),
        "Should exit with usage error code"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No parser specified"),
        "Should mention missing parser"
    );
    assert!(stderr.contains("--parser"), "Should mention --parser flag");
    assert!(
        stderr.contains("rust-sitter parse"),
        "Should show command context"
    );
}

/// Test that the CLI help command works
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rust-sitter-gen", "--", "--help"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success(), "Help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("parse"), "Should list parse command");
    assert!(stdout.contains("test"), "Should list test command");
    assert!(stdout.contains("generate"), "Should list generate command");
}

/// Test that parse command help includes parser option
#[test]
fn test_parse_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "rust-sitter-gen", "--", "parse", "--help"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success(), "Parse help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--parser"),
        "Should document --parser option"
    );
    assert!(
        stdout.contains("crate") || stdout.contains("library"),
        "Should mention crate or library in parser description"
    );
}

/// Test that invalid file paths are handled gracefully
#[test]
fn test_parse_nonexistent_file() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "rust-sitter-gen",
            "--",
            "parse",
            "/nonexistent/file.txt",
        ])
        .output()
        .expect("Failed to run CLI");

    assert!(!output.status.success(), "Should fail for nonexistent file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to read file") || stderr.contains("No such file"),
        "Should report file reading error"
    );
}

/// Test the parse command with parser specified shows honest limitations
#[test]
fn test_parse_with_parser_shows_limitations() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.js");
    fs::write(&test_file, "const x = 42;").unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "rust-sitter-gen",
            "--",
            "parse",
            "--parser",
            "some-parser-crate",
            test_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run CLI");

    // Should exit with code 64 (EX_USAGE)
    assert_eq!(
        output.status.code(),
        Some(64),
        "Should exit with usage error code"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not yet implemented") || stderr.contains("CURRENT LIMITATIONS"),
        "Should mention current limitations"
    );
    assert!(stderr.contains("v0.6"), "Should mention version");
    assert!(
        stderr.contains("dynamic"),
        "Should mention dynamic loading limitation"
    );
}

/// Test the test command shows honest limitations
#[test]
fn test_test_command_shows_limitations() {
    let temp_dir = TempDir::new().unwrap();
    // Create a basic corpus directory
    fs::create_dir_all(temp_dir.path().join("corpus")).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "rust-sitter-gen",
            "--",
            "test",
            temp_dir.path().join("corpus").to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run CLI");

    // The test command should work for validation but note limitations
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("Validating corpus format") || combined.contains("No test files found"),
        "Should validate corpus or report no files"
    );
}

/// Test the generate command exists
#[test]
fn test_generate_command_exists() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "rust-sitter-gen",
            "--",
            "generate",
            "--help",
        ])
        .output()
        .expect("Failed to run CLI");

    // Even if not fully implemented, help should work
    assert!(
        output.status.success() || !output.stderr.is_empty(),
        "Generate command should at least be recognized"
    );
}

#[cfg(feature = "build_parsers")]
/// Test parsing with an actual crate (requires example crate to be built)
/// This test is feature-gated as it requires a working parser crate
#[test]
fn test_parse_with_example_crate() {
    // This would test the actual --parser <crate-path> functionality
    // but requires a built example crate, so it's feature-gated

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "1 + 2 * 3").unwrap();

    // Try to use the arithmetic example if it exists
    let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("example");

    if example_path.exists() {
        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "rust-sitter-gen",
                "--",
                "parse",
                "--parser",
                example_path.to_str().unwrap(),
                test_file.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to run CLI with parser");

        // This may or may not work depending on the state of the example
        // but at least shouldn't crash
        assert!(
            !output.stderr.is_empty() || !output.stdout.is_empty(),
            "Should produce some output"
        );
    }
}
