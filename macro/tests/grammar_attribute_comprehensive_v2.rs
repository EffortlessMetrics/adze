#![allow(clippy::needless_range_loop)]

//! Comprehensive v2 tests for `#[adze::grammar]` attribute processing in adze-macro.
//!
//! Covers grammar/language/leaf/extra/word/external/prec/skip/repeat/delimited
//! attribute parsing on syn AST, attribute interaction, edge cases in attribute
//! values, complex type annotations, enum variant structures, and error detection.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Variant, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn has_adze_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| is_adze_attr(a, name))
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
        Item::Enum(e) if has_adze_attr(&e.attrs, "language") => Some(e.ident.to_string()),
        Item::Struct(s) if has_adze_attr(&s.attrs, "language") => Some(s.ident.to_string()),
        _ => None,
    })
}

fn find_struct<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| match i {
        Item::Struct(s) if s.ident == name => Some(s),
        _ => None,
    })
}

fn find_enum<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| match i {
        Item::Enum(e) if e.ident == name => Some(e),
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

fn field_names(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect()
}

fn variant_names(e: &ItemEnum) -> Vec<String> {
    e.variants.iter().map(|v| v.ident.to_string()).collect()
}

fn field_type_str(s: &ItemStruct, name: &str) -> String {
    s.fields
        .iter()
        .find(|f| f.ident.as_ref().map(|i| i == name).unwrap_or(false))
        .map(|f| f.ty.to_token_stream().to_string())
        .unwrap_or_default()
}

fn variant_field_count(v: &Variant) -> usize {
    v.fields.len()
}

fn count_items_with_attr(m: &ItemMod, attr_name: &str) -> usize {
    module_items(m)
        .iter()
        .filter(|item| match item {
            Item::Struct(s) => has_adze_attr(&s.attrs, attr_name),
            Item::Enum(e) => has_adze_attr(&e.attrs, attr_name),
            _ => false,
        })
        .count()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Grammar attribute parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_attr_recognized_on_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert!(has_adze_attr(&m.attrs, "grammar"));
}

#[test]
fn grammar_name_simple_string() {
    let m = parse_mod(quote! {
        #[adze::grammar("calc")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert_eq!(extract_grammar_name(&m), Some("calc".into()));
}

#[test]
fn grammar_name_with_special_chars() {
    let m = parse_mod(quote! {
        #[adze::grammar("my-lang_v3.1")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert_eq!(extract_grammar_name(&m), Some("my-lang_v3.1".into()));
}

#[test]
fn grammar_name_unicode() {
    let m = parse_mod(quote! {
        #[adze::grammar("语法")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert_eq!(extract_grammar_name(&m), Some("语法".into()));
}

#[test]
fn grammar_name_empty_string_is_valid() {
    let m = parse_mod(quote! {
        #[adze::grammar("")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert_eq!(extract_grammar_name(&m), Some(String::new()));
}

#[test]
fn grammar_name_with_spaces() {
    let m = parse_mod(quote! {
        #[adze::grammar("my grammar name")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert_eq!(extract_grammar_name(&m), Some("my grammar name".into()));
}

#[test]
fn grammar_name_absent_without_attr() {
    let m = parse_mod(quote! {
        mod g { pub struct R {} }
    });
    assert_eq!(extract_grammar_name(&m), None);
}

#[test]
fn grammar_module_ident_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod my_parser { #[adze::language] pub struct R {} }
    });
    assert_eq!(m.ident, "my_parser");
}

#[test]
fn grammar_attr_coexists_with_cfg() {
    let m = parse_mod(quote! {
        #[cfg(test)]
        #[adze::grammar("test")]
        #[allow(dead_code)]
        mod g { #[adze::language] pub struct R {} }
    });
    assert!(has_adze_attr(&m.attrs, "grammar"));
    assert_eq!(m.attrs.len(), 3);
}

#[test]
fn grammar_module_pub_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        pub mod g { #[adze::language] pub struct R {} }
    });
    assert!(matches!(m.vis, syn::Visibility::Public(_)));
}

#[test]
fn grammar_module_pub_crate_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        pub(crate) mod g { #[adze::language] pub struct R {} }
    });
    assert!(matches!(m.vis, syn::Visibility::Restricted(_)));
}

#[test]
fn grammar_module_inherited_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert!(matches!(m.vis, syn::Visibility::Inherited));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Language attribute parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn language_attr_on_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Root".into()));
}

#[test]
fn language_attr_on_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Expr".into()));
}

#[test]
fn language_count_exactly_one() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root { v: Child, }
            pub struct Child { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    assert_eq!(count_items_with_attr(&m, "language"), 1);
}

#[test]
fn language_not_present_when_absent() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            pub struct Root { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    assert_eq!(find_language_type(&m), None);
}

#[test]
fn language_struct_fields_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                child: Option<Child>,
            }
            pub struct Child { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert_eq!(field_names(s), vec!["name", "child"]);
}

#[test]
fn language_enum_variants_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Token {
                #[adze::leaf(text = "+")]
                Plus,
                #[adze::leaf(text = "-")]
                Minus,
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    let e = find_enum(&m, "Token").unwrap();
    assert_eq!(variant_names(e), vec!["Plus", "Minus", "Number"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Leaf attribute parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn leaf_text_attr_on_unit_variant() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Kw {
                #[adze::leaf(text = "if")]
                If,
                #[adze::leaf(text = "else")]
                Else,
            }
        }
    });
    let e = find_enum(&m, "Kw").unwrap();
    for v in &e.variants {
        assert!(has_adze_attr(&v.attrs, "leaf"));
    }
}

#[test]
fn leaf_pattern_attr_on_struct_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Ident {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "leaf"));
}

