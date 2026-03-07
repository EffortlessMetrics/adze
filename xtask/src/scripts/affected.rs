use super::{
    metadata::{CargoMetadata, cargo_metadata},
    support::{normalize_existing_path, repo_root, split_nul_output},
};
use anyhow::{Context, Result};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) fn run_affected_crates() -> Result<()> {
    let repo_root = repo_root()?;
    let changed = staged_changed_paths(&repo_root)?;
    if changed.is_empty() {
        return Ok(());
    }

    let metadata = cargo_metadata(true)?;
    let package_dirs = package_dirs(&metadata)?;
    let touched = affected_packages_for_paths(&repo_root, &package_dirs, &changed)?;

    for package_name in touched {
        println!("{package_name}");
    }
    Ok(())
}

fn staged_changed_paths(repo_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .arg("diff")
        .arg("--cached")
        .arg("--name-only")
        .arg("-z")
        .arg("--diff-filter=ACMR")
        .arg("--")
        .args([
            "*.rs",
            "build.rs",
            "*/build.rs",
            "Cargo.toml",
            "*/Cargo.toml",
        ])
        .output()
        .context("failed to query staged files")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    Ok(split_nul_output(&output.stdout))
}

fn package_dirs(metadata: &CargoMetadata) -> Result<Vec<(String, PathBuf)>> {
    metadata
        .packages
        .iter()
        .map(|package| {
            let manifest = PathBuf::from(&package.manifest_path);
            let dir = manifest
                .parent()
                .map(Path::to_path_buf)
                .with_context(|| format!("missing manifest parent for {}", package.name))?;
            Ok((package.name.clone(), normalize_existing_path(&dir)?))
        })
        .collect()
}

fn affected_packages_for_paths(
    repo_root: &Path,
    package_dirs: &[(String, PathBuf)],
    changed: &[String],
) -> Result<BTreeSet<String>> {
    let mut touched = BTreeSet::new();
    for file in changed {
        let candidate = repo_root.join(file);
        if !candidate.exists() {
            continue;
        }
        let abs = normalize_existing_path(&candidate)?;
        let mut best_match = None::<(&str, usize)>;
        for (package_name, dir) in package_dirs {
            if abs.starts_with(dir) {
                let len = dir.as_os_str().len();
                match best_match {
                    Some((_, best_len)) if best_len >= len => {}
                    _ => best_match = Some((package_name.as_str(), len)),
                }
            }
        }
        if let Some((package_name, _)) = best_match {
            touched.insert(package_name.to_owned());
        }
    }
    Ok(touched)
}

#[cfg(test)]
mod tests {
    use super::affected_packages_for_paths;
    use crate::scripts::support::normalize_existing_path;
    use std::{collections::BTreeSet, fs};

    #[test]
    fn affected_packages_match_existing_changed_files() {
        let temp = tempfile::tempdir().expect("tempdir should exist");
        let foo_dir = temp.path().join("crates/foo");
        let bar_dir = temp.path().join("crates/bar");
        fs::create_dir_all(foo_dir.join("src")).expect("foo dir should exist");
        fs::create_dir_all(bar_dir.join("src")).expect("bar dir should exist");
        fs::write(foo_dir.join("src/lib.rs"), "pub fn foo() {}\n").expect("write should succeed");
        fs::write(bar_dir.join("Cargo.toml"), "[package]\nname = \"bar\"\n")
            .expect("write should succeed");

        let package_dirs = vec![
            (
                "foo".to_owned(),
                normalize_existing_path(&foo_dir).expect("canonical foo path"),
            ),
            (
                "bar".to_owned(),
                normalize_existing_path(&bar_dir).expect("canonical bar path"),
            ),
        ];
        let changed = vec![
            "crates/foo/src/lib.rs".to_owned(),
            "crates/bar/Cargo.toml".to_owned(),
            "deleted.rs".to_owned(),
        ];

        let affected = affected_packages_for_paths(temp.path(), &package_dirs, &changed)
            .expect("matching should succeed");

        assert_eq!(
            affected,
            BTreeSet::from(["bar".to_owned(), "foo".to_owned()])
        );
    }
}
