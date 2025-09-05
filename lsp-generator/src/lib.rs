// LSP (Language Server Protocol) generator for rust-sitter
// Automatically generates language servers from rust-sitter grammars

use anyhow::{Context, Result};
use rust_sitter_ir::Grammar;
use std::fs;
use std::path::{Path, PathBuf};

pub mod codegen;
pub mod config;
pub mod features;

use codegen::LspCodeGenerator;
pub use config::LspConfig;
use features::{CompletionProvider, DiagnosticsProvider, HoverProvider, LspFeature};

/// Main LSP generator for rust-sitter grammars
pub struct LspGenerator {
    grammar: Grammar,
    config: LspConfig,
    features: Vec<Box<dyn LspFeature>>,
}

impl LspGenerator {
    /// Create a new LSP generator for a grammar
    pub fn new(grammar: Grammar) -> Self {
        Self {
            grammar,
            config: LspConfig::default(),
            features: Vec::new(),
        }
    }

    /// Configure the LSP generator
    pub fn with_config(mut self, config: LspConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable completion support
    pub fn with_completion(mut self) -> Self {
        self.features
            .push(Box::new(CompletionProvider::new(&self.grammar)));
        self
    }

    /// Enable hover support  
    pub fn with_hover(mut self) -> Self {
        self.features
            .push(Box::new(HoverProvider::new(&self.grammar)));
        self
    }

    /// Enable diagnostics support
    pub fn with_diagnostics(mut self) -> Self {
        self.features
            .push(Box::new(DiagnosticsProvider::new(&self.grammar)));
        self
    }

    /// Enable all features
    pub fn with_all_features(self) -> Self {
        self.with_completion().with_hover().with_diagnostics()
    }

    /// Generate the LSP server code
    pub fn generate(&self, output_dir: &Path) -> Result<()> {
        // Create output directory
        fs::create_dir_all(output_dir)?;

        // Generate main server code
        let generator = LspCodeGenerator::new(&self.grammar, &self.config);

        // Generate server.rs
        let server_code = generator.generate_server(&self.features)?;
        fs::write(output_dir.join("server.rs"), server_code)?;

        // Generate handlers.rs
        let handlers_code = generator.generate_handlers(&self.features)?;
        fs::write(output_dir.join("handlers.rs"), handlers_code)?;

        // Generate Cargo.toml
        let cargo_toml = generator.generate_cargo_toml()?;
        fs::write(output_dir.join("Cargo.toml"), cargo_toml)?;

        // Generate main.rs
        let main_code = generator.generate_main()?;
        fs::write(output_dir.join("main.rs"), main_code)?;

        Ok(())
    }
}

/// Builder API for generating LSP servers
pub struct LspBuilder {
    name: String,
    version: String,
    grammar_path: PathBuf,
    output_dir: PathBuf,
    features: Vec<String>,
}

impl LspBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            grammar_path: PathBuf::new(),
            output_dir: PathBuf::new(),
            features: Vec::new(),
        }
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn grammar_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.grammar_path = path.into();
        self
    }

    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    pub fn feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    pub fn build(self) -> Result<()> {
        // Load grammar
        let grammar = load_grammar(&self.grammar_path)?;

        // Create config
        let config = LspConfig {
            name: self.name,
            version: self.version,
            ..Default::default()
        };

        // Create generator
        let mut generator = LspGenerator::new(grammar).with_config(config);

        // Add requested features
        for feature in &self.features {
            match feature.as_str() {
                "completion" => generator = generator.with_completion(),
                "hover" => generator = generator.with_hover(),
                "diagnostics" => generator = generator.with_diagnostics(),
                "all" => generator = generator.with_all_features(),
                _ => eprintln!("Warning: Unknown feature: {}", feature),
            }
        }

        // Generate LSP server
        generator.generate(&self.output_dir)?;

        println!("✓ Generated LSP server at: {}", self.output_dir.display());
        Ok(())
    }
}

