//! Corpus test runner for grammar compatibility testing

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use xshell::{cmd, Shell};

/// Grammar test status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Pass,
    Fail(String),
    NotImplemented,
}

/// Results for a single grammar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarTestResult {
    pub name: String,
    pub status: TestStatus,
    pub parse_tests_passed: usize,
    pub parse_tests_total: usize,
    pub query_tests_passed: usize,
    pub query_tests_total: usize,
    pub error_message: Option<String>,
}

/// Overall corpus test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusTestResults {
    pub timestamp: String,
    pub total_grammars: usize,
    pub passing_grammars: usize,
    pub failing_grammars: usize,
    pub pass_rate: f64,
    pub grammar_results: HashMap<String, GrammarTestResult>,
}

pub struct CorpusRunner {
    corpus_path: PathBuf,
    output_path: PathBuf,
}

impl CorpusRunner {
    pub fn new(corpus_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            corpus_path,
            output_path,
        }
    }
    
    /// Run tests for all grammars in the corpus
    pub fn run_all(&self) -> Result<CorpusTestResults> {
        println!("Running corpus tests...");
        
        let mut results = CorpusTestResults {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_grammars: 0,
            passing_grammars: 0,
            failing_grammars: 0,
            pass_rate: 0.0,
            grammar_results: HashMap::new(),
        };
        
        // List of grammars to test (start with a subset)
        let grammars = vec![
            "javascript",
            "typescript", 
            "rust",
            "python",
            "go",
            "java",
            "c",
            "cpp",
            "ruby",
            "bash",
        ];
        
        for grammar_name in grammars {
            results.total_grammars += 1;
            
            match self.test_grammar(grammar_name) {
                Ok(result) => {
                    if matches!(result.status, TestStatus::Pass) {
                        results.passing_grammars += 1;
                    } else {
                        results.failing_grammars += 1;
                    }
                    results.grammar_results.insert(grammar_name.to_string(), result);
                }
                Err(e) => {
                    results.failing_grammars += 1;
                    results.grammar_results.insert(grammar_name.to_string(), GrammarTestResult {
                        name: grammar_name.to_string(),
                        status: TestStatus::Fail(e.to_string()),
                        parse_tests_passed: 0,
                        parse_tests_total: 0,
                        query_tests_passed: 0,
                        query_tests_total: 0,
                        error_message: Some(e.to_string()),
                    });
                }
            }
        }
        
        results.pass_rate = if results.total_grammars > 0 {
            (results.passing_grammars as f64 / results.total_grammars as f64) * 100.0
        } else {
            0.0
        };
        
        // Save results
        self.save_results(&results)?;
        
        Ok(results)
    }
    
    /// Test a single grammar
    pub fn test_grammar(&self, grammar_name: &str) -> Result<GrammarTestResult> {
        println!("Testing grammar: {}", grammar_name);
        
        let grammar_path = self.corpus_path.join(format!("tree-sitter-{}", grammar_name));
        
        // Check if grammar exists
        if !grammar_path.exists() {
            return Ok(GrammarTestResult {
                name: grammar_name.to_string(),
                status: TestStatus::NotImplemented,
                parse_tests_passed: 0,
                parse_tests_total: 0,
                query_tests_passed: 0,
                query_tests_total: 0,
                error_message: Some("Grammar not found in corpus".to_string()),
            });
        }
        
        // Try to parse grammar.js
        let grammar_js_path = grammar_path.join("grammar.js");
        if !grammar_js_path.exists() {
            return Ok(GrammarTestResult {
                name: grammar_name.to_string(),
                status: TestStatus::Fail("grammar.js not found".to_string()),
                parse_tests_passed: 0,
                parse_tests_total: 0,
                query_tests_passed: 0,
                query_tests_total: 0,
                error_message: Some("grammar.js not found".to_string()),
            });
        }
        
        // For now, just check if we can read the grammar.js file
        let grammar_content = fs::read_to_string(&grammar_js_path)
            .context("Failed to read grammar.js")?;
        
        // Try to parse with our grammar.js parser
        match rust_sitter_tool::parse_grammar_js(&grammar_content) {
            Ok(_grammar) => {
                Ok(GrammarTestResult {
                    name: grammar_name.to_string(),
                    status: TestStatus::Pass,
                    parse_tests_passed: 1,
                    parse_tests_total: 1,
                    query_tests_passed: 0,
                    query_tests_total: 0,
                    error_message: None,
                })
            }
            Err(e) => {
                Ok(GrammarTestResult {
                    name: grammar_name.to_string(),
                    status: TestStatus::Fail(format!("Parse error: {}", e)),
                    parse_tests_passed: 0,
                    parse_tests_total: 1,
                    query_tests_passed: 0,
                    query_tests_total: 0,
                    error_message: Some(e.to_string()),
                })
            }
        }
    }
    
    /// Save test results to file
    fn save_results(&self, results: &CorpusTestResults) -> Result<()> {
        fs::create_dir_all(&self.output_path)?;
        
        let results_file = self.output_path.join("corpus_results.json");
        let json = serde_json::to_string_pretty(results)?;
        fs::write(results_file, json)?;
        
        // Also save a summary
        let summary = self.generate_summary(results);
        let summary_file = self.output_path.join("corpus_summary.txt");
        fs::write(summary_file, summary)?;
        
        Ok(())
    }
    
    /// Generate a text summary of results
    fn generate_summary(&self, results: &CorpusTestResults) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("Corpus Test Results - {}\n", results.timestamp));
        summary.push_str(&format!("=====================================\n\n"));
        
        summary.push_str(&format!("Total Grammars: {}\n", results.total_grammars));
        summary.push_str(&format!("Passing: {} ({:.1}%)\n", results.passing_grammars, results.pass_rate));
        summary.push_str(&format!("Failing: {}\n\n", results.failing_grammars));
        
        summary.push_str("Grammar Results:\n");
        summary.push_str("---------------\n");
        
        let mut grammars: Vec<_> = results.grammar_results.iter().collect();
        grammars.sort_by_key(|(name, _)| name.as_str());
        
        for (name, result) in grammars {
            let status_symbol = match &result.status {
                TestStatus::Pass => "✅",
                TestStatus::Fail(_) => "❌",
                TestStatus::NotImplemented => "⏳",
            };
            
            summary.push_str(&format!("{} {} - ", status_symbol, name));
            
            match &result.status {
                TestStatus::Pass => {
                    summary.push_str(&format!("Pass ({}/{} parse tests)\n", 
                        result.parse_tests_passed, result.parse_tests_total));
                }
                TestStatus::Fail(msg) => {
                    summary.push_str(&format!("Fail: {}\n", msg));
                }
                TestStatus::NotImplemented => {
                    summary.push_str("Not implemented\n");
                }
            }
        }
        
        summary
    }
}

