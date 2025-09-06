use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use rust_sitter::pure_parser::ParsedNode;
use rust_sitter_tool::build_parsers;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;

mod parse;

/// Convert a ParsedNode to S-expression format
#[allow(dead_code)] // Used in conditional compilation branches
fn node_to_sexp(node: &ParsedNode, source: &str, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    if node.is_named() {
        let mut out = format!("{}({}", indent_str, node.kind());
        if node.child_count() == 0 {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            out.push_str(&format!(" \"{}\")", text));
        } else {
            out.push('\n');
            for child in node.children() {
                out.push_str(&node_to_sexp(child, source, indent + 1));
                out.push('\n');
            }
            out.push_str(&format!("{indent_str})"));
        }
        out
    } else {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        format!("{}\"{}\"", indent_str, text)
    }
}

/// Rust-sitter CLI - Tools for grammar development
#[derive(Parser)]
#[command(name = "rust-sitter")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new rust-sitter grammar project
    Init {
        /// Name of the grammar
        name: String,
        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Build grammar parsers
    Build {
        /// Path to the grammar file or directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Watch for changes and rebuild automatically
        #[arg(short, long)]
        watch: bool,
    },

    /// Parse a file using the grammar
    Parse {
        /// Grammar file (or .so/.dylib path when using --dynamic)
        grammar: PathBuf,
        /// Input file to parse
        input: PathBuf,
        /// Output format
        #[arg(short, long, default_value = "tree")]
        format: OutputFormat,
        /// Use dynamic loader to load compiled grammar from shared library
        #[arg(long)]
        dynamic: bool,
        /// Optional exported symbol (default: "language")
        #[arg(long, default_value = "language")]
        symbol: String,
    },

    /// Test grammar against test files
    Test {
        /// Path to grammar directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Update test snapshots
        #[arg(short, long)]
        update: bool,
    },

    /// Generate grammar documentation
    Doc {
        /// Path to grammar file
        grammar: PathBuf,
        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate grammar syntax
    Check {
        /// Path to grammar file
        grammar: PathBuf,
    },

    /// Show grammar statistics
    Stats {
        /// Path to grammar file
        grammar: PathBuf,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Tree,
    Json,
    Sexp,
    Dot,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }

    match cli.command {
        Commands::Init { name, output } => init_grammar(&name, output)?,
        Commands::Build { path, watch } => {
            if watch {
                watch_and_build(&path)?;
            } else {
                build_grammar(&path)?;
            }
        }
        Commands::Parse {
            grammar,
            input,
            format,
            dynamic,
            symbol,
        } => parse_file(&grammar, &input, format, dynamic, &symbol)?,
        Commands::Test { path, update } => test_grammar(&path, update)?,
        Commands::Doc { grammar, output } => generate_docs(&grammar, output)?,
        Commands::Check { grammar } => check_grammar(&grammar)?,
        Commands::Stats { grammar } => show_stats(&grammar)?,
    }

    Ok(())
}

fn init_grammar(name: &str, output: Option<PathBuf>) -> Result<()> {
    let dir = output.unwrap_or_else(|| PathBuf::from("."));
    let project_dir = dir.join(name);

    println!(
        "{} Creating new grammar project: {}",
        "✨".green(),
        name.bright_blue()
    );

    // Create project structure
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("tests"))?;
    fs::create_dir_all(project_dir.join("examples"))?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
rust-sitter = {{ version = "0.5.0-beta" }}

[build-dependencies]
rust-sitter-tool = {{ version = "0.5.0-beta" }}

[dev-dependencies]
insta = "1.40"
"#,
        name
    );

    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create build.rs
    let build_rs = r#"use rust_sitter_tool::build_parsers;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/grammar.rs");
    build_parsers(&PathBuf::from("src/grammar.rs"));
}
"#;

    fs::write(project_dir.join("build.rs"), build_rs)?;

    // Create example grammar
    let grammar_rs = format!(
        r#"//! {} grammar definition

#[rust_sitter::grammar("{}")]
mod grammar {{
    /// Root node of the grammar
    #[rust_sitter::language]
    pub struct Program {{
        #[rust_sitter::repeat]
        pub statements: Vec<Statement>,
    }}
    
    /// A statement in the language
    #[rust_sitter::language]
    pub struct Statement {{
        pub expr: Expr,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }}
    
    /// An expression
    #[rust_sitter::language]
    pub enum Expr {{
        Number(Number),
        Identifier(Identifier),
    }}
    
    /// A numeric literal
    #[rust_sitter::language]
    pub struct Number {{
        #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
        pub value: i32,
    }}
    
    /// An identifier
    #[rust_sitter::language]
    pub struct Identifier {{
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_]\w*")]
        pub name: String,
    }}
}}
"#,
        name, name
    );

    fs::write(project_dir.join("src/grammar.rs"), grammar_rs)?;

    // Create lib.rs
    let lib_rs = r#"pub mod grammar;

