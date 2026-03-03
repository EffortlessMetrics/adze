#![allow(clippy::needless_range_loop)]

//! Edge-case tests for grammar module handling in adze-macro.
//!
//! Exercises boundary conditions, unusual but valid module structures,
//! naming conventions, visibility modifiers, doc comments, deeply nested
//! types, and combinations of all annotation types.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemMod, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn extract_grammar_name(m: &ItemMod) -> Option<String> {
    m.attrs.iter().find_map(|a| {
        if !is_adze_attr(a, "grammar") {
            return None;
        }
        let expr: syn::Expr = a.parse_args().ok()?;
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = expr
        {
            Some(s.value())
        } else {
            None
        }
    })
}

fn find_language_type(m: &ItemMod) -> Option<String> {
    module_items(m).iter().find_map(|item| match item {
        Item::Enum(e) if e.attrs.iter().any(|a| is_adze_attr(a, "language")) => {
            Some(e.ident.to_string())
        }
        Item::Struct(s) if s.attrs.iter().any(|a| is_adze_attr(a, "language")) => {
            Some(s.ident.to_string())
        }
        _ => None,
    })
}

fn count_structs(m: &ItemMod) -> usize {
    module_items(m)
        .iter()
        .filter(|i| matches!(i, Item::Struct(_)))
        .count()
}

fn count_enums(m: &ItemMod) -> usize {
    module_items(m)
        .iter()
        .filter(|i| matches!(i, Item::Enum(_)))
        .count()
}

// ── 1. Empty grammar module ─────────────────────────────────────────────────

#[test]
fn edge_empty_grammar_module_has_no_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("empty")]
        mod grammar {}
    });
    assert!(module_items(&m).is_empty());
    assert_eq!(extract_grammar_name(&m), Some("empty".to_string()));
    assert_eq!(find_language_type(&m), None);
}

// ── 2. Grammar with single struct (no enum) ─────────────────────────────────

#[test]
fn edge_single_struct_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("single")]
        mod grammar {
            #[adze::language]
            pub struct OnlyNode {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }
        }
    });
    assert_eq!(count_structs(&m), 1);
    assert_eq!(count_enums(&m), 0);
    assert_eq!(find_language_type(&m), Some("OnlyNode".to_string()));
}

// ── 3. Grammar with struct + enum ───────────────────────────────────────────

#[test]
fn edge_struct_and_enum_together() {
    let m = parse_mod(quote! {
        #[adze::grammar("mixed")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                expr: Box<Expr>,
            }

            pub enum Expr {
                Lit(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
            }
        }
    });
    assert_eq!(count_structs(&m), 1);
    assert_eq!(count_enums(&m), 1);
    assert_eq!(find_language_type(&m), Some("Program".to_string()));
}

// ── 4. Grammar with multiple enums ──────────────────────────────────────────

#[test]
fn edge_multiple_enums_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("multi_enum")]
        mod grammar {
            #[adze::language]
            pub enum Stmt {
                ExprStmt(Expr),
                TypeRef(TypeName),
            }

            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
            }

            pub enum TypeName {
                #[adze::leaf(text = "int")]
                Int,
                #[adze::leaf(text = "str")]
                Str,
            }
        }
    });
    assert_eq!(count_enums(&m), 3);
    assert_eq!(find_language_type(&m), Some("Stmt".to_string()));
}

// ── 5. Deeply nested types: Box<Option<Vec<T>>> ─────────────────────────────

#[test]
fn edge_deeply_nested_field_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("nested")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                deep: Box<Option<Vec<Inner>>>,
            }

            pub struct Inner {
                #[adze::leaf(pattern = r"\w+")]
                val: String,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let field_ty = s.fields.iter().next().unwrap().ty.to_token_stream().to_string();
        assert!(field_ty.contains("Box"));
        assert!(field_ty.contains("Option"));
        assert!(field_ty.contains("Vec"));
        assert!(field_ty.contains("Inner"));
    } else {
        panic!("Expected struct");
    }
}

// ── 6. All annotation types combined in one module ──────────────────────────

