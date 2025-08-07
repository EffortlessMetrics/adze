// Beta testing framework for rust-sitter
// Tests compatibility with official Tree-sitter grammars

use anyhow::{Context, Result};
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::Grammar;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
                                result
                                    .errors
                                    .push(format!("Failed to compare with tree-sitter: {}", e));
                            }
                        }
                    }
                }
                Err(e) => {
                    result.failed_tests += 1;
                    result
                        .errors
                        .push(format!("Failed to parse {}: {}", test_file.display(), e));
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
    fn load_grammar(&self, _path: &Path) -> Result<Grammar> {
        // TODO: Implement grammar loading
        // This would parse the rust-sitter grammar definition
        unimplemented!("Grammar loading not yet implemented")
    }

    /// Generate parse table for grammar
    fn generate_parse_table(&self, _grammar: &Grammar) -> Result<ParseTable> {
        // TODO: Implement parse table generation
        // This would use the GLR core to generate tables
        unimplemented!("Parse table generation not yet implemented")
    }

    /// Test a single file
    fn test_file(
        &self,
        _grammar: &Grammar,
        _parse_table: &ParseTable,
        file_path: &Path,
    ) -> Result<FileTestResult> {
        use std::time::Instant;
        // TODO: Use proper parser when API is stable

        // Read file
        let content = fs::read_to_string(file_path)?;

        // Create parser
        // TODO: Implement actual parsing when API is stable
        // let mut parser = Parser::new(grammar.clone(), parse_table.clone());

        // Parse and measure time
        let start = Instant::now();
        // let tree = parser.parse(&content)?;
        let parse_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Serialize tree for comparison
        let output = self.serialize_tree(&content);

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
    fn serialize_tree(&self, _tree: &str) -> String {
        // TODO: Implement tree serialization
        // For now, return a placeholder
        "(placeholder)".to_string()
    }

    /// Generate compatibility report
    pub fn generate_report(&self) -> CompatibilityReport {
        let total_grammars = self.results.len();
        let passed_grammars = self.results.iter().filter(|r| r.passed).count();
        let total_tests: usize = self.results.iter().map(|r| r.total_tests).sum();
        let failed_tests: usize = self.results.iter().map(|r| r.failed_tests).sum();

        let avg_speedup = if !self.results.is_empty() {
            self.results
                .iter()
                .filter(|r| r.speedup > 0.0)
                .map(|r| r.speedup)
                .sum::<f64>()
                / self.results.len() as f64
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
    #[allow(dead_code)]
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

        md.push_str(&"# Rust Sitter Compatibility Report\n\n".to_string());
        md.push_str(&format!("**Version**: {}\n", self.version));
        md.push_str(&format!("**Date**: {}\n", self.date));
        md.push_str(&format!(
            "**Overall Compatibility**: {:.2}%\n",
            self.overall_compatibility
        ));
        md.push_str(&format!(
            "**Average Speedup**: {:.2}x\n\n",
            self.average_speedup
        ));

        md.push_str("## Summary\n\n");
        md.push_str(&format!("- Grammars Tested: {}\n", self.total_grammars));
        md.push_str(&format!(
            "- Grammars Passed: {} ({:.1}%)\n",
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

impl Default for TestSuite {
    fn default() -> Self {
        Self::new()
    }
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
            all_results
                .iter()
                .filter(|r| r.speedup > 0.0)
                .map(|r| r.speedup)
                .sum::<f64>()
                / all_results.len() as f64
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
        let result = report.save_markdown(&md_path);

        // Only check if file was created successfully, not exact content
        // since markdown formatting might vary
        assert!(result.is_ok());

        // Verify file exists
        assert!(md_path.exists());

        // Clean up
        let _ = fs::remove_file(&md_path);
    }

    #[test]
    fn test_grammar_test_result_creation() {
        let result = GrammarTestResult {
            name: "test-grammar".to_string(),
            version: "1.0.0".to_string(),
            passed: true,
            total_tests: 10,
            failed_tests: 2,
            parse_time_ms: 50.0,
            tree_sitter_time_ms: 75.0,
            speedup: 1.5,
            errors: vec![],
            compatibility_score: 80.0,
        };

        assert_eq!(result.name, "test-grammar");
        assert!(result.passed);
        assert_eq!(result.total_tests, 10);
        assert_eq!(result.failed_tests, 2);
        assert_eq!(result.speedup, 1.5);
        assert_eq!(result.compatibility_score, 80.0);
    }

    #[test]
    fn test_test_config_creation() {
        let config = TestConfig {
            grammar_path: PathBuf::from("path/to/grammar"),
            test_files: vec![PathBuf::from("test1.js"), PathBuf::from("test2.js")],
            tree_sitter_path: Some(PathBuf::from("/usr/bin/tree-sitter")),
            compare_output: true,
            benchmark: true,
            external_scanner: Some("scanner.so".to_string()),
        };

        assert_eq!(config.grammar_path, PathBuf::from("path/to/grammar"));
        assert_eq!(config.test_files.len(), 2);
        assert!(config.compare_output);
        assert!(config.benchmark);
        assert_eq!(config.external_scanner, Some("scanner.so".to_string()));
    }

    #[test]
    fn test_beta_tester_creation() {
        let config = TestConfig {
            grammar_path: PathBuf::from("test.grammar"),
            test_files: vec![],
            tree_sitter_path: None,
            compare_output: false,
            benchmark: false,
            external_scanner: None,
        };

        let tester = BetaTester::new(config.clone());
        assert!(tester.results.is_empty());
        assert_eq!(tester.config.grammar_path, config.grammar_path);
    }

    #[test]
    fn test_extract_parse_time() {
        let tester = BetaTester::new(TestConfig {
            grammar_path: PathBuf::from("test"),
            test_files: vec![],
            tree_sitter_path: None,
            compare_output: false,
            benchmark: false,
            external_scanner: None,
        });

        // Test with valid parse time
        let stderr = b"Some output\nParse time: 12.345ms\nMore output";
        let time = tester.extract_parse_time(stderr);
        assert_eq!(time, 12.345);

        // Test with no parse time
        let stderr = b"No timing information here";
        let time = tester.extract_parse_time(stderr);
        assert_eq!(time, 0.0);

        // Test with malformed parse time
        let stderr = b"Parse time: invalidms";
        let time = tester.extract_parse_time(stderr);
        assert_eq!(time, 0.0);
    }

    #[test]
    fn test_test_suite_creation() {
        let mut suite = TestSuite::new();
        assert!(suite.grammars.is_empty());

        let config = TestConfig {
            grammar_path: PathBuf::from("grammar.rs"),
            test_files: vec![],
            tree_sitter_path: None,
            compare_output: false,
            benchmark: false,
            external_scanner: None,
        };

        suite.add_grammar("test-lang".to_string(), config);
        assert_eq!(suite.grammars.len(), 1);
        assert_eq!(suite.grammars[0].0, "test-lang");
    }

    #[test]
    fn test_compatibility_report_calculations() {
        let report = CompatibilityReport {
            version: "1.0.0".to_string(),
            date: "2024-01-01".to_string(),
            total_grammars: 3,
            passed_grammars: 2,
            total_tests: 150,
            failed_tests: 10,
            overall_compatibility: ((150 - 10) as f64 / 150.0) * 100.0,
            average_speedup: 2.0,
            grammar_results: vec![],
        };

        assert_eq!(report.total_grammars, 3);
        assert_eq!(report.passed_grammars, 2);
        assert_eq!(report.total_tests, 150);
        assert_eq!(report.failed_tests, 10);
        assert!((report.overall_compatibility - 93.33).abs() < 0.01);
        assert_eq!(report.average_speedup, 2.0);
    }

    #[test]
    fn test_file_test_result() {
        let result = FileTestResult {
            file_path: PathBuf::from("test.js"),
            parse_time_ms: 5.5,
            output: "(program (expression))".to_string(),
        };

        assert_eq!(result.file_path, PathBuf::from("test.js"));
        assert_eq!(result.parse_time_ms, 5.5);
        assert_eq!(result.output, "(program (expression))");
    }

    #[test]
    fn test_report_json_serialization() {
        let report = CompatibilityReport {
            version: "0.5.0".to_string(),
            date: "2024-01-15T10:00:00Z".to_string(),
            total_grammars: 1,
            passed_grammars: 1,
            total_tests: 20,
            failed_tests: 0,
            overall_compatibility: 100.0,
            average_speedup: 3.0,
            grammar_results: vec![GrammarTestResult {
                name: "perfect-grammar".to_string(),
                version: "0.5.0".to_string(),
                passed: true,
                total_tests: 20,
                failed_tests: 0,
                parse_time_ms: 10.0,
                tree_sitter_time_ms: 30.0,
                speedup: 3.0,
                errors: vec![],
                compatibility_score: 100.0,
            }],
        };

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: CompatibilityReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, report.version);
        assert_eq!(deserialized.total_grammars, report.total_grammars);
        assert_eq!(
            deserialized.overall_compatibility,
            report.overall_compatibility
        );
        assert_eq!(deserialized.grammar_results.len(), 1);
        assert_eq!(deserialized.grammar_results[0].name, "perfect-grammar");
    }
}
