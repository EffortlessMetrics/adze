use super::{
    metadata::{MetadataPackage, PublishSetting, cargo_metadata},
    release_surface::{
        ValidationOutcome, load_release_surface_file, validate_fixed_release_surface,
    },
    support::{combined_output_lines, env_truthy, fail, info, pass, repo_root, warn},
};
use anyhow::{Context, Result, bail};
use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

const REQUIRED_FIELDS: &[&str] = &[
    "name",
    "version",
    "license",
    "description",
    "repository",
    "homepage",
];

pub(crate) fn run_validate_package_workspace(
    crate_file: Option<PathBuf>,
    strict: bool,
) -> Result<()> {
    let repo_root = repo_root()?;
    let crate_file = crate_file.unwrap_or_else(|| {
        env::var_os("RELEASE_CRATE_FILE")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| repo_root.join("scripts/release-crates.txt"))
    });
    let strict = strict || env_truthy("STRICT_PUBLISH_SURFACE");
    let metadata = cargo_metadata(true)?;
    let release_surface = load_release_surface_file(&crate_file)?;
    let packages_by_name = metadata
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<HashMap<_, _>>();

    println!("===========================================");
    println!("  Package Validation (dev workspace lane)");
    println!("===========================================");
    println!("Release surface file: {}", crate_file.display());
    println!("Strict publish surface: {strict}");
    println!();

    let mut errors = 0usize;
    let mut warnings = 0usize;

    println!("[1/3] Release surface policy");
    let surface_outcome = validate_fixed_release_surface(
        &metadata,
        crate_file.clone(),
        release_surface.as_slice(),
        strict,
    );
    report_outcome(&surface_outcome, &mut errors, &mut warnings);
    validate_workspace_dependency_closure(
        release_surface.as_slice(),
        &packages_by_name,
        &crate_file,
        &mut errors,
    );
    if surface_outcome.errors.is_empty() && surface_outcome.warnings.is_empty() {
        pass("Release surface is coherent for the current workspace topology");
    }
    println!();

    println!("[2/3] Manifest hygiene");
    for crate_name in &release_surface {
        let Some(package) = packages_by_name.get(crate_name.as_str()) else {
            fail(
                &format!("missing workspace package metadata for {crate_name}"),
                &mut errors,
            );
            continue;
        };

        println!("  -- {crate_name} ({})", package.manifest_path);
        validate_manifest(package, &mut errors, &mut warnings)?;
    }
    println!();

    println!("[3/3] Package contents (`cargo package --list --no-verify`)");
    for crate_name in &release_surface {
        println!("  -- {crate_name}");
        let output = cargo_package_list(crate_name)?;
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
    println!();

    println!("===========================================");
    println!("Summary:");
    println!("  Errors:   {errors}");
    println!("  Warnings: {warnings}");
    println!("===========================================");
    if errors > 0 {
        bail!("package workspace validation failed");
    }

    Ok(())
}

fn validate_manifest(
    package: &MetadataPackage,
    errors: &mut usize,
    warnings: &mut usize,
) -> Result<()> {
    let mut crate_errors = 0usize;

    for field in REQUIRED_FIELDS {
        if package_field(package, field).is_some() {
            pass(&format!("{field} present"));
        } else {
            fail(
                &format!("{field} missing in {}", package.manifest_path),
                errors,
            );
            crate_errors += 1;
        }
    }

    if looks_like_semver(&package.version) {
        pass(&format!("version '{}' looks like semver", package.version));
    } else {
        warn(
            &format!(
                "version '{}' may not be valid semver (expected X.Y.Z[-pre][+build])",
                package.version
            ),
            warnings,
        );
    }

    match package.publish.as_ref() {
        None | Some(PublishSetting::Bool(true)) => {
            pass("publish metadata allows crates.io release");
        }
        Some(PublishSetting::Registries(registries))
            if registries.iter().any(|registry| registry == "crates.io") =>
        {
            pass("publish metadata allows crates.io release");
        }
        Some(PublishSetting::Bool(false)) | Some(PublishSetting::Registries(_)) => {
            fail(
                &format!(
                    "publish metadata does not allow crates.io release in {}",
                    package.manifest_path
                ),
                errors,
            );
            crate_errors += 1;
        }
    }

    let readme_path = resolve_readme_path(package);
    if readme_path.is_file() {
        pass(&format!("README found ({})", readme_path.display()));
    } else {
        fail(
            &format!(
                "README not found for {} at {}",
                package.name,
                readme_path.display()
            ),
            errors,
        );
        crate_errors += 1;
    }

    let missing_versions = package
        .dependencies
        .iter()
        .filter(|dependency| dependency.path.is_some())
        .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
        .filter(|dependency| dependency.req.trim().is_empty() || dependency.req.trim() == "*")
        .map(|dependency| dependency.name.as_str())
        .collect::<Vec<_>>();
    if missing_versions.is_empty() {
        pass("all non-dev path dependencies specify a version");
    } else {
        for dependency_name in missing_versions {
            fail(
                &format!(
                    "dependency '{}' has path but no version in {}",
                    dependency_name, package.manifest_path
                ),
                errors,
            );
            crate_errors += 1;
        }
    }

    if crate_errors == 0 {
        pass("manifest hygiene checks passed");
    }
    Ok(())
}

