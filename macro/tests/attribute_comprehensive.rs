#![allow(clippy::needless_range_loop)]

//! Comprehensive attribute tests for the adze proc-macro crate.
//!
//! Tests cover attribute parsing, common-crate utilities (`NameValueExpr`,
//! `FieldThenParams`, type extraction/wrapping helpers), complex grammar
//! structures, and edge-case attribute interactions.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemEnum, ItemMod, ItemStruct, Token, Type, parse_quote};

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

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

// ── 1. NameValueExpr parsing: string value ──────────────────────────────────

#[test]
fn name_value_expr_string_value() {
    let nv: NameValueExpr = parse_quote!(text = "hello");
    assert_eq!(nv.path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "hello");
    } else {
        panic!("Expected string literal");
    }
}

// ── 2. NameValueExpr parsing: raw string value ──────────────────────────────

#[test]
fn name_value_expr_raw_string_pattern() {
    let nv: NameValueExpr = parse_quote!(pattern = r"\d+\.\d+");
    assert_eq!(nv.path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), r"\d+\.\d+");
    } else {
        panic!("Expected raw string literal");
    }
}

// ── 3. NameValueExpr parsing: boolean value ─────────────────────────────────

#[test]
fn name_value_expr_bool_value() {
    let nv: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nv.path.to_string(), "non_empty");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Bool(b),
        ..
    }) = &nv.expr
    {
        assert!(b.value);
    } else {
        panic!("Expected bool literal");
    }
}

// ── 4. NameValueExpr parsing: closure value ─────────────────────────────────

#[test]
fn name_value_expr_closure_value() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse::<u64>().unwrap());
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(nv.expr, syn::Expr::Closure(_)));
}

// ── 5. FieldThenParams: bare type only ──────────────────────────────────────

#[test]
fn field_then_params_bare_type() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "String");
}

// ── 6. FieldThenParams: type with one param ─────────────────────────────────

#[test]
fn field_then_params_single_param() {
    let ftp: FieldThenParams = parse_quote!(u32, non_empty = true);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "non_empty");
}

// ── 7. FieldThenParams: unit type with multiple params ──────────────────────

#[test]
fn field_then_params_unit_type_with_params() {
    let ftp: FieldThenParams = parse_quote!((), text = ",", other = 5);
    assert_eq!(ftp.params.len(), 2);
    let names: Vec<_> = ftp.params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["text", "other"]);
}

// ── 8. try_extract_inner_type: Vec<Box<T>> with Box in skip set ─────────────

#[test]
fn extract_inner_vec_of_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    // Extracts through Vec, giving Box<String>
    assert_eq!(inner.to_token_stream().to_string(), "Box < String >");
}

// ── 9. try_extract_inner_type: Option<Vec<T>> looking for Vec ───────────────

#[test]
fn extract_inner_option_vec_not_in_skip() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    // Option is not in skip set, so Vec inside it won't be found
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

// ── 10. try_extract_inner_type: Option<Vec<T>> with Option in skip ──────────

#[test]
fn extract_inner_option_vec_with_option_in_skip() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

// ── 11. filter_inner_type: nested Box<Box<T>> ──────────────────────────────

#[test]
fn filter_nested_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<u32>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "u32");
}

// ── 12. filter_inner_type: non-skip type left intact ────────────────────────

#[test]
fn filter_non_skip_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Vec < String >");
}

// ── 13. wrap_leaf_type: nested Option<Vec<T>> ───────────────────────────────

#[test]
fn wrap_nested_option_vec() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Vec < adze :: WithLeaf < i32 > > >"
    );
}

// ── 14. wrap_leaf_type: plain type without skip set ─────────────────────────

#[test]
fn wrap_plain_type_empty_skip() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(u64);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < u64 >"
    );
}

// ── 15. Grammar module: multiple item kinds preserved ───────────────────────

#[test]
fn grammar_module_preserves_all_item_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use std::collections::HashMap;

            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }

            pub struct Helper {
                data: String,
            }

            #[adze::extra]
            struct Whitespace {}
        }
    });
    let (_, items) = m.content.unwrap();
    // use, enum, struct, struct = 4 items
    assert_eq!(items.len(), 4);
}

// ── 16. Enum: variant with named fields preserves field names ───────────────

#[test]
fn enum_variant_named_fields_preserved() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            BinOp {
                #[adze::leaf(text = "+")]
                _op: (),
                lhs: Box<Expr>,
                rhs: Box<Expr>,
            },
        }
    };
    let variant = &e.variants[0];
    if let Fields::Named(ref named) = variant.fields {
        let names: Vec<_> = named
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["_op", "lhs", "rhs"]);
    } else {
        panic!("Expected named fields");
    }
}

// ── 17. Enum: mixed variant kinds (unit, tuple, struct) ─────────────────────

