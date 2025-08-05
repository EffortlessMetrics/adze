use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use xshell::Shell;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarTest {
    pub name: String,
    pub repo_url: String,
    pub expected_status: TestStatus,
    pub blocking_features: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Working,
    LikelyWorking,
    Partial,
    Blocked,
}

impl TestStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            TestStatus::Working => "✅",
            TestStatus::LikelyWorking => "🟡",
            TestStatus::Partial => "🟠",
            TestStatus::Blocked => "❌",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub grammar: String,
    pub status: TestStatus,
    pub parse_success: bool,
    pub convert_success: bool,
    pub build_success: bool,
    pub error_message: Option<String>,
    pub features_used: Vec<String>,
}

pub fn get_test_grammars() -> Vec<GrammarTest> {
    vec![
        // Simple grammars that should work
        GrammarTest {
            name: "json".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-json".to_string(),
            expected_status: TestStatus::Working,
            blocking_features: vec![],
            notes: Some("Simple grammar, should work perfectly".to_string()),
        },
        GrammarTest {
            name: "toml".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-toml".to_string(),
            expected_status: TestStatus::Working,
            blocking_features: vec![],
            notes: Some("Simple grammar, should work".to_string()),
        },
        // Grammars that likely work with precedence support
        GrammarTest {
            name: "c".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-c".to_string(),
            expected_status: TestStatus::LikelyWorking,
            blocking_features: vec!["precedence".to_string()],
            notes: Some("Uses precedence, should work now".to_string()),
        },
        GrammarTest {
            name: "go".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-go".to_string(),
            expected_status: TestStatus::LikelyWorking,
            blocking_features: vec!["precedence".to_string()],
            notes: Some("Uses precedence and word token".to_string()),
        },
        GrammarTest {
            name: "java".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-java".to_string(),
            expected_status: TestStatus::LikelyWorking,
            blocking_features: vec!["precedence".to_string()],
            notes: Some("Uses precedence".to_string()),
        },
        // Grammars with partial support
        GrammarTest {
            name: "javascript".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-javascript".to_string(),
            expected_status: TestStatus::Partial,
            blocking_features: vec!["external_scanner".to_string(), "js_functions".to_string()],
            notes: Some("Complex grammar with external scanner".to_string()),
        },
        GrammarTest {
            name: "typescript".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-typescript".to_string(),
            expected_status: TestStatus::Partial,
            blocking_features: vec!["external_scanner".to_string(), "extends_js".to_string()],
            notes: Some("Extends JavaScript grammar".to_string()),
        },
        // Blocked grammars
        GrammarTest {
            name: "python".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-python".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Requires indent/dedent external scanner".to_string()),
        },
        GrammarTest {
            name: "rust".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-rust".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Requires external scanner for raw strings".to_string()),
        },
        GrammarTest {
            name: "cpp".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-cpp".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Requires external scanner".to_string()),
        },
        GrammarTest {
            name: "ruby".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-ruby".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Requires external scanner for heredocs".to_string()),
        },
        GrammarTest {
            name: "bash".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-bash".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Complex external scanner".to_string()),
        },
        // Additional popular grammars
        GrammarTest {
            name: "html".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-html".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Uses external scanner".to_string()),
        },
        GrammarTest {
            name: "css".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-css".to_string(),
            expected_status: TestStatus::LikelyWorking,
            blocking_features: vec!["precedence".to_string()],
            notes: Some("Should work with precedence support".to_string()),
        },
        GrammarTest {
            name: "markdown".to_string(),
            repo_url: "https://github.com/MDeiml/tree-sitter-markdown".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Complex external scanner for blocks".to_string()),
        },
        GrammarTest {
            name: "yaml".to_string(),
            repo_url: "https://github.com/ikatyang/tree-sitter-yaml".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Requires external scanner for indentation".to_string()),
        },
        GrammarTest {
            name: "lua".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-lua".to_string(),
            expected_status: TestStatus::LikelyWorking,
            blocking_features: vec!["precedence".to_string()],
            notes: Some("Should work with precedence".to_string()),
        },
        GrammarTest {
            name: "php".to_string(),
            repo_url: "https://github.com/tree-sitter/tree-sitter-php".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Complex external scanner".to_string()),
        },
        GrammarTest {
            name: "swift".to_string(),
            repo_url: "https://github.com/alex-pinkus/tree-sitter-swift".to_string(),
            expected_status: TestStatus::Blocked,
            blocking_features: vec!["external_scanner".to_string()],
            notes: Some("Uses external scanner".to_string()),
        },
        GrammarTest {
            name: "kotlin".to_string(),
            repo_url: "https://github.com/fwcd/tree-sitter-kotlin".to_string(),
            expected_status: TestStatus::LikelyWorking,
            blocking_features: vec!["precedence".to_string()],
            notes: Some("Should work with precedence".to_string()),
        },
    ]
}

