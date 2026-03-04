//! Comprehensive tests for derive/proc-macro patterns used in adze.
//!
//! Covers:
//!   - Proc-macro attribute handling and recognition
//!   - Token stream parsing and code generation
//!   - Attribute validation logic (NameValueExpr, FieldThenParams)
//!   - Error reporting patterns (malformed input, missing attrs)
//!   - Macro infrastructure utilities (type extraction, leaf wrapping)
//!   - Edge cases: empty attrs, multiple attributes, nested attrs

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Expr, ExprLit, Fields, Item, ItemEnum, ItemMod, ItemStruct, Lit, Token, Type,
    parse_quote, parse2, punctuated::Punctuated,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

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

fn field_count(fields: &Fields) -> usize {
    match fields {
        Fields::Named(f) => f.named.len(),
        Fields::Unnamed(f) => f.unnamed.len(),
        Fields::Unit => 0,
    }
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    parse2(tokens).expect("failed to parse module")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 1: NameValueExpr parsing — the key-value parameter type used in attrs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn name_value_expr_string_literal() {
    let nv: NameValueExpr = parse_quote!(text = "hello");
    assert_eq!(nv.path.to_string(), "text");
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "hello");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn name_value_expr_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(i), ..
    }) = &nv.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 42);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn name_value_expr_bool_literal() {
    let nv: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nv.path.to_string(), "non_empty");
    if let Expr::Lit(ExprLit {
        lit: Lit::Bool(b), ..
    }) = &nv.expr
    {
        assert!(b.value);
    } else {
        panic!("Expected bool literal");
    }
}

#[test]
fn name_value_expr_closure_value() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse().unwrap());
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(nv.expr, Expr::Closure(_)));
}

#[test]
fn name_value_expr_raw_string_pattern() {
    let nv: NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nv.path.to_string(), "pattern");
}

#[test]
fn name_value_expr_punctuated_list() {
    let params: Punctuated<NameValueExpr, Token![,]> =
        parse_quote!(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap());
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].path.to_string(), "pattern");
    assert_eq!(params[1].path.to_string(), "transform");
}

#[test]
fn name_value_expr_single_item_punctuated() {
    let params: Punctuated<NameValueExpr, Token![,]> = parse_quote!(text = "+");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 2: FieldThenParams parsing — used for delimited attribute syntax
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn field_then_params_type_only() {
    let ftp: FieldThenParams = parse_quote!(MyType);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "MyType");
}

#[test]
fn field_then_params_with_named_params() {
    let ftp: FieldThenParams = parse_quote!(MyType, name = "test", count = 5);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "count");
}

#[test]
fn field_then_params_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    assert!(ftp.params.is_empty());
}

#[test]
fn field_then_params_with_attrs_on_field() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ",")]
        ()
    );
    assert_eq!(ftp.field.attrs.len(), 1);
    assert!(is_adze_attr(&ftp.field.attrs[0], "leaf"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 3: Type extraction utilities — try_extract_inner_type
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn extract_inner_type_vec_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_inner_type_option_i32() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

#[test]
fn extract_inner_type_not_matching() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < String >");
}

#[test]
fn extract_inner_type_through_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

#[test]
fn extract_inner_type_box_no_target_inside() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Box < String >");
}

#[test]
fn extract_inner_type_non_path_type_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "& str");
}

#[test]
fn extract_inner_type_nested_skip_types() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Option<u64>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "u64");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 4: filter_inner_type — unwrap container wrappers
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn filter_inner_type_unwrap_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Expr>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expr");
}

#[test]
fn filter_inner_type_unwrap_nested_box_arc() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_inner_type_non_skip_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Vec < i32 >");
}

#[test]
fn filter_inner_type_empty_skip_set() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Box < String >");
}

#[test]
fn filter_inner_type_tuple_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "(i32 , u32)");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 5: wrap_leaf_type — wrap types in adze::WithLeaf
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn wrap_leaf_type_plain_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_leaf_type_vec_wraps_inner() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_leaf_type_option_wraps_inner() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < u32 > >"
    );
}

