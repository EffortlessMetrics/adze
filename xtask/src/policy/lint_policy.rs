//! Clippy / rustc lint baseline checker.
//!
//! See `docs/policy/CLIPPY_POLICY.md` for the policy.
//!
//! This checker validates that:
//!
//! 1. The MSRV declared in `policy/clippy-lints.toml` matches the
//!    workspace `rust-version`.
//! 2. Every `[[active]]` lint is reflected in the workspace `[lints]`
//!    block.
//! 3. No `[[planned]]` lint is active before its `activate_when_msrv`.
//! 4. No bare `#[allow(...)]` is committed in the tree.
//! 5. Every `clippy-debt.toml` entry has an unexpired `expires`.

use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::Path;

use super::{CheckOutcome, ensure_reports_dir, no_panic_is_expired, workspace_root};

pub const SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Deserialize)]
struct PolicyFile {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    msrv: String,
    #[serde(default)]
    active: Vec<LintEntry>,
    #[serde(default)]
    planned: Vec<LintEntry>,
}

#[derive(Debug, Deserialize, Clone)]
struct LintEntry {
    name: String,
    #[serde(default)]
    level: Option<String>,
    #[serde(default)]
    activate_when_msrv: Option<String>,
    #[serde(default)]
    reason: Option<String>,
}

#[expect(
    dead_code,
    reason = "schema_version is reserved for a future per-file schema check."
)]
#[derive(Debug, Deserialize)]
struct DebtFile {
    #[serde(default)]
    schema_version: String,
    #[serde(default, rename = "debt")]
    debts: Vec<DebtEntry>,
}

