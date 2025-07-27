// LSP (Language Server Protocol) generator for rust-sitter
// Automatically generates language servers from rust-sitter grammars

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use rust_sitter_ir::Grammar;

pub mod codegen;
pub mod features;
pub mod config;

pub use config::LspConfig;
use codegen::LspCodeGenerator;
use features::{LspFeature, CompletionProvider, HoverProvider, DiagnosticsProvider};

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
        self.features.push(Box::new(CompletionProvider::new(&self.grammar)));
        self
    }

    /// Enable hover support  
    pub fn with_hover(mut self) -> Self {
        self.features.push(Box::new(HoverProvider::new(&self.grammar)));
        self
    }

    /// Enable diagnostics support
    pub fn with_diagnostics(mut self) -> Self {
        self.features.push(Box::new(DiagnosticsProvider::new(&self.grammar)));
        self
    }

    /// Enable all features
    pub fn with_all_features(self) -> Self {
        self.with_completion()
            .with_hover()
            .with_diagnostics()
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

fn load_grammar(_path: &Path) -> Result<Grammar> {
    // This would load the grammar from the compiled rust-sitter grammar
    // For now, return a placeholder
    todo!("Implement grammar loading from compiled rust-sitter parser")
}

#[cfg(test)]
mod tests {
    use super::*;

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
}