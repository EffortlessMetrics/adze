use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use rust_sitter_tool::build_parsers;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod parse;

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
    let test_rs = format!(
        r#"use insta::assert_debug_snapshot;
use {0}::parse;

#[test]
fn test_simple_program() {{
    assert_debug_snapshot!(parse("42; foo;"));
}}
"#,
        name
    );

    fs::write(project_dir.join("tests/basic.rs"), test_rs)?;

    // Create README
    let readme = format!(
        r#"# {0}

A rust-sitter grammar for {0}.

## Usage

```rust
use {0}::parse;

fn main() {{
    let ast = parse("42; foo;").expect("parse error");
    println!("{{:#?}}", ast);
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
        name
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

    println!(
        "{} Loading dynamic grammar: {}",
        "🔧".blue(),
        grammar.display()
    );
    let input_content = fs::read_to_string(input)?;

    unsafe {
        // Check if file exists
        if !grammar.exists() {
            anyhow::bail!("dynamic grammar not found: {}", grammar.display());
        }

        let lib = Library::new(grammar)?;
        // Build symbol name with null terminator
        let sym_name = {
            let mut s = symbol.as_bytes().to_vec();
            if !s.ends_with(b"\0") {
                s.push(0);
            }
            s
        };
        let get_language: libloading::Symbol<unsafe extern "C" fn() -> *const u8> =
            lib.get(&sym_name)?;
        let _lang_ptr = get_language();

        // TODO: Bridge to rust-sitter's pure parser using the language pointer
        println!(
            "{} Loaded language from: {}",
            "✓".green(),
            grammar.display()
        );
        println!("Input size: {} bytes", input_content.len());

        // For now, just show we loaded it successfully
        match format {
            OutputFormat::Json => println!("{{\"status\": \"dynamic loading successful\"}}"),
            _ => println!("Dynamic loading successful - parser integration pending"),
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

    // Try to build the grammar
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
