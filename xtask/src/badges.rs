use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const BADGE_ENDPOINT_DIR: &str = "badges";
const BADGE_ENDPOINT_TARGET_DIR: &str = "target/xtask/badges";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ShieldsEndpointBadge {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    label: String,
    message: String,
    color: String,
}

pub(crate) fn run(check: bool) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let target_dir = workspace_root.join(BADGE_ENDPOINT_TARGET_DIR);
    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("creating {}", target_dir.display()))?;

    let ripr_plus = ripr_plus_badge(&workspace_root)?;
    validate_shields_badge(&ripr_plus, Some("ripr+"))?;
    write_json_pretty(&target_dir.join("ripr-plus.json"), &ripr_plus)?;

    if check {
        let committed_dir = workspace_root.join(BADGE_ENDPOINT_DIR);
        compare_files(
            &committed_dir.join("ripr-plus.json"),
            &target_dir.join("ripr-plus.json"),
        )?;
        println!("badges: committed endpoints are current");
        return Ok(());
    }

    let committed_dir = workspace_root.join(BADGE_ENDPOINT_DIR);
    std::fs::create_dir_all(&committed_dir)
        .with_context(|| format!("creating {}", committed_dir.display()))?;
    std::fs::copy(
        target_dir.join("ripr-plus.json"),
        committed_dir.join("ripr-plus.json"),
    )
    .with_context(|| "copying generated ripr+ badge into badges/")?;

    println!("badges: refreshed public endpoint JSON under badges/");
    Ok(())
}

fn ripr_plus_badge(workspace_root: &Path) -> Result<ShieldsEndpointBadge> {
    let ripr_bin = std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string());

    ensure_test_efficiency_report(workspace_root)?;

    // Public README badge: repo-scoped, not PR/diff scoped.
    let mut command = Command::new(&ripr_bin);
    command
        .arg("check")
        .arg("--root")
        .arg(workspace_root)
        .arg("--format")
        .arg("repo-badge-plus-shields")
        .current_dir(workspace_root);

    let output = match run_with_timeout(&mut command, badge_timeout())? {
        Some(output) => output,
        None => return Ok(timeout_badge()),
    };

    if !output.status.success() {
        bail!(
            "{ripr_bin} repo-badge-plus-shields failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{ripr_bin} emitted invalid Shields endpoint JSON"))
}

fn badge_timeout() -> Duration {
    let seconds = std::env::var("ADZE_RIPR_BADGE_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10);
    Duration::from_secs(seconds)
}

fn run_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> Result<Option<std::process::Output>> {
    let mut child = command
        .spawn()
        .with_context(|| "spawning ripr badge command")?;
    let start = Instant::now();

    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map(Some).map_err(Into::into);
        }

        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            eprintln!(
                "badges: ripr repo-badge-plus-shields exceeded {}s; emitting timeout endpoint",
                timeout.as_secs()
            );
            return Ok(None);
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn timeout_badge() -> ShieldsEndpointBadge {
    ShieldsEndpointBadge {
        schema_version: 1,
        label: "ripr+".to_string(),
        message: "timeout".to_string(),
        color: "yellow".to_string(),
    }
}

fn ensure_test_efficiency_report(workspace_root: &Path) -> Result<()> {
    let report_path = workspace_root.join("target/ripr/reports/test-efficiency.json");
    if report_path.exists() {
        return Ok(());
    }

    if let Some(parent) = report_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let report = serde_json::json!({
        "schema_version": "0.1",
        "tests": [],
        "metrics": {
            "tests_scanned": 0,
            "reason_counts": {}
        }
    });
    let text = serde_json::to_string_pretty(&report)?;
    std::fs::write(&report_path, format!("{text}\n"))
        .with_context(|| format!("writing {}", report_path.display()))
}

pub(crate) fn validate_shields_badge(
    badge: &ShieldsEndpointBadge,
    expected_label: Option<&str>,
) -> Result<()> {
    if badge.schema_version != 1 {
        bail!("badge `{}` has unsupported schemaVersion", badge.label);
    }

    if let Some(expected_label) = expected_label
        && badge.label != expected_label
    {
        bail!(
            "badge label drifted: got `{}`, expected `{expected_label}`",
            badge.label
        );
    }

    if badge.message.trim().is_empty() {
        bail!("badge `{}` has empty message", badge.label);
    }

    if badge.color.trim().is_empty() {
        bail!("badge `{}` has empty color", badge.label);
    }

    Ok(())
}

fn write_json_pretty(path: &Path, badge: &ShieldsEndpointBadge) -> Result<()> {
    let json = serde_json::to_string_pretty(badge)?;
    std::fs::write(path, format!("{json}\n")).with_context(|| format!("writing {}", path.display()))
}

fn compare_files(committed: &Path, generated: &Path) -> Result<()> {
    let committed_text = std::fs::read_to_string(committed)
        .with_context(|| format!("reading committed badge {}", committed.display()))?;
    let generated_text = std::fs::read_to_string(generated)
        .with_context(|| format!("reading generated badge {}", generated.display()))?;

    if committed_text != generated_text {
        bail!(
            "badge endpoint drift: {} differs from generated {}; run `cargo xtask badges`",
            committed.display(),
            generated.display()
        );
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ripr_plus_badge_shape_is_stable() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr+".to_string(),
            message: "0".to_string(),
            color: "brightgreen".to_string(),
        };

        validate_shields_badge(&badge, Some("ripr+")).unwrap();
    }

    #[test]
    fn badge_shape_rejects_label_drift() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr".to_string(),
            message: "0".to_string(),
            color: "brightgreen".to_string(),
        };

        assert!(validate_shields_badge(&badge, Some("ripr+")).is_err());
    }
}
