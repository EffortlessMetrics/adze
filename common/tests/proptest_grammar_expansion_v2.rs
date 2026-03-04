//! Property-based and unit tests for grammar expansion in adze-common (v2).
//!
//! Covers: try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! NameValueExpr, FieldThenParams — determinism, idempotency, composition,
//! round-trip, and edge-case properties.

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Lowercase identifier that is not a Rust keyword.
fn safe_ident() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,8}".prop_filter("must not be keyword", |s| {
        !matches!(
            s.as_str(),
            "as" | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
                | "abstract"
                | "become"
                | "box"
                | "do"
                | "final"
                | "macro"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
                | "yield"
                | "try"
                | "gen"
        )
    })
}

fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box", "Arc", "Rc"][..])
}

/// Wrapper<Leaf> string and its parts.
fn single_wrapped() -> impl Strategy<Value = (String, String)> {
    (container_name(), leaf_type_name()).prop_map(|(c, l)| (c.to_string(), format!("{c}<{l}>")))
}

/// Types at depth 0–3.
fn random_type_string() -> impl Strategy<Value = String> {
    prop_oneof![
        leaf_type_name().prop_map(|s| s.to_string()),
        (container_name(), leaf_type_name()).prop_map(|(c, l)| format!("{c}<{l}>")),
        (container_name(), container_name(), leaf_type_name())
            .prop_map(|(a, b, l)| format!("{a}<{b}<{l}>>")),
        (
            container_name(),
            container_name(),
            container_name(),
            leaf_type_name()
        )
            .prop_map(|(a, b, c, l)| format!("{a}<{b}<{c}<{l}>>>")),
    ]
}

/// Subset of skip-set names.
fn skip_set_vec() -> impl Strategy<Value = Vec<&'static str>> {
    prop::collection::vec(
        prop::sample::select(&["Box", "Arc", "Rc", "Cell"][..]),
        0..=4,
    )
}

