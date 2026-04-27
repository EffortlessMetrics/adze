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
    let dependency_block = scaffold_dependency_block(&project_dir)?;
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
{}

[build-dependencies]
{}

[dev-dependencies]
insta = "1.40"
"#,
        name, dependency_block.0, dependency_block.1
    );

    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create build.rs
    let build_rs = r#"fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
}
"#;

    fs::write(project_dir.join("build.rs"), build_rs)?;

    // Create example grammar
    let lib_rs = format!(
        r#"//! {} grammar definition

/// Returns the current parser scaffold status.
///
/// This starter project is buildable immediately. To add parsing:
/// 1. define an `#[adze::grammar(...)]` module in `src/lib.rs`,
/// 2. call `adze_tool::build_parsers(...)` from `build.rs`.
pub fn parser_status() -> &'static str {{
    "Parser generation is not configured yet."
}}

#[cfg(test)]
mod tests {{
    #[test]
    fn test_scaffold_status_message_is_truthful() {{
        assert!(crate::parser_status().contains("not configured"));
    }}
}}
"#,
        name
    );

    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    // Create test that confirms the scaffold is truthful about next steps.
    let crate_ident = sanitize_crate_identifier(name);
    let test_rs = format!(
        r#"use {} as generated;

#[test]
fn test_scaffold_status_message() {{
    assert!(generated::parser_status().contains("not configured"));
}}
"#,
        crate_ident
    );

    fs::write(project_dir.join("tests/basic.rs"), test_rs)?;

    // Create README
    let readme = format!(
        r#"# {}

A adze grammar for {}.

## Usage

Build generated parser artifacts:

```bash
cargo build
```

Run grammar checks and tests:

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

fn scaffold_dependency_block(project_dir: &Path) -> Result<(String, String)> {
    if let Some((adze_path, tool_path)) = find_workspace_dependency_paths(project_dir) {
        return Ok((
            format!("adze = {{ path = \"{}\" }}", adze_path.display()),
            format!("adze-tool = {{ path = \"{}\" }}", tool_path.display()),
        ));
    }

    let version = env!("CARGO_PKG_VERSION");
    Ok((
        format!("adze = \"{version}\""),
        format!("adze-tool = \"{version}\""),
    ))
}

fn sanitize_crate_identifier(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    if out.is_empty() || out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        out.insert(0, '_');
    }

    out
}

fn find_workspace_dependency_paths(project_dir: &Path) -> Option<(PathBuf, PathBuf)> {
    let current_dir = std::env::current_dir().ok()?;

    for ancestor in current_dir.ancestors() {
        let runtime = ancestor.join("runtime").join("Cargo.toml");
        let tool = ancestor.join("tool").join("Cargo.toml");

        if runtime.exists() && tool.exists() {
            let runtime_rel = path_relative_to(&ancestor.join("runtime"), project_dir)?;
            let tool_rel = path_relative_to(&ancestor.join("tool"), project_dir)?;
            return Some((runtime_rel, tool_rel));
        }
    }

    None
}

fn path_relative_to(target: &Path, from: &Path) -> Option<PathBuf> {
    let target = target.canonicalize().ok()?;
    let from = from.canonicalize().ok()?;

    let target_components: Vec<_> = target.components().collect();
    let from_components: Vec<_> = from.components().collect();

    let common = target_components
        .iter()
        .zip(from_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut rel = PathBuf::new();

    for _ in common..from_components.len() {
        rel.push("..");
    }

    for component in target_components.iter().skip(common) {
        rel.push(component.as_os_str());
    }

    Some(rel)
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
    println!("{} Parsing file: {}", "📄".blue(), input.display());

    let input_content = fs::read_to_string(input)?;
    println!(
        "  Grammar: {}\n  Input: {} ({} bytes)",
        grammar.display(),
        input.display(),
        input_content.len()
    );

    if dynamic {
        #[cfg(feature = "dynamic")]
        {
            return parse_file_dynamic(grammar, input, _format, _symbol);
        }
        #[cfg(not(feature = "dynamic"))]
        {
            anyhow::bail!(
                "Dynamic parse mode is experimental and requires building adze-cli with --features dynamic"
            );
        }
    }

    anyhow::bail!(
        "Static `adze parse` mode is not implemented yet. This command is experimental; use `adze check`/`adze build` and parse via generated Rust APIs for now"
    )
}

#[cfg(feature = "dynamic")]
fn parse_file_dynamic(
    grammar: &Path,
    input: &Path,
    format: OutputFormat,
    _symbol: &str,
) -> Result<()> {
    let _ = (grammar, input, format);
    anyhow::bail!(
        "Dynamic `adze parse --dynamic` is experimental and not implemented yet. Grammar loading exists, but parser execution is pending"
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
