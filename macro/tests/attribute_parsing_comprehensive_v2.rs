//! Comprehensive attribute parsing tests v2 for the adze-macro crate.
//!
//! Tests cover:
//! 1. NameValueExpr parsing (keys, values, closures, edge cases)
//! 2. FieldThenParams parsing (bare types, params, generics)
//! 3. try_extract_inner_type (extraction, skip-through, non-path types)
//! 4. filter_inner_type (unwrapping, nesting, no-ops)
//! 5. wrap_leaf_type (wrapping, skip-set recursion, non-path types)
//! 6. Attribute detection on structs and enums (adze::* recognition)
//! 7. Proc macro input validation (token stream round-trips, field attrs)
//! 8. Edge cases (empty modules, nested generics, lifetime types, visibility)

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, ItemEnum, ItemMod, ItemStruct, Type, parse_quote, parse2};

// ── Helper Functions ─────────────────────────────────────────────────────────

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    parse2(tokens).expect("failed to parse module")
}

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().collect();
    segments.len() == 2 && segments[0].ident == "adze" && segments[1].ident == name
}

fn adze_attr_names(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            let segs: Vec<_> = attr.path().segments.iter().collect();
            if segs.len() == 2 && segs[0].ident == "adze" {
                Some(segs[1].ident.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ============================================================================
// 1. NameValueExpr PARSING — additional coverage
// ============================================================================

#[test]
fn nve_parse_path_expression_value() {
    let nv: NameValueExpr = parse_quote!(default = std::i32::MAX);
    assert_eq!(nv.path, "default");
    let s = nv.expr.to_token_stream().to_string();
    assert!(s.contains("MAX"));
}

#[test]
fn nve_parse_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path, "offset");
}

#[test]
fn nve_parse_float_value() {
    let nv: NameValueExpr = parse_quote!(weight = 3.14);
    assert_eq!(nv.path, "weight");
}

#[test]
fn nve_parse_char_literal() {
    let nv: NameValueExpr = parse_quote!(sep = ',');
    assert_eq!(nv.path, "sep");
}

#[test]
fn nve_parse_method_call_value() {
    let nv: NameValueExpr = parse_quote!(transform = |v: &str| v.trim().to_string());
    assert_eq!(nv.path, "transform");
    let s = nv.expr.to_token_stream().to_string();
    assert!(s.contains("trim"));
}

#[test]
fn nve_parse_block_expression_value() {
    let nv: NameValueExpr = parse_quote!(
        init = {
            let x = 1;
            x + 2
        }
    );
    assert_eq!(nv.path, "init");
}

#[test]
fn nve_parse_tuple_expression() {
    let nv: NameValueExpr = parse_quote!(pair = (1, 2));
    assert_eq!(nv.path, "pair");
}

#[test]
fn nve_parse_string_with_escapes() {
    let nv: NameValueExpr = parse_quote!(pattern = "hello\nworld");
    assert_eq!(nv.path, "pattern");
}

// ============================================================================
// 2. FieldThenParams PARSING — additional coverage
// ============================================================================

#[test]
fn ftp_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "()");
}

#[test]
fn ftp_box_type_with_param() {
    let ftp: FieldThenParams = parse_quote!(Box<Expr>, name = "child");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path, "name");
}

#[test]
fn ftp_option_type() {
    let ftp: FieldThenParams = parse_quote!(Option<String>);
    assert!(ftp.comma.is_none());
    assert_eq!(ts(&ftp.field.ty), "Option < String >");
}

#[test]
fn ftp_nested_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<Option<i32>>);
    assert_eq!(ts(&ftp.field.ty), "Vec < Option < i32 > >");
}

#[test]
fn ftp_with_three_params() {
    let ftp: FieldThenParams = parse_quote!(u8, a = 1, b = 2, c = 3);
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[0].path, "a");
    assert_eq!(ftp.params[1].path, "b");
    assert_eq!(ftp.params[2].path, "c");
}

#[test]
fn ftp_reference_type() {
    // Reference types in unnamed field position
    let ftp: FieldThenParams = parse_quote!(&str);
    assert_eq!(ts(&ftp.field.ty), "& str");
}

