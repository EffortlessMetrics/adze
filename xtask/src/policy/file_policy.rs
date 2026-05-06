//! Non-Rust file allowlist checker.
//!
//! See `docs/policy/FILE_POLICY.md` for the policy.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::{CheckOutcome, ensure_reports_dir, workspace_root};

pub const SCHEMA_VERSION: &str = "1.0";

const IMPLICIT_EXTS: &[&str] = &[
    "rs",
    "toml",
    "lock",
    "md",
    "txt",
    "snap",
    "proptest-regressions",
    "sha256",
    "sha",
];

const IMPLICIT_NAMES: &[&str] = &[
    ".gitignore",
    ".gitmodules",
    ".gitattributes",
    ".editorconfig",
    "LICENSE",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "LICENSE-APACHE-2",
    "CODEOWNERS",
];

#[derive(Debug, Deserialize)]
struct AllowlistFile {
    #[serde(default)]
    schema_version: String,
    #[serde(default, rename = "allow")]
    allows: Vec<AllowEntry>,
}

#[expect(
    dead_code,
    reason = "kind/classification/reason/covered_by/generated_by are required policy metadata kept for editorial review and report rendering; they are not part of the matching predicate."
)]
#[derive(Debug, Clone, Deserialize)]
struct AllowEntry {
    #[serde(default)]
    glob: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    surface: Option<String>,
    #[serde(default)]
    classification: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    covered_by: Option<Vec<String>>,
    #[serde(default)]
    expires: Option<String>,
    #[serde(default)]
    retired: Option<bool>,
    #[serde(default)]
    generated_by: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RunOpts {
    pub strict: bool,
}

pub fn run(opts: RunOpts) -> Result<CheckOutcome> {
    let root = workspace_root()?;
    let reports_dir = ensure_reports_dir(&root)?;
    let allowlist_path = root.join("policy/non-rust-allowlist.toml");
    let allowlist = read_allowlist(&allowlist_path)?;

    let tracked = list_tracked_files(&root)?;

    let matchers: Vec<(usize, glob::Pattern)> = allowlist
        .allows
        .iter()
        .enumerate()
        .filter_map(|(idx, e)| {
            let pat = e.glob.clone().or_else(|| e.path.clone())?;
            glob::Pattern::new(&pat).ok().map(|p| (idx, p))
        })
        .collect();

    let mut matched_indices: BTreeSet<usize> = BTreeSet::new();
    let mut unallowlisted: Vec<String> = Vec::new();

    for rel in &tracked {
        if implicit_allowed(rel) {
            continue;
        }
        let mut matched = false;
        for (idx, pat) in &matchers {
            if pat.matches(rel) {
                matched_indices.insert(*idx);
                matched = true;
                break;
            }
        }
        if !matched {
            unallowlisted.push(rel.clone());
        }
    }

    let stale: Vec<&AllowEntry> = allowlist
        .allows
        .iter()
        .enumerate()
        .filter(|(idx, e)| !matched_indices.contains(idx) && !e.retired.unwrap_or(false))
        .map(|(_, e)| e)
        .collect();

    let expired: Vec<&AllowEntry> = allowlist
        .allows
        .iter()
        .filter(|e| {
            e.expires
                .as_deref()
                .map(super::no_panic_is_expired)
                .unwrap_or(false)
        })
        .collect();

    let report_md = reports_dir.join("file-policy.md");
    let report_json = reports_dir.join("file-policy.json");
    write_md_report(&report_md, &tracked, &unallowlisted, &stale, &expired)?;
    write_json_report(&report_json, &tracked, &unallowlisted, &stale, &expired)?;

    let total = unallowlisted.len() + stale.len() + expired.len();
    let outcome = CheckOutcome {
        label: "file-policy",
        findings: total,
        report_md,
    };
    outcome.print_summary();

    if opts.strict && total > 0 {
        anyhow::bail!(
            "file policy: {} unallowlisted, {} stale, {} expired",
            unallowlisted.len(),
            stale.len(),
            expired.len()
        );
    }
    Ok(outcome)
}

fn implicit_allowed(rel: &str) -> bool {
    let last_slash = rel.rfind('/').map(|i| i + 1).unwrap_or(0);
    let base = &rel[last_slash..];
    if IMPLICIT_NAMES.iter().any(|n| n == &base) {
        return true;
    }
    if let Some(ext) = base.rsplit_once('.').map(|(_, e)| e)
        && IMPLICIT_EXTS.iter().any(|x| x == &ext)
    {
        return true;
    }
    // Files without an extension are exempt only when explicitly named above.
    false
}

fn read_allowlist(path: &Path) -> Result<AllowlistFile> {
    if !path.exists() {
        return Ok(AllowlistFile {
            schema_version: SCHEMA_VERSION.to_string(),
            allows: vec![],
        });
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let file: AllowlistFile =
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

fn list_tracked_files(root: &Path) -> Result<Vec<String>> {
    let out = Command::new("git")
        .arg("ls-files")
        .current_dir(root)
        .output()
        .context("run `git ls-files`")?;
    if !out.status.success() {
        anyhow::bail!(
            "`git ls-files` failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let s = String::from_utf8(out.stdout).context("decode git ls-files output")?;
    let mut v: Vec<String> = s
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    v.sort();
    Ok(v)
}

fn write_md_report(
    path: &Path,
    tracked: &[String],
    unallowlisted: &[String],
    stale: &[&AllowEntry],
    expired: &[&AllowEntry],
) -> Result<()> {
    let mut s = String::new();
    s.push_str("# File policy report\n\n");
    s.push_str(&format!(
        "- Tracked files: {}\n- Unallowlisted: {}\n- Stale entries: {}\n- Expired entries: {}\n\n",
        tracked.len(),
        unallowlisted.len(),
        stale.len(),
        expired.len(),
    ));

    if !unallowlisted.is_empty() {
        s.push_str("## Unallowlisted files (top 100)\n\n");
        for f in unallowlisted.iter().take(100) {
            s.push_str(&format!("- `{}`\n", f));
        }
        if unallowlisted.len() > 100 {
            s.push_str(&format!("\n…and {} more.\n", unallowlisted.len() - 100));
        }
    }

    if !stale.is_empty() {
        s.push_str("\n## Stale allowlist entries\n\n");
        for e in stale {
            s.push_str(&format!(
                "- `{}` ({}) — matched no tracked files.\n",
                e.glob
                    .clone()
                    .or_else(|| e.path.clone())
                    .unwrap_or_default(),
                e.surface.clone().unwrap_or_default()
            ));
        }
    }

    if !expired.is_empty() {
        s.push_str("\n## Expired allowlist entries\n\n");
        for e in expired {
            s.push_str(&format!(
                "- `{}` expired {}\n",
                e.glob
                    .clone()
                    .or_else(|| e.path.clone())
                    .unwrap_or_default(),
                e.expires.as_deref().unwrap_or("?")
            ));
        }
    }

    fs::write(path, s).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn write_json_report(
    path: &Path,
    tracked: &[String],
    unallowlisted: &[String],
    stale: &[&AllowEntry],
    expired: &[&AllowEntry],
) -> Result<()> {
    let payload = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "totals": {
            "tracked": tracked.len(),
            "unallowlisted": unallowlisted.len(),
            "stale": stale.len(),
            "expired": expired.len(),
        },
        "unallowlisted": unallowlisted,
        "stale": stale.iter().map(|e| serde_json::json!({
            "glob": e.glob, "path": e.path, "owner": e.owner,
        })).collect::<Vec<_>>(),
        "expired": expired.iter().map(|e| serde_json::json!({
            "glob": e.glob, "path": e.path, "expires": e.expires,
        })).collect::<Vec<_>>(),
    });
    fs::write(path, serde_json::to_string_pretty(&payload)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
