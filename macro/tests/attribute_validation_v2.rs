//! Comprehensive attribute-validation tests for the adze proc-macro crate.
//!
//! Tests exercise the public helpers re-exported from `adze-common` (via
//! `adze-common-syntax-core`), `syn`-level attribute parsing, and structural
//! patterns that the macro expansion relies on.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemEnum, ItemMod, ItemStruct, Token, Type, parse_quote};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Returns `true` when `attr` has a two-segment path `adze::<name>`.
fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

/// Collects all adze attribute names from a slice of attributes.
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

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    syn::parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    syn::parse2(tokens).expect("failed to parse enum")
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn non_leaf_set() -> HashSet<&'static str> {
    let mut s = HashSet::new();
    s.insert("Spanned");
    s.insert("Box");
    s.insert("Option");
    s.insert("Vec");
    s
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Attribute parsing patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn attr_parse_text_string_literal() {
    let nv: NameValueExpr = parse_quote!(text = "+");
    assert_eq!(nv.path.to_string(), "text");
    match &nv.expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => assert_eq!(s.value(), "+"),
        other => panic!("expected string literal, got: {}", other.to_token_stream()),
    }
}

#[test]
fn attr_parse_pattern_raw_string() {
    let nv: NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nv.path.to_string(), "pattern");
    match &nv.expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => assert_eq!(s.value(), r"\d+"),
        other => panic!(
            "expected raw string literal, got: {}",
            other.to_token_stream()
        ),
    }
}

#[test]
fn attr_parse_transform_closure() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse::<i32>().unwrap());
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(&nv.expr, syn::Expr::Closure(_)));
}

#[test]
fn attr_parse_boolean_true() {
    let nv: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nv.path.to_string(), "non_empty");
    match &nv.expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(b),
            ..
        }) => assert!(b.value),
        other => panic!("expected bool, got: {}", other.to_token_stream()),
    }
}

#[test]
fn attr_parse_boolean_false() {
    let nv: NameValueExpr = parse_quote!(non_empty = false);
    match &nv.expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(b),
            ..
        }) => assert!(!b.value),
        other => panic!("expected bool, got: {}", other.to_token_stream()),
    }
}

#[test]
fn attr_parse_multiple_name_values() {
    let params: Punctuated<NameValueExpr, Token![,]> =
        syn::parse_quote!(pattern = r"\d+", transform = |v| v.parse::<u64>().unwrap());
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].path.to_string(), "pattern");
    assert_eq!(params[1].path.to_string(), "transform");
}

#[test]
fn attr_parse_text_with_special_chars() {
    let nv: NameValueExpr = parse_quote!(text = "::=");
    match &nv.expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => assert_eq!(s.value(), "::="),
        other => panic!("expected string literal, got: {}", other.to_token_stream()),
    }
}

#[test]
fn attr_parse_complex_regex_pattern() {
    let nv: NameValueExpr = parse_quote!(pattern = r"[a-zA-Z_]\w*");
    match &nv.expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => assert_eq!(s.value(), r"[a-zA-Z_]\w*"),
        other => panic!("expected string literal, got: {}", other.to_token_stream()),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Type annotation patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn type_extract_option_inner() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = non_leaf_set();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

#[test]
fn type_extract_vec_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = non_leaf_set();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn type_extract_box_inner() {
    let ty: Type = parse_quote!(Box<Expr>);
    let skip = non_leaf_set();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    assert_eq!(inner.to_token_stream().to_string(), "Expr");
}

#[test]
fn type_extract_returns_false_for_mismatch() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = non_leaf_set();
    let (_inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn type_extract_plain_type_returns_false() {
    let ty: Type = parse_quote!(i32);
    let skip = non_leaf_set();
    assert!(!try_extract_inner_type(&ty, "Option", &skip).1);
    assert!(!try_extract_inner_type(&ty, "Vec", &skip).1);
    assert!(!try_extract_inner_type(&ty, "Box", &skip).1);
}

#[test]
fn type_filter_strips_option() {
    let ty: Type = parse_quote!(Option<Number>);
    let skip = non_leaf_set();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Number");
}

#[test]
fn type_filter_strips_vec() {
    let ty: Type = parse_quote!(Vec<Number>);
    let skip = non_leaf_set();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Number");
}

#[test]
fn type_filter_strips_nested_wrappers() {
    let ty: Type = parse_quote!(Vec<Box<Expr>>);
    let skip = non_leaf_set();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expr");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Derive patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn derive_debug_clone_with_leaf_attr() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        struct Number {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    };
    assert_eq!(s.ident, "Number");
    let derive_attr = s
        .attrs
        .iter()
        .find(|a| a.path().is_ident("derive"))
        .expect("derive attr missing");
    assert!(derive_attr.to_token_stream().to_string().contains("Debug"));
}

#[test]
fn derive_partialeq_eq_with_language_attr() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug, PartialEq, Eq)]
        #[adze::language]
        enum Expr {
            #[adze::leaf(text = "+")]
            Plus,
        }
    };
    let names = adze_attr_names(&e.attrs);
    assert!(names.contains(&"language".to_string()));
}