// ============================================================================
// 3. try_extract_inner_type — additional coverage
// ============================================================================

#[test]
fn extract_option_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_vec_of_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "Option < i32 >");
}

#[test]
fn extract_skip_spanned() {
    let skip: HashSet<&str> = ["Spanned"].into_iter().collect();
    let ty: Type = parse_quote!(Spanned<Vec<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

#[test]
fn extract_plain_ident_not_generic() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
}

#[test]
fn extract_deeply_nested_skip() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<bool>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "bool");
}

#[test]
fn extract_tuple_type_returns_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((u8, u16));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(ts(&inner), "(u8 , u16)");
}

#[test]
fn extract_skip_present_but_inner_mismatch() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<HashMap<String, i32>>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
}

// ============================================================================
// 4. filter_inner_type — additional coverage
// ============================================================================

#[test]
fn filter_triple_nested_unwrap() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<i64>>>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "i64");
}

#[test]
fn filter_non_skip_generic_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ts(&filter_inner_type(&ty, &skip)),
        "HashMap < String , i32 >"
    );
}

#[test]
fn filter_reference_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(&'static str);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "& 'static str");
}

#[test]
fn filter_slice_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!([u8]);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "[u8]");
}

#[test]
fn filter_single_skip_layer() {
    let skip: HashSet<&str> = ["Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Rc<Vec<u8>>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "Vec < u8 >");
}

// ============================================================================
// 5. wrap_leaf_type — additional coverage
// ============================================================================

#[test]
fn wrap_box_in_skip_set() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<i32>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "Box < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_double_nested_skip() {
    let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<String>>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_non_skip_generic_wraps_whole() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn wrap_reference_type_wraps_whole() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_tuple_type_wraps_whole() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((i32, String));
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < (i32 , String) >"
    );
}

#[test]
fn wrap_unit_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(());
    assert_eq!(ts(&wrap_leaf_type(&ty, &skip)), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_spanned_in_skip_wraps_inner() {
    let skip: HashSet<&str> = ["Spanned"].into_iter().collect();
    let ty: Type = parse_quote!(Spanned<Number>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "Spanned < adze :: WithLeaf < Number > >"
    );
}

// ============================================================================
// 6. ATTRIBUTE DETECTION ON STRUCTS
// ============================================================================

#[test]
fn detect_language_attr_on_enum() {
    let e = parse_enum(quote! {
        #[adze::language]
        enum Expr { A, B }
    });
    assert!(is_adze_attr(&e.attrs[0], "language"));
}

#[test]
fn detect_extra_attr_on_struct() {
    let s = parse_struct(quote! {
        #[adze::extra]
        struct Whitespace { _ws: () }
    });
    assert!(is_adze_attr(&s.attrs[0], "extra"));
}

#[test]
fn detect_external_attr() {
    let s = parse_struct(quote! {
        #[adze::external]
        struct IndentToken;
    });
    assert!(is_adze_attr(&s.attrs[0], "external"));
}

#[test]
fn detect_word_attr() {
    let s = parse_struct(quote! {
        #[adze::word]
        struct Identifier { name: String }
    });
    assert!(is_adze_attr(&s.attrs[0], "word"));
}

#[test]
fn detect_prec_attr_on_variant() {
    let e = parse_enum(quote! {
        enum E {
            #[adze::prec(2)]
            Compare(Box<E>, (), Box<E>),
        }
    });
    assert!(is_adze_attr(&e.variants[0].attrs[0], "prec"));
}

#[test]
fn detect_prec_left_attr_on_variant() {
    let e = parse_enum(quote! {
        enum E {
            #[adze::prec_left(1)]
            Add(Box<E>, (), Box<E>),
        }
    });
    assert!(is_adze_attr(&e.variants[0].attrs[0], "prec_left"));
}

#[test]
fn detect_prec_right_attr_on_variant() {
    let e = parse_enum(quote! {
        enum E {
            #[adze::prec_right(1)]
            Cons(Box<E>, (), Box<E>),
        }
    });
    assert!(is_adze_attr(&e.variants[0].attrs[0], "prec_right"));
}

#[test]
fn detect_leaf_attr_on_field() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    });
    if let Fields::Named(ref named) = s.fields {
        assert!(is_adze_attr(&named.named[0].attrs[0], "leaf"));
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn detect_skip_attr_on_field() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::skip(false)]
            visited: bool,
        }
    });
    if let Fields::Named(ref named) = s.fields {
        assert!(is_adze_attr(&named.named[0].attrs[0], "skip"));
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn detect_repeat_attr_on_field() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    });
    if let Fields::Named(ref named) = s.fields {
        assert!(is_adze_attr(&named.named[0].attrs[0], "repeat"));
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn detect_delimited_attr_on_field() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    });
    if let Fields::Named(ref named) = s.fields {
        assert!(is_adze_attr(&named.named[0].attrs[0], "delimited"));
    } else {
        panic!("expected named fields");
    }
}