#[test]
fn edge_all_annotation_types_combined() {
    let m = parse_mod(quote! {
        #[adze::grammar("all_annots")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_right(2)]
                Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
                #[adze::prec(3)]
                Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
            }

            pub struct Wrapper {
                inner: Box<Expr>,
                #[adze::skip(0usize)]
                meta: usize,
            }

            pub struct Items {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(#[adze::leaf(text = ",")] ())]
                elems: Vec<Wrapper>,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::external]
            struct IndentToken;
        }
    });
    // Verify all special annotations are present
    let items = module_items(&m);
    let has_language = items.iter().any(|i| match i {
        Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "language")),
        _ => false,
    });
    let has_word = items.iter().any(|i| match i {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "word")),
        _ => false,
    });
    let has_extra = items.iter().any(|i| match i {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "extra")),
        _ => false,
    });
    let has_external = items.iter().any(|i| match i {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "external")),
        _ => false,
    });
    assert!(has_language);
    assert!(has_word);
    assert!(has_extra);
    assert!(has_external);
    assert_eq!(count_structs(&m), 5); // Wrapper, Items, Identifier, Whitespace, IndentToken
    assert_eq!(count_enums(&m), 1);
}

// ── 7. Visibility: pub(crate) on grammar items ──────────────────────────────

#[test]
fn edge_pub_crate_visibility_on_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("vis")]
        pub(crate) mod grammar {
            #[adze::language]
            pub(crate) struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Restricted(_)));
    if let Item::Struct(s) = &module_items(&m)[0] {
        assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
    } else {
        panic!("Expected struct");
    }
}

// ── 8. Visibility: pub(super) on grammar module ─────────────────────────────

#[test]
fn edge_pub_super_visibility_on_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("vis_super")]
        pub(super) mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                n: String,
            }
        }
    });
    if let syn::Visibility::Restricted(r) = &m.vis {
        assert_eq!(r.path.to_token_stream().to_string(), "super");
    } else {
        panic!("Expected restricted visibility");
    }
}

// ── 9. Visibility: mixed pub/private items in module ────────────────────────

#[test]
fn edge_mixed_visibility_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("vis_mix")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(
                    #[adze::leaf(pattern = r"\d+")]
                    String
                ),
            }

            struct PrivateHelper {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }

            pub struct PublicHelper {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Struct(priv_s) = &items[1] {
        assert!(matches!(priv_s.vis, syn::Visibility::Inherited));
        assert_eq!(priv_s.ident, "PrivateHelper");
    } else {
        panic!("Expected private struct");
    }
    if let Item::Struct(pub_s) = &items[2] {
        assert!(matches!(pub_s.vis, syn::Visibility::Public(_)));
        assert_eq!(pub_s.ident, "PublicHelper");
    } else {
        panic!("Expected public struct");
    }
}

// ── 10. Doc comments on grammar items are preserved ─────────────────────────

#[test]
fn edge_doc_comments_on_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("doc")]
        mod grammar {
            /// The root expression type
            #[adze::language]
            pub struct Root {
                /// The primary token
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let doc_attrs: Vec<_> = s.attrs.iter().filter(|a| a.path().is_ident("doc")).collect();
        assert!(!doc_attrs.is_empty(), "doc comment should be preserved on struct");
        let field_docs: Vec<_> = s
            .fields
            .iter()
            .next()
            .unwrap()
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .collect();
        assert!(!field_docs.is_empty(), "doc comment should be preserved on field");
    } else {
        panic!("Expected struct");
    }
}

// ── 11. Doc comments on enum and variants ───────────────────────────────────

#[test]
fn edge_doc_comments_on_enum_variants() {
    let m = parse_mod(quote! {
        #[adze::grammar("doc_enum")]
        mod grammar {
            /// Main expression enum
            #[adze::language]
            pub enum Expr {
                /// A numeric literal
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    String
                ),
                /// A string literal
                #[adze::leaf(text = "hello")]
                Hello,
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        let enum_docs: Vec<_> = e.attrs.iter().filter(|a| a.path().is_ident("doc")).collect();
        assert!(!enum_docs.is_empty(), "enum should have doc comment");
        for variant in &e.variants {
            let var_docs: Vec<_> = variant
                .attrs
                .iter()
                .filter(|a| a.path().is_ident("doc"))
                .collect();
            assert!(
                !var_docs.is_empty(),
                "variant '{}' should have doc comment",
                variant.ident
            );
        }
    } else {
        panic!("Expected enum");
    }
}

// ── 12. Doc comment on the module itself ────────────────────────────────────

#[test]
fn edge_doc_comment_on_module() {
    let m = parse_mod(quote! {
        /// This module defines a calculator grammar
        #[adze::grammar("calc_doc")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                n: String,
            }
        }
    });
    let doc_attrs: Vec<_> = m.attrs.iter().filter(|a| a.path().is_ident("doc")).collect();
    assert!(!doc_attrs.is_empty(), "module doc comment should be preserved");
    assert_eq!(extract_grammar_name(&m), Some("calc_doc".to_string()));
}

