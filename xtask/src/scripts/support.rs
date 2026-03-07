use anyhow::{Context, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

pub(crate) fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to resolve repository root")?;
    if output.status.success() {
        let root = String::from_utf8(output.stdout).context("repo root was not valid UTF-8")?;
        return Ok(PathBuf::from(root.trim()));
    }
    env::current_dir().context("failed to read current directory")
}

pub(crate) fn split_nul_output(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .filter_map(|entry| String::from_utf8(entry.to_vec()).ok())
        .collect()
}

pub(crate) fn normalize_existing_path(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).with_context(|| format!("failed to canonicalize {}", path.display()))
}

pub(crate) fn combine_output(output: &Output) -> String {
    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        if !combined.ends_with('\n') {
            combined.push('\n');
        }
        combined.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    combined
}

pub(crate) fn combined_output_lines(output: &Output) -> Vec<String> {
    combine_output(output)
        .lines()
        .map(|line| line.to_owned())
        .collect()
}

pub(crate) fn env_truthy(key: &str) -> bool {
    env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub(crate) fn is_missing(value: Option<&str>) -> bool {
    value.is_none_or(|value| value.trim().is_empty())
}

pub(crate) fn pass(message: &str) {
    println!("  [ok] {message}");
}

pub(crate) fn fail(message: &str, errors: &mut usize) {
    println!("  [fail] {message}");
    *errors += 1;
}

pub(crate) fn warn(message: &str, warnings: &mut usize) {
    println!("  [warn] {message}");
    *warnings += 1;
}

pub(crate) fn info(message: &str) {
    println!("  [info] {message}");
}
