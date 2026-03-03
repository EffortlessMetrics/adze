#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for grammar module handling in the adze proc-macro crate.
//!
//! Covers module-level structure parsing, grammar name extraction, root type
//! (`language`) detection, item enumeration inside grammar modules, cross-type
//! references, field-type combinations, and edge cases in module definitions.

use std::collections::HashSet;

use adze_common::{FieldThenParams, NameValueExpr, try_extract_inner_type, wrap_leaf_type};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Token, Type, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn adze_attr_names(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|a| {
            let segs: Vec<_> = a.path().segments.iter().collect();
            if segs.len() == 2 && segs[0].ident == "adze" {
                Some(segs[1].ident.to_string())
            } else {
                None
            }
        })
        .collect()
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

/// Count items by kind inside a module.
fn count_items_by_kind(m: &ItemMod) -> (usize, usize, usize) {
    let items = module_items(m);
    let structs = items
        .iter()
        .filter(|i| matches!(i, Item::Struct(_)))
        .count();
    let enums = items.iter().filter(|i| matches!(i, Item::Enum(_))).count();
    let uses = items.iter().filter(|i| matches!(i, Item::Use(_))).count();
    (structs, enums, uses)
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

// ── 1. Grammar name extraction: simple name ─────────────────────────────────

#[test]
fn grammar_name_simple() {
    let m = parse_mod(quote! {
        #[adze::grammar("calc")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("calc".to_string()));
}

// ── 2. Grammar name extraction: name with underscores and digits ────────────

#[test]
fn grammar_name_complex() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_lang_v3_beta")]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(
        extract_grammar_name(&m),
        Some("my_lang_v3_beta".to_string())
    );
}

// ── 3. Grammar name missing returns None ────────────────────────────────────

#[test]
fn grammar_name_absent_returns_none() {
    let m = parse_mod(quote! {
        mod grammar {
            pub struct Root {}
        }
    });
    assert_eq!(extract_grammar_name(&m), None);
}

// ── 4. Language type detection: struct root ──────────────────────────────────

#[test]
fn language_type_struct_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                expr: Box<Expr>,
            }
            pub enum Expr {
                Lit(i32),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Program".to_string()));
}

// ── 5. Language type detection: enum root ────────────────────────────────────

#[test]
fn language_type_enum_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(i32),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Expression".to_string()));
}

// ── 6. Language type missing returns None ────────────────────────────────────

#[test]
fn language_type_absent_returns_none() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            pub struct Helper {}
        }
    });
    assert_eq!(find_language_type(&m), None);
}

// ── 7. Module item counting: structs, enums, uses ───────────────────────────

#[test]
fn module_item_counts() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use std::collections::HashMap;
            use std::fmt;

            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }

            pub struct Number {
                v: i32,
            }

            #[adze::extra]
            struct Whitespace {}
        }
    });
    let (structs, enums, uses) = count_items_by_kind(&m);
    assert_eq!(structs, 2);
    assert_eq!(enums, 1);
    assert_eq!(uses, 2);
}

// ── 8. Module content ordering is preserved ─────────────────────────────────

#[test]
fn module_item_order_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use std::collections::HashMap;

            #[adze::language]
            pub struct Root {
                child: Child,
            }

            pub struct Child {
                v: i32,
            }

            #[adze::extra]
            struct Whitespace {}
        }
    });
    let items = module_items(&m);
    assert!(matches!(&items[0], Item::Use(_)));
    assert!(matches!(&items[1], Item::Struct(s) if s.ident == "Root"));
    assert!(matches!(&items[2], Item::Struct(s) if s.ident == "Child"));
    assert!(matches!(&items[3], Item::Struct(s) if s.ident == "Whitespace"));
}

// ── 9. Module visibility: public module ─────────────────────────────────────

#[test]
fn module_pub_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        pub mod my_grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Public(_)));
    assert_eq!(m.ident.to_string(), "my_grammar");
}

// ── 10. Module visibility: private module ───────────────────────────────────

#[test]
fn module_private_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod private_grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Inherited));
}

// ── 11. Empty module body parses successfully ───────────────────────────────

#[test]
fn empty_module_parses() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {}
    });
    assert!(m.content.is_some());
    assert!(module_items(&m).is_empty());
}

// ── 12. Module with only use statements ─────────────────────────────────────

#[test]
fn module_with_only_uses() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use std::fmt;
            use std::collections::HashMap;
        }
    });
    let (structs, enums, uses) = count_items_by_kind(&m);
    assert_eq!(structs, 0);
    assert_eq!(enums, 0);
    assert_eq!(uses, 2);
}

