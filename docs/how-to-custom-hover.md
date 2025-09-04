# How-To: Implement Custom Hover Providers

This guide shows you how to extend rust-sitter's LSP generator with custom hover functionality for domain-specific languages or specialized documentation needs.

## Problem: Default Hover Not Sufficient

The built-in hover provider includes 45+ common language constructs, but you may need:
- Domain-specific terminology
- Custom documentation format
- Integration with external documentation systems
- Context-aware help based on grammar rules

## Solution: Custom HoverProvider Implementation

### Step 1: Extend the Documentation Database

Create a custom hover provider that extends the built-in functionality:

```rust
use rust_sitter_lsp_generator::features::HoverProvider;
use std::collections::HashMap;

pub struct CustomHoverProvider {
    base_provider: HoverProvider,
    custom_docs: HashMap<String, String>,
    grammar_docs: HashMap<String, String>,
}

impl CustomHoverProvider {
    pub fn new(grammar: &Grammar) -> Self {
        let base_provider = HoverProvider::new(grammar);
        let mut custom_docs = HashMap::new();
        let mut grammar_docs = HashMap::new();
        
        // Add domain-specific documentation
        custom_docs.insert("database".to_string(), 
            "**database**: A collection of organized data that can be queried".to_string());
        custom_docs.insert("migration".to_string(),
            "**migration**: A script that modifies database schema".to_string());
        custom_docs.insert("model".to_string(),
            "**model**: A data structure representing a database table".to_string());
        
        // Extract documentation from grammar rules
        for (_symbol_id, rule_name) in &grammar.rule_names {
            let doc = format!("**{}**: Grammar rule from {} language", 
                rule_name, grammar.name);
            grammar_docs.insert(rule_name.clone(), doc);
        }
        
        Self {
            base_provider,
            custom_docs,
            grammar_docs,
        }
    }
    
    pub fn lookup_documentation(&self, word: &str) -> Option<String> {
        // Try custom docs first
        if let Some(doc) = self.custom_docs.get(word) {
            return Some(doc.clone());
        }
        
        // Try grammar-specific docs
        if let Some(doc) = self.grammar_docs.get(word) {
            return Some(doc.clone());
        }
        
        // Fall back to built-in documentation
        self.base_provider.lookup_documentation(word)
    }
}
```

### Step 2: Implement Context-Aware Documentation

Add context awareness based on cursor position and surrounding code:

```rust
use rust_sitter::{Tree, Node};
use lsp_types::{Position, HoverParams};

impl CustomHoverProvider {
    pub fn get_contextual_documentation(
        &self,
        word: &str,
        tree: &Tree,
        position: Position,
    ) -> Option<String> {
        let node = tree.root_node().named_descendant_for_byte_range(
            position.line as usize * 100 + position.character as usize,
            position.line as usize * 100 + position.character as usize + word.len()
        );
        
        if let Some(node) = node {
            match node.kind() {
                "function_declaration" => {
                    Some(format!("**{}**: Function in {} context", word, node.kind()))
                }
                "type_declaration" => {
                    Some(format!("**{}**: Type definition", word))
                }
                "variable_declaration" => {
                    Some(format!("**{}**: Variable declaration", word))
                }
                _ => self.lookup_documentation(word)
            }
        } else {
            self.lookup_documentation(word)
        }
    }
}
```

### Step 3: Integrate with External Documentation

Connect to external documentation systems like docs.rs, MDN, or custom APIs:

```rust
use reqwest;
use serde_json::Value;

impl CustomHoverProvider {
    pub async fn fetch_external_docs(&self, word: &str) -> Option<String> {
        // Example: Fetch from docs.rs for Rust crates
        if word.chars().all(|c| c.is_lowercase() || c == '_') {
            let url = format!("https://docs.rs/{}/latest/search.json?query={}", word, word);
            
            if let Ok(response) = reqwest::get(&url).await {
                if let Ok(json) = response.json::<Value>().await {
                    if let Some(description) = json["results"][0]["description"].as_str() {
                        return Some(format!("**{}**: {} (from docs.rs)", word, description));
                    }
                }
            }
        }
        
        None
    }
    
    pub async fn get_documentation_with_fallback(&self, word: &str) -> Option<String> {
        // Try external docs first
        if let Some(external_doc) = self.fetch_external_docs(word).await {
            return Some(external_doc);
        }
        
        // Fall back to local documentation
        self.lookup_documentation(word)
    }
}
```

