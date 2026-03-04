#![allow(clippy::needless_range_loop)]

//! Property-based and deterministic tests for precedence attribute handling
//! in adze-macro.
//!
//! Covers `#[adze::prec(N)]`, `#[adze::prec_left(N)]`, `#[adze::prec_right(N)]`
//! attribute parsing via the public API (syn / quote / proc_macro2).
//! Tests integer values, enum variant placement, multi-level grammars,
//! attribute coexistence, validation of invalid inputs, and expansion
//! determinism.

use proc_macro2::TokenStream;
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

fn prec_value(attr: &Attribute) -> i32 {
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        i.base10_parse::<i32>().unwrap()
    } else {
        panic!("Expected int literal in precedence attribute");
    }
}

fn extract_prec_info(e: &ItemEnum) -> Vec<(String, String, i32)> {
    e.variants
        .iter()
        .filter_map(|v| {
            for kind in &["prec", "prec_left", "prec_right"] {
                if let Some(attr) = v.attrs.iter().find(|a| is_adze_attr(a, kind)) {
                    return Some((v.ident.to_string(), kind.to_string(), prec_value(attr)));
                }
            }
            None
        })
        .collect()
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

// ── 1. prec with integer value: small positive ──────────────────────────────

#[test]
fn prec_int_small_positive() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(3)]
            A(i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    assert_eq!(prec_value(attr), 3);
}

// ── 2. prec with integer value: boundary zero ───────────────────────────────

#[test]
fn prec_int_zero_boundary() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(0)]
            Z(i32),
        }
    };
    assert_eq!(extract_prec_info(&e)[0].2, 0);
}

// ── 3. prec with large integer via proptest ─────────────────────────────────

proptest! {
    #[test]
    fn prec_int_large_range(level in 500i32..=9999) {
        let tokens: TokenStream = format!(
            "pub enum E {{ #[adze::prec({level})] V(i32), }}"
        ).parse().unwrap();
        let e: ItemEnum = syn::parse2(tokens).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info[0].2, level);
    }
}

// ── 4. prec_left basic parse ────────────────────────────────────────────────

#[test]
fn prec_left_attr_basic_parse() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Lit(i32),
            #[adze::prec_left(4)]
            Add(Box<E>, Box<E>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info[0], ("Add".into(), "prec_left".into(), 4));
}

// ── 5. prec_right basic parse ───────────────────────────────────────────────

#[test]
fn prec_right_attr_basic_parse() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Lit(i32),
            #[adze::prec_right(6)]
            Exp(Box<E>, Box<E>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info[0], ("Exp".into(), "prec_right".into(), 6));
}

// ── 6. prec on enum variant: unnamed tuple ──────────────────────────────────

#[test]
fn prec_on_unnamed_tuple_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(10)]
            Pair(i32, String),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 1);
    assert!(matches!(e.variants[0].fields, Fields::Unnamed(_)));
}

// ── 7. prec on enum variant: named fields ───────────────────────────────────

#[test]
fn prec_on_named_fields_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec_left(2)]
            Op { lhs: i32, rhs: i32 },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let names: Vec<_> = n
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["lhs", "rhs"]);
    } else {
        panic!("Expected named fields");
    }
    assert_eq!(extract_prec_info(&e)[0].1, "prec_left");
}

// ── 8. prec on enum variant: unit variant ───────────────────────────────────

#[test]
fn prec_on_unit_variant_roundtrip() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::prec(1)]
            #[adze::leaf(text = "&&")]
            And,
            #[adze::prec(2)]
            #[adze::leaf(text = "||")]
            Or,
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 2);
    assert!(e.variants.iter().all(|v| matches!(v.fields, Fields::Unit)));
}

// ── 9. Multiple prec levels: ascending order preserved ──────────────────────

#[test]
fn multi_level_ascending_preserved() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Lit(i32),
            #[adze::prec_left(1)]
            Low(Box<E>, Box<E>),
            #[adze::prec_left(5)]
            Med(Box<E>, Box<E>),
            #[adze::prec_left(10)]
            Hi(Box<E>, Box<E>),
        }
    };
    let levels: Vec<i32> = extract_prec_info(&e).iter().map(|i| i.2).collect();
    assert_eq!(levels, vec![1, 5, 10]);
    // Strictly ascending
    for i in 0..levels.len() - 1 {
        assert!(levels[i] < levels[i + 1]);
    }
}

// ── 10. Multiple prec levels: all three kinds in one enum ───────────────────