#[test]
fn enum_mixed_variant_kinds() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Token {
            #[adze::leaf(text = "+")]
            Plus,
            Number(
                #[adze::leaf(pattern = r"\d+")]
                String
            ),
            Complex {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            },
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

// ── 18. Struct with multiple leaf fields ────────────────────────────────────

#[test]
fn struct_multiple_leaf_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Assignment {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let leaf_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
        .count();
    assert_eq!(leaf_count, 3);
}

// ── 19. Leaf param: text value extraction ───────────────────────────────────

#[test]
fn leaf_text_value_extracted_correctly() {
    let s: ItemStruct = parse_quote! {
        pub struct Sep {
            #[adze::leaf(text = "::")]
            _sep: (),
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
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "::");
    } else {
        panic!("Expected string literal");
    }
}

// ── 20. Leaf param: pattern with special regex chars ────────────────────────

#[test]
fn leaf_pattern_regex_special_chars() {
    let s: ItemStruct = parse_quote! {
        pub struct FloatLit {
            #[adze::leaf(pattern = r"-?\d+(\.\d+)?([eE][+-]?\d+)?")]
            value: String,
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
    assert_eq!(params[0].path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), r"-?\d+(\.\d+)?([eE][+-]?\d+)?");
    } else {
        panic!("Expected string literal");
    }
}

// ── 21. Delimited: inner leaf attribute parsed from nested position ─────────

#[test]
fn delimited_inner_leaf_text_is_parseable() {
    let s: ItemStruct = parse_quote! {
        pub struct Args {
            #[adze::delimited(
                #[adze::leaf(text = ";")]
                ()
            )]
            stmts: Vec<Stmt>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    // The delimited attribute should parse as a FieldThenParams
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    // The inner field is `()` and it should have a leaf attribute
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(inner_leaf);
    assert_eq!(params[0].path.to_string(), "text");
}

// ── 22. Grammar name: special characters allowed in string ──────────────────

#[test]
fn grammar_name_with_underscores_and_digits() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_grammar_v2")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "my_grammar_v2");
    } else {
        panic!("Expected string literal");
    }
}

// ── 23. Precedence: all three kinds on separate variants ────────────────────

#[test]
fn all_precedence_kinds_coexist() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(5)]
            Eq(Box<Expr>, Box<Expr>),
            #[adze::prec_left(10)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_right(15)]
            Assign(Box<Expr>, Box<Expr>),
        }
    };
    let attrs: Vec<_> = e
        .variants
        .iter()
        .flat_map(|v| adze_attr_names(&v.attrs))
        .collect();
    assert_eq!(attrs, vec!["prec", "prec_left", "prec_right"]);

    // Verify values parse correctly
    let values: Vec<i32> = e
        .variants
        .iter()
        .map(|v| {
            let attr = &v.attrs[0];
            let expr: syn::Expr = attr.parse_args().unwrap();
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(i),
                ..
            }) = expr
            {
                i.base10_parse::<i32>().unwrap()
            } else {
                panic!("Expected int literal");
            }
        })
        .collect();
    assert_eq!(values, vec![5, 10, 15]);
}

// ── 24. Extra + external on different structs in same module ────────────────

#[test]
fn extra_and_external_in_same_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {}

            #[adze::extra]
            struct Whitespace {}

            #[adze::external]
            struct IndentToken;
        }
    });
    let (_, items) = m.content.unwrap();
    let mut found_extra = false;
    let mut found_external = false;
    for item in &items {
        if let syn::Item::Struct(s) = item {
            if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) {
                found_extra = true;
            }
            if s.attrs.iter().any(|a| is_adze_attr(a, "external")) {
                found_external = true;
            }
        }
    }
    assert!(found_extra);
    assert!(found_external);
}

// ── 25. try_extract_inner_type: deeply nested skips ─────────────────────────

#[test]
fn extract_inner_deeply_nested_skips() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Vec<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

// ── 26. wrap_leaf_type: Box is not in skip set → gets wrapped ───────────────

#[test]
fn wrap_box_when_not_in_skip_set() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Box<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < Box < i32 > >"
    );
}

// ── 27. Attribute count: all 12 known attributes enumerated ─────────────────

#[test]
fn all_twelve_known_attributes() {
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
    // Each name should be unique
    let set: HashSet<_> = known.iter().collect();
    assert_eq!(set.len(), 12);
}

// ── 28. Struct: word + leaf combined ────────────────────────────────────────

#[test]
fn word_with_leaf_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    // Verify the pattern value
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "pattern");
}

// ── 29. Enum: leaf on tuple variant with transform ──────────────────────────

#[test]
fn leaf_tuple_variant_with_transform() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<f64>().unwrap())]
                f64
            ),
        }
    };
    let field = match &e.variants[0].fields {
        Fields::Unnamed(u) => &u.unnamed[0],
        _ => panic!("Expected unnamed fields"),
    };
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_owned()));
    assert!(names.contains(&"transform".to_owned()));
}

// ── 30. Grammar module: visibility and ident preserved ──────────────────────

#[test]
fn grammar_module_visibility_and_ident() {
    let m = parse_mod(quote! {
        #[adze::grammar("calculator")]
        pub mod calc_grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    assert_eq!(m.ident.to_string(), "calc_grammar");
    assert!(matches!(m.vis, syn::Visibility::Public(_)));
}
