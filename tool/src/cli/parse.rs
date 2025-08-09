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
/// This is an MVP implementation that provides clear error messages.
/// Future versions will support dynamic loading of compiled parsers.
pub fn parse_file(
    file_path: &Path,
    parser_path: Option<&Path>,
    format: OutputFormat,
    _show_fields: bool,
    _show_stats: bool,
) -> Result<()> {
    // Read the source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

    // Check if a crate path was provided
    if let Some(crate_path) = parser_path {
        // MVP: Provide honest feedback about the current limitations
        eprintln!("rust-sitter CLI v0.6.0 - Parse Command");
        eprintln!("========================================");
        eprintln!();
        eprintln!("STATUS: The dynamic parser loading feature is not yet implemented.");
        eprintln!();
        eprintln!("You specified: --parser {:?}", crate_path);
        eprintln!("File to parse: {:?} ({} bytes)", file_path, source.len());
        eprintln!();
        eprintln!("CURRENT LIMITATIONS:");
        eprintln!("  - Cannot dynamically load compiled parser libraries (.so/.dll)");
        eprintln!("  - Cannot invoke external parser crates directly");
        eprintln!("  - Parser must be integrated at compile time");
        eprintln!();
        eprintln!("HOW TO USE RUST-SITTER TODAY:");
        eprintln!();
        eprintln!("1. Define your grammar in Rust:");
        eprintln!("   ```rust");
        eprintln!("   #[rust_sitter::grammar(\"my_language\")]");
        eprintln!("   pub mod grammar {{");
        eprintln!("       #[rust_sitter::language]");
        eprintln!("       pub struct MyLanguage;");
        eprintln!("   }}");
        eprintln!("   ```");
        eprintln!();
        eprintln!("2. Build your parser:");
        eprintln!("   ```bash");
        eprintln!("   cargo build");
        eprintln!("   ```");
        eprintln!();
        eprintln!("3. Use it in your Rust code:");
        eprintln!("   ```rust");
        eprintln!("   use my_grammar::parse;");
        eprintln!("   let tree = parse(\"{}\");", 
            source.chars().take(30).collect::<String>().replace('\n', "\\n")
        );
        eprintln!("   ```");
        eprintln!();
        eprintln!("COMING SOON (v0.6.x):");
        eprintln!("  - Dynamic parser loading via --parser flag");
        eprintln!("  - Tree-sitter compatible CLI functionality");
        eprintln!();
        eprintln!("For updates, see: https://github.com/hydro-project/rust-sitter");
        
        // Return an honest error code
        std::process::exit(64); // EX_USAGE - command line usage error
    }

    // No parser specified - provide helpful message
    eprintln!("rust-sitter parse - No parser specified");
    eprintln!("========================================");
    eprintln!();
    eprintln!("To parse files with rust-sitter:");
    eprintln!();
    eprintln!("Option 1: Use a parser crate (not yet implemented)");
    eprintln!("  rust-sitter parse --parser <parser-crate> {}", file_path.display());
    eprintln!();
    eprintln!("Option 2: Integrate directly in Rust code (working today)");
    eprintln!("  1. Define your grammar using #[rust_sitter::grammar]");
    eprintln!("  2. Build it with `cargo build`");
    eprintln!("  3. Use the generated parse() function in your code");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  ```rust");
    eprintln!("  use my_grammar::parse;");
    eprintln!("  let result = parse(\"input text\");");
    eprintln!("  ```");
    eprintln!();
    eprintln!("Note: Full CLI functionality including dynamic parser loading");
    eprintln!("      is planned for v0.6.x releases.");

    // Return error to indicate no parser was specified
    std::process::exit(64) // EX_USAGE - command line usage error
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
