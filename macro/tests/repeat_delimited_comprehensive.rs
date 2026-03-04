#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for repeat and delimited annotation handling
//! in the adze proc-macro crate.
//!
//! Covers `#[adze::repeat]`, `#[adze::delimited]` attribute parsing,
//! `non_empty` parameter extraction, separator patterns, combined usage,
//! `Vec<T>` repeat inference, `Option<T>` interaction, and edge cases.

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

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn repeat_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

// ── 1. repeat: basic attribute recognition ──────────────────────────────────

#[test]
fn repeat_attr_recognized() {
    let s: ItemStruct = parse_quote! {
        pub struct NumberList {
            #[adze::repeat(non_empty = true)]
            numbers: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

// ── 2. repeat: non_empty = true extraction ──────────────────────────────────

#[test]
fn repeat_non_empty_true() {
    let s: ItemStruct = parse_quote! {
        pub struct NumberList {
            #[adze::repeat(non_empty = true)]
            numbers: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params = repeat_params(attr);
    assert_eq!(params.len(), 1);
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

// ── 3. repeat: non_empty = false extraction ─────────────────────────────────

#[test]
fn repeat_non_empty_false() {
    let s: ItemStruct = parse_quote! {
        pub struct Items {
            #[adze::repeat(non_empty = false)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let params = repeat_params(attr);
    assert_eq!(params[0].path.to_string(), "non_empty");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Bool(b),
        ..
    }) = &params[0].expr
    {
        assert!(!b.value);
    } else {
        panic!("Expected bool literal");
    }
}

// ── 4. delimited: basic attribute recognition ───────────────────────────────

#[test]
fn delimited_attr_recognized() {
    let s: ItemStruct = parse_quote! {
        pub struct CsvRow {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            values: Vec<Value>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

// ── 5. delimited: separator text extraction ─────────────────────────────────

#[test]
fn delimited_separator_text_extracted() {
    let s: ItemStruct = parse_quote! {
        pub struct Args {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            args: Vec<Arg>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(inner_leaf);
    assert_eq!(params[0].path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), ",");
    } else {
        panic!("Expected string literal");
    }
}

// ── 6. delimited: semicolon separator ───────────────────────────────────────

#[test]
fn delimited_semicolon_separator() {
    let s: ItemStruct = parse_quote! {
        pub struct Stmts {
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
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
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

// ── 7. delimited: pipe separator ────────────────────────────────────────────

#[test]
fn delimited_pipe_separator() {
    let s: ItemStruct = parse_quote! {
        pub struct Alternatives {
            #[adze::delimited(
                #[adze::leaf(text = "|")]
                ()
            )]
            alts: Vec<Alt>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(inner_leaf);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "|");
    } else {
        panic!("Expected string literal");
    }
}

// ── 8. delimited: multi-char separator ──────────────────────────────────────

#[test]
fn delimited_multi_char_separator() {
    let s: ItemStruct = parse_quote! {
        pub struct Paths {
            #[adze::delimited(
                #[adze::leaf(text = "::")]
                ()
            )]
            segments: Vec<Segment>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(inner_leaf);
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

// ── 9. delimited: inner field type is unit ──────────────────────────────────

#[test]
fn delimited_inner_field_is_unit_type() {
    let s: ItemStruct = parse_quote! {
        pub struct Items {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
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
}

// ── 10. combined: repeat + delimited on same field ──────────────────────────

#[test]
fn repeat_and_delimited_combined() {
    let s: ItemStruct = parse_quote! {
        pub struct NumberList {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            numbers: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attrs = adze_attr_names(&field.attrs);
    assert!(attrs.contains(&"repeat".to_string()));
    assert!(attrs.contains(&"delimited".to_string()));
}

// ── 11. combined: delimited + repeat ordering ───────────────────────────────

#[test]
fn delimited_before_repeat_ordering() {
    let s: ItemStruct = parse_quote! {
        pub struct Items {
            #[adze::delimited(
                #[adze::leaf(text = ";")]
                ()
            )]
            #[adze::repeat(non_empty = false)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attrs = adze_attr_names(&field.attrs);
    assert_eq!(attrs[0], "delimited");
    assert_eq!(attrs[1], "repeat");
}

// ── 12. Vec<T>: repeat inference on plain Vec field ─────────────────────────

#[test]
fn vec_type_repeat_inference() {
    let s: ItemStruct = parse_quote! {
        pub struct NumberList {
            numbers: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

// ── 13. Vec<T>: extract inner type through Box wrapper ──────────────────────

#[test]
fn vec_inner_type_through_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<Number>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

// ── 14. Option<T>: not mistaken for Vec ─────────────────────────────────────

#[test]
fn option_not_extracted_as_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Number>);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

// ── 15. Option<Vec<T>>: Vec extracted through Option skip ───────────────────

#[test]
fn option_vec_extracted_with_option_skip() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<Number>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

// ── 16. wrap_leaf_type: Vec<T> with Vec in skip set ─────────────────────────

#[test]
fn wrap_vec_in_skip_set() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Number>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < Number > >"
    );
}

// ── 17. enum: unnamed Vec field with repeat ─────────────────────────────────

#[test]
fn enum_unnamed_vec_with_repeat() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Numbers(
                #[adze::repeat(non_empty = true)]
                Vec<Number>
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let field = &u.unnamed[0];
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
        assert_eq!(field.ty.to_token_stream().to_string(), "Vec < Number >");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 18. enum: unnamed Vec field with delimited ──────────────────────────────

#[test]
fn enum_unnamed_vec_with_delimited() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            CommaSeparated(
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                Vec<Number>
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let field = &u.unnamed[0];
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 19. grammar module: struct with repeat field preserved ──────────────────

#[test]
fn grammar_module_struct_repeat_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct NumberList {
                #[adze::repeat(non_empty = true)]
                numbers: Vec<Number>,
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        }
    });
    let items = module_items(&m);
    // First item should be the struct
    if let Item::Struct(s) = &items[0] {
        assert_eq!(s.ident, "NumberList");
        let field = s.fields.iter().next().unwrap();
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    } else {
        panic!("Expected struct as first item");
    }
}

// ── 20. grammar module: struct with delimited field preserved ───────────────

#[test]
fn grammar_module_struct_delimited_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct CsvRow {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                values: Vec<Value>,
            }

            pub struct Value {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        assert_eq!(s.ident, "CsvRow");
        let field = s.fields.iter().next().unwrap();
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
    } else {
        panic!("Expected struct as first item");
    }
}

// ── 21. delimited with pattern separator ────────────────────────────────────

#[test]
fn delimited_pattern_separator() {
    let s: ItemStruct = parse_quote! {
        pub struct Items {
            #[adze::delimited(
                #[adze::leaf(pattern = r"\s*,\s*")]
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
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(inner_leaf);
    assert_eq!(params[0].path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), r"\s*,\s*");
    } else {
        panic!("Expected string literal");
    }
}

// ── 22. multiple Vec fields with different repeat configs ───────────────────

#[test]
fn multiple_vec_fields_different_repeat() {
    let s: ItemStruct = parse_quote! {
        pub struct Grammar {
            #[adze::repeat(non_empty = true)]
            rules: Vec<Rule>,
            #[adze::repeat(non_empty = false)]
            extras: Vec<Extra>,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert_eq!(fields.len(), 2);

    for (i, expected_non_empty) in [(0, true), (1, false)] {
        let attr = fields[i]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "repeat"))
            .unwrap();
        let params = repeat_params(attr);
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(b),
            ..
        }) = &params[0].expr
        {
            assert_eq!(b.value, expected_non_empty);
        } else {
            panic!("Expected bool literal for field {i}");
        }
    }
}

// ── 23. Vec<T> with no repeat annotation (bare repeat inference) ────────────

#[test]
fn vec_without_repeat_annotation() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    // No repeat attribute, but Vec<T> type should still be extractable
    assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Item");
}

// ── 24. delimited: FieldThenParams with no additional params ────────────────

#[test]
fn delimited_field_then_params_no_extra_params() {
    let s: ItemStruct = parse_quote! {
        pub struct Items {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
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
    // No additional params beyond the field
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

// ── 25. repeat + delimited: both attributes extractable independently ───────

#[test]
fn repeat_delimited_both_extractable() {
    let s: ItemStruct = parse_quote! {
        pub struct Params {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            params: Vec<Param>,
        }
    };
    let field = s.fields.iter().next().unwrap();

    // Extract repeat params
    let repeat_attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "repeat"))
        .unwrap();
    let rp = repeat_params(repeat_attr);
    assert_eq!(rp[0].path.to_string(), "non_empty");

    // Extract delimited params
    let delim_attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim_attr.parse_args().unwrap();
    assert!(ftp.field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

// ── 26. enum: variant with repeat + delimited + prec combined ───────────────

#[test]
fn enum_variant_repeat_delimited_prec() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(1)]
            Call(
                Box<Expr>,
                #[adze::leaf(text = "(")]
                (),
                #[adze::repeat(non_empty = false)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                Vec<Expr>,
                #[adze::leaf(text = ")")]
                (),
            ),
        }
    };
    // Verify prec on variant
    assert!(
        e.variants[1]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_left"))
    );

    // Verify repeat + delimited on Vec field
    if let Fields::Unnamed(ref u) = e.variants[1].fields {
        let vec_field = &u.unnamed[2];
        let attrs = adze_attr_names(&vec_field.attrs);
        assert!(attrs.contains(&"repeat".to_string()));
        assert!(attrs.contains(&"delimited".to_string()));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 27. wrap_leaf_type: Option<Vec<T>> with both in skip set ────────────────

#[test]
fn wrap_option_vec_both_in_skip() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<Number>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Vec < adze :: WithLeaf < Number > > >"
    );
}

// ── 28. delimited: arrow separator (=>) ─────────────────────────────────────

#[test]
fn delimited_arrow_separator() {
    let s: ItemStruct = parse_quote! {
        pub struct Mappings {
            #[adze::delimited(
                #[adze::leaf(text = "=>")]
                ()
            )]
            mappings: Vec<Mapping>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .unwrap();
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(inner_leaf);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "=>");
    } else {
        panic!("Expected string literal");
    }
}

// ── 29. struct: named field with Vec and Spanned wrapper ────────────────────

#[test]
fn vec_with_spanned_wrapper_type() {
    let skip: HashSet<&str> = ["Spanned"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Spanned<Number>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    // Extracts through Vec, giving Spanned<Number>
    assert_eq!(inner.to_token_stream().to_string(), "Spanned < Number >");
}

// ── 30. Vec<T>: type extraction when Vec is not the outermost ───────────────

#[test]
fn non_vec_outermost_not_extracted() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(HashMap<String, Vec<Number>>);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

// ── 31. repeat attr on enum named-field variant ─────────────────────────────

#[test]
fn repeat_on_enum_named_field_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Node {
            Block {
                #[adze::leaf(text = "{")]
                _open: (),
                #[adze::repeat(non_empty = false)]
                stmts: Vec<Stmt>,
                #[adze::leaf(text = "}")]
                _close: (),
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let stmts_field = n
            .named
            .iter()
            .find(|f| f.ident.as_ref().unwrap() == "stmts")
            .unwrap();
        assert!(stmts_field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    } else {
        panic!("Expected named fields");
    }
}

// ── 32. delimited + repeat in grammar module with extra ─────────────────────

#[test]
fn delimited_repeat_in_full_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("csv")]
        mod grammar {
            #[adze::language]
            pub struct CsvFile {
                #[adze::repeat(non_empty = false)]
                #[adze::delimited(
                    #[adze::leaf(text = "\n")]
                    ()
                )]
                rows: Vec<Row>,
            }

            pub struct Row {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                cells: Vec<Cell>,
            }

            pub struct Cell {
                #[adze::leaf(pattern = r"[^,\n]*")]
                value: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r" ")]
                _ws: (),
            }
        }
    });
    let items = module_items(&m);
    // Verify we have CsvFile, Row, Cell, and Whitespace structs
    let struct_names: Vec<_> = items
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i {
                Some(s.ident.to_string())
            } else {
                None
            }
        })
        .collect();
    assert!(struct_names.contains(&"CsvFile".to_string()));
    assert!(struct_names.contains(&"Row".to_string()));
    assert!(struct_names.contains(&"Cell".to_string()));
    assert!(struct_names.contains(&"Whitespace".to_string()));
}

// ── 33. repeat and delimited attribute count on field ────────────────────────

#[test]
fn repeat_delimited_attr_count_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Container {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let adze_count = field
        .attrs
        .iter()
        .filter(|a| {
            let segs: Vec<_> = a.path().segments.iter().collect();
            segs.len() == 2 && segs[0].ident == "adze"
        })
        .count();
    assert_eq!(adze_count, 2);
}

// ── 34. delimited: inner field has no extra params ──────────────────────────

#[test]
fn delimited_inner_no_extra_params() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ",")]
        ()
    );
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    // Verify the leaf attribute on the inner field
    assert_eq!(ftp.field.attrs.len(), 1);
    assert!(is_adze_attr(&ftp.field.attrs[0], "leaf"));
}

// ── 35. Vec inner type extraction with complex generic ──────────────────────

#[test]
fn vec_inner_type_complex_generic() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Box<Number>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    // Vec extracts to Box<Number>, not through Box
    assert_eq!(inner.to_token_stream().to_string(), "Box < Number >");
}