fn report_outcome(outcome: &ValidationOutcome, errors: &mut usize, warnings: &mut usize) {
    for warning in &outcome.warnings {
        warn(warning, warnings);
    }
    for error in &outcome.errors {
        fail(error, errors);
    }
}

fn validate_workspace_dependency_closure(
    release_surface: &[String],
    packages_by_name: &HashMap<&str, &MetadataPackage>,
    crate_file: &Path,
    errors: &mut usize,
) {
    let allowlist = release_surface
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();

    for crate_name in release_surface {
        let Some(package) = packages_by_name.get(crate_name.as_str()) else {
            continue;
        };

        let mut missing_workspace_deps = package
            .dependencies
            .iter()
            .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
            .filter(|dependency| !dependency.optional)
            .filter(|dependency| dependency.path.is_some())
            .filter(|dependency| dependency.name != package.name)
            .filter(|dependency| packages_by_name.contains_key(dependency.name.as_str()))
            .filter(|dependency| !allowlist.contains(dependency.name.as_str()))
            .map(|dependency| dependency.name.as_str())
            .collect::<Vec<_>>();
        missing_workspace_deps.sort_unstable();
        missing_workspace_deps.dedup();

        for dependency_name in missing_workspace_deps {
            fail(
                &format!(
                    "Dependency closure violation: '{crate_name}' depends on workspace crate '{}' outside {}",
                    dependency_name,
                    crate_file.display()
                ),
                errors,
            );
        }
    }
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
        _ => None,
    }?;
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
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

fn cargo_package_list(crate_name: &str) -> Result<std::process::Output> {
    Command::new("cargo")
        .args([
            OsString::from("package"),
            OsString::from("--list"),
            OsString::from("-p"),
            OsString::from(crate_name),
            OsString::from("--no-verify"),
            OsString::from("--allow-dirty"),
        ])
        .output()
        .with_context(|| format!("failed to run cargo package --list for {crate_name}"))
}

#[cfg(test)]
mod tests {
    use super::{looks_like_semver, validate_workspace_dependency_closure};
    use crate::scripts::metadata::MetadataPackage;
    use crate::scripts::metadata::{MetadataDependency, PublishSetting};
    use std::collections::HashMap;
    use std::path::Path;

    fn package(name: &str, deps: &[MetadataDependency]) -> MetadataPackage {
        MetadataPackage {
            id: format!("{name} 0.1.0 (path+file:///tmp/{name})"),
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
            description: Some("desc".to_owned()),
            license: Some("MIT".to_owned()),
            repository: Some("https://example.com".to_owned()),
            homepage: Some("https://example.com".to_owned()),
            edition: "2024".to_owned(),
            manifest_path: format!("/tmp/{name}/Cargo.toml"),
            publish: Some(PublishSetting::Registries(vec!["crates.io".to_owned()])),
            readme: Some("/tmp/README.md".to_owned()),
            dependencies: deps.to_vec(),
            features: HashMap::new(),
        }
    }

    fn dep(name: &str, kind: Option<&str>) -> MetadataDependency {
        MetadataDependency {
            name: name.to_owned(),
            kind: kind.map(ToOwned::to_owned),
            optional: false,
            path: Some(format!("/tmp/{name}")),
            req: "0.1.0".to_owned(),
        }
    }

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
    fn dependency_closure_flags_missing_workspace_path_dependency() {
        let a = package("a", &[dep("b", None)]);
        let b = package("b", &[]);
        let packages = HashMap::from([("a", &a), ("b", &b)]);
        let release_surface = vec!["a".to_owned()];
        let mut errors = 0usize;

        validate_workspace_dependency_closure(
            &release_surface,
            &packages,
            Path::new("scripts/release-crates.txt"),
            &mut errors,
        );

        assert_eq!(errors, 1);
    }

    #[test]
    fn dependency_closure_ignores_dev_only_workspace_dependency() {
        let a = package("a", &[dep("b", Some("dev"))]);
        let b = package("b", &[]);
        let packages = HashMap::from([("a", &a), ("b", &b)]);
        let release_surface = vec!["a".to_owned()];
        let mut errors = 0usize;

        validate_workspace_dependency_closure(
            &release_surface,
            &packages,
            Path::new("scripts/release-crates.txt"),
            &mut errors,
        );

        assert_eq!(errors, 0);
    }
}
