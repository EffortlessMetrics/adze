//! Policy stack: shared utilities for the no-panic / file-policy / lint-policy
//! checks. Kept deliberately minimal — the checks must run in CI quickly and
//! without external network calls.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub mod ci_lane_whitelist;
pub mod file_policy;
pub mod lint_policy;
pub mod no_panic;
pub mod package_boundary;
pub mod report;

/// Where the checks write their JSON/Markdown artefacts.
pub fn report_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join("target").join("policy")
}

pub fn ensure_report_dir(workspace_root: &Path) -> Result<PathBuf> {
    let dir = report_dir(workspace_root);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("creating report dir {}", dir.display()))?;
    Ok(dir)
}

/// Operating mode for the policy checks. Advisory by default; we will flip to
/// `BlockingAllowlist` once the baseline is committed and burned down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Report findings, never fail.
    Advisory,
    /// Fail on unallowlisted findings.
    BlockingAllowlist,
    /// Fail on unallowlisted, stale, or expired entries.
    BlockingStrict,
}

impl Mode {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "advisory" => Ok(Mode::Advisory),
            "blocking-allowlist" => Ok(Mode::BlockingAllowlist),
            "blocking-strict" => Ok(Mode::BlockingStrict),
            other => anyhow::bail!(
                "unknown policy mode: {other} (expected advisory|blocking-allowlist|blocking-strict)"
            ),
        }
    }
}

/// Resolve the workspace root by walking up from CWD until we find a
/// `Cargo.toml` containing `[workspace]`.
pub fn workspace_root() -> Result<PathBuf> {
    let mut cur = std::env::current_dir()?;
    loop {
        let candidate = cur.join("Cargo.toml");
        if candidate.exists() {
            let text = std::fs::read_to_string(&candidate)?;
            if text.contains("[workspace]") {
                return Ok(cur);
            }
        }
        if !cur.pop() {
            anyhow::bail!("could not locate workspace root from current directory");
        }
    }
}
