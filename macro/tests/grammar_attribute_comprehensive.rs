#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the `#[adze::grammar]` attribute in adze-macro.
//!
//! Covers grammar attribute on modules with structs, enums, mixed types,
//! grammar name extraction, inline modules, attribute presence detection,
//! module visibility, use statements, and empty grammar modules.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemMod, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn has_grammar_attr(m: &ItemMod) -> bool {
    m.attrs.iter().any(|a| is_adze_attr(a, "grammar"))
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

/// Extract the grammar name from a module's `#[adze::grammar("...")]` attribute.
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

/// Find the root type name (annotated with `#[adze::language]`) in a module.
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

fn struct_names(m: &ItemMod) -> Vec<String> {
    module_items(m)
        .iter()
        .filter_map(|i| match i {
            Item::Struct(s) => Some(s.ident.to_string()),
            _ => None,
        })
        .collect()
}

fn enum_names(m: &ItemMod) -> Vec<String> {
    module_items(m)
        .iter()
        .filter_map(|i| match i {
            Item::Enum(e) => Some(e.ident.to_string()),
            _ => None,
        })
        .collect()
}

// ── 1. Grammar on module with a single struct ───────────────────────────────

#[test]
fn grammar_module_with_single_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("single_struct")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    assert!(has_grammar_attr(&m));
    assert_eq!(struct_names(&m), vec!["Root"]);
}

// ── 2. Grammar on module with multiple structs ──────────────────────────────

#[test]
fn grammar_module_with_multiple_structs() {
    let m = parse_mod(quote! {
        #[adze::grammar("multi_struct")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                stmt: Statement,
            }

            pub struct Statement {
                #[adze::leaf(pattern = r"\w+")]
                text: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    assert_eq!(struct_names(&m), vec!["Program", "Statement", "Whitespace"]);
    assert_eq!(find_language_type(&m), Some("Program".to_string()));
}

// ── 3. Grammar on module with a single enum ─────────────────────────────────

#[test]
fn grammar_module_with_single_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("single_enum")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
            }
        }
    });
    assert!(has_grammar_attr(&m));
    assert_eq!(enum_names(&m), vec!["Expr"]);
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
}

// ── 4. Grammar on module with multiple enums ────────────────────────────────

#[test]
fn grammar_module_with_multiple_enums() {
    let m = parse_mod(quote! {
        #[adze::grammar("multi_enum")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Literal(Literal),
            }

            pub enum Literal {
                Int(i32),
                Float(f64),
            }
        }
    });
    assert_eq!(enum_names(&m), vec!["Expr", "Literal"]);
}

// ── 5. Grammar on module with mixed types (structs + enums) ─────────────────

#[test]
fn grammar_module_with_mixed_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("mixed")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                expr: Box<Expr>,
            }

            pub enum Expr {
                Number(i32),
                Add(Box<Expr>, Box<Expr>),
            }

            pub struct Number {
                v: i32,
            }
        }
    });
    assert_eq!(struct_names(&m), vec!["Program", "Number"]);
    assert_eq!(enum_names(&m), vec!["Expr"]);
}

// ── 6. Grammar module name extraction: simple ───────────────────────────────

#[test]
fn grammar_name_extraction_simple() {
    let m = parse_mod(quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("arithmetic".to_string()));
}

// ── 7. Grammar module name extraction: underscores and digits ───────────────

#[test]
fn grammar_name_extraction_with_underscores_digits() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_lang_v2")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("my_lang_v2".to_string()));
}

// ── 8. Grammar module name extraction: hyphenated name ──────────────────────

#[test]
fn grammar_name_extraction_hyphenated() {
    let m = parse_mod(quote! {
        #[adze::grammar("my-grammar")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("my-grammar".to_string()));
}

// ── 9. Grammar module name extraction: empty string ─────────────────────────

#[test]
fn grammar_name_extraction_empty_string() {
    let m = parse_mod(quote! {
        #[adze::grammar("")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(extract_grammar_name(&m), Some(String::new()));
}

// ── 10. Grammar with inline module: module ident preserved ──────────────────

#[test]
fn grammar_inline_module_ident_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod my_parser {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(m.ident.to_string(), "my_parser");
    assert!(m.content.is_some());
}

// ── 11. Grammar attribute presence detection: present ───────────────────────

#[test]
fn grammar_attribute_detected_when_present() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(has_grammar_attr(&m));
}

// ── 12. Grammar attribute presence detection: absent ────────────────────────

#[test]
fn grammar_attribute_not_detected_when_absent() {
    let m = parse_mod(quote! {
        mod grammar {
            pub struct Root {}
        }
    });
    assert!(!has_grammar_attr(&m));
}

// ── 13. Grammar attribute coexists with other attributes ────────────────────

#[test]
fn grammar_attribute_coexists_with_other_attrs() {
    let m = parse_mod(quote! {
        #[allow(dead_code)]
        #[adze::grammar("test")]
        #[cfg(test)]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(has_grammar_attr(&m));
    assert_eq!(m.attrs.len(), 3);
    let non_grammar: Vec<_> = m
        .attrs
        .iter()
        .filter(|a| !is_adze_attr(a, "grammar"))
        .collect();
    assert_eq!(non_grammar.len(), 2);
}

// ── 14. Grammar module visibility: pub ──────────────────────────────────────

#[test]
fn grammar_module_pub_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        pub mod my_grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Public(_)));
}

// ── 15. Grammar module visibility: private (inherited) ──────────────────────

#[test]
fn grammar_module_private_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod private_grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Inherited));
}

