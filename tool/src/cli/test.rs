use anyhow::Result;
use glob::glob;
use std::fs;
use std::path::Path;

/// Run parser tests on a corpus
pub fn run_tests(
    corpus_path: Option<&Path>,
    parser_path: Option<&Path>,
    filter: Option<&str>,
    _update: bool,
    _show_stats: bool,
) -> Result<()> {
    // Check if corpus exists
    let corpus = corpus_path.unwrap_or(Path::new("corpus"));

    if parser_path.is_some() {
        eprintln!("adze CLI v0.6.0 - Test Command");
        eprintln!("=====================================");
        eprintln!();
        eprintln!("STATUS: Corpus testing with external parsers is not yet implemented.");
        eprintln!();
        eprintln!("CURRENT LIMITATIONS:");
        eprintln!("  - Cannot run corpus tests with dynamically loaded parsers");
        eprintln!("  - Test runner cannot compile and execute parser crates");
        eprintln!("  - Corpus validation requires integrated parsers");
        eprintln!();
        eprintln!("HOW TO TEST PARSERS TODAY:");
        eprintln!();
        eprintln!("1. Write integration tests in your parser crate:");
        eprintln!("   ```rust");
        eprintln!("   #[test]");
        eprintln!("   fn test_parsing() {{");
        eprintln!("       let tree = parse(\"let x = 42\");");
        eprintln!("       assert!(tree.is_ok());");
        eprintln!("   }}");
        eprintln!("   ```");
        eprintln!();
        eprintln!("2. Use the insta crate for snapshot testing:");
        eprintln!("   ```rust");
        eprintln!("   use insta::assert_snapshot;");
        eprintln!("   assert_snapshot!(format!(\"{{:#?}}\", tree));");
        eprintln!("   ```");
        eprintln!();
        eprintln!("3. Run tests with cargo:");
        eprintln!("   ```bash");
        eprintln!("   cargo test");
        eprintln!("   cargo insta review  # to update snapshots");
        eprintln!("   ```");
        eprintln!();
        eprintln!("COMING SOON (v0.6.x):");
        eprintln!("  - Tree-sitter compatible corpus testing");
        eprintln!("  - Automatic test generation from corpus files");
        eprintln!();
        eprintln!("For updates, see: https://github.com/EffortlessMetrics/adze");

        std::process::exit(64); // EX_USAGE
    }

    if !corpus.exists() {
        eprintln!("adze test - Corpus directory not found");
        eprintln!("=============================================");
        eprintln!();
        eprintln!("Looking for corpus at: {:?}", corpus);
        eprintln!();
        eprintln!("The corpus directory should contain test files in Tree-sitter format:");
        eprintln!();
        eprintln!("corpus/");
        eprintln!("  expressions.txt");
        eprintln!("  statements.txt");
        eprintln!("  ...");
        eprintln!();
        eprintln!("Test file format:");
        eprintln!("==================");
        eprintln!("=== Test name");
        eprintln!("source code here");
        eprintln!("---");
        eprintln!("(expected");
        eprintln!("  (parse");
        eprintln!("    (tree)))");
        eprintln!();
        eprintln!("Note: Full corpus testing is not yet implemented.");
        eprintln!("      Currently, only test file validation is performed.");

        std::process::exit(1);
    }

    // Basic corpus validation only (no actual parsing)
    println!("adze test - Validating corpus format");
    println!("===========================================");
    println!();
    println!("Checking corpus at: {:?}", corpus);
    println!();

    // Find test files
    let pattern = corpus.join("**/*.txt");
    let test_files: Vec<_> = glob(pattern.to_str().unwrap())?
        .filter_map(Result::ok)
        .filter(|path| {
            if let Some(filter) = filter {
                path.to_string_lossy().contains(filter)
            } else {
                true
            }
        })
        .collect();

    if test_files.is_empty() {
        println!("No test files found in corpus.");
        println!();
        println!("Expected .txt files with Tree-sitter test format.");
        return Ok(());
    }

    println!("Found {} test files", test_files.len());
    println!();

    // Validate test format only
    let mut valid = 0;
    let mut invalid = 0;

    for test_file in &test_files {
        let content = fs::read_to_string(test_file)?;
        let test_cases = parse_test_format(&content)?;

        if test_cases.is_empty() {
            println!("  ✗ {} - no test cases found", test_file.display());
            invalid += 1;
        } else {
            println!(
                "  ✓ {} - {} test cases",
                test_file.display(),
                test_cases.len()
            );
            valid += 1;
        }
    }

    println!();
    println!("Validation Results:");
    println!("  Valid files:   {}", valid);
    println!("  Invalid files: {}", invalid);
    println!();
    println!("Note: This only validates the test file format.");
    println!("      Actual parsing tests are not yet implemented.");
    println!("      Use integration tests in your parser crate instead.");

    Ok(())
}

/// Parse Tree-sitter test format
fn parse_test_format(content: &str) -> Result<Vec<TestCase>> {
    let mut test_cases = Vec::new();
    let mut current_test = TestCase::default();
    let mut in_source = false;
    let mut in_expected = false;

    for line in content.lines() {
        if line.starts_with("===") {
            // Test separator
            if !current_test.name.is_empty() {
                test_cases.push(current_test);
                current_test = TestCase::default();
            }
            current_test.name = line.trim_start_matches('=').trim().to_string();
            in_source = true;
            in_expected = false;
        } else if line.starts_with("---") {
            // Expected output separator
            in_source = false;
            in_expected = true;
        } else if in_source {
            current_test.source.push_str(line);
            current_test.source.push('\n');
        } else if in_expected {
            current_test.expected.push_str(line);
            current_test.expected.push('\n');
        }
    }

    if !current_test.name.is_empty() {
        test_cases.push(current_test);
    }

    Ok(test_cases)
}

#[derive(Debug, Default)]
struct TestCase {
    name: String,
    source: String,
    expected: String,
}
