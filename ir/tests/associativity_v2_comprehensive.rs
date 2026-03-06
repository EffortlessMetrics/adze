//! Comprehensive tests for Associativity enum v2.

use adze_ir::Associativity;

#[test]
fn v2_assoc_left_debug() {
    let d = format!("{:?}", Associativity::Left);
    assert!(d.contains("Left"));
}

#[test]
fn v2_assoc_right_debug() {
    let d = format!("{:?}", Associativity::Right);
    assert!(d.contains("Right"));
}

#[test]
fn v2_assoc_left_clone() {
    let a = Associativity::Left;
    let c = a;
    assert_eq!(format!("{:?}", a), format!("{:?}", c));
}

#[test]
fn v2_assoc_right_clone() {
    let a = Associativity::Right;
    let c = a;
    assert_eq!(format!("{:?}", a), format!("{:?}", c));
}

#[test]
fn v2_assoc_serialize_left() {
    let json = serde_json::to_string(&Associativity::Left).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn v2_assoc_serialize_right() {
    let json = serde_json::to_string(&Associativity::Right).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn v2_assoc_roundtrip_left() {
    let a = Associativity::Left;
    let json = serde_json::to_string(&a).unwrap();
    let a2: Associativity = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", a), format!("{:?}", a2));
}

#[test]
fn v2_assoc_roundtrip_right() {
    let a = Associativity::Right;
    let json = serde_json::to_string(&a).unwrap();
    let a2: Associativity = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", a), format!("{:?}", a2));
}

#[test]
fn v2_assoc_in_vec() {
    let v = [Associativity::Left, Associativity::Right];
    assert_eq!(v.len(), 2);
}

#[test]
fn v2_assoc_match_left() {
    let a = Associativity::Left;
    match a {
        Associativity::Left => (),
        _ => panic!("wrong"),
    }
}

#[test]
fn v2_assoc_match_right() {
    let a = Associativity::Right;
    match a {
        Associativity::Right => (),
        _ => panic!("wrong"),
    }
}

#[test]
fn v2_assoc_in_builder() {
    use adze_ir::builder::GrammarBuilder;
    let g = GrammarBuilder::new("a")
        .token("n", "n")
        .token("plus", "\\+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    assert!(g.all_rules().count() >= 2);
}