### Step 4: Custom Handler Generation

Override the handler generation to use your custom logic:

```rust
use rust_sitter_lsp_generator::LspFeature;

impl LspFeature for CustomHoverProvider {
    fn name(&self) -> &str {
        "custom_hover"
    }
    
    fn generate_handler(&self) -> String {
        format!(r#"
use anyhow::{{Context, Result}};
use lsp_types::{{HoverParams, Hover, HoverContents, MarkedString}};
use std::collections::HashMap;

pub async fn handle_hover(params: HoverParams) -> Result<Option<Hover>> {{
    let word = get_word_at_position(&params)?;
    
    // Try context-aware documentation first
    let tree = parse_document(&params.text_document_position_params.text_document)?;
    let contents = if let Some(doc) = get_contextual_documentation(
        &word, &tree, params.text_document_position_params.position
    ) {{
        HoverContents::Scalar(MarkedString::String(doc))
    }} else if let Some(doc) = get_documentation_with_fallback(&word).await {{
        HoverContents::Scalar(MarkedString::String(doc))
    }} else {{
        return Ok(None);
    }};
    
    Ok(Some(Hover {{
        contents,
        range: None,
    }}))
}}

// Custom documentation lookup functions
{}

// Word extraction function  
{}
"#, 
            self.generate_lookup_functions(),
            self.generate_word_extraction()
        )
    }
    
    fn required_imports(&self) -> Vec<String> {
        vec![
            "use lsp_types::{HoverParams, Hover, HoverContents, MarkedString};".to_string(),
            "use anyhow::{Context, Result};".to_string(),
            "use reqwest;".to_string(),
            "use serde_json::Value;".to_string(),
        ]
    }
    
    fn capabilities(&self) -> serde_json::Value {
        serde_json::json!({
            "hoverProvider": true
        })
    }
}

impl CustomHoverProvider {
    fn generate_lookup_functions(&self) -> String {
        let custom_entries: Vec<String> = self.custom_docs
            .iter()
            .map(|(key, value)| format!("        (\"{}\", \"{}\")", key, value))
            .collect();
            
        format!(r#"
fn get_contextual_documentation(word: &str, tree: &Tree, position: Position) -> Option<String> {{
    // Implementation here
    lookup_documentation(word)
}}

async fn get_documentation_with_fallback(word: &str) -> Option<String> {{
    // Try external documentation
    if let Some(external_doc) = fetch_external_docs(word).await {{
        return Some(external_doc);
    }}
    
    lookup_documentation(word)
}}

fn lookup_documentation(word: &str) -> Option<String> {{
    let custom_docs: HashMap<&str, &str> = [
{}
    ].into_iter().collect();
    
    if let Some(doc) = custom_docs.get(word) {{
        return Some(doc.to_string());
    }}
    
    // Fall back to built-in docs
    let builtin_docs: HashMap<&str, &str> = [
        ("fn", "Declares a function"),
        ("let", "Declares a variable binding"),
        // ... other built-in entries
    ].into_iter().collect();
    
    builtin_docs.get(word).map(|doc| format!("**{{}}**: {{}}", word, doc))
}}

async fn fetch_external_docs(word: &str) -> Option<String> {{
    // External API integration
    None
}}
"#, custom_entries.join(",\n"))
    }
    
    fn generate_word_extraction(&self) -> String {
        r#"
fn get_word_at_position(params: &HoverParams) -> Result<String> {
    use anyhow::anyhow;
    let uri = &params.text_document_position_params.text_document.uri;
    let path = uri.to_file_path().map_err(|_| anyhow!("invalid uri"))?;
    let text = std::fs::read_to_string(path)?;
    let position = params.text_document_position_params.position;
    
    let line = text
        .lines()
        .nth(position.line as usize)
        .ok_or_else(|| anyhow!("line out of bounds"))?;
        
    let chars: Vec<char> = line.chars().collect();
    let mut start = position.character as usize;
    let mut end = start;
    
    while start > 0 {
        let c = chars[start - 1];
        if c.is_alphanumeric() || c == '_' {
            start -= 1;
        } else {
            break;
        }
    }
    
    while end < chars.len() {
        let c = chars[end];
        if c.is_alphanumeric() || c == '_' {
            end += 1;
        } else {
            break;
        }
    }
    
    Ok(chars[start..end].iter().collect())
}
"#.to_string()
    }
}
```

