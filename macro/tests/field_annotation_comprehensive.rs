#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for field-level annotation handling in the adze macro crate.
//!
//! Tests cover leaf annotation with text and regex patterns, skip field handling,
//! multiple annotations on the same field, field type inference (Vec → repeat,
//! Option → optional), field ordering, transform closures, field visibility,
//! and required vs optional fields.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
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

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn find_leaf_attr(attrs: &[Attribute]) -> &Attribute {
    attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap()
}

fn find_struct_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
}

fn expansion_skip_set() -> HashSet<&'static str> {
    ["Spanned", "Box", "Option", "Vec"].into_iter().collect()
}

// ── 1. Leaf annotation with text pattern on named field ─────────────────────

#[test]
fn field_leaf_text_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Punctuation {
            #[adze::leaf(text = "+")]
            _plus: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = find_leaf_attr(&field.attrs);
    let params = leaf_params(attr);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "+");
    } else {
        panic!("Expected string literal");
    }
}

// ── 2. Leaf annotation with regex pattern on named field ────────────────────

#[test]
fn field_leaf_regex_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Ident {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = find_leaf_attr(&field.attrs);
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), r"[a-zA-Z_]\w*");
    } else {
        panic!("Expected string literal");
    }
}

// ── 3. Leaf with regex pattern containing special chars ─────────────────────

#[test]
fn field_leaf_regex_complex_escapes() {
    let s: ItemStruct = parse_quote! {
        pub struct StringLit {
            #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#)]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let params = leaf_params(find_leaf_attr(&field.attrs));
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert!(s.value().contains(r#"([^"\\]|\\.)*"#));
    } else {
        panic!("Expected string literal");
    }
}

// ── 4. Skip field handling with bool default ────────────────────────────────

#[test]
fn field_skip_bool_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let skip_field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "visited"))
        .unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    let attr = skip_field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(expr.to_token_stream().to_string(), "false");
}

// ── 5. Skip field with integer default ──────────────────────────────────────

#[test]
fn field_skip_integer_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Counter {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(42)]
            count: i32,
        }
    };
    let skip_field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "count"))
        .unwrap();
    let attr = skip_field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(expr.to_token_stream().to_string(), "42");
}

// ── 6. Skip field with string default ───────────────────────────────────────

#[test]
fn field_skip_string_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Tagged {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(String::new())]
            tag: String,
        }
    };
    let skip_field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "tag"))
        .unwrap();
    let attr = skip_field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(expr.to_token_stream().to_string(), "String :: new ()");
}

// ── 7. Multiple annotations: delimited + repeat on same field ───────────────

#[test]
fn field_delimited_with_repeat_on_same_field() {
    let s: ItemStruct = parse_quote! {
        pub struct CsvList {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let names = adze_attr_names(&field.attrs);
    assert!(names.contains(&"delimited".to_string()));
    assert!(names.contains(&"repeat".to_string()));
    assert_eq!(names.len(), 2);
}

// ── 8. Leaf and skip cannot coexist — verify field annotations are distinct ─

#[test]
fn field_leaf_and_skip_on_different_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct MixedNode {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(0)]
            index: usize,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert_eq!(adze_attr_names(&fields[0].attrs), vec!["leaf"]);
    assert_eq!(adze_attr_names(&fields[1].attrs), vec!["skip"]);
}

// ── 9. Vec field type inference for repeat ──────────────────────────────────

#[test]
fn field_vec_type_implies_repeat() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Vec<Number>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

// ── 10. Option field type inference for optional ────────────────────────────

#[test]
fn field_option_type_implies_optional() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Option<Expr>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Expr");
}

// ── 11. Non-Vec non-Option field is required ────────────────────────────────

#[test]
fn field_plain_type_is_required() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Expr);
    let (_inner_vec, extracted_vec) = try_extract_inner_type(&ty, "Vec", &skip);
    let (_inner_opt, extracted_opt) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted_vec);
    assert!(!extracted_opt);
}