#[test]
fn leaf_text_attr_parsed_value() {
    let s: ItemStruct = parse_quote! {
        pub struct Plus {
            #[adze::leaf(text = "+")]
            _op: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let args: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> = leaf_attr
        .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
        .unwrap();
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].path.to_string(), "text");
}

#[test]
fn leaf_pattern_and_transform_both_present() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let args: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> = leaf_attr
        .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
        .unwrap();
    let names: Vec<_> = args.iter().map(|a| a.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_string()));
    assert!(names.contains(&"transform".to_string()));
}

#[test]
fn leaf_on_unnamed_enum_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                i32
            ),
        }
    };
    let v = &e.variants[0];
    let field = v.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "leaf"));
}

#[test]
fn leaf_on_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "9")]
        pub struct BigDigit;
    };
    assert!(has_adze_attr(&s.attrs, "leaf"));
}

#[test]
fn leaf_pattern_with_complex_regex() {
    let s: ItemStruct = parse_quote! {
        pub struct FloatLit {
            #[adze::leaf(pattern = r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "leaf"));
}

#[test]
fn leaf_text_multichar_operator() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Op {
                #[adze::leaf(text = "===")]
                StrictEq,
                #[adze::leaf(text = "!==")]
                StrictNeq,
                #[adze::leaf(text = "<<=")]
                ShlAssign,
            }
        }
    });
    let e = find_enum(&m, "Op").unwrap();
    assert_eq!(e.variants.len(), 3);
    for v in &e.variants {
        assert!(has_adze_attr(&v.attrs, "leaf"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Error cases (missing attributes, invalid syntax)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn missing_grammar_attr_means_no_name() {
    let m = parse_mod(quote! {
        mod g { pub struct Root {} }
    });
    assert!(!has_adze_attr(&m.attrs, "grammar"));
    assert_eq!(extract_grammar_name(&m), None);
}

#[test]
fn grammar_attr_with_non_string_arg_yields_none() {
    // Grammar with integer literal instead of string — extraction returns None
    let m: ItemMod = syn::parse2(quote! {
        #[adze::grammar(42)]
        mod g { pub struct Root {} }
    })
    .unwrap();
    assert_eq!(extract_grammar_name(&m), None);
}

#[test]
fn grammar_attr_with_no_args_parse_fails() {
    // `#[adze::grammar]` with no args — parse_args should fail
    let m: ItemMod = syn::parse2(quote! {
        #[adze::grammar]
        mod g { pub struct Root {} }
    })
    .unwrap();
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let result = attr.parse_args::<syn::Expr>();
    assert!(result.is_err());
}

#[test]
fn no_language_attr_detection() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            pub struct Foo { #[adze::leaf(pattern = r"\d+")] v: String, }
            pub enum Bar { A, B, }
        }
    });
    assert_eq!(find_language_type(&m), None);
    assert_eq!(count_items_with_attr(&m, "language"), 0);
}

