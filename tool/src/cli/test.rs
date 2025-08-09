use anyhow::{Context, Result, bail};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Test result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub file: String,
    pub passed: bool,
    pub parse_time_ms: f64,
    pub error: Option<String>,
}

/// Run parser tests on a corpus
pub fn run_tests(
    corpus_path: Option<&Path>,
    parser_path: Option<&Path>,
    filter: Option<&str>,
    _update: bool,
    show_stats: bool,
) -> Result<()> {
    // Default corpus path
    let corpus = corpus_path.unwrap_or(Path::new("corpus"));

    if !corpus.exists() {
        bail!("Corpus directory not found: {:?}", corpus);
    }

    println!("Running tests in corpus: {:?}", corpus);

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

    println!("Found {} test files", test_files.len());

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for test_file in &test_files {
        let result = run_single_test(test_file, parser_path)?;

        if result.passed {
            passed += 1;
            print!(".");
        } else {
            failed += 1;
            print!("F");
        }

        results.push(result);
    }

    println!("\n\nTest Results:");
    println!("  Passed: {}", passed);
    println!("  Failed: {}", failed);
    println!("  Total:  {}", test_files.len());

    // Show failures
    if failed > 0 {
        println!("\nFailures:");
        for result in &results {
            if !result.passed {
                println!(
                    "  {} - {}",
                    result.file,
                    result
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );
            }
        }
    }

    if show_stats {
        let total_parse_time: f64 = results.iter().map(|r| r.parse_time_ms).sum();
        let avg_parse_time = total_parse_time / results.len() as f64;

        println!("\nStatistics:");
        println!("  Total parse time: {:.2}ms", total_parse_time);
        println!("  Average parse time: {:.2}ms", avg_parse_time);
    }

    if failed > 0 {
        bail!("{} tests failed", failed);
    }

    Ok(())
}

/// Run a single test file
/// 
/// NOTE: This is currently a placeholder that always returns success.
/// Real test execution requires the compiled parser.
fn run_single_test(test_file: &Path, parser_path: Option<&Path>) -> Result<TestResult> {
    let file_name = test_file.to_string_lossy().to_string();
    let start = Instant::now();

    // Read test file
    let content = fs::read_to_string(test_file)
        .with_context(|| format!("Failed to read test file: {:?}", test_file))?;

    // Parse test format
    let test_cases = parse_test_format(&content)?;

    if parser_path.is_some() {
        eprintln!("Warning: Dynamic parser loading not yet supported");
    }

    // This is a placeholder - real implementation would:
    // 1. Load the compiled parser
    // 2. Parse each test case
    // 3. Compare with expected output
    // For now, return an error to be honest about the limitation
    
    let parse_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Return failure with helpful message
    Ok(TestResult {
        file: file_name,
        passed: false,
        parse_time_ms,
        error: Some("Test execution not implemented. Use cargo test with your generated parser instead.".to_string()),
    })
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