#[test]
fn wrap_leaf_type_nested_skip() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<f64>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < Option < adze :: WithLeaf < f64 > > >"
    );
}

#[test]
fn wrap_leaf_type_reference_wrapped() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_leaf_type_multiple_generic_args() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 6: Attribute recognition patterns — all adze:: attributes
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn recognize_grammar_attr() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {}
    });
    assert!(m.attrs.iter().any(|a| is_adze_attr(a, "grammar")));
}

#[test]
fn recognize_language_attr() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {}
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn recognize_extra_attr() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Ws { _ws: () }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn recognize_leaf_attr_on_field() {
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn recognize_skip_attr() {
    let s: ItemStruct = parse_quote! {
        struct N {
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

#[test]
fn recognize_word_attr() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        struct Ident { name: String }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

#[test]
fn recognize_external_attr() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

#[test]
fn recognize_repeat_attr() {
    let s: ItemStruct = parse_quote! {
        struct L {
            #[adze::repeat(non_empty = true)]
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

#[test]
fn recognize_delimited_attr() {
    let s: ItemStruct = parse_quote! {
        struct L {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

#[test]
fn recognize_prec_left_on_variant() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_left(1)]
            Add(Box<E>, Box<E>),
        }
    };
    assert!(
        e.variants[0]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_left"))
    );
}

#[test]
fn recognize_prec_right_on_variant() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_right(2)]
            Cons(Box<E>, Box<E>),
        }
    };
    assert!(
        e.variants[0]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_right"))
    );
}

#[test]
fn recognize_prec_on_variant() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec(3)]
            Cmp(Box<E>, Box<E>),
        }
    };
    assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "prec")));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 7: Attribute parameter extraction — parsing attr arguments
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn parse_leaf_text_param_from_field() {
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf(text = "+")]
            op: (),
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
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");
}

#[test]
fn parse_leaf_pattern_and_transform_from_field() {
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
            value: i32,
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
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_string()));
    assert!(names.contains(&"transform".to_string()));
}

