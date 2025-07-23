// Configuration for LSP generation

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    /// Name of the language server
    pub name: String,
    
    /// Version of the language server
    pub version: String,
    
    /// Language ID (e.g., "rust", "javascript")
    pub language_id: String,
    
    /// File extensions (e.g., [".rs", ".rust"])
    pub file_extensions: Vec<String>,
    
    /// Server capabilities configuration
    pub capabilities: ServerCapabilities,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            name: "rust-sitter-lsp".to_string(),
            version: "0.1.0".to_string(),
            language_id: "unknown".to_string(),
            file_extensions: vec![],
            capabilities: ServerCapabilities::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Enable incremental sync
    pub incremental_sync: bool,
    
    /// Enable semantic tokens
    pub semantic_tokens: bool,
    
    /// Enable code actions
    pub code_actions: bool,
    
    /// Enable formatting
    pub formatting: bool,
    
    /// Enable goto definition
    pub goto_definition: bool,
    
    /// Enable find references
    pub find_references: bool,
    
    /// Enable rename
    pub rename: bool,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            incremental_sync: true,
            semantic_tokens: false,
            code_actions: false,
            formatting: false,
            goto_definition: false,
            find_references: false,
            rename: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    
    /// Log to file
    pub file: Option<String>,
    
    /// Log to stderr
    pub stderr: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            stderr: true,
        }
    }
}