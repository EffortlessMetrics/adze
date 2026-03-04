#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `#[adze::language]` attribute handling in the adze macro crate.
//!
//! Tests cover language annotation on structs and enums, interaction with other
//! attributes (derives, visibility, leaf, extra), detection inside grammar modules,
//! multiple types with a single language root, and edge cases.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, parse_quote};

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

fn count_language_types(m: &ItemMod) -> usize {
    module_items(m)
        .iter()
        .filter(|item| match item {
            Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "language")),
            Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "language")),
            _ => false,
        })
        .count()
}

fn find_struct_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| match i {
        Item::Struct(s) if s.ident == name => Some(s),
        _ => None,
    })
}

fn find_enum_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| match i {
        Item::Enum(e) if e.ident == name => Some(e),
        _ => None,
    })
}

// ── 1. Language attribute recognized on struct ──────────────────────────────

#[test]
fn language_on_struct_recognized() {
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

// ── 2. Language attribute recognized on enum ────────────────────────────────

#[test]
fn language_on_enum_recognized() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expression {
            Number(i32),
            Add(Box<Expression>, Box<Expression>),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(e.ident.to_string(), "Expression");
}

// ── 3. Language attribute is path-style (no arguments) ──────────────────────

#[test]
fn language_attr_is_path_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {}
    };
    let lang_attr = s
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "language"))
        .unwrap();
    assert!(
        matches!(lang_attr.meta, syn::Meta::Path(_)),
        "Expected path-style attribute with no arguments"
    );
}

// ── 4. Language attribute path has two segments ─────────────────────────────

#[test]
fn language_attr_path_segments() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {}
    };
    let lang_attr = s
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "language"))
        .unwrap();
    let segs: Vec<_> = lang_attr.path().segments.iter().collect();
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].ident.to_string(), "adze");
    assert_eq!(segs[1].ident.to_string(), "language");
}

// ── 5. Language struct detected inside grammar module ───────────────────────

#[test]
fn language_struct_in_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Program".to_string()));
}

// ── 6. Language enum detected inside grammar module ─────────────────────────

#[test]
fn language_enum_in_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
}

// ── 7. Language with derive attributes ──────────────────────────────────────

#[test]
fn language_with_derives() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        #[adze::language]
        pub struct Program {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    let attr_names: Vec<_> = s
        .attrs
        .iter()
        .map(|a| a.to_token_stream().to_string())
        .collect();
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    // Derive is preserved alongside language
    assert!(attr_names.iter().any(|n| n.contains("derive")));
}

// ── 8. Language with derive on enum ─────────────────────────────────────────

#[test]
fn language_with_derives_on_enum() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug, PartialEq)]
        #[adze::language]
        pub enum Token {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    let derive_attr = e.attrs.iter().find(|a| a.path().is_ident("derive"));
    assert!(
        derive_attr.is_some(),
        "derive attribute should be preserved"
    );
}

// ── 9. Language with additional doc attributes ──────────────────────────────

#[test]
fn language_with_doc_comments() {
    let s: ItemStruct = parse_quote! {
        /// The root of the grammar.
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    // doc comment becomes a doc attribute
    let doc_attrs: Vec<_> = s
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .collect();
    assert!(!doc_attrs.is_empty(), "doc attribute should be preserved");
}

// ── 10. Pub struct language visibility ──────────────────────────────────────

#[test]
fn language_pub_struct_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
}

// ── 11. Private struct language visibility ──────────────────────────────────

#[test]
fn language_private_struct_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct Root {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Inherited));
}

// ── 12. Pub(crate) struct language visibility ───────────────────────────────

#[test]
fn language_pub_crate_struct_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub(crate) struct Root {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
}

// ── 13. Multiple types but only one language ────────────────────────────────

#[test]
fn only_one_language_among_multiple_types() {
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

            pub struct Helper {
                value: i32,
            }
        }
    });
    assert_eq!(count_language_types(&m), 1);
    assert_eq!(find_language_type(&m), Some("Program".to_string()));
}

// ── 14. Non-language types have no language attr ────────────────────────────

#[test]
fn non_language_types_have_no_language_attr() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(i32),
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+")]
                v: i32,
            }
        }
    });
    let number = find_struct_in_mod(&m, "Number").unwrap();
    assert!(!number.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ── 15. Language combined with leaf (enum variant leaf) ─────────────────────

#[test]
fn language_enum_with_leaf_variants() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Token {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    for v in &e.variants {
        let names = adze_attr_names(&v.attrs);
        assert!(names.contains(&"leaf".to_string()), "variant {}", v.ident);
    }
}

// ── 16. Language struct with leaf fields ─────────────────────────────────────

#[test]
fn language_struct_with_leaf_fields() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    if let Fields::Named(ref fields) = s.fields {
        let field = &fields.named[0];
        let names = adze_attr_names(&field.attrs);
        assert!(names.contains(&"leaf".to_string()));
    } else {
        panic!("Expected named fields");
    }
}

// ── 17. Language only attr among adze attrs ─────────────────────────────────

#[test]
fn language_is_only_adze_attr_on_type() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    let type_adze_attrs = adze_attr_names(&s.attrs);
    assert_eq!(type_adze_attrs, vec!["language"]);
}

// ── 18. Language absent means no language type found ─────────────────────────

#[test]
fn no_language_attr_means_none_found() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            pub struct Helper {
                value: i32,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    assert_eq!(find_language_type(&m), None);
    assert_eq!(count_language_types(&m), 0);
}

// ── 19. Language on struct with skip field ───────────────────────────────────

