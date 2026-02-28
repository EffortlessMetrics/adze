//! Helpers for loading test fixtures and corpus files.
//!
//! Supports two fixture formats:
//!
//! 1. **Plain text** – entire file contents returned as a `String`.
//! 2. **Tree-sitter corpus format** – `=== title\n<input>\n---\n<expected>` blocks
//!    parsed into [`CorpusEntry`] structs.

use std::fs;
use std::path::{Path, PathBuf};

/// A single entry from a tree-sitter style corpus file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusEntry {
    /// Human-readable test title (text after `===`).
    pub title: String,
    /// Source code to be parsed.
    pub input: String,
    /// Expected S-expression tree output.
    pub expected: String,
}

/// Resolve the project-root-relative fixtures directory.
///
/// Walks up from `CARGO_MANIFEST_DIR` until it finds `corpus/` or
/// returns `<manifest>/fixtures` as a fallback.
pub fn fixtures_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Prefer the workspace-level corpus/ directory.
    let workspace_corpus = manifest.parent().map(|p| p.join("corpus"));
    if let Some(ref dir) = workspace_corpus {
        if dir.is_dir() {
            return dir.clone();
        }
    }

    // Fall back to a per-crate fixtures/ directory.
    manifest.join("fixtures")
}

/// Load a fixture file by name from the fixtures directory.
///
/// The `name` should be a path relative to the fixtures root, e.g.
/// `"test.txt"` or `"javascript/statements.txt"`.
///
/// # Errors
///
/// Returns an error if the file does not exist or cannot be read.
pub fn load_fixture(name: &str) -> Result<String, std::io::Error> {
    let path = fixtures_dir().join(name);
    fs::read_to_string(&path)
}

/// Load a fixture, panicking with a descriptive message on failure.
pub fn load_fixture_or_panic(name: &str) -> String {
    load_fixture(name).unwrap_or_else(|e| {
        panic!(
            "failed to load fixture `{name}` from {}: {e}",
            fixtures_dir().display()
        )
    })
}

/// Parse a tree-sitter corpus file into individual test entries.
///
/// The corpus format uses `===` lines as entry headers and `---` to
/// separate source input from expected output:
///
/// ```text
/// === Title of test
/// source code here
/// ---
///
/// (expected_tree)
/// ```
pub fn parse_corpus(content: &str) -> Vec<CorpusEntry> {
    let mut entries = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("===") {
            let title = title.trim().to_string();

            // Collect input lines until `---`.
            let mut input_lines = Vec::new();
            for line in lines.by_ref() {
                if line.trim() == "---" {
                    break;
                }
                input_lines.push(line);
            }

            // Collect expected output until next `===` or EOF.
            let mut expected_lines = Vec::new();
            while let Some(&next) = lines.peek() {
                if next.trim().starts_with("===") {
                    break;
                }
                expected_lines.push(lines.next().unwrap());
            }

            let input = input_lines.join("\n");
            let expected = expected_lines.join("\n").trim().to_string();

            if !title.is_empty() {
                entries.push(CorpusEntry {
                    title,
                    input,
                    expected,
                });
            }
        }
    }

    entries
}

/// Load and parse a corpus file from the fixtures directory.
pub fn load_corpus(name: &str) -> Result<Vec<CorpusEntry>, std::io::Error> {
    let content = load_fixture(name)?;
    Ok(parse_corpus(&content))
}

/// Create a temporary fixture file that is cleaned up when `dir` is dropped.
///
/// Useful for tests that need to write files to disk and verify load
/// behaviour.
pub fn write_temp_fixture(
    dir: &Path,
    name: &str,
    content: &str,
) -> Result<PathBuf, std::io::Error> {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_corpus_basic() {
        let corpus = "\
=== Simple function
function test() {
  return 42;
}
---

(source_file
  (function_declaration
    name: (identifier)
    parameters: (formal_parameters)
    body: (statement_block
      (return_statement (number)))))
";
        let entries = parse_corpus(corpus);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Simple function");
        assert!(entries[0].input.contains("function test()"));
        assert!(entries[0].expected.contains("source_file"));
    }

    #[test]
    fn parse_corpus_multiple() {
        let corpus = "\
=== First
a
---
(a)
=== Second
b
---
(b)
";
        let entries = parse_corpus(corpus);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "First");
        assert_eq!(entries[1].title, "Second");
    }

    #[test]
    fn fixtures_dir_exists_or_fallback() {
        // Should not panic; returns a path regardless.
        let dir = fixtures_dir();
        assert!(dir.to_str().is_some());
    }

    #[test]
    fn write_and_load_temp_fixture() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_fixture(dir.path(), "hello.txt", "world").unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), "world");
    }

    #[test]
    fn load_corpus_from_workspace() {
        // The workspace `corpus/test.txt` uses the corpus format.
        let entries = load_corpus("test.txt");
        if let Ok(entries) = entries {
            assert!(!entries.is_empty());
            assert_eq!(entries[0].title, "Simple function");
        }
        // Not a hard failure if corpus dir is absent.
    }
}
