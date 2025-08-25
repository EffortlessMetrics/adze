use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use rust_sitter_tool::build_parsers;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod parse;

/// Rust-sitter CLI - Tools for grammar development
#[derive(Parser)]
#[command(
    name = "rust-sitter",
    author,
    version,
    about = "Rust-sitter CLI",
    long_about = "CLI tools for rust-sitter grammar development"
)]
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

    let crate_name = name.replace('-', "_");

    // Create example test
    let test_rs = format!(
        r#"use insta::assert_debug_snapshot;
use {crate_name}::grammar;

#[test]
fn test_simple_program() {{
    let input = "42; foo;";
    assert_debug_snapshot!(grammar::parse(input));
}}
"#,
        crate_name = crate_name
    );

    fs::write(project_dir.join("tests/basic.rs"), test_rs)?;

    // Create README
    let readme = format!(
        r#"# {name}

A rust-sitter grammar for {name}.

## Usage

```rust
use {crate_name}::grammar;

fn main() {{
    let input = "42; foo;";
    let parsed = grammar::parse(input);
    println!("{:?}", parsed);
}}
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
        name = name,
        crate_name = crate_name
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
                e.path().extension().map_or(false, |ext| ext == "rs")
                    && e.path().to_str().map_or(false, |s| s.contains("grammar"))
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
                    .any(|p| p.extension().map_or(false, |ext| ext == "rs"))
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
    symbol: &str,
) -> Result<()> {
    if dynamic {
        #[cfg(feature = "dynamic")]
        {
            return parse_file_dynamic(grammar, input, format, symbol);
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
    println!("{} Parsing file: {}", "📄".blue(), input.display());

    // Convert clap OutputFormat to our parse module's format
    let parse_format = match format {
        OutputFormat::Tree => parse::OutputFormat::Tree,
        OutputFormat::Json => parse::OutputFormat::Json,
        OutputFormat::Sexp => parse::OutputFormat::Sexp,
        OutputFormat::Dot => parse::OutputFormat::Dot,
    };

    // Try to parse with the generated parser
    match parse::parse_file_with_generated_parser(grammar, input, parse_format) {
        Ok(()) => Ok(()),
        Err(e) => {
            // If parsing fails, provide helpful guidance
            eprintln!("{} Failed to parse: {}", "❌".red(), e);
            eprintln!("\n{} Alternative approaches:", "💡".yellow());
            eprintln!("1. Ensure your grammar file is valid");
            eprintln!("2. Build your grammar with `rust-sitter build`");
            eprintln!("3. Use the generated parse() function in your Rust code:");
            eprintln!("\n   use my_grammar::parse;\n   let result = parse(\"input text\");\n");
            Err(e)
        }
    }
}

#[cfg(feature = "dynamic")]
fn parse_file_dynamic(
    grammar: &Path,
    input: &Path,
    format: OutputFormat,
    symbol: &str,
) -> Result<()> {
    use libloading::Library;
    use rust_sitter::pure_parser::{ParsedNode, Parser as PureParser, TSLanguage};

    fn print_tree(node: &ParsedNode, indent: usize) {
        let spaces = "  ".repeat(indent);
        println!("{}({}", spaces, node.kind());
        for child in &node.children {
            print_tree(child, indent + 1);
        }
        println!("{})", spaces);
    }

    fn node_to_sexp(node: &ParsedNode, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        if node.children.is_empty() {
            format!("{}({})", indent, node.kind())
        } else {
            let mut result = format!("{}({}", indent, node.kind());
            for child in &node.children {
                result.push('\n');
                result.push_str(&node_to_sexp(child, depth + 1));
            }
            result.push_str(&format!("\n{}{})", indent, ""));
            result
        }
    }

    fn add_node_to_dot(
        node: &ParsedNode,
        dot: &mut String,
        id_counter: &mut usize,
        parent_id: Option<usize>,
    ) {
        let current_id = *id_counter;
        *id_counter += 1;
        dot.push_str(&format!("  node{} [label=\"{}\"];\n", current_id, node.kind()));
        if let Some(pid) = parent_id {
            dot.push_str(&format!("  node{} -> node{};\n", pid, current_id));
        }
        for child in &node.children {
            add_node_to_dot(child, dot, id_counter, Some(current_id));
        }
    }

    println!(
        "{} Loading dynamic grammar: {}",
        "🔧".blue(),
        grammar.display()
    );
    let input_content = fs::read_to_string(input)?;

    unsafe {
        if !grammar.exists() {
            anyhow::bail!("dynamic grammar not found: {}", grammar.display());
        }

        let lib = Library::new(grammar)?;
        let sym_name = {
            let mut s = symbol.as_bytes().to_vec();
            if !s.ends_with(b"\0") {
                s.push(0);
            }
            s
        };
        let get_language: libloading::Symbol<unsafe extern "C" fn() -> *const u8> =
            lib.get(&sym_name)?;
        let lang_ptr = get_language();
        let language = &*(lang_ptr as *const TSLanguage);

        let mut parser = PureParser::new();
        parser
            .set_language(language)
            .map_err(|e| anyhow::anyhow!(e))?;

        let result = parser.parse_string(&input_content);

        if let Some(root) = result.root {
            match format {
                OutputFormat::Json => {
                    #[cfg(feature = "serialization")]
                    {
                        use rust_sitter::serialization::TreeSerializer;
                        let serializer = TreeSerializer::new(input_content.as_bytes());
                        let node = serializer.serialize_node(&root);
                        let json = serde_json::to_string_pretty(&node)?;
                        println!("{}", json);
                    }
                    #[cfg(not(feature = "serialization"))]
                    {
                        println!("{{\"error\": \"serialization feature not enabled\"}}");
                    }
                }
                OutputFormat::Sexp => println!("{}", node_to_sexp(&root, 0)),
                OutputFormat::Dot => {
                    let mut dot = String::from("digraph ParseTree {\n");
                    let mut id = 0;
                    add_node_to_dot(&root, &mut dot, &mut id, None);
                    dot.push_str("}\n");
                    println!("{}", dot);
                }
                OutputFormat::Tree => print_tree(&root, 0),
            }
        } else {
            eprintln!("{} Failed to parse input", "❌".red());
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
            docs.push_str(&line.trim_start_matches("///").trim());
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

    // Try to build the grammar
    unsafe {
        std::env::set_var("OUT_DIR", ".");
        std::env::set_var("TARGET", "x86_64-unknown-linux-gnu");
        std::env::set_var("OPT_LEVEL", "0");
        std::env::set_var("HOST", "x86_64-unknown-linux-gnu");
    }
    match std::panic::catch_unwind(|| build_parsers(grammar)) {
        Ok(_) => {
            println!("{} Grammar syntax is valid!", "✅".green());
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
