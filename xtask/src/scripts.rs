use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;
use xshell::Shell;

mod affected;
mod clippy;
mod metadata;
mod publish_order;
mod publish_ready;
mod release_surface;
mod support;

#[derive(Subcommand, Debug)]
pub(crate) enum ScriptsCommand {
    /// Emit the resolved release surface
    ReleaseSurface {
        /// Release-surface mode
        #[arg(long, value_enum)]
        mode: Option<release_surface::ReleaseSurfaceMode>,
        /// Override release crate allowlist path
        #[arg(long = "crate-file", value_name = "PATH")]
        crate_file: Option<PathBuf>,
        /// In auto mode, sync the computed order back to the crate file
        #[arg(long)]
        sync: bool,
    },
    /// Validate the configured release surface against workspace metadata
    ValidateReleaseSurface {
        /// Release-surface mode
        #[arg(long, value_enum)]
        mode: Option<release_surface::ReleaseSurfaceMode>,
        /// Override release crate allowlist path
        #[arg(long = "crate-file", value_name = "PATH")]
        crate_file: Option<PathBuf>,
        /// Fail when publishable crates are omitted from the fixed allowlist
        #[arg(long)]
        strict: bool,
    },
    /// Analyze publish order for publishable workspace crates
    PublishOrder {
        /// Run cargo publish --dry-run for each crate in order
        #[arg(long)]
        dry_run: bool,
        /// Only validate metadata requirements
        #[arg(long = "validate")]
        validate_only: bool,
    },
    /// Print the staged crates affected by Rust/Cargo manifest changes
    AffectedCrates,
    /// Verify the core publish surface is crates.io-ready
    CheckPublishReady {
        /// Compatibility flag retained from the shell script
        #[arg(long)]
        fix: bool,
    },
    /// Verify the supported publish surface is crates.io-ready
    CheckPublishReadiness,
    /// Run clippy package-by-package, skipping quarantined crates
    ClippyPerPackage {
        /// Mode to run
        #[arg(value_enum, default_value = "default")]
        mode: clippy::ClippyMode,
    },
    /// Collect clippy output logs per package
    ClippyCollect {
        /// Output directory for logs
        #[arg(long, default_value = "clippy-report")]
        outdir: PathBuf,
    },
}

pub(crate) fn run(_sh: &Shell, command: ScriptsCommand) -> Result<()> {
    match command {
        ScriptsCommand::ReleaseSurface {
            mode,
            crate_file,
            sync,
        } => release_surface::run_release_surface(mode, crate_file, sync),
        ScriptsCommand::ValidateReleaseSurface {
            mode,
            crate_file,
            strict,
        } => release_surface::run_validate_release_surface(mode, crate_file, strict),
        ScriptsCommand::PublishOrder {
            dry_run,
            validate_only,
        } => publish_order::run_publish_order(dry_run, validate_only),
        ScriptsCommand::AffectedCrates => affected::run_affected_crates(),
        ScriptsCommand::CheckPublishReady { fix } => publish_ready::run_publish_ready_core(fix),
        ScriptsCommand::CheckPublishReadiness => publish_ready::run_publish_readiness(),
        ScriptsCommand::ClippyPerPackage { mode } => clippy::run_clippy_per_package(mode),
        ScriptsCommand::ClippyCollect { outdir } => clippy::run_clippy_collect(&outdir),
    }
}
