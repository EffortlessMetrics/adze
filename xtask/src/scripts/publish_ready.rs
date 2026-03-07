use super::{
    metadata::{MetadataPackage, PublishSetting, cargo_metadata},
    support::{combined_output_lines, fail, info, pass, repo_root, warn},
};
use anyhow::{Context, Result, bail};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const CORE_PUBLISH_READY_CRATES: &[&str] = &[
    "adze-ir",
    "adze-glr-core",
    "adze-tablegen",
    "adze-common",
    "adze-macro",
    "adze",
    "adze-tool",
];

const SUPPORTED_PUBLISH_READY_CRATES: &[&str] = &[
    "adze-ir",
    "adze-glr-core",
    "adze-tablegen",
    "adze-common",
    "adze-macro",
    "adze",
    "adze-runtime",
    "adze-tool",
];

const MAX_CRATES_IO_FILE_SIZE: u64 = 10 * 1024 * 1024;

#[derive(Clone, Copy)]
enum PublishReadinessProfile {
    Core,
    Supported,
}

pub(crate) fn run_publish_ready_core(_fix: bool) -> Result<()> {
    run_publish_ready(PublishReadinessProfile::Core)
}

pub(crate) fn run_publish_readiness() -> Result<()> {
    run_publish_ready(PublishReadinessProfile::Supported)
}