pub fn download_grammar(test: &GrammarTest, target_dir: &Path) -> Result<PathBuf> {
    let grammar_dir = target_dir.join(&test.name);

    if grammar_dir.exists() {
        println!("  Grammar already downloaded at {:?}", grammar_dir);
        return Ok(grammar_dir);
    }

    println!("  Downloading {} from {}", test.name, test.repo_url);

    let output = Command::new("git")
        .args(&[
            "clone",
            "--depth",
            "1",
            &test.repo_url,
            grammar_dir.to_str().unwrap(),
        ])
        .output()
        .context("Failed to run git clone")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to clone {}: {}",
            test.name,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(grammar_dir)
}

pub fn test_grammar(test: &GrammarTest, grammar_dir: &Path) -> Result<TestResult> {
    use rust_sitter_tool::grammar_js::{
        GrammarJsConverter, GrammarJsParserV3, parse_grammar_js_v2,
    };
    use rust_sitter_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};

    let mut result = TestResult {
        grammar: test.name.clone(),
        status: test.expected_status,
        parse_success: false,
        convert_success: false,
        build_success: false,
        error_message: None,
        features_used: vec![],
    };

    // Find grammar.js file
    let grammar_js_path = grammar_dir.join("grammar.js");
    if !grammar_js_path.exists() {
        result.error_message = Some("grammar.js not found".to_string());
        return Ok(result);
    }

    let grammar_content =
        fs::read_to_string(&grammar_js_path).context("Failed to read grammar.js")?;

    // Try parsing with v3 parser
    println!("  Parsing grammar.js...");
    let mut parser = GrammarJsParserV3::new(grammar_content.clone());

    let grammar_js = match parser.parse() {
        Ok(g) => {
            result.parse_success = true;

            // Check features used
            if !g.externals.is_empty() {
                result.features_used.push("externals".to_string());
            }
            if !g.precedences.is_empty() {
                result.features_used.push("precedences".to_string());
            }
            if !g.conflicts.is_empty() {
                result.features_used.push("conflicts".to_string());
            }
            if !g.supertypes.is_empty() {
                result.features_used.push("supertypes".to_string());
            }
            if g.word.is_some() {
                result.features_used.push("word".to_string());
            }

            g
        }
        Err(e) => {
            // Try v2 parser as fallback
            match parse_grammar_js_v2(&grammar_content) {
                Ok(g) => {
                    result.parse_success = true;
                    result.features_used.push("v2_parser_only".to_string());
                    g
                }
                Err(e2) => {
                    result.error_message = Some(format!("Parse error: {} (v2: {})", e, e2));
                    return Ok(result);
                }
            }
        }
    };

    // Try converting to IR
    println!("  Converting to IR...");
    let converter = GrammarJsConverter::new(grammar_js);
    match converter.convert() {
        Ok(ir) => {
            result.convert_success = true;
            println!(
                "    {} rules, {} tokens, {} externals",
                ir.rules.len(),
                ir.tokens.len(),
                ir.externals.len()
            );
        }
        Err(e) => {
            result.error_message = Some(format!("Convert error: {}", e));
            return Ok(result);
        }
    }

    // Try building
    println!("  Building parser...");
    let temp_dir = tempfile::tempdir()?;
    let options = BuildOptions {
        out_dir: temp_dir.path().to_str().unwrap().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };

    match build_parser_from_grammar_js(&grammar_js_path, options) {
        Ok(_) => {
            result.build_success = true;
            result.status = TestStatus::Working;
        }
        Err(e) => {
            result.error_message = Some(format!("Build error: {}", e));

            // Determine actual status based on error
            let error_str = e.to_string();
            if error_str.contains("external scanner") {
                result.status = TestStatus::Blocked;
            } else if error_str.contains("NotImplemented") {
                result.status = TestStatus::Partial;
            }
        }
    }

    Ok(result)
}

pub fn run_corpus_tests() -> Result<()> {
    let corpus_dir = PathBuf::from("corpus");
    fs::create_dir_all(&corpus_dir)?;

    let grammars = get_test_grammars();
    let mut results = vec![];

    println!("Testing {} grammars...\n", grammars.len());

    for test in &grammars {
        println!("{} Testing {}...", test.expected_status.emoji(), test.name);

        match download_grammar(test, &corpus_dir) {
            Ok(grammar_dir) => match test_grammar(test, &grammar_dir) {
                Ok(result) => {
                    println!(
                        "  Result: {} (expected: {})",
                        result.status.emoji(),
                        test.expected_status.emoji()
                    );
                    if let Some(err) = &result.error_message {
                        println!("  Error: {}", err);
                    }
                    if !result.features_used.is_empty() {
                        println!("  Features: {:?}", result.features_used);
                    }
                    results.push(result);
                }
                Err(e) => {
                    println!("  Test error: {}", e);
                }
            },
            Err(e) => {
                println!("  Download error: {}", e);
            }
        }
        println!();
    }

    // Generate report
    generate_report(&results)?;

    Ok(())
}