// ── 13. Module attributes filtered correctly ────────────────────────────────

#[test]
fn module_attrs_separate_adze_from_others() {
    let m = parse_mod(quote! {
        #[allow(dead_code)]
        #[adze::grammar("test")]
        #[cfg(test)]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    let adze_attrs: Vec<_> = m
        .attrs
        .iter()
        .filter(|a| is_adze_attr(a, "grammar"))
        .collect();
    let non_adze_attrs: Vec<_> = m
        .attrs
        .iter()
        .filter(|a| {
            !a.path()
                .segments
                .iter()
                .next()
                .is_some_and(|s| s.ident == "adze")
        })
        .collect();
    assert_eq!(adze_attrs.len(), 1);
    assert_eq!(non_adze_attrs.len(), 2);
}

// ── 14. Multiple extra types in one module ──────────────────────────────────

#[test]
fn module_multiple_extras() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                v: String,
            }

            #[adze::extra]
            struct Whitespace {}

            #[adze::extra]
            struct Comment {}

            #[adze::extra]
            struct Newline {}
        }
    });
    let extras: Vec<_> = module_items(&m)
        .iter()
        .filter(|item| {
            if let Item::Struct(s) = item {
                s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            } else {
                false
            }
        })
        .collect();
    assert_eq!(extras.len(), 3);
}

// ── 15. Module with extra + external + word types ───────────────────────────

#[test]
fn module_mixed_special_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                ident: Identifier,
            }

            #[adze::word]
            pub struct Identifier {
                name: String,
            }

            #[adze::extra]
            struct Whitespace {}

            #[adze::external]
            struct IndentToken;
        }
    });
    let items = module_items(&m);
    let mut found_word = false;
    let mut found_extra = false;
    let mut found_external = false;
    for item in items {
        if let Item::Struct(s) = item {
            if s.attrs.iter().any(|a| is_adze_attr(a, "word")) {
                found_word = true;
            }
            if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) {
                found_extra = true;
            }
            if s.attrs.iter().any(|a| is_adze_attr(a, "external")) {
                found_external = true;
            }
        }
    }
    assert!(found_word);
    assert!(found_extra);
    assert!(found_external);
}

// ── 16. Cross-referencing types: struct referencing enum ─────────────────────

#[test]
fn module_struct_references_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                expr: Box<Expr>,
            }

            pub enum Expr {
                Number(i32),
                Add(Box<Expr>, Box<Expr>),
            }
        }
    });
    let items = module_items(&m);
    // Root is struct referencing the Expr enum
    if let Item::Struct(s) = &items[0] {
        assert_eq!(s.ident, "Program");
        let field = s.fields.iter().next().unwrap();
        let ty_str = field.ty.to_token_stream().to_string();
        assert!(ty_str.contains("Expr"), "Field should reference Expr type");
    } else {
        panic!("Expected struct as first item");
    }
}

// ── 17. Cross-referencing types: enum referencing multiple structs ───────────

#[test]
fn module_enum_references_structs() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Statement {
                Assign(Assignment),
                Print(PrintStmt),
            }

            pub struct Assignment {
                name: String,
                value: i32,
            }

            pub struct PrintStmt {
                text: String,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Enum(e) = &items[0] {
        assert_eq!(e.variants.len(), 2);
        // Check variant field types reference structs
        for variant in &e.variants {
            if let Fields::Unnamed(u) = &variant.fields {
                let ty_str = u.unnamed[0].ty.to_token_stream().to_string();
                assert!(
                    ty_str == "Assignment" || ty_str == "PrintStmt",
                    "Variant should reference a struct type, got: {ty_str}"
                );
            }
        }
    } else {
        panic!("Expected enum as first item");
    }
}

// ── 18. Field with Vec type detected via try_extract_inner_type ──────────────

#[test]
fn module_field_vec_inner_type_extraction() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct NumberList {
                numbers: Vec<Number>,
            }

            pub struct Number {
                v: i32,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let field = s.fields.iter().next().unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "Number");
    } else {
        panic!("Expected struct");
    }
}

// ── 19. Field with Option type detected ─────────────────────────────────────

#[test]
fn module_field_option_inner_type_extraction() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct MaybeNumber {
                num: Option<i32>,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let field = s.fields.iter().next().unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&field.ty, "Option", &skip);
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "i32");
    } else {
        panic!("Expected struct");
    }
}

// ── 20. Enum with all three variant kinds in one grammar ────────────────────

