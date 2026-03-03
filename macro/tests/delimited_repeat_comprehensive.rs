#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for delimited repeat patterns in adze-macro.
//!
//! Covers `Vec<T>` field detection, delimiter pattern recognition,
//! different delimiter types, optional trailing delimiters, nested
//! `Vec<Vec<T>>`, empty repeat handling, repeat with field names,
//! and repeat combined with other annotations.

use std::collections::HashSet;

use adze_common::{FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type};
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

fn extract_delim_text(field: &syn::Field) -> String {
    let delim = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "delimited"))
        .expect("no delimited attr");
    let ftp: FieldThenParams = delim.parse_args().unwrap();
    let inner_leaf = ftp
        .field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .expect("no leaf on delimiter");
    let params = leaf_params(inner_leaf);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        s.value()
    } else {
        panic!("Expected string literal in delimiter leaf");
    }
}

// ── 1. Vec<T> field detection: plain Vec recognized ─────────────────────────

#[test]
fn vec_field_detected_as_repeat() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Expr>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Expr");
}

// ── 2. Vec<T> field detection: non-Vec not detected ─────────────────────────

#[test]
fn non_vec_field_not_detected() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

// ── 3. Vec<T> field detection: Option<T> not confused with Vec ──────────────

#[test]
fn option_not_confused_with_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Expr>);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

// ── 4. Delimiter pattern: comma text literal ────────────────────────────────

#[test]
fn delimiter_comma_text() {
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
    assert_eq!(extract_delim_text(field), ",");
}

// ── 5. Delimiter pattern: semicolon ─────────────────────────────────────────

#[test]
fn delimiter_semicolon() {
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
    assert_eq!(extract_delim_text(field), ";");
}

// ── 6. Delimiter pattern: pipe ──────────────────────────────────────────────

#[test]
fn delimiter_pipe() {
    let s: ItemStruct = parse_quote! {
        pub struct Alts {
            #[adze::delimited(
                #[adze::leaf(text = "|")]
                ()
            )]
            alts: Vec<Alt>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(extract_delim_text(field), "|");
}

// ── 7. Delimiter pattern: double colon ──────────────────────────────────────

#[test]
fn delimiter_double_colon() {
    let s: ItemStruct = parse_quote! {
        pub struct Path {
            #[adze::delimited(
                #[adze::leaf(text = "::")]
                ()
            )]
            segments: Vec<Segment>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(extract_delim_text(field), "::");
}

// ── 8. Delimiter pattern: arrow (=>) ────────────────────────────────────────

#[test]
fn delimiter_fat_arrow() {
    let s: ItemStruct = parse_quote! {
        pub struct MatchArms {
            #[adze::delimited(
                #[adze::leaf(text = "=>")]
                ()
            )]
            arms: Vec<Arm>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(extract_delim_text(field), "=>");
}

// ── 9. Delimiter pattern: dot ───────────────────────────────────────────────

#[test]
fn delimiter_dot() {
    let s: ItemStruct = parse_quote! {
        pub struct DottedName {
            #[adze::delimited(
                #[adze::leaf(text = ".")]
                ()
            )]
            parts: Vec<Ident>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(extract_delim_text(field), ".");
}

// ── 10. Delimiter pattern: regex pattern separator ──────────────────────────

#[test]
fn delimiter_regex_pattern() {
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

// ── 11. Delimiter inner field type is unit ──────────────────────────────────

#[test]
fn delimiter_inner_type_is_unit() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ",")]
        ()
    );
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
}

// ── 12. Delimiter has no extra params ───────────────────────────────────────

#[test]
fn delimiter_no_extra_params() {
    let ftp: FieldThenParams = parse_quote!(
        #[adze::leaf(text = ";")]
        ()
    );
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

// ── 13. Empty repeat: Vec without repeat annotation still extractable ───────

#[test]
fn bare_vec_no_repeat_annotation() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            items: Vec<Node>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Node");
}

// ── 14. Repeat with non_empty = true ────────────────────────────────────────

