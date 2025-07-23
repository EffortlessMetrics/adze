use serde_json::Value;
use syn::{parse_quote, Item};

mod expansion;
use expansion::*;

mod grammar_converter;
pub use grammar_converter::GrammarConverter;

pub mod visualization;
pub use visualization::GrammarVisualizer;

pub mod grammar_js;
pub use grammar_js::{parse_grammar_js, GrammarJsConverter};

pub mod pure_rust_builder;
pub use pure_rust_builder::{build_parser, build_parser_for_crate, build_parser_from_grammar_js, BuildOptions, BuildResult};

pub mod cli;
pub mod scanner_build;

const GENERATED_SEMANTIC_VERSION: Option<(u8, u8, u8)> = Some((0, 25, 2));

/// Generates JSON strings defining Tree Sitter grammars for every Rust Sitter
/// grammar found in the given module and recursive submodules.
pub fn generate_grammars(root_file: &Path) -> Vec<Value> {
    let root_file = syn_inline_mod::parse_and_inline_modules(root_file).items;
    let mut out = vec![];
    root_file
        .iter()
        .for_each(|i| generate_all_grammars(i, &mut out));
    out
}

fn generate_all_grammars(item: &Item, out: &mut Vec<Value>) {
    if let Item::Mod(m) = item {
        m.content
            .iter()
            .for_each(|(_, items)| items.iter().for_each(|i| generate_all_grammars(i, out)));

        if m.attrs
            .iter()
            .any(|a| a.path() == &parse_quote!(rust_sitter::grammar))
        {
            out.push(generate_grammar(m))
        }
    }
}

#[cfg(feature = "build_parsers")]
use std::io::Write;
use std::path::Path;

#[cfg(feature = "build_parsers")]
use tree_sitter_generate::generate_parser_for_grammar;