// ── 13. Grammar module naming: single char name ─────────────────────────────

#[test]
fn edge_grammar_name_single_char() {
    let m = parse_mod(quote! {
        #[adze::grammar("x")]
        mod g {
            #[adze::language]
            pub struct R {
                #[adze::leaf(pattern = r".")]
                c: String,
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("x".to_string()));
    assert_eq!(m.ident, "g");
}

// ── 14. Grammar module naming: hyphenated grammar name ──────────────────────

#[test]
fn edge_grammar_name_with_hyphens() {
    let m = parse_mod(quote! {
        #[adze::grammar("my-cool-lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("my-cool-lang".to_string()));
}

// ── 15. Grammar module naming: empty string name ────────────────────────────

#[test]
fn edge_grammar_name_empty_string() {
    let m = parse_mod(quote! {
        #[adze::grammar("")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some(String::new()));
}

// ── 16. Grammar module naming: name with dots ───────────────────────────────

#[test]
fn edge_grammar_name_with_dots() {
    let m = parse_mod(quote! {
        #[adze::grammar("my.lang.v2")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("my.lang.v2".to_string()));
}

// ── 17. Grammar module naming: very long name ───────────────────────────────

#[test]
fn edge_grammar_name_very_long() {
    let m = parse_mod(quote! {
        #[adze::grammar("this_is_a_very_long_grammar_name_that_tests_boundary_conditions")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert_eq!(
        extract_grammar_name(&m),
        Some("this_is_a_very_long_grammar_name_that_tests_boundary_conditions".to_string())
    );
}

// ── 18. Module with only use statements and no types ────────────────────────

#[test]
fn edge_module_only_use_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("uses_only")]
        mod grammar {
            use std::fmt;
            use std::collections::HashMap;
            use std::io::Read;
        }
    });
    assert_eq!(module_items(&m).len(), 3);
    assert!(module_items(&m).iter().all(|i| matches!(i, Item::Use(_))));
    assert_eq!(find_language_type(&m), None);
}

// ── 19. Struct with no fields (unit struct) as language ──────────────────────

#[test]
fn edge_unit_struct_as_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("unit_lang")]
        mod grammar {
            #[adze::language]
            #[adze::leaf(text = "EOF")]
            pub struct Eof;
        }
    });
    assert_eq!(find_language_type(&m), Some("Eof".to_string()));
    if let Item::Struct(s) = &module_items(&m)[0] {
        assert!(matches!(s.fields, Fields::Unit));
    } else {
        panic!("Expected struct");
    }
}

// ── 20. Enum with single variant ────────────────────────────────────────────

#[test]
fn edge_enum_single_variant() {
    let m = parse_mod(quote! {
        #[adze::grammar("single_var")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                #[adze::leaf(text = "x")]
                X,
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].ident, "X");
    } else {
        panic!("Expected enum");
    }
}

// ── 21. Enum with many unit variants ────────────────────────────────────────

#[test]
fn edge_enum_many_unit_variants() {
    let m = parse_mod(quote! {
        #[adze::grammar("keywords")]
        mod grammar {
            #[adze::language]
            pub enum Keyword {
                #[adze::leaf(text = "if")]
                If,
                #[adze::leaf(text = "else")]
                Else,
                #[adze::leaf(text = "while")]
                While,
                #[adze::leaf(text = "for")]
                For,
                #[adze::leaf(text = "return")]
                Return,
                #[adze::leaf(text = "break")]
                Break,
                #[adze::leaf(text = "continue")]
                Continue,
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert_eq!(e.variants.len(), 7);
        let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        assert!(names.contains(&"If".to_string()));
        assert!(names.contains(&"Continue".to_string()));
    } else {
        panic!("Expected enum");
    }
}

// ── 22. Struct with tuple fields (unnamed) ──────────────────────────────────

#[test]
fn edge_struct_with_tuple_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("tuple")]
        mod grammar {
            #[adze::language]
            pub struct Pair(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                i32,
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                i32
            );
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        assert!(matches!(s.fields, Fields::Unnamed(_)));
        assert_eq!(s.fields.len(), 2);
    } else {
        panic!("Expected struct");
    }
}

// ── 23. Multiple extra structs: verify count ────────────────────────────────

#[test]
fn edge_three_extra_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("extras")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct LineComment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _c: (),
            }

            #[adze::extra]
            struct BlockComment {
                #[adze::leaf(pattern = r"/\*[^*]*\*/")]
                _c: (),
            }
        }
    });
    let extras: Vec<_> = module_items(&m)
        .iter()
        .filter(|i| match i {
            Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "extra")),
            _ => false,
        })
        .collect();
    assert_eq!(extras.len(), 3);
}

