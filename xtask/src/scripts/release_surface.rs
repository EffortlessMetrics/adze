use super::{
    metadata::{
        CargoMetadata, DependencySelection, cargo_metadata, dependency_map, publishable_packages,
        topological_sort,
    },
    support::{env_truthy, repo_root},
};
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ReleaseSurfaceMode {
    Fixed,
    Auto,
}

impl ReleaseSurfaceMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::Auto => "auto",
        }
    }
}

struct ReleaseSurfaceConfig {
    mode: ReleaseSurfaceMode,
    crate_file: PathBuf,
}

pub(crate) struct ValidationOutcome {
    pub(crate) errors: Vec<String>,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn run_release_surface(
    mode: Option<ReleaseSurfaceMode>,
    crate_file: Option<PathBuf>,
    sync: bool,
) -> Result<()> {
    let repo_root = repo_root()?;
    let config = release_surface_config(&repo_root, mode, crate_file)?;
    let sync = sync || env_truthy("RELEASE_CRATE_SYNC");
    let crates = resolve_release_surface(&repo_root, &config)?;

    if sync {
        if config.mode != ReleaseSurfaceMode::Auto {
            eprintln!("::warning::RELEASE_CRATE_SYNC is only used in auto mode; ignoring.");
        } else {
            sync_release_surface_file(&config.crate_file, &crates)?;
        }
    }

    for crate_name in crates {
        println!("{crate_name}");
    }
    Ok(())
}

pub(crate) fn run_validate_release_surface(
    mode: Option<ReleaseSurfaceMode>,
    crate_file: Option<PathBuf>,
    strict: bool,
) -> Result<()> {
    let repo_root = repo_root()?;
    let config = release_surface_config(&repo_root, mode, crate_file)?;
    let strict = strict || env_truthy("STRICT_PUBLISH_SURFACE");
    let metadata = cargo_metadata(true)?;
    let allowed_crates = resolve_release_surface(&repo_root, &config)?;

    if allowed_crates.is_empty() {
        bail!(
            "::error::Release surface is empty (mode: {}).",
            config.mode.as_str()
        );
    }

    let outcome = validate_release_surface(&metadata, &config, &allowed_crates, strict);
    for warning in &outcome.warnings {
        eprintln!("::warning::{warning}");
    }
    for error in &outcome.errors {
        eprintln!("::error::{error}");
    }
    if !outcome.errors.is_empty() {
        eprintln!("::error::Publish-surface validation failed.");
        bail!("publish-surface validation failed");
    }

    println!(
        "Publish-surface validation passed for mode={}: {}",
        config.mode.as_str(),
        allowed_crates.join(" ")
    );
    Ok(())
}

fn validate_release_surface(
    metadata: &CargoMetadata,
    config: &ReleaseSurfaceConfig,
    allowed_crates: &[String],
    strict: bool,
) -> ValidationOutcome {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut allowlist = HashSet::new();
    let mut allowlist_index = HashMap::new();

    for (index, crate_name) in allowed_crates.iter().enumerate() {
        if !allowlist.insert(crate_name.clone()) {
            errors.push(format!("Duplicate crate in allowlist: {crate_name}"));
            continue;
        }
        allowlist_index.insert(crate_name.clone(), index);
    }

    let mut seen_allowed = HashSet::new();
    let mut extra_publishable = Vec::new();
    let publishable_packages = publishable_packages(metadata);

    for package in &metadata.packages {
        let is_publishable = publishable_packages.contains_key(package.name.as_str());
        if !is_publishable {
            if config.mode == ReleaseSurfaceMode::Fixed && allowlist.contains(&package.name) {
                errors.push(format!(
                    "Allowlisted crate '{}' is not publishable (publish = false).",
                    package.name
                ));
            }
            continue;
        }

        if !allowlist.contains(&package.name) {
            if config.mode == ReleaseSurfaceMode::Fixed && strict {
                errors.push(format!(
                    "Unexpected publishable crate: {} ({})",
                    package.name, package.manifest_path
                ));
            } else if config.mode == ReleaseSurfaceMode::Fixed {
                extra_publishable.push(package.name.clone());
            }
            continue;
        }

        seen_allowed.insert(package.name.clone());
    }

    if config.mode == ReleaseSurfaceMode::Fixed {
        let package_names = allowed_crates
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let dependency_map = dependency_map(
            &publishable_packages,
            &package_names,
            DependencySelection::AllPublishable,
        );
        for (crate_name, deps) in dependency_map {
            for dep in deps {
                if !allowlist_index.contains_key(&dep) {
                    continue;
                }
                let crate_idx = allowlist_index[&crate_name];
                let dep_idx = allowlist_index[&dep];
                if dep_idx >= crate_idx {
                    errors.push(format!(
                        "Allowlist order violation: '{}' must be published before '{}'",
                        dep, crate_name
                    ));
                }
            }
        }
    }

    for crate_name in allowed_crates {
        if !seen_allowed.contains(crate_name) {
            errors.push(format!(
                "Allowlisted crate '{}' is not marked as publishable.",
                crate_name
            ));
        }
    }

    if config.mode == ReleaseSurfaceMode::Fixed
        && errors.is_empty()
        && !extra_publishable.is_empty()
    {
        if extra_publishable.len() <= 12 {
            warnings.push(format!(
                "Extra publishable crates are not in {}: {}",
                config.crate_file.display(),
                extra_publishable.join(" ")
            ));
        } else {
            let shown = extra_publishable[..12].join(" ");
            warnings.push(format!(
                "Extra publishable crates are not in {}: {}",
                config.crate_file.display(),
                shown
            ));
            warnings.push(format!(
                "... and {} more publishable crates",
                extra_publishable.len() - 12
            ));
        }
    }

    ValidationOutcome { errors, warnings }
}

pub(crate) fn validate_fixed_release_surface(
    metadata: &CargoMetadata,
    crate_file: PathBuf,
    allowed_crates: &[String],
    strict: bool,
) -> ValidationOutcome {
    validate_release_surface(
        metadata,
        &ReleaseSurfaceConfig {
            mode: ReleaseSurfaceMode::Fixed,
            crate_file,
        },
        allowed_crates,
        strict,
    )
}

fn release_surface_config(
    repo_root: &Path,
    mode: Option<ReleaseSurfaceMode>,
    crate_file: Option<PathBuf>,
) -> Result<ReleaseSurfaceConfig> {
    let mode = match mode {
        Some(mode) => mode,
        None => match env::var("RELEASE_SURFACE_MODE") {
            Ok(value) if !value.trim().is_empty() => parse_release_surface_mode(&value)?,
            _ => ReleaseSurfaceMode::Fixed,
        },
    };
    let crate_file = crate_file.unwrap_or_else(|| {
        env::var_os("RELEASE_CRATE_FILE")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| repo_root.join("scripts/release-crates.txt"))
    });
    Ok(ReleaseSurfaceConfig { mode, crate_file })
}