#[derive(Debug, Deserialize)]
struct DebtEntry {
    #[serde(default)]
    name: String,
    #[serde(rename = "crate", default)]
    crate_name: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    expires: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RunOpts {
    pub strict: bool,
}

pub fn run(opts: RunOpts) -> Result<CheckOutcome> {
    let root = workspace_root()?;
    let reports_dir = ensure_reports_dir(&root)?;

    let lints_path = root.join("policy/clippy-lints.toml");
    let debt_path = root.join("policy/clippy-debt.toml");

    let policy = read_policy(&lints_path)?;
    let debt = read_debt(&debt_path)?;

    let mut issues: Vec<String> = Vec::new();

    // 1) MSRV consistency. Normalize to the leading two version components
    // so "1.92" and "1.92.0" are treated as equal — the policy file lists
    // the minor while `rust-version` typically lists the patch.
    let cargo_msrv = read_workspace_msrv(&root)?;
    if !policy.msrv.is_empty() && !msrv_eq(&policy.msrv, &cargo_msrv) {
        issues.push(format!(
            "MSRV drift: policy/clippy-lints.toml = {}, Cargo.toml rust-version = {}",
            policy.msrv, cargo_msrv
        ));
    }

    // 2) Active lints reflected in workspace [lints].
    let workspace_toml = read_workspace_toml(&root)?;
    let active_in_workspace = collect_active_lints(&workspace_toml);
    for entry in &policy.active {
        if !active_in_workspace.contains(&entry.name) {
            issues.push(format!(
                "active policy lint `{}` not present in workspace [lints]",
                entry.name
            ));
        }
    }

    // 3) Planned lints must not be active yet.
    for entry in &policy.planned {
        if active_in_workspace.contains(&entry.name) {
            issues.push(format!(
                "planned policy lint `{}` is already active in workspace [lints]; promote it to [[active]]",
                entry.name
            ));
        }
    }

    // 4) Bare #[allow(...)] occurrences. We allow `#[expect(...)]`.
    let bare_allows = scan_bare_allows(&root)?;
    for hit in &bare_allows {
        issues.push(format!("bare #[allow(...)] at {}", hit));
    }

    // 5) Expired debt.
    for d in &debt.debts {
        if let Some(date) = &d.expires
            && no_panic_is_expired(date)
        {
            issues.push(format!(
                "expired clippy debt: `{}` in `{}` (owner: `{}`) expired {}",
                d.name, d.crate_name, d.owner, date
            ));
        }
    }

    let report_md = reports_dir.join("lint-policy.md");
    let report_json = reports_dir.join("lint-policy.json");
    write_md_report(
        &report_md,
        &policy,
        &debt,
        &active_in_workspace,
        &bare_allows,
        &issues,
    )?;
    write_json_report(&report_json, &policy, &issues)?;

    let outcome = CheckOutcome {
        label: "lint-policy",
        findings: issues.len(),
        report_md,
    };
    outcome.print_summary();

    if opts.strict && !issues.is_empty() {
        anyhow::bail!("lint policy: {} issue(s)", issues.len());
    }
    Ok(outcome)
}

fn read_policy(path: &Path) -> Result<PolicyFile> {
    if !path.exists() {
        return Ok(PolicyFile {
            schema_version: SCHEMA_VERSION.to_string(),
            msrv: String::new(),
            active: vec![],
            planned: vec![],
        });
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let file: PolicyFile =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if !file.schema_version.is_empty() && file.schema_version != SCHEMA_VERSION {
        anyhow::bail!(
            "{} has schema_version {} but checker expects {}",
            path.display(),
            file.schema_version,
            SCHEMA_VERSION
        );
    }
    Ok(file)
}

fn read_debt(path: &Path) -> Result<DebtFile> {
    if !path.exists() {
        return Ok(DebtFile {
            schema_version: SCHEMA_VERSION.to_string(),
            debts: vec![],
        });
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let file: DebtFile =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    Ok(file)
}

fn msrv_eq(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        let parts: Vec<&str> = s.split('.').collect();
        match parts.as_slice() {
            [maj, min, ..] => format!("{}.{}", maj, min),
            _ => s.to_string(),
        }
    };
    norm(a) == norm(b)
}

fn read_workspace_msrv(root: &Path) -> Result<String> {
    let toml_path = root.join("Cargo.toml");
    let text =
        fs::read_to_string(&toml_path).with_context(|| format!("read {}", toml_path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("parse {}", toml_path.display()))?;
    let v = value
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("rust-version"))
        .and_then(|r| r.as_str())
        .unwrap_or_default();
    Ok(v.to_string())
}

fn read_workspace_toml(root: &Path) -> Result<toml::Value> {
    let toml_path = root.join("Cargo.toml");
    let text =
        fs::read_to_string(&toml_path).with_context(|| format!("read {}", toml_path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("parse {}", toml_path.display()))?;
    Ok(value)
}

fn collect_active_lints(workspace: &toml::Value) -> Vec<String> {
    let mut out = Vec::new();
    let lints = match workspace
        .get("workspace")
        .and_then(|w| w.get("lints"))
        .and_then(|l| l.as_table())
    {
        Some(t) => t,
        None => return out,
    };
    for (group, entries) in lints {
        let table = match entries.as_table() {
            Some(t) => t,
            None => continue,
        };
        for (lint, _value) in table {
            out.push(format!("{}::{}", group, lint));
        }
    }
    out.sort();
    out
}

fn scan_bare_allows(root: &Path) -> Result<Vec<String>> {
    use walkdir::WalkDir;
    let re = Regex::new(r"#\[allow\(").expect("compiles");
    let mut hits = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_skipped(e.path()))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        for (lineno, line) in text.lines().enumerate() {
            if re.is_match(line) {
                // Allow `#[allow(...)]` if a `reason = "..."` is on the same line.
                if line.contains("reason =") {
                    continue;
                }
                hits.push(format!("{}:{}: `{}`", rel, lineno + 1, line.trim()));
            }
        }
    }
    Ok(hits)
}

fn is_skipped(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/target/") || s.contains("/.git/") || s.contains("/book/book/")
}

fn write_md_report(
    path: &Path,
    policy: &PolicyFile,
    debt: &DebtFile,
    active_in_workspace: &[String],
    bare_allows: &[String],
    issues: &[String],
) -> Result<()> {
    let mut s = String::new();
    s.push_str("# Lint policy report\n\n");
    s.push_str(&format!(
        "- Active lints in policy: {}\n- Planned lints: {}\n- Active lints in Cargo.toml: {}\n- Debt entries: {}\n- Bare #[allow] occurrences: {}\n- Issues: {}\n\n",
        policy.active.len(),
        policy.planned.len(),
        active_in_workspace.len(),
        debt.debts.len(),
        bare_allows.len(),
        issues.len(),
    ));

    if !issues.is_empty() {
        s.push_str("## Issues\n\n");
        for i in issues {
            s.push_str(&format!("- {}\n", i));
        }
    }

    if !bare_allows.is_empty() {
        s.push_str("\n## Bare #[allow(...)] occurrences (top 50)\n\n");
        for h in bare_allows.iter().take(50) {
            s.push_str(&format!("- {}\n", h));
        }
        if bare_allows.len() > 50 {
            s.push_str(&format!("\n…and {} more.\n", bare_allows.len() - 50));
        }
    }

    s.push_str("\n## Planned lint flips\n\n");
    let mut by_msrv: std::collections::BTreeMap<String, Vec<&LintEntry>> =
        std::collections::BTreeMap::new();
    for p in &policy.planned {
        by_msrv
            .entry(p.activate_when_msrv.clone().unwrap_or_default())
            .or_default()
            .push(p);
    }
    for (msrv, lints) in &by_msrv {
        s.push_str(&format!("\n### Activate when MSRV ≥ {}\n\n", msrv));
        for l in lints {
            s.push_str(&format!(
                "- `{}` → `{}` — {}\n",
                l.name,
                l.level.as_deref().unwrap_or("?"),
                l.reason.as_deref().unwrap_or("")
            ));
        }
    }

    fs::write(path, s).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_json_report(path: &Path, policy: &PolicyFile, issues: &[String]) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "msrv": policy.msrv,
        "totals": {
            "active": policy.active.len(),
            "planned": policy.planned.len(),
            "issues": issues.len(),
        },
        "issues": issues,
    });
    fs::write(path, serde_json::to_string_pretty(&payload)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
