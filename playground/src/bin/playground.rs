// CLI entry point for rust-sitter playground

use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_sitter_playground::{PlaygroundBuilder, PlaygroundFeature};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rust-sitter-playground")]
#[command(about = "Interactive grammar testing playground for rust-sitter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive CLI playground
    Cli {
        /// Path to grammar file
        #[arg(short, long)]
        grammar: PathBuf,

        /// Path to test file
        #[arg(short, long)]
        tests: Option<PathBuf>,
    },

    /// Launch web playground server
    Web {
        /// Path to grammar file
        #[arg(short, long)]
        grammar: PathBuf,

        /// Server port
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Path to test file
        #[arg(short, long)]
        tests: Option<PathBuf>,
    },

    /// Run tests without interactive mode
    Test {
        /// Path to grammar file
        #[arg(short, long)]
        grammar: PathBuf,

        /// Path to test file
        #[arg(short, long)]
        tests: PathBuf,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Analyze grammar
    Analyze {
        /// Path to grammar file
        #[arg(short, long)]
        grammar: PathBuf,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Cli { grammar, tests } => {
            println!("🚀 Launching CLI playground...");

            let mut builder = PlaygroundBuilder::new()
                .grammar(grammar.to_string_lossy())
                .feature(PlaygroundFeature::CliInterface);

            if let Some(test_file) = tests {
                builder = builder.tests(test_file.to_string_lossy());
            }

            builder.build()?;
        }

        Commands::Web {
            grammar,
            port,
            tests,
        } => {
            println!("🌐 Launching web playground on port {}...", port);

            let mut builder = PlaygroundBuilder::new()
                .grammar(grammar.to_string_lossy())
                .feature(PlaygroundFeature::WebInterface(port));

            if let Some(test_file) = tests {
                builder = builder.tests(test_file.to_string_lossy());
            }

            builder.build()?;
        }

        Commands::Test {
            grammar,
            tests,
            format,
        } => {
            println!("🧪 Running tests...");

            let _builder = PlaygroundBuilder::new()
                .grammar(grammar.to_string_lossy())
                .tests(tests.to_string_lossy())
                .feature(PlaygroundFeature::TestRunner)
                .build()?;

            // Format output based on requested format
            match format.as_str() {
                "json" => {
                    // JSON output would be implemented here
                }
                _ => {
                    // Text output is default
                }
            }
        }

        Commands::Analyze { grammar, format } => {
            println!("🔍 Analyzing grammar...");

            let _builder = PlaygroundBuilder::new()
                .grammar(grammar.to_string_lossy())
                .feature(PlaygroundFeature::Analysis)
                .build()?;

            // Format output based on requested format
            match format.as_str() {
                "json" => {
                    // JSON output would be implemented here
                }
                _ => {
                    // Text output is default
                }
            }
        }
    }

    Ok(())
}