// ---------------------------------------------------------------------------
// proptest! block — 30 property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // --- try_extract_inner_type ---

    /// P1: Plain ident never extracts as Option.
    #[test]
    fn plain_ident_never_option(name in safe_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_, ex) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!ex);
    }

    /// P2: Option<L> always extracts with target "Option".
    #[test]
    fn option_leaf_always_extracts(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(ex);
        prop_assert_eq!(inner.to_token_stream().to_string(), leaf);
    }

    /// P3: Vec<L> always extracts with target "Vec".
    #[test]
    fn vec_leaf_always_extracts(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, ex) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(ex);
        prop_assert_eq!(inner.to_token_stream().to_string(), leaf);
    }

    /// P4: Extraction with non-existent target always returns false.
    #[test]
    fn nonexistent_target_never_extracts(ts in random_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_, ex) = try_extract_inner_type(&ty, "ZZZNonExistent", &skip);
        prop_assert!(!ex);
    }

    /// P5: Extraction is deterministic — same input gives same output.
    #[test]
    fn extract_deterministic(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (a, ea) = try_extract_inner_type(&ty, "Option", &skip);
        let (b, eb) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert_eq!(ea, eb);
        prop_assert_eq!(a.to_token_stream().to_string(), b.to_token_stream().to_string());
    }

    /// P6: Skip-over Box reaches inner Option.
    #[test]
    fn skip_box_reaches_option(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<Option<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(ex);
        prop_assert_eq!(inner.to_token_stream().to_string(), leaf);
    }

    /// P7: Double skip through Box<Arc<Vec<L>>>.
    #[test]
    fn double_skip_reaches_vec(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<Arc<Vec<{leaf}>>>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let (inner, ex) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(ex);
        prop_assert_eq!(inner.to_token_stream().to_string(), leaf);
    }

    /// P8: When skip doesn't contain the outer wrapper, extraction fails.
    #[test]
    fn wrong_skip_does_not_extract(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Rc<Option<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (_, ex) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!ex);
    }

    /// P9: Non-match is idempotent (re-extracting returns same).
    #[test]
    fn extract_nonmatch_idempotent(name in leaf_type_name()) {
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (o1, e1) = try_extract_inner_type(&ty, "Option", &skip);
        let (o2, e2) = try_extract_inner_type(&o1, "Option", &skip);
        prop_assert!(!e1);
        prop_assert!(!e2);
        prop_assert_eq!(
            o1.to_token_stream().to_string(),
            o2.to_token_stream().to_string()
        );
    }

    /// P10: Nested Option<Option<L>> — can peel two layers.
    #[test]
    fn nested_option_double_peel(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Option<Option<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (mid, e1) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(e1);
        let (inner, e2) = try_extract_inner_type(&mid, "Option", &skip);
        prop_assert!(e2);
        prop_assert_eq!(inner.to_token_stream().to_string(), leaf);
    }

    // --- filter_inner_type ---

    /// P11: Empty skip set is identity.
    #[test]
    fn filter_empty_skip_identity(ts in random_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(
            filtered.to_token_stream().to_string(),
            ty.to_token_stream().to_string()
        );
    }

    /// P12: Filtering Box<L> with "Box" in skip gives L.
    #[test]
    fn filter_box_strips_layer(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), leaf);
    }

    /// P13: Filtering is idempotent — filter(filter(x)) == filter(x).
    #[test]
    fn filter_idempotent(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let f1 = filter_inner_type(&ty, &skip);
        let f2 = filter_inner_type(&f1, &skip);
        prop_assert_eq!(
            f1.to_token_stream().to_string(),
            f2.to_token_stream().to_string()
        );
    }

    /// P14: Filter is deterministic.
    #[test]
    fn filter_deterministic(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Arc<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Arc"].into_iter().collect();
        let a = filter_inner_type(&ty, &skip).to_token_stream().to_string();
        let b = filter_inner_type(&ty, &skip).to_token_stream().to_string();
        prop_assert_eq!(a, b);
    }

    /// P15: Filter strips nested Box<Arc<L>>.
    #[test]
    fn filter_strips_nested(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<Arc<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), leaf);
    }

    /// P16: Leaf types are unchanged by filter regardless of skip contents.
    #[test]
    fn filter_leaf_unchanged(leaf in leaf_type_name(), skip_names in skip_set_vec()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = skip_names.into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), leaf);
    }

    // --- wrap_leaf_type ---

    /// P17: Empty skip always wraps in WithLeaf.
    #[test]
    fn wrap_empty_skip_wraps(name in leaf_type_name()) {
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"));
    }

    /// P18: wrap_leaf_type is deterministic.
    #[test]
    fn wrap_deterministic(ts in random_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let a = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        let b = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert_eq!(a, b);
    }

    /// P19: Wrapped output always contains original leaf name.
    #[test]
    fn wrap_contains_leaf_name(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert!(s.contains(leaf));
    }

    /// P20: Vec<L> with Vec in skip → output starts with "Vec".
    #[test]
    fn wrap_vec_preserves_outer(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert!(s.starts_with("Vec <"));
        prop_assert!(s.contains("adze :: WithLeaf"));
    }

    /// P21: Option<Vec<L>> with both in skip wraps only the leaf.
    #[test]
    fn wrap_nested_skip_wraps_leaf(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Option<Vec<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert!(s.starts_with("Option <"));
        prop_assert!(s.contains("Vec <"));
        prop_assert!(s.contains("adze :: WithLeaf"));
    }

    /// P22: Wrapped type can be reparsed by syn.
    #[test]
    fn wrap_output_reparseable(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        let reparsed: Type = parse_str(&s).unwrap();
        prop_assert_eq!(
            reparsed.to_token_stream().to_string(),
            s
        );
    }

    // --- NameValueExpr ---

    /// P23: NameValueExpr round-trip: parse key=int, re-format, re-parse.
    #[test]
    fn nve_roundtrip_int(key in safe_ident(), val in 0i64..9999) {
        let src = format!("{key} = {val}");
        let p: NameValueExpr = syn::parse_str(&src).unwrap();
        let src2 = format!("{} = {}", p.path, p.expr.to_token_stream());
        let p2: NameValueExpr = syn::parse_str(&src2).unwrap();
        prop_assert_eq!(p.path.to_string(), p2.path.to_string());
        prop_assert_eq!(
            p.expr.to_token_stream().to_string(),
            p2.expr.to_token_stream().to_string()
        );
    }

    /// P24: NameValueExpr with string literal preserves key.
    #[test]
    fn nve_string_literal_key(key in safe_ident()) {
        let src = format!("{key} = \"hello\"");
        let p: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(p.path.to_string(), key);
    }

    // --- FieldThenParams ---

    /// P25: FieldThenParams bare leaf has no params.
    #[test]
    fn ftp_bare_no_params(leaf in leaf_type_name()) {
        let ftp: FieldThenParams = syn::parse_str(leaf).unwrap();
        prop_assert!(ftp.comma.is_none());
        prop_assert!(ftp.params.is_empty());
    }

    // --- composition ---

    /// P26: extract ∘ wrap: extract Option from Option<L>, wrap result, gives WithLeaf<L>.
    #[test]
    fn extract_then_wrap(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, _) = try_extract_inner_type(&ty, "Option", &skip);
        let s = wrap_leaf_type(&inner, &skip).to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"));
        prop_assert!(s.contains(leaf));
    }

    /// P27: filter then wrap: filter Box<L>, wrap result.
    #[test]
    fn filter_then_wrap(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &filter_skip);
        let wrap_skip: HashSet<&str> = HashSet::new();
        let s = wrap_leaf_type(&filtered, &wrap_skip).to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"));
        prop_assert!(s.contains(leaf));
    }

    /// P28: wrap on Container<L> NOT in skip wraps the entire thing.
    #[test]
    fn wrap_non_skip_container_wraps_all(
        container in container_name(),
        leaf in leaf_type_name()
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        // skip set does NOT contain the container
        let skip: HashSet<&str> = HashSet::new();
        let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert!(s.starts_with("adze :: WithLeaf <"));
    }

    /// P29: filter ∘ filter == filter (double application is idempotent) on nested.
    #[test]
    fn filter_double_application_nested(leaf in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<Arc<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let f1 = filter_inner_type(&ty, &skip);
        let f2 = filter_inner_type(&f1, &skip);
        prop_assert_eq!(
            f1.to_token_stream().to_string(),
            f2.to_token_stream().to_string()
        );
    }

    /// P30: Random type string never crashes any function.
    #[test]
    fn random_type_no_crash(ts in random_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
        let _ = filter_inner_type(&ty, &skip);
        let _ = try_extract_inner_type(&ty, "Option", &skip);
        // wrap may panic on non-path containers without angle brackets,
        // but our strategies only produce valid types that are safe.
    }
}

