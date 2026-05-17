//! Non-Rust file policy checker.
//!
//! Walks every git-tracked file (or every file under the workspace root if
//! we are not in a git checkout), filters out Rust sources, and reports
//! anything that is not matched by an `[[allow]]` entry in
//! `policy/non-rust-allowlist.toml`.

use anyhow::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

use super::{Mode, ensure_report_dir, workspace_root};

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct AllowlistFile {
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default, rename = "allow")]
    entries: Vec<AllowEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AllowEntry {
    #[serde(default)]
    pub glob: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    pub kind: String,
    pub owner: String,
    pub surface: String,
    pub classification: String,
    pub reason: String,
    #[serde(default)]
    pub covered_by: Vec<String>,
    #[serde(default)]
    pub expires: Option<String>,
    #[serde(default)]
    pub retired: Option<bool>,
    #[serde(default)]
    pub generated_by: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct FileReport {
    pub mode: String,
    pub total_non_rust: usize,
    pub matched: usize,
    pub allowlist_size: usize,
    pub unallowlisted: Vec<String>,
    pub unused_entries: Vec<UnusedEntry>,
    pub rust_migration_candidates: Vec<RustMigrationCandidate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedEntry {
    pub glob: String,
    pub kind: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RustMigrationCandidate {
    pub path: String,
    pub kind: String,
    pub owner: String,
    pub current_language: String,
    pub suggested_home: String,
    pub reason: String,
}

pub fn run_check(mode: Mode) -> Result<()> {
    let root = workspace_root()?;
    let report_dir = ensure_report_dir(&root)?;
    let entries = load_allowlist(&root)?;

    let mut builders = Vec::new();
    for (idx, entry) in entries.iter().enumerate() {
        if let Some(glob) = entry.glob.as_deref().or(entry.path.as_deref()) {
            match Glob::new(glob) {
                Ok(g) => builders.push((idx, g)),
                Err(err) => {
                    eprintln!("warning: invalid glob `{glob}` in non-rust-allowlist.toml: {err}");
                }
            }
        }
    }
    let mut gsb = GlobSetBuilder::new();
    for (_, g) in &builders {
        gsb.add(g.clone());
    }
    let set = gsb.build()?;

    let files = enumerate_files(&root)?;
    let mut report = FileReport {
        mode: format!("{mode:?}"),
        allowlist_size: entries.len(),
        ..Default::default()
    };

    let mut used: BTreeSet<usize> = BTreeSet::new();
    for path in &files {
        if !is_non_rust_candidate(path) {
            continue;
        }
        report.total_non_rust += 1;
        let matches = set.matches(path);
        if matches.is_empty() {
            report.unallowlisted.push(path.clone());
        } else {
            report.matched += 1;
            for m in &matches {
                used.insert(*m);
            }
            if let Some(candidate) = rust_migration_candidate(path, &matches, &entries) {
                report.rust_migration_candidates.push(candidate);
            }
        }
    }

    for (idx, _g) in &builders {
        if !used.contains(idx) {
            let entry = &entries[*idx];
            if entry.retired.unwrap_or(false) {
                continue;
            }
            report.unused_entries.push(UnusedEntry {
                glob: entry
                    .glob
                    .clone()
                    .or_else(|| entry.path.clone())
                    .unwrap_or_default(),
                kind: entry.kind.clone(),
                owner: entry.owner.clone(),
            });
        }
    }

    write_reports(&report_dir, &report)?;
    print_summary(&report);

    match mode {
        Mode::Advisory => Ok(()),
        Mode::BlockingAllowlist => {
            if !report.unallowlisted.is_empty() {
                anyhow::bail!(
                    "file-policy check failed: {} unallowlisted files",
                    report.unallowlisted.len()
                );
            }
            Ok(())
        }
        Mode::BlockingStrict => {
            if !report.unallowlisted.is_empty() || !report.unused_entries.is_empty() {
                anyhow::bail!(
                    "file-policy check failed: {} unallowlisted, {} unused entries",
                    report.unallowlisted.len(),
                    report.unused_entries.len()
                );
            }
            Ok(())
        }
    }
}

fn rust_migration_candidate(
    path: &str,
    matches: &[usize],
    entries: &[AllowEntry],
) -> Option<RustMigrationCandidate> {
    let entry = most_specific_entry(matches, entries)?;
    if !is_rust_migration_surface(path, entry) {
        return None;
    }
    Some(RustMigrationCandidate {
        path: path.to_string(),
        kind: entry.kind.clone(),
        owner: entry.owner.clone(),
        current_language: current_language(path),
        suggested_home: suggested_home(path, entry),
        reason: migration_reason(path, entry),
    })
}

fn most_specific_entry<'a>(matches: &[usize], entries: &'a [AllowEntry]) -> Option<&'a AllowEntry> {
    matches
        .iter()
        .filter_map(|idx| entries.get(*idx))
        .max_by_key(|entry| {
            entry
                .glob
                .as_deref()
                .or(entry.path.as_deref())
                .map(str::len)
                .unwrap_or_default()
        })
}

fn is_rust_migration_surface(path: &str, entry: &AllowEntry) -> bool {
    if entry.classification == "generated" || entry.classification == "config" {
        return false;
    }

    if is_grammar_definition(path) {
        return entry.surface == "grammar" || entry.kind.contains("grammar");
    }

    if is_shell_like(path) {
        return entry.classification == "tooling"
            || entry.surface == "tooling"
            || entry.surface == "build"
            || entry.surface == "release";
    }

    false
}

fn is_shell_like(path: &str) -> bool {
    matches!(
        extension(path),
        Some("sh" | "py" | "js" | "ts" | "mjs" | "cjs")
    ) && !path.starts_with("book/book/")
}

fn is_grammar_definition(path: &str) -> bool {
    path.ends_with("grammar.js")
}

fn extension(path: &str) -> Option<&str> {
    Path::new(path).extension().and_then(|ext| ext.to_str())
}

fn current_language(path: &str) -> String {
    match extension(path) {
        Some("sh") => "shell".to_string(),
        Some("py") => "python".to_string(),
        Some("js" | "mjs" | "cjs") => "javascript".to_string(),
        Some("ts") => "typescript".to_string(),
        Some(other) => other.to_string(),
        None => "unknown".to_string(),
    }
}

fn suggested_home(path: &str, entry: &AllowEntry) -> String {
    if is_grammar_definition(path) {
        return "Rust grammar crate using Adze annotations".to_string();
    }
    if path.starts_with(".githooks/") {
        return "xtask lint/preflight subcommand plus thin hook shim".to_string();
    }
    if path.starts_with("scripts/") || entry.surface == "tooling" || entry.surface == "release" {
        return "xtask subcommand".to_string();
    }
    "core Rust crate or xtask command".to_string()
}

fn migration_reason(path: &str, entry: &AllowEntry) -> String {
    if is_grammar_definition(path) {
        "Tree-sitter JavaScript grammar definitions are production grammar surface area; moving them into Rust annotations keeps grammar design in the core pipeline.".to_string()
    } else if path.starts_with("scripts/") || path.starts_with(".githooks/") {
        "Repository automation is executable logic; moving it into xtask gives it Rust types, Cargo dependencies, and workspace test coverage.".to_string()
    } else {
        format!(
            "{} is a non-Rust {} surface owned by {}; consider a Rust implementation when this file changes.",
            path, entry.surface, entry.owner
        )
    }
}

fn is_non_rust_candidate(rel: &str) -> bool {
    if rel.ends_with(".rs") {
        return false;
    }
    if rel.starts_with("target/") || rel == "target" {
        return false;
    }
    if rel.starts_with(".git/") {
        return false;
    }
    true
}

fn load_allowlist(root: &Path) -> Result<Vec<AllowEntry>> {
    let path = root.join("policy").join("non-rust-allowlist.toml");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let parsed: AllowlistFile =
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    Ok(parsed.entries)
}

fn enumerate_files(root: &Path) -> Result<Vec<String>> {
    if let Some(files) = git_ls_files(root) {
        return Ok(files);
    }
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        if let Ok(rel) = entry.path().strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(out)
}

fn is_ignored_dir(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 {
        return false;
    }
    matches!(
        entry.file_name().to_string_lossy().as_ref(),
        "target" | ".git" | "node_modules"
    )
}

fn git_ls_files(root: &Path) -> Option<Vec<String>> {
    let output = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut files = Vec::new();
    for chunk in output.stdout.split(|b| *b == 0) {
        if chunk.is_empty() {
            continue;
        }
        match std::str::from_utf8(chunk) {
            Ok(s) => files.push(s.replace('\\', "/")),
            Err(_) => continue,
        }
    }
    Some(files)
}

fn write_reports(dir: &Path, report: &FileReport) -> Result<()> {
    std::fs::write(
        dir.join("file-policy.json"),
        serde_json::to_string_pretty(report)?,
    )?;
    let mut md = String::new();
    md.push_str("# File policy report\n\n");
    md.push_str(&format!("- mode: `{}`\n", report.mode));
    md.push_str(&format!(
        "- non-rust files scanned: {}\n",
        report.total_non_rust
    ));
    md.push_str(&format!("- matched: {}\n", report.matched));
    md.push_str(&format!("- allowlist entries: {}\n", report.allowlist_size));
    md.push_str(&format!(
        "- unallowlisted: {}\n",
        report.unallowlisted.len()
    ));
    md.push_str(&format!(
        "- unused allowlist entries: {}\n",
        report.unused_entries.len()
    ));
    md.push_str(&format!(
        "- Rust migration candidates: {}\n",
        report.rust_migration_candidates.len()
    ));

    if !report.unallowlisted.is_empty() {
        md.push_str("\n## Unallowlisted (top 100)\n\n");
        for p in report.unallowlisted.iter().take(100) {
            md.push_str(&format!("- `{p}`\n"));
        }
    }
    if !report.rust_migration_candidates.is_empty() {
        md.push_str("\n## Rust migration candidates (top 100)\n\n");
        md.push_str("| path | current | suggested home | owner | why |\n|---|---|---|---|---|\n");
        for candidate in report.rust_migration_candidates.iter().take(100) {
            md.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                candidate.path,
                candidate.current_language,
                candidate.suggested_home,
                candidate.owner,
                candidate.reason
            ));
        }
    }
    if !report.unused_entries.is_empty() {
        md.push_str("\n## Unused allowlist entries\n\n");
        md.push_str("| glob | kind | owner |\n|---|---|---|\n");
        for e in &report.unused_entries {
            md.push_str(&format!("| `{}` | {} | {} |\n", e.glob, e.kind, e.owner));
        }
    }

    std::fs::write(dir.join("file-policy.md"), md)?;
    Ok(())
}

