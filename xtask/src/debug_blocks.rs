use anyhow::{Context, Result, bail};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct CheckOptions {
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

pub fn run(options: CheckOptions) -> Result<()> {
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
            bail!("debug-block validation failed");
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

pub fn self_test() -> Result<()> {
    println!("Running debug block validator self-tests...");
    test_valid_cases()?;
    println!("✅ Valid cases passed");
    test_invalid_cases()?;
    println!("✅ Invalid cases passed");
    test_fix_mode()?;
    println!("✅ Fix mode passed");
    println!("✅ All tests passed");
    Ok(())
}

fn discover_files(options: &CheckOptions) -> Result<Vec<PathBuf>> {
    if !options.files.is_empty() {
        return Ok(options
            .files
            .iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .cloned()
            .collect());
    }

    let git_args: Vec<String> = if options.changed_only {
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
    } else if let Some(rev) = &options.since {
        vec![
            "diff".into(),
            "--name-only".into(),
            format!("{rev}...HEAD"),
            "--".into(),
            "*.rs".into(),
        ]
    } else {
        vec!["ls-files".into(), "*.rs".into()]
    };

    match Command::new("git").args(&git_args).output() {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect()),
        _ => runtime_src_files(),
    }
}

fn runtime_src_files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit_rs_files(Path::new("runtime/src"), &mut files)?;
    Ok(files)
}

fn visit_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, files)?;
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
        for (line, message) in find_violations(path, prefer_index)? {
            violations.push(Violation {
                path: path.clone(),
                line,
                message,
            });
        }
    }
    Ok(violations)
}

pub fn find_violations(path: &Path, prefer_index: bool) -> Result<Vec<(usize, &'static str)>> {
    let text = if prefer_index {
        read_index(path).or_else(|_| read_worktree(path))?
    } else {
        read_worktree(path)?
    };

    Ok(find_violations_in_text(&text))
}

fn find_violations_in_text(text: &str) -> Vec<(usize, &'static str)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut index = 0;
    let mut violations = Vec::new();

    while index < lines.len() {
        let line = lines[index];
        if is_open(line) {
            if one_line_closed(line) {
                index += 1;
                continue;
            }

            let block_start = index + 1;
            let mut depth = 1;
            index += 1;
            while index < lines.len() {
                let current = lines[index];
                if !current.trim_start().starts_with("//") {
                    violations.push((
                        block_start,
                        "unterminated commented debug block (missing '// );')",
                    ));
                    break;
                }
                if is_open(current) && !one_line_closed(current) {
                    depth += 1;
                } else if is_commented_close(current) {
                    depth -= 1;
                    if depth == 0 {
                        index += 1;
                        break;
                    }
                }
                index += 1;
            }

            if index == lines.len() && depth > 0 {
                violations.push((
                    block_start,
                    "unterminated commented debug block at EOF (missing '// );')",
                ));
            }
        } else {
            index += 1;
        }
    }

    violations
}

