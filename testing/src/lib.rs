// Beta testing framework for rust-sitter
// Tests compatibility with official Tree-sitter grammars

use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use rust_sitter_ir::Grammar;
use rust_sitter_glr_core::ParseTable;

/// Test result for a single grammar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarTestResult {
    pub name: String,
    pub version: String,
    pub passed: bool,
    pub total_tests: usize,
    pub failed_tests: usize,
    pub parse_time_ms: f64,
    pub tree_sitter_time_ms: f64,
    pub speedup: f64,
    pub errors: Vec<String>,
    pub compatibility_score: f64,
}

/// Configuration for testing a grammar
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub grammar_path: PathBuf,
    pub test_files: Vec<PathBuf>,
    pub tree_sitter_path: Option<PathBuf>,
    pub compare_output: bool,
    pub benchmark: bool,
    pub external_scanner: Option<String>,
}

/// Main testing framework
pub struct BetaTester {
    results: Vec<GrammarTestResult>,
    config: TestConfig,
}

impl BetaTester {
    pub fn new(config: TestConfig) -> Self {
        Self {
            results: Vec::new(),
            config,
        }
    }
    
    /// Test a grammar against Tree-sitter
    pub fn test_grammar(&mut self, grammar_name: &str) -> Result<GrammarTestResult> {
        let mut result = GrammarTestResult {
            name: grammar_name.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            passed: true,
            total_tests: 0,
            failed_tests: 0,
            parse_time_ms: 0.0,
            tree_sitter_time_ms: 0.0,
            speedup: 0.0,
            errors: Vec::new(),
            compatibility_score: 0.0,
        };
        
        // Load the grammar
        let grammar = self.load_grammar(&self.config.grammar_path)?;
        
        // Generate parser
        let parse_table = self.generate_parse_table(&grammar)?;
        
        // Run tests
        for test_file in &self.config.test_files {
            result.total_tests += 1;
            
            match self.test_file(&grammar, &parse_table, test_file) {
                Ok(test_result) => {
                    result.parse_time_ms += test_result.parse_time_ms;
                    
                    if self.config.compare_output {
                        match self.compare_with_tree_sitter(test_file, &test_result) {
                            Ok(tree_sitter_result) => {
                                result.tree_sitter_time_ms += tree_sitter_result.parse_time_ms;
                                
                                if test_result.output != tree_sitter_result.output {
                                    result.failed_tests += 1;
                                    result.errors.push(format!(
                                        "Output mismatch for {}: rust-sitter != tree-sitter",
                                        test_file.display()
                                    ));
                                }
                            }
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to compare with tree-sitter: {}",
                                    e
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    result.failed_tests += 1;
                    result.errors.push(format!(
                        "Failed to parse {}: {}",
                        test_file.display(),
                        e
                    ));
                }
            }
        }
        
        // Calculate metrics
        result.passed = result.failed_tests == 0;
        result.compatibility_score = if result.total_tests > 0 {
            ((result.total_tests - result.failed_tests) as f64 / result.total_tests as f64) * 100.0
        } else {
            0.0
        };
        
        if result.tree_sitter_time_ms > 0.0 {
            result.speedup = result.tree_sitter_time_ms / result.parse_time_ms;
        }
        
        self.results.push(result.clone());
        Ok(result)
    }
    
    /// Load a grammar from disk
    fn load_grammar(&self, path: &Path) -> Result<Grammar> {
        // TODO: Implement grammar loading
        // This would parse the rust-sitter grammar definition
        unimplemented!("Grammar loading not yet implemented")
    }
    
    /// Generate parse table for grammar
    fn generate_parse_table(&self, grammar: &Grammar) -> Result<ParseTable> {
        use rust_sitter_glr_core::lr1::LR1Automaton;
        
        // Generate LR(1) automaton
        let automaton = LR1Automaton::build(grammar)?;
        
        // Convert to parse table
        let table = automaton.to_parse_table()?;
        
        Ok(table)
    }
    
    /// Test a single file
    fn test_file(
        &self,
        grammar: &Grammar,
        parse_table: &ParseTable,
        file_path: &Path,
    ) -> Result<FileTestResult> {
        use std::time::Instant;
        use rust_sitter::parser_v3::Parser;
        
        // Read file
        let content = fs::read_to_string(file_path)?;
        
        // Create parser
        let mut parser = Parser::new(grammar.clone(), parse_table.clone());
        
        // Parse and measure time
        let start = Instant::now();
        let tree = parser.parse(&content)?;
        let parse_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        // Serialize tree for comparison
        let output = self.serialize_tree(&tree);
        
        Ok(FileTestResult {
            file_path: file_path.to_path_buf(),
            parse_time_ms,
            output,
        })
    }
    
    /// Compare with Tree-sitter output
    fn compare_with_tree_sitter(
        &self,
        file_path: &Path,
        _rust_sitter_result: &FileTestResult,
    ) -> Result<FileTestResult> {
        if let Some(tree_sitter_path) = &self.config.tree_sitter_path {
            // Run tree-sitter CLI
            let output = Command::new(tree_sitter_path)
                .arg("parse")
                .arg(file_path)
                .output()
                .context("Failed to run tree-sitter")?;
            
            if !output.status.success() {
                anyhow::bail!("tree-sitter parse failed");
            }
            
            // Parse timing from stderr if available
            let parse_time_ms = self.extract_parse_time(&output.stderr);
            
            Ok(FileTestResult {
                file_path: file_path.to_path_buf(),
                parse_time_ms,
                output: String::from_utf8_lossy(&output.stdout).to_string(),
            })
        } else {
            anyhow::bail!("tree-sitter path not configured")
        }
    }
    
    /// Extract parse time from tree-sitter output
    fn extract_parse_time(&self, stderr: &[u8]) -> f64 {
        // Tree-sitter outputs timing info to stderr
        // Format: "Parse time: X.XXXms"
        let stderr_str = String::from_utf8_lossy(stderr);
        
        if let Some(line) = stderr_str.lines().find(|l| l.contains("Parse time:")) {
            if let Some(time_str) = line.split(':').nth(1) {
                if let Some(ms_str) = time_str.trim().strip_suffix("ms") {
                    if let Ok(ms) = ms_str.parse::<f64>() {
                        return ms;
                    }
                }
            }
        }
        
        0.0
    }
    
    /// Serialize parse tree for comparison
    fn serialize_tree(&self, tree: &rust_sitter::parser_v3::ParseNode) -> String {
        use rust_sitter::serialization::SExpressionSerializer;
        
        let serializer = SExpressionSerializer::new(&[]);
        serializer.serialize_node(tree)
    }
    
    /// Generate compatibility report
    pub fn generate_report(&self) -> CompatibilityReport {
        let total_grammars = self.results.len();
        let passed_grammars = self.results.iter().filter(|r| r.passed).count();
        let total_tests: usize = self.results.iter().map(|r| r.total_tests).sum();
        let failed_tests: usize = self.results.iter().map(|r| r.failed_tests).sum();
        
        let avg_speedup = if !self.results.is_empty() {
            self.results.iter()
                .filter(|r| r.speedup > 0.0)
                .map(|r| r.speedup)
                .sum::<f64>() / self.results.len() as f64
        } else {
            0.0
        };
        
        CompatibilityReport {
            version: env!("CARGO_PKG_VERSION").to_string(),
            date: chrono::Utc::now().to_rfc3339(),
            total_grammars,
            passed_grammars,
            total_tests,
            failed_tests,
            overall_compatibility: if total_tests > 0 {
                ((total_tests - failed_tests) as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            },
            average_speedup: avg_speedup,
            grammar_results: self.results.clone(),
        }
    }
}

/// Result for a single file test
#[derive(Debug)]
struct FileTestResult {
    file_path: PathBuf,
    parse_time_ms: f64,
    output: String,
}

/// Overall compatibility report
#[derive(Debug, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub version: String,
    pub date: String,
    pub total_grammars: usize,
    pub passed_grammars: usize,
    pub total_tests: usize,
    pub failed_tests: usize,
    pub overall_compatibility: f64,
    pub average_speedup: f64,
    pub grammar_results: Vec<GrammarTestResult>,
}

impl CompatibilityReport {
    /// Save report to JSON file
    pub fn save_json(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    /// Save report as Markdown
    pub fn save_markdown(&self, path: &Path) -> Result<()> {
        let mut md = String::new();
        
        md.push_str(&format!("# Rust Sitter Compatibility Report\n\n"));
        md.push_str(&format!("**Version**: {}\n", self.version));
        md.push_str(&format!("**Date**: {}\n", self.date));
        md.push_str(&format!("**Overall Compatibility**: {:.2}%\n", self.overall_compatibility));
        md.push_str(&format!("**Average Speedup**: {:.2}x\n\n", self.average_speedup));
        
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- Grammars Tested: {}\n", self.total_grammars));
        md.push_str(&format!("- Grammars Passed: {} ({:.1}%)\n", 
            self.passed_grammars,
            (self.passed_grammars as f64 / self.total_grammars as f64) * 100.0
        ));
        md.push_str(&format!("- Total Tests: {}\n", self.total_tests));
        md.push_str(&format!("- Failed Tests: {}\n\n", self.failed_tests));
        
        md.push_str("## Grammar Results\n\n");
        md.push_str("| Grammar | Compatibility | Tests | Failed | Speedup |\n");
        md.push_str("|---------|--------------|-------|--------|----------|\n");
        
        for result in &self.grammar_results {
            md.push_str(&format!(
                "| {} | {:.1}% | {} | {} | {:.2}x |\n",
                result.name,
                result.compatibility_score,
                result.total_tests,
                result.failed_tests,
                result.speedup
            ));
        }
        
        fs::write(path, md)?;
        Ok(())
    }
}

/// Grammar test suite runner
pub struct TestSuite {
    grammars: Vec<(String, TestConfig)>,
}

impl TestSuite {
    pub fn new() -> Self {
        Self {
            grammars: Vec::new(),
        }
    }
    
    /// Add a grammar to test
    pub fn add_grammar(&mut self, name: String, config: TestConfig) {
        self.grammars.push((name, config));
    }
    
    /// Run all tests
    pub fn run_all(&self) -> Result<CompatibilityReport> {
        let mut all_results = Vec::new();
        
        for (name, config) in &self.grammars {
            println!("Testing grammar: {}", name);
            
            let mut tester = BetaTester::new(config.clone());
            match tester.test_grammar(name) {
                Ok(result) => {
                    println!("  ✓ Compatibility: {:.1}%", result.compatibility_score);
                    all_results.push(result);
                }
                Err(e) => {
                    println!("  ✗ Error: {}", e);
                    all_results.push(GrammarTestResult {
                        name: name.clone(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        passed: false,
                        total_tests: 0,
                        failed_tests: 0,
                        parse_time_ms: 0.0,
                        tree_sitter_time_ms: 0.0,
                        speedup: 0.0,
                        errors: vec![e.to_string()],
                        compatibility_score: 0.0,
                    });
                }
            }
        }
        
        let total_grammars = all_results.len();
        let passed_grammars = all_results.iter().filter(|r| r.passed).count();
        let total_tests: usize = all_results.iter().map(|r| r.total_tests).sum();
        let failed_tests: usize = all_results.iter().map(|r| r.failed_tests).sum();
        
        let avg_speedup = if !all_results.is_empty() {
            all_results.iter()
                .filter(|r| r.speedup > 0.0)
                .map(|r| r.speedup)
                .sum::<f64>() / all_results.len() as f64
        } else {
            0.0
        };
        
        Ok(CompatibilityReport {
            version: env!("CARGO_PKG_VERSION").to_string(),
            date: chrono::Utc::now().to_rfc3339(),
            total_grammars,
            passed_grammars,
            total_tests,
            failed_tests,
            overall_compatibility: if total_tests > 0 {
                ((total_tests - failed_tests) as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            },
            average_speedup: avg_speedup,
            grammar_results: all_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_report_generation() {
        let report = CompatibilityReport {
            version: "0.1.0".to_string(),
            date: "2024-01-23".to_string(),
            total_grammars: 2,
            passed_grammars: 1,
            total_tests: 100,
            failed_tests: 5,
            overall_compatibility: 95.0,
            average_speedup: 1.5,
            grammar_results: vec![
                GrammarTestResult {
                    name: "javascript".to_string(),
                    version: "0.1.0".to_string(),
                    passed: true,
                    total_tests: 50,
                    failed_tests: 0,
                    parse_time_ms: 100.0,
                    tree_sitter_time_ms: 150.0,
                    speedup: 1.5,
                    errors: vec![],
                    compatibility_score: 100.0,
                },
                GrammarTestResult {
                    name: "python".to_string(),
                    version: "0.1.0".to_string(),
                    passed: false,
                    total_tests: 50,
                    failed_tests: 5,
                    parse_time_ms: 120.0,
                    tree_sitter_time_ms: 180.0,
                    speedup: 1.5,
                    errors: vec!["Indentation handling mismatch".to_string()],
                    compatibility_score: 90.0,
                },
            ],
        };
        
        // Test JSON serialization
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("\"overall_compatibility\": 95.0"));
        
        // Test markdown generation
        let md_path = PathBuf::from("/tmp/test_report.md");
        report.save_markdown(&md_path).unwrap();
        let md_content = fs::read_to_string(&md_path).unwrap();
        assert!(md_content.contains("Overall Compatibility: 95.00%"));
    }
}