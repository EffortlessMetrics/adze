//! CLI argument parsing tests for adze.
//!
//! These tests validate that clap argument parsing works correctly
//! without running the actual commands (no end-to-end execution).

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// End-to-end smoke tests (lightweight — no grammar compilation)
// ---------------------------------------------------------------------------

#[test]
fn test_cli_help() {
    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Adze CLI - Tools for grammar development",
        ));
}

#[test]
fn test_cli_help_subcommand() {
    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("stats"))
        .stdout(predicate::str::contains("version"));
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("adze"));
}

#[test]
fn test_cli_version_subcommand() {
    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("adze"));
}

#[test]
fn test_cli_no_args_shows_help() {
    let mut cmd = cargo_bin_cmd!("adze");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_cli_unknown_command() {
    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_init_generates_project_that_checks() {
    let temp = tempdir().expect("tempdir");
    let project_name = "fresh-lang";
    let project_dir = temp.path().join(project_name);

    let mut init_cmd = cargo_bin_cmd!("adze");
    init_cmd
        .args(["init", project_name, "--output"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Project created"));

    assert!(project_dir.join("Cargo.toml").exists());
    assert!(project_dir.join("src/grammar.rs").exists());
    assert!(project_dir.join("tests/basic.rs").exists());

    let status = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(&project_dir)
        .status()
        .expect("run cargo check for generated project");
    assert!(
        status.success(),
        "generated project should pass cargo check"
    );
}

#[test]
fn test_parse_static_mode_is_explicitly_unimplemented() {
    let temp = tempdir().expect("tempdir");
    let grammar_file = temp.path().join("grammar.rs");
    let input_file = temp.path().join("input.txt");
    std::fs::write(&grammar_file, "// placeholder grammar").expect("write grammar");
    std::fs::write(&input_file, "x").expect("write input");

    let mut cmd = cargo_bin_cmd!("adze");
    cmd.arg("parse")
        .arg(&grammar_file)
        .arg(&input_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "static parse mode is not implemented yet",
        ))
        .stderr(predicate::str::contains("experimental"));
}

// ---------------------------------------------------------------------------
// Unit tests for CLI argument parsing (no binary execution)
// ---------------------------------------------------------------------------

// Import the CLI types directly for parsing tests.
// The types are pub(crate) so we use `try_parse_from` on the binary's types
// via the clap trait. We re-derive a minimal mirror here to avoid exposing
// internal types outside the crate.
mod parsing {
    use clap::Parser;

    /// Minimal mirror of the real CLI struct for argument parsing tests.
    #[derive(Parser, Debug)]
    #[command(name = "adze")]
    #[command(about = "Adze CLI - Tools for grammar development")]
    #[command(author, version, long_about = None)]
    struct Cli {
        #[arg(short, long, global = true)]
        verbose: bool,

        #[command(subcommand)]
        command: Commands,
    }

    #[derive(clap::Subcommand, Debug)]
    enum Commands {
        Init {
            name: String,
            #[arg(short, long)]
            output: Option<std::path::PathBuf>,
        },
        Build {
            #[arg(default_value = ".")]
            path: std::path::PathBuf,
            #[arg(short, long)]
            watch: bool,
        },
        Parse {
            grammar: std::path::PathBuf,
            input: std::path::PathBuf,
            #[arg(short, long, default_value = "tree")]
            format: OutputFormat,
            #[arg(long)]
            dynamic: bool,
            #[arg(long, default_value = "language")]
            symbol: String,
        },
        Test {
            #[arg(default_value = ".")]
            path: std::path::PathBuf,
            #[arg(short, long)]
            update: bool,
        },
        Doc {
            grammar: std::path::PathBuf,
            #[arg(short, long)]
            output: Option<std::path::PathBuf>,
        },
        Check {
            grammar: std::path::PathBuf,
        },
        Stats {
            grammar: std::path::PathBuf,
        },
        Version,
    }

    #[derive(clap::ValueEnum, Clone, Debug)]
    enum OutputFormat {
        Tree,
        Json,
        Sexp,
        Dot,
    }

    // --- argument parsing unit tests ---

    #[test]
    fn parse_check_subcommand() {
        let cli = Cli::try_parse_from(["adze", "check", "grammar.rs"]).unwrap();
        assert!(!cli.verbose);
        match cli.command {
            Commands::Check { grammar } => {
                assert_eq!(grammar.to_str().unwrap(), "grammar.rs");
            }
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn parse_stats_subcommand() {
        let cli = Cli::try_parse_from(["adze", "stats", "my_grammar.rs"]).unwrap();
        match cli.command {
            Commands::Stats { grammar } => {
                assert_eq!(grammar.to_str().unwrap(), "my_grammar.rs");
            }
            _ => panic!("expected Stats command"),
        }
    }

    #[test]
    fn parse_version_subcommand() {
        let cli = Cli::try_parse_from(["adze", "version"]).unwrap();
        assert!(matches!(cli.command, Commands::Version));
    }

    #[test]
    fn parse_verbose_flag_global() {
        let cli = Cli::try_parse_from(["adze", "-v", "version"]).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn parse_init_with_output() {
        let cli = Cli::try_parse_from(["adze", "init", "my-lang", "-o", "/tmp/out"]).unwrap();
        match cli.command {
            Commands::Init { name, output } => {
                assert_eq!(name, "my-lang");
                assert_eq!(output.unwrap().to_str().unwrap(), "/tmp/out");
            }
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn parse_build_defaults() {
        let cli = Cli::try_parse_from(["adze", "build"]).unwrap();
        match cli.command {
            Commands::Build { path, watch } => {
                assert_eq!(path.to_str().unwrap(), ".");
                assert!(!watch);
            }
            _ => panic!("expected Build command"),
        }
    }

    #[test]
    fn parse_build_with_watch() {
        let cli = Cli::try_parse_from(["adze", "build", "src/", "--watch"]).unwrap();
        match cli.command {
            Commands::Build { path, watch } => {
                assert_eq!(path.to_str().unwrap(), "src/");
                assert!(watch);
            }
            _ => panic!("expected Build command"),
        }
    }

    #[test]
    fn parse_parse_command_full() {
        let cli = Cli::try_parse_from([
            "adze",
            "parse",
            "gram.rs",
            "input.txt",
            "--format",
            "json",
            "--dynamic",
            "--symbol",
            "my_lang",
        ])
        .unwrap();
        match cli.command {
            Commands::Parse {
                grammar,
                input,
                dynamic,
                symbol,
                ..
            } => {
                assert_eq!(grammar.to_str().unwrap(), "gram.rs");
                assert_eq!(input.to_str().unwrap(), "input.txt");
                assert!(dynamic);
                assert_eq!(symbol, "my_lang");
            }
            _ => panic!("expected Parse command"),
        }
    }

    #[test]
    fn parse_test_with_update() {
        let cli = Cli::try_parse_from(["adze", "test", "--update"]).unwrap();
        match cli.command {
            Commands::Test { path, update } => {
                assert_eq!(path.to_str().unwrap(), ".");
                assert!(update);
            }
            _ => panic!("expected Test command"),
        }
    }

    #[test]
    fn parse_doc_subcommand() {
        let cli = Cli::try_parse_from(["adze", "doc", "grammar.rs", "-o", "docs.md"]).unwrap();
        match cli.command {
            Commands::Doc { grammar, output } => {
                assert_eq!(grammar.to_str().unwrap(), "grammar.rs");
                assert_eq!(output.unwrap().to_str().unwrap(), "docs.md");
            }
            _ => panic!("expected Doc command"),
        }
    }

    #[test]
    fn parse_check_missing_arg_fails() {
        assert!(Cli::try_parse_from(["adze", "check"]).is_err());
    }

    #[test]
    fn parse_stats_missing_arg_fails() {
        assert!(Cli::try_parse_from(["adze", "stats"]).is_err());
    }

    #[test]
    fn parse_unknown_subcommand_fails() {
        assert!(Cli::try_parse_from(["adze", "foobar"]).is_err());
    }

    #[test]
    fn parse_no_subcommand_fails() {
        assert!(Cli::try_parse_from(["adze"]).is_err());
    }
}
