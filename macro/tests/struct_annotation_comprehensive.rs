#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for struct-level annotation handling in the adze macro crate.
//!
//! Tests cover struct as language root, leaf fields, skip fields, mixed field types,
//! generic wrappers (Box, Vec, Option), unit structs, tuple structs, struct visibility,
//! derives, and named fields with various annotations.

use adze_common::NameValueExpr;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemMod, ItemStruct, Token, parse_quote};

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

fn module_items(m: &ItemMod) -> &Vec<Item> {
    &m.content.as_ref().unwrap().1
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
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

// ── 1. Struct as language root ──────────────────────────────────────────────

#[test]
fn struct_language_root_recognized() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Program {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(s.ident.to_string(), "Program");
}

// ── 2. Struct language root in module context ───────────────────────────────

#[test]
fn struct_language_root_in_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    assert!(root.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ── 3. Struct with single leaf field ────────────────────────────────────────

#[test]
fn struct_single_leaf_field_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert_eq!(s.fields.iter().count(), 1);
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "pattern");
}

// ── 4. Struct with leaf text field ──────────────────────────────────────────

#[test]
fn struct_leaf_text_literal_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Semicolon {
            #[adze::leaf(text = ";")]
            _semi: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "text");
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

// ── 5. Struct with leaf transform field ─────────────────────────────────────

#[test]
fn struct_leaf_with_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct Number {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    let param_names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(param_names.contains(&"pattern".to_string()));
    assert!(param_names.contains(&"transform".to_string()));
}

// ── 6. Struct with skip field ───────────────────────────────────────────────

#[test]
fn struct_skip_field_recognized() {
    let s: ItemStruct = parse_quote! {
        pub struct MyNode {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
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
}

// ── 7. Struct with skip field default value ─────────────────────────────────

#[test]
fn struct_skip_field_has_default_expr() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(0)]
            counter: i32,
        }
    };
    let skip_field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "counter"))
        .unwrap();
    let attr = skip_field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(expr.to_token_stream().to_string(), "0");
}

// ── 8. Struct with mixed leaf and skip fields ───────────────────────────────

#[test]
fn struct_mixed_leaf_and_skip_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct MixedNode {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
            #[adze::leaf(text = ";")]
            _semi: (),
            #[adze::skip(false)]
            processed: bool,
        }
    };
    let adze_fields: Vec<_> = s.fields.iter().map(|f| adze_attr_names(&f.attrs)).collect();
    assert_eq!(adze_fields[0], vec!["leaf"]);
    assert_eq!(adze_fields[1], vec!["leaf"]);
    assert_eq!(adze_fields[2], vec!["skip"]);
}

// ── 9. Struct with Vec field ────────────────────────────────────────────────

#[test]
fn struct_vec_field_type() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct NumberList {
            numbers: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "Vec < Number >");
}

// ── 10. Struct with Option field ────────────────────────────────────────────

#[test]
fn struct_option_field_type() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct MaybeNumber {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            v: Option<i32>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "Option < i32 >");
}

// ── 11. Struct with Box field ───────────────────────────────────────────────

#[test]
fn struct_box_field_type() {
    let s: ItemStruct = parse_quote! {
        pub struct Wrapper {
            inner: Box<Inner>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "Box < Inner >");
}

// ── 12. Struct with multiple generic field types ────────────────────────────

#[test]
fn struct_multiple_generic_fields() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Program {
            items: Vec<Item>,
            name: Option<Identifier>,
            body: Box<Block>,
        }
    };
    let types: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    assert_eq!(types[0], "Vec < Item >");
    assert_eq!(types[1], "Option < Identifier >");
    assert_eq!(types[2], "Box < Block >");
}

// ── 13. Unit struct ─────────────────────────────────────────────────────────

#[test]
fn unit_struct_parses() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "nil")]
        struct Nil;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

// ── 14. Unit struct with leaf text ──────────────────────────────────────────

#[test]
fn unit_struct_leaf_text_value() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "true")]
        struct TrueKeyword;
    };
    let attr = s.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "true");
    } else {
        panic!("Expected string literal");
    }
}

// ── 15. Tuple struct ────────────────────────────────────────────────────────

#[test]
fn tuple_struct_single_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Wrapped(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            i32
        );
    };
    assert!(matches!(s.fields, Fields::Unnamed(_)));
    if let Fields::Unnamed(ref u) = s.fields {
        assert_eq!(u.unnamed.len(), 1);
    }
}

// ── 16. Tuple struct with multiple fields ───────────────────────────────────

#[test]
fn tuple_struct_multiple_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Pair(
            #[adze::leaf(pattern = r"\d+")]
            String,
            #[adze::leaf(text = ",")]
            (),
            #[adze::leaf(pattern = r"\d+")]
            String,
        );
    };
    if let Fields::Unnamed(ref u) = s.fields {
        assert_eq!(u.unnamed.len(), 3);
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 17. Struct public visibility ────────────────────────────────────────────

#[test]
fn struct_public_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
}

// ── 18. Struct private (inherited) visibility ───────────────────────────────

#[test]
fn struct_private_visibility() {
    let s: ItemStruct = parse_quote! {
        struct Internal {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Inherited));
}

// ── 19. Struct crate visibility ─────────────────────────────────────────────

