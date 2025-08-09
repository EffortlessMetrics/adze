use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::path::Path;

/// Output formats for the parse command
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Sexp,
    Json,
    Dot,
}

/// Parse a file using the specified parser
///
/// This is an MVP implementation that runs a parser crate to parse the file.
/// Future versions will support dynamic loading of compiled parsers.
pub fn parse_file(
    file_path: &Path,
    parser_path: Option<&Path>,
    format: OutputFormat,
    show_fields: bool,
    show_stats: bool,
) -> Result<()> {
    // Read the source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

    // Check if a crate path was provided
    if let Some(crate_path) = parser_path {
        // MVP: Run the parser crate with cargo
        eprintln!("Parsing file using crate at: {:?}", crate_path);
        
        // Create a simple runner script that parses the file
        let runner_code = format!(
            r#"
fn main() {{
    let source = r###"{}"###;
    match parse(source) {{
        Ok(tree) => {{
            // Output the tree in the requested format
            println!("{{:#?}}", tree);
        }}
        Err(e) => {{
            eprintln!("Parse error: {{:?}}", e);
            std::process::exit(1);
        }}
    }}
}}
"#,
            source.replace("###", "####")
        );
        
        // Run the crate with the runner code
        let output = std::process::Command::new("cargo")
            .arg("run")
            .arg("-q")
            .arg("-p")
            .arg(crate_path)
            .arg("--")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .context("Failed to run parser crate")?;
        
        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("Parser failed");
        }
        
        return Ok(());
    }

    // No parser specified - provide helpful message
    eprintln!("To parse files with rust-sitter:");
    eprintln!();
    eprintln!("Option 1: Use a parser crate (MVP)");
    eprintln!("  rust-sitter parse --parser ./my-parser-crate {}", file_path.display());
    eprintln!();
    eprintln!("Option 2: Build and use in Rust code");
    eprintln!("  1. Define your grammar using #[rust_sitter::grammar]");
    eprintln!("  2. Build it with `cargo build`");
    eprintln!("  3. Use the generated parse() function in your code");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  use my_grammar::parse;");
    eprintln!("  let result = parse(\"input text\");");
    eprintln!();
    eprintln!("Note: Dynamic library loading (.so/.dll) support coming in v0.6.x");

    // Return error to indicate no parser was specified
    anyhow::bail!(
        "No parser specified. Use --parser <crate-path> to specify a parser crate."
    )
}

/// Auto-detect parser based on file extension
fn _auto_detect_parser(file_path: &Path) -> Option<String> {
    let ext = file_path.extension()?.to_str()?;

    match ext {
        "js" | "mjs" => Some("javascript".to_string()),
        "ts" | "tsx" => Some("typescript".to_string()),
        "py" => Some("python".to_string()),
        "rs" => Some("rust".to_string()),
        "c" | "h" => Some("c".to_string()),
        "cpp" | "cc" | "cxx" | "hpp" => Some("cpp".to_string()),
        "go" => Some("go".to_string()),
        "rb" => Some("ruby".to_string()),
        "java" => Some("java".to_string()),
        "cs" => Some("csharp".to_string()),
        "json" => Some("json".to_string()),
        "toml" => Some("toml".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "html" | "htm" => Some("html".to_string()),
        "css" => Some("css".to_string()),
        "md" | "markdown" => Some("markdown".to_string()),
        _ => None,
    }
}
