use adze_grammar_analysis_core::analyze_grammar_file;
use adze_tool::{build_parsers, pure_rust_builder::BuildResult};
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Adze CLI
#[derive(Parser, Debug)]
#[command(name = "adze")]
#[command(about = "Adze CLI - Tools for grammar development")]
#[command(author, version, long_about = None)]
pub(crate) struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Initialize a new adze grammar project
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

    /// Parse a file using the grammar (experimental: parser execution not yet wired)
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

    /// Show version information
    Version,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum OutputFormat {
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
        Commands::Version => print_version(),
    }

    Ok(())
}

fn init_grammar(name: &str, output: Option<PathBuf>) -> Result<()> {
    let dir = output.unwrap_or_else(|| PathBuf::from("."));
    let project_dir = dir.join(name);
    let scaffold_version = released_version(env!("CARGO_PKG_VERSION"));
    let crate_name = name.replace('-', "_");
    let language_name = crate_name.clone();
    let local_paths = discover_local_adze_paths();
    let adze_dep = if let Some((runtime_path, _tool_path)) = &local_paths {
        format!(r#"adze = {{ path = "{}" }}"#, runtime_path.display())
    } else {
        format!(
            r#"adze = {{ git = "https://github.com/effortlessmetrics/adze", tag = "v{}", package = "adze" }}"#,
            scaffold_version
        )
    };

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
{adze_dep}
"#,
        name,
        adze_dep = adze_dep
    );

    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create build.rs
    let build_rs = r#"fn main() {
    // Placeholder build script: rerun when the grammar file changes.
    // To generate parser artifacts, add `adze-tool` as a build-dependency
    // and call `adze_tool::build_parsers(...)` here once your grammar is ready.
    println!("cargo:rerun-if-changed=src/grammar.rs");
}
"#;

    fs::write(project_dir.join("build.rs"), build_rs)?;

    // Create example grammar
    let grammar_rs = format!(
        r#"//! {} grammar definition
//!
//! This scaffold is intentionally buildable out-of-the-box.
//! Replace these placeholder AST types with `#[adze::grammar("{}")]`
//! declarations once you're ready to generate parser artifacts.

/// Root node of the grammar.
pub struct Program {{
    pub first: Statement,
    pub rest: Vec<Statement>,
}}

/// A statement in the language.
pub struct Statement {{
    pub expr: Expr,
}}

/// An expression.
pub enum Expr {{
    Number(Number),
    Identifier(Identifier),
}}

/// A numeric literal.
pub struct Number {{
    pub value: i32,
}}

/// An identifier.
pub struct Identifier {{
    pub name: String,
}}
"#,
        name, language_name
    );

    fs::write(project_dir.join("src/grammar.rs"), grammar_rs)?;

    // Create lib.rs
    let lib_rs = r#"pub mod grammar;

pub use grammar::*;
"#;

    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create example test
    let test_rs = r#"use std::mem::size_of;

use __CRATE__::grammar::{Identifier, Number, Program, Statement};

#[test]
fn test_generated_grammar_types_compile() {
    assert!(size_of::<Program>() > 0);
    assert!(size_of::<Statement>() > 0);
    assert!(size_of::<Number>() > 0);
    assert!(size_of::<Identifier>() > 0);
}
"#
    .replace("__CRATE__", &crate_name);

    fs::write(project_dir.join("tests/basic.rs"), test_rs)?;

    // Create README
    let readme = format!(
        r#"# {}

A adze grammar for {}.

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
    if scaffold_version != env!("CARGO_PKG_VERSION") {
        println!(
            "{} Using published dependency line {} (CLI version is {}).",
            "ℹ️ ".blue(),
            scaffold_version.bright_blue(),
            env!("CARGO_PKG_VERSION").bright_black()
        );
    }
    if local_paths.is_some() {
        println!(
            "{} Using local path dependencies because an adze workspace was detected.",
            "ℹ️ ".blue()
        );
    } else {
        println!(
            "{} Using git dependencies pinned to release tag v{}.",
            "ℹ️ ".blue(),
            scaffold_version.bright_blue()
        );
    }
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
    _format: OutputFormat,
    dynamic: bool,
    _symbol: &str,
) -> Result<()> {
    if dynamic {
        #[cfg(feature = "dynamic")]
        {
            return parse_file_dynamic(grammar, input, _format, _symbol);
        }
        #[cfg(not(feature = "dynamic"))]
        {
            anyhow::bail!(
                "dynamic parsing is experimental: rebuild adze-cli with --features dynamic"
            )
        }
    }
    println!("{} Parsing file: {}", "📄".blue(), input.display());

    let input_content = fs::read_to_string(input)?;
    println!(
        "  Grammar: {}\n  Input: {} ({} bytes)",
        grammar.display(),
        input.display(),
        input_content.len()
    );
    anyhow::bail!(
        "static parse mode is not implemented yet. `adze parse` is currently experimental and does not produce parse trees"
    )
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

        let _ = (format, input_content.len());
        println!(
            "{} Loaded language symbol '{}' from {}",
            "🧪".yellow(),
            symbol,
            grammar.display()
        );
    }

    anyhow::bail!(
        "dynamic parse mode is experimental: grammar loading works, but executing the parser is not implemented yet"
    )
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

    let results = analyze_grammar_file(grammar, false)?;
    println!(
        "{} Grammar syntax is valid ({})!",
        "✅".green(),
        if results.len() == 1 {
            "1 grammar definition".to_string()
        } else {
            format!("{} grammar definitions", results.len())
        }
    );

    Ok(())
}

fn show_stats(grammar: &Path) -> Result<()> {
    let results = analyze_grammar_file(grammar, false)?;
    println!("{} Grammar statistics:", "📊".blue());

    for result in results {
        print_stats_summary(&result);
    }

    Ok(())
}

fn print_stats_summary(result: &BuildResult) {
    println!(
        "  {} {}",
        "Grammar:".bright_black(),
        result.grammar_name.bright_green()
    );
    println!(
        "    {} {}",
        "States:".bright_black(),
        result.build_stats.state_count
    );
    println!(
        "    {} {}",
        "Symbols:".bright_black(),
        result.build_stats.symbol_count
    );
    println!(
        "    {} {}",
        "Conflicts:".bright_black(),
        result.build_stats.conflict_cells
    );
}

fn print_version() {
    println!("adze {}", env!("CARGO_PKG_VERSION"));
}

fn released_version(version: &str) -> String {
    version
        .split_once('-')
        .map_or(version, |(base, _)| base)
        .to_string()
}

fn discover_local_adze_paths() -> Option<(PathBuf, PathBuf)> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let runtime_path = current.join("runtime");
        let tool_path = current.join("tool");
        if runtime_path.join("Cargo.toml").exists() && tool_path.join("Cargo.toml").exists() {
            return Some((runtime_path, tool_path));
        }
        if !current.pop() {
            return None;
        }
    }
}
