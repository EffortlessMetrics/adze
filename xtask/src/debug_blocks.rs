use anyhow::{Context, Result, bail};
use camino::{Utf8Path, Utf8PathBuf};
use regex::Regex;
use std::{env, fs, process::Command, sync::OnceLock};

#[derive(Debug, Clone)]
pub struct DebugBlockOptions {
    pub fix: bool,
    pub changed_only: bool,
    pub since: Option<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub line: usize,
    pub message: &'static str,
}

pub fn run(options: &DebugBlockOptions) -> Result<()> {
    let files = discover_files(options)?;

    if options.fix {
        let mut changed_any = false;
        for path in &files {
            if path.exists() && fix_file(path).with_context(|| format!("fixing {path}"))? {
                println!("fixed: {path}");
                changed_any = true;
            }
        }

        let mut remaining = Vec::new();
        for path in &files {
            if path.exists() {
                for violation in find_violations(path, false)
                    .with_context(|| format!("checking {path} after --fix"))?
                {
                    remaining.push((path.clone(), violation));
                }
            }
        }

        if !remaining.is_empty() {
            println!("❌ Still found unterminated blocks after --fix:");
            for (path, violation) in &remaining {
                emit_violation(path, violation);
            }
            bail!("debug-block validation failed after --fix");
        }

        if changed_any {
            println!("✅ Auto-fixes applied, no remaining unterminated debug blocks.");
        } else {
            println!("✅ No fixes required.");
        }
        return Ok(());
    }

    let mut violations = Vec::new();
    for path in &files {
        if !path.exists() {
            continue;
        }
        for violation in find_violations(path, options.changed_only)
            .with_context(|| format!("checking {path}"))?
        {
            violations.push((path.clone(), violation));
        }
    }

    if !violations.is_empty() {
        for (path, violation) in &violations {
            emit_violation(path, violation);
        }
        bail!("debug-block validation failed");
    }

    Ok(())
}

pub fn run_self_tests() {
    // The unit tests below carry the behavioral coverage formerly provided by
    // tools/test_debug_blocks.py. This hook intentionally stays lightweight for
    // `cargo xtask lint`: if the binary was built, the validator is available.
    println!(
        "✓ debug-block validator is implemented in Rust (unit-tested by `cargo test -p xtask debug_blocks`)"
    );
}

fn discover_files(options: &DebugBlockOptions) -> Result<Vec<Utf8PathBuf>> {
    if !options.files.is_empty() {
        return Ok(options
            .files
            .iter()
            .filter(|file| file.ends_with(".rs"))
            .map(Utf8PathBuf::from)
            .collect());
    }

    let output = if options.changed_only {
        git_output(&[
            "diff",
            "--name-only",
            "--cached",
            "--diff-filter=ACMR",
            "--",
            "*.rs",
        ])
    } else if let Some(since) = &options.since {
        git_output(&[
            "diff",
            "--name-only",
            &format!("{since}...HEAD"),
            "--",
            "*.rs",
        ])
    } else {
        git_output(&["ls-files", "*.rs"])
    };

    match output {
        Ok(output) => Ok(output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(Utf8PathBuf::from)
            .collect()),
        Err(_) => {
            let mut files = Vec::new();
            collect_rs_files(Utf8Path::new("runtime/src"), &mut files)?;
            Ok(files)
        }
    }
}

fn collect_rs_files(dir: &Utf8Path, files: &mut Vec<Utf8PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("reading {dir}"))? {
        let entry = entry?;
        let path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|path| anyhow::anyhow!("non-UTF-8 path: {}", path.display()))?;
        if path.is_dir() {
            collect_rs_files(&path, files)?;
        } else if path.extension() == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

pub fn find_violations(path: &Utf8Path, prefer_index: bool) -> Result<Vec<Violation>> {
    let text = if prefer_index {
        read_index(path).unwrap_or_else(|_| read_worktree_lossy(path).unwrap_or_default())
    } else {
        read_worktree_lossy(path)?
    };
    Ok(find_violations_in_text(&text))
}

fn find_violations_in_text(text: &str) -> Vec<Violation> {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    let mut violations = Vec::new();

    while i < lines.len() {
        let line = lines[i];
        if is_open(line) {
            if one_line_closed(line) {
                i += 1;
                continue;
            }

            let block_start = i + 1;
            let mut depth = 1;
            i += 1;
            let mut closed = false;
            while i < lines.len() {
                let cur = lines[i];
                if !cur.trim_start().starts_with("//") {
                    violations.push(Violation {
                        line: block_start,
                        message: "unterminated commented debug block (missing '// );')",
                    });
                    closed = true;
                    break;
                }
                if is_open(cur) && !one_line_closed(cur) {
                    depth += 1;
                } else if commented_close(cur) {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        closed = true;
                        break;
                    }
                }
                i += 1;
            }
            if !closed {
                violations.push(Violation {
                    line: block_start,
                    message: "unterminated commented debug block at EOF (missing '// );')",
                });
            }
        } else {
            i += 1;
        }
    }

    violations
}