// ── 12. Field ordering preserved in struct ──────────────────────────────────

#[test]
fn field_ordering_preserved() {
    let s: ItemStruct = parse_quote! {
        pub struct Assignment {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
            #[adze::leaf(text = ";")]
            _semi: (),
        }
    };
    let names: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(names, vec!["name", "_eq", "value", "_semi"]);
}

// ── 13. Field ordering preserved in enum tuple variant ──────────────────────

#[test]
fn field_ordering_in_enum_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    let variant = &e.variants[0];
    if let Fields::Unnamed(ref u) = variant.fields {
        assert_eq!(u.unnamed.len(), 3);
        // Only the middle field has a leaf attr
        assert!(u.unnamed[0].attrs.is_empty());
        assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        assert!(u.unnamed[2].attrs.is_empty());
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 14. Field with transform closure ────────────────────────────────────────

#[test]
fn field_transform_closure_basic() {
    let s: ItemStruct = parse_quote! {
        pub struct Number {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let attr = find_leaf_attr(&s.fields.iter().next().unwrap().attrs);
    let params = leaf_params(attr);
    let transform = params
        .iter()
        .find(|p| p.path.to_string() == "transform")
        .unwrap();
    assert!(matches!(transform.expr, syn::Expr::Closure(_)));
}

// ── 15. Field with typed transform closure ──────────────────────────────────

#[test]
fn field_transform_closure_with_type_annotation() {
    let s: ItemStruct = parse_quote! {
        pub struct Number {
            #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
            value: i32,
        }
    };
    let attr = find_leaf_attr(&s.fields.iter().next().unwrap().attrs);
    let params = leaf_params(attr);
    let transform = params
        .iter()
        .find(|p| p.path.to_string() == "transform")
        .unwrap();
    if let syn::Expr::Closure(c) = &transform.expr {
        assert_eq!(c.inputs.len(), 1);
        assert!(c.body.to_token_stream().to_string().contains("i32"));
    } else {
        panic!("Expected closure");
    }
}

// ── 16. Field with block-body transform closure ─────────────────────────────

#[test]
fn field_transform_closure_block_body() {
    let s: ItemStruct = parse_quote! {
        pub struct Number {
            #[adze::leaf(pattern = r"\d+", transform = |v| {
                let n: u32 = v.parse().unwrap();
                n * 2
            })]
            value: u32,
        }
    };
    let attr = find_leaf_attr(&s.fields.iter().next().unwrap().attrs);
    let params = leaf_params(attr);
    let transform = params
        .iter()
        .find(|p| p.path.to_string() == "transform")
        .unwrap();
    assert!(matches!(transform.expr, syn::Expr::Closure(_)));
}

// ── 17. Field visibility: pub field in pub struct ───────────────────────────

#[test]
fn field_pub_in_pub_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct Open {
            #[adze::leaf(pattern = r"\w+")]
            pub name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Public(_)));
}

// ── 18. Field visibility: inherited (private) field ─────────────────────────

#[test]
fn field_inherited_visibility() {
    let s: ItemStruct = parse_quote! {
        pub struct Closed {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Inherited));
}

// ── 19. Field visibility: pub(crate) field ──────────────────────────────────

#[test]
fn field_crate_visibility() {
    let s: ItemStruct = parse_quote! {
        pub struct Semi {
            #[adze::leaf(pattern = r"\w+")]
            pub(crate) name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Restricted(_)));
}

// ── 20. Required field (plain type) vs optional (Option) ────────────────────