#[test]
fn derive_hash_with_grammar_types() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        struct Token {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let has_derive = s.attrs.iter().any(|a| a.path().is_ident("derive"));
    assert!(has_derive);
    let has_leaf = s
        .fields
        .iter()
        .any(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    assert!(has_leaf);
}

#[test]
fn derive_serde_with_leaf_struct() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    let derive_str = s
        .attrs
        .iter()
        .find(|a| a.path().is_ident("derive"))
        .unwrap()
        .to_token_stream()
        .to_string();
    assert!(derive_str.contains("Serialize"));
    assert!(derive_str.contains("Deserialize"));
}

#[test]
fn derive_default_with_skip_field() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Default)]
        struct MyNode {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let field_names: Vec<_> = s
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect();
    assert_eq!(field_names, ["value", "visited"]);
}

#[test]
fn derive_copy_clone_with_unit_variant_leaf() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug, Copy, Clone)]
        enum Operator {
            #[adze::leaf(text = "+")]
            Add,
            #[adze::leaf(text = "-")]
            Sub,
        }
    };
    assert_eq!(e.variants.len(), 2);
    for v in &e.variants {
        assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

#[test]
fn derive_multiple_attrs_ordering_preserved() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(s.attrs[0].path().is_ident("derive"));
    assert!(is_adze_attr(&s.attrs[1], "extra"));
}

#[test]
fn derive_empty_derives_still_valid() {
    // Parsing an enum with no custom derives, only adze attrs
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        enum Expr {
            #[adze::leaf(text = "x")]
            X,
        }
    };
    assert!(!e.attrs.iter().any(|a| a.path().is_ident("derive")));
    assert!(is_adze_attr(&e.attrs[0], "language"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Visibility patterns (7 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn vis_pub_struct_with_language() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Program {
            stmts: Vec<Stmt>,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
}

#[test]
fn vis_pub_crate_struct_with_language() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub(crate) struct Program {
            stmts: Vec<Stmt>,
        }
    };
    match &s.vis {
        syn::Visibility::Restricted(r) => {
            assert_eq!(r.path.to_token_stream().to_string(), "crate");
        }
        other => panic!("expected pub(crate), got: {}", other.to_token_stream()),
    }
}

#[test]
fn vis_private_struct_with_extra() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Inherited));
}

#[test]
fn vis_pub_enum_with_language() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            #[adze::leaf(text = "x")]
            X,
        }
    };
    assert!(matches!(e.vis, syn::Visibility::Public(_)));
}

#[test]
fn vis_pub_field_in_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"\w+")]
            pub name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Public(_)));
}

#[test]
fn vis_private_field_in_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Inherited));
}

#[test]
fn vis_pub_module_for_grammar() {
    let m = parse_mod(quote! {
        pub mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Public(_)));
    assert_eq!(m.ident, "grammar");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Generic type patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn generic_option_field_type_extraction() {
    let ty: Type = parse_quote!(Option<Number>);
    let skip = non_leaf_set();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

#[test]
fn generic_vec_field_type_extraction() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let skip = non_leaf_set();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(inner.to_token_stream().to_string(), "Statement");
}

#[test]
fn generic_box_recursive_type_extraction() {
    let ty: Type = parse_quote!(Box<Expr>);
    let skip = non_leaf_set();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    assert_eq!(inner.to_token_stream().to_string(), "Expr");
}

#[test]
fn generic_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<Number>>);
    let skip = non_leaf_set();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Number");
}

#[test]
fn generic_vec_box_nested() {
    let ty: Type = parse_quote!(Vec<Box<Expr>>);
    let skip = non_leaf_set();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expr");
}

#[test]
fn generic_spanned_type_extraction() {
    let ty: Type = parse_quote!(Spanned<Number>);
    let skip = non_leaf_set();
    let (inner, found) = try_extract_inner_type(&ty, "Spanned", &skip);
    assert!(found);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

#[test]
fn generic_wrap_leaf_option_type() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = non_leaf_set();
    let wrapped = wrap_leaf_type(&ty, &skip);
    // wrap_leaf_type should preserve the outer Option but replace the inner
    let wrapped_str = wrapped.to_token_stream().to_string();
    assert!(
        wrapped_str.contains("Option"),
        "wrapped type should contain Option: {wrapped_str}"
    );
}