// ── 24. Non-adze attributes survive on items ────────────────────────────────

#[test]
fn edge_non_adze_attrs_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("attr_test")]
        mod grammar {
            #[derive(Debug, Clone)]
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }

            #[derive(PartialEq)]
            pub struct Helper {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let derive_attrs: Vec<_> = s
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("derive"))
            .collect();
        assert!(!derive_attrs.is_empty(), "derive should survive on Root");
    } else {
        panic!("Expected struct");
    }
    if let Item::Struct(s) = &module_items(&m)[1] {
        let derive_attrs: Vec<_> = s
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("derive"))
            .collect();
        assert!(!derive_attrs.is_empty(), "derive should survive on Helper");
    } else {
        panic!("Expected struct");
    }
}

// ── 25. cfg attribute alongside adze attributes ─────────────────────────────

#[test]
fn edge_cfg_attr_on_module() {
    let m = parse_mod(quote! {
        #[cfg(feature = "parser")]
        #[adze::grammar("cfg_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    let cfg_attrs: Vec<_> = m
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg"))
        .collect();
    assert_eq!(cfg_attrs.len(), 1);
    assert_eq!(extract_grammar_name(&m), Some("cfg_test".to_string()));
}

// ── 26. Module ident differs from grammar name ──────────────────────────────

#[test]
fn edge_module_ident_differs_from_grammar_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("calculator")]
        mod my_parser_impl {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                n: String,
            }
        }
    });
    assert_eq!(m.ident, "my_parser_impl");
    assert_eq!(extract_grammar_name(&m), Some("calculator".to_string()));
}

// ── 27. Enum variant with named fields (struct-like variant) ────────────────

#[test]
fn edge_enum_variant_all_named_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("named_var")]
        mod grammar {
            #[adze::language]
            pub enum Node {
                BinaryOp {
                    lhs: Box<Node>,
                    #[adze::leaf(text = "+")]
                    _op: (),
                    rhs: Box<Node>,
                },
                Literal {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    value: i32,
                },
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert_eq!(e.variants.len(), 2);
        for variant in &e.variants {
            assert!(
                matches!(variant.fields, Fields::Named(_)),
                "variant '{}' should have named fields",
                variant.ident
            );
        }
    } else {
        panic!("Expected enum");
    }
}

// ── 28. Enum with mixed variant kinds: unit, tuple, struct ──────────────────

