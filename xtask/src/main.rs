use anyhow::Result;
use clap::{Parser, Subcommand};
use xshell::Shell;

mod baseline;
mod bench;
mod corpus;
mod dashboard;
mod golden;
mod grammar_json;
mod lint;
mod profile;
mod test_grammars;
mod test_local_grammars;

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
    /// Download Tree-sitter grammar corpus
    DownloadCorpus {
        /// Target directory for corpus
        #[arg(short, long, default_value = "./corpus")]
        target: String,
    },
    /// Test grammars against Tree-sitter corpus
    TestCorpus {
        /// Path to corpus directory
        #[arg(short, long, default_value = "./corpus")]
        corpus: String,
        /// Output directory for results
        #[arg(short, long, default_value = "./target/corpus-results")]
        output: String,
    },
    /// Test a specific grammar from the corpus
    TestGrammar {
        /// Grammar name (e.g., javascript, rust, python)
        grammar: String,
        /// Path to corpus directory
        #[arg(short, long, default_value = "./corpus")]
        corpus: String,
    },
    /// Generate dashboard data from test results
    DashboardData {
        /// Input directory with test results
        #[arg(short, long, default_value = "./target/corpus-results")]
        input: String,
        /// Output file for dashboard data
        #[arg(short, long, default_value = "./dashboard/data.json")]
        output: String,
    },
    /// Initialize dashboard project
    InitDashboard {
        /// Dashboard directory
        #[arg(short, long, default_value = "./dashboard")]
        dir: String,
    },
    /// Test top 20 grammars for compatibility
    TestGrammars {
        /// Output format
        #[arg(short, long, value_enum, default_value = "markdown")]
        format: OutputFormat,
    },
    /// Test local grammar examples
    TestLocal,
    /// Test fixture grammars with pure-Rust backend
    TestPureRust {
        /// Grammar to test (python, rust, c)
        #[arg(value_enum)]
        grammar: Grammar,
        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run benchmarks with optional baseline saving
    Bench {
        /// Save results as a new baseline
        #[arg(long)]
        save_baseline: bool,
        /// Baseline name (defaults to version from Cargo.toml)
        #[arg(long)]
        baseline_name: Option<String>,
    },
    /// Profile CPU or memory usage
    Profile {
        /// Profile type: cpu or memory
        #[arg(value_enum)]
        profile_type: ProfileType,
        /// Grammar to profile
        #[arg(value_enum)]
        grammar: ProfileGrammar,
        /// Fixture size
        #[arg(value_enum)]
        size: FixtureSize,
        /// Output JSON metrics
        #[arg(long)]
        json: bool,
    },
    /// Save current benchmark results as a baseline (without running benchmarks)
    SaveBaseline {
        /// Baseline version name (e.g., "v0.8.0")
        version: String,
    },
    /// Compare current benchmarks against baseline
    CompareBaseline {
        /// Baseline version to compare against (e.g., "v0.8.0")
        baseline_version: String,
        /// Regression threshold percentage (default: 5.0)
        #[arg(long, default_value = "5.0")]
        threshold: f64,
    },
    /// Run all lint checks (fmt -> no-mangle -> debug-block validator -> clippy)
    ///
    /// Examples:
    ///   cargo xtask lint --fast               # 3-5s: fmt/validator/no-mangle + clippy on core crates
    ///   cargo xtask lint --changed-only       # pre-commit mirror (staged .rs)
    ///   cargo xtask lint --since origin/main  # PR-diff mirror
    ///   cargo xtask lint --fix                # auto-fix formatting and debug blocks
    Lint {
        /// Auto-fix debug blocks (adds `// );` where missing) and run `cargo fmt` write-mode
        #[arg(long)]
        fix: bool,
        /// Only scan staged .rs files (uses Git index)
        #[arg(long)]
        changed_only: bool,
        /// Scan diff since a Git rev/range (e.g. `main`, `origin/main`, `abc123..HEAD`)
        #[arg(long, value_name = "REV")]
        since: Option<String>,
        /// Fast mode: skip self-tests and limit clippy to core crates (3-5s checks)
        #[arg(long)]
        fast: bool,
        /// Extra args passed to `cargo clippy` after `--`
        #[arg(last = true)]
        clippy_args: Vec<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum OutputFormat {
    Markdown,
    Json,
    Console,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Grammar {
    Arithmetic,
    Javascript,
    Rust,
    Python,
    C,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum ProfileType {
    Cpu,
    Memory,
}

impl From<ProfileType> for profile::ProfileType {
    fn from(pt: ProfileType) -> Self {
        match pt {
            ProfileType::Cpu => profile::ProfileType::Cpu,
            ProfileType::Memory => profile::ProfileType::Memory,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum ProfileGrammar {
    Python,
    Javascript,
    Arithmetic,
}

impl From<ProfileGrammar> for profile::ProfileGrammar {
    fn from(pg: ProfileGrammar) -> Self {
        match pg {
            ProfileGrammar::Python => profile::ProfileGrammar::Python,
            ProfileGrammar::Javascript => profile::ProfileGrammar::Javascript,
            ProfileGrammar::Arithmetic => profile::ProfileGrammar::Arithmetic,
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum FixtureSize {
    Small,
    Medium,
    Large,
}

impl From<FixtureSize> for profile::FixtureSize {
    fn from(fs: FixtureSize) -> Self {
        match fs {
            FixtureSize::Small => profile::FixtureSize::Small,
            FixtureSize::Medium => profile::FixtureSize::Medium,
            FixtureSize::Large => profile::FixtureSize::Large,
        }
    }
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
        Commands::DownloadCorpus { target } => {
            corpus::download_corpus(std::path::Path::new(&target))?;
        }
        Commands::TestCorpus { corpus, output } => {
            let runner = corpus::CorpusRunner::new(corpus.into(), output.into());
            let results = runner.run_all()?;
            println!(
                "\nCorpus test complete: {:.1}% pass rate",
                results.pass_rate
            );
        }
        Commands::TestGrammar { grammar, corpus } => {
            let runner = corpus::CorpusRunner::new(corpus.into(), "./target/corpus-results".into());
            let result = runner.test_grammar(&grammar)?;
            println!("Grammar {} status: {:?}", grammar, result.status);
        }
        Commands::DashboardData { input, output } => {
            dashboard::generate_dashboard_data(
                std::path::Path::new(&input),
                std::path::Path::new(&output),
            )?;
        }
        Commands::InitDashboard { dir } => {
            dashboard::init_dashboard(std::path::Path::new(&dir))?;
        }
        Commands::TestGrammars { format: _ } => {
            test_grammars::run_corpus_tests()?;
        }
        Commands::TestLocal => {
            test_local_grammars::test_local_grammars()?;
        }
        Commands::TestPureRust { grammar, verbose } => {
            test_grammars::test_pure_rust(&sh, grammar, verbose)?;
        }
        Commands::Bench {
            save_baseline,
            baseline_name,
        } => {
            bench::run_benchmarks(&sh, save_baseline, baseline_name)?;
        }
        Commands::Profile {
            profile_type,
            grammar,
            size,
            json,
        } => {
            profile::profile(&sh, profile_type.into(), grammar.into(), size.into(), json)?;
        }
        Commands::SaveBaseline { version } => {
            baseline::save_baseline(&sh, &version)?;
        }
        Commands::CompareBaseline {
            baseline_version,
            threshold,
        } => {
            baseline::compare_baseline(&sh, &baseline_version, threshold)?;
        }
        Commands::Lint {
            fix,
            changed_only,
            since,
            fast,
            clippy_args,
        } => {
            lint::lint(&sh, fix, changed_only, since, fast, clippy_args)?;
        }
    }

    Ok(())
}