#[test]
fn generic_wrap_leaf_vec_spanned() {
    let ty: Type = parse_quote!(Vec<Spanned<i32>>);
    let skip = non_leaf_set();
    let wrapped = wrap_leaf_type(&ty, &skip);
    let wrapped_str = wrapped.to_token_stream().to_string();
    assert!(
        wrapped_str.contains("Vec"),
        "wrapped type should contain Vec: {wrapped_str}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Error display (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn error_malformed_name_value_missing_eq() {
    let result = syn::parse2::<NameValueExpr>(quote!(text));
    assert!(result.is_err(), "missing `=` should fail");
}

#[test]
fn error_malformed_name_value_missing_value() {
    let result = syn::parse2::<NameValueExpr>(quote!(text =));
    assert!(result.is_err(), "missing value after `=` should fail");
}

#[test]
fn error_empty_name_value_stream() {
    let result = syn::parse2::<NameValueExpr>(TokenStream::new());
    assert!(result.is_err(), "empty token stream should fail");
}

#[test]
fn error_field_then_params_empty() {
    let result = syn::parse2::<FieldThenParams>(TokenStream::new());
    assert!(
        result.is_err(),
        "empty input should fail for FieldThenParams"
    );
}

#[test]
fn error_punctuated_nv_trailing_comma() {
    // Trailing comma in punctuated list should parse fine via parse_terminated
    let result: syn::Result<Punctuated<NameValueExpr, Token![,]>> = syn::parse::Parser::parse2(
        Punctuated::<NameValueExpr, Token![,]>::parse_terminated,
        quote!(text = "+",),
    );
    // parse_terminated allows trailing comma
    assert!(result.is_ok(), "trailing comma should be accepted");
    assert_eq!(result.unwrap().len(), 1);
}

#[test]
fn error_name_value_integer_key() {
    // Key must be an identifier
    let result = syn::parse2::<NameValueExpr>(quote!(42 = "hello"));
    assert!(result.is_err(), "integer key should fail");
}

#[test]
fn error_double_eq_in_name_value() {
    // `text == "+"` is not valid for NameValueExpr
    let result = syn::parse2::<NameValueExpr>(quote!(text == "+"));
    // The `==` is not a single `=` token; syn should reject this
    assert!(result.is_err(), "double == should fail");
}

#[test]
fn error_syn_parse_malformed_module() {
    // A module with no brace body should parse into ItemMod with None content
    let m: ItemMod = parse_quote!(
        mod mymod;
    );
    assert!(
        m.content.is_none(),
        "semicolon module should have no content"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Module structure (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn module_grammar_attr_recognized() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "x")]
                X,
            }
        }
    });
    let names = adze_attr_names(&m.attrs);
    assert!(names.contains(&"grammar".to_string()));
}

#[test]
fn module_language_type_found() {
    let m = parse_mod(quote! {
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "x")]
                X,
            }
        }
    });
    let items = &m.content.unwrap().1;
    let has_language = items.iter().any(|item| {
        if let syn::Item::Enum(e) = item {
            e.attrs.iter().any(|a| is_adze_attr(a, "language"))
        } else {
            false
        }
    });
    assert!(has_language, "should find language-annotated enum");
}

#[test]
fn module_extra_type_found() {
    let m = parse_mod(quote! {
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = &m.content.unwrap().1;
    let has_extra = items.iter().any(|item| {
        if let syn::Item::Struct(s) = item {
            s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
        } else {
            false
        }
    });
    assert!(has_extra, "should find extra-annotated struct");
}

#[test]
fn module_multiple_types_in_grammar() {
    let m = parse_mod(quote! {
        mod grammar {
            #[adze::language]
            pub struct Program {
                stmt: Statement,
            }

            pub enum Statement {
                #[adze::leaf(pattern = r"\w+")]
                Ident(String),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = &m.content.unwrap().1;
    assert_eq!(items.len(), 3, "module should contain 3 items");
}

#[test]
fn module_grammar_attr_contains_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(text = "x")]
                X,
            }
        }
    });
    let grammar_attr = m
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "grammar"))
        .expect("grammar attr missing");
    let grammar_name: syn::LitStr = grammar_attr.parse_args().unwrap();
    assert_eq!(grammar_name.value(), "arithmetic");
}

