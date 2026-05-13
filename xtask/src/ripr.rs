use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

const PR_DIR: &str = "target/ripr/pr";
const REVIEW_DIR: &str = "target/ripr/review";

pub(crate) fn ripr_pr(check: bool, base: Option<String>, head: String) -> Result<()> {
    let root = workspace_root_path()?;
    let out_dir = root.join(PR_DIR);

    if check {
        check_pr_contract(&out_dir)?;
        println!("ripr-pr: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    let ripr_bin = std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string());

    let mut command = Command::new(&ripr_bin);
    command
        .arg("check")
        .arg("--root")
        .arg(&root)
        .arg("--format")
        .arg("repo-exposure-json")
        .current_dir(&root);
    let diff_path;
    if let Some(base) = base.as_deref() {
        if head == "HEAD" {
            command.arg("--base").arg(base);
        } else {
            diff_path = out_dir.join("input.diff");
            write_git_diff(&root, base, &head, &diff_path)?;
            command.arg("--diff").arg(&diff_path);
        }
    }

    match run_with_timeout(&mut command, ripr_timeout()) {
        Ok(Some(output)) if output.status.success() => {
            let value: Value = serde_json::from_slice(&output.stdout)
                .with_context(|| format!("{ripr_bin} emitted invalid PR evidence JSON"))?;
            write_json(&json_path, &value)?;
            write_markdown(&md_path, &repo_exposure_markdown(&value))?;
        }
        Ok(Some(output)) => {
            write_advisory_stub(
                &json_path,
                &md_path,
                "ripr-pr",
                &format!(
                    "{ripr_bin} exited non-zero: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            )?;
        }
        Ok(None) => {
            write_advisory_stub(&json_path, &md_path, "ripr-pr", "ripr-pr timed out")?;
        }
        Err(err) => {
            write_advisory_stub(
                &json_path,
                &md_path,
                "ripr-pr",
                &format!("{ripr_bin} is unavailable: {err}"),
            )?;
        }
    }

    check_pr_contract(&out_dir)
}

pub(crate) fn review_comments(check: bool, base: String, head: String) -> Result<()> {
    let root = workspace_root_path()?;
    let out_dir = root.join(REVIEW_DIR);

    if check {
        check_review_contract(&out_dir)?;
        println!("ripr-review-comments: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    let json_path = out_dir.join("comments.json");
    let md_path = out_dir.join("comments.md");
    let ripr_bin = std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string());

    let mut command = Command::new(&ripr_bin);
    command
        .arg("review-comments")
        .arg("--root")
        .arg(&root)
        .arg("--base")
        .arg(&base)
        .arg("--head")
        .arg(&head)
        .arg("--out")
        .arg(&json_path)
        .current_dir(&root);

    match run_with_timeout(&mut command, ripr_timeout()) {
        Ok(Some(output)) if output.status.success() => {
            if !md_path.exists() {
                let value = read_json(&json_path)?;
                write_markdown(&md_path, &review_markdown(&value))?;
            }
        }
        Ok(Some(output)) => {
            write_advisory_stub(
                &json_path,
                &md_path,
                "ripr-review-comments",
                &format!(
                    "{ripr_bin} exited non-zero: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            )?;
        }
        Ok(None) => {
            write_advisory_stub(
                &json_path,
                &md_path,
                "ripr-review-comments",
                "ripr-review-comments timed out",
            )?;
        }
        Err(err) => {
            write_advisory_stub(
                &json_path,
                &md_path,
                "ripr-review-comments",
                &format!("{ripr_bin} is unavailable: {err}"),
            )?;
        }
    }

    check_review_contract(&out_dir)
}

fn ripr_timeout() -> Duration {
    let seconds = std::env::var("ADZE_RIPR_PR_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10);
    Duration::from_secs(seconds)
}

fn run_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> Result<Option<std::process::Output>> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().with_context(|| "spawning ripr command")?;
    let start = Instant::now();

    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map(Some).map_err(Into::into);
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn write_git_diff(root: &Path, base: &str, head: &str, out: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--no-ext-diff")
        .arg(base)
        .arg(head)
        .current_dir(root)
        .output()
        .with_context(|| format!("running git diff {base} {head}"))?;
    if !output.status.success() {
        bail!(
            "git diff {base} {head} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::write(out, output.stdout).with_context(|| format!("writing {}", out.display()))
}

fn check_pr_contract(out_dir: &Path) -> Result<()> {
    let json_path = out_dir.join("repo-exposure.json");
    let md_path = out_dir.join("repo-exposure.md");
    let value = read_json(&json_path)?;
    require_schema_version(&value)?;
    require_nonempty_file(&md_path)?;
    Ok(())
}

fn check_review_contract(out_dir: &Path) -> Result<()> {
    let json_path = out_dir.join("comments.json");
    let md_path = out_dir.join("comments.md");
    let value = read_json(&json_path)?;
    require_schema_version(&value)?;
    require_array_field(&value, "comments")?;
    require_array_field(&value, "summary_only")?;
    require_nonempty_file(&md_path)?;
    Ok(())
}

fn read_json(path: &Path) -> Result<Value> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn require_schema_version(value: &Value) -> Result<()> {
    if value
        .get("schema_version")
        .and_then(Value::as_u64)
        .is_some()
    {
        return Ok(());
    }
    if value.get("schemaVersion").and_then(Value::as_u64).is_some() {
        return Ok(());
    }
    bail!("RIPR evidence JSON is missing schema_version/schemaVersion");
}

fn require_array_field(value: &Value, field: &str) -> Result<()> {
    if value.get(field).and_then(Value::as_array).is_none() {
        bail!("RIPR review JSON is missing `{field}` array");
    }
    Ok(())
}

fn require_nonempty_file(path: &Path) -> Result<()> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if text.trim().is_empty() {
        bail!("{} is empty", path.display());
    }
    Ok(())
}

fn write_advisory_stub(json_path: &Path, md_path: &Path, kind: &str, reason: &str) -> Result<()> {
    let value = if kind == "ripr-review-comments" {
        json!({
            "schema_version": 1,
            "status": "skipped",
            "reason": reason,
            "comments": [],
            "summary_only": [{"title": "RIPR unavailable", "body": reason}],
            "suppressed": [],
            "warnings": [reason]
        })
    } else {
        json!({
            "schema_version": 1,
            "status": "skipped",
            "reason": reason,
            "findings": []
        })
    };
    write_json(json_path, &value)?;
    let md = format!(
        "# RIPR advisory\n\nRIPR evidence was not produced by the binary.\n\nReason: {reason}\n"
    );
    write_markdown(md_path, &md)?;
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    std::fs::write(path, format!("{text}\n")).with_context(|| format!("writing {}", path.display()))
}

fn write_markdown(path: &Path, text: &str) -> Result<()> {
    std::fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}

fn repo_exposure_markdown(value: &Value) -> String {
    format!(
        "# RIPR PR Evidence\n\nPR-scoped repository exposure evidence was produced.\n\n```json\n{}\n```\n",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
    )
}

fn review_markdown(value: &Value) -> String {
    let comments = value
        .get("comments")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let summary_only = value
        .get("summary_only")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    format!(
        "# RIPR Review Guidance\n\n- Line-placeable comments: `{comments}`\n- Summary-only findings: `{summary_only}`\n"
    )
}

fn workspace_root_path() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("reading current directory")?;
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.exists() {
            let text = std::fs::read_to_string(&manifest)
                .with_context(|| format!("reading {}", manifest.display()))?;
            if text.contains("[workspace]") && dir.join("xtask").is_dir() {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            bail!("could not find workspace root from current directory");
        }
    }
}