#[test]
fn multi_level_three_kinds_one_enum() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Lit(i32),
            #[adze::prec(1)]
            Cmp(Box<E>, Box<E>),
            #[adze::prec_left(2)]
            Add(Box<E>, Box<E>),
            #[adze::prec_left(3)]
            Mul(Box<E>, Box<E>),
            #[adze::prec_right(4)]
            Pow(Box<E>, Box<E>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 4);
    let kinds: Vec<&str> = info.iter().map(|i| i.1.as_str()).collect();
    assert_eq!(kinds, vec!["prec", "prec_left", "prec_left", "prec_right"]);
}

// ── 11. Multiple prec levels via proptest: N variants with sequential levels ─

proptest! {
    #[test]
    fn multi_level_sequential_proptest(n in 2usize..=6) {
        let mut variant_strs = vec!["Lit(i32)".to_string()];
        for i in 0..n {
            variant_strs.push(format!(
                "#[adze::prec_left({i})] V{i}(Box<E>, Box<E>)"
            ));
        }
        let src = format!("pub enum E {{ {} }}", variant_strs.join(", "));
        let e: ItemEnum = syn::parse2(src.parse().unwrap()).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), n);
        for i in 0..n {
            prop_assert_eq!(info[i].2, i as i32);
        }
    }
}

// ── 12. Prec with leaf attr coexistence on same variant ─────────────────────

#[test]
fn prec_with_leaf_coexistence() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec_left(1)]
            Add(
                Box<E>,
                #[adze::leaf(text = "+")]
                (),
                Box<E>,
            ),
        }
    };
    // Variant-level prec
    assert!(
        e.variants[0]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_left"))
    );
    // Field-level leaf on the operator field
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 13. Prec with skip attr on sibling field ────────────────────────────────

#[test]
fn prec_with_skip_on_sibling() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec_left(2)]
            Tagged {
                lhs: Box<E>,
                #[adze::leaf(text = "+")]
                _op: (),
                rhs: Box<E>,
                #[adze::skip(false)]
                meta: bool,
            },
        }
    };
    let variant_attrs = adze_attr_names(&e.variants[0].attrs);
    assert_eq!(variant_attrs, vec!["prec_left"]);
    if let Fields::Named(ref n) = e.variants[0].fields {
        let field_attrs: Vec<_> = n
            .named
            .iter()
            .flat_map(|f| adze_attr_names(&f.attrs))
            .collect();
        assert!(field_attrs.contains(&"leaf".to_string()));
        assert!(field_attrs.contains(&"skip".to_string()));
    }
}

// ── 14. Prec + repeat on sibling field ──────────────────────────────────────

#[test]
fn prec_with_repeat_on_sibling() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Lit(i32),
            #[adze::prec_left(1)]
            Call(
                Box<E>,
                #[adze::leaf(text = "(")]
                (),
                #[adze::repeat(non_empty = false)]
                Vec<E>,
                #[adze::leaf(text = ")")]
                (),
            ),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info[0].0, "Call");
    if let Fields::Unnamed(ref u) = e.variants[1].fields {
        assert!(u.unnamed[2].attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    }
}

// ── 15. Invalid prec value: negative integer parses as syn expression ───────

#[test]
fn prec_negative_value_parses_as_unary() {
    // A negative literal parses but as a unary-minus expression, not an int literal.
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(-1)]
            V(i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    // It's a unary expression, not a bare int literal
    assert!(matches!(expr, syn::Expr::Unary(_)));
}

// ── 16. Invalid prec value: string literal instead of int ───────────────────

#[test]
fn prec_string_value_wrong_type() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec("high")]
            V(i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    // Should be a Lit but with a Str, not Int
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        }) => {}
        other => panic!("Expected string literal, got: {other:?}"),
    }
}

// ── 17. Invalid: prec with no arguments fails parse_args ────────────────────

#[test]
fn prec_no_args_fails_parse() {
    // #[adze::prec] (no parens) — the attribute has no delimiter list
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec]
            V(i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let result = attr.parse_args::<syn::Expr>();
    assert!(
        result.is_err(),
        "parse_args should fail when no argument list"
    );
}

// ── 18. Invalid: prec with float literal ────────────────────────────────────

#[test]
fn prec_float_value_wrong_type() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(1.5)]
            V(i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Float(_),
            ..
        }) => {}
        other => panic!("Expected float literal, got: {other:?}"),
    }
}

// ── 19. Invalid: prec with multiple arguments ───────────────────────────────

#[test]
fn prec_multiple_args_parses_differently() {
    // #[adze::prec(1, 2)] — syn treats "1, 2" as a sequence; parse_args::<Expr>
    // would fail because there's a trailing comma.
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(1, 2)]
            V(i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    // parse_args expects a single expression; "1, 2" is not one expression
    let result = attr.parse_args::<syn::Expr>();
    assert!(result.is_err(), "parse_args should fail with multiple args");
}

// ── 20. Expansion determinism: same input yields same token stream ──────────

