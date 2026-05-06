use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, Utc};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct LintEntry {
    name: String,
    level: String,
    status: String,
    activate_when_msrv: Option<String>,
}

#[derive(Debug)]
struct DebtEntry {
    lint: Option<String>,
    path: Option<String>,
    owner: Option<String>,
    reason: Option<String>,
    expires: Option<String>,
}

pub fn check_lint_policy() -> Result<()> {
    let root = repo_root()?;
    let cargo_toml = fs::read_to_string(root.join("Cargo.toml")).context("read Cargo.toml")?;
    let policy_toml = fs::read_to_string(root.join("policy/clippy-lints.toml"))
        .context("read policy/clippy-lints.toml")?;

    let workspace_msrv =
        workspace_msrv(&cargo_toml).context("workspace.package.rust-version missing")?;
    let policy_msrv = scalar(&policy_toml, "msrv").context("policy msrv missing")?;
    ensure_same_msrv(&workspace_msrv, &policy_msrv)?;
    ensure_toolchain_msrv(&root, &workspace_msrv)?;
    ensure_policy_posture(&policy_toml)?;
    ensure_lints_inherit(&root, &cargo_toml)?;
    ensure_no_test_carveouts(&root)?;
    ensure_active_lints_match(&cargo_toml, &policy_toml)?;
    ensure_planned_lints_are_gated(&cargo_toml, &policy_toml, &workspace_msrv)?;
    ensure_debt_entries(&root)?;

    println!("✓ lint policy is coherent for MSRV {workspace_msrv}");
    Ok(())
}

