//! Rust implementation of the commented-debug-block validator.
//!
//! The checker detects half-commented `eprintln!`, `println!`, and `dbg!`
//! blocks such as:
//!
//! ```text
//! // eprintln!(
//! //   "msg: {}",
//! //   x
//! let real_code = 42;
//! ```
//!
//! These are easy to create while temporarily commenting out debug output and
//! can accidentally hide following real code in the same comment region.

use anyhow::{Context, Result, bail};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct CheckArgs {
    pub fix: bool,
    pub changed_only: bool,
    pub since: Option<String>,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub line: usize,
    pub message: &'static str,
}

pub fn run_check(args: CheckArgs) -> Result<()> {
    let files = discover_files(&args)?;

    if args.fix {
        let mut changed_any = false;
        for path in &files {
            if path.exists() && fix_file(path)? {
                println!("fixed: {}", path.display());
                changed_any = true;
            }
        }

        let mut remaining = Vec::new();
        for path in &files {
            for violation in find_violations(path, false)? {
                remaining.push((path.clone(), violation));
            }
        }

        if !remaining.is_empty() {
            println!("❌ Still found unterminated blocks after --fix:");
            for (path, violation) in remaining {
                emit_violation(&path, &violation);
            }
            bail!("unterminated commented debug blocks remain after --fix");
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
        for violation in find_violations(path, args.changed_only)? {
            violations.push((path.clone(), violation));
        }
    }

    if !violations.is_empty() {
        for (path, violation) in violations {
            emit_violation(&path, &violation);
        }
        bail!("unterminated commented debug blocks found");
    }

    Ok(())
}

pub fn find_violations(path: &Path, prefer_index: bool) -> Result<Vec<Violation>> {
    let text = if prefer_index {
        read_index(path).unwrap_or_else(|_| read_worktree(path).unwrap_or_default())
    } else {
        read_worktree(path)?
    };
    Ok(find_violations_in_text(&text))
}

pub fn find_violations_in_text(text: &str) -> Vec<Violation> {
    let lines: Vec<&str> = text.lines().collect();
    let mut violations = Vec::new();
    let mut i = 0;

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

            while i < lines.len() {
                let current = lines[i];
                if !current.trim_start().starts_with("//") {
                    violations.push(Violation {
                        line: block_start,
                        message: "unterminated commented debug block (missing '// );')",
                    });
                    break;
                }

                if is_open(current) && !one_line_closed(current) {
                    depth += 1;
                } else if is_commented_close(current) {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }

            if i >= lines.len() && depth > 0 {
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

pub fn fix_file(path: &Path) -> Result<bool> {
    let text = read_worktree(path)?;
    let newline = detect_newline(&text);
    let mut lines: Vec<String> = text.lines().map(ToOwned::to_owned).collect();
    let mut i = 0;
    let mut changed = false;

    while i < lines.len() {
        let line = &lines[i];
        if is_open(line) && !one_line_closed(line) {
            let indent = leading_whitespace(line).to_string();
            let mut depth = 1;
            let mut last_comment_idx = i;
            i += 1;

            while i < lines.len() && lines[i].trim_start().starts_with("//") {
                last_comment_idx = i;
                let current = &lines[i];
                if is_multiline_open(current) {
                    depth += 1;
                } else if is_commented_close(current) {
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
        fs::write(path, format!("{}{}", lines.join(newline), newline))
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(changed)
}

fn discover_files(args: &CheckArgs) -> Result<Vec<PathBuf>> {
    if !args.files.is_empty() {
        return Ok(args
            .files
            .iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .cloned()
            .collect());
    }

    let git_args: Vec<String> = if args.changed_only {
        [
            "diff",
            "--name-only",
            "--cached",
            "--diff-filter=ACMR",
            "--",
            "*.rs",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    } else if let Some(since) = &args.since {
        vec![
            "diff".to_owned(),
            "--name-only".to_owned(),
            format!("{since}...HEAD"),
            "--".to_owned(),
            "*.rs".to_owned(),
        ]
    } else {
        vec!["ls-files".to_owned(), "*.rs".to_owned()]
    };
    let git_arg_refs: Vec<&str> = git_args.iter().map(String::as_str).collect();

    match run_git(&git_arg_refs) {
        Ok(stdout) => Ok(stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect()),
        Err(_) => Ok(WalkDir::new("runtime/src")
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .collect()),
    }
}

fn is_open(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("///") || trimmed.starts_with("//!") || !trimmed.starts_with("//") {
        return false;
    }
    let after_comment = trimmed[2..].trim_start();
    ["eprintln!", "println!", "dbg!"].iter().any(|prefix| {
        after_comment
            .strip_prefix(prefix)
            .is_some_and(starts_with_open_paren)
    })
}

fn starts_with_open_paren(rest: &str) -> bool {
    rest.trim_start().starts_with('(')
}

fn one_line_closed(line: &str) -> bool {
    line.contains(");")
}

fn is_multiline_open(line: &str) -> bool {
    is_open(line) && !one_line_closed(line)
}

fn is_commented_close(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("//") {
        return false;
    }
    trimmed[2..].trim_start().starts_with(");")
}

fn leading_whitespace(line: &str) -> &str {
    let end = line
        .char_indices()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx))
        .unwrap_or(line.len());
    &line[..end]
}

fn detect_newline(text: &str) -> &'static str {
    if text.contains("\r\n") { "\r\n" } else { "\n" }
}

fn read_worktree(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

fn read_index(path: &Path) -> Result<String> {
    let root = repo_root()?;
    let rel = path.strip_prefix(&root).unwrap_or(path);
    let rel_posix = rel
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    run_git(&["show", &format!(":{rel_posix}")])
}

fn emit_violation(path: &Path, violation: &Violation) {
    if env::var_os("GITHUB_ACTIONS").is_some() {
        let rel = repo_root()
            .ok()
            .and_then(|root| path.strip_prefix(root).ok().map(PathBuf::from))
            .unwrap_or_else(|| path.to_path_buf())
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        println!(
            "::error file={rel},line={},title=Commented debug block::{}",
            violation.line, violation.message
        );
    } else {
        println!(
            "{}:{}: {}",
            path.display(),
            violation.line,
            violation.message
        );
    }
}

fn repo_root() -> Result<PathBuf> {
    Ok(PathBuf::from(
        run_git(&["rev-parse", "--show-toplevel"])?.trim(),
    ))
}

fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn valid_cases_are_accepted() {
        let valid_cases = [
            r#"// eprintln!("debug msg");"#,
            "// eprintln!(\n//   \"msg: {}\",\n//   x\n// );",
            "// eprintln!(\n//   \"msg\"\n// ); // done debugging",
            "// eprintln!(\n//   \"outer: {}\", \n//   // println!(\n//   //   \"inner\"\n//   // );\n//   value\n// );",
            r#"bail!("error message");"#,
            r#"panic!("unexpected state");"#,
            r#"debug_assert!(condition, "failed");"#,
            "// Some other comment\neprintln!(\"active debug\");",
            "/// eprintln!(\n///   \"doc comment\"\n/// );",
            "//! dbg!(\n//!   inner_docs\n//! );",
        ];

        for case in valid_cases {
            assert_eq!(find_violations_in_text(case), Vec::new(), "{case}");
        }
    }

    #[test]
    fn invalid_cases_are_reported() {
        let cases = [
            (
                "// eprintln!(\n//   \"msg: {}\",\n//   x\nlet y = 42;",
                "unterminated commented debug block (missing '// );')",
            ),
            (
                "// eprintln!(\n//   \"msg\"\n//   no closer here",
                "unterminated commented debug block at EOF (missing '// );')",
            ),
            (
                "// eprintln!(\n//   \"outer\",\n//   // println!(\n//   //   \"inner\"\n//   x\n// missing one );",
                "unterminated commented debug block at EOF (missing '// );')",
            ),
        ];

        for (case, expected) in cases {
            let violations = find_violations_in_text(case);
            assert!(!violations.is_empty(), "{case}");
            assert_eq!(violations[0].message, expected);
        }
    }

    #[test]
    fn fix_mode_inserts_missing_closer_before_real_code() -> Result<()> {
        let file = NamedTempFile::new()?;
        let path = file.path().to_path_buf();
        fs::write(&path, "// eprintln!(\n//   \"needs fix\"\nlet x = 1;")?;

        assert!(fix_file(&path)?);
        let fixed = fs::read_to_string(&path)?;
        assert_eq!(
            fixed.trim(),
            "// eprintln!(\n//   \"needs fix\"\n// );\nlet x = 1;"
        );
        assert_eq!(find_violations(&path, false)?, Vec::new());
        Ok(())
    }

    #[test]
    fn fix_mode_preserves_crlf_newlines() -> Result<()> {
        let file = NamedTempFile::new()?;
        let path = file.path().to_path_buf();
        fs::write(&path, "// dbg!(\r\n//   value\r\nlet x = 1;\r\n")?;

        assert!(fix_file(&path)?);
        let fixed = fs::read_to_string(&path)?;
        assert!(fixed.contains("// );\r\n"));
        assert_eq!(find_violations(&path, false)?, Vec::new());
        Ok(())
    }
}
