use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use crate::policy;

const BADGE_ENDPOINT_DIR: &str = "badges";
const BADGE_ENDPOINT_TARGET_DIR: &str = "target/xtask/badges";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
struct ShieldsEndpointBadge {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    label: String,
    message: String,
    color: String,
}

pub fn run(check: bool) -> Result<()> {
    let workspace_root = policy::workspace_root()?;
    let target_dir = workspace_root.join(BADGE_ENDPOINT_TARGET_DIR);
    std::fs::create_dir_all(&target_dir)
        .with_context(|| format!("creating {}", target_dir.display()))?;

    let ripr_plus = ripr_plus_badge(&workspace_root)?;
    validate_shields_badge(&ripr_plus, Some("ripr+"))?;
    write_json_pretty(&target_dir.join("ripr-plus.json"), &ripr_plus)?;

    if check {
        let committed = workspace_root
            .join(BADGE_ENDPOINT_DIR)
            .join("ripr-plus.json");
        let generated = target_dir.join("ripr-plus.json");
        compare_files(&committed, &generated)?;
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
    .with_context(|| "copying generated ripr+ badge into badges/".to_string())?;

    println!("badges: refreshed public endpoint JSON under badges/");
    Ok(())
}

fn ripr_plus_badge(workspace_root: &Path) -> Result<ShieldsEndpointBadge> {
    let ripr_bin = std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string());
    let output = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(workspace_root)
        .arg("--format")
        .arg("repo-badge-plus-shields")
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("running {ripr_bin} for repo-scoped ripr+ badge"))?;

    if !output.status.success() {
        anyhow::bail!(
            "{ripr_bin} repo-badge-plus-shields failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let badge: ShieldsEndpointBadge = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{ripr_bin} emitted invalid Shields endpoint JSON"))?;
    Ok(badge)
}

fn validate_shields_badge(
    badge: &ShieldsEndpointBadge,
    expected_label: Option<&str>,
) -> Result<()> {
    if badge.schema_version != 1 {
        anyhow::bail!("badge `{}` has unsupported schemaVersion", badge.label);
    }
    if let Some(expected_label) = expected_label
        && badge.label != expected_label
    {
        anyhow::bail!(
            "badge label drifted: got `{}`, expected `{expected_label}`",
            badge.label
        );
    }
    if badge.message.trim().is_empty() {
        anyhow::bail!("badge `{}` has empty message", badge.label);
    }
    if badge.color.trim().is_empty() {
        anyhow::bail!("badge `{}` has empty color", badge.label);
    }
    Ok(())
}

fn write_json_pretty(path: &Path, badge: &ShieldsEndpointBadge) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(badge)?;
    std::fs::write(path, [bytes, b"\n".to_vec()].concat())
        .with_context(|| format!("writing {}", path.display()))
}

fn compare_files(committed: &Path, generated: &Path) -> Result<()> {
    let committed_bytes = std::fs::read(committed)
        .with_context(|| format!("reading committed badge {}", committed.display()))?;
    let generated_bytes = std::fs::read(generated)
        .with_context(|| format!("reading generated badge {}", generated.display()))?;
    if committed_bytes != generated_bytes {
        anyhow::bail!(
            "badge endpoint drifted: run `cargo xtask badges` and commit {}",
            committed.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
    fn rejects_empty_badge_message() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr+".to_string(),
            message: " ".to_string(),
            color: "brightgreen".to_string(),
        };

        assert!(validate_shields_badge(&badge, Some("ripr+")).is_err());
    }

    #[test]
    fn badge_paths_are_workspace_relative() {
        assert_eq!(PathBuf::from(BADGE_ENDPOINT_DIR), PathBuf::from("badges"));
        assert_eq!(
            PathBuf::from(BADGE_ENDPOINT_TARGET_DIR),
            PathBuf::from("target/xtask/badges")
        );
    }
}