#[test]
fn grammar_on_empty_module_has_no_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("empty")]
        mod g {}
    });
    assert!(module_items(&m).is_empty());
    assert_eq!(find_language_type(&m), None);
}

#[test]
fn leaf_attr_missing_from_plain_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Foo {
            name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(!has_adze_attr(&field.attrs, "leaf"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Multiple grammars / multiple types in same module
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn module_with_multiple_structs() {
    let m = parse_mod(quote! {
        #[adze::grammar("multi")]
        mod g {
            #[adze::language]
            pub struct Program { stmt: Statement, }
            pub struct Statement { #[adze::leaf(pattern = r"\w+")] text: String, }
            pub struct Helper { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    assert_eq!(struct_names(&m), vec!["Program", "Statement", "Helper"]);
    assert_eq!(find_language_type(&m), Some("Program".into()));
}

#[test]
fn module_with_multiple_enums() {
    let m = parse_mod(quote! {
        #[adze::grammar("multi_enum")]
        mod g {
            #[adze::language]
            pub enum Expr { Lit(Literal), }
            pub enum Literal { Int(i32), Float(f64), }
        }
    });
    assert_eq!(enum_names(&m), vec!["Expr", "Literal"]);
}

#[test]
fn module_with_mixed_structs_and_enums() {
    let m = parse_mod(quote! {
        #[adze::grammar("mixed")]
        mod g {
            #[adze::language]
            pub struct Program { expr: Box<Expr>, }
            pub enum Expr { Number(i32), Add(Box<Expr>, Box<Expr>), }
            pub struct Number { v: i32, }
        }
    });
    assert_eq!(struct_names(&m), vec!["Program", "Number"]);
    assert_eq!(enum_names(&m), vec!["Expr"]);
}

#[test]
fn module_with_multiple_extra_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Code { #[adze::leaf(pattern = r"\w+")] t: String, }
            #[adze::extra]
            struct Whitespace { #[adze::leaf(pattern = r"\s")] _ws: (), }
            #[adze::extra]
            struct Comment { #[adze::leaf(pattern = r"//[^\n]*")] _c: (), }
        }
    });
    assert_eq!(count_items_with_attr(&m, "extra"), 2);
}

#[test]
fn module_preserves_item_order() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            use std::fmt;
            #[adze::language]
            pub enum Expr { Lit(i32), }
            pub struct Helper { v: i32, }
            #[adze::extra]
            struct Ws {}
        }
    });
    let items = module_items(&m);
    assert!(matches!(&items[0], Item::Use(_)));
    assert!(matches!(&items[1], Item::Enum(_)));
    assert!(matches!(&items[2], Item::Struct(s) if s.ident == "Helper"));
    assert!(matches!(&items[3], Item::Struct(s) if s.ident == "Ws"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Complex type annotations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn field_type_box() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root { child: Box<Child>, }
            pub struct Child { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert_eq!(field_type_str(s, "child"), "Box < Child >");
}

#[test]
fn field_type_option() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                val: Option<i32>,
            }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert_eq!(field_type_str(s, "val"), "Option < i32 >");
}

#[test]
fn field_type_vec() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root { items: Vec<Item>, }
            pub struct Item { #[adze::leaf(pattern = r"\w+")] v: String, }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert_eq!(field_type_str(s, "items"), "Vec < Item >");
}

#[test]
fn field_type_spanned() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            use adze::Spanned;
            #[adze::language]
            pub struct Root { child: Spanned<Child>, }
            pub struct Child { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert_eq!(field_type_str(s, "child"), "Spanned < Child >");
}

#[test]
fn field_type_vec_of_spanned() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            use adze::Spanned;
            #[adze::language]
            pub struct Root { items: Vec<Spanned<Child>>, }
            pub struct Child { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert_eq!(field_type_str(s, "items"), "Vec < Spanned < Child > >");
}

#[test]
fn field_type_option_box() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root { child: Option<Box<Child>>, }
            pub struct Child { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert_eq!(field_type_str(s, "child"), "Option < Box < Child > >");
}

#[test]
fn enum_recursive_box_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
                Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    let neg = &e.variants[1];
    assert_eq!(variant_field_count(neg), 2);
}

#[test]
fn enum_binary_op_box_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    let add = &e.variants[1];
    assert_eq!(variant_field_count(add), 3);
    assert!(has_adze_attr(&add.attrs, "prec_left"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Edge cases in attribute values
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_name_with_dots() {
    let m = parse_mod(quote! {
        #[adze::grammar("lang.v2.0")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert_eq!(extract_grammar_name(&m), Some("lang.v2.0".into()));
}

#[test]
fn grammar_name_with_slashes() {
    let m = parse_mod(quote! {
        #[adze::grammar("path/to/grammar")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert_eq!(extract_grammar_name(&m), Some("path/to/grammar".into()));
}

#[test]
fn leaf_text_with_escaped_quote() {
    let s: ItemStruct = parse_quote! {
        pub struct Escaped {
            #[adze::leaf(text = "\"")]
            _q: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "leaf"));
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let args: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> = leaf_attr
        .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
        .unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &args[0].expr
    {
        assert_eq!(s.value(), "\"");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn leaf_text_empty_string() {
    let s: ItemStruct = parse_quote! {
        pub struct Empty {
            #[adze::leaf(text = "")]
            _e: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let args: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> = leaf_attr
        .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
        .unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &args[0].expr
    {
        assert_eq!(s.value(), "");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn leaf_pattern_raw_string() {
    let s: ItemStruct = parse_quote! {
        pub struct Raw {
            #[adze::leaf(pattern = r#"["\n]+"#)]
            v: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "leaf"));
}

#[test]
fn prec_left_zero_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(0)]
            Add(Box<Expr>, Box<Expr>),
        }
    };
    assert!(has_adze_attr(&e.variants[0].attrs, "prec_left"));
}

#[test]
fn prec_right_high_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_right(100)]
            Power(Box<Expr>, Box<Expr>),
        }
    };
    assert!(has_adze_attr(&e.variants[0].attrs, "prec_right"));
}

#[test]
fn prec_no_assoc_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(5)]
            Compare(Box<Expr>, Box<Expr>),
        }
    };
    assert!(has_adze_attr(&e.variants[0].attrs, "prec"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Extra attribute
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extra_attr_recognized_on_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root { #[adze::leaf(pattern = r"\w+")] v: String, }
            #[adze::extra]
            struct Ws { #[adze::leaf(pattern = r"\s")] _ws: (), }
        }
    });
    let ws = find_struct(&m, "Ws").unwrap();
    assert!(has_adze_attr(&ws.attrs, "extra"));
}

#[test]
fn extra_attr_not_on_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root { #[adze::leaf(pattern = r"\w+")] v: String, }
        }
    });
    let root = find_struct(&m, "Root").unwrap();
    assert!(!has_adze_attr(&root.attrs, "extra"));
    assert!(has_adze_attr(&root.attrs, "language"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Word attribute
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn word_attr_recognized() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Code { ident: Identifier, }
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let ident = find_struct(&m, "Identifier").unwrap();
    assert!(has_adze_attr(&ident.attrs, "word"));
}

#[test]
fn word_attr_count_single() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Code { ident: Identifier, }
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    assert_eq!(count_items_with_attr(&m, "word"), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. External attribute
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn external_attr_recognized() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Code { #[adze::leaf(pattern = r"\w+")] t: String, }
            #[adze::external]
            struct IndentToken { #[adze::leaf(pattern = r"\t+")] _indent: (), }
        }
    });
    let indent = find_struct(&m, "IndentToken").unwrap();
    assert!(has_adze_attr(&indent.attrs, "external"));
}

#[test]
fn external_attr_count() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Code { #[adze::leaf(pattern = r"\w+")] t: String, }
            #[adze::external]
            struct Indent {}
            #[adze::external]
            struct Dedent {}
        }
    });
    assert_eq!(count_items_with_attr(&m, "external"), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Skip attribute
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn skip_attr_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct MyNode {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert!(!has_adze_attr(&fields[0].attrs, "skip"));
    assert!(has_adze_attr(&fields[1].attrs, "skip"));
}

#[test]
fn skip_attr_with_default_value() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::skip(0)]
            count: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "skip"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Repeat attribute
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn repeat_attr_on_vec_field() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "repeat"));
}