// ---------------------------------------------------------------------------
// Unit tests — 25 additional
// ---------------------------------------------------------------------------

// --- try_extract_inner_type ---

/// U1: Reference type is not extractable.
#[test]
fn u01_extract_ref_type_unchanged() {
    let ty: Type = parse_str("&str").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ex);
    assert_eq!(inner.to_token_stream().to_string(), "& str");
}

/// U2: Tuple type is not extractable.
#[test]
fn u02_extract_tuple_unchanged() {
    let ty: Type = parse_str("(i32, u32)").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let (_, ex) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ex);
}

/// U3: Extract Option from Option<Vec<u8>> yields Vec<u8>.
#[test]
fn u03_extract_option_of_vec() {
    let ty: Type = parse_str("Option<Vec<u8>>").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ex);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < u8 >");
}

/// U4: Skip Box then skip Arc to reach Vec.
#[test]
fn u04_double_skip_box_arc_vec() {
    let ty: Type = parse_str("Box<Arc<Vec<String>>>").unwrap();
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let (inner, ex) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ex);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

/// U5: Skip container that doesn't contain target returns original.
#[test]
fn u05_skip_without_target_returns_original() {
    let ty: Type = parse_str("Box<String>").unwrap();
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (out, ex) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ex);
    assert_eq!(out.to_token_stream().to_string(), "Box < String >");
}

// --- filter_inner_type ---

/// U6: Filter tuple type is unchanged.
#[test]
fn u06_filter_tuple_unchanged() {
    let ty: Type = parse_str("(i32, u32)").unwrap();
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "(i32 , u32)");
}

