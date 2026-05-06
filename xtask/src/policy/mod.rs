//! Policy-stack checkers.
//!
//! This module implements the semantic gates described in
//! `docs/policy/`. The three core checkers are:
//!
//! * [`no_panic`] — panic-family receipts (`policy/no-panic-allowlist.toml`).
//! * [`file_policy`] — non-Rust file receipts (`policy/non-rust-allowlist.toml`).
//! * [`lint_policy`] — Clippy / rustc lint baseline (`policy/clippy-lints.toml`).
//!
//! The combined entry point [`report`] aggregates all three into
//! `target/policy/reports/policy-summary.md`.
//!
//! All checkers default to **advisory** mode: they always write reports,
//! but only exit non-zero when invoked with `--strict`.

pub mod file_policy;
pub mod lint_policy;
pub mod no_panic;
pub mod report;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Repository-relative path to the report output directory.
pub const REPORTS_DIR: &str = "target/policy/reports";

/// Returns the workspace root by walking up from the current directory
/// looking for the root `Cargo.toml`.
pub fn workspace_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("read current dir")?;
    let mut dir: &Path = &cwd;
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.is_file() {
            // Look for `[workspace]` to ensure this is the root, not a member.
            let text = std::fs::read_to_string(&candidate)
                .with_context(|| format!("read {}", candidate.display()))?;
            if text.contains("[workspace]") {
                return Ok(dir.to_path_buf());
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => anyhow::bail!("could not locate workspace root from {}", cwd.display()),
        }
    }
}

/// Ensures `target/policy/reports/` exists and returns its path.
pub fn ensure_reports_dir(root: &Path) -> Result<PathBuf> {
    let dir = root.join(REPORTS_DIR);
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    Ok(dir)
}

/// Outcome shared across checkers: a count of findings and the path to
/// the human-readable report. `findings` of zero is a clean run.
pub struct CheckOutcome {
    pub label: &'static str,
    pub findings: usize,
    pub report_md: PathBuf,
}

/// True if `date` parses as ISO `YYYY-MM-DD` and lies in the past.
pub fn no_panic_is_expired(date: &str) -> bool {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d < chrono::Local::now().date_naive())
        .unwrap_or(false)
}

impl CheckOutcome {
    pub fn print_summary(&self) {
        if self.findings == 0 {
            eprintln!(
                "[policy:{}] OK — report at {}",
                self.label,
                self.report_md.display()
            );
        } else {
            eprintln!(
                "[policy:{}] {} finding(s) — see {}",
                self.label,
                self.findings,
                self.report_md.display()
            );
        }
    }
}
