#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `#[adze::skip]` attribute handling in adze-macro.
//!
//! The skip attribute marks fields that do not correspond to anything in the
//! input string. At runtime the field is populated with the expression given
//! as the attribute argument. These tests verify parsing, attribute recognition,
//! default-expression extraction, interaction with other annotations, and
//! applicability to both struct and enum-variant fields.

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

fn module_items(m: &ItemMod) -> &Vec<Item> {
    &m.content.as_ref().unwrap().1
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

fn find_enum_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == name { Some(e) } else { None }
        } else {
            None
        }
    })
}

fn skip_expr_str(attr: &Attribute) -> String {
    attr.parse_args::<syn::Expr>()
        .expect("skip attribute should contain an expression")
        .to_token_stream()
        .to_string()
}

// ── 1. Skip on a single struct field ────────────────────────────────────────

#[test]
fn skip_single_struct_field_recognized() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "visited"))
        .unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 2. Skip default expression — bool false ─────────────────────────────────

#[test]
fn skip_bool_false_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(false)]
            flag: bool,
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
    assert_eq!(skip_expr_str(attr), "false");
}

// ── 3. Skip default expression — integer literal ────────────────────────────

#[test]
fn skip_integer_literal_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(42)]
            count: i32,
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
    assert_eq!(skip_expr_str(attr), "42");
}

// ── 4. Skip default expression — typed integer (u32) ────────────────────────

#[test]
fn skip_typed_integer_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(0u32)]
            counter: u32,
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
    assert_eq!(skip_expr_str(attr), "0u32");
}

// ── 5. Skip default expression — usize ──────────────────────────────────────

#[test]
fn skip_usize_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(0usize)]
            idx: usize,
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
    assert_eq!(skip_expr_str(attr), "0usize");
}

// ── 6. Skip default expression — constructor call ───────────────────────────

#[test]
fn skip_constructor_call_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(String::new())]
            label: String,
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
    assert_eq!(skip_expr_str(attr), "String :: new ()");
}

// ── 7. Skip default expression — Vec::new() ────────────────────────────────

#[test]
fn skip_vec_new_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(Vec::new())]
            items: Vec<i32>,
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
    assert_eq!(skip_expr_str(attr), "Vec :: new ()");
}

// ── 8. Skip default expression — None ───────────────────────────────────────

#[test]
fn skip_none_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(None)]
            maybe: Option<i32>,
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
    assert_eq!(skip_expr_str(attr), "None");
}

// ── 9. Skip on multiple struct fields ───────────────────────────────────────

#[test]
fn skip_multiple_struct_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Multi {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(false)]
            processed: bool,
            #[adze::skip(0u32)]
            counter: u32,
            #[adze::skip(String::new())]
            memo: String,
        }
    };
    let skip_fields: Vec<_> = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(skip_fields, vec!["processed", "counter", "memo"]);
}

// ── 10. Skip mixed with leaf fields preserves field order ───────────────────

#[test]
fn skip_mixed_with_leaf_field_order() {
    let s: ItemStruct = parse_quote! {
        pub struct Ordered {
            #[adze::skip(0)]
            pre_meta: i32,
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(false)]
            post_meta: bool,
        }
    };
    let annotations: Vec<Vec<String>> =
        s.fields.iter().map(|f| adze_attr_names(&f.attrs)).collect();
    assert_eq!(annotations[0], vec!["skip"]);
    assert_eq!(annotations[1], vec!["leaf"]);
    assert_eq!(annotations[2], vec!["skip"]);
}

// ── 11. All fields skipped struct ───────────────────────────────────────────

#[test]
fn all_fields_skipped_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct AllSkipped {
            #[adze::skip(false)]
            a: bool,
            #[adze::skip(0)]
            b: i32,
            #[adze::skip(String::new())]
            c: String,
        }
    };
    let non_skip: Vec<_> = s
        .fields
        .iter()
        .filter(|f| !f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .collect();
    assert!(non_skip.is_empty(), "all fields should be skip");
    assert_eq!(s.fields.iter().count(), 3);
}

// ── 12. Skip field type is bool ─────────────────────────────────────────────

#[test]
fn skip_field_type_bool() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(true)]
            flag: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "bool");
}

// ── 13. Skip field type is i32 ──────────────────────────────────────────────

#[test]
fn skip_field_type_i32() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(0)]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "i32");
}

// ── 14. Skip field type is String ───────────────────────────────────────────

#[test]
fn skip_field_type_string() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(String::new())]
            label: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "String");
}

// ── 15. Skip field type is Option ───────────────────────────────────────────

#[test]
fn skip_field_type_option() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(None)]
            maybe: Option<String>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "Option < String >");
}

// ── 16. Skip field type is Vec ──────────────────────────────────────────────

#[test]
fn skip_field_type_vec() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(Vec::new())]
            items: Vec<u64>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "Vec < u64 >");
}

// ── 17. Skip on public field ────────────────────────────────────────────────

#[test]
fn skip_public_field_visibility() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(false)]
            pub visible: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Public(_)));
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 18. Skip on private (inherited) field ───────────────────────────────────

#[test]
fn skip_private_field_visibility() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(false)]
            private_flag: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Inherited));
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 19. Skip on pub(crate) field ────────────────────────────────────────────

#[test]
fn skip_crate_visibility_field() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(0)]
            pub(crate) internal: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Restricted(_)));
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 20. Skip on enum variant named field ────────────────────────────────────