#[test]
fn parse_precedence_integer_from_variant() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_left(99)]
            V(Box<E>, Box<E>),
        }
    };
    let attr = &e.variants[0].attrs[0];
    let expr: Expr = attr.parse_args().unwrap();
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(i), ..
    }) = &expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 99);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn parse_skip_bool_value() {
    let s: ItemStruct = parse_quote! {
        struct N {
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    if let Expr::Lit(ExprLit {
        lit: Lit::Bool(b), ..
    }) = &expr
    {
        assert!(!b.value);
    } else {
        panic!("Expected bool literal");
    }
}

#[test]
fn parse_grammar_name_string() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_lang")]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = expr
    {
        assert_eq!(s.value(), "my_lang");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn parse_repeat_non_empty_param() {
    let s: ItemStruct = parse_quote! {
        struct L {
            #[adze::repeat(non_empty = true)]
            items: Vec<Number>,
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "non_empty");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 8: Attribute validation — checking error conditions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn unknown_adze_attr_parsed_but_not_recognized() {
    let s: ItemStruct = parse_quote! {
        #[adze::nonexistent]
        struct S;
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names, vec!["nonexistent"]);
    let known = [
        "grammar",
        "language",
        "leaf",
        "word",
        "prec",
        "prec_left",
        "prec_right",
        "extra",
        "skip",
        "delimited",
        "repeat",
        "external",
    ];
    assert!(!known.contains(&names[0].as_str()));
}

#[test]
fn non_adze_attr_not_in_adze_names() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[serde(rename_all = "camelCase")]
        #[adze::language]
        struct S;
    };
    let adze_names = adze_attr_names(&s.attrs);
    assert_eq!(adze_names, vec!["language"]);
}

#[test]
fn grammar_attr_without_parens_is_meta_path() {
    // #[adze::grammar] without arguments — the meta style is Path (no args)
    let m = parse_mod(quote! {
        #[adze::grammar]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    assert!(matches!(&attr.meta, syn::Meta::Path(_)));
}

#[test]
fn grammar_attr_with_non_string_parses_as_expr() {
    // #[adze::grammar(42)] — the expansion would reject this, but syn parses it
    let m = parse_mod(quote! {
        #[adze::grammar(42)]
        mod grammar {}
    });
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    assert!(matches!(
        expr,
        Expr::Lit(ExprLit {
            lit: Lit::Int(_),
            ..
        })
    ));
}

#[test]
fn leaf_attr_empty_parens_parses_empty_params() {
    // #[adze::leaf()] — empty argument list
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf()]
            x: (),
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
    let params = attr
        .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 0);
}

#[test]
fn malformed_name_value_expr_fails_parse() {
    // Missing the `= expr` part: just an ident alone is not a valid NameValueExpr
    let result: syn::Result<NameValueExpr> = syn::parse_str("justident");
    assert!(result.is_err());
}

#[test]
fn malformed_name_value_expr_missing_value_fails() {
    let result: syn::Result<NameValueExpr> = syn::parse_str("key =");
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 9: Edge cases — empty attrs, multiple attrs, structural preservation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn struct_with_no_attrs() {
    let s: ItemStruct = parse_quote! { struct Plain { x: i32 } };
    assert!(s.attrs.is_empty());
    assert_eq!(adze_attr_names(&s.attrs).len(), 0);
}

#[test]
fn multiple_adze_attrs_on_same_item() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        #[adze::language]
        struct Ident { name: String }
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"word".to_string()));
    assert!(names.contains(&"language".to_string()));
}

#[test]
fn multiple_adze_attrs_on_same_field() {
    let s: ItemStruct = parse_quote! {
        struct L {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ";")]
                ()
            )]
            items: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let names = adze_attr_names(&field.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"repeat".to_string()));
    assert!(names.contains(&"delimited".to_string()));
}

#[test]
fn mixed_adze_and_derive_attrs() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        #[adze::language]
        #[derive(PartialEq)]
        struct S { x: i32 }
    };
    let adze_count = s
        .attrs
        .iter()
        .filter(|a| {
            a.path()
                .segments
                .iter()
                .next()
                .map(|seg| seg.ident == "adze")
                .unwrap_or(false)
        })
        .count();
    let derive_count = s
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .count();
    assert_eq!(adze_count, 1);
    assert_eq!(derive_count, 2);
}

#[test]
fn enum_all_unit_leaf_variants() {
    let e: ItemEnum = parse_quote! {
        enum Kw {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
            #[adze::leaf(text = "while")]
            While,
        }
    };
    for v in &e.variants {
        assert_eq!(field_count(&v.fields), 0);
        assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

#[test]
fn unit_struct_with_leaf_text() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "9")]
        struct BigDigit;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn attrs_preserve_field_names() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct P {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            count: usize,
        }
    };
    let names: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(names, vec!["name", "count"]);
}

#[test]
fn attrs_preserve_variant_names() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_left(1)]
            Add(Box<E>, Box<E>),
            #[adze::prec_right(2)]
            Pow(Box<E>, Box<E>),
            Lit(i32),
        }
    };
    let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
    assert_eq!(names, vec!["Add", "Pow", "Lit"]);
}

#[test]
fn grammar_module_preserves_item_count() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {}

            #[adze::extra]
            struct Ws {}
        }
    });
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 10: Token stream generation and code gen patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn quote_generates_impl_with_extract() {
    let ty_name = format_ident!("MyNode");
    let grammar_name = "test_grammar";
    let tokens = quote! {
        impl ::adze::Extract<#ty_name> for #ty_name {
            type LeafFn = ();
            const GRAMMAR_NAME: &'static str = #grammar_name;
            fn extract(node: Option<()>, source: &[u8], _last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
                todo!()
            }
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("MyNode"));
    assert!(s.contains("test_grammar"));
    assert!(s.contains("Extract"));
}