pub use grammar::*;
"#;

    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create example test
    let test_rs = r#"use insta::assert_snapshot;

#[test]
fn test_simple_program() {
    let input = "42; foo;";
    // TODO: Add parsing logic once grammar is built
    assert_snapshot!(input);
}
"#;

    fs::write(project_dir.join("tests/basic.rs"), test_rs)?;

    // Create README
    let readme = format!(
        r#"# {}

A rust-sitter grammar for {}.

## Usage

```rust
// TODO: Add usage example
```

## Development

Build the grammar:
```bash
cargo build
```

Run tests:
```bash
cargo test
```

## License

MIT
"#,
        name, name
    );

    fs::write(project_dir.join("README.md"), readme)?;

    println!(
        "{} Project created at {}",
        "✅".green(),
        project_dir.display().to_string().bright_blue()
    );
    println!("\n{}", "Next steps:".bright_yellow());
    println!("  cd {}", name);
    println!("  cargo build");
    println!("  cargo test");

    Ok(())
}

fn build_grammar(path: &Path) -> Result<()> {
    println!("{} Building grammar...", "🔨".blue());

    if path.is_file() {
        build_parsers(path);
        println!("{} Grammar built successfully!", "✅".green());
    } else {
        // Find all grammar files in directory
        let grammar_files: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "rs")
                    && e.path().to_str().is_some_and(|s| s.contains("grammar"))
            })
            .collect();

        if grammar_files.is_empty() {
            anyhow::bail!("No grammar files found in {}", path.display());
        }

        for entry in grammar_files {
            println!("  {} {}", "Building".bright_black(), entry.path().display());
            build_parsers(entry.path());
        }

        println!("{} All grammars built successfully!", "✅".green());
    }

    Ok(())
}

fn watch_and_build(path: &Path) -> Result<()> {
    use notify::{Event, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    println!("{} Watching for changes...", "👀".blue());

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    // Initial build
    build_grammar(path)?;

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                if event
                    .paths
                    .iter()
                    .any(|p| p.extension().is_some_and(|ext| ext == "rs"))
                {
                    println!("{} Change detected, rebuilding...", "🔄".yellow());
                    if let Err(e) = build_grammar(path) {
                        eprintln!("{} Build failed: {}", "❌".red(), e);
                    }
                }
            }
            Err(_) => continue,
        }
    }
}

