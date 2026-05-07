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

    fs::write(
        project_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "adze_readme_quickstart"
version = "0.1.0"
edition = "2024"

[dependencies]
adze = {{ path = "{runtime_path}", default-features = false }}

[build-dependencies]
adze-tool = {{ path = "{tool_path}" }}

[features]
default = ["pure-rust"]
pure-rust = ["adze/pure-rust"]
"#
        ),
    )
    .expect("write Cargo.toml");

    fs::write(
        project_dir.join("build.rs"),
        r#"use std::path::PathBuf;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(adze_unsafe_attrs)");
    adze_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
"#,
    )
    .expect("write build.rs");

    fs::write(
        project_dir.join("src/lib.rs"),
        r#"#[adze::grammar("arithmetic")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug, PartialEq, Eq)]
    pub enum Expr {
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
            i32,
        ),

        #[adze::prec_left(1)]
        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),

        #[adze::prec_left(2)]
        Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s+")]
        _ws: (),
    }
}
"#,
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

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli crate has workspace parent")
        .to_path_buf()
}

fn toml_path(path: PathBuf) -> String {
    path.display().to_string().replace('\\', "/")
}
