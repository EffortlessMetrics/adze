use super::{
    metadata::{
        DependencySelection, MetadataPackage, cargo_metadata, dependency_map, publishable_packages,
        topological_sort,
    },
    support::is_missing,
};
use anyhow::{Context, Result, bail};
use std::process::Command;

pub(crate) fn run_publish_order(dry_run: bool, validate_only: bool) -> Result<()> {
    let metadata = cargo_metadata(true)?;
    let publishable = publishable_packages(&metadata);
    if publishable.is_empty() {
        bail!("error: no publishable crates found in workspace.");
    }

    let publishable_names = metadata
        .packages
        .iter()
        .filter(|package| publishable.contains_key(package.name.as_str()))
        .map(|package| package.name.clone())
        .collect::<Vec<_>>();
    let package_names = publishable_names
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let dependency_map = dependency_map(
        &publishable,
        &package_names,
        DependencySelection::WorkspacePathOnly,
    );
    let blockers = metadata_blockers(
        publishable_names
            .iter()
            .filter_map(|name| publishable.get(name.as_str()).copied()),
    );

    if validate_only {
        println!(
            "=== Metadata validation for {} publishable crates ===",
            publishable_names.len()
        );
        println!();
        if blockers.is_empty() {
            println!(
                "OK: all {} crates have required metadata.",
                publishable_names.len()
            );
            return Ok(());
        }

        println!("BLOCKING ISSUES:");
        for blocker in &blockers {
            println!("  x {blocker}");
        }
        bail!("publish-order metadata validation failed");
    }

    let ordered = topological_sort(&publishable_names, &dependency_map)
        .context("error: circular dependency detected")?;

    println!("=== crates.io publish order ({} crates) ===", ordered.len());
    println!();
    for (index, crate_name) in ordered.iter().enumerate() {
        let package = publishable
            .get(crate_name.as_str())
            .with_context(|| format!("missing publishable package metadata for {crate_name}"))?;
        let deps = dependency_map
            .get(crate_name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        let dep_label = if deps.is_empty() {
            "no workspace deps".to_owned()
        } else {
            format!("depends on: {}", deps.join(", "))
        };
        println!(
            "{:>2}. {} v{} ({})",
            index + 1,
            crate_name,
            package.version,
            dep_label
        );
    }

    if !blockers.is_empty() {
        println!();
        println!("BLOCKING ISSUES:");
        for blocker in &blockers {
            println!("  x {blocker}");
        }
    }

    if dry_run {
        println!();
        println!("=== Running cargo publish --dry-run ===");
        println!();
        let mut failures = 0;
        for crate_name in &ordered {
            println!(">>> [{crate_name}] cargo publish --dry-run ...");
            let status = Command::new("cargo")
                .args(["publish", "-p", crate_name, "--dry-run"])
                .status()
                .with_context(|| {
                    format!("failed to run cargo publish --dry-run for {crate_name}")
                })?;
            if status.success() {
                println!("  OK: {crate_name}");
            } else {
                println!("  FAIL: {crate_name}");
                failures += 1;
            }
            println!();
        }
        if failures > 0 {
            bail!("FAIL: {failures} crate(s) failed dry-run publish.");
        }
        println!("=== All dry-run publishes succeeded ===");
    }

    Ok(())
}

fn metadata_blockers<'a>(packages: impl IntoIterator<Item = &'a MetadataPackage>) -> Vec<String> {
    packages
        .into_iter()
        .filter_map(|package| {
            let mut issues = Vec::new();
            if is_missing(package.description.as_deref()) {
                issues.push("missing description");
            }
            if is_missing(package.license.as_deref()) {
                issues.push("missing license");
            }
            if is_missing(package.repository.as_deref()) {
                issues.push("missing repository");
            }
            if issues.is_empty() {
                None
            } else {
                Some(format!("{}: {}", package.name, issues.join(", ")))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::metadata_blockers;
    use crate::scripts::metadata::MetadataPackage;
    use std::collections::HashMap;

    fn package(
        name: &str,
        description: Option<&str>,
        license: Option<&str>,
        repository: Option<&str>,
    ) -> MetadataPackage {
        MetadataPackage {
            id: format!("{name} 0.1.0 (path+file:///tmp/{name})"),
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
            description: description.map(ToOwned::to_owned),
            license: license.map(ToOwned::to_owned),
            repository: repository.map(ToOwned::to_owned),
            homepage: None,
            edition: "2024".to_owned(),
            manifest_path: format!("/tmp/{name}/Cargo.toml"),
            publish: None,
            readme: None,
            dependencies: Vec::new(),
            features: HashMap::new(),
        }
    }

    #[test]
    fn metadata_blockers_capture_missing_fields() {
        let blockers = metadata_blockers([
            &package(
                "good",
                Some("desc"),
                Some("MIT"),
                Some("https://example.com"),
            ),
            &package("missing", None, Some("MIT"), None),
        ]);

        assert_eq!(
            blockers,
            vec!["missing: missing description, missing repository"]
        );
    }
}
