use super::{
    metadata::{cargo_metadata, packages_with_feature, workspace_packages},
    support::{combine_output, repo_root},
};
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use std::{collections::HashSet, fs, path::Path, process::Command};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ClippyMode {
    Default,
    C2rust,
}

impl ClippyMode {
    fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::C2rust => "c2rust",
        }
    }
}

pub(crate) fn run_clippy_per_package(mode: ClippyMode) -> Result<()> {
    let metadata = cargo_metadata(true)?;
    let quarantine = load_quarantine()?;
    let packages = workspace_packages(&metadata);
    let c2rust_packages = packages_with_feature(&packages, "tree-sitter-c2rust");
    let mut ran_checks = false;

    for package in &packages {
        if quarantine.contains(&package.name) {
            println!("  skip (quarantined): {}", package.name);
            continue;
        }

        match mode {
            ClippyMode::Default => {
                println!("  clippy (default): {}", package.name);
                let status = Command::new("cargo")
                    .args([
                        "clippy",
                        "-q",
                        "-p",
                        &package.name,
                        "--all-targets",
                        "--no-deps",
                        "--",
                        "-D",
                        "warnings",
                    ])
                    .status()
                    .with_context(|| format!("failed to run clippy for {}", package.name))?;
                if !status.success() {
                    bail!(
                        "Clippy failed for package: {} (default features)\n  To reproduce: cargo clippy -q -p {} --all-targets --no-deps -- -D warnings",
                        package.name,
                        package.name
                    );
                }
                ran_checks = true;
            }
            ClippyMode::C2rust => {
                if !c2rust_packages.contains(package.name.as_str()) {
                    println!("  skip (no c2rust feature): {}", package.name);
                    continue;
                }
                println!("  clippy (c2rust): {}", package.name);
                let status = Command::new("cargo")
                    .args([
                        "clippy",
                        "-q",
                        "-p",
                        &package.name,
                        "--all-targets",
                        "--no-default-features",
                        "--features",
                        "tree-sitter-c2rust",
                        "--no-deps",
                        "--",
                        "-D",
                        "warnings",
                    ])
                    .status()
                    .with_context(|| format!("failed to run clippy for {}", package.name))?;
                if !status.success() {
                    bail!(
                        "Clippy failed for package: {} (tree-sitter-c2rust)\n  To reproduce: cargo clippy -q -p {} --all-targets --no-default-features --features tree-sitter-c2rust --no-deps -- -D warnings",
                        package.name,
                        package.name
                    );
                }
                ran_checks = true;
            }
        }
    }

    if ran_checks {
        println!("OK: clippy-per-package ({})", mode.label());
    } else {
        println!("WARNING: no packages were checked (all quarantined or skipped)");
    }

    Ok(())
}

pub(crate) fn run_clippy_collect(outdir: &Path) -> Result<()> {
    let metadata = cargo_metadata(true)?;
    let packages = workspace_packages(&metadata);
    let quarantine = load_quarantine()?;
    let c2rust_packages = packages_with_feature(&packages, "tree-sitter-c2rust");
    fs::create_dir_all(outdir).with_context(|| format!("failed to create {}", outdir.display()))?;

    println!("Packages to consider:");
    for package in &packages {
        if quarantine.contains(&package.name) {
            println!("  skip (quarantined): {}", package.name);
        } else {
            println!("  will check (default): {}", package.name);
        }
    }

    let mut failures_default = Vec::new();
    let mut failures_c2rust = Vec::new();

    println!();
    println!("=== Running default-feature clippy per-package (no-deps) ===");
    for package in &packages {
        if quarantine.contains(&package.name) {
            continue;
        }
        let out = outdir.join(format!("{}-default.txt", package.name.replace('/', "-")));
        println!();
        println!("--- {}: default -> {}", package.name, out.display());
        let output = Command::new("cargo")
            .args([
                "clippy",
                "-p",
                &package.name,
                "--all-targets",
                "--no-deps",
                "--",
                "-D",
                "warnings",
            ])
            .output()
            .with_context(|| format!("failed to run default clippy for {}", package.name))?;
        fs::write(&out, combine_output(&output))
            .with_context(|| format!("failed to write {}", out.display()))?;
        if output.status.success() {
            println!("OK: {}", package.name);
        } else {
            println!(
                "FAILED: {} (exit {:?}) -- log: {}",
                package.name,
                output.status.code(),
                out.display()
            );
            failures_default.push(package.name.clone());
        }
    }

    println!();
    println!("=== Running tree-sitter-c2rust feature clippy (no-deps) ===");
    for package in &packages {
        if quarantine.contains(&package.name) {
            continue;
        }
        if !c2rust_packages.contains(package.name.as_str()) {
            println!("  skip (no c2rust feature): {}", package.name);
            continue;
        }
        let out = outdir.join(format!("{}-c2rust.txt", package.name.replace('/', "-")));
        println!();
        println!("--- {}: c2rust -> {}", package.name, out.display());
        let output = Command::new("cargo")
            .args([
                "clippy",
                "-p",
                &package.name,
                "--all-targets",
                "--no-default-features",
                "--features",
                "tree-sitter-c2rust",
                "--no-deps",
                "--",
                "-D",
                "warnings",
            ])
            .output()
            .with_context(|| format!("failed to run c2rust clippy for {}", package.name))?;
        fs::write(&out, combine_output(&output))
            .with_context(|| format!("failed to write {}", out.display()))?;
        if output.status.success() {
            println!("OK: {}", package.name);
        } else {
            println!(
                "FAILED: {} (exit {:?}) -- log: {}",
                package.name,
                output.status.code(),
                out.display()
            );
            failures_c2rust.push(package.name.clone());
        }
    }

    println!();
    println!("=== Summary ===");
    println!("Reports saved under: {}/", outdir.display());
    if failures_default.is_empty() && failures_c2rust.is_empty() {
        println!("All non-quarantined packages passed Clippy (both modes where applicable).");
        return Ok(());
    }

    if failures_default.is_empty() {
        println!("No default-feature failures.");
    } else {
        println!("Default-feature failures ({}):", failures_default.len());
        for package in &failures_default {
            println!(
                "  - {} -> {}/{}-default.txt",
                package,
                outdir.display(),
                package.replace('/', "-")
            );
        }
    }

    if failures_c2rust.is_empty() {
        println!("No c2rust-mode failures.");
    } else {
        println!("c2rust-mode failures ({}):", failures_c2rust.len());
        for package in &failures_c2rust {
            println!(
                "  - {} -> {}/{}-c2rust.txt",
                package,
                outdir.display(),
                package.replace('/', "-")
            );
        }
    }

    bail!("One or more packages failed Clippy. See logs.");
}

fn load_quarantine() -> Result<HashSet<String>> {
    let path = repo_root()?.join(".clippy-quarantine");
    if !path.is_file() {
        return Ok(HashSet::new());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(parse_quarantine(&contents))
}

fn parse_quarantine(contents: &str) -> HashSet<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_quarantine;
    use std::collections::HashSet;

    #[test]
    fn parse_quarantine_ignores_comments_and_blank_lines() {
        let parsed = parse_quarantine("# note\n\nadze\n  adze-tool  \n");

        assert_eq!(
            parsed,
            HashSet::from(["adze".to_owned(), "adze-tool".to_owned()])
        );
    }
}