#[test]
fn module_enum_mixed_variant_kinds() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                #[adze::leaf(text = "+")]
                Plus,

                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),

                Complex {
                    #[adze::leaf(pattern = r"[a-z]+")]
                    name: String,
                    #[adze::leaf(text = ":")]
                    _colon: (),
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    value: i32,
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

// ── 21. Struct with delimited + repeat on same field ────────────────────────

#[test]
fn module_field_delimited_and_repeat() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct ArgList {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                args: Vec<Arg>,
            }

            pub struct Arg {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let field = s.fields.iter().next().unwrap();
        let names = adze_attr_names(&field.attrs);
        assert!(names.contains(&"repeat".to_string()));
        assert!(names.contains(&"delimited".to_string()));

        // Verify repeat params
        let repeat_attr = field
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "repeat"))
            .unwrap();
        let params = leaf_params(repeat_attr);
        assert_eq!(params[0].path.to_string(), "non_empty");

        // Verify delimited inner field
        let delim_attr = field
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "delimited"))
            .unwrap();
        let ftp: FieldThenParams = delim_attr.parse_args().unwrap();
        let inner_leaf = ftp
            .field
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let inner_params = leaf_params(inner_leaf);
        assert_eq!(inner_params[0].path.to_string(), "text");
    } else {
        panic!("Expected struct");
    }
}

// ── 22. Enum with precedence variants in a grammar module ───────────────────

#[test]
fn module_enum_precedence_hierarchy() {
    let m = parse_mod(quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(1)]
                Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Div(Box<Expr>, #[adze::leaf(text = "/")] (), Box<Expr>),
                #[adze::prec_right(3)]
                Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert_eq!(e.variants.len(), 6);
        // Verify precedence values
        let prec_variants: Vec<(&str, &str, i32)> = e
            .variants
            .iter()
            .filter_map(|v| {
                v.attrs.iter().find_map(|a| {
                    for name in &["prec", "prec_left", "prec_right"] {
                        if is_adze_attr(a, name) {
                            let expr: syn::Expr = a.parse_args().ok()?;
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Int(i),
                                ..
                            }) = expr
                            {
                                return Some((
                                    v.ident.to_string().leak() as &str,
                                    *name,
                                    i.base10_parse::<i32>().unwrap(),
                                ));
                            }
                        }
                    }
                    None
                })
            })
            .collect();
        assert_eq!(prec_variants.len(), 5);
        // Add and Sub have prec 1
        assert!(
            prec_variants
                .iter()
                .any(|(n, a, v)| *n == "Add" && *a == "prec_left" && *v == 1)
        );
        assert!(
            prec_variants
                .iter()
                .any(|(n, a, v)| *n == "Pow" && *a == "prec_right" && *v == 3)
        );
    } else {
        panic!("Expected enum");
    }
}

// ── 23. Module with Spanned wrapper type ────────────────────────────────────

#[test]
fn module_spanned_wrapper_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub struct Program {
                items: Vec<Spanned<Item>>,
            }

            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[1] {
        let field = s.fields.iter().next().unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
        assert!(extracted);
        // Vec extracts Spanned<Item>; the skip set is for outer wrappers
        assert_eq!(inner.to_token_stream().to_string(), "Spanned < Item >");
    } else {
        panic!("Expected struct");
    }
}

// ── 24. Module with skip field alongside regular fields ─────────────────────

#[test]
fn module_struct_skip_field() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Node {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                #[adze::skip(0usize)]
                visit_count: usize,
                #[adze::skip(false)]
                visited: bool,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let fields: Vec<_> = s.fields.iter().collect();
        assert_eq!(fields.len(), 3);
        // First field has leaf attr
        assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        // Second and third have skip attr
        assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
        assert!(fields[2].attrs.iter().any(|a| is_adze_attr(a, "skip")));
    } else {
        panic!("Expected struct");
    }
}

// ── 25. wrap_leaf_type through Box<Option<T>> ───────────────────────────────

#[test]
fn wrap_leaf_type_box_option() {
    let skip: HashSet<&str> = ["Box", "Option", "Vec", "Spanned"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Box < Option < adze :: WithLeaf < i32 > > >"
    );
}

// ── 26. Module ident is preserved ───────────────────────────────────────────

#[test]
fn module_ident_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod my_custom_grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(m.ident.to_string(), "my_custom_grammar");
}

// ── 27. Module with multiple enums (one language, one not) ──────────────────

#[test]
fn module_multiple_enums_one_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(i32),
                Op(Operator, Box<Expr>, Box<Expr>),
            }

            pub enum Operator {
                #[adze::leaf(text = "+")]
                Plus,
                #[adze::leaf(text = "-")]
                Minus,
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
    let (structs, enums, _) = count_items_by_kind(&m);
    assert_eq!(enums, 2);
    assert_eq!(structs, 0);
}

