// Tool crate is mostly safe, with minimal unsafe for optimizations
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

//! Build tool for adze parser generation

use serde_json::Value;
use syn::{Item, parse_quote};

mod expansion;
use expansion::*;

mod grammar_converter;
pub use grammar_converter::GrammarConverter;

pub mod visualization;
pub use visualization::GrammarVisualizer;

pub mod grammar_js;
pub use grammar_js::{GrammarJsConverter, parse_grammar_js};

pub mod pure_rust_builder;
pub use pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_for_crate, build_parser_from_grammar_js,
};

pub mod cli;
pub mod scanner_build;

pub mod error;
pub use error::{Result as ToolResult, ToolError};

// Use tree-sitter-generate's version for compatibility
// Version 0.25.1 is what we depend on in Cargo.toml
const GENERATED_SEMANTIC_VERSION: Option<(u8, u8, u8)> = Some((0, 25, 1));

/// Generates JSON strings defining Tree Sitter grammars for every Adze
/// grammar found in the given module and recursive submodules.
pub fn generate_grammars(root_file: &Path) -> ToolResult<Vec<Value>> {
    let root_file = syn_inline_mod::parse_and_inline_modules(root_file).items;
    let mut out = vec![];
    for i in root_file.iter() {
        generate_all_grammars(i, &mut out)?;
    }
    Ok(out)
}

fn generate_all_grammars(item: &Item, out: &mut Vec<Value>) -> ToolResult<()> {
    if let Item::Mod(m) = item {
        if let Some((_, items)) = &m.content {
            for item in items {
                generate_all_grammars(item, out)?;
            }
        }

        if m.attrs
            .iter()
            .any(|a| a.path() == &parse_quote!(adze::grammar))
        {
            out.push(generate_grammar(m)?);
        }
    }
    Ok(())
}

#[cfg(feature = "build_parsers")]
use std::io::Write;
use std::path::Path;

#[cfg(feature = "build_parsers")]
use tree_sitter_generate::generate_parser_for_grammar;