#[test]
fn module_nested_struct_fields_have_leaf() {
    let m = parse_mod(quote! {
        mod grammar {
            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
            }
        }
    });
    let items = &m.content.unwrap().1;
    let number = items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item
            && s.ident == "Number"
        {
            return Some(s);
        }
        None
    });
    assert!(number.is_some());
    let field = number.unwrap().fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn module_enum_variants_with_prec_attrs() {
    let m = parse_mod(quote! {
        mod grammar {
            #[adze::language]
            pub enum Expr {
                #[adze::leaf(pattern = r"\d+")]
                Number(i32),
                #[adze::prec_left(1)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_right(2)]
                Pow(Box<Expr>, Box<Expr>),
                #[adze::prec(3)]
                Compare(Box<Expr>, Box<Expr>),
            }
        }
    });
    let items = &m.content.unwrap().1;
    let expr_enum = items.iter().find_map(|item| {
        if let syn::Item::Enum(e) = item
            && e.ident == "Expr"
        {
            return Some(e);
        }
        None
    });
    let expr = expr_enum.expect("Expr enum not found");
    let variant_attrs: Vec<_> = expr
        .variants
        .iter()
        .flat_map(|v| adze_attr_names(&v.attrs))
        .collect();
    assert!(variant_attrs.contains(&"leaf".to_string()));
    assert!(variant_attrs.contains(&"prec_left".to_string()));
    assert!(variant_attrs.contains(&"prec_right".to_string()));
    assert!(variant_attrs.contains(&"prec".to_string()));
}

#[test]
fn module_word_attr_on_struct() {
    let m = parse_mod(quote! {
        mod grammar {
            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let items = &m.content.unwrap().1;
    let ident_struct = items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item
            && s.ident == "Identifier"
        {
            return Some(s);
        }
        None
    });
    let s = ident_struct.expect("Identifier struct not found");
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Additional attribute interaction tests (bonus: 8 more → 63 total)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn field_then_params_basic_parsing() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ",")]
        ()
    );
    assert!(ftp.field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    assert!(matches!(ftp.field.ty, Type::Tuple(_)));
}

#[test]
fn field_then_params_with_trailing_params() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ";")]
        (),
        non_empty = true
    );
    assert!(!ftp.params.is_empty());
    assert_eq!(ftp.params[0].path.to_string(), "non_empty");
}

#[test]
fn leaf_attr_text_and_pattern_separate() {
    // Verify we can parse leaf attrs with text
    let attr: Attribute = parse_quote!(#[adze::leaf(text = "+")]);
    let params = leaf_params(&attr);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");

    // And leaf attrs with pattern
    let attr2: Attribute = parse_quote!(#[adze::leaf(pattern = r"\d+")]);
    let params2 = leaf_params(&attr2);
    assert_eq!(params2.len(), 1);
    assert_eq!(params2[0].path.to_string(), "pattern");
}

#[test]
fn leaf_attr_with_all_three_params() {
    let attr: Attribute = parse_quote!(
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
    );
    let params = leaf_params(&attr);
    assert_eq!(params.len(), 2);
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_string()));
    assert!(names.contains(&"transform".to_string()));
}

#[test]
fn struct_fields_count_matches_expected() {
    let s = parse_struct(quote! {
        struct NumberList {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            numbers: Vec<Number>,
        }
    });
    assert_eq!(s.fields.iter().count(), 1);
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ident.as_ref().unwrap(), "numbers");
    // Should have both repeat and delimited attrs
    let attr_names = adze_attr_names(&field.attrs);
    assert!(attr_names.contains(&"repeat".to_string()));
    assert!(attr_names.contains(&"delimited".to_string()));
}

#[test]
fn enum_mixed_variant_kinds() {
    let e = parse_enum(quote! {
        #[adze::language]
        enum Expr {
            // Unit variant with leaf
            #[adze::leaf(text = "nil")]
            Nil,
            // Unnamed tuple variant
            Number(
                #[adze::leaf(pattern = r"\d+")]
                String
            ),
            // Named struct variant
            BinOp {
                #[adze::leaf(text = "+")]
                _op: (),
                left: Box<Expr>,
                right: Box<Expr>,
            },
        }
    });
    assert_eq!(e.variants.len(), 3);
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

#[test]
fn external_attr_on_struct() {
    let s = parse_struct(quote! {
        #[adze::external]
        struct IndentToken {
            #[adze::leaf(pattern = r"\t+")]
            _indent: (),
        }
    });
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

#[test]
fn skip_attr_with_default_value() {
    let s = parse_struct(quote! {
        struct MyNode {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(0)]
            counter: usize,
        }
    });
    let skip_field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "counter"))
        .unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}
