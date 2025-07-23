// Interactive grammar playground for rust-sitter
// Provides web-based and CLI interfaces for testing grammars

use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use rust_sitter_ir::{Grammar, Rule, Symbol, SymbolId};
use rust_sitter_glr_core::ParseTable;

pub mod web;
pub mod cli;
pub mod visualizer;
pub mod analyzer;

/// Playground session for interactive grammar testing
#[derive(Debug)]
pub struct PlaygroundSession {
    grammar: Grammar,
    parse_table: Option<ParseTable>,
    test_cases: Vec<TestCase>,
    analysis_cache: HashMap<String, AnalysisResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub input: String,
    pub expected_tree: Option<String>,
    pub should_pass: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub success: bool,
    pub tree: Option<String>,
    pub errors: Vec<ParseError>,
    pub timing: ParseTiming,
    pub visualization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseTiming {
    pub lexing_ms: f64,
    pub parsing_ms: f64,
    pub total_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub grammar_stats: GrammarStats,
    pub conflicts: Vec<Conflict>,
    pub ambiguities: Vec<Ambiguity>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarStats {
    pub rule_count: usize,
    pub terminal_count: usize,
    pub nonterminal_count: usize,
    pub max_rule_length: usize,
    pub avg_rule_length: f64,
    pub nullable_rules: usize,
    pub left_recursive_rules: usize,
    pub right_recursive_rules: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub kind: ConflictKind,
    pub state: usize,
    pub symbol: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictKind {
    ShiftReduce,
    ReduceReduce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ambiguity {
    pub rule: String,
    pub example: String,
    pub parse_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub level: SuggestionLevel,
    pub message: String,
    pub rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionLevel {
    Info,
    Warning,
    Error,
}

impl PlaygroundSession {
    /// Create a new playground session
    pub fn new(grammar: Grammar) -> Self {
        Self {
            grammar,
            parse_table: None,
            test_cases: Vec::new(),
            analysis_cache: HashMap::new(),
        }
    }

    /// Initialize the parse table
    pub fn initialize(&mut self) -> Result<()> {
        use rust_sitter_glr_core::{LR1Builder, FirstFollowSets};
        
        let first_follow = FirstFollowSets::compute(&self.grammar)?;
        let builder = LR1Builder::new(&self.grammar, &first_follow);
        self.parse_table = Some(builder.build()?);
        
        Ok(())
    }

    /// Parse input text
    pub fn parse(&self, input: &str) -> Result<ParseResult> {
        let start_time = std::time::Instant::now();
        
        // Placeholder for actual parsing
        let parse_time = start_time.elapsed();
        
        Ok(ParseResult {
            success: true,
            tree: Some(format!("(program {})", input)),
            errors: Vec::new(),
            timing: ParseTiming {
                lexing_ms: 0.5,
                parsing_ms: parse_time.as_secs_f64() * 1000.0,
                total_ms: parse_time.as_secs_f64() * 1000.0,
            },
            visualization: None,
        })
    }

    /// Add a test case
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }

    /// Run all test cases
    pub fn run_tests(&self) -> Vec<(TestCase, ParseResult)> {
        self.test_cases.iter()
            .map(|test| {
                let result = self.parse(&test.input).unwrap_or_else(|e| ParseResult {
                    success: false,
                    tree: None,
                    errors: vec![ParseError {
                        message: e.to_string(),
                        line: 0,
                        column: 0,
                        offset: 0,
                        length: 0,
                    }],
                    timing: ParseTiming {
                        lexing_ms: 0.0,
                        parsing_ms: 0.0,
                        total_ms: 0.0,
                    },
                    visualization: None,
                });
                (test.clone(), result)
            })
            .collect()
    }

    /// Analyze the grammar
    pub fn analyze_grammar(&mut self) -> Result<&AnalysisResult> {
        let cache_key = format!("{:?}", self.grammar);
        
        if !self.analysis_cache.contains_key(&cache_key) {
            let analysis = analyzer::analyze_grammar(&self.grammar)?;
            self.analysis_cache.insert(cache_key.clone(), analysis);
        }
        
        Ok(self.analysis_cache.get(&cache_key).unwrap())
    }

    /// Generate visualization
    pub fn visualize_tree(&self, tree: &str) -> Result<String> {
        visualizer::generate_tree_svg(tree)
    }

    /// Export session data
    pub fn export(&self) -> Result<String> {
        let data = PlaygroundExport {
            grammar_name: self.grammar.name.clone(),
            test_cases: self.test_cases.clone(),
            analysis: self.analysis_cache.values().cloned().collect(),
        };
        
        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// Import session data
    pub fn import(&mut self, data: &str) -> Result<()> {
        let export: PlaygroundExport = serde_json::from_str(data)?;
        self.test_cases = export.test_cases;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaygroundExport {
    grammar_name: String,
    test_cases: Vec<TestCase>,
    analysis: Vec<AnalysisResult>,
}

/// Interactive grammar playground builder
pub struct PlaygroundBuilder {
    grammar_path: Option<String>,
    test_file: Option<String>,
    output_dir: Option<String>,
    features: Vec<PlaygroundFeature>,
}

#[derive(Debug, Clone)]
pub enum PlaygroundFeature {
    WebInterface(u16), // port
    CliInterface,
    Visualization,
    Analysis,
    TestRunner,
}

impl PlaygroundBuilder {
    pub fn new() -> Self {
        Self {
            grammar_path: None,
            test_file: None,
            output_dir: None,
            features: Vec::new(),
        }
    }

    pub fn grammar(mut self, path: impl Into<String>) -> Self {
        self.grammar_path = Some(path.into());
        self
    }

    pub fn tests(mut self, path: impl Into<String>) -> Self {
        self.test_file = Some(path.into());
        self
    }

    pub fn output(mut self, dir: impl Into<String>) -> Self {
        self.output_dir = Some(dir.into());
        self
    }

    pub fn feature(mut self, feature: PlaygroundFeature) -> Self {
        self.features.push(feature);
        self
    }

    pub fn build(self) -> Result<()> {
        // Load grammar
        let grammar = self.load_grammar()?;
        let mut session = PlaygroundSession::new(grammar);
        session.initialize()?;

        // Load test cases if provided
        if let Some(test_file) = self.test_file {
            let tests = self.load_tests(&test_file)?;
            for test in tests {
                session.add_test_case(test);
            }
        }

        // Launch features
        for feature in self.features {
            match feature {
                PlaygroundFeature::WebInterface(port) => {
                    web::launch_server(session.clone(), port)?;
                }
                PlaygroundFeature::CliInterface => {
                    cli::run_interactive(session.clone())?;
                }
                PlaygroundFeature::Visualization => {
                    // Enable visualization in session
                }
                PlaygroundFeature::Analysis => {
                    let analysis = session.analyze_grammar()?;
                    println!("Grammar Analysis: {:?}", analysis);
                }
                PlaygroundFeature::TestRunner => {
                    let results = session.run_tests();
                    for (test, result) in results {
                        println!("{}: {}", test.name, if result.success { "PASS" } else { "FAIL" });
                    }
                }
            }
        }

        Ok(())
    }

    fn load_grammar(&self) -> Result<Grammar> {
        // Placeholder - would load from file
        Ok(Grammar::default())
    }

    fn load_tests(&self, path: &str) -> Result<Vec<TestCase>> {
        // Placeholder - would load from file
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playground_session() {
        let grammar = Grammar::default();
        let mut session = PlaygroundSession::new(grammar);
        
        session.add_test_case(TestCase {
            name: "simple".to_string(),
            input: "1 + 2".to_string(),
            expected_tree: Some("(expr (num 1) + (num 2))".to_string()),
            should_pass: true,
            tags: vec!["arithmetic".to_string()],
        });
        
        let results = session.run_tests();
        assert_eq!(results.len(), 1);
    }
}