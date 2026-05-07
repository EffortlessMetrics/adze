//! Aggregate policy report. Runs every check (advisory) and emits a single
//! Markdown summary at `target/policy/policy-report.md`.

use anyhow::Result;
use std::path::Path;

use super::{ensure_report_dir, workspace_root};

pub fn run() -> Result<()> {
    let root = workspace_root()?;
    let dir = ensure_report_dir(&root)?;

    let _ = super::no_panic::run_check(super::Mode::Advisory);
    let _ = super::file_policy::run_check(super::Mode::Advisory);
    let _ = super::lint_policy::run_check(super::Mode::Advisory);

    let mut md = String::new();
    md.push_str("# Policy report\n\n");
    md.push_str("This report aggregates the three policy checks. Each subsection\n");
    md.push_str("links to its dedicated artefact.\n\n");
    append_section(&mut md, "No-panic", &dir.join("no-panic.md"))?;
    append_section(&mut md, "File policy", &dir.join("file-policy.md"))?;
    append_section(&mut md, "Lint policy", &dir.join("lint-policy.md"))?;
    std::fs::write(dir.join("policy-report.md"), md)?;
    println!("wrote {}", dir.join("policy-report.md").display());
    Ok(())
}

fn append_section(out: &mut String, title: &str, path: &Path) -> Result<()> {
    out.push_str(&format!("\n## {title}\n\n"));
    if path.exists() {
        let body = std::fs::read_to_string(path)?;
        for (i, line) in body.lines().enumerate() {
            if i == 0 && line.starts_with("# ") {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
    } else {
        out.push_str(&format!("(no artefact at `{}`)\n", path.display()));
    }
    Ok(())
}