fn run_publish_ready(profile: PublishReadinessProfile) -> Result<()> {
    let repo_root = repo_root()?;
    let metadata = cargo_metadata(true)?;
    let packages_by_name = metadata
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<std::collections::HashMap<_, _>>();

    let (
        title,
        crate_names,
        required_fields,
        validate_publish_flag,
        file_size_limit,
        allow_dirty,
        check_todo_markers,
    ) = match profile {
        PublishReadinessProfile::Core => (
            "Adze Publish Readiness Check",
            CORE_PUBLISH_READY_CRATES,
            &[
                "name",
                "version",
                "license",
                "description",
                "repository",
                "homepage",
            ][..],
            false,
            None,
            false,
            true,
        ),
        PublishReadinessProfile::Supported => (
            "Publish Readiness Check - All Supported Crates",
            SUPPORTED_PUBLISH_READY_CRATES,
            &[
                "name",
                "version",
                "description",
                "license",
                "repository",
                "edition",
            ][..],
            true,
            Some(MAX_CRATES_IO_FILE_SIZE),
            true,
            false,
        ),
    };

    println!("{}", "=".repeat(title.len() + 4));
    println!("  {title}");
    println!("{}", "=".repeat(title.len() + 4));
    println!();

    let mut errors = 0usize;
    let mut warnings = 0usize;

    println!("[1/7] License files");
    for license_file in ["LICENSE-APACHE", "LICENSE-MIT"] {
        if repo_root.join(license_file).is_file() {
            pass(&format!("{license_file} exists"));
        } else {
            fail(&format!("{license_file} missing at repo root"), &mut errors);
        }
    }
    println!();

    println!("[2/7] MSRV consistency");
    let workspace_msrv = read_quoted_assignment(&repo_root.join("Cargo.toml"), "rust-version")
        .context("failed to read workspace rust-version")?;
    let toolchain_channel =
        read_quoted_assignment(&repo_root.join("rust-toolchain.toml"), "channel")
            .context("failed to read rust-toolchain channel")?;
    if workspace_msrv == toolchain_channel {
        pass(&format!(
            "Workspace MSRV ({workspace_msrv}) matches rust-toolchain.toml ({toolchain_channel})"
        ));
    } else {
        fail(
            &format!("MSRV mismatch: workspace={workspace_msrv} toolchain={toolchain_channel}"),
            &mut errors,
        );
    }
    println!();

    println!("[3/7] Cargo.toml required fields");
    for crate_name in crate_names {
        let package = packages_by_name
            .get(crate_name)
            .with_context(|| format!("missing workspace package metadata for {crate_name}"))?;
        println!("  -- {crate_name} ({})", package.manifest_path);
        for field in required_fields {
            if package_field(package, field).is_some() {
                pass(&format!("{field} present"));
            } else {
                fail(
                    &format!("{field} missing in {}", package.manifest_path),
                    &mut errors,
                );
            }
        }
        if matches!(profile, PublishReadinessProfile::Supported) {
            if looks_like_semver(&package.version) {
                pass(&format!("version '{}' is valid semver", package.version));
            } else {
                warn(
                    &format!(
                        "version '{}' may not be valid semver (expected X.Y.Z[-pre][+build])",
                        package.version
                    ),
                    &mut warnings,
                );
            }
        }
    }
    println!();

    println!("[4/7] README availability");
    for crate_name in crate_names {
        let package = packages_by_name
            .get(crate_name)
            .with_context(|| format!("missing workspace package metadata for {crate_name}"))?;
        let manifest_dir = manifest_dir(package)?;
        let primary_readme = resolve_readme_path(package);
        if primary_readme.is_file() {
            pass(&format!(
                "{crate_name} README found ({})",
                package.readme.as_deref().unwrap_or("README.md")
            ));
            continue;
        }
        if matches!(profile, PublishReadinessProfile::Supported) {
            let fallback = manifest_dir.join("README");
            if fallback.is_file() {
                pass(&format!(
                    "{crate_name} README found (README without extension)"
                ));
                continue;
            }
        }
        fail(
            &format!(
                "{crate_name} README not found at {}",
                primary_readme.display()
            ),
            &mut errors,
        );
    }
    println!();

    if validate_publish_flag {
        println!("[5/7] publish flag (should not be false)");
        for crate_name in crate_names {
            let package = packages_by_name
                .get(crate_name)
                .with_context(|| format!("missing workspace package metadata for {crate_name}"))?;
            match package.publish.as_ref() {
                Some(PublishSetting::Bool(false)) => fail(
                    &format!(
                        "{crate_name}: publish = false is set (must be true or absent for public crates)"
                    ),
                    &mut errors,
                ),
                Some(PublishSetting::Bool(true)) => {
                    pass(&format!("{crate_name}: publish = true is set"));
                }
                _ => pass(&format!(
                    "{crate_name}: publish flag not set (defaults to true)"
                )),
            }
        }
    } else {
        println!("[5/7] Path dependency leak check (deps must have version)");
        for crate_name in crate_names {
            let package = packages_by_name
                .get(crate_name)
                .with_context(|| format!("missing workspace package metadata for {crate_name}"))?;
            let missing_versions = package
                .dependencies
                .iter()
                .filter(|dependency| dependency.path.is_some())
                .filter(|dependency| {
                    dependency.req.trim().is_empty() || dependency.req.trim() == "*"
                })
                .map(|dependency| dependency.name.as_str())
                .collect::<Vec<_>>();
            if missing_versions.is_empty() {
                pass(&format!(
                    "{crate_name}: all path dependencies have versions"
                ));
            } else {
                for dependency_name in missing_versions {
                    fail(
                        &format!(
                            "{crate_name}: dependency '{}' has path but no version in {}",
                            dependency_name, package.manifest_path
                        ),
                        &mut errors,
                    );
                }
            }
        }
    }
    println!();

    println!(
        "[6/7] {}",
        if check_todo_markers {
            "TODO/FIXME/HACK in public API (lib.rs)"
        } else {
            "cargo package --list (dry-run packaging)"
        }
    );
    if check_todo_markers {
        for crate_name in crate_names {
            let package = packages_by_name
                .get(crate_name)
                .with_context(|| format!("missing workspace package metadata for {crate_name}"))?;
            let lib_rs = manifest_dir(package)?.join("src/lib.rs");
            if !lib_rs.is_file() {
                warn(&format!("{crate_name}: no lib.rs found"), &mut warnings);
                continue;
            }
            let contents = fs::read_to_string(&lib_rs)
                .with_context(|| format!("failed to read {}", lib_rs.display()))?;
            let markers = contents
                .lines()
                .enumerate()
                .filter_map(|(index, line)| {
                    if contains_marker(line) {
                        Some(format!("{}:{}", index + 1, line.trim()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if markers.is_empty() {
                pass(&format!("{crate_name}: no TODO/FIXME/HACK in lib.rs"));
            } else {
                warn(
                    &format!(
                        "{crate_name}: {} TODO/FIXME/HACK markers in lib.rs",
                        markers.len()
                    ),
                    &mut warnings,
                );
                for line in markers.iter().take(5) {
                    info(line);
                }
            }
        }
    } else {
        for crate_name in crate_names {
            println!("  -- {crate_name}");
            let output = cargo_package_list(crate_name, allow_dirty)?;
            if output.status.success() {
                let count = String::from_utf8_lossy(&output.stdout).lines().count();
                pass(&format!("package --list succeeded ({count} files)"));
            } else {
                fail(
                    &format!("package --list failed for {crate_name}"),
                    &mut errors,
                );
                for line in combined_output_lines(&output).into_iter().take(10) {
                    info(&line);
                }
            }
        }
    }
    println!();

    println!(
        "[7/7] {}",
        if let Some(limit) = file_size_limit {
            format!(
                "File size limits (crates.io max {} MB)",
                limit / (1024 * 1024)
            )
        } else {
            "cargo package --list (dry-run packaging)".to_owned()
        }
    );
    if let Some(limit) = file_size_limit {
        for crate_name in crate_names {
            let package = packages_by_name
                .get(crate_name)
                .with_context(|| format!("missing workspace package metadata for {crate_name}"))?;
            let output = cargo_package_list(crate_name, allow_dirty)?;
            if !output.status.success() {
                warn(
                    &format!("{crate_name}: could not list package contents (skipping size check)"),
                    &mut warnings,
                );
                continue;
            }
            let manifest_dir = manifest_dir(package)?;
            let mut oversized = Vec::new();
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let file_path = manifest_dir.join(line);
                if !file_path.is_file() {
                    continue;
                }
                let size = file_path
                    .metadata()
                    .with_context(|| format!("failed to stat {}", file_path.display()))?
                    .len();
                if size > limit {
                    oversized.push(format!("{} ({} bytes)", line, size));
                }
            }
            if oversized.is_empty() {
                pass(&format!(
                    "{crate_name}: all packaged files are within size limits"
                ));
            } else {
                for file in oversized {
                    fail(
                        &format!("{crate_name}: oversized file {file} > 10MB"),
                        &mut errors,
                    );
                }
            }
        }
    } else {
        for crate_name in crate_names {
            println!("  -- {crate_name}");
            let output = cargo_package_list(crate_name, allow_dirty)?;
            if output.status.success() {
                let count = String::from_utf8_lossy(&output.stdout).lines().count();
                pass(&format!("package --list succeeded ({count} files)"));
            } else {
                fail(
                    &format!("package --list failed for {crate_name}"),
                    &mut errors,
                );
                for line in combined_output_lines(&output).into_iter().take(10) {
                    info(&line);
                }
            }
        }
    }
    println!();

    println!("{}", "=".repeat(title.len() + 4));
    match profile {
        PublishReadinessProfile::Core => {
            if errors == 0 && warnings == 0 {
                println!("All checks passed.");
            } else if errors == 0 {
                println!("Passed with {warnings} warning(s).");
            } else {
                println!("{errors} error(s), {warnings} warning(s).");
            }
            println!("{}", "=".repeat(title.len() + 4));
            if errors > 0 {
                bail!("publish readiness failed");
            }
        }
        PublishReadinessProfile::Supported => {
            println!("Summary:");
            println!("  Errors:   {errors}");
            println!("  Warnings: {warnings}");
            println!("{}", "=".repeat(title.len() + 4));
            if errors > 0 {
                bail!("publish readiness failed");
            }
        }
    }

    Ok(())
}

fn manifest_dir(package: &MetadataPackage) -> Result<PathBuf> {
    let manifest = PathBuf::from(&package.manifest_path);
    manifest
        .parent()
        .map(Path::to_path_buf)
        .with_context(|| format!("missing manifest parent for {}", package.name))
}

fn resolve_readme_path(package: &MetadataPackage) -> PathBuf {
    let manifest_dir = PathBuf::from(&package.manifest_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    match package.readme.as_deref() {
        Some(path) => {
            let path = PathBuf::from(path);
            if path.is_absolute() {
                path
            } else {
                manifest_dir.join(path)
            }
        }
        None => manifest_dir.join("README.md"),
    }
}

fn package_field<'a>(package: &'a MetadataPackage, field: &str) -> Option<&'a str> {
    let value = match field {
        "name" => Some(package.name.as_str()),
        "version" => Some(package.version.as_str()),
        "license" => package.license.as_deref(),
        "description" => package.description.as_deref(),
        "repository" => package.repository.as_deref(),
        "homepage" => package.homepage.as_deref(),
        "edition" => Some(package.edition.as_str()),
        _ => None,
    }?;
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn read_quoted_assignment(path: &Path, key: &str) -> Result<String> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(key) {
            continue;
        }
        if let Some(value) = trimmed.split('"').nth(1) {
            return Ok(value.to_owned());
        }
    }
    bail!("missing assignment for {key} in {}", path.display())
}

fn contains_marker(line: &str) -> bool {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|token| matches!(token, "TODO" | "FIXME" | "HACK"))
}

fn looks_like_semver(version: &str) -> bool {
    let (main, suffix) = version.split_once(['-', '+']).unwrap_or((version, ""));
    let parts = main.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()))
    {
        return false;
    }
    if suffix.is_empty() {
        return true;
    }
    suffix
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '+')
}

