mod example_integration;

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    use anyhow::{Context, Result};
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::PathBuf;

    /// Represents a golden test case
    struct GoldenTest {
        language: &'static str,
        fixture_name: &'static str,
    }

    impl GoldenTest {
        fn fixture_path(&self) -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(self.language)
                .join("fixtures")
                .join(self.fixture_name)
        }

        fn expected_hash_path(&self) -> PathBuf {
            let base_name = self
                .fixture_name
                .replace(&format!(".{}", self.language), "");
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(self.language)
                .join("expected")
                .join(format!("{}.sha256", base_name))
        }

        fn expected_sexp_path(&self) -> PathBuf {
            let base_name = self
                .fixture_name
                .replace(&format!(".{}", self.language), "");
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(self.language)
                .join("expected")
                .join(format!("{}.sexp", base_name))
        }
    }

    /// Parse a file with rust-sitter and return S-expression
    fn parse_with_rust_sitter(language: &str, source: &str) -> Result<String> {
        match language {
            "python" => parse_python(source),
            "javascript" => parse_javascript(source),
            _ => anyhow::bail!("Unsupported language: {}", language),
        }
    }

    /// Parse Python source code and return S-expression
    #[cfg(feature = "python-grammar")]
    fn parse_python(source: &str) -> Result<String> {
        // This would use the rust-sitter generated Python parser
        // For now, return a placeholder
        todo!("Python parser integration")
    }

    #[cfg(not(feature = "python-grammar"))]
    fn parse_python(_source: &str) -> Result<String> {
        anyhow::bail!("Python grammar feature not enabled")
    }

    /// Parse JavaScript source code and return S-expression
    #[cfg(feature = "javascript-grammar")]
    fn parse_javascript(source: &str) -> Result<String> {
        // This would use the rust-sitter generated JavaScript parser
        // For now, return a placeholder
        todo!("JavaScript parser integration")
    }

    #[cfg(not(feature = "javascript-grammar"))]
    fn parse_javascript(_source: &str) -> Result<String> {
        anyhow::bail!("JavaScript grammar feature not enabled")
    }

    /// Compute SHA256 hash of a string
    fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Run a golden test
    fn run_golden_test(test: GoldenTest) -> Result<()> {
        // Read the source file
        let source = fs::read_to_string(test.fixture_path())
            .with_context(|| format!("Failed to read fixture: {}", test.fixture_name))?;

        // Parse with rust-sitter
        let sexp = parse_with_rust_sitter(test.language, &source)
            .with_context(|| format!("Failed to parse {}", test.fixture_name))?;

        // Check if we're in "update" mode
        if std::env::var("UPDATE_GOLDEN").is_ok() {
            // Update mode: generate new reference files
            println!("Updating golden reference for {}", test.fixture_name);

            // Note: In real implementation, we'd run tree-sitter here
            // For now, we just save what rust-sitter produces
            fs::write(test.expected_sexp_path(), &sexp)?;

            let hash = compute_hash(&sexp);
            fs::write(test.expected_hash_path(), &hash)?;

            return Ok(());
        }

        // Normal mode: compare against reference
        if test.expected_hash_path().exists() {
            // Hash-based comparison (more efficient)
            let expected_hash = fs::read_to_string(test.expected_hash_path())
                .with_context(|| "Failed to read expected hash")?
                .trim()
                .to_string();

            let actual_hash = compute_hash(&sexp);

            if actual_hash != expected_hash {
                // On hash mismatch, show more detailed error
                if test.expected_sexp_path().exists() {
                    let _expected_sexp = fs::read_to_string(test.expected_sexp_path())
                        .with_context(|| "Failed to read expected S-expression")?;

                    // Save actual output for debugging
                    let debug_path = test.expected_sexp_path().with_extension("actual.sexp");
                    fs::write(&debug_path, &sexp)?;

                    anyhow::bail!(
                        "Parse tree mismatch for {}:\n\
                         Expected hash: {}\n\
                         Actual hash:   {}\n\
                         \n\
                         Expected S-expression saved to: {}\n\
                         Actual S-expression saved to: {}\n\
                         \n\
                         To update golden files, run with UPDATE_GOLDEN=1",
                        test.fixture_name,
                        expected_hash,
                        actual_hash,
                        test.expected_sexp_path().display(),
                        debug_path.display()
                    );
                }

                anyhow::bail!(
                    "Parse tree hash mismatch for {}:\n\
                     Expected: {}\n\
                     Actual:   {}",
                    test.fixture_name,
                    expected_hash,
                    actual_hash
                );
            }
        } else {
            // No reference file exists
            anyhow::bail!(
                "No golden reference found for {}. \
                 Run ./generate_references.sh to create reference files.",
                test.fixture_name
            );
        }

        Ok(())
    }

    // Python golden tests
    #[test]
    #[cfg(feature = "python-grammar")]
    fn python_tokenize_golden() -> Result<()> {
        run_golden_test(GoldenTest {
            language: "python",
            fixture_name: "tokenize_sample.py",
        })
    }

    // JavaScript golden tests
    #[test]
    #[cfg(feature = "javascript-grammar")]
    fn javascript_react_dom_golden() -> Result<()> {
        run_golden_test(GoldenTest {
            language: "javascript",
            fixture_name: "react_dom_sample.js",
        })
    }

    // Test to ensure reference generation script exists
    #[test]
    fn reference_script_exists() {
        let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generate_references.sh");
        assert!(
            script_path.exists(),
            "Reference generation script not found at: {}",
            script_path.display()
        );
    }
}
