use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

const RIPR_PR_DIR: &str = "target/ripr/pr";
const RIPR_REVIEW_DIR: &str = "target/ripr/review";

pub fn run_pr(check: bool) -> Result<()> {
    let workspace_root = crate::policy::workspace_root()?;
    let out_dir = workspace_root.join(RIPR_PR_DIR);

    if check {
        check_pr_contract(&out_dir)?;
        println!("ripr-pr: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("creating RIPR PR dir {}", out_dir.display()))?;
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    let ripr_bin = ripr_bin();

    run_command(
        std::process::Command::new(&ripr_bin)
            .arg("check")
            .arg("--root")
            .arg(&workspace_root)
            .arg("--format")
            .arg("repo-exposure-json")
            .current_dir(&workspace_root),
        &json_path,
        &format!("{ripr_bin} repo-exposure-json"),
    )?;

    run_command(
        std::process::Command::new(&ripr_bin)
            .arg("check")
            .arg("--root")
            .arg(&workspace_root)
            .arg("--format")
            .arg("repo-exposure-md")
            .current_dir(&workspace_root),
        &md_path,
        &format!("{ripr_bin} repo-exposure-md"),
    )?;

    check_pr_contract(&out_dir)?;
    println!("ripr-pr: wrote PR-scoped evidence under target/ripr/pr/");
    Ok(())
}

pub fn run_review_comments(check: bool) -> Result<()> {
    let workspace_root = crate::policy::workspace_root()?;
    let out_dir = workspace_root.join(RIPR_REVIEW_DIR);
    let json_path = out_dir.join("comments.json");
    let md_path = out_dir.join("comments.md");

    if check {
        check_review_contract(&json_path, &md_path)?;
        println!("ripr-review-comments: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("creating RIPR review dir {}", out_dir.display()))?;
    let ripr_bin = ripr_bin();
    let base = std::env::var("RIPR_BASE").unwrap_or_else(|_| "origin/main".to_string());
    let head = std::env::var("RIPR_HEAD").unwrap_or_else(|_| "HEAD".to_string());

    let output = std::process::Command::new(&ripr_bin)
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
        .with_context(|| format!("running {ripr_bin} review-comments"))?;

    if !output.status.success() {
        anyhow::bail!(
            "{ripr_bin} review-comments failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    write_review_markdown(&json_path, &md_path)?;
    check_review_contract(&json_path, &md_path)?;
    println!("ripr-review-comments: wrote review guidance under target/ripr/review/");
    Ok(())
}

fn ripr_bin() -> String {
    std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string())
}

fn run_command(command: &mut std::process::Command, output_path: &Path, label: &str) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("running {label}"))?;

    if !output.status.success() {
        anyhow::bail!(
            "{label} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    std::fs::write(output_path, output.stdout)
        .with_context(|| format!("writing {}", output_path.display()))
}

fn check_pr_contract(out_dir: &Path) -> Result<()> {
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    validate_json_file(&json_path)?;
    validate_nonempty_file(&md_path)?;
    Ok(())
}

fn check_review_contract(json_path: &Path, md_path: &Path) -> Result<()> {
    let value = validate_json_file(json_path)?;
    if !value.is_object() {
        anyhow::bail!("{} must contain a JSON object", json_path.display());
    }
    validate_nonempty_file(md_path)?;
    Ok(())
}

fn validate_json_file(path: &Path) -> Result<Value> {
    let text = validate_nonempty_file(path)?;
    serde_json::from_str(&text).with_context(|| format!("parsing JSON {}", path.display()))
}

fn validate_nonempty_file(path: &Path) -> Result<String> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading required RIPR output {}", path.display()))?;
    if text.trim().is_empty() {
        anyhow::bail!("required RIPR output {} is empty", path.display());
    }
    Ok(text)
}

fn write_review_markdown(json_path: &Path, md_path: &Path) -> Result<()> {
    let value = validate_json_file(json_path)?;
    let comment_count = value
        .get("comments")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let warning_count = value
        .get("warnings")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let summary = format!(
        "# RIPR review guidance\n\n- Review comments: {comment_count}\n- Warnings: {warning_count}\n"
    );

    std::fs::write(md_path, summary).with_context(|| format!("writing {}", md_path.display()))
}

#[allow(dead_code)]
fn _pr_dir_for_tests(workspace_root: &Path) -> PathBuf {
    workspace_root.join(RIPR_PR_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_contract_requires_json_object() {
        let dir = tempfile::tempdir().unwrap();
        let json_path = dir.path().join("comments.json");
        let md_path = dir.path().join("comments.md");
        std::fs::write(&json_path, "[]\n").unwrap();
        std::fs::write(&md_path, "# comments\n").unwrap();

        assert!(check_review_contract(&json_path, &md_path).is_err());
    }

    #[test]
    fn review_markdown_summarizes_json_contract() {
        let dir = tempfile::tempdir().unwrap();
        let json_path = dir.path().join("comments.json");
        let md_path = dir.path().join("comments.md");
        std::fs::write(
            &json_path,
            r#"{"comments":[{"path":"src/lib.rs","line":1}],"warnings":["stub"]}"#,
        )
        .unwrap();

        write_review_markdown(&json_path, &md_path).unwrap();

        let markdown = std::fs::read_to_string(md_path).unwrap();
        assert!(markdown.contains("Review comments: 1"));
        assert!(markdown.contains("Warnings: 1"));
    }
}