#[test]
fn prec_expansion_deterministic() {
    let make_enum = || -> ItemEnum {
        parse_quote! {
            pub enum E {
                Lit(i32),
                #[adze::prec_left(1)]
                Add(Box<E>, Box<E>),
                #[adze::prec_right(2)]
                Exp(Box<E>, Box<E>),
                #[adze::prec(3)]
                Cmp(Box<E>, Box<E>),
            }
        }
    };
    let a = make_enum().to_token_stream().to_string();
    let b = make_enum().to_token_stream().to_string();
    assert_eq!(
        a, b,
        "Two identical parses must produce identical token streams"
    );
}

// ── 21. Determinism: attribute order within variant is preserved ─────────────

#[test]
fn prec_attr_order_deterministic() {
    let e1: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(5)]
            #[adze::leaf(text = "x")]
            X,
        }
    };
    let e2: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(5)]
            #[adze::leaf(text = "x")]
            X,
        }
    };
    let a1 = adze_attr_names(&e1.variants[0].attrs);
    let a2 = adze_attr_names(&e2.variants[0].attrs);
    assert_eq!(a1, a2);
    assert_eq!(a1, vec!["prec", "leaf"]);
}

// ── 22. Determinism via proptest: random level yields stable output ──────────

proptest! {
    #[test]
    fn prec_determinism_proptest(level in 0i32..=200) {
        let src = format!(
            "pub enum E {{ Lit(i32), #[adze::prec_left({level})] V(Box<E>, Box<E>), }}"
        );
        let e1: ItemEnum = syn::parse2(src.parse().unwrap()).unwrap();
        let e2: ItemEnum = syn::parse2(src.parse().unwrap()).unwrap();
        prop_assert_eq!(
            e1.to_token_stream().to_string(),
            e2.to_token_stream().to_string()
        );
    }
}

// ── 23. Prec in grammar module with extra struct ────────────────────────────

#[test]
fn prec_in_grammar_module_with_extra() {
    let m = parse_mod(quote! {
        #[adze::grammar("test_prec_extra")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = module_items(&m);
    assert!(items.iter().any(|i| matches!(i, Item::Enum(_))));
    assert!(items.iter().any(|i| matches!(i, Item::Struct(_))));
}

// ── 24. Prec in grammar module: variant positions stable ────────────────────

#[test]
fn prec_grammar_module_variant_positions() {
    let m = parse_mod(quote! {
        #[adze::grammar("pos")]
        mod grammar {
            #[adze::language]
            pub enum E {
                A(#[adze::leaf(pattern = r"\w+")] String),
                #[adze::prec_left(1)]
                B(Box<E>, #[adze::leaf(text = "+")] (), Box<E>),
                #[adze::prec_right(2)]
                C(Box<E>, #[adze::leaf(text = "^")] (), Box<E>),
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert_eq!(e.variants[0].ident, "A");
        assert_eq!(e.variants[1].ident, "B");
        assert_eq!(e.variants[2].ident, "C");
    } else {
        panic!("Expected enum");
    }
}

// ── 25. Prec attr path segments are exactly ["adze", kind] ──────────────────

#[test]
fn prec_attr_path_structure() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(1)]
            A(i32),
            #[adze::prec_left(2)]
            B(i32),
            #[adze::prec_right(3)]
            C(i32),
        }
    };
    for (i, expected) in ["prec", "prec_left", "prec_right"].iter().enumerate() {
        let attr = &e.variants[i].attrs[0];
        let segs: Vec<_> = attr
            .path()
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        assert_eq!(segs, vec!["adze", *expected]);
    }
}

// ── 26. Prec_left with identical level on >2 variants ───────────────────────

#[test]
fn prec_left_same_level_three_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Lit(i32),
            #[adze::prec_left(1)]
            Add(Box<E>, Box<E>),
            #[adze::prec_left(1)]
            Sub(Box<E>, Box<E>),
            #[adze::prec_left(1)]
            Or(Box<E>, Box<E>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 3);
    assert!(info.iter().all(|i| i.2 == 1 && i.1 == "prec_left"));
}

// ── 27. Prec does not introduce discriminant values ─────────────────────────

#[test]
fn prec_no_discriminant() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec(1)]
            A(i32),
            #[adze::prec_left(2)]
            B(i32),
        }
    };
    for v in &e.variants {
        assert!(
            v.discriminant.is_none(),
            "prec should not add discriminant to {}",
            v.ident
        );
    }
}

// ── 28. Prec_right on deeply nested Box type ────────────────────────────────