// ============================================================================
// 7. PROC MACRO INPUT VALIDATION — token stream & module parsing
// ============================================================================

#[test]
fn module_with_grammar_attr_roundtrip() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            struct S;
        }
    });
    assert_eq!(m.ident, "grammar");
    assert!(is_adze_attr(&m.attrs[0], "grammar"));
}

#[test]
fn module_content_present() {
    let m = parse_mod(quote! {
        mod my_grammar {
            struct A;
            enum B { X }
        }
    });
    let (_, items) = m.content.expect("module should have content");
    assert_eq!(items.len(), 2);
}

#[test]
fn module_semicolon_form_has_no_content() {
    let m: ItemMod = parse_quote!(
        mod external;
    );
    assert!(m.content.is_none());
}

#[test]
fn multiple_attrs_on_module() {
    let m = parse_mod(quote! {
        #[cfg(test)]
        #[adze::grammar("arithmetic")]
        mod grammar {
            struct S;
        }
    });
    let names = adze_attr_names(&m.attrs);
    assert_eq!(names, vec!["grammar"]);
    // Total attrs includes cfg
    assert_eq!(m.attrs.len(), 2);
}

#[test]
fn grammar_attr_contains_string_value() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_lang")]
        mod grammar {
            struct S;
        }
    });
    let attr = &m.attrs[0];
    let tokens = attr.to_token_stream().to_string();
    assert!(tokens.contains("my_lang"));
}

#[test]
fn pub_module_visibility_preserved() {
    let m = parse_mod(quote! {
        pub mod grammar {
            struct S;
        }
    });
    assert!(matches!(m.vis, syn::Visibility::Public(_)));
}

#[test]
fn enum_with_multiple_variant_attrs() {
    let e = parse_enum(quote! {
        enum Expr {
            #[adze::leaf(text = "1")]
            One,
            #[adze::prec_left(1)]
            Add(Box<Expr>, (), Box<Expr>),
            #[adze::prec_right(2)]
            Pow(Box<Expr>, (), Box<Expr>),
        }
    });
    assert!(is_adze_attr(&e.variants[0].attrs[0], "leaf"));
    assert!(is_adze_attr(&e.variants[1].attrs[0], "prec_left"));
    assert!(is_adze_attr(&e.variants[2].attrs[0], "prec_right"));
}

#[test]
fn struct_with_no_attrs_has_empty_attr_list() {
    let s = parse_struct(quote! {
        struct Plain { x: i32 }
    });
    assert!(adze_attr_names(&s.attrs).is_empty());
}

#[test]
fn enum_variant_with_named_fields_attrs() {
    let e = parse_enum(quote! {
        enum E {
            Named {
                #[adze::leaf(text = "!")]
                _bang: (),
                value: Box<E>,
            }
        }
    });
    if let Fields::Named(ref named) = e.variants[0].fields {
        assert!(is_adze_attr(&named.named[0].attrs[0], "leaf"));
        assert!(named.named[1].attrs.is_empty());
    } else {
        panic!("expected named fields on variant");
    }
}

