//! Parse command implementation for rust-sitter CLI

use anyhow::{Context, Result};
// Pure parser imports commented out as they're not used in this module currently
// use rust_sitter::pure_parser::{ParsedNode, Parser, TSLanguage};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// S-expression conversion function moved to main.rs where it's actually used
// Keeping this commented for reference:
/*
fn node_to_sexp(node: &ParsedNode, source: &str, indent: usize) -> String {
    // Implementation in main.rs
}
*/

/// Formats for output
#[allow(dead_code)] // Legacy code - keeping for reference
#[derive(Clone)]
pub enum OutputFormat {
    Tree,
    Json,
    Sexp,
    Dot,
}

/// Parse a file using a generated parser
#[allow(dead_code)] // Legacy code - keeping for reference
pub fn parse_file_with_generated_parser(
    grammar_path: &Path,
    input_path: &Path,
    format: OutputFormat,
) -> Result<()> {
    // Read the input file
    let input_content = fs::read_to_string(input_path).context("Failed to read input file")?;

    // Create a temporary directory for the parsing project
    let temp_dir = TempDir::new()?;
    let project_dir = temp_dir.path();

    // Generate a minimal Cargo.toml
    let cargo_toml = r#"[package]
name = "parser-runner"
version = "0.1.0"
edition = "2021"

[dependencies]
rust-sitter = { version = "0.6.0", path = "../../../runtime", features = ["serialization"] }
rust-sitter-tool = { version = "0.6.0", path = "../../../tool" }
serde_json = "1.0"

[build-dependencies]
rust-sitter-tool = { version = "0.6.0", path = "../../../tool" }
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Copy the grammar file
    let grammar_content = fs::read_to_string(grammar_path)?;
    fs::create_dir_all(project_dir.join("src"))?;
    fs::write(project_dir.join("src/grammar.rs"), &grammar_content)?;

    // Generate build.rs
    let build_rs = r#"
use rust_sitter_tool::build_parsers;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/grammar.rs");
    build_parsers(&PathBuf::from("src/grammar.rs"));
}
"#;
    fs::write(project_dir.join("build.rs"), build_rs)?;

    // Generate main.rs that will parse and output
    let main_rs = generate_parser_main(&input_content, format)?;
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // Run cargo build and execute
    println!("Building parser...");
    let build_output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(project_dir)
        .output()
        .context("Failed to build parser")?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        anyhow::bail!("Failed to build parser:\n{}", stderr);
    }

    // Run the parser
    println!("Running parser...");
    let run_output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .current_dir(project_dir)
        .output()
        .context("Failed to run parser")?;

    if !run_output.status.success() {
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        anyhow::bail!("Failed to run parser:\n{}", stderr);
    }

    // Print the output
    print!("{}", String::from_utf8_lossy(&run_output.stdout));

    Ok(())
}

/// Generate the main.rs file content based on output format
#[allow(dead_code)] // Legacy code - keeping for reference
fn generate_parser_main(input_content: &str, format: OutputFormat) -> Result<String> {
    let input_escaped = input_content.replace('\\', "\\\\").replace('"', "\\\"");

    let format_code = match format {
        OutputFormat::Json => {
            r#"
    #[cfg(feature = "serialization")]
    {
        use rust_sitter::serialization::{TreeSerializer};
        let serializer = TreeSerializer::new(input.as_bytes());
        match serializer.serialize_tree(&tree) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Serialization error: {}", e),
        }
    }
    #[cfg(not(feature = "serialization"))]
    {
        println!("{{\"error\": \"Serialization feature not enabled\"}}");
    }
"#
        }
        OutputFormat::Sexp => {
            r#"
    // Simple S-expression output
    fn node_to_sexp(node: &rust_sitter::Node, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        if node.is_named() {
            let children: Vec<_> = node.children().collect();
            if children.is_empty() {
                format!("{}({})", indent, node.kind())
            } else {
                let mut result = format!("{}({}", indent, node.kind());
                for child in children {
                    result.push('\n');
                    result.push_str(&node_to_sexp(&child, depth + 1));
                }
                result.push_str(&format!("\n{})", indent));
                result
            }
        } else {
            format!("{}\"{}\"", indent, node.kind())
        }
    }
    
    println!("{}", node_to_sexp(&tree.root_node(), 0));
"#
        }
        OutputFormat::Dot => {
            r#"
    // DOT graph output
    let mut dot = String::from("digraph ParseTree {\n");
    let mut node_id = 0;
    
    fn add_node_to_dot(node: &rust_sitter::Node, dot: &mut String, id_counter: &mut usize, parent_id: Option<usize>) -> usize {
        let current_id = *id_counter;
        *id_counter += 1;
        
        let label = if node.is_named() {
            node.kind().to_string()
        } else {
            format!("\"{}\"", node.kind())
        };
        
        dot.push_str(&format!("  node{} [label=\"{}\"];\n", current_id, label));
        
        if let Some(pid) = parent_id {
            dot.push_str(&format!("  node{} -> node{};\n", pid, current_id));
        }
        
        for child in node.children() {
            add_node_to_dot(&child, dot, id_counter, Some(current_id));
        }
        
        current_id
    }
    
    add_node_to_dot(&tree.root_node(), &mut dot, &mut node_id, None);
    dot.push_str("}\n");
    println!("{}", dot);
"#
        }
        OutputFormat::Tree => {
            r#"
    // Default tree output
    fn print_tree(node: &rust_sitter::Node, indent: usize) {
        let spaces = "  ".repeat(indent);
        if node.is_named() {
            println!("{}({}", spaces, node.kind());
            for child in node.children() {
                print_tree(&child, indent + 1);
            }
            println!("{})", spaces);
        } else {
            println!("{}\"{}\"", spaces, node.kind());
        }
    }
    
    print_tree(&tree.root_node(), 0);
"#
        }
    };

    Ok(format!(
        r#"
use rust_sitter::Parser;

fn main() {{
    let input = "{}";
    
    // Parse the input
    let mut parser = Parser::new();
    // Note: The actual language setup would be done here
    // For now, we'll use a placeholder
    
    match parser.parse(input, None) {{
        Some(tree) => {{
            {}
        }}
        None => {{
            eprintln!("Failed to parse input");
            std::process::exit(1);
        }}
    }}
}}
"#,
        input_escaped, format_code
    ))
}
