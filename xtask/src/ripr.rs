use anyhow::{Context, Result};
use serde_json::json;
use std::path::Path;
use std::process::Command;

use crate::policy;

const PR_DIR: &str = "target/ripr/pr";
const REVIEW_DIR: &str = "target/ripr/review";

pub struct RiprArgs {
    pub check: bool,
    pub base: String,
    pub head: String,
}

pub fn run_pr(args: RiprArgs) -> Result<()> {
    let workspace_root = policy::workspace_root()?;
    let dir = workspace_root.join(PR_DIR);
    if args.check {
        check_pr_contract(&dir)?;
        println!("ripr-pr: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let json_path = dir.join("repo-exposure.json");
    let md_path = dir.join("repo-exposure.md");
    let ripr_bin = ripr_bin();

    let json_result = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(&args.base)
        .arg("--head")
        .arg(&args.head)
        .arg("--format")
        .arg("repo-exposure-json")
        .current_dir(&workspace_root)
        .output();

    match json_result {
        Ok(output) if output.status.success() => {
            let value: serde_json::Value = serde_json::from_slice(&output.stdout)
                .with_context(|| "ripr emitted invalid repo exposure JSON")?;
            write_json(&json_path, &value)?;
            write_markdown_report(&ripr_bin, &workspace_root, &args, &md_path)?;
        }
        Ok(output) => {
            write_unavailable_pr(
                &json_path,
                &md_path,
                &format!(
                    "{ripr_bin} repo-exposure-json failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
                &args,
            )?;
        }
        Err(err) => {
            write_unavailable_pr(
                &json_path,
                &md_path,
                &format!("could not run {ripr_bin}: {err}"),
                &args,
            )?;
        }
    }

    println!("ripr-pr: wrote {}", dir.display());
    Ok(())
}

pub fn run_review_comments(args: RiprArgs) -> Result<()> {
    let workspace_root = policy::workspace_root()?;
    let dir = workspace_root.join(REVIEW_DIR);
    if args.check {
        check_review_contract(&dir)?;
        println!("ripr-review-comments: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let json_path = dir.join("comments.json");
    let md_path = dir.join("comments.md");
    let ripr_bin = ripr_bin();

    let output = Command::new(&ripr_bin)
        .arg("review-comments")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(&args.base)
        .arg("--head")
        .arg(&args.head)
        .arg("--out")
        .arg(&json_path)
        .current_dir(&workspace_root)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            validate_json_file(&json_path)?;
            if !md_path.exists() {
                std::fs::write(
                    &md_path,
                    "# RIPR Review Guidance\n\nNo line-placeable guidance was produced.\n",
                )
                .with_context(|| format!("writing {}", md_path.display()))?;
            }
        }
        Ok(output) => {
            write_unavailable_review(
                &json_path,
                &md_path,
                &format!(
                    "{ripr_bin} review-comments failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
                &args,
            )?;
        }
        Err(err) => {
            write_unavailable_review(
                &json_path,
                &md_path,
                &format!("could not run {ripr_bin}: {err}"),
                &args,
            )?;
        }
    }

    println!("ripr-review-comments: wrote {}", dir.display());
    Ok(())
}

fn write_markdown_report(
    ripr_bin: &str,
    workspace_root: &Path,
    args: &RiprArgs,
    md_path: &Path,
) -> Result<()> {
    let output = Command::new(ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(workspace_root)
        .arg("--base")
        .arg(&args.base)
        .arg("--head")
        .arg(&args.head)
        .arg("--format")
        .arg("repo-exposure-md")
        .current_dir(workspace_root)
        .output();

    match output {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => {
            std::fs::write(md_path, output.stdout)
                .with_context(|| format!("writing {}", md_path.display()))?;
        }
        _ => {
            std::fs::write(
                md_path,
                format!(
                    "# RIPR PR Evidence\n\nGenerated JSON evidence for `{}`..`{}`.\n",
                    args.base, args.head
                ),
            )
            .with_context(|| format!("writing {}", md_path.display()))?;
        }
    }
    Ok(())
}

fn write_unavailable_pr(
    json_path: &Path,
    md_path: &Path,
    reason: &str,
    args: &RiprArgs,
) -> Result<()> {
    let value = json!({
        "schemaVersion": 1,
        "tool": "ripr",
        "kind": "repo-exposure",
        "base": args.base,
        "head": args.head,
        "status": "unavailable",
        "findings": [],
        "warnings": [reason],
    });
    write_json(json_path, &value)?;
    std::fs::write(
        md_path,
        format!(
            "# RIPR PR Evidence\n\nRIPR evidence is unavailable for `{}`..`{}`.\n\nReason: {}\n",
            args.base, args.head, reason
        ),
    )
    .with_context(|| format!("writing {}", md_path.display()))?;
    Ok(())
}

fn write_unavailable_review(
    json_path: &Path,
    md_path: &Path,
    reason: &str,
    args: &RiprArgs,
) -> Result<()> {
    let value = json!({
        "schemaVersion": 1,
        "tool": "ripr",
        "kind": "review-comments",
        "base": args.base,
        "head": args.head,
        "comments": [],
        "summary_only": [],
        "suppressed": [],
        "warnings": [reason],
    });
    write_json(json_path, &value)?;
    std::fs::write(
        md_path,
        format!(
            "# RIPR Review Guidance\n\nNo RIPR review guidance was produced for `{}`..`{}`.\n\nReason: {}\n",
            args.base, args.head, reason
        ),
    )
    .with_context(|| format!("writing {}", md_path.display()))?;
    Ok(())
}

fn check_pr_contract(dir: &Path) -> Result<()> {
    let json_path = dir.join("repo-exposure.json");
    let md_path = dir.join("repo-exposure.md");
    validate_json_file(&json_path)?;
    validate_nonempty_file(&md_path)?;
    Ok(())
}

fn check_review_contract(dir: &Path) -> Result<()> {
    let json_path = dir.join("comments.json");
    let md_path = dir.join("comments.md");
    validate_json_file(&json_path)?;
    validate_nonempty_file(&md_path)?;
    Ok(())
}

fn validate_json_file(path: &Path) -> Result<()> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing JSON {}", path.display()))?;
    if !value.is_object() {
        anyhow::bail!("{} must contain a JSON object", path.display());
    }
    Ok(())
}

fn validate_nonempty_file(path: &Path) -> Result<()> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if text.trim().is_empty() {
        anyhow::bail!("{} must not be empty", path.display());
    }
    Ok(())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(path, [bytes, b"\n".to_vec()].concat())
        .with_context(|| format!("writing {}", path.display()))
}

fn ripr_bin() -> String {
    std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string())
}
