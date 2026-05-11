use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn readme_arithmetic_quickstart_builds_and_runs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_dir = temp.path().join("adze_readme_quickstart");
    fs::create_dir_all(project_dir.join("src")).expect("create src");
    fs::create_dir_all(project_dir.join("tests")).expect("create tests");

    let repo_root = repo_root();
    let runtime_path = toml_path(repo_root.join("runtime"));
    let tool_path = toml_path(repo_root.join("tool"));
    let readme = include_str!("../../README.md");
    let manifest_snippet = fenced_block_after(readme, "## Install", "toml")
        .expect("README install section should include a TOML dependency block");
    let build_rs = fenced_block_after(readme, "Add a `build.rs`", "rust")
        .expect("README install section should include a build.rs block");
    let grammar_snippet = fenced_block_starting_with(readme, "rust", "#[adze::grammar")
        .expect("README should include the arithmetic grammar quickstart block");
    assert!(
        grammar_snippet.contains(r#"let expr = grammar::parse("1 + 2 * 3")?;"#),
        "README grammar block should show the documented parser call"
    );

    fs::write(
        project_dir.join("Cargo.toml"),
        downstream_manifest(manifest_snippet, &runtime_path, &tool_path),
    )
    .expect("write Cargo.toml");

    fs::write(project_dir.join("build.rs"), build_rs).expect("write build.rs");

    fs::write(
        project_dir.join("src/lib.rs"),
        grammar_module_from_readme(grammar_snippet),
    )
    .expect("write lib.rs");

    fs::write(
        project_dir.join("tests/readme_quickstart.rs"),
        r#"use adze_readme_quickstart::grammar::{self, Expr};

#[test]
fn readme_expression_respects_precedence() {
    let expr = grammar::parse("1 + 2 * 3").expect("README expression should parse");

    assert_eq!(
        expr,
        Expr::Add(
            Box::new(Expr::Number(1)),
            (),
            Box::new(Expr::Mul(
                Box::new(Expr::Number(2)),
                (),
                Box::new(Expr::Number(3)),
            )),
        )
    );
}

#[test]
fn readme_bad_input_reports_useful_diagnostic() {
    let source = "1 + @";
    let errors = grammar::parse(source).expect_err("bad README input should fail clearly");
    let first = errors
        .first()
        .expect("bad README input should produce at least one parse error");

    assert_eq!(
        first.byte_span(),
        4..5,
        "diagnostic should point at the invalid token"
    );
    assert!(
        !first.expected.is_empty(),
        "diagnostic should report expected tokens"
    );
    assert!(
        first.expected.iter().any(|name| name == r"/\d+/"),
        "diagnostic should name the expected number token, got {:?}",
        first.expected
    );

    let rendered = first.display_with_source(source).to_string();
    assert!(
        rendered.contains("bytes 4..5"),
        "rendered diagnostic should include the byte span: {rendered}"
    );
    assert!(
        rendered.contains("expected one of:"),
        "rendered diagnostic should include expected-token context: {rendered}"
    );
    assert!(
        rendered.contains("    ^"),
        "rendered diagnostic should place a caret under the invalid token: {rendered}"
    );
}
"#,
    )
    .expect("write quickstart test");

    let output = Command::new("cargo")
        .arg("test")
        .arg("--quiet")
        .current_dir(&project_dir)
        .output()
        .expect("run cargo test in README quickstart crate");

    assert!(
        output.status.success(),
        "README quickstart crate should build and parse into the documented typed AST\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn getting_started_quickstart_builds_and_runs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_dir = temp.path().join("adze_getting_started_quickstart");
    fs::create_dir_all(project_dir.join("src")).expect("create src");

    let repo_root = repo_root();
    let runtime_path = toml_path(repo_root.join("runtime"));
    let tool_path = toml_path(repo_root.join("tool"));
    let tutorial = include_str!("../../docs/tutorials/getting-started.md");
    let manifest_snippet = fenced_block_after(tutorial, "### Installation", "toml")
        .expect("Getting Started tutorial should include a TOML dependency block");
    let build_rs = fenced_block_after(tutorial, "Create `build.rs`", "rust")
        .expect("Getting Started tutorial should include a build.rs block");
    let lib_rs = fenced_block_after(tutorial, "Create `src/lib.rs`", "rust")
        .expect("Getting Started tutorial should include a src/lib.rs block");

    fs::write(
        project_dir.join("Cargo.toml"),
        tutorial_downstream_manifest(manifest_snippet, &runtime_path, &tool_path),
    )
    .expect("write Cargo.toml");

    fs::write(project_dir.join("build.rs"), build_rs).expect("write build.rs");
    fs::write(project_dir.join("src/lib.rs"), lib_rs).expect("write lib.rs");

    let output = Command::new("cargo")
        .arg("test")
        .arg("--quiet")
        .current_dir(&project_dir)
        .output()
        .expect("run cargo test in Getting Started quickstart crate");

    assert!(
        output.status.success(),
        "Getting Started quickstart crate should build and parse through the documented public API\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn readme_stable_claims_are_in_stable_product_lane() {
    let readme = include_str!("../../README.md");
    let support_tiers = include_str!("../../docs/status/SUPPORT_TIERS.md");
    let stable_lane = include_str!("../../scripts/ci-product-stable.sh");
    let proof_commands = readme_stable_proof_commands(readme);

    assert!(
        !proof_commands.is_empty(),
        "README capability table should include Stable proof commands"
    );

    for command in proof_commands {
        assert!(
            support_tiers.contains(&command),
            "README Stable proof command must be documented in docs/status/SUPPORT_TIERS.md:\n{command}"
        );

        if is_required_gate(&command) {
            continue;
        }

        assert!(
            stable_lane.contains(&command),
            "README Stable proof command must be present in scripts/ci-product-stable.sh:\n{command}"
        );
    }
}

fn tutorial_downstream_manifest(readme_toml: &str, runtime_path: &str, tool_path: &str) -> String {
    assert!(
        readme_toml.contains(r#"adze = { version = "0.8.0-dev", default-features = false }"#),
        "Getting Started install block should document the adze runtime dependency"
    );
    assert!(
        readme_toml.contains(r#"adze-tool = "0.8.0-dev""#),
        "Getting Started install block should document the adze-tool build dependency"
    );

    let dependencies = readme_toml
        .replace(
            r#"adze = { version = "0.8.0-dev", default-features = false }"#,
            &format!(r#"adze = {{ path = "{runtime_path}", default-features = false }}"#),
        )
        .replace(
            r#"adze-tool = "0.8.0-dev""#,
            &format!(r#"adze-tool = {{ path = "{tool_path}" }}"#),
        );

    format!(
        r#"[package]
name = "adze_getting_started_quickstart"
version = "0.1.0"
edition = "2024"

{dependencies}
"#
    )
}

fn downstream_manifest(readme_toml: &str, runtime_path: &str, tool_path: &str) -> String {
    assert!(
        readme_toml.contains(r#"adze = { version = "0.8", default-features = false }"#),
        "README install block should document the adze runtime dependency"
    );
    assert!(
        readme_toml.contains(r#"adze-tool = "0.8""#),
        "README install block should document the adze-tool build dependency"
    );

    let dependencies = readme_toml
        .replace(
            r#"adze = { version = "0.8", default-features = false }"#,
            &format!(r#"adze = {{ path = "{runtime_path}", default-features = false }}"#),
        )
        .replace(
            r#"adze-tool = "0.8""#,
            &format!(r#"adze-tool = {{ path = "{tool_path}" }}"#),
        );

    format!(
        r#"[package]
name = "adze_readme_quickstart"
version = "0.1.0"
edition = "2024"

{dependencies}
"#
    )
}

fn readme_stable_proof_commands(readme: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_capability_table = false;

    for line in readme.lines() {
        if line == "### Capability table" {
            in_capability_table = true;
            continue;
        }

        if in_capability_table && line.starts_with("##") {
            break;
        }

        if !in_capability_table {
            continue;
        }

        if !line.starts_with('|') || !line.contains("| **Stable** |") {
            continue;
        }

        let columns: Vec<&str> = line.split('|').collect();
        assert!(
            columns.len() >= 4,
            "README Stable capability row should have a proof column: {line}"
        );

        let proof = columns[3];
        let row_commands = inline_code_spans(proof);
        assert!(
            !row_commands.is_empty(),
            "README Stable capability row should name at least one proof command: {line}"
        );

        commands.extend(row_commands);
    }

    commands.sort();
    commands.dedup();
    commands
}

fn inline_code_spans(text: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };

        spans.push(after_start[..end].to_string());
        rest = &after_start[end + 1..];
    }

    spans
}

fn is_required_gate(command: &str) -> bool {
    matches!(command, "just ci-supported" | "CI / ci-supported")
}

fn grammar_module_from_readme(snippet: &str) -> String {
    let parser_call = "\nlet expr = grammar::parse";
    let grammar = snippet
        .split(parser_call)
        .next()
        .expect("README grammar snippet should have a grammar module before the parser call")
        .trim_end();

    format!("{grammar}\n")
}

fn fenced_block_after<'a>(text: &'a str, marker: &str, language: &str) -> Option<&'a str> {
    let start = text.find(marker)?;
    fenced_blocks(&text[start..], language).into_iter().next()
}

fn fenced_block_starting_with<'a>(text: &'a str, language: &str, prefix: &str) -> Option<&'a str> {
    fenced_blocks(text, language)
        .into_iter()
        .find(|block| block.trim_start().starts_with(prefix))
}

fn fenced_blocks<'a>(text: &'a str, language: &str) -> Vec<&'a str> {
    let fence = format!("```{language}");
    let mut blocks = Vec::new();
    let mut rest = text;

    while let Some(idx) = rest.find(&fence) {
        let after_fence = &rest[idx + fence.len()..];
        let Some(line_end) = after_fence.find('\n') else {
            break;
        };
        let body_start = idx + fence.len() + line_end + 1;
        let body = &rest[body_start..];
        let Some(body_end) = body.find("\n```") else {
            break;
        };
        blocks.push(body[..body_end].trim_end_matches('\r'));
        rest = &body[body_end + "\n```".len()..];
    }

    blocks
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli crate has workspace parent")
        .to_path_buf()
}

fn toml_path(path: PathBuf) -> String {
    path.display().to_string().replace('\\', "/")
}