// ── 16. Grammar module visibility: pub(crate) ───────────────────────────────

#[test]
fn grammar_module_pub_crate_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        pub(crate) mod restricted_grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Restricted(_)));
    assert_eq!(m.ident.to_string(), "restricted_grammar");
}

// ── 17. Grammar with use statements ─────────────────────────────────────────

#[test]
fn grammar_module_with_use_statements() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use std::fmt;
            use std::collections::HashMap;

            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    let uses: Vec<_> = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Use(_)))
        .collect();
    assert_eq!(uses.len(), 2);
}

// ── 18. Grammar with use statement: adze::Spanned ───────────────────────────

#[test]
fn grammar_module_with_adze_use() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub struct Root {
                child: Spanned<Child>,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        }
    });
    let uses: Vec<_> = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Use(_)))
        .collect();
    assert_eq!(uses.len(), 1);
    assert_eq!(struct_names(&m), vec!["Root", "Child"]);
}

// ── 19. Empty grammar module ────────────────────────────────────────────────

#[test]
fn empty_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("empty")]
        mod grammar {}
    });
    assert!(has_grammar_attr(&m));
    assert!(module_items(&m).is_empty());
    assert_eq!(extract_grammar_name(&m), Some("empty".to_string()));
}

// ── 20. Grammar name absent returns None from extraction ────────────────────

#[test]
fn grammar_name_absent_returns_none() {
    let m = parse_mod(quote! {
        mod grammar {
            pub struct Root {}
        }
    });
    assert_eq!(extract_grammar_name(&m), None);
}

// ── 21. Grammar module preserves item order ─────────────────────────────────

#[test]
fn grammar_module_preserves_item_order() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use std::fmt;

            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }

            pub struct Helper {
                v: i32,
            }

            #[adze::extra]
            struct Whitespace {}
        }
    });
    let items = module_items(&m);
    assert!(matches!(&items[0], Item::Use(_)));
    assert!(matches!(&items[1], Item::Enum(e) if e.ident == "Expr"));
    assert!(matches!(&items[2], Item::Struct(s) if s.ident == "Helper"));
    assert!(matches!(&items[3], Item::Struct(s) if s.ident == "Whitespace"));
}

// ── 22. Grammar struct language type has correct fields ─────────────────────

#[test]
fn grammar_struct_language_type_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("fields_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                child: Option<Child>,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        assert_eq!(s.ident, "Root");
        let field_names: Vec<_> = s
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        assert_eq!(field_names, vec!["name", "child"]);
    } else {
        panic!("Expected struct Root");
    }
}

// ── 23. Grammar enum language type has correct variants ─────────────────────

#[test]
fn grammar_enum_language_type_variants() {
    let m = parse_mod(quote! {
        #[adze::grammar("variants_test")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                #[adze::leaf(text = "+")]
                Plus,
                #[adze::leaf(text = "-")]
                Minus,
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    String
                ),
            }
        }
    });
    let items = module_items(&m);
    if let Item::Enum(e) = &items[0] {
        let variant_names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        assert_eq!(variant_names, vec!["Plus", "Minus", "Number"]);
    } else {
        panic!("Expected enum Token");
    }
}

// ── 24. Grammar module with mixed struct and enum language is struct ─────────

#[test]
fn grammar_module_language_on_struct_not_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("mixed_lang")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                expr: Box<Expr>,
            }

            pub enum Expr {
                Number(i32),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Program".to_string()));
}

// ── 25. Grammar with inline module containing extra and word ────────────────