fn cargo_package_list(crate_name: &str, allow_dirty: bool) -> Result<std::process::Output> {
    let mut args = vec![
        OsString::from("package"),
        OsString::from("--list"),
        OsString::from("-p"),
        OsString::from(crate_name),
        OsString::from("--no-verify"),
    ];
    if allow_dirty {
        args.push(OsString::from("--allow-dirty"));
    }
    Command::new("cargo")
        .args(args)
        .output()
        .with_context(|| format!("failed to run cargo package --list for {crate_name}"))
}

#[cfg(test)]
mod tests {
    use super::{contains_marker, looks_like_semver};

    #[test]
    fn looks_like_semver_accepts_release_and_prerelease_versions() {
        assert!(looks_like_semver("1.2.3"));
        assert!(looks_like_semver("1.2.3-alpha.1"));
    }

    #[test]
    fn looks_like_semver_rejects_invalid_versions() {
        assert!(!looks_like_semver("1.2"));
        assert!(!looks_like_semver("1.2.x"));
    }

    #[test]
    fn contains_marker_detects_public_api_markers() {
        assert!(contains_marker("// TODO: trim this later"));
        assert!(contains_marker("// FIXME(issue): still open"));
        assert!(!contains_marker("// plain comment"));
        assert!(!contains_marker("// HACKATHON demo"));
        assert!(!contains_marker("// internal_TODO_marker"));
    }
}