pub fn fix_file(path: &Utf8Path) -> Result<bool> {
    let text = read_worktree_lossy(path)?;
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(ToOwned::to_owned).collect();
    let mut i = 0;
    let mut changed = false;

    while i < lines.len() {
        let line = &lines[i];
        if is_open(line) && !one_line_closed(line) {
            let indent = leading_whitespace(line).to_owned();
            let mut depth = 1;
            let mut last_comment_idx = i;
            i += 1;
            while i < lines.len() && lines[i].trim_start().starts_with("//") {
                last_comment_idx = i;
                let cur = &lines[i];
                if is_open(cur) && !one_line_closed(cur) {
                    depth += 1;
                } else if commented_close(cur) {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                i += 1;
            }

            if depth > 0 {
                for _ in 0..depth {
                    lines.insert(last_comment_idx + 1, format!("{indent}// );"));
                    last_comment_idx += 1;
                }
                changed = true;
                i = last_comment_idx + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    if changed {
        let mut out = lines.join(newline);
        out.push_str(newline);
        fs::write(path, out).with_context(|| format!("writing {path}"))?;
    }
    Ok(changed)
}

fn is_open(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix("//") else {
        return false;
    };
    if rest.starts_with('/') || rest.starts_with('!') {
        return false;
    }
    let rest = rest.trim_start();
    for macro_name in ["eprintln!", "println!", "dbg!"] {
        if let Some(after_macro) = rest.strip_prefix(macro_name) {
            return after_macro.trim_start().starts_with('(');
        }
    }
    false
}

fn one_line_closed(line: &str) -> bool {
    one_line_closed_re().is_match(line)
}

fn commented_close(line: &str) -> bool {
    commented_close_re().is_match(line)
}

fn leading_whitespace(line: &str) -> &str {
    let end = line
        .char_indices()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx))
        .unwrap_or(line.len());
    &line[..end]
}

fn one_line_closed_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\);\s*(?://.*)?$").unwrap())
}

fn commented_close_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*//\s*\)\s*;\s*(?://.*)?$").unwrap())
}

fn read_worktree_lossy(path: &Utf8Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("reading {path}"))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_index(path: &Utf8Path) -> Result<String> {
    let root = repo_root()?;
    let rel_path = path.strip_prefix(&root).unwrap_or(path);
    let rel = rel_path.as_str().replace('\\', "/");
    let output = Command::new("git")
        .args(["show", &format!(":{rel}")])
        .output()
        .context("running git show")?;
    if !output.status.success() {
        bail!("git show :{rel} failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn emit_violation(path: &Utf8Path, violation: &Violation) {
    if env::var_os("GITHUB_ACTIONS").is_some() {
        let root = repo_root().ok();
        let rel_path = root
            .as_deref()
            .and_then(|root| path.strip_prefix(root).ok())
            .unwrap_or(path);
        let rel = rel_path.as_str().replace('\\', "/");
        println!(
            "::error file={rel},line={},title=Commented debug block::{}",
            violation.line, violation.message
        );
    } else {
        println!("{path}:{}: {}", violation.line, violation.message);
    }
}

fn repo_root() -> Result<Utf8PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git rev-parse --show-toplevel")?;
    if !output.status.success() {
        bail!("git rev-parse --show-toplevel failed");
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok(Utf8PathBuf::from(root))
}

fn git_output(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!("git {} failed", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_blocks_accepts_valid_cases() {
        let valid_cases = [
            r#"// eprintln!("debug msg");"#,
            "// eprintln!(\n//   \"msg: {}\",\n//   x\n// );",
            "// eprintln!(\n//   \"msg\"\n// ); // done debugging",
            "// eprintln!(\n//   \"outer: {}\", \n//   // println!(\n//   //   \"inner\"\n//   // );\n//   value\n// );",
            r#"bail!("error message");"#,
            r#"panic!("unexpected state");"#,
            r#"debug_assert!(condition, "failed");"#,
            "// Some other comment\neprintln!(\"active debug\");",
            "/// eprintln!(\n/// docs only",
            "//! dbg!(\n//! docs only",
        ];

        for (idx, content) in valid_cases.iter().enumerate() {
            assert_eq!(
                find_violations_in_text(content),
                Vec::new(),
                "valid case {} was incorrectly flagged",
                idx + 1
            );
        }
    }

    #[test]
    fn debug_blocks_reports_invalid_cases() {
        let invalid_cases = [
            (
                "// eprintln!(\n//   \"msg: {}\",\n//   x\nlet y = 42;",
                "unterminated commented debug block",
            ),
            (
                "// eprintln!(\n//   \"msg\"\n//   no closer here",
                "unterminated commented debug block at EOF",
            ),
            (
                "// eprintln!(\n//   \"outer\",\n//   // println!(\n//   //   \"inner\"\n//   x\n// missing one );",
                "unterminated commented debug block",
            ),
        ];

        for (content, expected) in invalid_cases {
            let violations = find_violations_in_text(content);
            assert!(!violations.is_empty(), "invalid case was not flagged");
            assert!(
                violations[0].message.contains(expected),
                "expected message containing {expected:?}, got {:?}",
                violations[0].message
            );
        }
    }

    #[test]
    fn debug_blocks_fix_inserts_missing_closer() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = Utf8PathBuf::from_path_buf(dir.path().join("broken.rs"))
            .map_err(|path| anyhow::anyhow!("non-UTF-8 temp path: {}", path.display()))?;
        fs::write(&path, "// eprintln!(\n//   \"needs fix\"\nlet x = 1;")?;

        assert!(fix_file(&path)?);
        assert_eq!(
            fs::read_to_string(&path)?.trim(),
            "// eprintln!(\n//   \"needs fix\"\n// );\nlet x = 1;"
        );
        assert_eq!(find_violations(&path, false)?, Vec::new());
        Ok(())
    }
}