#[test]
fn field_required_vs_optional_type() {
    let s: ItemStruct = parse_quote! {
        pub struct MaybeTyped {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = ":")]
            _colon: (),
            typ: Option<TypeRef>,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    // name is String → required
    assert_eq!(fields[0].ty.to_token_stream().to_string(), "String");
    // _colon is () → required
    assert_eq!(fields[1].ty.to_token_stream().to_string(), "()");
    // typ is Option<TypeRef> → optional
    assert_eq!(
        fields[2].ty.to_token_stream().to_string(),
        "Option < TypeRef >"
    );

    let skip = expansion_skip_set();
    let (_, is_opt) = try_extract_inner_type(&fields[2].ty, "Option", &skip);
    assert!(is_opt);
    let (_, is_opt_name) = try_extract_inner_type(&fields[0].ty, "Option", &skip);
    assert!(!is_opt_name);
}

// ── 21. Box<T> field type is filtered down to T ─────────────────────────────

#[test]
fn field_box_type_filter() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Box<Expr>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expr");
}

// ── 22. Nested Box<Option<T>> filter strips both wrappers ───────────────────

#[test]
fn field_nested_box_option_filter() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Box<Option<Expr>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expr");
}

// ── 23. wrap_leaf_type on leaf-annotated Option<i32> ────────────────────────

#[test]
fn field_wrap_leaf_option_i32() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < i32 > >"
    );
}

// ── 24. wrap_leaf_type on leaf-annotated Vec<String> ────────────────────────

#[test]
fn field_wrap_leaf_vec_string() {
    let skip = expansion_skip_set();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < String > >"
    );
}

// ── 25. Delimited field inner leaf parsed via FieldThenParams ───────────────

#[test]
fn field_delimited_inner_leaf_parsed() {
    let s: ItemStruct = parse_quote! {
        pub struct CommaSep {
            #[adze::delimited(
                #[adze::leaf(text = ";")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    let inner_leaf = find_leaf_attr(&ftp.field.attrs);
    let params = leaf_params(inner_leaf);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), ";");
    } else {
        panic!("Expected string literal");
    }
}

// ── 26. Leaf text on enum unit variant field ────────────────────────────────

#[test]
fn field_leaf_text_enum_unit_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Keyword {
            #[adze::leaf(text = "let")]
            Let,
            #[adze::leaf(text = "fn")]
            Fn,
        }
    };
    let texts: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let params = leaf_params(find_leaf_attr(&v.attrs));
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &params[0].expr
            {
                s.value()
            } else {
                panic!("Expected string literal");
            }
        })
        .collect();
    assert_eq!(texts, vec!["let", "fn"]);
}

// ── 27. Leaf on unnamed (tuple) field in enum variant ───────────────────────

#[test]
fn field_leaf_on_unnamed_enum_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                u32
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let field = &u.unnamed[0];
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        assert_eq!(field.ty.to_token_stream().to_string(), "u32");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 28. Multiple leaf fields preserve respective params ─────────────────────