#[cfg(feature = "build_parsers")]
/// Using the `cc` crate, generates and compiles a C parser with Tree Sitter
/// for every Adze grammar found in the given module and recursive
/// submodules.
pub fn build_parsers(root_file: &Path) {
    // Determine which builder to use - check both env vars for compatibility
    let use_pure_rust = std::env::var("CARGO_FEATURE_PURE_RUST").is_ok()
        || std::env::var("ADZE_USE_PURE_RUST").is_ok();

    // Debug to file to bypass any stderr capture issues
    {
        use std::io::Write;
        if let Ok(mut f) = std::fs::File::create("/tmp/adze_debug.txt") {
            writeln!(f, "build_parsers called for: {}", root_file.display()).ok();
            writeln!(
                f,
                "CARGO_FEATURE_PURE_RUST={:?}",
                std::env::var("CARGO_FEATURE_PURE_RUST")
            )
            .ok();
            writeln!(
                f,
                "ADZE_USE_PURE_RUST={:?}",
                std::env::var("ADZE_USE_PURE_RUST")
            )
            .ok();
            writeln!(f, "use_pure_rust={}", use_pure_rust).ok();
        }
    }

    if use_pure_rust {
        // Use pure-Rust builder exclusively
        use pure_rust_builder::{BuildOptions, build_parser_for_crate};
        let options = BuildOptions::default();
        match build_parser_for_crate(root_file, options) {
            Ok(results) => {
                for result in results {
                    println!("cargo:rerun-if-changed={}", result.parser_path);
                    println!("Built pure-Rust parser for {}", result.grammar_name);
                }
            }
            Err(e) => {
                eprintln!("Failed to build pure-Rust parser: {}", e);
                // Print the full error chain
                let mut source = e.source();
                while let Some(err) = source {
                    eprintln!("  Caused by: {}", err);
                    source = err.source();
                }
                panic!("FATAL: Pure-Rust parser generation failed: {:#}", e);
            }
        }
        // Critical: don't fall through to C generation
        return;
    }

    // If we get here, use C-based generation exclusively
    use std::env;
    let out_dir = env::var("OUT_DIR").unwrap();
    let emit_artifacts: bool = env::var("ADZE_EMIT_ARTIFACTS")
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);

    // Only check CLI if explicitly requested (for debugging)
    if std::env::var("ADZE_REQUIRE_TS_CLI").is_ok()
        && let Err(e) = std::process::Command::new("tree-sitter")
            .arg("--version")
            .output()
    {
        eprintln!("Warning: tree-sitter CLI not found or not executable");
        eprintln!("  Details: {}", e);
        eprintln!("  Hint: Install tree-sitter CLI >= 0.22 with: npm install -g tree-sitter-cli");
        eprintln!("  Then verify with: tree-sitter --version");
    }

    for grammar in generate_grammars(root_file).unwrap() {
        let grammar_str = serde_json::to_string(&grammar).unwrap();
        if emit_artifacts {
            eprintln!(
                "Generated grammar JSON:\n{}",
                serde_json::to_string_pretty(&grammar).unwrap()
            );
        }

        // Dump grammar JSON for debugging C-backend failures
        let dump_path = env::var("OUT_DIR")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("last_grammar.json"));
        if let Some(p) = &dump_path {
            let _ = std::fs::write(p, &grammar_str);
        }

        // Better error handling for C generation
        let (grammar_name, grammar_c) = match generate_parser_for_grammar(
            &grammar_str,
            GENERATED_SEMANTIC_VERSION,
        ) {
            Ok(result) => {
                // Also save a per-grammar copy for easier debugging
                if let Some(base_path) = &dump_path {
                    let named_path = base_path.with_file_name(format!("grammar_{}.json", result.0));
                    let _ = std::fs::write(named_path, &grammar_str);
                }
                result
            }
            Err(e) => {
                eprintln!("ERROR: Tree-sitter C generation failed for grammar");
                eprintln!("  Error: {}", e);
                eprintln!(
                    "  Hint: Ensure tree-sitter CLI >= 0.22 is on PATH (run `tree-sitter --version`)"
                );
                eprintln!("  Hint: Check that the grammar JSON is valid");
                if emit_artifacts {
                    eprintln!("  Debug: See generated grammar JSON above");
                }
                if let Some(p) = &dump_path {
                    eprintln!("  Debug: Wrote grammar JSON to {}", p.display());
                }
                panic!("C backend parser generation failed: {}", e);
            }
        };
        let tempfile = tempfile::Builder::new()
            .prefix("grammar")
            .tempdir()
            .unwrap();

        let dir = if emit_artifacts {
            let grammar_dir = Path::new(out_dir.as_str()).join(format!("grammar_{grammar_name}",));
            if grammar_dir.is_dir() {
                std::fs::remove_dir_all(&grammar_dir).expect("Couldn't clear old artifacts");
            }
            std::fs::DirBuilder::new()
                .recursive(true)
                .create(grammar_dir.clone())
                .expect("Couldn't create grammar JSON directory");
            grammar_dir
        } else {
            tempfile.path().into()
        };

        let grammar_file = dir.join("parser.c");
        let mut f = std::fs::File::create(grammar_file).unwrap();

        f.write_all(grammar_c.as_bytes()).unwrap();
        drop(f);

        // emit grammar into the build out_dir
        let mut grammar_json_file =
            std::fs::File::create(dir.join(format!("{grammar_name}.json"))).unwrap();
        grammar_json_file
            .write_all(serde_json::to_string_pretty(&grammar).unwrap().as_bytes())
            .unwrap();
        drop(grammar_json_file);

        let header_dir = dir.join("tree_sitter");
        std::fs::create_dir(&header_dir).unwrap();
        let mut parser_file = std::fs::File::create(header_dir.join("parser.h")).unwrap();
        parser_file
            .write_all(tree_sitter::PARSER_HEADER.as_bytes())
            .unwrap();
        drop(parser_file);

        let sysroot_dir = dir.join("sysroot");
        let target = env::var("TARGET").unwrap_or_else(|_| {
            // Fallback to the current target if TARGET is not set
            std::env::consts::ARCH.to_string() + "-" + std::env::consts::OS
        });
        if target.starts_with("wasm32") {
            std::fs::create_dir(&sysroot_dir).unwrap();
            let mut stdint = std::fs::File::create(sysroot_dir.join("stdint.h")).unwrap();
            stdint
                .write_all(include_bytes!("wasm-sysroot/stdint.h"))
                .unwrap();
            drop(stdint);

            let mut stdlib = std::fs::File::create(sysroot_dir.join("stdlib.h")).unwrap();
            stdlib
                .write_all(include_bytes!("wasm-sysroot/stdlib.h"))
                .unwrap();
            drop(stdlib);

            let mut stdio = std::fs::File::create(sysroot_dir.join("stdio.h")).unwrap();
            stdio
                .write_all(include_bytes!("wasm-sysroot/stdio.h"))
                .unwrap();
            drop(stdio);

            let mut stdbool = std::fs::File::create(sysroot_dir.join("stdbool.h")).unwrap();
            stdbool
                .write_all(include_bytes!("wasm-sysroot/stdbool.h"))
                .unwrap();
            drop(stdbool);
        }

        let mut c_config = cc::Build::new();
        c_config.std("c11").include(&dir).include(&sysroot_dir);

        // Cross-platform warning suppression
        c_config.warnings(false); // Portable way to disable warnings

        // Platform-specific optimizations
        if cfg!(target_env = "msvc") {
            c_config.flag_if_supported("/EHsc"); // Enable C++ exceptions for MSVC
        } else {
            c_config.flag_if_supported("-fno-exceptions"); // Disable exceptions for GCC/Clang
        }

        c_config.file(dir.join("parser.c"));

        // Check for optional scanner files in both generated dir and source root
        // Try generated dir first (tree-sitter CLI output)
        let scanner_paths = [
            (dir.join("scanner.c"), false),
            (dir.join("scanner.cc"), true),
            (dir.join("scanner.cpp"), true),
        ];

        // Also check source root and src/scanner subdir for manually written scanners
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let src_dir = Path::new(&manifest_dir).join("src");
            let scanner_subdir = src_dir.join("scanner");
            let additional_paths = [
                (src_dir.join("scanner.c"), false),
                (src_dir.join("scanner.cc"), true),
                (src_dir.join("scanner.cpp"), true),
                (scanner_subdir.join("scanner.c"), false),
                (scanner_subdir.join("scanner.cc"), true),
                (scanner_subdir.join("scanner.cpp"), true),
            ];

            // Check all paths, preferring generated over source
            for (path, is_cpp) in scanner_paths.iter().chain(additional_paths.iter()) {
                if path.exists() {
                    if *is_cpp {
                        c_config.cpp(true);
                    }
                    c_config.file(path);
                    break; // Use first scanner found
                }
            }
        } else {
            // No manifest dir, just check generated paths
            for (path, is_cpp) in scanner_paths.iter() {
                if path.exists() {
                    if *is_cpp {
                        c_config.cpp(true);
                    }
                    c_config.file(path);
                    break;
                }
            }
        }

        // Sanitize grammar name for library archive name
        let lib_name: String = grammar_name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        c_config.compile(&lib_name);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::{GENERATED_SEMANTIC_VERSION, generate_grammar};
    use tree_sitter_generate::generate_parser_for_grammar;

    #[test]
    fn enum_with_named_field() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Number(
                            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                            u32
                    ),
                    Neg {
                        #[adze::leaf(text = "!")]
                        _bang: (),
                        value: Box<Expr>,
                    }
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn enum_transformed_fields() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expression {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn enum_recursive() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expression {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                    Neg(
                        #[adze::leaf(text = "-", transform = |v| ())]
                        (),
                        Box<Expression>
                    ),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn enum_prec_left() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expression {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                    #[adze::prec_left(1)]
                    Sub(
                        Box<Expression>,
                        #[adze::leaf(text = "-", transform = |v| ())]
                        (),
                        Box<Expression>
                    ),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn grammar_with_extras() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expression {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                }

                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s", transform = |_v| ())]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn grammar_unboxed_field() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Language {
                    e: Expression,
                }

                pub enum Expression {
                    Number(
                        #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn grammar_repeat() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            pub mod grammar {
                #[adze::language]
                pub struct NumberList {
                    #[adze::repeat(non_empty = true)]
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    numbers: Vec<Number>,
                }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32,
                }

                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn grammar_repeat_no_delimiter() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            pub mod grammar {
                #[adze::language]
                pub struct NumberList {
                    #[adze::repeat(non_empty = true)]
                    numbers: Vec<Number>,
                }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32,
                }

                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn grammar_repeat1() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            pub mod grammar {
                #[adze::language]
                pub struct NumberList {
                    #[adze::repeat(non_empty = true)]
                    #[adze::delimited(
                        #[adze::leaf(text = ",")]
                        ()
                    )]
                    numbers: Vec<Number>,
                }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32,
                }

                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn struct_optional() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Language {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: Option<i32>,
                    #[adze::leaf(pattern = r" ", transform = |v| ())]
                    space: (),
                    t: Option<Number>,
                }

                pub struct Number {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn enum_with_unamed_vector() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                pub struct Number {
                        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                        value: u32
                }

                #[adze::language]
                pub enum Expr {
                    Numbers(
                        #[adze::repeat(non_empty = true)]
                        Vec<Number>
                    )
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    #[test]
    fn spanned_in_vec() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test")]
            mod grammar {
                use adze::Spanned;

                #[adze::language]
                pub struct NumberList {
                    #[adze::repeat(non_empty = true)]
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    numbers: Vec<Spanned<i32>>,
                }

                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(
            &serde_json::to_string(&grammar).unwrap(),
            GENERATED_SEMANTIC_VERSION,
        )
        .unwrap();
    }

    /// CRITICAL BUG REPRODUCTION: Test Binary variant generation with inlining
    /// https://github.com/EffortlessMetrics/adze/issues/BINARY_VARIANT_MISSING
    #[test]
    fn test_binary_variant_inlined_generation() {
        // This test reproduces the critical bug where Binary variant disappears
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[adze::grammar("test_binary")]
            pub mod grammar {
                #[adze::language]
                #[derive(Debug)]
                pub enum Expr {
                    Binary(
                        Box<Expr>,
                        #[adze::leaf(pattern = r"[-+*/]")] String,
                        Box<Expr>,
                    ),
                    Number(#[adze::leaf(pattern = r"\d+")] i32),
                }

                /// Whitespace handling - match real ambiguous_expr grammar
                #[adze::extra]
                struct Whitespace {
                    #[adze::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!("Failed to parse test module")
        };

        eprintln!("\n=== Testing Binary Variant Inlined Generation ===\n");

        let grammar = generate_grammar(&m).expect("Failed to generate grammar");
        eprintln!(
            "Generated grammar:\n{}",
            serde_json::to_string_pretty(&grammar).unwrap()
        );

        // Extract rules
        let rules = grammar.get("rules").expect("No rules in grammar");
        let rules_obj = rules.as_object().expect("Rules not an object");

        eprintln!("\n=== All Rules ===");
        for (name, _rule) in rules_obj {
            eprintln!("  - {}", name);
        }

        // Find the Expr rule
        let expr_rule = rules_obj.get("Expr").expect("No Expr rule found!");
        eprintln!(
            "\n=== Expr Rule ===\n{}",
            serde_json::to_string_pretty(expr_rule).unwrap()
        );

        // Expr should be a CHOICE
        let expr_type = expr_rule.get("type").and_then(serde_json::Value::as_str);
        assert_eq!(expr_type, Some("CHOICE"), "Expr should be a CHOICE");

        // Get CHOICE members
        let members = expr_rule.get("members").expect("No members in Expr CHOICE");
        let members_array = members.as_array().expect("Members not an array");

        eprintln!("\n=== Expr CHOICE Members ({}) ===", members_array.len());
        for (i, member) in members_array.iter().enumerate() {
            eprintln!(
                "Member {}:\n{}",
                i,
                serde_json::to_string_pretty(member).unwrap()
            );
        }

        // CRITICAL ASSERTION: Expr CHOICE should have 2 members (Binary + Number)
        assert_eq!(
            members_array.len(),
            2,
            "CONTRACT VIOLATION: Expr should have 2 CHOICE members (Binary + Number), found {}.\n\
             This indicates the Binary variant is missing from grammar generation!",
            members_array.len()
        );

        // Check first member (should be Binary - a SEQ with 3 fields)
        let binary_member = &members_array[0];
        let binary_type = binary_member
            .get("type")
            .and_then(serde_json::Value::as_str);

        assert_eq!(
            binary_type,
            Some("SEQ"),
            "Binary variant should be inlined as SEQ, got: {:?}",
            binary_type
        );

        // Binary SEQ should have 3 members (Expr, Op, Expr)
        let binary_members = binary_member
            .get("members")
            .expect("No members in Binary SEQ");
        let binary_members_array = binary_members
            .as_array()
            .expect("Binary members not an array");

        assert_eq!(
            binary_members_array.len(),
            3,
            "Binary SEQ should have 3 members (Expr, Op, Expr), found {}",
            binary_members_array.len()
        );

        // Check second member (should be Number - a PATTERN)
        let number_member = &members_array[1];
        let number_type = number_member
            .get("type")
            .and_then(serde_json::Value::as_str);

        assert_eq!(
            number_type,
            Some("PATTERN"),
            "Number variant should be inlined as PATTERN, got: {:?}",
            number_type
        );

        eprintln!("\n✅ TEST PASSED: Binary variant generates correctly!\n");
    }

    #[cfg(feature = "build_parsers")]
    #[test]
    fn test_emit_artifacts_functionality() {
        use std::env;
        use std::path::Path;

        // Set up test environment
        let original_target = env::var("TARGET").ok();
        let original_out_dir = env::var("OUT_DIR").ok();
        let original_emit = env::var("ADZE_EMIT_ARTIFACTS").ok();
        let original_opt_level = env::var("OPT_LEVEL").ok();
        let original_host = env::var("HOST").ok();
        let original_profile = env::var("PROFILE").ok();

        // Set required environment variables for the current platform
        let target = if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc"
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin"
        } else {
            "x86_64-unknown-linux-gnu"
        };

        unsafe {
            env::set_var("TARGET", target);
            env::set_var("OPT_LEVEL", "0");
            env::set_var("HOST", target);
            env::set_var("PROFILE", "debug");
            env::set_var("ADZE_EMIT_ARTIFACTS", "true");
        }

        let test_dir = "./test_emit_artifacts_output";
        std::fs::create_dir_all(test_dir).unwrap();
        unsafe {
            env::set_var("OUT_DIR", test_dir);
        }

        // Create a simple test grammar file
        let test_grammar = r#"
#[adze::grammar("test_emit")]
mod grammar {
    #[adze::language]
    pub enum Expression {
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
            i32
        ),
    }
}
"#;

        let grammar_file = "test_emit_grammar.rs";
        std::fs::write(grammar_file, test_grammar).unwrap();

        // Test that build_parsers doesn't panic with ADZE_EMIT_ARTIFACTS=true
        let result = std::panic::catch_unwind(|| {
            super::build_parsers(Path::new(grammar_file));
        });

        // Clean up
        let _ = std::fs::remove_file(grammar_file);
        let _ = std::fs::remove_dir_all(test_dir);

        // Restore original environment variables
        unsafe {
            match original_target {
                Some(val) => env::set_var("TARGET", val),
                None => env::remove_var("TARGET"),
            }
        }
        unsafe {
            match original_out_dir {
                Some(val) => env::set_var("OUT_DIR", val),
                None => env::remove_var("OUT_DIR"),
            }
        }
        unsafe {
            match original_emit {
                Some(val) => env::set_var("ADZE_EMIT_ARTIFACTS", val),
                None => env::remove_var("ADZE_EMIT_ARTIFACTS"),
            }
        }
        unsafe {
            match original_opt_level {
                Some(val) => env::set_var("OPT_LEVEL", val),
                None => env::remove_var("OPT_LEVEL"),
            }
        }
        unsafe {
            match original_host {
                Some(val) => env::set_var("HOST", val),
                None => env::remove_var("HOST"),
            }
        }
        unsafe {
            match original_profile {
                Some(val) => env::set_var("PROFILE", val),
                None => env::remove_var("PROFILE"),
            }
        }

        // Assert that the function completed successfully
        assert!(
            result.is_ok(),
            "build_parsers should not panic with ADZE_EMIT_ARTIFACTS=true"
        );
    }
}