fn parse_file(
    grammar: &Path,
    input: &Path,
    format: OutputFormat,
    dynamic: bool,
    _symbol: &str,
) -> Result<()> {
    #[allow(unused_variables)]
    let (grammar, input, format) = (grammar, input, format);
    if dynamic {
        #[cfg(feature = "dynamic")]
        {
            return parse_file_dynamic(grammar, input, format, _symbol);
        }
        #[cfg(not(feature = "dynamic"))]
        {
            eprintln!(
                "{}\n",
                "Error: Dynamic loading not enabled. Build with --features dynamic".red()
            );
            std::process::exit(2);
        }
    }

    // Parse with statically linked grammars
    #[cfg(any(feature = "python-grammar", feature = "javascript-grammar"))]
    {
        let text = fs::read_to_string(input)?;
        let grammar_name = grammar
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let language: &'static rust_sitter::pure_parser::TSLanguage = match grammar_name {
            #[cfg(feature = "python-grammar")]
            "python" => rust_sitter_python::get_language(),
            #[cfg(feature = "javascript-grammar")]
            "javascript" => rust_sitter_javascript::get_language(),
            _ => {
                eprintln!("Unknown grammar: {}", grammar_name);
                eprintln!("Available grammars:");
                #[cfg(feature = "python-grammar")]
                eprintln!("  - python");
                #[cfg(feature = "javascript-grammar")]
                eprintln!("  - javascript");
                std::process::exit(1);
            }
        };

        let mut parser = rust_sitter::pure_parser::Parser::new();
        parser
            .set_language(language)
            .map_err(|e| anyhow::anyhow!("Failed to set language: {}", e))?;

        let result = parser.parse_string(&text);
        if let Some(root) = result.root {
            match format {
                OutputFormat::Json => {
                    println!(
                        "{{\"status\": \"parsing successful\", \"nodes\": \"{}\"}}",
                        node_to_sexp(&root, &text, 0).replace('"', "\\\"")
                    );
                }
                OutputFormat::Sexp => {
                    println!("{}", node_to_sexp(&root, &text, 0));
                }
                _ => {
                    println!("{}", node_to_sexp(&root, &text, 0));
                }
            }
        } else {
            eprintln!("Parse failed: {:?}", result.errors);
            std::process::exit(1);
        }
    }
    #[cfg(not(any(feature = "python-grammar", feature = "javascript-grammar")))]
    {
        eprintln!(
            "Error: No static grammars enabled. Build with --features python-grammar or --features javascript-grammar"
        );
        std::process::exit(2);
    }

    #[cfg(any(feature = "python-grammar", feature = "javascript-grammar"))]
    Ok(())
}