#[test]
fn skip_enum_variant_named_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Literal {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
                #[adze::skip(false)]
                cached: bool,
            },
        }
    };
    let variant = &e.variants[0];
    assert_eq!(variant.ident, "Literal");
    if let Fields::Named(ref named) = variant.fields {
        let skip_field = named
            .named
            .iter()
            .find(|f| f.ident.as_ref().is_some_and(|i| i == "cached"))
            .unwrap();
        assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    } else {
        panic!("Expected named fields");
    }
}

// ── 21. Skip on enum variant unnamed field ──────────────────────────────────

#[test]
fn skip_enum_variant_unnamed_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            WithMeta(
                #[adze::leaf(pattern = r"\d+")]
                String,
                #[adze::skip(false)]
                bool,
            ),
        }
    };
    let variant = &e.variants[0];
    if let Fields::Unnamed(ref unnamed) = variant.fields {
        let skip_field = &unnamed.unnamed[1];
        assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 22. Multiple skip fields in enum variant ────────────────────────────────

#[test]
fn skip_multiple_enum_variant_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Annotated {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                #[adze::skip(0)]
                line: i32,
                #[adze::skip(0)]
                col: i32,
            },
        }
    };
    let variant = &e.variants[0];
    if let Fields::Named(ref named) = variant.fields {
        let skip_count = named
            .named
            .iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
            .count();
        assert_eq!(skip_count, 2);
    } else {
        panic!("Expected named fields");
    }
}

// ── 23. Skip combined with leaf on different fields ─────────────────────────

#[test]
fn skip_combined_with_leaf_different_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Statement {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
            #[adze::skip(false)]
            evaluated: bool,
        }
    };
    let annotations: Vec<Vec<String>> =
        s.fields.iter().map(|f| adze_attr_names(&f.attrs)).collect();
    assert_eq!(annotations[0], vec!["leaf"]);
    assert_eq!(annotations[1], vec!["leaf"]);
    assert_eq!(annotations[2], vec!["leaf"]);
    assert_eq!(annotations[3], vec!["skip"]);
}

// ── 24. Skip in grammar module struct ───────────────────────────────────────

#[test]
fn skip_in_grammar_module_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
                #[adze::skip(false)]
                visited: bool,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    let skip_field = root
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "visited"))
        .unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 25. Skip in grammar module non-root struct ──────────────────────────────

#[test]
fn skip_in_grammar_module_non_root_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Child,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                #[adze::skip(0)]
                depth: i32,
            }
        }
    });
    let child = find_struct_in_mod(&m, "Child").unwrap();
    let skip_field = child
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "depth"))
        .unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 26. Skip in grammar module enum variant ─────────────────────────────────

#[test]
fn skip_in_grammar_module_enum_variant() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num {
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    value: i32,
                    #[adze::skip(false)]
                    negated: bool,
                },
            }
        }
    });
    let expr = find_enum_in_mod(&m, "Expr").unwrap();
    let variant = &expr.variants[0];
    if let Fields::Named(ref named) = variant.fields {
        let skip_field = named
            .named
            .iter()
            .find(|f| f.ident.as_ref().is_some_and(|i| i == "negated"))
            .unwrap();
        assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    } else {
        panic!("Expected named fields");
    }
}

// ── 27. Skip default expression — bool true ─────────────────────────────────

#[test]
fn skip_bool_true_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(true)]
            active: bool,
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
    assert_eq!(skip_expr_str(attr), "true");
}

// ── 28. Skip default expression — negative integer ──────────────────────────

#[test]
fn skip_negative_integer_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(-1)]
            sentinel: i32,
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
    assert_eq!(skip_expr_str(attr), "- 1");
}

// ── 29. Skip does not carry leaf annotation ─────────────────────────────────

#[test]
fn skip_field_has_no_leaf_attr() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(false)]
            flag: bool,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(!field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 30. Skip field name preserved after parsing ─────────────────────────────

#[test]
fn skip_field_name_preserved() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::leaf(pattern = r"\w+")]
            token: String,
            #[adze::skip(false)]
            my_metadata_field: bool,
        }
    };
    let names: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(names, vec!["token", "my_metadata_field"]);
}

// ── 31. Skip default expression — float literal ─────────────────────────────

#[test]
fn skip_float_literal_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(0.0)]
            score: f64,
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
    assert_eq!(skip_expr_str(attr), "0.0");
}

// ── 32. Skip default expression — char literal ──────────────────────────────

#[test]
fn skip_char_literal_default() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip('x')]
            tag: char,
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
    assert_eq!(skip_expr_str(attr), "'x'");
}

// ── 33. Skip attr count equals expected on mixed struct ─────────────────────

#[test]
fn skip_attr_count_in_mixed_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct Mixed {
            #[adze::leaf(pattern = r"\d+")]
            a: String,
            #[adze::skip(false)]
            b: bool,
            #[adze::leaf(text = ";")]
            c: (),
            #[adze::skip(0)]
            d: i32,
            #[adze::skip(None)]
            e: Option<String>,
        }
    };
    let skip_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .count();
    let leaf_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
        .count();
    assert_eq!(skip_count, 3);
    assert_eq!(leaf_count, 2);
}

// ── 34. Skip with derive on struct does not interfere ───────────────────────

#[test]
fn skip_with_derive_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        pub struct N {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(false)]
            checked: bool,
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
    assert_eq!(derive_count, 1);
    let skip_field = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "checked"))
        .unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

// ── 35. Skip default expression — Default::default() ────────────────────────

#[test]
fn skip_default_trait_call() {
    let s: ItemStruct = parse_quote! {
        pub struct N {
            #[adze::skip(Default::default())]
            data: Vec<u8>,
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
    assert_eq!(skip_expr_str(attr), "Default :: default ()");
}