fn print_summary(report: &FileReport) {
    println!("file-policy check ({})", report.mode);
    println!("  non-rust scanned: {}", report.total_non_rust);
    println!("  matched:          {}", report.matched);
    println!("  unallowlisted:    {}", report.unallowlisted.len());
    println!("  unused entries:   {}", report.unused_entries.len());
    println!(
        "  Rust migrations:  {}",
        report.rust_migration_candidates.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: &str, surface: &str, classification: &str) -> AllowEntry {
        AllowEntry {
            glob: Some("scripts/**".to_string()),
            path: None,
            kind: kind.to_string(),
            owner: "core/build".to_string(),
            surface: surface.to_string(),
            classification: classification.to_string(),
            reason: "test entry".to_string(),
            covered_by: Vec::new(),
            expires: None,
            retired: None,
            generated_by: None,
        }
    }

    #[test]
    fn rust_migration_candidate_accepts_tooling_scripts() {
        let entries = vec![entry("shell_tooling", "tooling", "tooling")];
        let candidate = rust_migration_candidate("scripts/check.sh", &[0], &entries)
            .expect("tooling script should be a migration candidate");

        assert_eq!(candidate.current_language, "shell");
        assert_eq!(candidate.suggested_home, "xtask subcommand");
    }

    #[test]
    fn rust_migration_candidate_skips_language_fixtures() {
        let entries = vec![entry("language_fixture", "fixtures", "test")];
        let candidate = rust_migration_candidate("fixtures/example.py", &[0], &entries);

        assert!(candidate.is_none());
    }

    #[test]
    fn rust_migration_candidate_routes_grammar_js_to_rust_grammar() {
        let entries = vec![entry("grammar_input", "grammar", "production")];
        let candidate = rust_migration_candidate("grammars/example/grammar.js", &[0], &entries)
            .expect("production grammar.js should be a migration candidate");

        assert_eq!(candidate.current_language, "javascript");
        assert_eq!(
            candidate.suggested_home,
            "Rust grammar crate using Adze annotations"
        );
    }
}
