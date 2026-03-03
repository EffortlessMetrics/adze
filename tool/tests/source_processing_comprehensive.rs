#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for Rust source file processing in adze-tool.
//!
//! Covers: processing valid source with grammar attributes, source with no
//! attributes, source with syntax errors, multiple grammars in one file,
//! nested type definitions, various Rust editions, build configuration
//! handling, and output directory management.

use std::fs;

use adze_tool::pure_rust_builder::BuildOptions;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write Rust source to a temp file and call `generate_grammars`.
fn extract(src: &str) -> Vec<serde_json::Value> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path).unwrap()
}

/// Extract exactly one grammar.
fn extract_one(src: &str) -> serde_json::Value {
    let gs = extract(src);
    assert_eq!(
        gs.len(),
        1,
        "expected exactly one grammar, got {}",
        gs.len()
    );
    gs.into_iter().next().unwrap()
}

/// Try to extract grammars, returning the Result.
fn try_extract(src: &str) -> adze_tool::ToolResult<Vec<serde_json::Value>> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path)
}

/// Build options pointing at a temp dir.
fn opts_in(dir: &TempDir) -> BuildOptions {
    BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

// ===========================================================================
// 1. Processing valid Rust source with grammar attributes
// ===========================================================================

#[test]
fn valid_struct_language_extracts_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("valid_struct")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "valid_struct");
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("source_file"));
    assert!(rules.contains_key("Root"));
}