#[test]
fn struct_crate_visibility() {
    let s: ItemStruct = parse_quote! {
        pub(crate) struct CrateOnly {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
}

// ── 20. Struct with derive attributes preserved ─────────────────────────────

#[test]
fn struct_derive_attrs_preserved() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        #[adze::language]
        pub struct Program {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert_eq!(s.attrs.len(), 2);
    let derive_attr = s.attrs.iter().find(|a| {
        a.path()
            .segments
            .iter()
            .next()
            .is_some_and(|s| s.ident == "derive")
    });
    assert!(derive_attr.is_some());
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ── 21. Struct with multiple derive and adze attrs ──────────────────────────

#[test]
fn struct_multiple_derive_and_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[derive(Clone, PartialEq)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let derive_count = s
        .attrs
        .iter()
        .filter(|a| {
            a.path()
                .segments
                .iter()
                .next()
                .is_some_and(|s| s.ident == "derive")
        })
        .count();
    assert_eq!(derive_count, 2);
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ── 22. Struct field name preservation ──────────────────────────────────────

#[test]
fn struct_field_names_preserved() {
    let s: ItemStruct = parse_quote! {
        pub struct Statement {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            identifier: String,
            #[adze::leaf(text = "=")]
            _equals: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let names: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(names, vec!["identifier", "_equals", "value"]);
}

// ── 23. Struct with underscore-prefixed fields ──────────────────────────────

#[test]
fn struct_underscore_prefixed_fields() {
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
    let names: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(names, vec!["_open", "content", "_close"]);
    // All three fields have leaf annotations
    for field in s.fields.iter() {
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 24. Struct with Vec and repeat annotation ───────────────────────────────

#[test]
fn struct_vec_with_repeat() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct NumberList {
            #[adze::repeat(non_empty = true)]
            numbers: Vec<Number>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    assert_eq!(field.ty.to_token_stream().to_string(), "Vec < Number >");
}

// ── 25. Struct with Vec and delimited annotation ────────────────────────────

#[test]
fn struct_vec_with_delimited() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct CommaSeparated {
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

// ── 26. Struct with reference to another struct ─────────────────────────────

#[test]
fn struct_cross_reference_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                header: Header,
                body: Body,
            }

            pub struct Header {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            pub struct Body {
                #[adze::leaf(pattern = r"[^}]+")]
                content: String,
            }
        }
    });
    let program = find_struct_in_mod(&m, "Program").unwrap();
    let field_types: Vec<_> = program
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    assert_eq!(field_types, vec!["Header", "Body"]);
    assert!(find_struct_in_mod(&m, "Header").is_some());
    assert!(find_struct_in_mod(&m, "Body").is_some());
}

// ── 27. Struct with word annotation ─────────────────────────────────────────

#[test]
fn struct_word_annotation() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    assert_eq!(s.fields.iter().count(), 1);
}

// ── 28. Struct with external annotation ─────────────────────────────────────

#[test]
fn struct_external_annotation_unit() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
    assert!(matches!(s.fields, Fields::Unit));
}

// ── 29. Struct with Option and non-leaf reference ───────────────────────────

#[test]
fn struct_option_non_leaf_reference() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Language {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                v: Option<i32>,
                t: Option<Number>,
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                v: i32,
            }
        }
    });
    let lang = find_struct_in_mod(&m, "Language").unwrap();
    let field_types: Vec<_> = lang
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    assert_eq!(field_types[0], "Option < i32 >");
    assert_eq!(field_types[1], "Option < Number >");
}

// ── 30. Struct field type is unit for punctuation ───────────────────────────

#[test]
fn struct_unit_typed_punctuation_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Assignment {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::leaf(text = ";")]
            _semi: (),
        }
    };
    let unit_fields: Vec<_> = s
        .fields
        .iter()
        .filter(|f| f.ty.to_token_stream().to_string() == "()")
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(unit_fields, vec!["_eq", "_semi"]);
}

// ── 31. Struct extra annotation ─────────────────────────────────────────────

#[test]
fn struct_extra_with_leaf_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

// ── 32. Struct field count matches definition ───────────────────────────────

#[test]
fn struct_field_count_matches() {
    let s: ItemStruct = parse_quote! {
        pub struct FiveFields {
            #[adze::leaf(pattern = r"\w+")]
            a: String,
            #[adze::leaf(text = ",")]
            _b: (),
            #[adze::leaf(pattern = r"\w+")]
            c: String,
            #[adze::skip(0)]
            d: i32,
            e: Option<Other>,
        }
    };
    assert_eq!(s.fields.iter().count(), 5);
}

// ── 33. Struct with all annotation combinations in module ───────────────────

#[test]
fn struct_all_annotation_types_in_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("combo")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                stmts: Vec<Stmt>,
            }

            pub struct Stmt {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
                #[adze::leaf(text = ";")]
                _semi: (),
                #[adze::skip(false)]
                checked: bool,
            }

            #[adze::word]
            pub struct Keyword {
                #[adze::leaf(pattern = r"[a-z]+")]
                word: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::external]
            struct Indent;
        }
    });
    let items = module_items(&m);
    let mut found_language = false;
    let mut found_word = false;
    let mut found_extra = false;
    let mut found_external = false;
    for item in items {
        if let Item::Struct(s) = item {
            if s.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                found_language = true;
            }
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
    assert!(found_language);
    assert!(found_word);
    assert!(found_extra);
    assert!(found_external);
}

// ── 34. Struct non-adze attrs not confused ──────────────────────────────────

#[test]
fn struct_non_adze_attrs_not_confused() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone, PartialEq)]
        #[cfg(test)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    let adze_names = adze_attr_names(&s.attrs);
    assert_eq!(adze_names, vec!["language"]);
    assert_eq!(s.attrs.len(), 3);
}

// ── 35. Struct leaf pattern with complex regex ──────────────────────────────

#[test]
fn struct_leaf_complex_regex_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct StringLiteral {
            #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#)]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert!(s.value().contains(r#"([^"\\]|\\.)*"#));
    } else {
        panic!("Expected string literal");
    }
}
