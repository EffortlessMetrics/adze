// Configuration for LSP generation

use serde::{Deserialize, Serialize};

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
            name: "adze-lsp".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_config_default() {
        let config = LspConfig::default();

        assert_eq!(config.name, "adze-lsp");
        assert_eq!(config.version, "0.1.0");
        assert_eq!(config.language_id, "unknown");
        assert!(config.file_extensions.is_empty());
    }

    #[test]
    fn test_server_capabilities_default() {
        let capabilities = ServerCapabilities::default();

        assert!(capabilities.incremental_sync);
        assert!(!capabilities.semantic_tokens);
        assert!(!capabilities.code_actions);
        assert!(!capabilities.formatting);
        assert!(!capabilities.goto_definition);
        assert!(!capabilities.find_references);
        assert!(!capabilities.rename);
    }

    #[test]
    fn test_logging_config_default() {
        let logging = LoggingConfig::default();

        assert_eq!(logging.level, "info");
        assert!(logging.file.is_none());
        assert!(logging.stderr);
    }

    #[test]
    fn test_lsp_config_serialization() {
        let config = LspConfig {
            name: "my-lsp".to_string(),
            version: "1.0.0".to_string(),
            language_id: "mylang".to_string(),
            file_extensions: vec![".ml".to_string(), ".mli".to_string()],
            capabilities: ServerCapabilities::default(),
            logging: LoggingConfig::default(),
        };

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"name\":\"my-lsp\""));
        assert!(json.contains("\"language_id\":\"mylang\""));

        // Test deserialization
        let deserialized: LspConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.file_extensions, config.file_extensions);
    }

    #[test]
    fn test_server_capabilities_custom() {
        let capabilities = ServerCapabilities {
            incremental_sync: false,
            semantic_tokens: true,
            code_actions: true,
            formatting: true,
            goto_definition: true,
            find_references: true,
            rename: true,
        };

        assert!(!capabilities.incremental_sync);
        assert!(capabilities.semantic_tokens);
        assert!(capabilities.code_actions);
        assert!(capabilities.formatting);
        assert!(capabilities.goto_definition);
        assert!(capabilities.find_references);
        assert!(capabilities.rename);
    }

    #[test]
    fn test_logging_config_with_file() {
        let logging = LoggingConfig {
            level: "debug".to_string(),
            file: Some("/var/log/lsp.log".to_string()),
            stderr: false,
        };

        assert_eq!(logging.level, "debug");
        assert_eq!(logging.file, Some("/var/log/lsp.log".to_string()));
        assert!(!logging.stderr);
    }
}
