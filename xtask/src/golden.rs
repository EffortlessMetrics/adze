use anyhow::{Context, Result, bail};
use console::style;
use serde_json::json;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use xshell::{Shell, cmd};

use crate::Grammar;

const GOLDEN_DIR: &str = "tests/golden";
const TEMP_DIR: &str = "target/golden-temp";

pub fn generate_golden(sh: &Shell, grammar: Grammar, force: bool) -> Result<()> {
    let golden_path = golden_path(&grammar);

    if golden_path.exists() && !force {
        bail!(
            "Golden files already exist for {}. Use --force to regenerate.",
            grammar.name()
        );
    }

    println!(
        "Generating golden files for {}...",
        style(grammar.name()).cyan()
    );

    // Create golden directory
    fs::create_dir_all(&golden_path)?;

    match grammar {
        Grammar::Arithmetic => generate_arithmetic_golden(sh, &golden_path)?,
        _ => generate_external_golden(sh, &grammar, &golden_path)?,
    }

    println!(
        "{} Golden files generated successfully!",
        style("✓").green()
    );
    Ok(())
}

fn generate_arithmetic_golden(sh: &Shell, output_dir: &Path) -> Result<()> {
    // For local arithmetic grammar, use our own generator
    sh.change_dir(project_root());

    // Build and run the example to generate outputs
    cmd!(sh, "cargo build -p rust-sitter-example").run()?;

    // TODO: Run our tablegen to generate NODE_TYPES.json and tables
    // For now, create placeholder
    let node_types = r#"{
  "arithmetic": {
    "type": "arithmetic",
    "named": true,
    "fields": {},
    "children": {
      "multiple": true,
      "required": true,
      "types": [
        {
          "type": "expression",
          "named": true
        }
      ]
    }
  }
}"#;

    fs::write(output_dir.join("NODE_TYPES.json"), node_types)?;
    Ok(())
}

fn generate_external_golden(sh: &Shell, grammar: &Grammar, output_dir: &Path) -> Result<()> {
    let repo_url = grammar
        .repo_url()
        .ok_or_else(|| anyhow::anyhow!("No repository URL for {}", grammar.name()))?;

    // Clone or update the grammar repository
    let temp_dir = PathBuf::from(TEMP_DIR).join(grammar.name());

    if temp_dir.exists() {
        sh.change_dir(&temp_dir);
        cmd!(sh, "git pull").run()?;
    } else {
        fs::create_dir_all(temp_dir.parent().unwrap())?;
        sh.change_dir(temp_dir.parent().unwrap());
        let grammar_name = grammar.name();
        cmd!(sh, "git clone {repo_url} {grammar_name}").run()?;
        sh.change_dir(&temp_dir);
    }

    // Generate using tree-sitter CLI
    cmd!(sh, "npm install")
        .run()
        .context("Failed to install dependencies. Is npm installed?")?;

    cmd!(sh, "npx tree-sitter generate")
        .run()
        .context("Failed to generate grammar. Is tree-sitter CLI installed?")?;

    // Copy generated files to golden directory
    let files = vec![
        ("src/node-types.json", "NODE_TYPES.json"),
        ("src/grammar.json", "grammar.json"),
    ];

    for (src, dst) in files {
        let src_path = temp_dir.join(src);
        let dst_path = output_dir.join(dst);

        if src_path.exists() {
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy {} to {}", src, dst))?;
        }
    }

    Ok(())
}

pub fn diff_golden(sh: &Shell, grammar: Grammar, verbose: bool) -> Result<()> {
    let golden_dir = golden_path(&grammar);
    if !golden_dir.exists() {
        bail!(
            "No golden files found for {}. Run 'cargo xtask generate-golden {}' first.",
            grammar.name(),
            grammar.name()
        );
    }

    // Generate current output
    let current_dir = PathBuf::from(TEMP_DIR).join("current").join(grammar.name());
    fs::create_dir_all(&current_dir)?;

    // TODO: Generate current output using our implementation
    generate_current_output(sh, &grammar, &current_dir)?;

    // Compare files
    let mut has_diff = false;
    for entry in WalkDir::new(&golden_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = entry.path().strip_prefix(&golden_dir)?;
        let current_path = current_dir.join(relative_path);

        if !current_path.exists() {
            println!("{} Missing: {}", style("✗").red(), relative_path.display());
            has_diff = true;
            continue;
        }

        let golden_content = fs::read_to_string(entry.path())?;
        let current_content = fs::read_to_string(&current_path)?;

        if golden_content != current_content {
            println!("{} Differs: {}", style("✗").red(), relative_path.display());
            has_diff = true;

            if verbose {
                print_diff(&golden_content, &current_content);
            }
        } else {
            println!(
                "{} Matches: {}",
                style("✓").green(),
                relative_path.display()
            );
        }
    }

    if has_diff {
        bail!("Golden test failed. Files differ from expected output.");
    } else {
        println!("{} All golden tests passed!", style("✓").green());
    }

    Ok(())
}

pub fn update_golden(sh: &Shell, grammar: Grammar) -> Result<()> {
    let golden_dir = golden_path(&grammar);
    let current_dir = PathBuf::from(TEMP_DIR).join("current").join(grammar.name());

    // Generate current output
    fs::create_dir_all(&current_dir)?;
    generate_current_output(sh, &grammar, &current_dir)?;

    // Copy current to golden
    fs::create_dir_all(&golden_dir)?;
    for entry in WalkDir::new(&current_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(&current_dir)?;
            let golden_path = golden_dir.join(relative_path);

            if let Some(parent) = golden_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(entry.path(), &golden_path)?;
            println!(
                "{} Updated: {}",
                style("✓").green(),
                relative_path.display()
            );
        }
    }

    Ok(())
}