#[cfg(feature = "build_parsers")]
/// Using the `cc` crate, generates and compiles a C parser with Tree Sitter
/// for every Rust Sitter grammar found in the given module and recursive
/// submodules.
pub fn build_parsers(root_file: &Path) {
    // Check if we should use the new pure-Rust builder
    if std::env::var("RUST_SITTER_USE_PURE_RUST").is_ok() {
        use pure_rust_builder::{build_parser_for_crate, BuildOptions};
        let options = BuildOptions::default();
        match build_parser_for_crate(root_file, options) {
            Ok(results) => {
                for result in results {
                    println!("cargo:rerun-if-changed={}", result.parser_path);
                    println!("Built pure-Rust parser for {}", result.grammar_name);
                }
                return;
            }
            Err(e) => {
                eprintln!("Failed to build pure-Rust parser: {}", e);
                eprintln!("Falling back to C parser generation");
            }
        }
    }
    use std::env;
    let out_dir = env::var("OUT_DIR").unwrap();
    let emit_artifacts: bool = env::var("RUST_SITTER_EMIT_ARTIFACTS")
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);
    generate_grammars(root_file).iter().for_each(|grammar| {
        let (grammar_name, grammar_c) =
            generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
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
            .write_all(serde_json::to_string_pretty(grammar).unwrap().as_bytes())
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
        c_config
            .flag_if_supported("-Wno-unused-label")
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-unused-but-set-variable")
            .flag_if_supported("-Wno-trigraphs")
            .flag_if_supported("-Wno-everything");
        c_config.file(dir.join("parser.c"));

        c_config.compile(&grammar_name);
    });
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::{generate_grammar, GENERATED_SEMANTIC_VERSION};
    use tree_sitter_generate::generate_parser_for_grammar;

    #[test]
    fn enum_with_named_field() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                #[rust_sitter::language]
                pub enum Expr {
                    Number(
                            #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                            u32
                    ),
                    Neg {
                        #[rust_sitter::leaf(text = "!")]
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

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn enum_transformed_fields() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                #[rust_sitter::language]
                pub enum Expression {
                    Number(
                        #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn enum_recursive() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                #[rust_sitter::language]
                pub enum Expression {
                    Number(
                        #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                    Neg(
                        #[rust_sitter::leaf(text = "-", transform = |v| ())]
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

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn enum_prec_left() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                #[rust_sitter::language]
                pub enum Expression {
                    Number(
                        #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                    #[rust_sitter::prec_left(1)]
                    Sub(
                        Box<Expression>,
                        #[rust_sitter::leaf(text = "-", transform = |v| ())]
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

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn grammar_with_extras() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                #[rust_sitter::language]
                pub enum Expression {
                    Number(
                        #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                }

                #[rust_sitter::extra]
                struct Whitespace {
                    #[rust_sitter::leaf(pattern = r"\s", transform = |_v| ())]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn grammar_unboxed_field() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                #[rust_sitter::language]
                pub struct Language {
                    e: Expression,
                }

                pub enum Expression {
                    Number(
                        #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                        i32
                    ),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn grammar_repeat() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            pub mod grammar {
                #[rust_sitter::language]
                pub struct NumberList {
                    #[rust_sitter::delimited(
                        #[rust_sitter::leaf(text = ",")]
                        ()
                    )]
                    numbers: Vec<Number>,
                }

                pub struct Number {
                    #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32,
                }

                #[rust_sitter::extra]
                struct Whitespace {
                    #[rust_sitter::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn grammar_repeat_no_delimiter() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            pub mod grammar {
                #[rust_sitter::language]
                pub struct NumberList {
                    numbers: Vec<Number>,
                }

                pub struct Number {
                    #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32,
                }

                #[rust_sitter::extra]
                struct Whitespace {
                    #[rust_sitter::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn grammar_repeat1() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            pub mod grammar {
                #[rust_sitter::language]
                pub struct NumberList {
                    #[rust_sitter::repeat(non_empty = true)]
                    #[rust_sitter::delimited(
                        #[rust_sitter::leaf(text = ",")]
                        ()
                    )]
                    numbers: Vec<Number>,
                }

                pub struct Number {
                    #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32,
                }

                #[rust_sitter::extra]
                struct Whitespace {
                    #[rust_sitter::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn struct_optional() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                #[rust_sitter::language]
                pub struct Language {
                    #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: Option<i32>,
                    #[rust_sitter::leaf(pattern = r" ", transform = |v| ())]
                    space: (),
                    t: Option<Number>,
                }

                pub struct Number {
                    #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    v: i32
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn enum_with_unamed_vector() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                pub struct Number {
                        #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                        value: u32
                }

                #[rust_sitter::language]
                pub enum Expr {
                    Numbers(
                        #[rust_sitter::repeat(non_empty = true)]
                        Vec<Number>
                    )
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[test]
    fn spanned_in_vec() {
        let m = if let syn::Item::Mod(m) = parse_quote! {
            #[rust_sitter::grammar("test")]
            mod grammar {
                use rust_sitter::Spanned;

                #[rust_sitter::language]
                pub struct NumberList {
                    #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    numbers: Vec<Spanned<i32>>,
                }

                #[rust_sitter::extra]
                struct Whitespace {
                    #[rust_sitter::leaf(pattern = r"\s")]
                    _whitespace: (),
                }
            }
        } {
            m
        } else {
            panic!()
        };

        let grammar = generate_grammar(&m);
        insta::assert_snapshot!(grammar);
        generate_parser_for_grammar(&grammar.to_string(), GENERATED_SEMANTIC_VERSION).unwrap();
    }

    #[cfg(feature = "build_parsers")]
    #[test]
    fn test_emit_artifacts_functionality() {
        use std::env;
        use std::path::Path;
        
        // Set up test environment
        let original_target = env::var("TARGET").ok();
        let original_out_dir = env::var("OUT_DIR").ok();
        let original_emit = env::var("RUST_SITTER_EMIT_ARTIFACTS").ok();
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
            env::set_var("RUST_SITTER_EMIT_ARTIFACTS", "true");
        }
        
        let test_dir = "./test_emit_artifacts_output";
        std::fs::create_dir_all(test_dir).unwrap();
        unsafe {
            env::set_var("OUT_DIR", test_dir);
        }
        
        // Create a simple test grammar file
        let test_grammar = r#"
#[rust_sitter::grammar("test_emit")]
mod grammar {
    #[rust_sitter::language]
    pub enum Expression {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
            i32
        ),
    }
}
"#;
        
        let grammar_file = "test_emit_grammar.rs";
        std::fs::write(grammar_file, test_grammar).unwrap();
        
        // Test that build_parsers doesn't panic with RUST_SITTER_EMIT_ARTIFACTS=true
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
                Some(val) => env::set_var("RUST_SITTER_EMIT_ARTIFACTS", val),
                None => env::remove_var("RUST_SITTER_EMIT_ARTIFACTS"),
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
        assert!(result.is_ok(), "build_parsers should not panic with RUST_SITTER_EMIT_ARTIFACTS=true");
    }
}