#[test]
fn repeat_non_empty_parsed() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let args: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> = attr
        .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
        .unwrap();
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].path.to_string(), "non_empty");
}

#[test]
fn repeat_without_non_empty() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(!has_adze_attr(&field.attrs, "repeat"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Delimited attribute
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn delimited_attr_on_vec_field() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "delimited"));
}

#[test]
fn delimited_combined_with_repeat() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::delimited(
                #[adze::leaf(text = ";")]
                ()
            )]
            #[adze::repeat(non_empty = true)]
            stmts: Vec<Stmt>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "delimited"));
    assert!(has_adze_attr(&field.attrs, "repeat"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Precedence attributes
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn prec_left_attr_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    };
    assert!(has_adze_attr(&e.variants[0].attrs, "prec_left"));
}

#[test]
fn prec_right_attr_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_right(2)]
            Cons(Box<Expr>, Box<Expr>),
        }
    };
    assert!(has_adze_attr(&e.variants[0].attrs, "prec_right"));
}

#[test]
fn prec_attr_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(3)]
            Eq(Box<Expr>, Box<Expr>),
        }
    };
    assert!(has_adze_attr(&e.variants[0].attrs, "prec"));
}

#[test]
fn multiple_prec_levels_in_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
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
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    assert_eq!(e.variants.len(), 6);
    // First has no prec
    assert!(!has_adze_attr(&e.variants[0].attrs, "prec_left"));
    // Add and Sub have prec_left
    assert!(has_adze_attr(&e.variants[1].attrs, "prec_left"));
    assert!(has_adze_attr(&e.variants[2].attrs, "prec_left"));
    // Mul and Div have prec_left
    assert!(has_adze_attr(&e.variants[3].attrs, "prec_left"));
    assert!(has_adze_attr(&e.variants[4].attrs, "prec_left"));
    // Pow has prec_right
    assert!(has_adze_attr(&e.variants[5].attrs, "prec_right"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Enum variant structures
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn enum_unit_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Kw {
            #[adze::leaf(text = "if")]
            If,
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
}

#[test]
fn enum_unnamed_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unnamed(_)));
    assert_eq!(variant_field_count(&e.variants[0]), 1);
}

