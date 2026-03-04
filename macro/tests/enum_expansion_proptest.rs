#![allow(clippy::needless_range_loop)]

//! Property-based tests for enum expansion in adze-macro.
//!
//! Covers: simple unit-variant enums, tuple variants, struct variants,
//! mixed variant kinds, CHOICE-rule structure, variant naming in generated
//! rules, attributes on variants, and expansion determinism.

use proptest::prelude::*;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, parse_quote};

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

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn find_enum<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == name { Some(e) } else { None }
        } else {
            None
        }
    })
}

fn variant_names(e: &ItemEnum) -> Vec<String> {
    e.variants.iter().map(|v| v.ident.to_string()).collect()
}

/// Build the expected Tree-sitter symbol for a variant: `EnumName_VariantName`
fn expected_symbol(enum_name: &str, variant_name: &str) -> String {
    format!("{enum_name}_{variant_name}")
}

fn variant_is_unit(v: &syn::Variant) -> bool {
    matches!(v.fields, Fields::Unit)
}

fn variant_is_unnamed(v: &syn::Variant) -> bool {
    matches!(v.fields, Fields::Unnamed(_))
}

fn variant_is_named(v: &syn::Variant) -> bool {
    matches!(v.fields, Fields::Named(_))
}

fn field_type_strings(v: &syn::Variant) -> Vec<String> {
    match &v.fields {
        Fields::Unnamed(u) => u
            .unnamed
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Named(n) => n
            .named
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Unit => vec![],
    }
}

// ── 1. Simple enum with unit variants – count preserved ─────────────────────

proptest! {
    #[test]
    fn unit_variant_count_preserved(count in 2usize..=8) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("kw{i}");
                quote! {
                    #[adze::leaf(text = #text)]
                    #name
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Token { #(#variant_tokens),* }
            }
        });
        let e = find_enum(&m, "Token").unwrap();
        prop_assert_eq!(e.variants.len(), count);
        for v in &e.variants {
            prop_assert!(variant_is_unit(v));
        }
    }
}

// ── 2. Unit variants each carry a leaf attribute ────────────────────────────

