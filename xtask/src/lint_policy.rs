use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

const POLICY_PATH: &str = "policy/clippy-lints.toml";
const DEBT_PATH: &str = "policy/clippy-debt.toml";
const CLIPPY_CONFIG_PATH: &str = "clippy.toml";
const TOOLCHAIN_PATH: &str = "rust-toolchain.toml";
const CARGO_PATH: &str = "Cargo.toml";

const TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

const REQUIRED_PLANNED: &[(&str, &str, &str)] = &[
    ("clippy::same_length_and_capacity", "deny", "1.94"),
    ("clippy::manual_ilog2", "warn", "1.94"),
    ("clippy::decimal_bitwise_operands", "warn", "1.94"),
    ("clippy::needless_type_cast", "warn", "1.94"),
    ("clippy::disallowed_fields", "deny", "1.95"),
    ("clippy::manual_checked_ops", "warn", "1.95"),
    ("clippy::manual_take", "warn", "1.95"),
    ("clippy::manual_pop_if", "warn", "1.95"),
    ("clippy::duration_suboptimal_units", "warn", "1.95"),
    ("clippy::unnecessary_trailing_comma", "warn", "1.95"),
];

#[derive(Debug, Deserialize)]
struct LintPolicy {
    schema: u64,
    msrv: String,
    policy: PolicyConfig,
    #[serde(default)]
    lint: Vec<LintEntry>,
}

#[derive(Debug, Deserialize)]
struct PolicyConfig {
    panic_free_tests: bool,
    allow_test_carveouts: bool,
    suppression_style: String,
    blanket_categories: bool,
    #[serde(default)]
    lint_inheritance: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LintEntry {
    name: String,
    level: String,
    status: String,
    reason: String,
    #[serde(default)]
    activate_when_msrv: Option<String>,
    #[serde(default)]
    class: Option<String>,
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
    let root_cargo = fs::read_to_string(CARGO_PATH).context("reading root Cargo.toml")?;
    let root_value = parse_toml(CARGO_PATH, &root_cargo)?;
    let policy_text =
        fs::read_to_string(POLICY_PATH).context("reading policy/clippy-lints.toml")?;
    let policy: LintPolicy =
        toml::from_str(&policy_text).context("parsing policy/clippy-lints.toml")?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    check_policy_header(&policy, &mut errors);
    check_msrv(&root_value, &policy, &mut errors)?;
    check_toolchain(&policy, &mut errors)?;
    check_root_lints(&root_value, &policy, &mut errors);
    check_planned_lints(&policy, &mut errors);
    check_clippy_config(&policy, &mut errors)?;
    check_debt_ledger(&mut errors)?;
    check_allowlists_exist(&mut errors);
    check_lint_inheritance(&root_cargo, &policy, &mut errors, &mut warnings)?;

    for warning in &warnings {
        eprintln!("warning: {warning}");
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("error: {error}");
        }
        bail!("lint policy check failed with {} error(s)", errors.len());
    }

    println!(
        "lint policy OK: {} active lint(s), {} planned upgrade lint(s)",
        policy
            .lint
            .iter()
            .filter(|lint| lint.status == "active")
            .count(),
        policy
            .lint
            .iter()
            .filter(|lint| lint.status == "planned")
            .count()
    );
    Ok(())
}

fn parse_toml(path: &str, contents: &str) -> Result<toml::Value> {
    contents
        .parse::<toml::Value>()
        .with_context(|| format!("parsing {path}"))
}

fn check_policy_header(policy: &LintPolicy, errors: &mut Vec<String>) {
    if policy.schema != 1 {
        errors.push(format!("{POLICY_PATH} schema must be 1"));
    }
    if !policy.policy.panic_free_tests {
        errors.push("policy.panic_free_tests must be true".to_string());
    }
    if policy.policy.allow_test_carveouts {
        errors.push("policy.allow_test_carveouts must be false".to_string());
    }
    if policy.policy.suppression_style != "expect-with-reason" {
        errors.push("policy.suppression_style must be expect-with-reason".to_string());
    }
    if policy.policy.blanket_categories {
        errors.push("policy.blanket_categories must be false".to_string());
    }
}

fn check_msrv(
    root_value: &toml::Value,
    policy: &LintPolicy,
    errors: &mut Vec<String>,
) -> Result<()> {
    let workspace_msrv = root_value
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("rust-version"))
        .and_then(toml::Value::as_str)
        .context("workspace.package.rust-version is missing")?;
    if workspace_msrv != policy.msrv {
        errors.push(format!(
            "workspace.package.rust-version ({workspace_msrv}) must match {POLICY_PATH} msrv ({})",
            policy.msrv
        ));
    }
    Ok(())
}

