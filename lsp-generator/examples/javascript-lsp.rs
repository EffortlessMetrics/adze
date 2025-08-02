// Example: Generate an LSP server for JavaScript

use anyhow::Result;
use rust_sitter_lsp_generator::LspBuilder;

fn main() -> Result<()> {
    // Generate JavaScript LSP server with all features
    LspBuilder::new("javascript-lsp")
        .version("1.0.0")
        .grammar_path("../grammars/javascript/src/lib.rs")
        .output_dir("./javascript-lsp-server")
        .feature("all")
        .build()?;

    println!("JavaScript LSP server generated!");
    Ok(())
}