fn repo_root() -> Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("run git rev-parse --show-toplevel")?;
    if !output.status.success() {
        bail!("git rev-parse --show-toplevel failed");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

fn workspace_msrv(cargo_toml: &str) -> Option<String> {
    let section = section(cargo_toml, "workspace.package")?;
    scalar(section, "rust-version")
}

fn ensure_same_msrv(workspace_msrv: &str, policy_msrv: &str) -> Result<()> {
    if normalize_version(workspace_msrv) != normalize_version(policy_msrv) {
        bail!(
            "workspace MSRV {workspace_msrv} does not match policy/clippy-lints.toml msrv {policy_msrv}"
        );
    }
    Ok(())
}

fn ensure_toolchain_msrv(root: &Path, workspace_msrv: &str) -> Result<()> {
    let toolchain =
        fs::read_to_string(root.join("rust-toolchain.toml")).context("read rust-toolchain.toml")?;
    let channel = scalar(&toolchain, "channel").context("rust-toolchain.toml channel missing")?;
    if normalize_version(&channel) != normalize_version(workspace_msrv) {
        bail!(
            "rust-toolchain.toml channel {channel} does not match workspace MSRV {workspace_msrv}"
        );
    }
    Ok(())
}

fn ensure_policy_posture(policy_toml: &str) -> Result<()> {
    let policy = section(policy_toml, "policy").context("[policy] section missing")?;
    require_scalar(policy, "panic_free_tests", "true")?;
    require_scalar(policy, "allow_test_carveouts", "false")?;
    require_scalar(policy, "suppression_style", "expect-with-reason")?;
    require_scalar(policy, "blanket_categories", "false")?;
    Ok(())
}

fn ensure_lints_inherit(root: &Path, cargo_toml: &str) -> Result<()> {
    let members = workspace_members(cargo_toml)?;
    let mut missing = Vec::new();
    for member in members {
        let manifest = root.join(member).join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let text = fs::read_to_string(&manifest)
            .with_context(|| format!("read {}", manifest.display()))?;
        if !has_lints_workspace(&text) {
            missing.push(path_for_display(root, &manifest));
        }
    }
    if !missing.is_empty() {
        bail!(
            "workspace members missing [lints] workspace = true:\n{}",
            missing.join("\n")
        );
    }
    Ok(())
}

fn ensure_no_test_carveouts(root: &Path) -> Result<()> {
    let path = root.join("clippy.toml");
    if !path.exists() {
        bail!("clippy.toml is required, even if it only documents that no repo carveouts exist");
    }
    let text = fs::read_to_string(path).context("read clippy.toml")?;
    for key in [
        "allow-unwrap-in-tests",
        "allow-expect-in-tests",
        "allow-panic-in-tests",
        "allow-indexing-slicing-in-tests",
        "allow-dbg-in-tests",
    ] {
        for line in text.lines().map(str::trim) {
            if line.starts_with(key) && line.contains("true") {
                bail!("clippy.toml must not enable test carveout {key}");
            }
        }
    }
    Ok(())
}

fn ensure_active_lints_match(cargo_toml: &str, policy_toml: &str) -> Result<()> {
    let cargo_lints = cargo_lints(cargo_toml);
    let active: Vec<_> = parse_lints(policy_toml)
        .into_iter()
        .filter(|lint| lint.status == "active")
        .collect();
    let mut missing = Vec::new();
    for lint in active {
        match cargo_lints.get(&lint.name) {
            Some(level) if level == &lint.level => {}
            Some(level) => missing.push(format!(
                "{} is {level:?} in Cargo.toml but {:?} in policy",
                lint.name, lint.level
            )),
            None => missing.push(format!(
                "{} is active in policy but missing from Cargo.toml",
                lint.name
            )),
        }
    }
    if !missing.is_empty() {
        bail!("active lint policy mismatch:\n{}", missing.join("\n"));
    }
    Ok(())
}

fn ensure_planned_lints_are_gated(
    cargo_toml: &str,
    policy_toml: &str,
    workspace_msrv: &str,
) -> Result<()> {
    let cargo_lints = cargo_lints(cargo_toml);
    let mut early = Vec::new();
    for lint in parse_lints(policy_toml)
        .into_iter()
        .filter(|lint| lint.status == "planned")
    {
        if lint
            .activate_when_msrv
            .as_deref()
            .is_some_and(|msrv| version_lt(workspace_msrv, msrv))
            && cargo_lints.contains_key(&lint.name)
        {
            early.push(format!(
                "{} is active before MSRV {}",
                lint.name,
                lint.activate_when_msrv.unwrap()
            ));
        }
    }
    for planned in parse_planned(policy_toml) {
        if planned
            .activate_when_msrv
            .as_deref()
            .is_some_and(|msrv| version_lt(workspace_msrv, msrv))
            && cargo_lints.contains_key(&planned.name)
        {
            early.push(format!(
                "{} is active before MSRV {}",
                planned.name,
                planned.activate_when_msrv.unwrap()
            ));
        }
    }
    if !early.is_empty() {
        bail!("planned lints activated too early:\n{}", early.join("\n"));
    }
    Ok(())
}

fn ensure_debt_entries(root: &Path) -> Result<()> {
    let path = root.join("policy/clippy-debt.toml");
    let text = fs::read_to_string(path).context("read policy/clippy-debt.toml")?;
    let today = Utc::now().date_naive();
    let mut errors = Vec::new();
    for (index, debt) in parse_debt(&text).into_iter().enumerate() {
        let label = format!("debt #{}", index + 1);
        require_present(&mut errors, &label, "lint", debt.lint.as_deref());
        require_present(&mut errors, &label, "path", debt.path.as_deref());
        require_present(&mut errors, &label, "owner", debt.owner.as_deref());
        require_present(&mut errors, &label, "reason", debt.reason.as_deref());
        match debt.expires.as_deref() {
            Some(expires) if !expires.trim().is_empty() => {
                match NaiveDate::parse_from_str(expires, "%Y-%m-%d") {
                    Ok(date) if date >= today => {}
                    Ok(date) => errors.push(format!("{label} expired on {date}")),
                    Err(err) => errors.push(format!(
                        "{label} has invalid expires date {expires:?}: {err}"
                    )),
                }
            }
            _ => errors.push(format!("{label} missing expires")),
        }
    }
    if !errors.is_empty() {
        bail!("invalid clippy debt ledger:\n{}", errors.join("\n"));
    }
    Ok(())
}

fn require_present(errors: &mut Vec<String>, label: &str, field: &str, value: Option<&str>) {
    if value.is_none_or(|value| value.trim().is_empty()) {
        errors.push(format!("{label} missing {field}"));
    }
}

fn cargo_lints(cargo_toml: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for namespace in ["rust", "clippy"] {
        if let Some(sec) = section(cargo_toml, &format!("workspace.lints.{namespace}")) {
            for line in sec
                .lines()
                .map(strip_comment)
                .map(str::trim)
                .filter(|line| !line.is_empty())
            {
                if let Some((name, level)) = line.split_once('=') {
                    out.insert(
                        format!("{namespace}::{}", name.trim()),
                        unquote(level.trim()).to_owned(),
                    );
                }
            }
        }
    }
    out
}

fn workspace_members(cargo_toml: &str) -> Result<Vec<String>> {
    let workspace = section(cargo_toml, "workspace").context("[workspace] section missing")?;
    let mut members = Vec::new();
    let mut in_members = false;
    for raw in workspace.lines() {
        let line = strip_comment(raw).trim();
        if line.starts_with("members") && line.contains('[') {
            in_members = true;
        }
        if in_members {
            let mut rest = line;
            while let Some(start) = rest.find('"') {
                let after = &rest[start + 1..];
                let Some(end) = after.find('"') else { break };
                members.push(after[..end].to_owned());
                rest = &after[end + 1..];
            }
            if line.contains(']') {
                break;
            }
        }
    }
    if members.is_empty() {
        bail!("workspace members list is empty or could not be parsed");
    }
    Ok(members)
}

fn has_lints_workspace(text: &str) -> bool {
    section(text, "lints").is_some_and(|sec| {
        sec.lines()
            .map(strip_comment)
            .map(str::trim)
            .any(|line| line == "workspace = true")
    })
}

fn parse_lints(policy_toml: &str) -> Vec<LintEntry> {
    parse_array_table(policy_toml, "lint")
        .into_iter()
        .map(|block| LintEntry {
            name: scalar(block, "name").unwrap_or_default(),
            level: scalar(block, "level").unwrap_or_default(),
            status: scalar(block, "status").unwrap_or_default(),
            activate_when_msrv: scalar(block, "activate_when_msrv"),
        })
        .collect()
}

fn parse_planned(policy_toml: &str) -> Vec<LintEntry> {
    parse_array_table(policy_toml, "planned")
        .into_iter()
        .map(|block| LintEntry {
            name: scalar(block, "name").unwrap_or_default(),
            level: scalar(block, "level").unwrap_or_default(),
            status: "planned".to_owned(),
            activate_when_msrv: scalar(block, "activate_when_msrv"),
        })
        .collect()
}

fn parse_debt(text: &str) -> Vec<DebtEntry> {
    parse_array_table(text, "debt")
        .into_iter()
        .map(|block| DebtEntry {
            lint: scalar(block, "lint"),
            path: scalar(block, "path"),
            owner: scalar(block, "owner"),
            reason: scalar(block, "reason"),
            expires: scalar(block, "expires"),
        })
        .collect()
}

fn parse_array_table<'a>(text: &'a str, table: &str) -> Vec<&'a str> {
    let marker = format!("[[{table}]]");
    let mut blocks = Vec::new();
    let mut start = None;
    let mut offset = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            if let Some(start_offset) = start.take() {
                blocks.push(&text[start_offset..offset]);
            }
            if trimmed == marker {
                start = Some(offset + line.len() + 1);
            }
        }
        offset += line.len() + 1;
    }
    if let Some(start_offset) = start {
        blocks.push(&text[start_offset..]);
    }
    blocks
}

