//! Lint-policy checker.
//!
//! Verifies that:
//! 1. The workspace MSRV in `Cargo.toml` matches `policy/clippy-lints.toml`.
//! 2. No `clippy.toml` introduces panic-family test carveouts.
//! 3. No `[[planned]]` lint is activated before its `activate_when_msrv`.
//!
//! Runs in advisory mode by default — failures are reported but do not stop
//! CI. Once we are confident the manifest matches reality everywhere, this
//! will graduate to blocking.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

use super::{Mode, ensure_report_dir, workspace_root};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PolicyFile {
    msrv: String,
    #[serde(default)]
    active: Active,
    #[serde(default, rename = "planned")]
    planned: Vec<Planned>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct Active {
    #[serde(default)]
    rust: BTreeMap<String, String>,
    #[serde(default)]
    clippy: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Planned {
    name: String,
    level: String,
    activate_when_msrv: String,
    reason: String,
}

#[derive(Debug, Default)]
struct Findings {
    issues: Vec<String>,
}

pub fn run_check(mode: Mode) -> Result<()> {
    let root = workspace_root()?;
    let report_dir = ensure_report_dir(&root)?;

    let policy_path = root.join("policy").join("clippy-lints.toml");
    if !policy_path.exists() {
        anyhow::bail!("policy/clippy-lints.toml is missing; cannot run lint-policy check");
    }
    let policy: PolicyFile = toml::from_str(&std::fs::read_to_string(&policy_path)?)
        .with_context(|| format!("parsing {}", policy_path.display()))?;

    let mut findings = Findings::default();

    check_msrv_consistency(&root, &policy, &mut findings)?;
    check_no_test_carveouts(&root, &mut findings)?;
    check_planned_not_active_early(&root, &policy, &mut findings)?;

    let summary = format!(
        "lint-policy check ({mode:?}): {} issue(s)",
        findings.issues.len()
    );
    println!("{summary}");
    for issue in &findings.issues {
        println!("  - {issue}");
    }

    let mut md = String::from("# Lint policy report\n\n");
    md.push_str(&format!("- mode: `{mode:?}`\n"));
    md.push_str(&format!("- issues: {}\n\n", findings.issues.len()));
    if findings.issues.is_empty() {
        md.push_str("No issues found.\n");
    } else {
        md.push_str("## Issues\n\n");
        for issue in &findings.issues {
            md.push_str(&format!("- {issue}\n"));
        }
    }
    std::fs::write(report_dir.join("lint-policy.md"), md)?;

    match mode {
        Mode::Advisory => Ok(()),
        Mode::BlockingAllowlist | Mode::BlockingStrict => {
            if !findings.issues.is_empty() {
                anyhow::bail!(
                    "lint-policy check failed with {} issue(s)",
                    findings.issues.len()
                );
            }
            Ok(())
        }
    }
}

fn check_msrv_consistency(root: &Path, policy: &PolicyFile, findings: &mut Findings) -> Result<()> {
    let cargo = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let toolchain_path = root.join("rust-toolchain.toml");
    let toolchain = if toolchain_path.exists() {
        std::fs::read_to_string(&toolchain_path)?
    } else {
        String::new()
    };

    let cargo_msrv = extract_kv(&cargo, "rust-version");
    let toolchain_msrv = extract_kv(&toolchain, "channel");

    if let Some(cm) = cargo_msrv.as_deref() {
        if !cm.starts_with(&policy.msrv) {
            findings.issues.push(format!(
                "Cargo.toml rust-version `{cm}` does not match policy MSRV `{}`",
                policy.msrv
            ));
        }
    } else {
        findings
            .issues
            .push("Cargo.toml is missing workspace.package.rust-version".into());
    }

    if let Some(tm) = toolchain_msrv.as_deref()
        && !tm.starts_with(&policy.msrv)
    {
        findings.issues.push(format!(
            "rust-toolchain.toml channel `{tm}` does not match policy MSRV `{}`",
            policy.msrv
        ));
    }

    Ok(())
}

fn extract_kv(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(key) {
            let value = rest
                .trim_start()
                .strip_prefix('=')?
                .trim()
                .trim_matches('"')
                .to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn check_no_test_carveouts(root: &Path, findings: &mut Findings) -> Result<()> {
    let banned: &[&str] = &[
        "allow-unwrap-in-tests",
        "allow-expect-in-tests",
        "allow-panic-in-tests",
        "allow-indexing-slicing-in-tests",
        "allow-dbg-in-tests",
    ];
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "clippy.toml" {
            continue;
        }
        let rel = entry.path().strip_prefix(root).unwrap_or(entry.path());
        let text = match std::fs::read_to_string(entry.path()) {
            Ok(t) => t,
            Err(_) => continue,
        };
        for bad in banned {
            if text.contains(bad) {
                findings.issues.push(format!(
                    "{}: forbidden test carveout `{}` (no-panic policy bans this)",
                    rel.display(),
                    bad
                ));
            }
        }
    }
    Ok(())
}

fn check_planned_not_active_early(
    root: &Path,
    policy: &PolicyFile,
    findings: &mut Findings,
) -> Result<()> {
    let cargo = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let active_msrv = extract_kv(&cargo, "rust-version").unwrap_or_else(|| policy.msrv.clone());
    for planned in &policy.planned {
        let target = &planned.activate_when_msrv;
        if version_geq(&active_msrv, target) {
            continue;
        }
        if cargo.contains(&planned.name) {
            findings.issues.push(format!(
                "planned lint `{}` referenced in Cargo.toml before MSRV {target}",
                planned.name
            ));
        }
    }
    Ok(())
}

fn version_geq(a: &str, b: &str) -> bool {
    parse_version(a)
        .and_then(|av| parse_version(b).map(|bv| av >= bv))
        .unwrap_or(false)
}

fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let mut parts = s.split('.').filter_map(|p| p.parse::<u32>().ok());
    let major = parts.next()?;
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compare_works() {
        assert!(version_geq("1.96.0", "1.95"));
        assert!(version_geq("1.95.0", "1.95"));
        assert!(!version_geq("1.94.0", "1.95"));
    }

    #[test]
    fn extract_kv_handles_spaced_assignment() {
        let text = "rust-version = \"1.95.0\"\nother = 1\n";
        assert_eq!(extract_kv(text, "rust-version").as_deref(), Some("1.95.0"));
    }

    #[test]
    fn extract_kv_handles_tight_assignment() {
        let text = "channel=\"1.95.0\"\n";
        assert_eq!(extract_kv(text, "channel").as_deref(), Some("1.95.0"));
    }
}
