use anyhow::Result;
use clap::{Parser, Subcommand};
use xshell::Shell;

mod golden;
mod grammar_json;

#[derive(Parser)]
#[command(author, version, about = "Rust Sitter development tasks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate golden test files from tree-sitter
    GenerateGolden {
        /// Grammar to generate golden files for
        #[arg(value_enum)]
        grammar: Grammar,
        /// Force regeneration even if files exist
        #[arg(short, long)]
        force: bool,
    },
    /// Compare generated output against golden files
    DiffGolden {
        /// Grammar to compare
        #[arg(value_enum)]
        grammar: Grammar,
        /// Show detailed diff output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Update golden files with current output
    UpdateGolden {
        /// Grammar to update
        #[arg(value_enum)]
        grammar: Grammar,
    },
    /// Run all golden tests
    TestGolden {
        /// Show detailed output for failures
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum Grammar {
    Arithmetic,
    Javascript,
    Rust,
    Python,
    C,
}

impl Grammar {
    fn name(&self) -> &'static str {
        match self {
            Grammar::Arithmetic => "arithmetic",
            Grammar::Javascript => "javascript", 
            Grammar::Rust => "rust",
            Grammar::Python => "python",
            Grammar::C => "c",
        }
    }
    
    fn repo_url(&self) -> Option<&'static str> {
        match self {
            Grammar::Arithmetic => None, // Local example
            Grammar::Javascript => Some("https://github.com/tree-sitter/tree-sitter-javascript"),
            Grammar::Rust => Some("https://github.com/tree-sitter/tree-sitter-rust"),
            Grammar::Python => Some("https://github.com/tree-sitter/tree-sitter-python"),
            Grammar::C => Some("https://github.com/tree-sitter/tree-sitter-c"),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;
    
    match cli.command {
        Commands::GenerateGolden { grammar, force } => {
            golden::generate_golden(&sh, grammar, force)?;
        }
        Commands::DiffGolden { grammar, verbose } => {
            golden::diff_golden(&sh, grammar, verbose)?;
        }
        Commands::UpdateGolden { grammar } => {
            golden::update_golden(&sh, grammar)?;
        }
        Commands::TestGolden { verbose } => {
            golden::test_all_golden(&sh, verbose)?;
        }
    }
    
    Ok(())
}