fn section<'a>(text: &'a str, wanted: &str) -> Option<&'a str> {
    let marker = format!("[{wanted}]");
    let start_line = text.find(&marker)?;
    let after = start_line + marker.len();
    let rest = &text[after..];
    let end = rest
        .find('\n')
        .map(|first_newline| {
            let body = &rest[first_newline + 1..];
            body.find("\n[")
                .map_or(text.len(), |next| after + first_newline + 1 + next)
        })
        .unwrap_or(text.len());
    Some(&text[after..end])
}

fn scalar(text: &str, key: &str) -> Option<String> {
    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() == key {
            return Some(unquote(right.trim()).to_owned());
        }
    }
    None
}

fn require_scalar(text: &str, key: &str, expected: &str) -> Result<()> {
    let actual = scalar(text, key).with_context(|| format!("[policy] {key} missing"))?;
    if actual != expected {
        bail!("[policy] {key} is {actual:?}, expected {expected:?}");
    }
    Ok(())
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(before, _)| before)
}

fn unquote(value: &str) -> &str {
    value.trim().trim_matches('"')
}

fn normalize_version(version: &str) -> String {
    let mut parts: Vec<_> = version.split('.').collect();
    while parts.len() < 3 {
        parts.push("0");
    }
    parts.join(".")
}

fn version_lt(left: &str, right: &str) -> bool {
    let parse = |version: &str| -> Vec<u64> {
        normalize_version(version)
            .split('.')
            .map(|part| part.parse().unwrap_or(0))
            .collect()
    };
    parse(left) < parse(right)
}

fn path_for_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