/// U7: Filter triple-nested containers.
#[test]
fn u07_filter_triple_nested() {
    let ty: Type = parse_str("Box<Arc<Rc<String>>>").unwrap();
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

/// U8: Filter non-matching container leaves it intact.
#[test]
fn u08_filter_non_matching_intact() {
    let ty: Type = parse_str("Vec<i32>").unwrap();
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Vec < i32 >");
}

/// U9: Filter Arc<f64> yields f64.
#[test]
fn u09_filter_arc_f64() {
    let ty: Type = parse_str("Arc<f64>").unwrap();
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "f64");
}

/// U10: Filter with empty skip preserves Box<String>.
#[test]
fn u10_filter_empty_skip_preserves() {
    let ty: Type = parse_str("Box<String>").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Box < String >");
}

// --- wrap_leaf_type ---

/// U11: Wrap bare String.
#[test]
fn u11_wrap_bare_string() {
    let ty: Type = parse_str("String").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
    assert_eq!(s, "adze :: WithLeaf < String >");
}

/// U12: Wrap array type wraps entirely.
#[test]
fn u12_wrap_array_type() {
    let ty: Type = parse_str("[u8; 4]").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
    assert!(s.contains("adze :: WithLeaf"));
}

/// U13: Wrap Result<A, B> with Result in skip wraps both args.
#[test]
fn u13_wrap_result_both_args() {
    let ty: Type = parse_str("Result<String, i32>").unwrap();
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
    assert!(s.contains("adze :: WithLeaf < String >"));
    assert!(s.contains("adze :: WithLeaf < i32 >"));
}

/// U14: Wrap then reparse round-trip.
#[test]
fn u14_wrap_reparse_roundtrip() {
    let ty: Type = parse_str("i32").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    let tokens = wrapped.to_token_stream().to_string();
    let reparsed: Type = parse_str(&tokens).unwrap();
    assert_eq!(reparsed.to_token_stream().to_string(), tokens);
}

/// U15: Wrap Vec<Option<bool>> with both in skip, leaf gets wrapped.
#[test]
fn u15_wrap_vec_option_leaf() {
    let ty: Type = parse_str("Vec<Option<bool>>").unwrap();
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let s = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
    assert!(s.starts_with("Vec <"));
    assert!(s.contains("Option <"));
    assert!(s.contains("adze :: WithLeaf < bool >"));
}

// --- NameValueExpr ---

/// U16: Parse key = "value".
#[test]
fn u16_nve_string_value() {
    let nv: NameValueExpr = parse_quote!(key = "value");
    assert_eq!(nv.path.to_string(), "key");
}

/// U17: Parse precedence = 5.
#[test]
fn u17_nve_int_value() {
    let nv: NameValueExpr = parse_quote!(precedence = 5);
    assert_eq!(nv.path.to_string(), "precedence");
    assert_eq!(nv.expr.to_token_stream().to_string(), "5");
}

/// U18: Parse flag = true.
#[test]
fn u18_nve_bool_value() {
    let nv: NameValueExpr = parse_quote!(flag = true);
    assert_eq!(nv.path.to_string(), "flag");
}

/// U19: Parse module = foo (path expr).
#[test]
fn u19_nve_path_expr() {
    let nv: NameValueExpr = parse_quote!(module = foo);
    assert_eq!(nv.path.to_string(), "module");
    assert_eq!(nv.expr.to_token_stream().to_string(), "foo");
}

/// U20: Clone preserves equality.
#[test]
fn u20_nve_clone_eq() {
    let nv: NameValueExpr = parse_quote!(key = 42);
    let nv2 = nv.clone();
    assert_eq!(nv, nv2);
}

// --- FieldThenParams ---

/// U21: Bare field type, no params.
#[test]
fn u21_ftp_bare() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

/// U22: Field with two params.
#[test]
fn u22_ftp_two_params() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test", value = 42);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
}

/// U23: Field with single param.
#[test]
fn u23_ftp_single_param() {
    let ftp: FieldThenParams = parse_quote!(u32, limit = 100);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "limit");
}

/// U24: Complex field type Vec<String> with param.
#[test]
fn u24_ftp_complex_field() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>, sep = ",");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "sep");
}

/// U25: Clone preserves equality for FieldThenParams.
#[test]
fn u25_ftp_clone_eq() {
    let ftp: FieldThenParams = parse_quote!(i32, name = "x");
    let ftp2 = ftp.clone();
    assert_eq!(ftp, ftp2);
}