#[test]
fn edge_enum_mixed_variant_kinds() {
    let m = parse_mod(quote! {
        #[adze::grammar("mixed_kinds")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "nil")]
                Nil,
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                Assignment {
                    #[adze::leaf(pattern = r"[a-z]+")]
                    name: String,
                    #[adze::leaf(text = "=")]
                    _eq: (),
                    value: Box<Expr>,
                },
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert!(matches!(e.variants[0].fields, Fields::Unit));
        assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
        assert!(matches!(e.variants[2].fields, Fields::Named(_)));
    } else {
        panic!("Expected enum");
    }
}

// ── 29. Deeply nested type: Vec<Vec<T>> ─────────────────────────────────────

#[test]
fn edge_nested_vec_of_vec() {
    let m = parse_mod(quote! {
        #[adze::grammar("nested_vec")]
        mod grammar {
            #[adze::language]
            pub struct Matrix {
                rows: Vec<Vec<Cell>>,
            }

            pub struct Cell {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let ty = s.fields.iter().next().unwrap().ty.to_token_stream().to_string();
        assert!(ty.contains("Vec < Vec"));
    } else {
        panic!("Expected struct");
    }
}

// ── 30. Deeply nested type: Option<Box<Option<T>>> ──────────────────────────

#[test]
fn edge_option_box_option_nesting() {
    let m = parse_mod(quote! {
        #[adze::grammar("deep_opt")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                maybe: Option<Box<Option<Inner>>>,
            }

            pub struct Inner {
                #[adze::leaf(pattern = r"\w+")]
                val: String,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let ty = s.fields.iter().next().unwrap().ty.to_token_stream().to_string();
        assert!(ty.contains("Option < Box < Option"));
    } else {
        panic!("Expected struct");
    }
}

// ── 31. Multiple precedence levels on enum variants ─────────────────────────

#[test]
fn edge_multiple_prec_levels() {
    let m = parse_mod(quote! {
        #[adze::grammar("prec")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                #[adze::prec_right(3)]
                Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
                #[adze::prec(4)]
                Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert_eq!(e.variants.len(), 5);
        // Check that prec variants have the expected adze attrs
        let prec_variants: Vec<_> = e
            .variants
            .iter()
            .filter(|v| {
                v.attrs.iter().any(|a| {
                    is_adze_attr(a, "prec_left")
                        || is_adze_attr(a, "prec_right")
                        || is_adze_attr(a, "prec")
                })
            })
            .collect();
        assert_eq!(prec_variants.len(), 4);
    } else {
        panic!("Expected enum");
    }
}

// ── 32. Struct with multiple skip fields ────────────────────────────────────

#[test]
fn edge_multiple_skip_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("multi_skip")]
        mod grammar {
            #[adze::language]
            pub struct Node {
                #[adze::leaf(pattern = r"\w+")]
                value: String,
                #[adze::skip(false)]
                visited: bool,
                #[adze::skip(0u32)]
                depth: u32,
                #[adze::skip(String::new())]
                tag: String,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let skip_fields: Vec<_> = s
            .fields
            .iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
            .collect();
        assert_eq!(skip_fields.len(), 3);
    } else {
        panic!("Expected struct");
    }
}

// ── 33. Module with use + const + type alias (non-type items) ───────────────

#[test]
fn edge_module_with_non_type_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("misc_items")]
        mod grammar {
            use std::fmt;

            const MAX_DEPTH: usize = 100;

            type Identifier = String;

            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    let items = module_items(&m);
    assert!(items.iter().any(|i| matches!(i, Item::Use(_))));
    assert!(items.iter().any(|i| matches!(i, Item::Const(_))));
    assert!(items.iter().any(|i| matches!(i, Item::Type(_))));
    assert_eq!(find_language_type(&m), Some("Root".to_string()));
}

// ── 34. Grammar name with unicode characters ────────────────────────────────

#[test]
fn edge_grammar_name_unicode() {
    let m = parse_mod(quote! {
        #[adze::grammar("grüße_日本語")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert_eq!(
        extract_grammar_name(&m),
        Some("grüße_日本語".to_string())
    );
}

// ── 35. Language on enum, all other items are structs ────────────────────────

#[test]
fn edge_language_enum_with_many_helper_structs() {
    let m = parse_mod(quote! {
        #[adze::grammar("helpers")]
        mod grammar {
            #[adze::language]
            pub enum Stmt {
                Decl(Declaration),
                Assign(Assignment),
                Print(PrintStmt),
            }

            pub struct Declaration {
                #[adze::leaf(text = "let")]
                _kw: (),
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            pub struct Assignment {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
                #[adze::leaf(text = "=")]
                _eq: (),
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }

            pub struct PrintStmt {
                #[adze::leaf(text = "print")]
                _kw: (),
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    assert_eq!(count_structs(&m), 4);
    assert_eq!(count_enums(&m), 1);
    assert_eq!(find_language_type(&m), Some("Stmt".to_string()));
}