#[test]
fn language_struct_with_skip_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct MyNode {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    if let Fields::Named(ref fields) = s.fields {
        assert_eq!(fields.named.len(), 2);
        let skip_field = &fields.named[1];
        assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    } else {
        panic!("Expected named fields");
    }
}

// ── 20. Language on enum with prec_left variant ─────────────────────────────

#[test]
fn language_enum_with_prec_left_variant() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expression {
            Number(i32),
            #[adze::prec_left(1)]
            Add(Box<Expression>, Box<Expression>),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    let add_variant = e.variants.iter().find(|v| v.ident == "Add").unwrap();
    let names = adze_attr_names(&add_variant.attrs);
    assert!(names.contains(&"prec_left".to_string()));
}

// ── 21. Language on struct with Vec field ────────────────────────────────────

#[test]
fn language_struct_with_vec_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct NumberList {
            numbers: Vec<Number>,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    if let Fields::Named(ref fields) = s.fields {
        let ty_str = fields.named[0].ty.to_token_stream().to_string();
        assert!(ty_str.contains("Vec"), "field type should be Vec<Number>");
    } else {
        panic!("Expected named fields");
    }
}

// ── 22. Language on struct with Option field ────────────────────────────────

#[test]
fn language_struct_with_option_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct MaybeValue {
            value: Option<i32>,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    if let Fields::Named(ref fields) = s.fields {
        let ty_str = fields.named[0].ty.to_token_stream().to_string();
        assert!(ty_str.contains("Option"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 23. Language struct with Box field ───────────────────────────────────────

#[test]
fn language_struct_with_box_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Wrapper {
            inner: Box<Expr>,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    if let Fields::Named(ref fields) = s.fields {
        let ty_str = fields.named[0].ty.to_token_stream().to_string();
        assert!(ty_str.contains("Box"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 24. Language on enum with mixed named and unnamed variants ──────────────

#[test]
fn language_enum_mixed_variant_kinds() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Number(i32),
            Neg {
                #[adze::leaf(text = "!")]
                _bang: (),
                value: Box<Expr>,
            },
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    let number = e.variants.iter().find(|v| v.ident == "Number").unwrap();
    assert!(matches!(number.fields, Fields::Unnamed(_)));
    let neg = e.variants.iter().find(|v| v.ident == "Neg").unwrap();
    assert!(matches!(neg.fields, Fields::Named(_)));
}

// ── 25. Language coexists with extra types ──────────────────────────────────

#[test]
fn language_coexists_with_extra_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
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
    assert_eq!(find_language_type(&m), Some("Code".to_string()));
    assert_eq!(count_language_types(&m), 1);
    // extras are not language types
    let ws = find_struct_in_mod(&m, "Whitespace").unwrap();
    assert!(!ws.attrs.iter().any(|a| is_adze_attr(a, "language")));
    let comment = find_struct_in_mod(&m, "Comment").unwrap();
    assert!(!comment.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ── 26. Language attr order does not matter ──────────────────────────────────

#[test]
fn language_attr_after_derive() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ── 27. Language attr before derive ─────────────────────────────────────────

#[test]
fn language_attr_before_derive() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[derive(Clone)]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    let derive_attr = s.attrs.iter().find(|a| a.path().is_ident("derive"));
    assert!(derive_attr.is_some());
}

// ── 28. Language on unit struct ─────────────────────────────────────────────

#[test]
fn language_on_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct EmptyRoot;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert!(matches!(s.fields, Fields::Unit));
}

// ── 29. Language on tuple struct ────────────────────────────────────────────

#[test]
fn language_on_tuple_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root(String);
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert!(matches!(s.fields, Fields::Unnamed(_)));
}

// ── 30. Language enum with unit variants ────────────────────────────────────

#[test]
fn language_enum_with_unit_variants() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Keyword {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
            #[adze::leaf(text = "while")]
            While,
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(e.variants.len(), 3);
    for v in &e.variants {
        assert!(matches!(v.fields, Fields::Unit));
    }
}

// ── 31. Language detection ignores non-struct/enum items ─────────────────────

#[test]
fn language_detection_ignores_fn_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }

            fn helper() -> i32 { 42 }
        }
    });
    // Only the struct should be detected as language
    assert_eq!(count_language_types(&m), 1);
    assert_eq!(find_language_type(&m), Some("Root".to_string()));
}

// ── 32. Language struct preserves field count ────────────────────────────────

#[test]
fn language_struct_field_count_preserved() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Complex {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::leaf(pattern = r"\d+")]
            count: String,
            #[adze::skip(0)]
            meta: i32,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    if let Fields::Named(ref fields) = s.fields {
        assert_eq!(fields.named.len(), 3);
    } else {
        panic!("Expected named fields");
    }
}

// ── 33. Language enum preserves variant count ────────────────────────────────

#[test]
fn language_enum_variant_count_preserved() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Number(i32),
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(1)]
            Sub(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, Box<Expr>),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(e.variants.len(), 4);
}

// ── 34. Language type name identity ─────────────────────────────────────────

#[test]
fn language_type_name_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_lang")]
        mod grammar {
            #[adze::language]
            pub struct MyCustomRootName {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("MyCustomRootName".to_string()));
}

// ── 35. Language enum with prec_right variant ───────────────────────────────

#[test]
fn language_enum_with_prec_right() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Lit(i32),
            #[adze::prec_right(1)]
            Cons(Box<Expr>, Box<Expr>),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    let cons = e.variants.iter().find(|v| v.ident == "Cons").unwrap();
    let names = adze_attr_names(&cons.attrs);
    assert!(names.contains(&"prec_right".to_string()));
}