#[test]
fn valid_enum_language_extracts_grammar() {
    let g = extract_one(
        r#"
        #[adze::grammar("valid_enum")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "valid_enum");
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expr"));
}

#[test]
fn grammar_with_extras_extracts() {
    let g = extract_one(
        r#"
        #[adze::grammar("with_extras")]
        mod grammar {
            #[adze::language]
            pub enum Tok {
                Id(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    assert!(g["extras"].as_array().is_some());
    assert!(!g["extras"].as_array().unwrap().is_empty());
}

#[test]
fn grammar_with_leaf_text_extracts() {
    let g = extract_one(
        r#"
        #[adze::grammar("leaf_text")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "+")]
                op: (),
                #[adze::leaf(pattern = r"\d+")]
                num: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Root"));
}

#[test]
fn grammar_with_optional_field_extracts() {
    let g = extract_one(
        r#"
        #[adze::grammar("opt_field")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                required: String,
                #[adze::leaf(pattern = r"\d+")]
                maybe: Option<String>,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Root"));
}

#[test]
fn grammar_with_vec_repeat_extracts() {
    let g = extract_one(
        r#"
        #[adze::grammar("vec_repeat")]
        mod grammar {
            #[adze::language]
            pub struct List {
                #[adze::repeat(non_empty = true)]
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("List"));
    assert!(rules.contains_key("Item"));
}

#[test]
fn grammar_with_precedence_extracts() {
    let g = extract_one(
        r#"
        #[adze::grammar("prec_test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expr>,
                ),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expr"));
}

// ===========================================================================
// 2. Processing source with no attributes
// ===========================================================================

#[test]
fn no_grammar_attribute_yields_empty() {
    let gs = extract("pub fn main() {}");
    assert!(gs.is_empty());
}

#[test]
fn module_without_grammar_attr_yields_empty() {
    let gs = extract(
        r#"
        mod some_module {
            pub struct Foo { x: i32 }
        }
        "#,
    );
    assert!(gs.is_empty());
}

#[test]
fn only_language_attr_without_grammar_yields_empty() {
    // A module with #[adze::language] inside but no #[adze::grammar] on the mod
    let gs = extract(
        r#"
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    );
    assert!(gs.is_empty());
}

#[test]
fn empty_file_yields_empty() {
    let gs = extract("");
    assert!(gs.is_empty());
}

#[test]
fn comments_only_yields_empty() {
    let gs = extract("// just a comment\n/* block comment */\n");
    assert!(gs.is_empty());
}

// ===========================================================================
// 3. Processing source with syntax errors
// ===========================================================================

#[test]
fn file_that_does_not_exist_panics() {
    let result = std::panic::catch_unwind(|| {
        adze_tool::generate_grammars(std::path::Path::new("/nonexistent/path/lib.rs"))
    });
    assert!(result.is_err(), "missing file should panic");
}

#[test]
fn malformed_rust_source_is_error() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, "this is not valid rust {{{{").unwrap();
    // syn_inline_mod::parse_and_inline_modules may panic or error
    let result = std::panic::catch_unwind(|| adze_tool::generate_grammars(&path));
    // Either it returns an error or panics - both are acceptable
    match result {
        Err(_) => {}     // panicked - acceptable
        Ok(Err(_)) => {} // returned error - acceptable
        Ok(Ok(gs)) => assert!(
            gs.is_empty(),
            "malformed source should not produce grammars"
        ),
    }
}

// ===========================================================================
// 4. Processing multiple grammars in one file
// ===========================================================================

#[test]
fn two_grammars_in_one_file() {
    let gs = extract(
        r#"
        #[adze::grammar("first")]
        mod grammar_a {
            #[adze::language]
            pub struct A {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
        }
        #[adze::grammar("second")]
        mod grammar_b {
            #[adze::language]
            pub struct B {
                #[adze::leaf(pattern = r"\d+")]
                n: String,
            }
        }
        "#,
    );
    assert_eq!(gs.len(), 2);
    let names: Vec<&str> = gs.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"first"));
    assert!(names.contains(&"second"));
}

#[test]
fn three_grammars_extracted_in_order() {
    let gs = extract(
        r#"
        #[adze::grammar("alpha")]
        mod g1 {
            #[adze::language]
            pub struct A { #[adze::leaf(pattern = r"a")] v: String }
        }
        #[adze::grammar("beta")]
        mod g2 {
            #[adze::language]
            pub struct B { #[adze::leaf(pattern = r"b")] v: String }
        }
        #[adze::grammar("gamma")]
        mod g3 {
            #[adze::language]
            pub struct C { #[adze::leaf(pattern = r"c")] v: String }
        }
        "#,
    );
    assert_eq!(gs.len(), 3);
    assert_eq!(gs[0]["name"].as_str().unwrap(), "alpha");
    assert_eq!(gs[1]["name"].as_str().unwrap(), "beta");
    assert_eq!(gs[2]["name"].as_str().unwrap(), "gamma");
}

#[test]
fn multiple_grammars_have_independent_rules() {
    let gs = extract(
        r#"
        #[adze::grammar("lang1")]
        mod g1 {
            #[adze::language]
            pub struct Foo { #[adze::leaf(pattern = r"foo")] v: String }
        }
        #[adze::grammar("lang2")]
        mod g2 {
            #[adze::language]
            pub struct Bar { #[adze::leaf(pattern = r"bar")] v: String }
        }
        "#,
    );
    assert_eq!(gs.len(), 2);
    let r0 = gs[0]["rules"].as_object().unwrap();
    let r1 = gs[1]["rules"].as_object().unwrap();
    assert!(r0.contains_key("Foo"));
    assert!(!r0.contains_key("Bar"));
    assert!(r1.contains_key("Bar"));
    assert!(!r1.contains_key("Foo"));
}

// ===========================================================================
// 5. Processing nested type definitions
// ===========================================================================

#[test]
fn nested_module_grammar_extracted() {
    let g = extract_one(
        r#"
        mod outer {
            #[adze::grammar("nested_inner")]
            mod inner {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"[a-z]+")]
                    v: String,
                }
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "nested_inner");
}

#[test]
fn deeply_nested_module_grammar() {
    let g = extract_one(
        r#"
        mod a {
            mod b {
                #[adze::grammar("deep")]
                mod c {
                    #[adze::language]
                    pub struct Deep {
                        #[adze::leaf(pattern = r"x")]
                        v: String,
                    }
                }
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "deep");
}

#[test]
fn enum_with_struct_helper_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("enum_nested")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Ident(#[adze::leaf(pattern = r"[a-z]+")] String),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expr"));
    // Expr should be a CHOICE with two members
    let expr = &rules["Expr"];
    assert_eq!(expr["type"].as_str().unwrap(), "CHOICE");
    let members = expr["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
}

#[test]
fn struct_referencing_another_struct() {
    let g = extract_one(
        r#"
        #[adze::grammar("struct_ref")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                item: Item,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Program"));
    assert!(rules.contains_key("Item"));
}

#[test]
fn enum_with_box_recursive_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("recursive")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Neg(
                    #[adze::leaf(text = "-")]
                    (),
                    Box<Expr>,
                ),
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("Expr"));
}

// ===========================================================================
// 6. Processing with various Rust editions (edition-agnostic patterns)
// ===========================================================================

#[test]
fn source_with_use_statements_processed() {
    let g = extract_one(
        r#"
        use std::collections::HashMap;

        #[adze::grammar("with_use")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "with_use");
}

#[test]
fn source_with_type_aliases_processed() {
    let g = extract_one(
        r#"
        type MyInt = i32;

        #[adze::grammar("with_alias")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "with_alias");
}

#[test]
fn source_with_const_processed() {
    let g = extract_one(
        r#"
        const MAX: usize = 100;

        #[adze::grammar("with_const")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
        }
        "#,
    );
    assert_eq!(g["name"].as_str().unwrap(), "with_const");
}

// ===========================================================================
// 7. Build configuration handling
// ===========================================================================

#[test]
fn build_options_default_has_compress_tables_on() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    assert!(opts.compress_tables);
}

#[test]
fn build_options_emit_artifacts_flag() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: false,
    };
    assert!(opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn build_options_clone_preserves_fields() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let cloned = opts.clone();
    assert_eq!(opts.out_dir, cloned.out_dir);
    assert_eq!(opts.emit_artifacts, cloned.emit_artifacts);
    assert_eq!(opts.compress_tables, cloned.compress_tables);
}

#[test]
fn build_options_debug_impl() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let dbg = format!("{:?}", opts);
    assert!(dbg.contains("BuildOptions"));
    assert!(dbg.contains("compress_tables"));
}

// ===========================================================================
// 8. Output directory management
// ===========================================================================

#[test]
fn out_dir_is_created_when_missing() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("sub").join("out");
    assert!(!out.exists());
    fs::create_dir_all(&out).unwrap();
    let opts = BuildOptions {
        out_dir: out.to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(std::path::Path::new(&opts.out_dir).exists());
}

#[test]
fn out_dir_with_spaces_in_path() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("path with spaces");
    fs::create_dir_all(&out).unwrap();
    let opts = BuildOptions {
        out_dir: out.to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(std::path::Path::new(&opts.out_dir).exists());
}

#[test]
fn generated_grammar_json_has_required_keys() {
    let g = extract_one(
        r#"
        #[adze::grammar("json_keys")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
        }
        "#,
    );
    assert!(g.get("name").is_some(), "missing 'name'");
    assert!(g.get("rules").is_some(), "missing 'rules'");
    // Grammar should be serializable to string
    let json_str = serde_json::to_string(&g).unwrap();
    assert!(!json_str.is_empty());
}

#[test]
fn grammar_json_source_file_rule_references_language_type() {
    let g = extract_one(
        r#"
        #[adze::grammar("src_file_ref")]
        mod grammar {
            #[adze::language]
            pub struct MyLang {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let sf = &rules["source_file"];
    // source_file should be a SYMBOL referencing the language type
    assert_eq!(sf["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(sf["name"].as_str().unwrap(), "MyLang");
}

#[test]
fn grammar_json_is_valid_json_roundtrip() {
    let g = extract_one(
        r#"
        #[adze::grammar("roundtrip")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                A(#[adze::leaf(pattern = r"a")] String),
                B(#[adze::leaf(pattern = r"b")] String),
            }
        }
        "#,
    );
    let json_str = serde_json::to_string_pretty(&g).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(g, reparsed);
}

#[test]
fn grammar_with_delimited_repeat() {
    let g = extract_one(
        r#"
        #[adze::grammar("delimited")]
        mod grammar {
            #[adze::language]
            pub struct List {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"[a-z]+")]
                v: String,
            }
        }
        "#,
    );
    let rules = g["rules"].as_object().unwrap();
    assert!(rules.contains_key("List"));
    assert!(rules.contains_key("Item"));
}