#[test]
fn repeat_non_empty_true_extracted() {
    let s: ItemStruct = parse_quote! {
        pub struct Numbers {
            #[adze::repeat(non_empty = true)]
            nums: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field.attrs.iter().find(|a| is_adze_attr(a, "repeat")).unwrap();
    let params = repeat_params(attr);
    assert_eq!(params[0].path.to_string(), "non_empty");
    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) = &params[0].expr {
        assert!(b.value);
    } else {
        panic!("Expected bool literal");
    }
}

// ── 15. Repeat with non_empty = false ───────────────────────────────────────

#[test]
fn repeat_non_empty_false_extracted() {
    let s: ItemStruct = parse_quote! {
        pub struct Things {
            #[adze::repeat(non_empty = false)]
            things: Vec<Thing>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field.attrs.iter().find(|a| is_adze_attr(a, "repeat")).unwrap();
    let params = repeat_params(attr);
    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Bool(b), .. }) = &params[0].expr {
        assert!(!b.value);
    } else {
        panic!("Expected bool literal");
    }
}

// ── 16. Repeat with field name: named struct field ──────────────────────────

#[test]
fn repeat_on_named_struct_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Program {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ";")]
                ()
            )]
            statements: Vec<Statement>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ident.as_ref().unwrap(), "statements");
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

// ── 17. Repeat on unnamed enum field ────────────────────────────────────────