proptest! {
    #[test]
    fn unit_variants_carry_leaf_attr(count in 2usize..=6) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("K{i}"), proc_macro2::Span::call_site());
                let text = format!("k{i}");
                quote! {
                    #[adze::leaf(text = #text)]
                    #name
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Keyword { #(#variant_tokens),* }
            }
        });
        let e = find_enum(&m, "Keyword").unwrap();
        for v in &e.variants {
            prop_assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 3. Tuple variant field count preserved ──────────────────────────────────

proptest! {
    #[test]
    fn tuple_variant_field_count(field_count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|_| quote! { i32 })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Lit(#(#fields),*)
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        prop_assert!(variant_is_unnamed(&e.variants[0]));
        if let Fields::Unnamed(u) = &e.variants[0].fields {
            prop_assert_eq!(u.unnamed.len(), field_count);
        }
    }
}

// ── 4. Tuple variant field types preserved ──────────────────────────────────

proptest! {
    #[test]
    fn tuple_variant_field_types(n_box in 0usize..=2, n_leaf in 1usize..=3) {
        let mut fields = Vec::new();
        for _ in 0..n_box {
            fields.push(quote! { Box<Expr> });
        }
        for _ in 0..n_leaf {
            fields.push(quote! { #[adze::leaf(pattern = r"\d+")] String });
        }
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Combo(#(#fields),*)
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let types = field_type_strings(&e.variants[0]);
        prop_assert_eq!(types.len(), n_box + n_leaf);
        for i in 0..n_box {
            prop_assert_eq!(&types[i], "Box < Expr >");
        }
        for i in 0..n_leaf {
            prop_assert_eq!(&types[n_box + i], "String");
        }
    }
}

// ── 5. Struct variant field names preserved ─────────────────────────────────

proptest! {
    #[test]
    fn struct_variant_field_names(field_count in 1usize..=5) {
        let expected: Vec<String> = (0..field_count).map(|i| format!("f{i}")).collect();
        let fields: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! { #ident: i32 }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Record { #(#fields),* }
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        prop_assert!(variant_is_named(&e.variants[0]));
        if let Fields::Named(n) = &e.variants[0].fields {
            let names: Vec<String> = n.named.iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect();
            prop_assert_eq!(names, expected);
        }
    }
}

// ── 6. Struct variant field types preserved ─────────────────────────────────

proptest! {
    #[test]
    fn struct_variant_field_types(count in 1usize..=4) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
            if i % 2 == 0 {
                quote! { #ident: String }
            } else {
                quote! { #ident: Box<Expr> }
            }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Node { #(#fields),* }
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let types = field_type_strings(&e.variants[0]);
        prop_assert_eq!(types.len(), count);
        for i in 0..count {
            if i % 2 == 0 {
                prop_assert_eq!(&types[i], "String");
            } else {
                prop_assert_eq!(&types[i], "Box < Expr >");
            }
        }
    }
}

// ── 7. Mixed variant kinds: all three present ───────────────────────────────

proptest! {
    #[test]
    fn mixed_variant_kinds(n_unit in 1usize..=3, n_tuple in 1usize..=3, n_named in 1usize..=3) {
        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_unit {
            let name = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            let text = format!("u{i}");
            tokens.push(quote! { #[adze::leaf(text = #text)] #name });
        }
        for i in 0..n_tuple {
            let name = syn::Ident::new(&format!("T{i}"), proc_macro2::Span::call_site());
            tokens.push(quote! { #name(#[adze::leaf(pattern = r"\d+")] i32) });
        }
        for i in 0..n_named {
            let name = syn::Ident::new(&format!("N{i}"), proc_macro2::Span::call_site());
            tokens.push(quote! { #name { #[adze::leaf(pattern = r"\w+")] val: String } });
        }
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr { #(#tokens),* }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let total = n_unit + n_tuple + n_named;
        prop_assert_eq!(e.variants.len(), total);
        for i in 0..n_unit {
            prop_assert!(variant_is_unit(&e.variants[i]));
        }
        for i in 0..n_tuple {
            prop_assert!(variant_is_unnamed(&e.variants[n_unit + i]));
        }
        for i in 0..n_named {
            prop_assert!(variant_is_named(&e.variants[n_unit + n_tuple + i]));
        }
    }
}

// ── 8. Enum variants form implicit CHOICE: all names appear ─────────────────

proptest! {
    #[test]
    fn enum_variants_form_choice(count in 2usize..=7) {
        let expected: Vec<String> = (0..count).map(|i| format!("Alt{i}")).collect();
        let variant_tokens: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! { #ident(#[adze::leaf(pattern = r"\d+")] i32) }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr { #(#variant_tokens),* }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let names = variant_names(e);
        // Each variant is one arm of the CHOICE; all expected names must appear
        prop_assert_eq!(names, expected);
    }
}

// ── 9. CHOICE ordering matches source declaration order ─────────────────────

proptest! {
    #[test]
    fn choice_ordering_matches_declaration(count in 2usize..=6) {
        let expected: Vec<String> = (0..count).map(|i| format!("Opt{i}")).collect();
        let variant_tokens: Vec<proc_macro2::TokenStream> = expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            let text = name.to_lowercase();
            quote! { #[adze::leaf(text = #text)] #ident }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Op { #(#variant_tokens),* }
            }
        });
        let e = find_enum(&m, "Op").unwrap();
        let actual: Vec<String> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        prop_assert_eq!(actual, expected);
    }
}

// ── 10. Variant naming follows EnumName_VariantName convention ──────────────

proptest! {
    #[test]
    fn variant_naming_convention(n_variants in 1usize..=5) {
        let variant_names_expected: Vec<String> = (0..n_variants)
            .map(|i| format!("Var{i}"))
            .collect();
        let variant_tokens: Vec<proc_macro2::TokenStream> = variant_names_expected.iter().map(|name| {
            let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
            quote! { #ident(#[adze::leaf(pattern = r"\w+")] String) }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum MyEnum { #(#variant_tokens),* }
            }
        });
        let e = find_enum(&m, "MyEnum").unwrap();
        for v in &e.variants {
            let sym = expected_symbol("MyEnum", &v.ident.to_string());
            // The expansion code generates symbols like "MyEnum_Var0"
            prop_assert!(sym.starts_with("MyEnum_"));
            prop_assert!(sym.contains(&v.ident.to_string()));
        }
    }
}

// ── 11. prec_left attribute on variant preserved ────────────────────────────

proptest! {
    #[test]
    fn prec_left_on_variant(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] i32),
                    #[adze::prec_left(#lit)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let add = e.variants.iter().find(|v| v.ident == "Add").unwrap();
        prop_assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    }
}

// ── 12. prec_right attribute on variant preserved ───────────────────────────

proptest! {
    #[test]
    fn prec_right_on_variant(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] i32),
                    #[adze::prec_right(#lit)]
                    Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let cons = e.variants.iter().find(|v| v.ident == "Cons").unwrap();
        prop_assert!(cons.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
    }
}

// ── 13. prec (no assoc) attribute on variant preserved ──────────────────────

proptest! {
    #[test]
    fn prec_no_assoc_on_variant(prec in 1i32..=15) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] i32),
                    #[adze::prec(#lit)]
                    Cmp(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let cmp = e.variants.iter().find(|v| v.ident == "Cmp").unwrap();
        prop_assert!(cmp.attrs.iter().any(|a| is_adze_attr(a, "prec")));
    }
}

// ── 14. Multiple prec variants coexist ──────────────────────────────────────

proptest! {
    #[test]
    fn multiple_prec_variants(p1 in 1i32..=5, p2 in 6i32..=10) {
        let lit1 = proc_macro2::Literal::i32_unsuffixed(p1);
        let lit2 = proc_macro2::Literal::i32_unsuffixed(p2);
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] i32),
                    #[adze::prec_left(#lit1)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                    #[adze::prec_left(#lit2)]
                    Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let add = e.variants.iter().find(|v| v.ident == "Add").unwrap();
        let mul = e.variants.iter().find(|v| v.ident == "Mul").unwrap();
        prop_assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
        prop_assert!(mul.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    }
}

// ── 15. Leaf text value round-trips through attribute ────────────────────────

proptest! {
    #[test]
    fn leaf_text_roundtrip_in_variant(idx in 0usize..=4) {
        let keywords = ["+", "-", "==", "!=", ">="];
        let kw = keywords[idx];
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Op {
                    #[adze::leaf(text = #kw)]
                    V
                }
            }
        });
        let e = find_enum(&m, "Op").unwrap();
        let attr = e.variants[0].attrs.iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> =
            attr.parse_args_with(syn::punctuated::Punctuated::parse_terminated).unwrap();
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &params[0].expr {
            prop_assert_eq!(s.value(), kw);
        } else {
            prop_assert!(false, "Expected string literal in leaf text");
        }
    }
}

// ── 16. Leaf pattern value round-trips on tuple field ────────────────────────

proptest! {
    #[test]
    fn leaf_pattern_roundtrip_on_field(idx in 0usize..=3) {
        let patterns = [r"\d+", r"[a-z]+", r"\w+", r"\s+"];
        let pat = patterns[idx];
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Lit(#[adze::leaf(pattern = #pat)] String)
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let attr = u.unnamed[0].attrs.iter()
                .find(|a| is_adze_attr(a, "leaf"))
                .unwrap();
            let params: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> =
                attr.parse_args_with(syn::punctuated::Punctuated::parse_terminated).unwrap();
            prop_assert_eq!(params[0].path.to_string(), "pattern");
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 17. language attribute on enum is preserved ─────────────────────────────

proptest! {
    #[test]
    fn language_attr_on_enum(count in 1usize..=5) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site()))
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Root { #(#names(#[adze::leaf(pattern = r"\w+")] String)),* }
            }
        });
        let e = find_enum(&m, "Root").unwrap();
        prop_assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
    }
}

// ── 18. Enum with Box<Self> recursive field ─────────────────────────────────

proptest! {
    #[test]
    fn recursive_box_self_field(depth_variants in 1usize..=3) {
        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        tokens.push(quote! { Leaf(#[adze::leaf(pattern = r"\d+")] i32) });
        for i in 0..depth_variants {
            let name = syn::Ident::new(&format!("Rec{i}"), proc_macro2::Span::call_site());
            let text = format!("r{i}");
            tokens.push(quote! { #name(#[adze::leaf(text = #text)] (), Box<Expr>) });
        }
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr { #(#tokens),* }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        prop_assert_eq!(e.variants.len(), 1 + depth_variants);
        for i in 0..depth_variants {
            let v = &e.variants[1 + i];
            let types = field_type_strings(v);
            prop_assert!(types.iter().any(|t| t.contains("Box")));
        }
    }
}

// ── 19. Enum with Vec field in tuple variant ────────────────────────────────

proptest! {
    #[test]
    fn vec_field_in_tuple_variant(count in 1usize..=3) {
        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..count {
            let name = syn::Ident::new(&format!("List{i}"), proc_macro2::Span::call_site());
            tokens.push(quote! { #name(Vec<i32>) });
        }
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr { #(#tokens),* }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        for v in &e.variants {
            let types = field_type_strings(v);
            prop_assert!(types[0].contains("Vec"));
        }
    }
}

// ── 20. Enum with Option field in struct variant ────────────────────────────

proptest! {
    #[test]
    fn option_field_in_struct_variant(count in 1usize..=3) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count).map(|i| {
            let ident = syn::Ident::new(&format!("opt{i}"), proc_macro2::Span::call_site());
            quote! { #ident: Option<i32> }
        }).collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Maybe { #(#fields),* }
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let types = field_type_strings(&e.variants[0]);
        for t in &types {
            prop_assert!(t.contains("Option"));
        }
    }
}

// ── 21. Variant with both prec and leaf attrs on different parts ────────────

proptest! {
    #[test]
    fn prec_on_variant_leaf_on_field(prec in 1i32..=10) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(#[adze::leaf(pattern = r"\d+")] i32),
                    #[adze::prec_left(#lit)]
                    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let add = e.variants.iter().find(|v| v.ident == "Add").unwrap();
        // prec_left is on the variant
        prop_assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
        // leaf is on the field, not the variant
        prop_assert!(!add.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        if let Fields::Unnamed(ref u) = add.fields {
            prop_assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 22. Unannotated variants have no adze attrs ────────────────────────────

proptest! {
    #[test]
    fn unannotated_variants_clean(count in 1usize..=5) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site()))
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr { #(#names(i32)),* }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        for v in &e.variants {
            prop_assert!(adze_attr_names(&v.attrs).is_empty());
        }
    }
}

// ── 23. Annotated and unannotated variants coexist ──────────────────────────

proptest! {
    #[test]
    fn annotated_and_plain_coexist(n_annotated in 1usize..=3, n_plain in 1usize..=3) {
        let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
        for i in 0..n_annotated {
            let name = syn::Ident::new(&format!("A{i}"), proc_macro2::Span::call_site());
            let text = format!("a{i}");
            tokens.push(quote! { #[adze::leaf(text = #text)] #name });
        }
        for i in 0..n_plain {
            let name = syn::Ident::new(&format!("P{i}"), proc_macro2::Span::call_site());
            tokens.push(quote! { #name(#[adze::leaf(pattern = r"\d+")] i32) });
        }
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr { #(#tokens),* }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        prop_assert_eq!(e.variants.len(), n_annotated + n_plain);
        for i in 0..n_annotated {
            prop_assert!(!adze_attr_names(&e.variants[i].attrs).is_empty());
        }
        for i in 0..n_plain {
            prop_assert!(adze_attr_names(&e.variants[n_annotated + i].attrs).is_empty());
        }
    }
}

// ── 24. Precedence integer value round-trips ────────────────────────────────

proptest! {
    #[test]
    fn prec_value_roundtrips(prec in 0i32..=20) {
        let lit = proc_macro2::Literal::i32_unsuffixed(prec);
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    #[adze::prec_left(#lit)]
                    Op(Box<Expr>, Box<Expr>)
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        let attr = e.variants[0].attrs.iter()
            .find(|a| is_adze_attr(a, "prec_left"))
            .unwrap();
        let expr: syn::Expr = attr.parse_args().unwrap();
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), .. }) = expr {
            prop_assert_eq!(i.base10_parse::<i32>().unwrap(), prec);
        } else {
            prop_assert!(false, "Expected int literal for prec value");
        }
    }
}

// ── 25. Expansion determinism: same input yields same token stream ──────────

proptest! {
    #[test]
    fn expansion_determinism(count in 2usize..=5) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("v{i}");
                quote! { #[adze::leaf(text = #text)] #name }
            })
            .collect();
        let build = || -> String {
            let tokens = variant_tokens.clone();
            let m: ItemMod = syn::parse2(quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub enum Tok { #(#tokens),* }
                }
            }).unwrap();
            m.to_token_stream().to_string()
        };
        let first = build();
        let second = build();
        prop_assert_eq!(first, second);
    }
}

// ── 26. Determinism with mixed variant types ────────────────────────────────

proptest! {
    #[test]
    fn determinism_mixed_variants(n_unit in 1usize..=2, n_tuple in 1usize..=2) {
        let build = || -> String {
            let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();
            for i in 0..n_unit {
                let name = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
                let text = format!("u{i}");
                tokens.push(quote! { #[adze::leaf(text = #text)] #name });
            }
            for i in 0..n_tuple {
                let name = syn::Ident::new(&format!("T{i}"), proc_macro2::Span::call_site());
                tokens.push(quote! { #name(#[adze::leaf(pattern = r"\d+")] i32) });
            }
            let m: ItemMod = syn::parse2(quote! {
                #[adze::grammar("test")]
                mod grammar {
                    #[adze::language]
                    pub enum Expr { #(#tokens),* }
                }
            }).unwrap();
            m.to_token_stream().to_string()
        };
        prop_assert_eq!(build(), build());
    }
}

// ── 27. Enum ident preserved across grammar module ──────────────────────────

proptest! {
    #[test]
    fn enum_ident_preserved(idx in 0usize..=4) {
        let names = ["Expr", "Token", "Stmt", "Decl", "Pattern"];
        let name = syn::Ident::new(names[idx], proc_macro2::Span::call_site());
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum #name {
                    A(#[adze::leaf(pattern = r"\d+")] i32),
                }
            }
        });
        let e = find_enum(&m, names[idx]).unwrap();
        prop_assert_eq!(e.ident.to_string(), names[idx]);
    }
}

// ── 28. Enum with no adze attrs on the enum itself (only on variants) ───────

proptest! {
    #[test]
    fn no_adze_attr_on_enum_without_language(count in 1usize..=4) {
        let names: Vec<syn::Ident> = (0..count)
            .map(|i| syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site()))
            .collect();
        // An enum without #[adze::language] inside a grammar module
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    val: String,
                }
                pub enum Helper { #(#names(i32)),* }
            }
        });
        // Helper enum should have no adze attrs on it
        let e = find_enum(&m, "Helper").unwrap();
        prop_assert!(adze_attr_names(&e.attrs).is_empty());
    }
}

// ── 29. Variant with leaf+transform attr on field ───────────────────────────

proptest! {
    #[test]
    fn leaf_transform_on_field(idx in 0usize..=2) {
        let patterns = [r"\d+", r"[0-9]+", r"\d{1,10}"];
        let pat = patterns[idx];
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    Num(
                        #[adze::leaf(pattern = #pat, transform = |v| v.parse::<i32>().unwrap())]
                        i32
                    ),
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        if let Fields::Unnamed(ref u) = e.variants[0].fields {
            let attr = u.unnamed[0].attrs.iter()
                .find(|a| is_adze_attr(a, "leaf"))
                .unwrap();
            let params: syn::punctuated::Punctuated<adze_common::NameValueExpr, syn::Token![,]> =
                attr.parse_args_with(syn::punctuated::Punctuated::parse_terminated).unwrap();
            let param_names: Vec<String> = params.iter().map(|p| p.path.to_string()).collect();
            prop_assert!(param_names.contains(&"pattern".to_string()));
            prop_assert!(param_names.contains(&"transform".to_string()));
        } else {
            prop_assert!(false, "Expected unnamed fields");
        }
    }
}

// ── 30. Enum visibility preserved ───────────────────────────────────────────

#[test]
fn enum_pub_visibility_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                A(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    assert!(matches!(e.vis, syn::Visibility::Public(_)));
}

// ── 31. Enum with repeat attr on Vec field ──────────────────────────────────

#[test]
fn enum_repeat_attr_on_vec_field() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: u32,
            }
            #[adze::language]
            pub enum Expr {
                Numbers(
                    #[adze::repeat(non_empty = true)]
                    Vec<Number>
                ),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert!(u.unnamed[0].attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 32. Enum with named variant containing leaf on struct field ──────────────

#[test]
fn named_variant_leaf_on_struct_field() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Neg {
                    #[adze::leaf(text = "!")]
                    _bang: (),
                    value: Box<Expr>,
                },
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    assert!(variant_is_named(&e.variants[0]));
    if let Fields::Named(ref n) = e.variants[0].fields {
        assert!(n.named[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        assert!(n.named[1].attrs.iter().all(|a| !is_adze_attr(a, "leaf")));
    }
}

// ── 33. Expected symbol string computed correctly ───────────────────────────

proptest! {
    #[test]
    fn expected_symbol_format(idx in 0usize..=3) {
        let enum_names = ["Expr", "Token", "Stmt", "Op"];
        let variant_names_arr = ["Add", "Literal", "Return", "Plus"];
        let sym = expected_symbol(enum_names[idx], variant_names_arr[idx]);
        prop_assert_eq!(sym, format!("{}_{}", enum_names[idx], variant_names_arr[idx]));
    }
}

// ── 34. Enum variants have no discriminants ─────────────────────────────────

proptest! {
    #[test]
    fn enum_variants_no_discriminant(count in 1usize..=6) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                quote! { #name(#[adze::leaf(pattern = r"\d+")] i32) }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr { #(#variant_tokens),* }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        for v in &e.variants {
            prop_assert!(v.discriminant.is_none());
        }
    }
}

// ── 35. Grammar module still contains the enum after parse ──────────────────

proptest! {
    #[test]
    fn enum_present_in_grammar_module(count in 1usize..=4) {
        let variant_tokens: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("v{i}");
                quote! { #[adze::leaf(text = #text)] #name }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum MyLang { #(#variant_tokens),* }
            }
        });
        // The enum must be found inside the module
        prop_assert!(find_enum(&m, "MyLang").is_some());
        let e = find_enum(&m, "MyLang").unwrap();
        prop_assert_eq!(e.variants.len(), count);
    }
}