### Step 5: Register Your Custom Provider

Integrate your custom provider with the LSP configuration:

```rust
use rust_sitter_lsp_generator::{LspConfig, generate_lsp};

let custom_hover = CustomHoverProvider::new(&grammar);

let config = LspConfig::builder()
    .name("my-language-lsp")
    .language("my_language")
    .with_custom_handler("textDocument/hover", Box::new(custom_hover))
    .build();

generate_lsp(&grammar, &config, "target/my-language-lsp")?;
```

## Advanced Patterns

### Pattern 1: Grammar-Rule-Specific Help

```rust
impl CustomHoverProvider {
    fn get_rule_specific_help(&self, word: &str, rule_context: &str) -> Option<String> {
        match (word, rule_context) {
            ("id", "table_definition") => Some("**id**: Primary key field".to_string()),
            ("name", "user_model") => Some("**name**: User display name field".to_string()),
            ("created_at", _) => Some("**created_at**: Timestamp field".to_string()),
            _ => None
        }
    }
}
```

### Pattern 2: Multi-Language Support

```rust
impl CustomHoverProvider {
    fn get_localized_documentation(&self, word: &str, locale: &str) -> Option<String> {
        let docs = match locale {
            "es" => self.spanish_docs.get(word),
            "fr" => self.french_docs.get(word),
            "de" => self.german_docs.get(word),
            _ => self.english_docs.get(word),
        };
        
        docs.cloned()
    }
}
```

### Pattern 3: Cached External Lookups

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CachedHoverProvider {
    cache: Arc<RwLock<HashMap<String, String>>>,
    cache_ttl: Duration,
}

impl CachedHoverProvider {
    pub async fn get_cached_documentation(&self, word: &str) -> Option<String> {
        // Check cache first
        if let Some(cached) = self.cache.read().await.get(word) {
            return Some(cached.clone());
        }
        
        // Fetch from external source
        if let Some(doc) = self.fetch_external_docs(word).await {
            // Update cache
            self.cache.write().await.insert(word.to_string(), doc.clone());
            return Some(doc);
        }
        
        None
    }
}
```

## Testing Your Custom Provider

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::Grammar;
    
    #[test]
    fn test_custom_documentation_lookup() {
        let grammar = Grammar::new("test");
        let provider = CustomHoverProvider::new(&grammar);
        
        // Test custom documentation
        assert_eq!(
            provider.lookup_documentation("database"),
            Some("**database**: A collection of organized data that can be queried".to_string())
        );
        
        // Test fallback to built-in
        assert!(provider.lookup_documentation("fn").is_some());
        
        // Test unknown word
        assert!(provider.lookup_documentation("unknown_word").is_none());
    }
    
    #[tokio::test]
    async fn test_external_documentation() {
        let grammar = Grammar::new("test");
        let provider = CustomHoverProvider::new(&grammar);
        
        // Mock external API call
        let result = provider.get_documentation_with_fallback("tokio").await;
        assert!(result.is_some());
    }
}
```

## Performance Considerations

1. **Async Operations**: Use async/await for external API calls
2. **Caching**: Cache external documentation to reduce latency
3. **Timeouts**: Set reasonable timeouts for external requests
4. **Fallbacks**: Always provide fallback documentation
5. **Rate Limiting**: Respect external API rate limits

## Common Use Cases

- **Domain-Specific Languages**: Add terminology for SQL, HTML, CSS
- **Framework Documentation**: Integrate with React, Angular, Vue docs
- **API Documentation**: Link to OpenAPI/Swagger specifications
- **Internal Documentation**: Company-specific coding standards
- **Multilingual Support**: Provide documentation in multiple languages

This approach gives you complete control over hover functionality while leveraging rust-sitter's infrastructure for LSP generation.