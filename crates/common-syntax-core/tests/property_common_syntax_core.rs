use std::collections::HashSet;

use adze_common_syntax_core::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use syn::{Type, parse_str};

#[derive(Debug, Clone, Copy)]
enum WrapperKind {
    Box,
    Option,
    Vec,
    Arc,
}

impl WrapperKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Box => "Box",
            Self::Option => "Option",
            Self::Vec => "Vec",
            Self::Arc => "Arc",
        }
    }
}

fn base_type() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("u8"), Just("String"), Just("bool"), Just("Node"),]
}

fn wrapper_strategy() -> impl Strategy<Value = WrapperKind> {
    prop_oneof![
        1 => Just(WrapperKind::Box),
        1 => Just(WrapperKind::Option),
        1 => Just(WrapperKind::Vec),
        1 => Just(WrapperKind::Arc),
    ]
}

fn build_type(base: &'static str, wrappers: &[WrapperKind]) -> Type {
    let mut rendered = base.to_string();
    for wrapper in wrappers.iter().rev() {
        rendered = format!("{}<{}>", wrapper.as_str(), rendered);
    }

    parse_str::<Type>(&rendered).unwrap_or_else(|error| {
        panic!("unexpected parse failure for generated type `{rendered}`: {error}")
    })
}

fn make_skip_set(
    want_box: bool,
    want_option: bool,
    want_vec: bool,
    want_arc: bool,
) -> HashSet<&'static str> {
    let mut set = HashSet::new();
    if want_box {
        set.insert("Box");
    }
    if want_option {
        set.insert("Option");
    }
    if want_vec {
        set.insert("Vec");
    }
    if want_arc {
        set.insert("Arc");
    }
    set
}

proptest! {
    #[test]
    fn filter_is_idempotent(
        base in base_type(),
        wrappers in prop::collection::vec(wrapper_strategy(), 0..4),
        want_box in any::<bool>(),
        want_option in any::<bool>(),
        want_vec in any::<bool>(),
        want_arc in any::<bool>(),
    ) {
        let original = build_type(base, &wrappers);
        let skip_over = make_skip_set(want_box, want_option, want_vec, want_arc);
        let filtered_once = filter_inner_type(&original, &skip_over);
        let filtered_twice = filter_inner_type(&filtered_once, &skip_over);

        prop_assert_eq!(filtered_once.to_token_stream().to_string(), filtered_twice.to_token_stream().to_string());
    }

    #[test]
    fn try_extract_inner_type_is_consistent_with_match_path(
        base in base_type(),
        wrappers in prop::collection::vec(wrapper_strategy(), 0..4),
        target in prop_oneof![Just("Box"), Just("Option"), Just("Vec"), Just("Arc")],
        want_box in any::<bool>(),
        want_option in any::<bool>(),
        want_vec in any::<bool>(),
        want_arc in any::<bool>(),
    ) {
        let original = build_type(base, &wrappers);
        let skip_over = make_skip_set(want_box, want_option, want_vec, want_arc);
        let (inner, extracted) = try_extract_inner_type(&original, target, &skip_over);
        let original_text = original.to_token_stream().to_string();
        let inner_text = inner.to_token_stream().to_string();

        if extracted {
            prop_assert_ne!(original_text, inner_text);
        } else {
            prop_assert_eq!(original_text, inner_text);
        }
    }

    #[test]
    fn wrap_leaf_type_always_wraps_leaf(
        base in base_type(),
        wrappers in prop::collection::vec(wrapper_strategy(), 0..4),
        want_box in any::<bool>(),
        want_option in any::<bool>(),
        want_vec in any::<bool>(),
        want_arc in any::<bool>(),
    ) {
        let original = build_type(base, &wrappers);
        let skip_over = make_skip_set(want_box, want_option, want_vec, want_arc);
        let wrapped = wrap_leaf_type(&original, &skip_over);

        prop_assert!(wrapped.to_token_stream().to_string().contains("WithLeaf"));
    }
}