// ── 28. Module with derive attrs preserved alongside adze attrs ─────────────

#[test]
fn module_derive_and_adze_attrs_coexist() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[derive(Debug, Clone)]
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let derive_count = s
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("derive"))
            .count();
        let adze_count = s
            .attrs
            .iter()
            .filter(|a| is_adze_attr(a, "language"))
            .count();
        assert_eq!(derive_count, 1);
        assert_eq!(adze_count, 1);
    } else {
        panic!("Expected struct");
    }
}

// ── 29. Delimited inner field type parsed correctly ─────────────────────────

#[test]
fn delimited_inner_type_is_unit() {
    let s: ItemStruct = parse_quote! {
        pub struct Csv {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            values: Vec<Value>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim_attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim_attr.parse_args().unwrap();
    // Inner field type should be unit `()`
    let ty_str = ftp.field.ty.to_token_stream().to_string();
    assert_eq!(ty_str, "()");
}

// ── 30. Grammar module with recursive struct via Box ────────────────────────

#[test]
fn module_recursive_struct_via_box() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct ListNode {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
                next: Option<Box<ListNode>>,
            }
        }
    });
    if let Item::Struct(s) = &module_items(&m)[0] {
        let fields: Vec<_> = s.fields.iter().collect();
        assert_eq!(fields.len(), 2);
        let next_ty = fields[1].ty.to_token_stream().to_string();
        assert!(next_ty.contains("Box"));
        assert!(next_ty.contains("ListNode"));
    } else {
        panic!("Expected struct");
    }
}

// ── 31. Module with unit struct leaf type ────────────────────────────────────

#[test]
fn module_unit_struct_leaf() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                digit: BigDigit,
            }

            #[adze::leaf(text = "9")]
            pub struct BigDigit;
        }
    });
    if let Item::Struct(s) = &module_items(&m)[1] {
        assert_eq!(s.ident, "BigDigit");
        assert!(s.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        assert!(matches!(s.fields, Fields::Unit));
    } else {
        panic!("Expected struct");
    }
}

// ── 32. Leaf params: text + transform together ──────────────────────────────

#[test]
fn leaf_text_with_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct Bool {
            #[adze::leaf(text = "true", transform = |_| true)]
            v: bool,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["text", "transform"]);
}

// ── 33. Large grammar module with many types ────────────────────────────────

#[test]
fn large_module_many_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("lang")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub enum Stmt {
                VarDecl(VarDecl),
                FnDecl(FnDecl),
                ExprStmt(Expr),
            }

            pub struct VarDecl {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
                #[adze::leaf(text = "=")]
                _eq: (),
                value: Expr,
            }

            pub struct FnDecl {
                #[adze::leaf(text = "fn")]
                _kw: (),
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
                params: Vec<Param>,
                body: Box<Expr>,
            }

            pub struct Param {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                Ident(
                    #[adze::leaf(pattern = r"[a-z]+")]
                    String
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _c: (),
            }
        }
    });
    let (structs, enums, uses) = count_items_by_kind(&m);
    assert_eq!(structs, 5); // VarDecl, FnDecl, Param, Whitespace, Comment
    assert_eq!(enums, 2); // Stmt, Expr
    assert_eq!(uses, 1);
    assert_eq!(find_language_type(&m), Some("Stmt".to_string()));
    assert_eq!(extract_grammar_name(&m), Some("lang".to_string()));
}

// ── 34. Enum variant named fields: all attrs extractable ────────────────────

#[test]
fn enum_variant_named_fields_attrs() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                BinOp {
                    lhs: Box<Expr>,
                    #[adze::leaf(text = "+")]
                    _op: (),
                    rhs: Box<Expr>,
                },
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        if let Fields::Named(ref named) = e.variants[0].fields {
            let field_names: Vec<_> = named
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect();
            assert_eq!(field_names, vec!["lhs", "_op", "rhs"]);
            // Middle field has leaf attr
            let op_field = &named.named[1];
            assert!(op_field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        } else {
            panic!("Expected named fields");
        }
    } else {
        panic!("Expected enum");
    }
}

// ── 35. try_extract_inner_type: Vec<Spanned<T>> with Spanned in skip ────────

#[test]
fn extract_vec_spanned_inner_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Spanned<Number>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    // Extracting from Vec yields the direct inner type Spanned<Number>
    assert_eq!(inner.to_token_stream().to_string(), "Spanned < Number >");
}