/// Download Tree-sitter grammar corpus
pub fn download_corpus(target_dir: &Path) -> Result<()> {
    let sh = Shell::new()?;
    
    println!("Downloading Tree-sitter grammar corpus...");
    
    // Create target directory
    fs::create_dir_all(target_dir)?;
    
    // List of grammars to download
    let grammars = vec![
        ("javascript", "https://github.com/tree-sitter/tree-sitter-javascript"),
        ("typescript", "https://github.com/tree-sitter/tree-sitter-typescript"),
        ("rust", "https://github.com/tree-sitter/tree-sitter-rust"),
        ("python", "https://github.com/tree-sitter/tree-sitter-python"),
        ("go", "https://github.com/tree-sitter/tree-sitter-go"),
        ("java", "https://github.com/tree-sitter/tree-sitter-java"),
        ("c", "https://github.com/tree-sitter/tree-sitter-c"),
        ("cpp", "https://github.com/tree-sitter/tree-sitter-cpp"),
        ("ruby", "https://github.com/tree-sitter/tree-sitter-ruby"),
        ("bash", "https://github.com/tree-sitter/tree-sitter-bash"),
    ];
    
    for (name, url) in grammars {
        let grammar_dir = target_dir.join(format!("tree-sitter-{}", name));
        
        if grammar_dir.exists() {
            println!("  {} already exists, updating...", name);
            sh.change_dir(&grammar_dir);
            cmd!(sh, "git pull").run()?;
        } else {
            println!("  Cloning {}...", name);
            sh.change_dir(target_dir);
            cmd!(sh, "git clone --depth 1 {url}").run()?;
        }
    }
    
    println!("Corpus download complete!");
    Ok(())
}