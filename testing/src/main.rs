// CLI tool for running rust-sitter beta tests

use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_sitter_testing::{BetaTester, TestConfig, TestSuite};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rust-sitter-test")]
#[command(about = "Beta testing tool for rust-sitter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a single grammar
    Test {
        /// Grammar name
        #[arg(short, long)]
        grammar: String,

        /// Path to grammar definition
        #[arg(short = 'p', long)]
        path: PathBuf,

        /// Test files or directories
        #[arg(short = 'f', long)]
        files: Vec<PathBuf>,

        /// Path to tree-sitter CLI for comparison
        #[arg(long)]
        tree_sitter: Option<PathBuf>,

        /// Enable benchmarking
        #[arg(short, long)]
        benchmark: bool,

        /// External scanner name
        #[arg(long)]
        scanner: Option<String>,
    },

    /// Run full test suite
    Suite {
        /// Configuration file
        #[arg(short, long, default_value = "test-suite.json")]
        config: PathBuf,

        /// Output directory for reports
        #[arg(short, long, default_value = "reports")]
        output: PathBuf,
    },

    /// Test official Tree-sitter grammar corpus
    Corpus {
        /// Path to tree-sitter repository
        #[arg(short, long)]
        tree_sitter_path: PathBuf,

        /// Grammars to test (defaults to all)
        #[arg(short, long)]
        grammars: Vec<String>,

        /// Output directory for reports
        #[arg(short, long, default_value = "corpus-reports")]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            grammar,
            path,
            files,
            tree_sitter,
            benchmark,
            scanner,
        } => test_single_grammar(grammar, path, files, tree_sitter, benchmark, scanner),
        Commands::Suite { config, output } => run_test_suite(config, output),
        Commands::Corpus {
            tree_sitter_path,
            grammars,
            output,
        } => test_corpus(tree_sitter_path, grammars, output),
    }
}

fn test_single_grammar(
    name: String,
    path: PathBuf,
    files: Vec<PathBuf>,
    tree_sitter: Option<PathBuf>,
    benchmark: bool,
    scanner: Option<String>,
) -> Result<()> {
    println!("Testing grammar: {}", name);

    let config = TestConfig {
        grammar_path: path,
        test_files: files,
        tree_sitter_path: tree_sitter.clone(),
        compare_output: tree_sitter.is_some(),
        benchmark,
        external_scanner: scanner,
    };

    let mut tester = BetaTester::new(config);
    let result = tester.test_grammar(&name)?;

    // Print results
    println!("\nResults:");
    println!("  Compatibility: {:.2}%", result.compatibility_score);
    println!(
        "  Tests passed: {}/{}",
        result.total_tests - result.failed_tests,
        result.total_tests
    );

    if result.speedup > 0.0 {
        println!(
            "  Performance: {:.2}x faster than tree-sitter",
            result.speedup
        );
    }

    if !result.errors.is_empty() {
        println!("\nErrors:");
        for error in &result.errors {
            println!("  - {}", error);
        }
    }

    Ok(())
}

fn run_test_suite(config_path: PathBuf, output_dir: PathBuf) -> Result<()> {
    println!("Running test suite from: {}", config_path.display());

    // Load suite configuration
    let config_str = std::fs::read_to_string(&config_path)?;
    let suite_config: SuiteConfig = serde_json::from_str(&config_str)?;

    // Create output directory
    std::fs::create_dir_all(&output_dir)?;

    // Build test suite
    let mut suite = TestSuite::new();

    for grammar_config in suite_config.grammars {
        let test_config = TestConfig {
            grammar_path: grammar_config.path,
            test_files: grammar_config.test_files,
            tree_sitter_path: grammar_config.tree_sitter_path,
            compare_output: true,
            benchmark: true,
            external_scanner: grammar_config.external_scanner,
        };

        suite.add_grammar(grammar_config.name, test_config);
    }

    // Run all tests
    let report = suite.run_all()?;

    // Save reports
    let json_path = output_dir.join("compatibility-report.json");
    let md_path = output_dir.join("compatibility-report.md");

    report.save_json(&json_path)?;
    report.save_markdown(&md_path)?;

    println!("\nTest suite complete!");
    println!(
        "  Overall compatibility: {:.2}%",
        report.overall_compatibility
    );
    println!("  Reports saved to: {}", output_dir.display());

    Ok(())
}