#[test]
fn enum_variant_unnamed_fields_with_leaf() {
    let e = parse_enum(quote! {
        enum E {
            Num(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                i32
            ),
        }
    });
    if let Fields::Unnamed(ref unnamed) = e.variants[0].fields {
        assert!(is_adze_attr(&unnamed.unnamed[0].attrs[0], "leaf"));
    } else {
        panic!("expected unnamed fields on variant");
    }
}

// ============================================================================
// 8. EDGE CASES
// ============================================================================

#[test]
fn non_adze_attr_not_detected() {
    let s = parse_struct(quote! {
        #[derive(Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        struct S { f: u8 }
    });
    assert!(adze_attr_names(&s.attrs).is_empty());
}

#[test]
fn single_segment_path_not_detected_as_adze() {
    let s = parse_struct(quote! {
        #[test]
        struct S { f: u8 }
    });
    assert!(adze_attr_names(&s.attrs).is_empty());
}

#[test]
fn three_segment_path_not_detected_as_adze() {
    let s = parse_struct(quote! {
        #[adze::foo::bar]
        struct S { f: u8 }
    });
    // Our helper only checks len == 2
    assert!(adze_attr_names(&s.attrs).is_empty());
}

#[test]
fn doc_comments_not_counted_as_adze() {
    let s = parse_struct(quote! {
        /// This is a doc comment
        #[adze::language]
        struct S { f: u8 }
    });
    assert_eq!(adze_attr_names(&s.attrs), vec!["language"]);
}

#[test]
fn empty_enum_variant_list() {
    let e = parse_enum(quote! {
        enum Empty {}
    });
    assert!(e.variants.is_empty());
}

#[test]
fn unit_struct_with_leaf_attr() {
    let s = parse_struct(quote! {
        #[adze::leaf(text = "keyword")]
        struct Kw;
    });
    assert!(is_adze_attr(&s.attrs[0], "leaf"));
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn tuple_struct_with_leaf_field() {
    let s = parse_struct(quote! {
        struct Wrapper(
            #[adze::leaf(pattern = r"\w+")]
            String
        );
    });
    if let Fields::Unnamed(ref unnamed) = s.fields {
        assert!(is_adze_attr(&unnamed.unnamed[0].attrs[0], "leaf"));
    } else {
        panic!("expected unnamed fields");
    }
}

#[test]
fn generic_struct_attrs_preserved() {
    let s = parse_struct(quote! {
        #[adze::language]
        struct Node<T> where T: Clone {
            value: T,
        }
    });
    assert!(is_adze_attr(&s.attrs[0], "language"));
    assert!(s.generics.params.len() == 1);
    assert!(s.generics.where_clause.is_some());
}

#[test]
fn multiple_leaf_fields_on_struct() {
    let s = parse_struct(quote! {
        struct BinOp {
            #[adze::leaf(pattern = r"\d+")]
            left: String,
            #[adze::leaf(text = "+")]
            op: (),
            #[adze::leaf(pattern = r"\d+")]
            right: String,
        }
    });
    if let Fields::Named(ref named) = s.fields {
        assert_eq!(named.named.len(), 3);
        assert!(is_adze_attr(&named.named[0].attrs[0], "leaf"));
        assert!(is_adze_attr(&named.named[1].attrs[0], "leaf"));
        assert!(is_adze_attr(&named.named[2].attrs[0], "leaf"));
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn leaf_with_text_and_pattern_both_parsed() {
    // Both text= and pattern= are valid NameValueExpr inside leaf
    let nv_text: NameValueExpr = parse_quote!(text = "+");
    let nv_pattern: NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nv_text.path, "text");
    assert_eq!(nv_pattern.path, "pattern");
}

#[test]
fn leaf_with_transform_closure_complex() {
    let nv: NameValueExpr = parse_quote!(transform = |v: &str| v.parse::<i32>().unwrap());
    assert_eq!(nv.path, "transform");
    let s = nv.expr.to_token_stream().to_string();
    assert!(s.contains("i32"));
}

#[test]
fn repeat_non_empty_false() {
    let nv: NameValueExpr = parse_quote!(non_empty = false);
    assert_eq!(nv.path, "non_empty");
}

#[test]
fn extract_inner_from_vec_of_box() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Box<Expr>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "Box < Expr >");
}

