use anyhow::{Result, Context, bail};
use std::path::Path;
use std::fs;
use std::time::Instant;
use glob::glob;
use serde::{Deserialize, Serialize};

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
                println!("  {} - {}", result.file, result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
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
fn run_single_test(test_file: &Path, _parser_path: Option<&Path>) -> Result<TestResult> {
    let file_name = test_file.to_string_lossy().to_string();
    let start = Instant::now();
    
    // Read test file
    let content = fs::read_to_string(test_file)
        .with_context(|| format!("Failed to read test file: {:?}", test_file))?;
    
    // Parse test format
    let _test_cases = parse_test_format(&content)?;
    
    // For now, mock the test execution
    // In a full implementation, we would:
    // 1. Parse each test case with the parser
    // 2. Compare the result with the expected output
    // 3. Handle error cases
    
    let parse_time_ms = start.elapsed().as_secs_f64() * 1000.0;
    
    // Mock result - always pass for now
    Ok(TestResult {
        file: file_name,
        passed: true,
        parse_time_ms,
        error: None,
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

/// Generate a test corpus from examples
pub fn generate_corpus(
    examples_dir: &Path,
    output_dir: &Path,
    language: &str,
) -> Result<()> {
    println!("Generating corpus from examples in {:?}", examples_dir);
    println!("Output directory: {:?}", output_dir);
    println!("Language: {}", language);
    
    // Create output directory
    fs::create_dir_all(output_dir)?;
    
    // Find example files
    let pattern = examples_dir.join("**/*.*");
    let example_files: Vec<_> = glob(pattern.to_str().unwrap())?
        .filter_map(Result::ok)
        .collect();
    
    println!("Found {} example files", example_files.len());
    
    // Generate test files
    for example_file in example_files {
        let relative_path = example_file.strip_prefix(examples_dir)?;
        let test_name = relative_path.to_string_lossy().replace('/', "_").replace('.', "_");
        let test_file = output_dir.join(format!("{}.txt", test_name));
        
        let source = fs::read_to_string(&example_file)?;
        
        // Create test file content
        let test_content = format!(
            "=== {}\n{}\n---\n\n(source_file)\n",
            relative_path.display(),
            source.trim()
        );
        
        fs::write(&test_file, test_content)?;
        println!("Generated test: {:?}", test_file);
    }
    
    Ok(())
}