pub fn fix_file(path: &Path) -> Result<bool> {
    let text = read_worktree(path)?;
    let newline = detect_newline(&text);
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let mut index = 0;
    let mut changed = false;

    while index < lines.len() {
        let line = &lines[index];
        if is_open(line) && !one_line_closed(line) {
            let indent = leading_whitespace(line).to_owned();
            let mut depth = 1;
            let mut last_comment_idx = index;
            index += 1;

            while index < lines.len() && lines[index].trim_start().starts_with("//") {
                last_comment_idx = index;
                let current = &lines[index];
                if is_multiline_open(current) {
                    depth += 1;
                } else if is_commented_close(current) {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                index += 1;
            }

            if depth > 0 {
                for _ in 0..depth {
                    lines.insert(last_comment_idx + 1, format!("{indent}// );"));
                    last_comment_idx += 1;
                }
                changed = true;
                index = last_comment_idx + 1;
            } else {
                index += 1;
            }
        } else {
            index += 1;
        }
    }

    if changed {
        fs::write(path, format!("{}{}", lines.join(newline), newline))
            .with_context(|| format!("writing {}", path.display()))?;
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
    ["eprintln!", "println!", "dbg!"]
        .into_iter()
        .any(|macro_name| {
            rest.strip_prefix(macro_name)
                .is_some_and(|tail| tail.trim_start().starts_with('('))
        })
}

fn one_line_closed(line: &str) -> bool {
    line.find(");")
        .is_some_and(|pos| trailing_comment_or_empty(&line[pos + 2..]))
}

fn is_multiline_open(line: &str) -> bool {
    is_open(line) && !one_line_closed(line)
}

fn is_commented_close(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix("//") else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix(')') else {
        return false;
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix(';') else {
        return false;
    };
    trailing_comment_or_empty(rest)
}

fn trailing_comment_or_empty(rest: &str) -> bool {
    let rest = rest.trim_start();
    rest.is_empty() || rest.starts_with("//")
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
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_index(path: &Path) -> Result<String> {
    let root = repo_root()?;
    let rel = path.strip_prefix(&root).unwrap_or(path);
    let rel = rel
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");
    let output = Command::new("git")
        .args(["show", &format!(":{rel}")])
        .output()
        .with_context(|| format!("reading staged blob for {}", path.display()))?;
    if !output.status.success() {
        bail!("git show failed for {}", path.display());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("resolving repository root")?;
    if !output.status.success() {
        bail!("git rev-parse --show-toplevel failed");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
    ))
}

fn emit_violation(violation: &Violation) {
    if env::var_os("GITHUB_ACTIONS").is_some() {
        let rel = repo_root()
            .ok()
            .and_then(|root| violation.path.strip_prefix(root).ok().map(PathBuf::from))
            .unwrap_or_else(|| violation.path.clone())
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

fn test_valid_cases() -> Result<()> {
    let valid_cases = [
        "// eprintln!(\"debug msg\");",
        "// eprintln!(\n//   \"msg: {}\",\n//   x\n// );",
        "// eprintln!(\n//   \"msg\"\n// ); // done debugging",
        "// eprintln!(\n//   \"outer: {}\", \n//   // println!(\n//   //   \"inner\"\n//   // );\n//   value\n// );",
        "bail!(\"error message\");",
        "panic!(\"unexpected state\");",
        "debug_assert!(condition, \"failed\");",
        "// Some other comment\neprintln!(\"active debug\");",
        "/// eprintln!(\"doc mention\"\nlet value = 1;",
        "//! println!(\"inner doc mention\"\nlet value = 1;",
    ];

    for (idx, content) in valid_cases.iter().enumerate() {
        let violations = find_violations_in_text(content);
        if !violations.is_empty() {
            bail!("valid case {} incorrectly flagged: {violations:?}", idx + 1);
        }
    }
    Ok(())
}

fn test_invalid_cases() -> Result<()> {
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

    for (idx, (content, expected_msg)) in invalid_cases.iter().enumerate() {
        let violations = find_violations_in_text(content);
        if violations.is_empty() {
            bail!("invalid case {} was not flagged", idx + 1);
        }
        if !violations[0].1.contains(expected_msg) {
            bail!(
                "invalid case {} reported wrong message: expected {expected_msg:?}, got {:?}",
                idx + 1,
                violations[0].1
            );
        }
    }
    Ok(())
}

fn test_fix_mode() -> Result<()> {
    let dir = tempfile::tempdir().context("creating debug-block self-test tempdir")?;
    let path = dir.path().join("broken.rs");
    let broken = "// eprintln!(\n//   \"needs fix\"\nlet x = 1;";
    let expected_fixed = "// eprintln!(\n//   \"needs fix\"\n// );\nlet x = 1;\n";
    fs::write(&path, broken).context("writing debug-block self-test fixture")?;

    if !fix_file(&path)? {
        bail!("fix mode did not detect broken block");
    }

    let fixed = fs::read_to_string(&path).context("reading fixed self-test fixture")?;
    if fixed != expected_fixed {
        bail!("fix mode produced wrong output:\nexpected:\n{expected_fixed}\ngot:\n{fixed}");
    }

    let violations = find_violations(&path, false)?;
    if !violations.is_empty() {
        bail!("fix mode left violations: {violations:?}");
    }

    Ok(())
}