fn load_grammar(path: &Path) -> Result<Grammar> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read grammar file {}", path.display()))?;
    let grammar = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse grammar from {}", path.display()))?;
    Ok(grammar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use rust_sitter_ir::builder::GrammarBuilder;
    use tempfile::NamedTempFile;

    #[test]
    fn test_lsp_builder() {
        let builder = LspBuilder::new("my-language-server")
            .version("1.0.0")
            .grammar_path("path/to/grammar")
            .output_dir("output/lsp")
            .feature("completion")
            .feature("hover");

        // Test would verify builder configuration
        assert_eq!(builder.name, "my-language-server");
        assert_eq!(builder.version, "1.0.0");
        assert_eq!(builder.features.len(), 2);
    }

    #[test]
    fn test_lsp_generator_creation() {
        let grammar = Grammar::default();
        let generator = LspGenerator::new(grammar);

        assert!(generator.features.is_empty());
        assert_eq!(generator.config.name, "rust-sitter-lsp");
    }

    #[test]
    fn test_lsp_generator_with_config() {
        let grammar = Grammar::default();
        let config = LspConfig {
            name: "test-lsp".to_string(),
            version: "0.2.0".to_string(),
            ..Default::default()
        };

        let generator = LspGenerator::new(grammar).with_config(config);
        assert_eq!(generator.config.name, "test-lsp");
        assert_eq!(generator.config.version, "0.2.0");
    }

    #[test]
    fn test_lsp_builder_default_values() {
        let builder = LspBuilder::new("test");

        assert_eq!(builder.name, "test");
        assert_eq!(builder.version, "0.1.0");
        assert!(builder.grammar_path.as_os_str().is_empty());
        assert!(builder.output_dir.as_os_str().is_empty());
        assert!(builder.features.is_empty());
    }

    #[test]
    fn test_lsp_builder_fluent_api() {
        let builder = LspBuilder::new("lang-server")
            .version("2.0.0")
            .grammar_path("/path/to/grammar.rs")
            .output_dir("/path/to/output")
            .feature("completion")
            .feature("hover")
            .feature("diagnostics");

        assert_eq!(builder.name, "lang-server");
        assert_eq!(builder.version, "2.0.0");
        assert_eq!(builder.grammar_path, PathBuf::from("/path/to/grammar.rs"));
        assert_eq!(builder.output_dir, PathBuf::from("/path/to/output"));
        assert_eq!(builder.features, vec!["completion", "hover", "diagnostics"]);
    }

    #[test]
    fn test_lsp_generator_with_features() {
        let grammar = Grammar::default();
        let generator = LspGenerator::new(grammar.clone())
            .with_completion()
            .with_hover()
            .with_diagnostics();

        assert_eq!(generator.features.len(), 3);
    }

    #[test]
    fn test_lsp_generator_with_all_features() {
        let grammar = Grammar::default();
        let generator = LspGenerator::new(grammar).with_all_features();

        // with_all_features should add completion, hover, and diagnostics
        assert_eq!(generator.features.len(), 3);
    }

    #[test]
    fn test_lsp_builder_feature_recognition() {
        let features = vec!["completion", "hover", "diagnostics", "all", "unknown"];
        let builder = LspBuilder::new("test");

        // Test that all feature strings are accepted
        let mut b = builder;
        for feature in features {
            b = b.feature(feature);
        }

        assert_eq!(b.features.len(), 5);
        assert!(b.features.contains(&"completion".to_string()));
        assert!(b.features.contains(&"hover".to_string()));
        assert!(b.features.contains(&"diagnostics".to_string()));
        assert!(b.features.contains(&"all".to_string()));
        assert!(b.features.contains(&"unknown".to_string()));
    }

    #[test]
    fn test_load_grammar_from_file() -> Result<()> {
        let grammar = GrammarBuilder::new("test")
            .token("NUMBER", "[0-9]+")
            .rule("expr", vec!["NUMBER"])
            .start("expr")
            .build();

        let mut file = NamedTempFile::new()?;
        serde_json::to_writer(file.as_file_mut(), &grammar)?;

        let loaded = super::load_grammar(file.path())?;

        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.tokens.len(), 1);
        let start = loaded.start_symbol().expect("start symbol");
        assert_eq!(loaded.rules.get(&start).map(|r| r.len()), Some(1));
        Ok(())
    }

    #[test]
    fn test_hover_provider_creation() {
        let grammar = Grammar::default();
        let hover_provider = features::HoverProvider::new(&grammar);

        assert_eq!(hover_provider.name(), "hover");
        assert!(hover_provider.documentation.is_empty());
    }

    #[test]
    fn test_hover_provider_capabilities() {
        let grammar = Grammar::default();
        let hover_provider = features::HoverProvider::new(&grammar);
        let capabilities = hover_provider.capabilities();

        assert_eq!(capabilities["hoverProvider"], serde_json::json!(true));
    }

    #[test]
    fn test_hover_provider_handler_generation() {
        let grammar = Grammar::default();
        let hover_provider = features::HoverProvider::new(&grammar);
        let handler_code = hover_provider.generate_handler();

        // Verify the generated code contains essential components
        assert!(handler_code.contains("handle_hover"));
        assert!(handler_code.contains("get_word_at_position"));
        assert!(handler_code.contains("lookup_documentation"));

        // Verify it includes error handling
        assert!(handler_code.contains("Result"));
        assert!(handler_code.contains("Context"));

        // Verify it includes LSP types
        assert!(handler_code.contains("HoverParams"));
        assert!(handler_code.contains("Hover"));
        assert!(handler_code.contains("HoverContents"));
    }

    #[test]
    fn test_hover_provider_documentation_map() {
        let docs = features::HoverProvider::build_documentation_map();

        // Should have common programming language keywords
        assert!(!docs.is_empty());

        // Verify some key entries
        assert!(docs.contains(&("fn", "Declares a function")));
        assert!(docs.contains(&("let", "Declares a variable binding")));
        assert!(docs.contains(&("if", "Conditional expression")));
        assert!(docs.contains(&("String", "UTF-8 encoded, growable string type")));

        // Verify multiple languages are supported
        assert!(docs.contains(&("def", "Defines a function"))); // Python
        assert!(docs.contains(&("function", "Declares a function"))); // JavaScript
        assert!(docs.contains(&("class", "Declares a class"))); // General
    }

    #[test]
    fn test_hover_provider_format_entries() {
        let test_entries = vec![
            ("test1", "Test description 1"),
            ("test2", "Test description 2"),
        ];

        let formatted = features::HoverProvider::format_documentation_entries(&test_entries);
        let expected = "        (\"test1\", \"Test description 1\"),\n        (\"test2\", \"Test description 2\")";

        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_hover_provider_required_imports() {
        let grammar = Grammar::default();
        let hover_provider = features::HoverProvider::new(&grammar);
        let imports = hover_provider.required_imports();

        assert!(!imports.is_empty());
        assert!(imports[0].contains("HoverParams"));
        assert!(imports[0].contains("Hover"));
        assert!(imports[0].contains("HoverContents"));
        assert!(imports[0].contains("MarkedString"));
    }

    #[test]
    fn test_lsp_generator_with_hover() {
        let grammar = Grammar::default();
        let generator = LspGenerator::new(grammar).with_hover();

        assert_eq!(generator.features.len(), 1);
        assert_eq!(generator.features[0].name(), "hover");
    }

    #[test]
    fn test_hover_utf8_word_boundaries() {
        // Test that the generated handler code properly handles UTF-8
        let grammar = Grammar::default();
        let hover_provider = features::HoverProvider::new(&grammar);
        let handler_code = hover_provider.generate_handler();

        // Should use char-based iteration, not byte-based
        assert!(handler_code.contains("chars: Vec<char>"));
        assert!(handler_code.contains("line.chars()"));
        assert!(handler_code.contains("is_alphanumeric()"));
    }

    #[test]
    fn test_hover_error_handling_patterns() {
        let grammar = Grammar::default();
        let hover_provider = features::HoverProvider::new(&grammar);
        let handler_code = hover_provider.generate_handler();

        // Should have proper error handling for common cases
        assert!(handler_code.contains("line out of bounds"));
        assert!(handler_code.contains("invalid uri"));
        assert!(handler_code.contains("anyhow"));
    }

    #[test]
    fn test_lsp_builder_with_hover_feature() {
        let builder = LspBuilder::new("test-lsp").feature("hover");

        assert!(builder.features.contains(&"hover".to_string()));
    }

    #[test]
    fn test_multiple_features_including_hover() {
        let grammar = Grammar::default();
        let generator = LspGenerator::new(grammar)
            .with_completion()
            .with_hover()
            .with_diagnostics();

        assert_eq!(generator.features.len(), 3);
        assert!(generator.features.iter().any(|f| f.name() == "hover"));
        assert!(generator.features.iter().any(|f| f.name() == "completion"));
        assert!(generator.features.iter().any(|f| f.name() == "diagnostics"));
    }

    #[test]
    fn test_with_all_features_includes_hover() {
        let grammar = Grammar::default();
        let generator = LspGenerator::new(grammar).with_all_features();

        assert_eq!(generator.features.len(), 3);
        assert!(generator.features.iter().any(|f| f.name() == "hover"));
    }
}