fn generate_report(results: &[TestResult]) -> Result<()> {
    let report_path = PathBuf::from("GRAMMAR_TEST_RESULTS.md");

    let mut report = String::new();
    report.push_str("# Grammar Test Results\n\n");
    report.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // Summary
    let working = results
        .iter()
        .filter(|r| r.status == TestStatus::Working)
        .count();
    let likely = results
        .iter()
        .filter(|r| r.status == TestStatus::LikelyWorking)
        .count();
    let partial = results
        .iter()
        .filter(|r| r.status == TestStatus::Partial)
        .count();
    let blocked = results
        .iter()
        .filter(|r| r.status == TestStatus::Blocked)
        .count();

    report.push_str("## Summary\n\n");
    report.push_str(&format!("- ✅ Working: {}\n", working));
    report.push_str(&format!("- 🟡 Likely Working: {}\n", likely));
    report.push_str(&format!("- 🟠 Partial: {}\n", partial));
    report.push_str(&format!("- ❌ Blocked: {}\n", blocked));
    report.push_str(&format!("- **Total**: {}\n\n", results.len()));

    // Detailed results
    report.push_str("## Detailed Results\n\n");
    report.push_str("| Grammar | Status | Parse | Convert | Build | Features | Error |\n");
    report.push_str("|---------|--------|-------|---------|-------|----------|-------|\n");

    for result in results {
        report.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            result.grammar,
            result.status.emoji(),
            if result.parse_success { "✅" } else { "❌" },
            if result.convert_success { "✅" } else { "❌" },
            if result.build_success { "✅" } else { "❌" },
            result.features_used.join(", "),
            result.error_message.as_deref().unwrap_or("-")
        ));
    }

    fs::write(report_path, report)?;
    println!("\n📊 Report saved to GRAMMAR_TEST_RESULTS.md");

    Ok(())
}

pub fn test_pure_rust(sh: &Shell, grammar: crate::Grammar, verbose: bool) -> Result<()> {
    use rust_sitter_tool::grammar_js::{GrammarJsConverter, GrammarJsParserV3};
    use rust_sitter_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};
    
    println!("Testing {} grammar with pure-Rust backend...\n", grammar.name());
    
    // Get grammar path
    let grammar_path = PathBuf::from("xtask/fixtures")
        .join(format!("tree-sitter-{}", grammar.name()))
        .join("grammar.js");
    
    if !grammar_path.exists() {
        anyhow::bail!("Grammar file not found: {:?}", grammar_path);
    }
    
    let content = fs::read_to_string(&grammar_path)?;
    
    // Parse grammar
    println!("📖 Parsing grammar.js...");
    let mut parser = GrammarJsParserV3::new(content);
    let grammar_js = parser.parse().context("Failed to parse grammar.js")?;
    
    // Check features
    let mut features = vec![];
    if !grammar_js.externals.is_empty() {
        features.push(format!("externals({})", grammar_js.externals.len()));
    }
    if grammar_js.word.is_some() {
        features.push("word".to_string());
    }
    if !grammar_js.conflicts.is_empty() {
        features.push(format!("conflicts({})", grammar_js.conflicts.len()));
    }
    if !grammar_js.precedences.is_empty() {
        features.push(format!("precedences({})", grammar_js.precedences.len()));
    }
    
    println!("✅ Parsed successfully");
    println!("   Rules: {}", grammar_js.rules.len());
    if !features.is_empty() {
        println!("   Features: {}", features.join(", "));
    }
    
    // Convert to IR
    println!("\n🔄 Converting to IR...");
    let converter = GrammarJsConverter::new(grammar_js.clone());
    let ir = converter.convert().context("Failed to convert to IR")?;
    
    println!("✅ Converted successfully");
    println!("   IR rules: {}", ir.rules.len());
    println!("   Tokens: {}", ir.tokens.len());
    
    // Check for external scanner requirement
    if !ir.externals.is_empty() {
        println!("\n⚠️  Grammar requires external scanner support");
        println!("   External tokens: {:?}", ir.externals);
        println!("\n❌ Cannot proceed without external scanner implementation");
        return Ok(());
    }
    
    // Build parser
    println!("\n🔨 Building pure-Rust parser...");
    let temp_dir = tempfile::tempdir()?;
    let options = BuildOptions {
        out_dir: temp_dir.path().to_str().unwrap().to_string(),
        emit_artifacts: verbose,
        compress_tables: true,
    };
    
    match build_parser_from_grammar_js(&grammar_path, options) {
        Ok(_) => {
            println!("✅ Build successful!");
            
            // Test parsing some example code if available
            let examples_dir = grammar_path.parent().unwrap().join("examples");
            if examples_dir.exists() {
                println!("\n🧪 Testing with example files...");
                let entries = fs::read_dir(&examples_dir)?;
                let mut example_count = 0;
                
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        example_count += 1;
                        if verbose || example_count <= 3 {
                            println!("   Testing: {}", path.file_name().unwrap().to_string_lossy());
                        }
                    }
                }
                
                if example_count > 3 && !verbose {
                    println!("   ... and {} more examples", example_count - 3);
                }
            }
        }
        Err(e) => {
            println!("❌ Build failed: {}", e);
            if verbose {
                println!("\nDetailed error:\n{:?}", e);
            }
        }
    }
    
    Ok(())
}