#[test]
fn enum_named_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Neg {
                #[adze::leaf(text = "!")]
                _bang: (),
                value: Box<Expr>,
            }
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Named(_)));
    assert_eq!(variant_field_count(&e.variants[0]), 2);
}

#[test]
fn enum_mixed_variant_styles() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::leaf(text = "nil")]
                Nil,
                Neg {
                    #[adze::leaf(text = "-")]
                    _minus: (),
                    inner: Box<Expr>,
                },
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    assert!(matches!(e.variants[0].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[1].fields, Fields::Unit));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

#[test]
fn enum_variant_with_multiple_unnamed_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    assert_eq!(variant_field_count(&e.variants[0]), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Use statements in grammar modules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn use_statements_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            use std::fmt;
            use std::collections::HashMap;
            #[adze::language]
            pub struct Root { #[adze::leaf(pattern = r"\w+")] v: String, }
        }
    });
    let uses: Vec<_> = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Use(_)))
        .collect();
    assert_eq!(uses.len(), 2);
}

#[test]
fn adze_spanned_use_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            use adze::Spanned;
            #[adze::language]
            pub struct Root { child: Spanned<Child>, }
            pub struct Child { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    let uses: Vec<_> = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Use(_)))
        .collect();
    assert_eq!(uses.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Attribute listing / introspection
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_12_known_attrs() {
    let known = [
        "grammar",
        "language",
        "leaf",
        "skip",
        "prec",
        "prec_left",
        "prec_right",
        "delimited",
        "repeat",
        "extra",
        "external",
        "word",
    ];
    assert_eq!(known.len(), 12);
}

#[test]
fn adze_attr_names_extracted() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["language", "word"]);
}