#[test]
fn quote_variant_detection_pattern() {
    let _enum_name = format_ident!("Expr");
    let variant_name = "Number";
    let expected_symbol = format!("Expr_{variant_name}");
    let tokens = quote! {
        if node.kind() == #expected_symbol {
            return Self::Number(value);
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("Expr_Number"));
}

#[test]
fn quote_iteration_over_variants() {
    let variants: Vec<(Ident, String)> = vec![
        (format_ident!("Num"), "Expr_Num".to_string()),
        (format_ident!("Add"), "Expr_Add".to_string()),
    ];
    let detection_arms: Vec<_> = variants
        .iter()
        .map(|(name, symbol)| {
            quote! {
                if node.kind() == #symbol {
                    return #name;
                }
            }
        })
        .collect();
    let tokens = quote! { #(#detection_arms)* };
    let s = tokens.to_string();
    assert!(s.contains("Expr_Num"));
    assert!(s.contains("Expr_Add"));
}

#[test]
fn format_variant_symbol_name() {
    let enum_name = "Expression";
    let variant_name = "Number";
    let expected = format!("{enum_name}_{variant_name}");
    assert_eq!(expected, "Expression_Number");
}

#[test]
fn quote_extern_fn_declaration() {
    let grammar = "json";
    let fn_name = format_ident!("tree_sitter_{grammar}");
    let tokens = quote! {
        unsafe extern "C" {
            fn #fn_name() -> ::adze::tree_sitter::Language;
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("tree_sitter_json"));
}

#[test]
fn roundtrip_struct_with_attrs() {
    let original: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Node {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(original.ident, reparsed.ident);
    assert_eq!(field_count(&original.fields), field_count(&reparsed.fields));
}

#[test]
fn roundtrip_enum_with_attrs() {
    let original: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_left(1)]
            A(i32),
            B(String),
        }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(original.variants.len(), reparsed.variants.len());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 11: Complex grammar patterns — realistic attribute combinations
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn complex_enum_all_prec_variants() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec(1)]
            A(i32),
            #[adze::prec_left(2)]
            B(i32),
            #[adze::prec_right(3)]
            C(i32),
        }
    };
    let attrs: Vec<String> = e
        .variants
        .iter()
        .flat_map(|v| adze_attr_names(&v.attrs))
        .collect();
    assert_eq!(attrs, vec!["prec", "prec_left", "prec_right"]);
    // Verify all precedence values parse
    for (v, expected) in e.variants.iter().zip([1i32, 2, 3]) {
        let expr: Expr = v.attrs[0].parse_args().unwrap();
        if let Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) = expr
        {
            assert_eq!(i.base10_parse::<i32>().unwrap(), expected);
        } else {
            panic!("Expected int literal for {}", v.ident);
        }
    }
}

#[test]
fn complex_grammar_module_structure() {
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
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 2);
    // First item is the enum
    if let Item::Enum(ref e) = items[0] {
        assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        assert_eq!(e.variants.len(), 2);
    } else {
        panic!("Expected enum as first item");
    }
    // Second item is the extra struct
    if let Item::Struct(ref s) = items[1] {
        assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    } else {
        panic!("Expected struct as second item");
    }
}