#[test]
fn grammar_module_with_extra_and_word() {
    let m = parse_mod(quote! {
        #[adze::grammar("full")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                ident: Identifier,
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
        }
    });
    let items = module_items(&m);
    let has_word = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "word"))
        } else {
            false
        }
    });
    let has_extra = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
        } else {
            false
        }
    });
    assert!(has_word);
    assert!(has_extra);
}

// ── 26. Grammar module ident differs from grammar name ──────────────────────

#[test]
fn grammar_module_ident_differs_from_grammar_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("calculator")]
        mod calc_parser {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(m.ident.to_string(), "calc_parser");
    assert_eq!(extract_grammar_name(&m), Some("calculator".to_string()));
}

// ── 27. Grammar attribute path is exactly adze::grammar ─────────────────────

#[test]
fn grammar_attribute_path_is_exact() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    let grammar_attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let path_str = grammar_attr.path().to_token_stream().to_string();
    assert_eq!(path_str, "adze :: grammar");
}

// ── 28. Grammar module with external scanner type ───────────────────────────

#[test]
fn grammar_module_with_external_scanner() {
    let m = parse_mod(quote! {
        #[adze::grammar("ext_test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }

            #[adze::external]
            struct IndentToken;
        }
    });
    let items = module_items(&m);
    let has_external = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "external"))
        } else {
            false
        }
    });
    assert!(has_external);
}

// ── 29. Grammar module with only use statements (no types) ──────────────────

#[test]
fn grammar_module_only_use_statements() {
    let m = parse_mod(quote! {
        #[adze::grammar("use_only")]
        mod grammar {
            use std::fmt;
            use std::collections::HashMap;
        }
    });
    assert!(has_grammar_attr(&m));
    assert!(struct_names(&m).is_empty());
    assert!(enum_names(&m).is_empty());
    let uses: Vec<_> = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Use(_)))
        .collect();
    assert_eq!(uses.len(), 2);
}

// ── 30. Grammar name extraction with special characters ─────────────────────

#[test]
fn grammar_name_with_special_characters() {
    let m = parse_mod(quote! {
        #[adze::grammar("my.lang/v1")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("my.lang/v1".to_string()));
}

// ── 31. Grammar module multiple attributes on single item ───────────────────

#[test]
fn grammar_module_item_with_multiple_adze_attrs() {
    let m = parse_mod(quote! {
        #[adze::grammar("multi_attr")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                ident: Identifier,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Struct(s) = &items[1] {
        assert_eq!(s.ident, "Identifier");
        assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    } else {
        panic!("Expected struct Identifier at index 1");
    }
}

// ── 32. Grammar module content is brace-delimited (not semicolon) ───────────

#[test]
fn grammar_module_is_inline_not_file() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    // An inline module has content; a `mod grammar;` would have None
    assert!(m.content.is_some());
    assert!(m.semi.is_none());
}

// ── 33. Grammar module with struct having Vec fields ────────────────────────

#[test]
fn grammar_module_struct_with_vec_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("vec_test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                statements: Vec<Statement>,
            }

            pub struct Statement {
                #[adze::leaf(pattern = r"\w+")]
                text: String,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        assert_eq!(s.ident, "Program");
        let field = s.fields.iter().next().unwrap();
        let ty_str = field.ty.to_token_stream().to_string();
        assert!(ty_str.contains("Vec"), "Expected Vec field, got: {ty_str}");
    } else {
        panic!("Expected struct Program");
    }
}

// ── 34. Grammar module with struct having Option fields ─────────────────────

#[test]
fn grammar_module_struct_with_option_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("opt_test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Option<Child>,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        let field = s.fields.iter().next().unwrap();
        let ty_str = field.ty.to_token_stream().to_string();
        assert!(
            ty_str.contains("Option"),
            "Expected Option field, got: {ty_str}"
        );
    } else {
        panic!("Expected struct Root");
    }
}

// ── 35. Grammar module enum with unit and tuple variants ────────────────────

#[test]
fn grammar_module_enum_with_unit_and_tuple_variants() {
    let m = parse_mod(quote! {
        #[adze::grammar("variant_test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "+")]
                Plus,
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    String
                ),
                Neg {
                    #[adze::leaf(text = "-")]
                    _sign: (),
                    value: Box<Expr>,
                },
            }
        }
    });
    let items = module_items(&m);
    if let Item::Enum(e) = &items[0] {
        assert_eq!(e.variants.len(), 3);
        assert!(matches!(e.variants[0].fields, Fields::Unit));
        assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
        assert!(matches!(e.variants[2].fields, Fields::Named(_)));
    } else {
        panic!("Expected enum Expr");
    }
}
