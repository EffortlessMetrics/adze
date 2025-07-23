// Code generation for LSP servers

use rust_sitter_ir::Grammar;
use crate::config::LspConfig;
use crate::features::LspFeature;
use anyhow::Result;

pub struct LspCodeGenerator<'a> {
    grammar: &'a Grammar,
    config: &'a LspConfig,
}

impl<'a> LspCodeGenerator<'a> {
    pub fn new(grammar: &'a Grammar, config: &'a LspConfig) -> Self {
        Self { grammar, config }
    }

    /// Generate the main server implementation
    pub fn generate_server(&self, features: &[Box<dyn LspFeature>]) -> Result<String> {
        let capabilities = self.generate_capabilities(features);
        
        Ok(format!(r#"// Generated LSP server for {}
use tower_lsp::{{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server}};
use tokio::{{sync::Mutex, io::{{AsyncReadExt, AsyncWriteExt}}}};
use std::sync::Arc;

#[derive(Debug)]
pub struct {} {{
    client: Client,
    state: Arc<Mutex<ServerState>>,
}}

#[derive(Debug, Default)]
struct ServerState {{
    documents: std::collections::HashMap<Url, String>,
}}

impl {} {{
    pub fn new(client: Client) -> Self {{
        Self {{
            client,
            state: Arc::new(Mutex::new(ServerState::default())),
        }}
    }}
}}

#[tower_lsp::async_trait]
impl LanguageServer for {} {{
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {{
        Ok(InitializeResult {{
            capabilities: ServerCapabilities {{
                {}
                ..Default::default()
            }},
            ..Default::default()
        }})
    }}

    async fn initialized(&self, _: InitializedParams) {{
        self.client
            .log_message(MessageType::INFO, "Language server initialized!")
            .await;
    }}

    async fn shutdown(&self) -> Result<()> {{
        Ok(())
    }}

    async fn did_open(&self, params: DidOpenTextDocumentParams) {{
        let mut state = self.state.lock().await;
        state.documents.insert(
            params.text_document.uri.clone(),
            params.text_document.text.clone(),
        );
        
        // Trigger diagnostics
        let diagnostics = crate::handlers::handle_diagnostics(
            params.text_document.uri.clone(),
            &params.text_document.text,
        ).await.unwrap_or_default();
        
        self.client.publish_diagnostics(
            params.text_document.uri,
            diagnostics,
            None,
        ).await;
    }}

    async fn did_change(&self, params: DidChangeTextDocumentParams) {{
        let mut state = self.state.lock().await;
        if let Some(doc) = state.documents.get_mut(&params.text_document.uri) {{
            // Apply changes (assuming full document sync)
            if let Some(change) = params.content_changes.into_iter().next() {{
                *doc = change.text;
            }}
        }}
        
        // Re-trigger diagnostics
        if let Some(text) = state.documents.get(&params.text_document.uri) {{
            let diagnostics = crate::handlers::handle_diagnostics(
                params.text_document.uri.clone(),
                text,
            ).await.unwrap_or_default();
            
            self.client.publish_diagnostics(
                params.text_document.uri,
                diagnostics,
                None,
            ).await;
        }}
    }}

    async fn did_close(&self, params: DidCloseTextDocumentParams) {{
        let mut state = self.state.lock().await;
        state.documents.remove(&params.text_document.uri);
    }}
}}"#,
            self.config.name,
            self.config.name.to_case(convert_case::Case::Pascal),
            self.config.name.to_case(convert_case::Case::Pascal),
            self.config.name.to_case(convert_case::Case::Pascal),
            capabilities
        ))
    }

    /// Generate handlers module
    pub fn generate_handlers(&self, features: &[Box<dyn LspFeature>]) -> Result<String> {
        let mut imports = vec![
            "use anyhow::Result;".to_string(),
            "use lsp_types::*;".to_string(),
        ];
        
        let mut handlers = Vec::new();
        
        for feature in features {
            imports.extend(feature.required_imports());
            handlers.push(feature.generate_handler());
        }
        
        Ok(format!(r#"// Generated handlers for LSP server
{}

{}
"#,
            imports.join("\n"),
            handlers.join("\n\n")
        ))
    }

    /// Generate Cargo.toml
    pub fn generate_cargo_toml(&self) -> Result<String> {
        Ok(format!(r#"[package]
name = "{}"
version = "{}"
edition = "2021"

[dependencies]
tower-lsp = "0.20"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
rust-sitter = {{ path = "../runtime" }}
{} = {{ path = "../grammars/{}" }}

[[bin]]
name = "{}-lsp"
path = "main.rs"
"#,
            self.config.name,
            self.config.version,
            self.grammar.name,
            self.grammar.name,
            self.config.name
        ))
    }

    /// Generate main.rs
    pub fn generate_main(&self) -> Result<String> {
        Ok(format!(r#"// Generated main entry point for LSP server
mod server;
mod handlers;

use tower_lsp::{{LspService, Server}};
use tracing_subscriber;

#[tokio::main]
async fn main() {{
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("{}={}")
        .init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {{
        server::{}::new(client)
    }});

    Server::new(stdin, stdout, socket).serve(service).await;
}}
"#,
            self.config.name,
            self.config.logging.level,
            self.config.name.to_case(convert_case::Case::Pascal)
        ))
    }

    fn generate_capabilities(&self, features: &[Box<dyn LspFeature>]) -> String {
        let mut capabilities = Vec::new();
        
        // Merge capabilities from all features
        for feature in features {
            let caps = feature.capabilities();
            if let Some(obj) = caps.as_object() {
                for (key, value) in obj {
                    capabilities.push(format!(
                        "{}: {},",
                        key,
                        serde_json::to_string(value).unwrap()
                    ));
                }
            }
        }
        
        capabilities.join("\n                ")
    }
}

// Helper trait for case conversion (simple implementation)
trait CaseConvert {
    fn to_case(&self, case: convert_case::Case) -> String;
}

impl CaseConvert for String {
    fn to_case(&self, case: convert_case::Case) -> String {
        match case {
            convert_case::Case::Pascal => {
                self.split('_')
                    .map(|s| {
                        let mut chars = s.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().chain(chars).collect(),
                        }
                    })
                    .collect()
            }
            _ => self.clone(),
        }
    }
}

mod convert_case {
    pub enum Case {
        Pascal,
    }
}