fn parse_release_surface_mode(raw: &str) -> Result<ReleaseSurfaceMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "fixed" => Ok(ReleaseSurfaceMode::Fixed),
        "auto" => Ok(ReleaseSurfaceMode::Auto),
        other => bail!(
            "::error::Invalid RELEASE_SURFACE_MODE '{}'. Expected fixed|auto.",
            other
        ),
    }
}

fn resolve_release_surface(repo_root: &Path, config: &ReleaseSurfaceConfig) -> Result<Vec<String>> {
    match config.mode {
        ReleaseSurfaceMode::Fixed => load_release_surface_file(&config.crate_file),
        ReleaseSurfaceMode::Auto => {
            let metadata = cargo_metadata(true)?;
            let publishable = publishable_packages(&metadata);
            if publishable.is_empty() {
                bail!("::error::No publishable crates found in workspace metadata.");
            }
            let mut ordered_names = publishable
                .keys()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>();
            ordered_names.sort();
            let package_names = ordered_names.iter().map(String::as_str).collect::<Vec<_>>();
            let dependency_map = dependency_map(
                &publishable,
                &package_names,
                DependencySelection::AllPublishable,
            );
            topological_sort(&ordered_names, &dependency_map).with_context(|| {
                format!(
                    "::error::Could not order publishable crates in a dependency-safe way for {}",
                    repo_root.display()
                )
            })
        }
    }
}

fn sync_release_surface_file(path: &Path, crates: &[String]) -> Result<()> {
    let mut file =
        fs::File::create(path).with_context(|| format!("failed to create {}", path.display()))?;
    writeln!(
        file,
        "# Auto-generated publish order from workspace metadata (all publishable crates)."
    )?;
    for crate_name in crates {
        writeln!(file, "{crate_name}")?;
    }
    Ok(())
}

pub(crate) fn load_release_surface_file(path: &Path) -> Result<Vec<String>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("::error::Missing allowlist file: {}", path.display()))?;
    parse_release_surface_file(&contents, path)
}