fn test_corpus(
    tree_sitter_path: PathBuf,
    grammars: Vec<String>,
    output_dir: PathBuf,
) -> Result<()> {
    println!("Testing Tree-sitter grammar corpus");

    // Create output directory
    std::fs::create_dir_all(&output_dir)?;

    // Find available grammars
    let available_grammars = find_corpus_grammars(&tree_sitter_path)?;

    // Filter grammars to test
    let grammars_to_test: Vec<_> = if grammars.is_empty() {
        available_grammars
    } else {
        available_grammars
            .into_iter()
            .filter(|g| grammars.contains(&g.name))
            .collect()
    };

    println!("Found {} grammars to test", grammars_to_test.len());

    // Build test suite
    let mut suite = TestSuite::new();

    for grammar_info in grammars_to_test {
        println!("  - {}", grammar_info.name);

        let test_config = TestConfig {
            grammar_path: grammar_info.grammar_path,
            test_files: grammar_info.test_files,
            tree_sitter_path: Some(tree_sitter_path.join("tree-sitter")),
            compare_output: true,
            benchmark: true,
            external_scanner: grammar_info
                .has_scanner
                .then_some(grammar_info.name.clone()),
        };

        suite.add_grammar(grammar_info.name, test_config);
    }

    // Run all tests
    let report = suite.run_all()?;

    // Save reports
    let json_path = output_dir.join("corpus-compatibility-report.json");
    let md_path = output_dir.join("corpus-compatibility-report.md");

    report.save_json(&json_path)?;
    report.save_markdown(&md_path)?;

    println!("\nCorpus testing complete!");
    println!(
        "  Overall compatibility: {:.2}%",
        report.overall_compatibility
    );
    println!("  Average speedup: {:.2}x", report.average_speedup);
    println!("  Reports saved to: {}", output_dir.display());

    Ok(())
}

/// Find grammars in the Tree-sitter repository
fn find_corpus_grammars(tree_sitter_path: &PathBuf) -> Result<Vec<CorpusGrammarInfo>> {
    let mut grammars = Vec::new();

    // Common grammar locations in tree-sitter repos
    let grammar_dirs = vec![
        tree_sitter_path.join("test/fixtures/grammars"),
        tree_sitter_path.join("grammars"),
    ];

    for dir in grammar_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            // Check if it's a grammar directory
            let grammar_js = path.join("grammar.js");
            if !grammar_js.exists() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_string())
                .unwrap_or_default();

            // Find test files
            let mut test_files = Vec::new();

            let test_dir = path.join("test/corpus");
            if test_dir.exists() {
                for test_entry in std::fs::read_dir(&test_dir)? {
                    let test_entry = test_entry?;
                    let test_path = test_entry.path();

                    if test_path.extension().and_then(|e| e.to_str()) == Some("txt") {
                        test_files.push(test_path);
                    }
                }
            }

            // Check for external scanner
            let has_scanner =
                path.join("src/scanner.c").exists() || path.join("src/scanner.cc").exists();

            grammars.push(CorpusGrammarInfo {
                name: name.clone(),
                grammar_path: grammar_js,
                test_files,
                has_scanner,
            });
        }
    }

    Ok(grammars)
}

/// Suite configuration loaded from JSON
#[derive(Debug, serde::Deserialize)]
struct SuiteConfig {
    grammars: Vec<GrammarConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct GrammarConfig {
    name: String,
    path: PathBuf,
    test_files: Vec<PathBuf>,
    tree_sitter_path: Option<PathBuf>,
    external_scanner: Option<String>,
}

/// Information about a corpus grammar
struct CorpusGrammarInfo {
    name: String,
    grammar_path: PathBuf,
    test_files: Vec<PathBuf>,
    has_scanner: bool,
}
