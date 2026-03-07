use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    process::Command,
};

#[derive(Deserialize)]
pub(crate) struct CargoMetadata {
    #[serde(default)]
    pub(crate) packages: Vec<MetadataPackage>,
    #[serde(default)]
    pub(crate) workspace_members: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub(crate) struct MetadataPackage {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) license: Option<String>,
    #[serde(default)]
    pub(crate) repository: Option<String>,
    #[serde(default)]
    pub(crate) homepage: Option<String>,
    pub(crate) edition: String,
    pub(crate) manifest_path: String,
    #[serde(default)]
    pub(crate) publish: Option<PublishSetting>,
    #[serde(default)]
    pub(crate) readme: Option<String>,
    #[serde(default)]
    pub(crate) dependencies: Vec<MetadataDependency>,
    #[serde(default)]
    pub(crate) features: HashMap<String, Vec<String>>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum PublishSetting {
    Bool(bool),
    Registries(Vec<String>),
}

#[derive(Clone, Deserialize)]
pub(crate) struct MetadataDependency {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) kind: Option<String>,
    #[serde(default)]
    pub(crate) optional: bool,
    #[serde(default)]
    pub(crate) path: Option<String>,
    #[serde(default)]
    pub(crate) req: String,
}

pub(crate) fn cargo_metadata(no_deps: bool) -> Result<CargoMetadata> {
    let mut args = vec![
        "metadata".to_owned(),
        "--format-version".to_owned(),
        "1".to_owned(),
    ];
    if no_deps {
        args.insert(1, "--no-deps".to_owned());
    }
    let output = Command::new("cargo")
        .args(args)
        .output()
        .context("failed to run cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout).context("failed to parse cargo metadata JSON")
}

pub(crate) fn publishable_packages(metadata: &CargoMetadata) -> HashMap<&str, &MetadataPackage> {
    metadata
        .packages
        .iter()
        .filter(|package| is_publishable(package))
        .map(|package| (package.name.as_str(), package))
        .collect()
}

pub(crate) fn workspace_packages(metadata: &CargoMetadata) -> Vec<MetadataPackage> {
    let workspace_members = metadata
        .workspace_members
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    metadata
        .packages
        .iter()
        .filter(|package| workspace_members.contains(&package.id))
        .cloned()
        .collect()
}

pub(crate) fn packages_with_feature<'a>(
    packages: &'a [MetadataPackage],
    feature: &str,
) -> HashSet<&'a str> {
    packages
        .iter()
        .filter(|package| package.features.contains_key(feature))
        .map(|package| package.name.as_str())
        .collect()
}

pub(crate) enum DependencySelection {
    AllPublishable,
    WorkspacePathOnly,
}

pub(crate) fn dependency_map(
    publishable: &HashMap<&str, &MetadataPackage>,
    package_names: &[&str],
    selection: DependencySelection,
) -> HashMap<String, BTreeSet<String>> {
    let mut map = HashMap::new();
    for crate_name in package_names {
        let package = match publishable.get(crate_name) {
            Some(package) => *package,
            None => continue,
        };
        let deps = package
            .dependencies
            .iter()
            .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
            .filter(|dependency| !dependency.optional)
            .filter(|dependency| match selection {
                DependencySelection::AllPublishable => true,
                DependencySelection::WorkspacePathOnly => dependency.path.is_some(),
            })
            .filter(|dependency| publishable.contains_key(dependency.name.as_str()))
            .filter(|dependency| dependency.name != package.name)
            .map(|dependency| dependency.name.clone())
            .collect::<BTreeSet<_>>();
        map.insert((*crate_name).to_owned(), deps);
    }
    map
}

pub(crate) fn topological_sort(
    package_names: &[String],
    dependency_map: &HashMap<String, BTreeSet<String>>,
) -> Result<Vec<String>> {
    let mut indegree = package_names
        .iter()
        .map(|name| (name.clone(), 0usize))
        .collect::<HashMap<_, _>>();
    let mut dependents = HashMap::<String, Vec<String>>::new();

    for (crate_name, deps) in dependency_map {
        for dep in deps {
            if !indegree.contains_key(dep) || crate_name == dep {
                continue;
            }
            *indegree.get_mut(crate_name).expect("crate must exist") += 1;
            dependents
                .entry(dep.clone())
                .or_default()
                .push(crate_name.clone());
        }
    }

    let mut ordered = Vec::with_capacity(package_names.len());
    let mut seen = HashSet::new();

    loop {
        let mut progress = false;
        for crate_name in package_names {
            if seen.contains(crate_name) {
                continue;
            }
            if indegree.get(crate_name).copied().unwrap_or_default() != 0 {
                continue;
            }
            seen.insert(crate_name.clone());
            ordered.push(crate_name.clone());
            progress = true;
            if let Some(dependents_for_crate) = dependents.get(crate_name) {
                for dependent in dependents_for_crate {
                    if let Some(value) = indegree.get_mut(dependent) {
                        *value = value.saturating_sub(1);
                    }
                }
            }
        }
        if !progress {
            break;
        }
    }

    if ordered.len() != package_names.len() {
        let unresolved = package_names
            .iter()
            .filter(|crate_name| !seen.contains(*crate_name))
            .cloned()
            .collect::<Vec<_>>();
        bail!("Unordered crates: {}", unresolved.join(" "));
    }

    Ok(ordered)
}

fn is_publishable(package: &MetadataPackage) -> bool {
    match package.publish.as_ref() {
        None => true,
        Some(PublishSetting::Bool(true)) => true,
        Some(PublishSetting::Bool(false)) => false,
        Some(PublishSetting::Registries(registries)) => {
            registries.iter().any(|registry| registry == "crates.io")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::topological_sort;
    use std::collections::{BTreeSet, HashMap};

    #[test]
    fn topological_sort_respects_dependencies() {
        let names = vec!["b".to_owned(), "a".to_owned(), "c".to_owned()];
        let mut deps = HashMap::new();
        deps.insert("b".to_owned(), BTreeSet::from(["a".to_owned()]));
        deps.insert("a".to_owned(), BTreeSet::new());
        deps.insert(
            "c".to_owned(),
            BTreeSet::from(["a".to_owned(), "b".to_owned()]),
        );
        let ordered = topological_sort(&names, &deps).expect("topo sort should succeed");
        assert_eq!(ordered, vec!["a", "b", "c"]);
    }

    #[test]
    fn topological_sort_rejects_cycles() {
        let names = vec!["a".to_owned(), "b".to_owned()];
        let mut deps = HashMap::new();
        deps.insert("a".to_owned(), BTreeSet::from(["b".to_owned()]));
        deps.insert("b".to_owned(), BTreeSet::from(["a".to_owned()]));

        let err = topological_sort(&names, &deps).expect_err("cycle must fail");
        assert!(err.to_string().contains("Unordered crates"));
    }
}