#[test]
fn filter_box_of_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<i32>>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn wrap_leaf_deeply_nested_skip_set() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<String>>>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip)),
        "Vec < Option < Box < adze :: WithLeaf < String > > > >"
    );
}

#[test]
fn attr_text_value_with_special_chars() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::leaf(text = "::")]
            sep: (),
        }
    });
    let attr_str = s.fields.iter().next().unwrap().attrs[0]
        .to_token_stream()
        .to_string();
    assert!(attr_str.contains("::"));
}

#[test]
fn attr_text_value_multichar_operator() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::leaf(text = "==")]
            eq_op: (),
        }
    });
    let attr_str = s.fields.iter().next().unwrap().attrs[0]
        .to_token_stream()
        .to_string();
    assert!(attr_str.contains("=="));
}

#[test]
fn ftp_with_closure_param() {
    let ftp: FieldThenParams = parse_quote!(i32, transform = |v: &str| v.parse::<i32>().unwrap());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path, "transform");
}

#[test]
fn extract_inner_option_of_vec() {
    // Not skipping Option, looking for Vec
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    // Option is not in skip set, so it won't look inside
    assert!(!ok);
}

#[test]
fn extract_inner_with_option_in_skip() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

#[test]
fn wrap_leaf_preserves_non_type_generic_args() {
    // Result<T, E> in skip set wraps both type arguments
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty: Type = parse_quote!(Result<u32, String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let s = ts(&wrapped);
    assert!(s.contains("WithLeaf < u32 >"));
    assert!(s.contains("WithLeaf < String >"));
}

#[test]
fn enum_all_unit_variants_detection() {
    let e = parse_enum(quote! {
        enum Keywords {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
            #[adze::leaf(text = "while")]
            While,
            #[adze::leaf(text = "for")]
            For,
        }
    });
    assert_eq!(e.variants.len(), 4);
    for v in &e.variants {
        assert!(is_adze_attr(&v.attrs[0], "leaf"));
        assert!(matches!(v.fields, Fields::Unit));
    }
}

#[test]
fn struct_visibility_variants() {
    let s = parse_struct(quote! {
        pub struct PubStruct { pub f: u8 }
    });
    assert!(matches!(s.vis, syn::Visibility::Public(_)));

    let s2 = parse_struct(quote! {
        struct PrivStruct { f: u8 }
    });
    assert!(matches!(s2.vis, syn::Visibility::Inherited));
}

#[test]
fn module_items_include_both_struct_and_enum() {
    let m = parse_mod(quote! {
        mod grammar {
            struct Number { v: i32 }
            enum Expr {
                Num(i32),
                Add(Box<Expr>, Box<Expr>),
            }
        }
    });
    let (_, items) = m.content.unwrap();
    let structs = items
        .iter()
        .filter(|i| matches!(i, syn::Item::Struct(_)))
        .count();
    let enums = items
        .iter()
        .filter(|i| matches!(i, syn::Item::Enum(_)))
        .count();
    assert_eq!(structs, 1);
    assert_eq!(enums, 1);
}

#[test]
fn roundtrip_struct_with_multiple_field_attrs() {
    let s = parse_struct(quote! {
        struct S {
            #[adze::leaf(pattern = r"\d+")]
            #[adze::skip(0)]
            field: i32,
        }
    });
    let reparsed = parse_struct(quote! { #s });
    if let Fields::Named(ref named) = reparsed.fields {
        let attrs = adze_attr_names(&named.named[0].attrs);
        assert_eq!(attrs, vec!["leaf", "skip"]);
    } else {
        panic!("expected named fields");
    }
}

#[test]
fn nve_equality_check() {
    let nv1: NameValueExpr = parse_quote!(key = "value");
    let nv2: NameValueExpr = parse_quote!(key = "value");
    assert_eq!(nv1.path, nv2.path);
}

#[test]
fn filter_inner_preserves_simple_type() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(i32);
    assert_eq!(ts(&filter_inner_type(&ty, &skip)), "i32");
}
