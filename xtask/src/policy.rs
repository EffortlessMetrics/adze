use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde::Deserialize;
use std::{collections::BTreeMap, env, fs, path::Path, process::Command};

type LintMap = BTreeMap<String, String>;

const CLIPPY_TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

#[derive(Debug, Deserialize)]
struct ClippyPolicyLedger {
    schema: u64,
    msrv: String,
    policy: ClippyPolicy,
    #[serde(default)]
    lint: Vec<LintEntry>,
}

#[derive(Debug, Deserialize)]
struct ClippyPolicy {
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
struct ClippyDebtLedger {
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

#[derive(Debug, Deserialize)]
struct NoPanicAllowlist {
    schema_version: String,
    #[serde(default)]
    allow: Vec<NoPanicAllow>,
}

#[derive(Debug, Deserialize)]
struct NoPanicAllow {
    path: String,
    family: String,
    classification: String,
    owner: String,
    explanation: String,
    #[serde(default)]
    expires: Option<String>,
    selector: PanicSelector,
    #[serde(default)]
    last_seen: Option<LastSeen>,
}

#[derive(Debug, Deserialize)]
struct PanicSelector {
    kind: String,
    container: String,
    callee: String,
    #[serde(default)]
    receiver_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LastSeen {
    line: u64,
    column: u64,
}

#[derive(Debug, Deserialize)]
struct NonRustAllowlist {
    schema_version: String,
    #[serde(default)]
    allow: Vec<NonRustAllow>,
}

#[derive(Debug, Deserialize)]
struct NonRustAllow {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    glob: Option<String>,
    kind: String,
    owner: String,
    reason: String,
    surface: String,
    classification: String,
    #[serde(default)]
    covered_by: Vec<String>,
    #[serde(default)]
    expires: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoManifest {
    workspace: Workspace,
}

#[derive(Debug, Deserialize)]
struct Workspace {
    package: WorkspacePackage,
    #[serde(default)]
    lints: WorkspaceLints,
}

#[derive(Debug, Deserialize)]
struct WorkspacePackage {
    #[serde(rename = "rust-version")]
    rust_version: String,
}

#[derive(Debug, Default, Deserialize)]
struct WorkspaceLints {
    #[serde(default)]
    rust: LintMap,
    #[serde(default)]
    clippy: LintMap,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: Vec<String>,
    workspace_root: String,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    id: String,
    manifest_path: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct PackageManifest {
    #[serde(default)]
    lints: Option<PackageLints>,
}

#[derive(Debug, Deserialize)]
struct PackageLints {
    #[serde(default)]
    workspace: bool,
}

pub fn check_lint_policy() -> Result<()> {
    let root = repo_root()?;
    let cargo = read_toml::<CargoManifest>(&root.join("Cargo.toml"))?;
    let ledger = read_toml::<ClippyPolicyLedger>(&root.join("policy/clippy-lints.toml"))?;
    let debt = read_toml::<ClippyDebtLedger>(&root.join("policy/clippy-debt.toml"))?;

    ensure_ledger_header(&ledger)?;
    ensure_msrv_matches(&cargo, &ledger)?;
    ensure_lints_match_manifest(&cargo.workspace.lints, &ledger)?;
    ensure_planned_lints_not_active(&cargo.workspace.lints, &ledger)?;
    ensure_workspace_lint_inheritance(&root)?;
    ensure_no_test_carveouts(&root.join("clippy.toml"))?;
    ensure_debt_is_reviewable(&debt)?;

    println!(
        "✓ lint policy ok: {} active lints, {} planned lints, {} debt entries",
        active_lints(&ledger).count(),
        planned_lints(&ledger).count(),
        debt.debt.len()
    );
    Ok(())
}

pub fn check_no_panic_family() -> Result<()> {
    let root = repo_root()?;
    let allowlist = read_toml::<NoPanicAllowlist>(&root.join("policy/no-panic-allowlist.toml"))?;
    ensure_no_panic_allowlist(&allowlist)?;
    println!(
        "✓ no-panic allowlist ok: {} semantic exceptions",
        allowlist.allow.len()
    );
    Ok(())
}

pub fn check_file_policy() -> Result<()> {
    let root = repo_root()?;
    let allowlist = read_toml::<NonRustAllowlist>(&root.join("policy/non-rust-allowlist.toml"))?;
    ensure_non_rust_allowlist(&allowlist)?;
    println!(
        "✓ non-rust allowlist ok: {} policy exceptions",
        allowlist.allow.len()
    );
    Ok(())
}

pub fn policy_report() -> Result<()> {
    let root = repo_root()?;
    let ledger = read_toml::<ClippyPolicyLedger>(&root.join("policy/clippy-lints.toml"))?;
    let debt = read_toml::<ClippyDebtLedger>(&root.join("policy/clippy-debt.toml"))?;
    let no_panic = read_toml::<NoPanicAllowlist>(&root.join("policy/no-panic-allowlist.toml"))?;
    let non_rust = read_toml::<NonRustAllowlist>(&root.join("policy/non-rust-allowlist.toml"))?;

    println!("policy report");
    println!("  clippy schema: {}", ledger.schema);
    println!("  clippy msrv: {}", ledger.msrv);
    println!("  active lints: {}", active_lints(&ledger).count());
    println!("  planned lints: {}", planned_lints(&ledger).count());
    println!("  clippy debt entries: {}", debt.debt.len());
    println!("  no-panic exceptions: {}", no_panic.allow.len());
    println!("  non-rust exceptions: {}", non_rust.allow.len());
    Ok(())
}

fn ensure_no_panic_allowlist(allowlist: &NoPanicAllowlist) -> Result<()> {
    if allowlist.schema_version != "0.3" {
        bail!("policy/no-panic-allowlist.toml schema_version must be 0.3");
    }
    for entry in &allowlist.allow {
        require_non_empty(&entry.path, "allow.path")?;
        require_non_empty(&entry.family, "allow.family")?;
        require_non_empty(&entry.classification, "allow.classification")?;
        require_non_empty(&entry.owner, "allow.owner")?;
        require_non_empty(&entry.explanation, "allow.explanation")?;
        require_non_empty(&entry.selector.kind, "allow.selector.kind")?;
        require_non_empty(&entry.selector.container, "allow.selector.container")?;
        require_non_empty(&entry.selector.callee, "allow.selector.callee")?;
        if let Some(receiver) = &entry.selector.receiver_fingerprint {
            require_non_empty(receiver, "allow.selector.receiver_fingerprint")?;
        }
        if let Some(last_seen) = &entry.last_seen {
            if last_seen.line == 0 || last_seen.column == 0 {
                bail!("allow.last_seen line and column must be one-based");
            }
        }
        ensure_optional_expiry(entry.expires.as_deref(), &entry.path)?;
    }
    Ok(())
}

fn ensure_non_rust_allowlist(allowlist: &NonRustAllowlist) -> Result<()> {
    if allowlist.schema_version != "1.0" {
        bail!("policy/non-rust-allowlist.toml schema_version must be 1.0");
    }
    for entry in &allowlist.allow {
        if entry.path.is_none() == entry.glob.is_none() {
            bail!("non-rust allow entries must set exactly one of path or glob");
        }
        if let Some(path) = &entry.path {
            require_non_empty(path, "allow.path")?;
        }
        if let Some(glob) = &entry.glob {
            require_non_empty(glob, "allow.glob")?;
        }
        require_non_empty(&entry.kind, "allow.kind")?;
        require_non_empty(&entry.owner, "allow.owner")?;
        require_non_empty(&entry.reason, "allow.reason")?;
        require_non_empty(&entry.surface, "allow.surface")?;
        require_non_empty(&entry.classification, "allow.classification")?;
        if matches!(
            entry.classification.as_str(),
            "production" | "test" | "tooling"
        ) && entry.covered_by.is_empty()
        {
            bail!(
                "non-rust allow entry for {} must include covered_by",
                entry.owner
            );
        }
        ensure_optional_expiry(
            entry.expires.as_deref(),
            entry
                .path
                .as_deref()
                .or(entry.glob.as_deref())
                .unwrap_or("<unknown>"),
        )?;
    }
    Ok(())
}

fn ensure_optional_expiry(expires: Option<&str>, label: &str) -> Result<()> {
    if let Some(expires) = expires {
        let today = chrono::Utc::now().date_naive();
        let expires_date = NaiveDate::parse_from_str(expires, "%Y-%m-%d")
            .with_context(|| format!("{label} has invalid expires date"))?;
        if expires_date < today {
            bail!("{label} expired on {expires}");
        }
    }
    Ok(())
}

fn ensure_ledger_header(ledger: &ClippyPolicyLedger) -> Result<()> {
    if ledger.schema != 1 {
        bail!("policy/clippy-lints.toml schema must be 1");
    }
    if !ledger.policy.panic_free_tests {
        bail!("policy/clippy-lints.toml must keep panic_free_tests = true");
    }
    if ledger.policy.allow_test_carveouts {
        bail!("policy/clippy-lints.toml must keep allow_test_carveouts = false");
    }
    if ledger.policy.suppression_style != "expect-with-reason" {
        bail!("policy/clippy-lints.toml must use suppression_style = \"expect-with-reason\"");
    }
    if ledger.policy.blanket_categories {
        bail!("policy/clippy-lints.toml must keep blanket_categories = false");
    }
    for lint in &ledger.lint {
        require_non_empty(&lint.name, "lint.name")?;
        require_non_empty(&lint.level, "lint.level")?;
        require_non_empty(&lint.status, "lint.status")?;
        require_non_empty(&lint.class, "lint.class")?;
        require_non_empty(&lint.reason, "lint.reason")?;
        if lint.status != "active" && lint.status != "planned" {
            bail!("lint {} has unsupported status {}", lint.name, lint.status);
        }
        if lint.status == "planned" && lint.activate_when_msrv.is_none() {
            bail!("planned lint {} is missing activate_when_msrv", lint.name);
        }
    }
    Ok(())
}

fn ensure_msrv_matches(cargo: &CargoManifest, ledger: &ClippyPolicyLedger) -> Result<()> {
    if cargo.workspace.package.rust_version != ledger.msrv {
        bail!(
            "workspace.package.rust-version {} does not match policy msrv {}",
            cargo.workspace.package.rust_version,
            ledger.msrv
        );
    }
    Ok(())
}

fn ensure_lints_match_manifest(lints: &WorkspaceLints, ledger: &ClippyPolicyLedger) -> Result<()> {
    let manifest_lints = flatten_lints(lints);
    for lint in active_lints(ledger) {
        let actual = manifest_lints.get(&lint.name).ok_or_else(|| {
            anyhow::anyhow!("active lint {} is missing from root Cargo.toml", lint.name)
        })?;
        if actual != &lint.level {
            bail!(
                "active lint {} is {} in policy but {} in Cargo.toml",
                lint.name,
                lint.level,
                actual
            );
        }
    }
    for (name, level) in manifest_lints {
        let Some(policy_lint) = ledger.lint.iter().find(|lint| lint.name == name) else {
            bail!("Cargo.toml lint {name} = {level} is missing from policy/clippy-lints.toml");
        };
        if policy_lint.status != "active" {
            bail!(
                "Cargo.toml lint {name} is marked {} in policy",
                policy_lint.status
            );
        }
    }
    Ok(())
}

fn ensure_planned_lints_not_active(
    lints: &WorkspaceLints,
    ledger: &ClippyPolicyLedger,
) -> Result<()> {
    let manifest_lints = flatten_lints(lints);
    for lint in planned_lints(ledger) {
        if manifest_lints.contains_key(&lint.name) {
            bail!(
                "planned lint {} must not be active before MSRV {}",
                lint.name,
                lint.activate_when_msrv.as_deref().unwrap_or("<missing>")
            );
        }
    }
    Ok(())
}

fn ensure_workspace_lint_inheritance(root: &Path) -> Result<()> {
    let metadata = cargo_metadata(root)?;
    let enforce = env::var_os("ADZE_ENFORCE_LINT_INHERITANCE").is_some();
    let mut missing = Vec::new();
    for package in metadata.packages {
        if !metadata
            .workspace_members
            .iter()
            .any(|id| id == &package.id)
        {
            continue;
        }
        let manifest_path = Path::new(&package.manifest_path);
        let manifest = read_toml::<PackageManifest>(manifest_path)?;
        if !manifest.lints.map(|lints| lints.workspace).unwrap_or(false) {
            let relative_path = manifest_path
                .strip_prefix(&metadata.workspace_root)
                .unwrap_or(manifest_path);
            missing.push(format!("{} ({})", package.name, relative_path.display()));
        }
    }
    if !missing.is_empty() {
        if enforce {
            bail!(
                "{} workspace members are missing [lints] workspace = true: {}",
                missing.len(),
                missing.join(", ")
            );
        }
        println!(
            "⚠️  lint inheritance is advisory in this PR: {} workspace members still need [lints] workspace = true",
            missing.len()
        );
    }
    Ok(())
}

fn ensure_no_test_carveouts(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    for (line_number, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        for carveout in CLIPPY_TEST_CARVEOUTS {
            if trimmed.starts_with(carveout) {
                bail!(
                    "clippy.toml must not set {carveout} (line {})",
                    line_number + 1
                );
            }
        }
    }
    Ok(())
}

fn ensure_debt_is_reviewable(ledger: &ClippyDebtLedger) -> Result<()> {
    if ledger.schema != 1 {
        bail!("policy/clippy-debt.toml schema must be 1");
    }
    let today = chrono::Utc::now().date_naive();
    for entry in &ledger.debt {
        require_non_empty(&entry.lint, "debt.lint")?;
        require_non_empty(&entry.path, "debt.path")?;
        require_non_empty(&entry.owner, "debt.owner")?;
        require_non_empty(&entry.reason, "debt.reason")?;
        require_non_empty(&entry.expires, "debt.expires")?;
        let expires = NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d")
            .with_context(|| format!("debt {} has invalid expires date", entry.lint))?;
        if expires < today {
            bail!(
                "debt {} for {} expired on {}",
                entry.lint,
                entry.path,
                entry.expires
            );
        }
    }
    Ok(())
}

fn flatten_lints(lints: &WorkspaceLints) -> LintMap {
    let mut out = LintMap::new();
    out.extend(
        lints
            .rust
            .iter()
            .map(|(name, level)| (name.clone(), level.clone())),
    );
    out.extend(
        lints
            .clippy
            .iter()
            .map(|(name, level)| (format!("clippy::{name}"), level.clone())),
    );
    out
}

fn active_lints(ledger: &ClippyPolicyLedger) -> impl Iterator<Item = &LintEntry> {
    ledger.lint.iter().filter(|lint| lint.status == "active")
}

fn planned_lints(ledger: &ClippyPolicyLedger) -> impl Iterator<Item = &LintEntry> {
    ledger.lint.iter().filter(|lint| lint.status == "planned")
}

fn require_non_empty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{field} must not be empty");
    }
    Ok(())
}

fn repo_root() -> Result<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to run git rev-parse --show-toplevel")?;
    if !output.status.success() {
        bail!(
            "git rev-parse --show-toplevel failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(std::path::PathBuf::from(
        String::from_utf8(output.stdout)
            .context("git returned a non-utf8 repo root")?
            .trim(),
    ))
}

fn cargo_metadata(root: &Path) -> Result<CargoMetadata> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()
        .context("failed to run cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout).context("failed to parse cargo metadata")
}

fn read_toml<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))
}