#[test]
fn prec_right_nested_box_types() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Lit(i32),
            #[adze::prec_right(5)]
            Deep(Box<E>, Box<E>, Box<E>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[1].fields {
        assert_eq!(u.unnamed.len(), 3);
        for f in &u.unnamed {
            assert!(f.ty.to_token_stream().to_string().contains("Box"));
        }
    }
    assert_eq!(extract_prec_info(&e)[0].2, 5);
}

// ── 29. Prec attrs do not leak across variants ──────────────────────────────

#[test]
fn prec_does_not_leak_across_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Plain(i32),
            #[adze::prec_left(7)]
            Tagged(i32, i32),
            AlsoPlain(String),
        }
    };
    // Only the middle variant has prec
    assert!(
        e.variants[0]
            .attrs
            .iter()
            .all(|a| !is_adze_attr(a, "prec_left"))
    );
    assert!(
        e.variants[1]
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "prec_left"))
    );
    assert!(
        e.variants[2]
            .attrs
            .iter()
            .all(|a| !is_adze_attr(a, "prec_left"))
    );
}

// ── 30. Proptest: prec_right value extraction round-trip ────────────────────

proptest! {
    #[test]
    fn prec_right_value_roundtrip(level in 0i32..=300) {
        let src = format!(
            "pub enum E {{ #[adze::prec_right({level})] V(i32), }}"
        );
        let e: ItemEnum = syn::parse2(src.parse().unwrap()).unwrap();
        let attr = e.variants[0].attrs.iter()
            .find(|a| is_adze_attr(a, "prec_right")).unwrap();
        prop_assert_eq!(prec_value(attr), level);
    }
}

// ── 31. Grammar with four operator groups at distinct levels ────────────────

#[test]
fn four_operator_groups_distinct_levels() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            Lit(i32),
            #[adze::prec(1)]
            Eq(Box<E>, Box<E>),
            #[adze::prec_left(2)]
            Add(Box<E>, Box<E>),
            #[adze::prec_left(3)]
            Mul(Box<E>, Box<E>),
            #[adze::prec_right(4)]
            Pow(Box<E>, Box<E>),
        }
    };
    let info = extract_prec_info(&e);
    let levels: Vec<i32> = info.iter().map(|i| i.2).collect();
    assert_eq!(levels, vec![1, 2, 3, 4]);
    // All distinct
    let set: std::collections::HashSet<i32> = levels.iter().copied().collect();
    assert_eq!(set.len(), 4);
}

// ── 32. Prec attr token stream serialization is stable ──────────────────────

#[test]
fn prec_attr_serialization_stable() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec_left(42)]
            V(i32, i32),
        }
    };
    let s1 = e.variants[0].attrs[0].to_token_stream().to_string();
    let s2 = e.variants[0].attrs[0].to_token_stream().to_string();
    assert_eq!(s1, s2);
    assert!(s1.contains("prec_left"));
    assert!(s1.contains("42"));
}

// ── 33. Empty parens: prec with empty args ──────────────────────────────────

#[test]
fn prec_empty_parens_fails_parse() {
    let e: ItemEnum = parse_quote! {
        pub enum E {
            #[adze::prec()]
            V(i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let result = attr.parse_args::<syn::Expr>();
    assert!(result.is_err(), "Empty parens should fail parse_args");
}

// ── 34. Proptest: mixed kinds at random single level ────────────────────────

proptest! {
    #[test]
    fn mixed_kinds_random_level(level in 1i32..=50) {
        let src = format!(
            "pub enum E {{ \
                Lit(i32), \
                #[adze::prec({level})] Cmp(Box<E>, Box<E>), \
                #[adze::prec_left({level})] Add(Box<E>, Box<E>), \
                #[adze::prec_right({level})] Pow(Box<E>, Box<E>), \
            }}"
        );
        let e: ItemEnum = syn::parse2(src.parse().unwrap()).unwrap();
        let info = extract_prec_info(&e);
        prop_assert_eq!(info.len(), 3);
        for entry in &info {
            prop_assert_eq!(entry.2, level);
        }
        let kinds: Vec<&str> = info.iter().map(|i| i.1.as_str()).collect();
        prop_assert_eq!(kinds, vec!["prec", "prec_left", "prec_right"]);
    }
}

// ── 35. Prec variant ident preserved through to_token_stream ────────────────

#[test]
fn prec_variant_ident_in_token_stream() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Addition(Box<Expr>, Box<Expr>),
            #[adze::prec_right(2)]
            Exponentiation(Box<Expr>, Box<Expr>),
        }
    };
    let ts = e.to_token_stream().to_string();
    assert!(
        ts.contains("Addition"),
        "Token stream must contain variant name 'Addition'"
    );
    assert!(
        ts.contains("Exponentiation"),
        "Token stream must contain variant name 'Exponentiation'"
    );
    assert!(ts.contains("prec_left"));
    assert!(ts.contains("prec_right"));
}