#[test]
fn field_adze_attr_names() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::delimited(#[adze::leaf(text = ",")] ())]
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let names = adze_attr_names(&field.attrs);
    assert_eq!(names, vec!["delimited", "repeat"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Full grammar structure tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn arithmetic_grammar_structure() {
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
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _whitespace: (),
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("arithmetic".into()));
    assert_eq!(find_language_type(&m), Some("Expr".into()));
    assert_eq!(count_items_with_attr(&m, "extra"), 1);
    let e = find_enum(&m, "Expr").unwrap();
    assert_eq!(e.variants.len(), 5);
}

#[test]
fn json_like_grammar_structure() {
    let m = parse_mod(quote! {
        #[adze::grammar("json")]
        mod grammar {
            #[adze::language]
            pub enum Value {
                Null(#[adze::leaf(text = "null")] ()),
                Bool(#[adze::leaf(pattern = r"true|false")] String),
                Number(#[adze::leaf(pattern = r"-?\d+(\.\d+)?")] String),
                Str(
                    #[adze::leaf(text = "\"")] (),
                    #[adze::leaf(pattern = r#"[^"]*"#)] String,
                    #[adze::leaf(text = "\"")] (),
                ),
                Array(
                    #[adze::leaf(text = "[")] (),
                    #[adze::delimited(#[adze::leaf(text = ",")] ())]
                    Vec<Box<Value>>,
                    #[adze::leaf(text = "]")] (),
                ),
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("json".into()));
    let e = find_enum(&m, "Value").unwrap();
    assert_eq!(
        variant_names(e),
        vec!["Null", "Bool", "Number", "Str", "Array"]
    );
}

#[test]
fn grammar_with_all_attr_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("full")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub struct Program {
                stmts: Vec<Spanned<Statement>>,
            }

            pub enum Statement {
                Let {
                    #[adze::leaf(text = "let")]
                    _kw: (),
                    name: Identifier,
                    #[adze::leaf(text = "=")]
                    _eq: (),
                    value: Box<Expr>,
                },
            }

            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                Ident(Identifier),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_right(2)]
                Pow(Box<Expr>, #[adze::leaf(text = "**")] (), Box<Expr>),
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            #[adze::external]
            struct IndentToken {
                #[adze::leaf(pattern = r"\t+")]
                _indent: (),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _comment: (),
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("full".into()));
    assert_eq!(find_language_type(&m), Some("Program".into()));
    assert_eq!(count_items_with_attr(&m, "word"), 1);
    assert_eq!(count_items_with_attr(&m, "external"), 1);
    assert_eq!(count_items_with_attr(&m, "extra"), 2);
    assert_eq!(count_items_with_attr(&m, "language"), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 19. NameValueExpr parsing via adze_common
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn name_value_expr_text_param() {
    let nv: adze_common::NameValueExpr = parse_quote!(text = "+");
    assert_eq!(nv.path.to_string(), "text");
}

#[test]
fn name_value_expr_pattern_param() {
    let nv: adze_common::NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nv.path.to_string(), "pattern");
}

#[test]
fn name_value_expr_transform_closure() {
    let nv: adze_common::NameValueExpr = parse_quote!(transform = |v| v.parse().unwrap());
    assert_eq!(nv.path.to_string(), "transform");
}

#[test]
fn name_value_expr_non_empty_bool() {
    let nv: adze_common::NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nv.path.to_string(), "non_empty");
}

// ═══════════════════════════════════════════════════════════════════════════
// 20. Roundtrip: quote → parse → back to tokens
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_module_roundtrips_through_tokens() {
    let original = quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    };
    let parsed: ItemMod = syn::parse2(original.clone()).unwrap();
    let reparsed: ItemMod = syn::parse2(parsed.to_token_stream()).unwrap();
    assert_eq!(
        extract_grammar_name(&parsed),
        extract_grammar_name(&reparsed)
    );
    assert_eq!(find_language_type(&parsed), find_language_type(&reparsed));
}

#[test]
fn enum_with_attrs_roundtrips() {
    let original = quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Expr {
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    };
    let parsed: ItemMod = syn::parse2(original).unwrap();
    let reparsed: ItemMod = syn::parse2(parsed.to_token_stream()).unwrap();
    let e1 = find_enum(&parsed, "Expr").unwrap();
    let e2 = find_enum(&reparsed, "Expr").unwrap();
    assert_eq!(variant_names(e1), variant_names(e2));
    assert_eq!(
        has_adze_attr(&e1.variants[0].attrs, "prec_left"),
        has_adze_attr(&e2.variants[0].attrs, "prec_left"),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 21. Structural edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn struct_with_no_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root {}
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    assert_eq!(s.fields.len(), 0);
}

#[test]
fn struct_unit_no_braces() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "x")]
        pub struct Unit;
    };
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn enum_single_variant() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    assert_eq!(e.variants.len(), 1);
}

#[test]
fn enum_many_variants() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub enum Kw {
                #[adze::leaf(text = "if")] If,
                #[adze::leaf(text = "else")] Else,
                #[adze::leaf(text = "while")] While,
                #[adze::leaf(text = "for")] For,
                #[adze::leaf(text = "return")] Return,
                #[adze::leaf(text = "break")] Break,
                #[adze::leaf(text = "continue")] Continue,
                #[adze::leaf(text = "fn")] Fn,
                #[adze::leaf(text = "let")] Let,
                #[adze::leaf(text = "const")] Const,
            }
        }
    });
    let e = find_enum(&m, "Kw").unwrap();
    assert_eq!(e.variants.len(), 10);
    for v in &e.variants {
        assert!(has_adze_attr(&v.attrs, "leaf"));
    }
}