fn check_toolchain(policy: &LintPolicy, errors: &mut Vec<String>) -> Result<()> {
    let toolchain_text =
        fs::read_to_string(TOOLCHAIN_PATH).context("reading rust-toolchain.toml")?;
    let toolchain = parse_toml(TOOLCHAIN_PATH, &toolchain_text)?;
    let channel = toolchain
        .get("toolchain")
        .and_then(|toolchain| toolchain.get("channel"))
        .and_then(toml::Value::as_str)
        .context("toolchain.channel is missing")?;
    if channel != policy.msrv {
        errors.push(format!(
            "rust-toolchain.toml channel ({channel}) must match {POLICY_PATH} msrv ({})",
            policy.msrv
        ));
    }
    Ok(())
}

fn check_root_lints(root_value: &toml::Value, policy: &LintPolicy, errors: &mut Vec<String>) {
    let Some(workspace_lints) = root_value
        .get("workspace")
        .and_then(|workspace| workspace.get("lints"))
    else {
        errors.push("root Cargo.toml must define [workspace.lints]".to_string());
        return;
    };

    let mut active = BTreeMap::new();
    for entry in policy.lint.iter().filter(|entry| entry.status == "active") {
        active.insert(entry.name.as_str(), entry.level.as_str());
        if entry.reason.trim().is_empty() {
            errors.push(format!("active lint {} must include a reason", entry.name));
        }
        if entry.class.as_deref().unwrap_or_default().trim().is_empty() {
            errors.push(format!("active lint {} must include a class", entry.name));
        }
    }

    for (name, expected_level) in active {
        let Some((tool, lint_name)) = name.split_once("::") else {
            errors.push(format!("lint name {name} must include a tool namespace"));
            continue;
        };
        let Some(actual_level) = workspace_lints
            .get(tool)
            .and_then(|tool_lints| tool_lints.get(lint_name))
            .and_then(toml::Value::as_str)
        else {
            errors.push(format!("root Cargo.toml is missing active lint {name}"));
            continue;
        };
        if actual_level != expected_level {
            errors.push(format!(
                "root Cargo.toml lint {name} is {actual_level}, expected {expected_level}"
            ));
        }
    }
}

fn check_planned_lints(policy: &LintPolicy, errors: &mut Vec<String>) {
    let planned: BTreeMap<_, _> = policy
        .lint
        .iter()
        .filter(|entry| entry.status == "planned")
        .map(|entry| (entry.name.as_str(), entry))
        .collect();

    for (name, level, activate_when_msrv) in REQUIRED_PLANNED {
        let Some(entry) = planned.get(name) else {
            errors.push(format!("missing planned lint {name}"));
            continue;
        };
        if entry.level != *level {
            errors.push(format!(
                "planned lint {name} has level {}, expected {level}",
                entry.level
            ));
        }
        if entry.activate_when_msrv.as_deref() != Some(*activate_when_msrv) {
            errors.push(format!(
                "planned lint {name} must activate at MSRV {activate_when_msrv}"
            ));
        }
        if entry.reason.trim().is_empty() {
            errors.push(format!("planned lint {name} must include a reason"));
        }
    }

    for entry in policy.lint.iter().filter(|entry| entry.status == "planned") {
        let Some(activate) = entry.activate_when_msrv.as_deref() else {
            errors.push(format!(
                "planned lint {} must include activate_when_msrv",
                entry.name
            ));
            continue;
        };
        if compare_version(&policy.msrv, activate).is_ge() {
            errors.push(format!(
                "planned lint {} is still planned even though MSRV {} has reached {}",
                entry.name, policy.msrv, activate
            ));
        }
    }
}

fn check_clippy_config(policy: &LintPolicy, errors: &mut Vec<String>) -> Result<()> {
    let config = fs::read_to_string(CLIPPY_CONFIG_PATH).context("reading clippy.toml")?;
    for carveout in TEST_CARVEOUTS {
        if config.contains(carveout) && !policy.policy.allow_test_carveouts {
            errors.push(format!(
                "clippy.toml must not contain test carveout {carveout}"
            ));
        }
    }
    Ok(())
}

