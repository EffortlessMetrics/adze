// LSP feature implementations for rust-sitter grammars

use rust_sitter_ir::{Grammar, TokenPattern};

/// Trait for LSP features
pub trait LspFeature: Send + Sync {
    /// Get the name of this feature
    fn name(&self) -> &str;

    /// Generate handler code for this feature
    fn generate_handler(&self) -> String;

    /// Get required imports for this feature
    fn required_imports(&self) -> Vec<String>;

    /// Get capabilities for this feature
    fn capabilities(&self) -> serde_json::Value;
}

/// Completion provider for LSP
pub struct CompletionProvider {
    keywords: Vec<String>,
    symbols: Vec<String>,
}

impl CompletionProvider {
    pub fn new(grammar: &Grammar) -> Self {
        let mut keywords = Vec::new();
        let mut symbols = Vec::new();

        // Extract keywords from tokens
        for (_id, token) in &grammar.tokens {
            match &token.pattern {
                TokenPattern::String(value) => {
                    if value.chars().all(|c| c.is_alphabetic() || c == '_') {
                        keywords.push(value.clone());
                    }
                }
                _ => {}
            }
        }

        // Extract symbols from rule names
        for (_symbol_id, name) in &grammar.rule_names {
            symbols.push(name.clone());
        }

        Self { keywords, symbols }
    }
}

impl LspFeature for CompletionProvider {
    fn name(&self) -> &str {
        "completion"
    }

    fn generate_handler(&self) -> String {
        format!(
            r#"
pub async fn handle_completion(
    params: lsp_types::CompletionParams,
) -> Result<Option<lsp_types::CompletionResponse>> {{
    let items = vec![
        {}
    ];
    
    Ok(Some(lsp_types::CompletionResponse::Array(items)))
}}

fn create_keyword_completions() -> Vec<lsp_types::CompletionItem> {{
    vec![
        {}
    ]
}}

fn create_symbol_completions() -> Vec<lsp_types::CompletionItem> {{
    vec![
        {}
    ]
}}"#,
            // Keywords completion items
            self.keywords
                .iter()
                .map(|k| format!(
                    r#"lsp_types::CompletionItem {{
                        label: "{}".to_string(),
                        kind: Some(lsp_types::CompletionItemKind::KEYWORD),
                        ..Default::default()
                    }}"#,
                    k
                ))
                .collect::<Vec<_>>()
                .join(",\n        "),
            // Keyword function
            self.keywords
                .iter()
                .map(|k| format!(
                    r#"lsp_types::CompletionItem {{
                        label: "{}".to_string(),
                        kind: Some(lsp_types::CompletionItemKind::KEYWORD),
                        ..Default::default()
                    }}"#,
                    k
                ))
                .collect::<Vec<_>>()
                .join(",\n        "),
            // Symbol function
            self.symbols
                .iter()
                .map(|s| format!(
                    r#"lsp_types::CompletionItem {{
                        label: "{}".to_string(),
                        kind: Some(lsp_types::CompletionItemKind::CLASS),
                        ..Default::default()
                    }}"#,
                    s
                ))
                .collect::<Vec<_>>()
                .join(",\n        ")
        )
    }

    fn required_imports(&self) -> Vec<String> {
        vec![
            "use lsp_types::{CompletionParams, CompletionResponse, CompletionItem, CompletionItemKind};".to_string()
        ]
    }

    fn capabilities(&self) -> serde_json::Value {
        serde_json::json!({
            "completionProvider": {
                "resolveProvider": false,
                "triggerCharacters": [".", ":"]
            }
        })
    }
}

/// Hover provider for LSP
pub struct HoverProvider {
    #[allow(dead_code)]
    documentation: std::collections::HashMap<String, String>,
}

impl HoverProvider {
    pub fn new(grammar: &Grammar) -> Self {
        let mut documentation = std::collections::HashMap::new();

        // Generate documentation from grammar rules
        for (_symbol_id, rule_name) in &grammar.rule_names {
            let doc = format!("Grammar rule: {}", rule_name);
            documentation.insert(rule_name.clone(), doc);
        }

        Self { documentation }
    }
}

impl LspFeature for HoverProvider {
    fn name(&self) -> &str {
        "hover"
    }

    fn generate_handler(&self) -> String {
        r#"
pub async fn handle_hover(
    params: lsp_types::HoverParams,
) -> Result<Option<lsp_types::Hover>> {
    // Get the word under cursor
    let word = get_word_at_position(&params)?;
    
    // Look up documentation
    let contents = match lookup_documentation(&word) {
        Some(doc) => lsp_types::HoverContents::Scalar(
            lsp_types::MarkedString::String(doc)
        ),
        None => return Ok(None),
    };
    
    Ok(Some(lsp_types::Hover {
        contents,
        range: None,
    }))
}

fn get_word_at_position(params: &lsp_types::HoverParams) -> Result<String> {
    // Implementation would extract word at cursor position
    todo!("Extract word at position")
}

fn lookup_documentation(word: &str) -> Option<String> {
    // Implementation would look up documentation
    todo!("Look up documentation for word")
}"#
        .to_string()
    }

    fn required_imports(&self) -> Vec<String> {
        vec!["use lsp_types::{HoverParams, Hover, HoverContents, MarkedString};".to_string()]
    }

    fn capabilities(&self) -> serde_json::Value {
        serde_json::json!({
            "hoverProvider": true
        })
    }
}

/// Diagnostics provider for LSP
pub struct DiagnosticsProvider {
    grammar_name: String,
}

impl DiagnosticsProvider {
    pub fn new(grammar: &Grammar) -> Self {
        Self {
            grammar_name: grammar.name.clone(),
        }
    }
}

impl LspFeature for DiagnosticsProvider {
    fn name(&self) -> &str {
        "diagnostics"
    }

    fn generate_handler(&self) -> String {
        format!(
            r#"
pub async fn handle_diagnostics(
    uri: lsp_types::Url,
    text: &str,
) -> Result<Vec<lsp_types::Diagnostic>> {{
    let mut diagnostics = Vec::new();
    
    // Parse the text
    match {}::parse(text) {{
        Ok(_ast) => {{
            // No syntax errors
        }}
        Err(errors) => {{
            for error in errors {{
                diagnostics.push(lsp_types::Diagnostic {{
                    range: lsp_types::Range {{
                        start: offset_to_position(text, error.start),
                        end: offset_to_position(text, error.end),
                    }},
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("rust-sitter".to_string()),
                    message: error.message,
                    related_information: None,
                    tags: None,
                    data: None,
                }});
            }}
        }}
    }}
    
    Ok(diagnostics)
}}

fn offset_to_position(text: &str, offset: usize) -> lsp_types::Position {{
    let mut line = 0;
    let mut character = 0;
    
    for (i, ch) in text.char_indices() {{
        if i >= offset {{
            break;
        }}
        if ch == '\n' {{
            line += 1;
            character = 0;
        }} else {{
            character += 1;
        }}
    }}
    
    lsp_types::Position {{ line, character }}
}}"#,
            self.grammar_name
        )
    }

    fn required_imports(&self) -> Vec<String> {
        vec!["use lsp_types::{Diagnostic, DiagnosticSeverity, Range, Position, Url};".to_string()]
    }

    fn capabilities(&self) -> serde_json::Value {
        serde_json::json!({
            "textDocumentSync": {
                "openClose": true,
                "change": 1,  // Full document sync
                "save": true
            }
        })
    }
}
