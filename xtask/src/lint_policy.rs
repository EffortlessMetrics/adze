use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use toml::Value;

const ROOT_MANIFEST: &str = "Cargo.toml";
const POLICY_LEDGER: &str = "policy/clippy-lints.toml";
const DEBT_LEDGER: &str = "policy/clippy-debt.toml";
const CLIPPY_CONFIG: &str = "clippy.toml";

const TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

#[derive(Debug, Deserialize)]
struct LintPolicyLedger {
    schema: u64,
    msrv: String,
    policy: PolicyFlags,
    #[serde(default)]
    lint: Vec<LintEntry>,
}

#[derive(Debug, Deserialize)]
struct PolicyFlags {
    panic_free_tests: bool,
    allow_test_carveouts: bool,
    suppression_style: String,
    blanket_categories: bool,
}

#[derive(Debug, Deserialize)]
struct LintEntry {
    name: String,
    level: String,
    status: String,
    #[serde(default)]
    activate_when_msrv: Option<String>,
    class: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct DebtLedger {
    schema: u64,
    #[serde(default)]
    debt: Vec<DebtEntry>,
}

#[derive(Debug, Deserialize)]
struct DebtEntry {
    lint: String,
    path: String,
    owner: String,
    reason: String,
    expires: String,
}

pub fn check_lint_policy() -> Result<()> {
    let root = repo_root()?;
    let manifest = read_toml(root.join(ROOT_MANIFEST))?;
    let ledger: LintPolicyLedger = read_toml(root.join(POLICY_LEDGER))?;
    let debt: DebtLedger = read_toml(root.join(DEBT_LEDGER))?;

    check_ledger_shape(&ledger)?;
    check_msrv(&manifest, &ledger)?;
    check_root_lints(&manifest, &ledger)?;
    check_clippy_toml(&root)?;
    check_debt(&debt)?;

    println!(
        "✓ lint policy ok: {} active lint(s), {} planned lint(s), {} debt entry/entries",
        ledger
            .lint
            .iter()
            .filter(|lint| lint.status == "active")
            .count(),
        ledger
            .lint
            .iter()
            .filter(|lint| lint.status == "planned")
            .count(),
        debt.debt.len()
    );
    Ok(())
}

fn repo_root() -> Result<std::path::PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to run git rev-parse --show-toplevel")?;
    ensure!(
        output.status.success(),
        "git rev-parse --show-toplevel failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(std::path::PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

fn read_toml<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let path = path.as_ref();
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

fn check_ledger_shape(ledger: &LintPolicyLedger) -> Result<()> {
    ensure!(
        ledger.schema == 1,
        "policy/clippy-lints.toml schema must be 1"
    );
    ensure!(ledger.msrv == "1.93", "lint policy MSRV must be 1.93");
    ensure!(
        ledger.policy.panic_free_tests,
        "panic_free_tests must be true"
    );
    ensure!(
        !ledger.policy.allow_test_carveouts,
        "allow_test_carveouts must be false"
    );
    ensure!(
        ledger.policy.suppression_style == "expect-with-reason",
        "suppression_style must be expect-with-reason"
    );
    ensure!(
        !ledger.policy.blanket_categories,
        "blanket_categories must be false"
    );

    let mut seen = BTreeSet::new();
    for lint in &ledger.lint {
        ensure!(!lint.name.trim().is_empty(), "lint entry has empty name");
        ensure!(
            matches!(lint.level.as_str(), "forbid" | "deny" | "warn"),
            "{} has unsupported level {}",
            lint.name,
            lint.level
        );
        ensure!(
            matches!(lint.status.as_str(), "active" | "planned"),
            "{} has unsupported status {}",
            lint.name,
            lint.status
        );
        ensure!(
            !lint.class.trim().is_empty(),
            "{} is missing class",
            lint.name
        );
        ensure!(
            !lint.reason.trim().is_empty(),
            "{} is missing reason",
            lint.name
        );
        ensure!(
            seen.insert(&lint.name),
            "duplicate lint entry {}",
            lint.name
        );
        if lint.status == "planned" {
            ensure!(
                lint.activate_when_msrv.is_some(),
                "planned lint {} must have activate_when_msrv",
                lint.name
            );
        }
    }
    Ok(())
}

fn check_msrv(manifest: &Value, ledger: &LintPolicyLedger) -> Result<()> {
    let manifest_msrv = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("rust-version"))
        .and_then(Value::as_str)
        .context("Cargo.toml must define workspace.package.rust-version")?;
    ensure!(
        manifest_msrv == ledger.msrv,
        "workspace.package.rust-version ({manifest_msrv}) must match policy MSRV ({})",
        ledger.msrv
    );
    Ok(())
}

fn check_root_lints(manifest: &Value, ledger: &LintPolicyLedger) -> Result<()> {
    let root_lints = collect_root_lints(manifest)?;
    let mut missing = Vec::new();
    let mut mismatched = Vec::new();

    for lint in ledger.lint.iter().filter(|lint| lint.status == "active") {
        match root_lints.get(&lint.name) {
            Some(level) if level == &lint.level => {}
            Some(level) => mismatched.push(format!(
                "{}: Cargo.toml has {}, policy has {}",
                lint.name, level, lint.level
            )),
            None => missing.push(lint.name.clone()),
        }
    }

    if !missing.is_empty() || !mismatched.is_empty() {
        bail!(
            "active lint policy mismatch\nmissing: {:?}\nmismatched: {:?}",
            missing,
            mismatched
        );
    }

    for lint in ledger.lint.iter().filter(|lint| lint.status == "planned") {
        if root_lints.contains_key(&lint.name) {
            bail!(
                "planned lint {} is active before MSRV {}",
                lint.name,
                lint.activate_when_msrv.as_deref().unwrap_or("<unknown>")
            );
        }
    }

    Ok(())
}

fn collect_root_lints(manifest: &Value) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    let lints = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("lints"))
        .context("Cargo.toml must define workspace.lints")?;

    for namespace in ["rust", "clippy"] {
        let table = lints
            .get(namespace)
            .and_then(Value::as_table)
            .with_context(|| format!("Cargo.toml must define workspace.lints.{namespace}"))?;
        for (name, value) in table {
            let level = value.as_str().with_context(|| {
                format!("workspace lint {namespace}::{name} must be a string level")
            })?;
            let key = if namespace == "clippy" {
                format!("clippy::{name}")
            } else {
                name.to_string()
            };
            out.insert(key, level.to_string());
        }
    }
    Ok(out)
}

fn check_clippy_toml(root: &Path) -> Result<()> {
    let path = root.join(CLIPPY_CONFIG);
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let config: Value =
        toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;

    for carveout in TEST_CARVEOUTS {
        if config.get(*carveout).is_some() {
            bail!("clippy.toml must not set test carveout {carveout}");
        }
    }
    Ok(())
}

fn check_debt(debt: &DebtLedger) -> Result<()> {
    ensure!(debt.schema == 1, "policy/clippy-debt.toml schema must be 1");
    let today = Utc::now().date_naive();
    for entry in &debt.debt {
        ensure!(!entry.lint.trim().is_empty(), "debt entry missing lint");
        ensure!(!entry.path.trim().is_empty(), "debt entry missing path");
        ensure!(!entry.owner.trim().is_empty(), "debt entry missing owner");
        ensure!(!entry.reason.trim().is_empty(), "debt entry missing reason");
        let expires = NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d")
            .with_context(|| format!("debt entry {} has invalid expires", entry.lint))?;
        ensure!(
            expires >= today,
            "debt entry {} for {} expired on {}",
            entry.lint,
            entry.path,
            entry.expires
        );
    }
    Ok(())
}
