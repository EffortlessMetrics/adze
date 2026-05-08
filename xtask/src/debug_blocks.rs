use anyhow::{Context, Result, bail};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct DebugBlockOptions {
    pub fix: bool,
    pub changed_only: bool,
    pub since: Option<String>,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub path: PathBuf,
    pub line: usize,
    pub message: &'static str,
}

pub fn run(options: DebugBlockOptions) -> Result<()> {
    let files = discover_files(&options)?;

    if options.fix {
        let mut changed_any = false;
        for path in &files {
            if path.exists() && fix_file(path)? {
                println!("fixed: {}", path.display());
                changed_any = true;
            }
        }

        let remaining = collect_violations(&files, false)?;
        if !remaining.is_empty() {
            println!("❌ Still found unterminated blocks after --fix:");
            for violation in &remaining {
                emit_violation(violation);
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

    let violations = collect_violations(&files, options.changed_only)?;
    if !violations.is_empty() {
        for violation in &violations {
            emit_violation(violation);
        }
        bail!("debug-block validation failed");
    }

    Ok(())
}

fn discover_files(options: &DebugBlockOptions) -> Result<Vec<PathBuf>> {
    if !options.files.is_empty() {
        return Ok(options
            .files
            .iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .cloned()
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
        Ok(stdout) => Ok(stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect()),
        Err(_) => {
            let mut fallback = Vec::new();
            collect_runtime_rs(Path::new("runtime/src"), &mut fallback)?;
            Ok(fallback)
        }
    }
}

fn collect_runtime_rs(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_runtime_rs(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn collect_violations(files: &[PathBuf], prefer_index: bool) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for path in files {
        if !path.exists() {
            continue;
        }
        violations.extend(find_violations(path, prefer_index)?);
    }
    Ok(violations)
}

pub fn find_violations(path: &Path, prefer_index: bool) -> Result<Vec<Violation>> {
    let text = if prefer_index {
        read_index(path).unwrap_or_else(|_| read_worktree_lossy(path).unwrap_or_default())
    } else {
        read_worktree_lossy(path)?
    };
    Ok(find_violations_in_text(path, &text))
}

fn find_violations_in_text(path: &Path, text: &str) -> Vec<Violation> {
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
            let mut depth = 1usize;
            i += 1;
            while i < lines.len() {
                let current = lines[i];
                if !current.trim_start().starts_with("//") {
                    violations.push(Violation {
                        path: path.to_path_buf(),
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
                    path: path.to_path_buf(),
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
    let text = read_worktree_lossy(path)?;
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let mut i = 0;
    let mut changed = false;

    while i < lines.len() {
        let line = &lines[i];
        if is_open(line) && !one_line_closed(line) {
            let indent = leading_whitespace(line).to_owned();
            let mut depth = 1usize;
            let mut last_comment_idx = i;
            i += 1;

            while i < lines.len() && lines[i].trim_start().starts_with("//") {
                last_comment_idx = i;
                let current = &lines[i];
                if is_open(current) && !one_line_closed(current) {
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
            .with_context(|| format!("write {}", path.display()))?;
    }

    Ok(changed)
}

fn is_open(line: &str) -> bool {
    let Some(after_slashes) = line.trim_start().strip_prefix("//") else {
        return false;
    };
    if after_slashes.starts_with('/') || after_slashes.starts_with('!') {
        return false;
    }
    let rest = after_slashes.trim_start();
    ["eprintln!", "println!", "dbg!"].iter().any(|macro_name| {
        rest.strip_prefix(macro_name)
            .is_some_and(has_open_paren_after_ws)
    })
}

fn has_open_paren_after_ws(rest: &str) -> bool {
    rest.trim_start().starts_with('(')
}

fn one_line_closed(line: &str) -> bool {
    line.contains(");")
}

fn is_commented_close(line: &str) -> bool {
    let Some(after_slashes) = line.trim_start().strip_prefix("//") else {
        return false;
    };
    let rest = after_slashes.trim_start();
    let Some(rest) = rest.strip_prefix(')') else {
        return false;
    };
    rest.trim_start().starts_with(';')
}

fn leading_whitespace(line: &str) -> &str {
    let len = line.len() - line.trim_start().len();
    &line[..len]
}

fn emit_violation(violation: &Violation) {
    if env::var_os("GITHUB_ACTIONS").is_some() {
        let rel = repo_relative_path(&violation.path).unwrap_or_else(|| violation.path.clone());
        let rel = rel
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        println!(
            "::error file={rel},line={},title=Commented debug block::{}",
            violation.line, violation.message
        );
    } else {
        println!(
            "{}:{}: {}",
            violation.path.display(),
            violation.line,
            violation.message
        );
    }
}

fn repo_relative_path(path: &Path) -> Option<PathBuf> {
    let root = repo_root().ok()?;
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().ok()?.join(path)
    };
    absolute.strip_prefix(root).ok().map(Path::to_path_buf)
}

fn read_worktree_lossy(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_index(path: &Path) -> Result<String> {
    let root = repo_root()?;
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let rel = absolute
        .strip_prefix(&root)
        .with_context(|| format!("{} is outside {}", path.display(), root.display()))?;
    let rel = rel
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    let stdout = git_output(&["show", &format!(":{rel}")])?;
    Ok(stdout)
}

fn repo_root() -> Result<PathBuf> {
    let root = git_output(&["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(root.trim()))
}

fn git_output(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn write_test_file(content: &str) -> NamedTempFile {
        let file = NamedTempFile::with_suffix(".rs").expect("create temp file");
        fs::write(file.path(), content).expect("write temp file");
        file
    }

    #[test]
    fn valid_cases_are_not_flagged() {
        let valid_cases = [
            r#"// eprintln!("debug msg");"#,
            "// eprintln!(\n//   \"msg: {}\",\n//   x\n// );",
            "// eprintln!(\n//   \"msg\"\n// ); // done debugging",
            "// eprintln!(\n//   \"outer: {}\", \n//   // println!(\n//   //   \"inner\"\n//   // );\n//   value\n// );",
            r#"bail!("error message");"#,
            r#"panic!("unexpected state");"#,
            r#"debug_assert!(condition, "failed");"#,
            "// Some other comment\neprintln!(\"active debug\");",
        ];

        for (index, content) in valid_cases.iter().enumerate() {
            let file = write_test_file(content);
            let violations = find_violations(file.path(), false).expect("find violations");
            assert!(
                violations.is_empty(),
                "valid case {} incorrectly flagged: {violations:?}",
                index + 1
            );
        }
    }

    #[test]
    fn invalid_cases_are_flagged() {
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

        for (index, (content, expected_message)) in invalid_cases.iter().enumerate() {
            let file = write_test_file(content);
            let violations = find_violations(file.path(), false).expect("find violations");
            assert!(
                !violations.is_empty(),
                "invalid case {} was not flagged",
                index + 1
            );
            assert!(
                violations[0].message.contains(expected_message),
                "invalid case {} wrong message: {:?}",
                index + 1,
                violations[0].message
            );
        }
    }

    #[test]
    fn fix_mode_adds_missing_closer() {
        let broken = "// eprintln!(\n//   \"needs fix\"\nlet x = 1;";
        let expected = "// eprintln!(\n//   \"needs fix\"\n// );\nlet x = 1;\n";
        let file = write_test_file(broken);

        assert!(fix_file(file.path()).expect("fix file"));
        let fixed = fs::read_to_string(file.path()).expect("read fixed file");
        assert_eq!(fixed, expected);
        assert!(
            find_violations(file.path(), false)
                .expect("find violations")
                .is_empty()
        );
    }

    #[test]
    fn fix_mode_preserves_crlf_newlines() {
        let file = write_test_file("// dbg!(\r\n//   value\r\nlet x = 1;\r\n");

        assert!(fix_file(file.path()).expect("fix file"));
        let fixed = fs::read_to_string(file.path()).expect("read fixed file");
        assert!(fixed.contains("// );\r\n"));
        assert_eq!(
            find_violations(file.path(), false).expect("find violations"),
            Vec::new()
        );
    }

    #[test]
    fn doc_comments_are_ignored() {
        let file = write_test_file(
            "/// eprintln!(\n/// docs only\nfn main() {}\n//! dbg!(\n//! docs only",
        );
        let violations = find_violations(file.path(), false).expect("find violations");
        assert!(violations.is_empty());
    }
}
