use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

const RIPR_PR_DIR: &str = "target/ripr/pr";
const RIPR_REVIEW_DIR: &str = "target/ripr/review";

pub(crate) fn pr(check: bool, base: Option<String>, head: Option<String>) -> Result<()> {
    let workspace_root = crate::policy::workspace_root()?;
    let out_dir = workspace_root.join(RIPR_PR_DIR);

    if check {
        return check_pr_contract(&out_dir);
    }

    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let base = resolve_base(base);
    let head = head.unwrap_or_else(|| "HEAD".to_string());
    let ripr_bin = ripr_bin();

    let json = run_ripr_capture(
        &ripr_bin,
        &workspace_root,
        &[
            "check",
            "--root",
            workspace_root.as_os_str().to_string_lossy().as_ref(),
            "--base",
            &base,
            "--head",
            &head,
            "--format",
            "repo-exposure-json",
        ],
        "repo exposure JSON",
    )?;
    let md = run_ripr_capture(
        &ripr_bin,
        &workspace_root,
        &[
            "check",
            "--root",
            workspace_root.as_os_str().to_string_lossy().as_ref(),
            "--base",
            &base,
            "--head",
            &head,
            "--format",
            "repo-exposure-md",
        ],
        "repo exposure Markdown",
    )?;

    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    std::fs::write(&json_path, json)
        .with_context(|| format!("failed to write {}", json_path.display()))?;
    std::fs::write(&md_path, md)
        .with_context(|| format!("failed to write {}", md_path.display()))?;

    check_pr_contract(&out_dir)?;
    println!("ripr-pr: wrote PR-scoped evidence under target/ripr/pr/");
    Ok(())
}

pub(crate) fn review_comments(
    check: bool,
    base: Option<String>,
    head: Option<String>,
) -> Result<()> {
    let workspace_root = crate::policy::workspace_root()?;
    let out_dir = workspace_root.join(RIPR_REVIEW_DIR);
    let json_path = out_dir.join("comments.json");
    let md_path = out_dir.join("comments.md");

    if check {
        validate_json_file(&json_path)?;
        require_non_empty(&md_path)?;
        println!("ripr-review-comments: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let base = resolve_base(base);
    let head = head.unwrap_or_else(|| "HEAD".to_string());
    let ripr_bin = ripr_bin();
    let output = Command::new(&ripr_bin)
        .arg("review-comments")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(&base)
        .arg("--head")
        .arg(&head)
        .arg("--out")
        .arg(&json_path)
        .current_dir(&workspace_root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin} review-comments"))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} review-comments failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    validate_json_file(&json_path)?;
    require_non_empty(&md_path)?;
    println!("ripr-review-comments: wrote review guidance under target/ripr/review/");
    Ok(())
}

fn check_pr_contract(out_dir: &Path) -> Result<()> {
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    validate_json_file(&json_path)?;
    require_non_empty(&md_path)?;
    println!("ripr-pr: output contract is intact");
    Ok(())
}

fn validate_json_file(path: &Path) -> Result<Value> {
    let bytes =
        std::fs::read(path).with_context(|| format!("missing required file {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("invalid JSON in {}", path.display()))
}

fn require_non_empty(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("missing required file {}", path.display()))?;
    if metadata.len() == 0 {
        bail!("required file is empty: {}", path.display());
    }
    Ok(())
}

fn resolve_base(base: Option<String>) -> String {
    base.or_else(|| std::env::var("RIPR_BASE").ok())
        .unwrap_or_else(|| "origin/main".to_string())
}

fn ripr_bin() -> String {
    std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string())
}

fn run_ripr_capture(
    ripr_bin: &str,
    workspace_root: &Path,
    args: &[&str],
    description: &str,
) -> Result<Vec<u8>> {
    let output = Command::new(ripr_bin)
        .args(args)
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin} for {description}"))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} {description} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    if output.stdout.is_empty() {
        bail!("{ripr_bin} {description} emitted empty output");
    }

    Ok(output.stdout)
}

#[allow(dead_code)]
fn _target_path(workspace_root: &Path, rel: &str) -> PathBuf {
    workspace_root.join(rel)
}