#[test]
fn repeat_on_unnamed_enum_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            List(
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

// ── 18. Delimited on unnamed enum field ─────────────────────────────────────

#[test]
fn delimited_on_unnamed_enum_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Csv(
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                Vec<Value>
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert!(u.unnamed[0].attrs.iter().any(|a| is_adze_attr(a, "delimited")));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 19. Combined repeat + delimited: both attributes present ────────────────

#[test]
fn combined_repeat_and_delimited() {
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
    let names = adze_attr_names(&field.attrs);
    assert!(names.contains(&"repeat".to_string()));
    assert!(names.contains(&"delimited".to_string()));
    assert_eq!(names.len(), 2);
}

// ── 20. Combined: delimited before repeat ordering preserved ────────────────

#[test]
fn delimited_first_then_repeat_ordering() {
    let s: ItemStruct = parse_quote! {
        pub struct Items {
            #[adze::delimited(
                #[adze::leaf(text = "|")]
                ()
            )]
            #[adze::repeat(non_empty = false)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let names = adze_attr_names(&field.attrs);
    assert_eq!(names[0], "delimited");
    assert_eq!(names[1], "repeat");
}

// ── 21. Nested Vec<Vec<T>>: outer Vec extractable ───────────────────────────

#[test]
fn nested_vec_outer_extractable() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Vec<Number>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < Number >");
}

// ── 22. Nested Vec<Vec<T>>: inner Vec also extractable ──────────────────────

#[test]
fn nested_vec_inner_extractable() {
    let skip: HashSet<&str> = HashSet::new();
    let outer: Type = parse_quote!(Vec<Vec<Number>>);
    let (mid, _) = try_extract_inner_type(&outer, "Vec", &skip);
    let (inner, extracted) = try_extract_inner_type(&mid, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

// ── 23. Nested Vec: wrap_leaf_type with Vec in skip set ─────────────────────

#[test]
fn nested_vec_wrap_leaf() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Vec<Number>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < Vec < adze :: WithLeaf < Number > > >"
    );
}

// ── 24. Repeat combined with prec_left on enum variant ──────────────────────

#[test]
fn repeat_combined_with_prec_left() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(1)]
            Call(
                Box<Expr>,
                #[adze::leaf(text = "(")]
                (),
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
    assert!(e.variants[1].attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    if let Fields::Unnamed(ref u) = e.variants[1].fields {
        let vec_field = &u.unnamed[2];
        assert!(vec_field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        assert_eq!(vec_field.ty.to_token_stream().to_string(), "Vec < Expr >");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 25. Repeat combined with skip field ─────────────────────────────────────

#[test]
fn repeat_alongside_skip_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Container {
            #[adze::repeat(non_empty = false)]
            items: Vec<Item>,
            #[adze::skip(0usize)]
            count: usize,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert_eq!(fields.len(), 2);
    assert!(fields[0].attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    assert!(fields[1].attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 26. Multiple Vec fields with different delimiters ───────────────────────

#[test]
fn multiple_vec_fields_different_delimiters() {
    let s: ItemStruct = parse_quote! {
        pub struct Table {
            #[adze::delimited(
                #[adze::leaf(text = "\n")]
                ()
            )]
            rows: Vec<Row>,
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            headers: Vec<Header>,
        }
    };
    let fields: Vec<_> = s.fields.iter().collect();
    assert_eq!(extract_delim_text(fields[0]), "\n");
    assert_eq!(extract_delim_text(fields[1]), ",");
}

// ── 27. Vec<T> through Box skip ─────────────────────────────────────────────

#[test]
fn vec_extracted_through_box_skip() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<Number>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Number");
}

// ── 28. Option<Vec<T>> with Option skip ─────────────────────────────────────

#[test]
fn option_vec_extracted_through_option_skip() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<Item>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Item");
}

// ── 29. Enum named-field variant with delimited ─────────────────────────────

#[test]
fn enum_named_field_variant_with_delimited() {
    let e: ItemEnum = parse_quote! {
        pub enum Node {
            Block {
                #[adze::leaf(text = "{")]
                _open: (),
                #[adze::delimited(
                    #[adze::leaf(text = ";")]
                    ()
                )]
                stmts: Vec<Stmt>,
                #[adze::leaf(text = "}")]
                _close: (),
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let stmts = n.named.iter().find(|f| f.ident.as_ref().unwrap() == "stmts").unwrap();
        assert!(stmts.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        assert_eq!(extract_delim_text(stmts), ";");
    } else {
        panic!("Expected named fields");
    }
}

// ── 30. Grammar module: delimited + repeat fields preserved ─────────────────

#[test]
fn grammar_module_delimited_repeat_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct ArgList {
                #[adze::repeat(non_empty = false)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                args: Vec<Arg>,
            }

            pub struct Arg {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        assert_eq!(s.ident, "ArgList");
        let field = s.fields.iter().next().unwrap();
        let names = adze_attr_names(&field.attrs);
        assert!(names.contains(&"repeat".to_string()));
        assert!(names.contains(&"delimited".to_string()));
    } else {
        panic!("Expected struct");
    }
}

// ── 31. filter_inner_type strips Vec wrapper ────────────────────────────────

#[test]
fn filter_inner_type_strips_vec() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Number>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Number");
}

// ── 32. filter_inner_type leaves non-Vec unchanged ──────────────────────────

#[test]
fn filter_inner_type_leaves_non_vec() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Number>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Option < Number >");
}

// ── 33. wrap_leaf_type: Option<Vec<T>> with both in skip ────────────────────

#[test]
fn wrap_leaf_option_vec_both_skipped() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<Number>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Vec < adze :: WithLeaf < Number > > >"
    );
}

// ── 34. Attr count: exactly 2 adze attrs on combined field ──────────────────

#[test]
fn attr_count_repeat_delimited_is_two() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let adze_count = field.attrs.iter().filter(|a| {
        let segs: Vec<_> = a.path().segments.iter().collect();
        segs.len() == 2 && segs[0].ident == "adze"
    }).count();
    assert_eq!(adze_count, 2);
}

// ── 35. Full grammar module with nested delimited repeats ───────────────────

#[test]
fn grammar_module_nested_delimited_structures() {
    let m = parse_mod(quote! {
        #[adze::grammar("csv")]
        mod grammar {
            #[adze::language]
            pub struct CsvFile {
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
                #[adze::leaf(pattern = r"[^,\n]+")]
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

    // Verify CsvFile has delimited field
    if let Item::Struct(csv) = &items[0] {
        let field = csv.fields.iter().next().unwrap();
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        assert_eq!(extract_delim_text(field), "\n");
    }
    // Verify Row has delimited field
    if let Item::Struct(row) = &items[1] {
        let field = row.fields.iter().next().unwrap();
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
        assert_eq!(extract_delim_text(field), ",");
    }
}