fn check_debt_ledger(errors: &mut Vec<String>) -> Result<()> {
    let debt_text = fs::read_to_string(DEBT_PATH).context("reading policy/clippy-debt.toml")?;
    let debt: DebtLedger = toml::from_str(&debt_text).context("parsing policy/clippy-debt.toml")?;
    if debt.schema != 1 {
        errors.push(format!("{DEBT_PATH} schema must be 1"));
    }
    let today = chrono::Utc::now().date_naive();
    for (idx, entry) in debt.debt.iter().enumerate() {
        let label = format!("{DEBT_PATH} debt entry #{idx}");
        if entry.lint.trim().is_empty() {
            errors.push(format!("{label} must include lint"));
        }
        if entry.path.trim().is_empty() {
            errors.push(format!("{label} must include path"));
        }
        if entry.owner.trim().is_empty() {
            errors.push(format!("{label} must include owner"));
        }
        if entry.reason.trim().is_empty() {
            errors.push(format!("{label} must include reason"));
        }
        match chrono::NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d") {
            Ok(expires) if expires < today => {
                errors.push(format!("{label} expired on {expires}"));
            }
            Ok(_) => {}
            Err(err) => errors.push(format!("{label} has invalid expires date: {err}")),
        }
    }
    Ok(())
}

fn check_allowlists_exist(errors: &mut Vec<String>) {
    for path in [
        "policy/no-panic-allowlist.toml",
        "policy/non-rust-allowlist.toml",
    ] {
        if !Path::new(path).exists() {
            errors.push(format!("missing required policy allowlist {path}"));
        }
    }
}

fn check_lint_inheritance(
    root_cargo: &str,
    policy: &LintPolicy,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let metadata = cargo_metadata()?;
    let root = std::env::current_dir().context("resolving current directory")?;
    let enforce = policy.policy.lint_inheritance.as_deref() == Some("required");
    let mut missing = Vec::new();
    let members = metadata
        .get("workspace_members")
        .and_then(serde_json::Value::as_array)
        .context("cargo metadata missing workspace_members")?;
    let packages = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .context("cargo metadata missing packages")?;

    for member in members {
        let Some(member_id) = member.as_str() else {
            continue;
        };
        let Some(package) = packages.iter().find(|package| {
            package.get("id").and_then(serde_json::Value::as_str) == Some(member_id)
        }) else {
            continue;
        };
        let Some(manifest_path) = package
            .get("manifest_path")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        let manifest = Path::new(manifest_path);
        if manifest == Path::new(CARGO_PATH) {
            continue;
        }
        let contents = fs::read_to_string(manifest)
            .with_context(|| format!("reading {}", manifest.display()))?;
        if !manifest_has_lint_inheritance(&contents) {
            let rel = manifest.strip_prefix(&root).unwrap_or(manifest);
            missing.push(rel.display().to_string());
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    let message = format!(
        "{} workspace member manifest(s) do not yet inherit [lints] workspace = true",
        missing.len()
    );
    if enforce {
        errors.push(format!("{message}: {}", missing.join(", ")));
    } else {
        warnings.push(format!(
            "{message}; policy is staged until existing lint debt is migrated"
        ));
    }

    if !root_cargo.contains("[workspace.lints.clippy]") {
        errors.push("root Cargo.toml must include [workspace.lints.clippy]".to_string());
    }
    Ok(())
}

fn cargo_metadata() -> Result<serde_json::Value> {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .context("running cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout).context("parsing cargo metadata JSON")
}

fn manifest_has_lint_inheritance(contents: &str) -> bool {
    let mut in_lints = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_lints = trimmed == "[lints]";
            continue;
        }
        if in_lints && trimmed == "workspace = true" {
            return true;
        }
    }
    false
}

#[derive(Debug, PartialEq, Eq)]
enum VersionOrdering {
    Lt,
    Eq,
    Gt,
}

impl VersionOrdering {
    fn is_ge(&self) -> bool {
        matches!(self, Self::Eq | Self::Gt)
    }
}

fn compare_version(lhs: &str, rhs: &str) -> VersionOrdering {
    let parse = |version: &str| -> Vec<u64> {
        version
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let lhs = parse(lhs);
    let rhs = parse(rhs);
    for idx in 0..lhs.len().max(rhs.len()) {
        let left = lhs.get(idx).copied().unwrap_or(0);
        let right = rhs.get(idx).copied().unwrap_or(0);
        if left < right {
            return VersionOrdering::Lt;
        }
        if left > right {
            return VersionOrdering::Gt;
        }
    }
    VersionOrdering::Eq
}