#[test]
fn field_multiple_leaves_different_params() {
    let s: ItemStruct = parse_quote! {
        pub struct BinOp {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            lhs: i32,
            #[adze::leaf(text = "+")]
            _op: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            rhs: i32,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    let lhs_params = leaf_params(find_leaf_attr(&fields[0].attrs));
    assert_eq!(lhs_params[0].path.to_string(), "pattern");
    assert_eq!(lhs_params[1].path.to_string(), "transform");

    let op_params = leaf_params(find_leaf_attr(&fields[1].attrs));
    assert_eq!(op_params.len(), 1);
    assert_eq!(op_params[0].path.to_string(), "text");

    let rhs_params = leaf_params(find_leaf_attr(&fields[2].attrs));
    assert_eq!(rhs_params[0].path.to_string(), "pattern");
}

// ── 29. Struct field count matches definition precisely ─────────────────────

#[test]
fn field_count_matches_definition() {
    let s: ItemStruct = parse_quote! {
        pub struct FullStatement {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
            #[adze::skip(false)]
            checked: bool,
            tail: Option<Trailer>,
        }
    };
    assert_eq!(s.fields.iter().count(), 5);
}

// ── 30. Unit-typed fields for punctuation ───────────────────────────────────

#[test]
fn field_unit_type_for_punctuation() {
    let s: ItemStruct = parse_quote! {
        pub struct Bracketed {
            #[adze::leaf(text = "(")]
            _open: (),
            #[adze::leaf(pattern = r"\w+")]
            content: String,
            #[adze::leaf(text = ")")]
            _close: (),
        }
    };
    let unit_names: Vec<_> = s
        .fields
        .iter()
        .filter(|f| f.ty.to_token_stream().to_string() == "()")
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(unit_names, vec!["_open", "_close"]);
}

// ── 31. Underscore-prefixed field names for discarded tokens ────────────────

#[test]
fn field_underscore_prefix_convention() {
    let s: ItemStruct = parse_quote! {
        pub struct LetBinding {
            #[adze::leaf(text = "let")]
            _kw: (),
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    };
    let underscore_fields: Vec<_> = s
        .fields
        .iter()
        .filter(|f| f.ident.as_ref().unwrap().to_string().starts_with('_'))
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(underscore_fields, vec!["_kw", "_eq"]);
}

// ── 32. Enum named-variant field with leaf ──────────────────────────────────

#[test]
fn field_enum_named_variant_leaf() {
    let e: ItemEnum = parse_quote! {
        pub enum Stmt {
            Assign {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                target: String,
                #[adze::leaf(text = "=")]
                _eq: (),
                rhs: Box<Expr>,
            },
        }
    };
    if let Fields::Named(ref named) = e.variants[0].fields {
        assert_eq!(named.named.len(), 3);
        // First two fields have leaf, third does not
        assert!(named.named[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        assert!(named.named[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        assert!(!named.named[2].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    } else {
        panic!("Expected named fields");
    }
}

// ── 33. Cross-struct field type references in a module ──────────────────────

#[test]
fn field_cross_struct_type_reference() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                header: Header,
                body: Vec<Statement>,
            }

            pub struct Header {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            pub struct Statement {
                #[adze::leaf(pattern = r"[^\n]+")]
                line: String,
            }
        }
    });
    let program = find_struct_in_mod(&m, "Program").unwrap();
    let field_types: Vec<_> = program
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    assert_eq!(field_types[0], "Header");
    assert_eq!(field_types[1], "Vec < Statement >");
    assert!(find_struct_in_mod(&m, "Header").is_some());
    assert!(find_struct_in_mod(&m, "Statement").is_some());
}

// ── 34. Repeat annotation non_empty param extraction ────────────────────────

#[test]
fn field_repeat_non_empty_param() {
    let s: ItemStruct = parse_quote! {
        pub struct NonEmptyList {
            #[adze::repeat(non_empty = true)]
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params: Punctuated<NameValueExpr, Token![,]> =
        attr.parse_args_with(Punctuated::parse_terminated).unwrap();
    assert_eq!(params[0].path.to_string(), "non_empty");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Bool(b),
        ..
    }) = &params[0].expr
    {
        assert!(b.value);
    } else {
        panic!("Expected bool literal");
    }
}

// ── 35. Annotation classification per field in mixed struct ─────────────────

#[test]
fn field_annotation_classification() {
    let s: ItemStruct = parse_quote! {
        pub struct FullNode {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
            #[adze::leaf(text = ";")]
            _semi: (),
            #[adze::skip(false)]
            processed: bool,
            #[adze::repeat(non_empty = true)]
            children: Vec<Child>,
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            params: Vec<Param>,
            optional: Option<Trailer>,
        }
    };
    let field_annotations: Vec<Vec<String>> =
        s.fields.iter().map(|f| adze_attr_names(&f.attrs)).collect();
    assert_eq!(field_annotations[0], vec!["leaf"]);
    assert_eq!(field_annotations[1], vec!["leaf"]);
    assert_eq!(field_annotations[2], vec!["skip"]);
    assert_eq!(field_annotations[3], vec!["repeat"]);
    assert_eq!(field_annotations[4], vec!["delimited"]);
    assert!(field_annotations[5].is_empty()); // unannotated Option field
}