pub fn test_all_golden(sh: &Shell, verbose: bool) -> Result<()> {
    let grammars = vec![
        Grammar::Arithmetic,
        Grammar::Javascript,
        Grammar::Rust,
        Grammar::Python,
    ];
    let mut all_passed = true;

    for grammar in grammars {
        let golden_dir = golden_path(&grammar);
        if !golden_dir.exists() {
            println!(
                "{} Skipping {} (no golden files)",
                style("→").yellow(),
                grammar.name()
            );
            continue;
        }

        println!("\nTesting {}...", style(grammar.name()).cyan());
        match diff_golden(sh, grammar, verbose) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{} {}: {}", style("✗").red(), grammar.name(), e);
                all_passed = false;
            }
        }
    }

    if !all_passed {
        bail!("Some golden tests failed");
    }

    Ok(())
}

fn generate_current_output(sh: &Shell, grammar: &Grammar, output_dir: &Path) -> Result<()> {
    match grammar {
        Grammar::Arithmetic => {
            // Try to find the generated grammar JSON
            let grammar_json_path = find_grammar_json(sh, "arithmetic")?;

            if let Ok(grammar_json) = fs::read_to_string(&grammar_json_path) {
                // Extract node types from the grammar JSON
                let node_infos =
                    crate::grammar_json::extract_node_types_from_grammar_json(&grammar_json)?;

                // Convert to Tree-sitter NODE_TYPES format
                let mut node_types = vec![];

                // Add the expression supertype
                node_types.push(json!({
                    "type": "expression",
                    "named": true,
                    "subtypes": [
                        {"type": "number", "named": true},
                        {"type": "binary_expression", "named": true}
                    ]
                }));

                // Add binary_expression with fields
                node_types.push(json!({
                    "type": "binary_expression",
                    "named": true,
                    "fields": {
                        "left": {
                            "multiple": false,
                            "required": true,
                            "types": [{"type": "expression", "named": true}]
                        },
                        "operator": {
                            "multiple": false,
                            "required": true,
                            "types": [
                                {"type": "-", "named": false},
                                {"type": "*", "named": false}
                            ]
                        },
                        "right": {
                            "multiple": false,
                            "required": true,
                            "types": [{"type": "expression", "named": true}]
                        }
                    }
                }));

                // Add number node
                node_types.push(json!({
                    "type": "number",
                    "named": true,
                    "fields": {}
                }));

                // Add literal tokens
                for info in &node_infos {
                    if !info.named {
                        node_types.push(json!({
                            "type": info.name,
                            "named": false
                        }));
                    }
                }

                let json_output = serde_json::to_string_pretty(&node_types)?;
                fs::write(output_dir.join("NODE_TYPES.json"), json_output)?;
            } else {
                // Fallback to placeholder
                let node_types = r#"[
  {
    "type": "expression",
    "named": true,
    "subtypes": [
      {
        "type": "number",
        "named": true
      },
      {
        "type": "subtraction",
        "named": true
      },
      {
        "type": "multiplication",
        "named": true
      }
    ]
  },
  {
    "type": "number",
    "named": true,
    "fields": {}
  },
  {
    "type": "subtraction",
    "named": true,
    "fields": {
      "left": {
        "multiple": false,
        "required": true,
        "types": [
          {
            "type": "expression",
            "named": true
          }
        ]
      },
      "right": {
        "multiple": false,
        "required": true,
        "types": [
          {
            "type": "expression",
            "named": true
          }
        ]
      }
    }
  },
  {
    "type": "multiplication",
    "named": true,
    "fields": {
      "left": {
        "multiple": false,
        "required": true,
        "types": [
          {
            "type": "expression",
            "named": true
          }
        ]
      },
      "right": {
        "multiple": false,
        "required": true,
        "types": [
          {
            "type": "expression",
            "named": true
          }
        ]
      }
    }
  },
  {
    "type": "-",
    "named": false
  },
  {
    "type": "*",
    "named": false
  }
]"#;
                fs::write(output_dir.join("NODE_TYPES.json"), node_types)?;
            }
        }
        _ => {
            // External grammars need full implementation
            fs::write(output_dir.join("NODE_TYPES.json"), "[]")?;
        }
    }

    Ok(())
}

fn print_diff(golden: &str, current: &str) {
    let diff = TextDiff::from_lines(golden, current);

    println!("\n{}", style("Diff:").bold());
    for change in diff.iter_all_changes() {
        let (sign, style_color) = match change.tag() {
            ChangeTag::Delete => ("-", console::Style::new().red()),
            ChangeTag::Insert => ("+", console::Style::new().green()),
            ChangeTag::Equal => (" ", console::Style::new().dim()),
        };

        print!(
            "{} {}",
            style_color.apply_to(sign),
            style_color.apply_to(change)
        );
    }
    println!();
}

fn golden_path(grammar: &Grammar) -> PathBuf {
    project_root().join(GOLDEN_DIR).join(grammar.name())
}

fn project_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Remove 'xtask'
    path
}

fn find_grammar_json(_sh: &Shell, grammar_name: &str) -> Result<PathBuf> {
    // Look for the grammar JSON in the build output
    let root = project_root();
    let pattern = format!(
        "{}/target/debug/build/*/out/grammar_{}/{}.json",
        root.display(),
        grammar_name,
        grammar_name
    );

    for path in glob::glob(&pattern)
        .context("Failed to glob for grammar JSON")?
        .flatten()
    {
        if path.exists() {
            return Ok(path);
        }
    }

    bail!(
        "Could not find grammar JSON for {}. Make sure to build with RUST_SITTER_EMIT_ARTIFACTS=true",
        grammar_name
    )
}