#[test]
fn deeply_nested_generic_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            use adze::Spanned;
            #[adze::language]
            pub struct Root { items: Vec<Spanned<Option<Box<Child>>>>, }
            pub struct Child { #[adze::leaf(pattern = r"\d+")] v: String, }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    let ty = field_type_str(s, "items");
    assert!(ty.contains("Vec"));
    assert!(ty.contains("Spanned"));
    assert!(ty.contains("Option"));
    assert!(ty.contains("Box"));
    assert!(ty.contains("Child"));
}

#[test]
fn unnamed_vec_in_enum_variant() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: u32,
            }
            #[adze::language]
            pub enum Expr {
                Numbers(
                    #[adze::repeat(non_empty = true)]
                    Vec<Number>
                ),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    let v = &e.variants[0];
    assert_eq!(variant_field_count(v), 1);
    let field = v.fields.iter().next().unwrap();
    assert!(has_adze_attr(&field.attrs, "repeat"));
}

#[test]
fn module_with_only_use_statements() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            use std::fmt;
            use std::collections::HashMap;
        }
    });
    assert_eq!(module_items(&m).len(), 2);
    assert_eq!(find_language_type(&m), None);
}

#[test]
fn field_with_multiple_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::delimited(#[adze::leaf(text = ",")] ())]
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
            #[adze::skip(false)]
            meta: bool,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    let items_attrs = adze_attr_names(&fields[0].attrs);
    assert_eq!(items_attrs, vec!["delimited", "repeat"]);
    let meta_attrs = adze_attr_names(&fields[1].attrs);
    assert_eq!(meta_attrs, vec!["skip"]);
}

#[test]
fn grammar_name_long_string() {
    let m = parse_mod(quote! {
        #[adze::grammar("a_very_long_grammar_name_that_tests_limits")]
        mod g { #[adze::language] pub struct R {} }
    });
    assert_eq!(
        extract_grammar_name(&m),
        Some("a_very_long_grammar_name_that_tests_limits".into())
    );
}