fn parse_release_surface_file(contents: &str, path: &Path) -> Result<Vec<String>> {
    let mut crates = Vec::new();
    let mut seen = HashSet::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let crate_name = trimmed
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_owned();
        if crate_name.is_empty() {
            continue;
        }
        if !seen.insert(crate_name.clone()) {
            bail!("::error::Duplicate crate in allowlist: {crate_name}");
        }
        crates.push(crate_name);
    }
    if crates.is_empty() {
        bail!("::error::Allowlist is empty: {}", path.display());
    }
    Ok(crates)
}

#[cfg(test)]
mod tests {
    use super::{
        ReleaseSurfaceConfig, ReleaseSurfaceMode, load_release_surface_file,
        parse_release_surface_file, parse_release_surface_mode, validate_release_surface,
    };
    use crate::scripts::metadata::{
        CargoMetadata, MetadataDependency, MetadataPackage, PublishSetting,
    };
    use std::{collections::HashMap, path::PathBuf};

    fn package(name: &str, publish: Option<PublishSetting>, deps: &[&str]) -> MetadataPackage {
        MetadataPackage {
            id: format!("{name} 0.1.0 (path+file:///tmp/{name})"),
            name: name.to_owned(),
            version: "0.1.0".to_owned(),
            description: Some(format!("{name} crate")),
            license: Some("MIT".to_owned()),
            repository: Some("https://example.com/adze".to_owned()),
            homepage: None,
            edition: "2024".to_owned(),
            manifest_path: format!("/tmp/{name}/Cargo.toml"),
            publish,
            readme: None,
            dependencies: deps
                .iter()
                .map(|dep| MetadataDependency {
                    name: (*dep).to_owned(),
                    kind: None,
                    optional: false,
                    path: Some(format!("/tmp/{dep}")),
                    req: "0.1.0".to_owned(),
                })
                .collect(),
            features: HashMap::new(),
        }
    }

    fn metadata(packages: Vec<MetadataPackage>) -> CargoMetadata {
        let workspace_members = packages.iter().map(|package| package.id.clone()).collect();
        CargoMetadata {
            packages,
            workspace_members,
        }
    }

    #[test]
    fn parse_release_surface_rejects_duplicates() {
        let err =
            parse_release_surface_file("adze\nadze\n", std::path::Path::new("release-crates.txt"))
                .expect_err("duplicate entries must fail");
        assert!(err.to_string().contains("Duplicate crate"));
    }

    #[test]
    fn parse_release_surface_mode_rejects_invalid_values() {
        let err = parse_release_surface_mode("weird").expect_err("invalid mode must fail");
        assert!(err.to_string().contains("Invalid RELEASE_SURFACE_MODE"));
    }

    #[test]
    fn load_release_surface_reports_missing_file() {
        let temp = tempfile::tempdir().expect("tempdir should exist");
        let err = load_release_surface_file(&temp.path().join("missing.txt"))
            .expect_err("missing file must fail");
        assert!(err.to_string().contains("Missing allowlist file"));
    }

    #[test]
    fn validate_release_surface_reports_order_violation() {
        let config = ReleaseSurfaceConfig {
            mode: ReleaseSurfaceMode::Fixed,
            crate_file: PathBuf::from("scripts/release-crates.txt"),
        };
        let metadata = metadata(vec![package("a", None, &[]), package("b", None, &["a"])]);
        let allowed = vec!["b".to_owned(), "a".to_owned()];

        let outcome = validate_release_surface(&metadata, &config, &allowed, false);

        assert!(
            outcome
                .errors
                .iter()
                .any(|error| error.contains("must be published before"))
        );
    }

    #[test]
    fn validate_release_surface_strict_mode_flags_extra_publishable_crates() {
        let config = ReleaseSurfaceConfig {
            mode: ReleaseSurfaceMode::Fixed,
            crate_file: PathBuf::from("scripts/release-crates.txt"),
        };
        let metadata = metadata(vec![package("a", None, &[]), package("b", None, &[])]);
        let allowed = vec!["a".to_owned()];

        let outcome = validate_release_surface(&metadata, &config, &allowed, true);

        assert!(
            outcome
                .errors
                .iter()
                .any(|error| error.contains("Unexpected publishable crate: b"))
        );
    }

    #[test]
    fn validate_release_surface_flags_non_publishable_allowlist_entries() {
        let config = ReleaseSurfaceConfig {
            mode: ReleaseSurfaceMode::Fixed,
            crate_file: PathBuf::from("scripts/release-crates.txt"),
        };
        let metadata = metadata(vec![package("a", Some(PublishSetting::Bool(false)), &[])]);
        let allowed = vec!["a".to_owned()];

        let outcome = validate_release_surface(&metadata, &config, &allowed, false);

        assert!(
            outcome
                .errors
                .iter()
                .any(|error| error.contains("not publishable"))
        );
    }
}