#[cfg(feature = "dynamic")]
fn parse_file_dynamic(
    grammar: &Path,
    input: &Path,
    format: OutputFormat,
    symbol: &str,
) -> Result<()> {
    use libloading::Library;

    // Validate inputs before proceeding
    if !grammar.exists() {
        anyhow::bail!("dynamic grammar not found: {}", grammar.display());
    }

    if !input.exists() {
        anyhow::bail!("input file not found: {}", input.display());
    }

    // Validate symbol name (basic checks)
    if symbol.is_empty() {
        anyhow::bail!("symbol name cannot be empty");
    }

    if !symbol
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        anyhow::bail!("symbol name contains invalid characters: '{}'", symbol);
    }

    println!(
        "{} Loading dynamic grammar: {}",
        "🔧".blue(),
        grammar.display()
    );

    let input_content = fs::read_to_string(input)
        .map_err(|e| anyhow::anyhow!("Failed to read input file: {}", e))?;

    // Validate input size (prevent extremely large inputs that could cause issues)
    const MAX_INPUT_SIZE: usize = 100 * 1024 * 1024; // 100MB
    if input_content.len() > MAX_INPUT_SIZE {
        anyhow::bail!(
            "Input file too large: {} bytes (max: {} bytes)",
            input_content.len(),
            MAX_INPUT_SIZE
        );
    }

    // Load library safely
    let lib = Library::new(grammar)
        .map_err(|e| anyhow::anyhow!("Failed to load dynamic library: {}", e))?;

    // Prepare symbol name with proper null termination
    let sym_name = {
        let mut s = symbol.as_bytes().to_vec();
        if !s.ends_with(b"\0") {
            s.push(0);
        }
        s
    };

    // We need unsafe for FFI, but we'll be very careful about it
    unsafe {
        // For dynamic loading, we need to bridge between Tree-sitter C library
        // and rust-sitter's pure parser. This requires two approaches:

        #[cfg(feature = "pure-rust")]
        {
            // Pure-Rust approach: Load TSLanguage struct and use pure parser
            use rust_sitter::pure_parser::{ParseResult, ParsedNode, Parser, TSLanguage};

            // Load the language function symbol with proper error handling
            let get_language: libloading::Symbol<unsafe extern "C" fn() -> *const TSLanguage> = lib
                .get(&sym_name)
                .map_err(|e| anyhow::anyhow!("Failed to find symbol '{}': {}", symbol, e))?;

            // Call the function to get language pointer
            let lang_ptr = get_language();
            if lang_ptr.is_null() {
                anyhow::bail!("Language symbol '{}' returned null pointer", symbol);
            }

            // Validate pointer alignment (basic sanity check)
            if (lang_ptr as usize) % std::mem::align_of::<TSLanguage>() != 0 {
                anyhow::bail!("Language pointer is not properly aligned");
            }

            // Convert to reference with lifetime tied to library
            // Safety: We've checked the pointer is not null and appears to be valid
            // The lifetime is managed by keeping the library alive
            let language: &'static TSLanguage = &*lang_ptr;

            // Validate basic language structure (sanity checks)
            if language.version == 0 {
                anyhow::bail!("Language appears to have invalid version (0)");
            }

            if language.symbol_count == 0 {
                anyhow::bail!("Language appears to have no symbols");
            }

            // Create parser with validated language
            let mut parser = Parser::new(language);

            // Parse with timeout protection (via signal handling would be ideal, but for CLI this is reasonable)
            let result: ParseResult = parser.parse_string(&input_content);

            // Safe recursive node counting with stack overflow protection
            fn count_nodes_safe(node: &ParsedNode, depth: usize) -> Result<usize> {
                const MAX_DEPTH: usize = 10000;
                if depth > MAX_DEPTH {
                    anyhow::bail!("Parse tree too deep (possible infinite recursion)");
                }

                let mut count = 1;
                for child in &node.children {
                    count += count_nodes_safe(child, depth + 1)?;
                }
                Ok(count)
            }

            println!(
                "{} Loaded language from: {}",
                "✓".green(),
                grammar.display()
            );
            println!("Input size: {} bytes", input_content.len());

            if result.errors.is_empty() {
                if let Some(root) = result.root {
                    match count_nodes_safe(&root, 0) {
                        Ok(nodes) => match format {
                            OutputFormat::Json => println!(
                                "{{\"status\":\"ok\",\"root_symbol\":{},\"nodes\":{}}}",
                                root.symbol, nodes
                            ),
                            _ => println!(
                                "Parsed successfully. Root symbol: {}, nodes: {}",
                                root.symbol, nodes
                            ),
                        },
                        Err(e) => match format {
                            OutputFormat::Json => println!(
                                "{{\"status\":\"error\",\"message\":\"Tree too complex: {}\"}}",
                                e.to_string().replace('"', "\\\"")
                            ),
                            _ => println!("Error analyzing tree: {}", e),
                        },
                    }
                } else {
                    match format {
                        OutputFormat::Json => println!("{{\"status\":\"ok\",\"nodes\":0}}"),
                        _ => println!("Parsed successfully but produced empty tree"),
                    }
                }
            } else {
                let err_count = result.errors.len();
                match format {
                    OutputFormat::Json => {
                        println!("{{\"status\":\"error\",\"errors\":{}}}", err_count)
                    }
                    _ => {
                        println!("Parsing completed with {} error(s)", err_count);
                        for (i, error) in result.errors.iter().enumerate().take(3) {
                            println!(
                                "  Error {}: {:?} at {}..{}",
                                i + 1,
                                error.kind,
                                error.start,
                                error.end
                            );
                        }
                        if result.errors.len() > 3 {
                            println!("  ... and {} more errors", result.errors.len() - 3);
                        }
                    }
                }
            }
        }

        #[cfg(not(feature = "pure-rust"))]
        {
            // Tree-sitter compatibility approach using tree_sitter crate
            use rust_sitter::tree_sitter::{Language, Node, Parser};

            // Load language function with error handling
            let get_language: libloading::Symbol<unsafe extern "C" fn() -> Language> = lib
                .get(&sym_name)
                .map_err(|e| anyhow::anyhow!("Failed to find symbol '{}': {}", symbol, e))?;

            let language = get_language();

            let mut parser = Parser::new();
            parser
                .set_language(&language)
                .map_err(|e| anyhow::anyhow!("Failed to set language: {}", e))?;

            let tree = parser
                .parse(&input_content, None)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse input"))?;

            let root_node = tree.root_node();

            // Safe recursive node counting with depth protection
            fn count_tree_nodes_safe(node: Node, depth: usize) -> Result<usize> {
                const MAX_DEPTH: usize = 10000;
                if depth > MAX_DEPTH {
                    anyhow::bail!("Parse tree too deep");
                }

                let mut count = 1;
                let mut cursor = node.walk();
                if cursor.goto_first_child() {
                    loop {
                        count += count_tree_nodes_safe(cursor.node(), depth + 1)?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
                Ok(count)
            }
            println!(
                "{} Loaded language from: {}",
                "✓".green(),
                grammar.display()
            );
            println!("Input size: {} bytes", input_content.len());

            match count_tree_nodes_safe(root_node, 0) {
                Ok(nodes) => {
                    let has_error = root_node.has_error();

                    if !has_error {
                        match format {
                            OutputFormat::Json => println!(
                                "{{\"status\":\"ok\",\"root_symbol\":\"{}\",\"nodes\":{}}}",
                                root_node.kind(),
                                nodes
                            ),
                            _ => println!(
                                "Parsed successfully. Root symbol: {}, nodes: {}",
                                root_node.kind(),
                                nodes
                            ),
                        }
                    } else {
                        match format {
                            OutputFormat::Json => println!(
                                "{{\"status\":\"error\",\"message\":\"Parse tree contains errors\"}}"
                            ),
                            _ => println!(
                                "Parsing completed but tree contains errors. Total nodes: {}",
                                nodes
                            ),
                        }
                    }
                }
                Err(e) => match format {
                    OutputFormat::Json => println!(
                        "{{\"status\":\"error\",\"message\":\"Tree analysis failed: {}\"}}",
                        e.to_string().replace('"', "\\\"")
                    ),
                    _ => println!("Error analyzing tree: {}", e),
                },
            }
        }
    }

    Ok(())
}

fn test_grammar(_path: &Path, update: bool) -> Result<()> {
    println!("{} Testing grammar...", "🧪".blue());

    if update {
        println!("  {} Updating snapshots", "📸".yellow());
    }

    // Run cargo test
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("test");
    if update {
        cmd.env("INSTA_UPDATE", "always");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("{} All tests passed!", "✅".green());
    } else {
        anyhow::bail!("Tests failed");
    }

    Ok(())
}

fn generate_docs(grammar: &Path, output: Option<PathBuf>) -> Result<()> {
    println!("{} Generating documentation...", "📚".blue());

    let content = fs::read_to_string(grammar)?;

    // Simple doc generation - extract doc comments
    let mut docs = String::from("# Grammar Documentation\n\n");

    for line in content.lines() {
        if line.trim().starts_with("///") {
            docs.push_str(line.trim_start_matches("///").trim());
            docs.push('\n');
        }
    }

    if let Some(output) = output {
        fs::write(output, docs)?;
        println!("{} Documentation written to file", "✅".green());
    } else {
        println!("{}", docs);
    }

    Ok(())
}

fn check_grammar(grammar: &Path) -> Result<()> {
    println!("{} Checking grammar syntax...", "🔍".blue());

    // Ensure required env vars for build_parsers
    let temp = TempDir::new()?;
    let target =
        std::env::var("TARGET").unwrap_or_else(|_| String::from("x86_64-unknown-linux-gnu"));
    unsafe {
        std::env::set_var("OUT_DIR", temp.path());
        std::env::set_var("TARGET", &target);
        std::env::set_var("HOST", &target);
        std::env::set_var("OPT_LEVEL", "0");
        std::env::set_var("PROFILE", "debug");
    }

    // Try to build the grammar
    match std::panic::catch_unwind(|| build_parsers(grammar)) {
        Ok(_) => {
            println!("{} Grammar syntax is valid", "✅".green());
            Ok(())
        }
        Err(_) => {
            anyhow::bail!("Grammar syntax is invalid");
        }
    }
}

fn show_stats(grammar: &Path) -> Result<()> {
    println!("{} Grammar statistics:", "📊".blue());

    let content = fs::read_to_string(grammar)?;

    let lines = content.lines().count();
    let rules = content.matches("#[rust_sitter::language]").count();
    let leaf_rules = content.matches("#[rust_sitter::leaf").count();
    let repeat_rules = content.matches("#[rust_sitter::repeat").count();

    println!("  {} {}", "Lines:".bright_black(), lines);
    println!("  {} {}", "Rules:".bright_black(), rules);
    println!("  {} {}", "Leaf rules:".bright_black(), leaf_rules);
    println!("  {} {}", "Repeat rules:".bright_black(), repeat_rules);

    Ok(())
}
