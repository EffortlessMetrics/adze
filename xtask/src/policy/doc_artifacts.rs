//! Document artifact ledger checker.
//!
//! Validates `policy/doc-artifacts.toml`:
//! - parses correctly
//! - every listed artifact path exists
//! - artifact IDs are unique
//! - linked artifact IDs and paths resolve
//! - kind values are valid
//! - status values are valid
//! - required header fields exist

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::{SimpleCheckMode, workspace_root};

const POLICY_PATH: &str = "policy/doc-artifacts.toml";

const VALID_KINDS: &[&str] = &["proposal", "spec", "adr", "plan", "goal", "handoff"];
const VALID_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "implemented",
    "active",
    "complete",
    "superseded",
    "paused",
];

#[derive(Debug, Default, Deserialize)]
struct DocArtifactsFile {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    policy: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    status: String,
    #[serde(default, rename = "artifact")]
    artifacts: Vec<ArtifactEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactEntry {
    id: String,
    kind: String,
    path: String,
    status: String,
    owner: String,
    #[serde(default)]
    milestone: String,
    #[serde(default)]
    source_of_truth_for: Vec<String>,
    #[serde(default)]
    links_to: Vec<String>,
}

pub fn run(mode: &str) -> Result<()> {
    let mode = SimpleCheckMode::parse(mode)?;
    let root = workspace_root()?;
    let policy_path = root.join(POLICY_PATH);

    let raw = std::fs::read_to_string(&policy_path)
        .with_context(|| format!("reading {}", policy_path.display()))?;

    let file: DocArtifactsFile =
        toml::from_str(&raw).with_context(|| format!("parsing {}", POLICY_PATH))?;

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Header fields
    if file.schema_version.is_empty() {
        errors.push("missing schema_version".into());
    }
    if file.policy.is_empty() {
        errors.push("missing policy".into());
    }
    if file.status.is_empty() {
        errors.push("missing status".into());
    }

    // ID uniqueness
    let mut seen_ids: BTreeSet<String> = BTreeSet::new();
    for art in &file.artifacts {
        if !seen_ids.insert(art.id.clone()) {
            errors.push(format!("duplicate artifact ID: {}", art.id));
        }
    }

    // Build ID set for link resolution
    let all_ids: BTreeSet<&str> = file.artifacts.iter().map(|a| a.id.as_str()).collect();

    // Per-artifact checks
    for art in &file.artifacts {
        // Required fields
        if art.id.is_empty() {
            errors.push("artifact with empty id".into());
        }
        if art.kind.is_empty() {
            errors.push(format!("artifact {} missing kind", art.id));
        }
        if art.path.is_empty() {
            errors.push(format!("artifact {} missing path", art.id));
        }
        if art.status.is_empty() {
            errors.push(format!("artifact {} missing status", art.id));
        }
        if art.owner.is_empty() {
            errors.push(format!("artifact {} missing owner", art.id));
        }

        // Kind validation
        if !VALID_KINDS.contains(&art.kind.as_str()) {
            errors.push(format!(
                "artifact {} has invalid kind '{}'",
                art.id, art.kind
            ));
        }

        // Status validation
        if !VALID_STATUSES.contains(&art.status.as_str()) {
            errors.push(format!(
                "artifact {} has invalid status '{}'",
                art.id, art.status
            ));
        }

        // Path existence
        let artifact_path = root.join(&art.path);
        if !artifact_path.exists() {
            errors.push(format!(
                "artifact {} path does not exist: {}",
                art.id, art.path
            ));
        }

        // Kind-path consistency
        if !art.path.is_empty() && !art.kind.is_empty() {
            let path_matches_kind = match art.kind.as_str() {
                "proposal" => art.path.contains("proposals/"),
                "spec" => art.path.contains("specs/"),
                "adr" => art.path.contains("adr/"),
                "plan" => art.path.contains("plans/"),
                "goal" => art.path.contains(".adze/goals/"),
                "handoff" => art.path.contains("handoffs/"),
                _ => true,
            };
            if !path_matches_kind {
                warnings.push(format!(
                    "artifact {} kind '{}' may not match path '{}'",
                    art.id, art.kind, art.path
                ));
            }
        }

        // Link resolution
        for link in &art.links_to {
            if !all_ids.contains(link.as_str()) {
                // Links can also be paths or external refs (not artifact IDs)
                let link_path = root.join(link);
                if !link_path.exists()
                    && !link.starts_with("docs/")
                    && !link.starts_with("policy/")
                    && !link.starts_with("scripts/")
                {
                    warnings.push(format!(
                        "artifact {} links_to '{}' which is not a known artifact ID or existing path",
                        art.id, link
                    ));
                }
            }
        }
    }

    // Report
    println!(
        "doc-artifacts: {} artifacts registered",
        file.artifacts.len()
    );

    for w in &warnings {
        eprintln!("  warning: {w}");
    }
    for e in &errors {
        eprintln!("  error: {e}");
    }

    if errors.is_empty() {
        println!(
            "doc-artifacts: all checks passed ({} warnings)",
            warnings.len()
        );
        Ok(())
    } else if mode == SimpleCheckMode::Advisory {
        eprintln!(
            "doc-artifacts: advisory mode reported {} errors in {}",
            errors.len(),
            POLICY_PATH
        );
        Ok(())
    } else {
        anyhow::bail!(
            "doc-artifacts: {} errors found in {}",
            errors.len(),
            POLICY_PATH
        )
    }
}