#[test]
fn enum_named_field_variant() {
    let e: ItemEnum = parse_quote! {
        enum E {
            Named {
                #[adze::leaf(text = "!")]
                _bang: (),
                value: Box<E>,
            }
        }
    };
    assert_eq!(field_count(&e.variants[0].fields), 2);
    if let Fields::Named(ref f) = e.variants[0].fields {
        assert!(f.named[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    } else {
        panic!("Expected named fields");
    }
}

#[test]
fn struct_with_vec_option_box_fields() {
    let s: ItemStruct = parse_quote! {
        struct Complex {
            items: Vec<Item>,
            maybe: Option<Item>,
            child: Box<Item>,
        }
    };
    let types: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    assert!(types[0].contains("Vec"));
    assert!(types[1].contains("Option"));
    assert!(types[2].contains("Box"));
}

#[test]
fn all_twelve_adze_attrs_recognized() {
    // Verify all documented attribute names are parseable
    let known = [
        "grammar",
        "language",
        "leaf",
        "word",
        "prec",
        "prec_left",
        "prec_right",
        "extra",
        "skip",
        "delimited",
        "repeat",
        "external",
    ];
    for attr_name in &known {
        let ident = Ident::new(attr_name, Span::call_site());
        let tokens = quote! {
            #[adze::#ident]
            struct S;
        };
        let s: ItemStruct = parse2(tokens).unwrap();
        assert!(
            is_adze_attr(&s.attrs[0], attr_name),
            "Failed to recognize adze::{attr_name}"
        );
    }
}

#[test]
fn meta_style_path_for_no_arg_attrs() {
    // Attributes without arguments have Meta::Path style
    for attr_name in ["language", "extra", "external", "word"] {
        let ident = Ident::new(attr_name, Span::call_site());
        let s: ItemStruct = parse2(quote! {
            #[adze::#ident]
            struct S;
        })
        .unwrap();
        assert!(
            matches!(&s.attrs[0].meta, syn::Meta::Path(_)),
            "Expected Path meta for adze::{attr_name}"
        );
    }
}

#[test]
fn meta_style_list_for_arg_attrs() {
    // Attributes with arguments have Meta::List style
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "+")]
        struct S;
    };
    assert!(matches!(&s.attrs[0].meta, syn::Meta::List(_)));

    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_left(1)]
            V(i32),
        }
    };
    assert!(matches!(&e.variants[0].attrs[0].meta, syn::Meta::List(_)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Section 12: Code generation building blocks
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn generate_tree_sitter_fn_name() {
    let grammar_name = "python";
    let fn_ident = format_ident!("tree_sitter_{grammar_name}");
    assert_eq!(fn_ident.to_string(), "tree_sitter_python");
}

#[test]
fn generate_variant_symbol_names() {
    let enum_name = "Expression";
    let variants = ["Number", "Add", "Subtract"];
    let symbols: Vec<String> = variants
        .iter()
        .map(|v| format!("{enum_name}_{v}"))
        .collect();
    assert_eq!(
        symbols,
        vec!["Expression_Number", "Expression_Add", "Expression_Subtract"]
    );
}

#[test]
fn generate_parse_fn_with_root_type() {
    let root_type = format_ident!("Expr");
    let grammar_name = "test";
    let _doc = format!("[`{root_type}`]");
    let tokens = quote! {
        pub fn parse(input: &str) -> core::result::Result<#root_type, Vec<::adze::errors::ParseError>> {
            ::adze::__private::parse::<#root_type>(input, || language())
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("Expr"));
    assert!(s.contains("parse"));
    // Verify grammar_name was defined (used elsewhere)
    assert_eq!(grammar_name, "test");
}

#[test]
fn is_sitter_attr_pattern() {
    // Reproduce the is_sitter_attr check from expansion.rs
    let check = |attr: &Attribute| -> bool {
        attr.path()
            .segments
            .iter()
            .next()
            .map(|seg| seg.ident == "adze")
            .unwrap_or(false)
    };
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[derive(Debug)]
        #[serde(rename = "x")]
        struct S;
    };
    let adze_count = s.attrs.iter().filter(|a| check(a)).count();
    assert_eq!(adze_count, 1);
}

#[test]
fn retain_non_sitter_attrs_pattern() {
    // Test the pattern used in expansion.rs to strip adze attrs
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[adze::language]
        #[cfg(test)]
        struct S;
    };
    let mut attrs = s.attrs.clone();
    attrs.retain(|a| {
        !a.path()
            .segments
            .iter()
            .next()
            .map(|seg| seg.ident == "adze")
            .unwrap_or(false)
    });
    assert_eq!(attrs.len(), 2);
    assert!(attrs[0].path().is_ident("derive"));
    assert!(attrs[1].path().is_ident("cfg"));
}
