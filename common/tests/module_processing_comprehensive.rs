#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for module processing logic in adze-common.
//!
//! Covers: module item extraction, struct extraction, enum extraction,
//! mixed items, visibility handling, attribute handling, item ordering,
//! empty modules, and modules with use items.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Item, ItemMod, Type, Visibility, parse_str};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_mod(src: &str) -> ItemMod {
    parse_str(src).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn type_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. Module item extraction
// ===========================================================================

#[test]
fn extract_single_struct_from_module() {
    let m = parse_mod("mod m { struct Foo; }");
    let items = module_items(&m);
    assert_eq!(items.len(), 1);
    assert!(matches!(&items[0], Item::Struct(s) if s.ident == "Foo"));
}

#[test]
fn extract_single_enum_from_module() {
    let m = parse_mod("mod m { enum Color { Red, Green, Blue } }");
    let items = module_items(&m);
    assert_eq!(items.len(), 1);
    assert!(matches!(&items[0], Item::Enum(e) if e.ident == "Color"));
}

#[test]
fn extract_single_fn_from_module() {
    let m = parse_mod("mod m { fn helper() {} }");
    let items = module_items(&m);
    assert_eq!(items.len(), 1);
    assert!(matches!(&items[0], Item::Fn(f) if f.sig.ident == "helper"));
}

#[test]
fn extract_const_from_module() {
    let m = parse_mod("mod m { const MAX: u32 = 100; }");
    let items = module_items(&m);
    assert_eq!(items.len(), 1);
    assert!(matches!(&items[0], Item::Const(_)));
}

#[test]
fn extract_type_alias_from_module() {
    let m = parse_mod("mod m { type Num = i64; }");
    let items = module_items(&m);
    assert_eq!(items.len(), 1);
    assert!(matches!(&items[0], Item::Type(_)));
}

// ===========================================================================
// 2. Module struct extraction
// ===========================================================================

#[test]
fn struct_with_named_fields_extracted() {
    let m = parse_mod("mod m { struct Point { x: f64, y: f64 } }");
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        let names: Vec<_> = s
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        assert_eq!(names, vec!["x", "y"]);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn struct_field_types_extracted() {
    let m = parse_mod("mod m { struct Node { value: Vec<String> } }");
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        let field = s.fields.iter().next().unwrap();
        let (inner, ok) = try_extract_inner_type(&field.ty, "Vec", &skip_set(&[]));
        assert!(ok);
        assert_eq!(type_str(&inner), "String");
    } else {
        panic!("expected struct");
    }
}

#[test]
fn struct_with_box_field_filter() {
    let m = parse_mod("mod m { struct Wrapper { inner: Box<Expr> } }");
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        let field = s.fields.iter().next().unwrap();
        let filtered = filter_inner_type(&field.ty, &skip_set(&["Box"]));
        assert_eq!(type_str(&filtered), "Expr");
    } else {
        panic!("expected struct");
    }
}

#[test]
fn struct_field_wrap_leaf() {
    let m = parse_mod("mod m { struct Leaf { tok: Token } }");
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        let field = s.fields.iter().next().unwrap();
        let wrapped = wrap_leaf_type(&field.ty, &skip_set(&[]));
        assert_eq!(type_str(&wrapped), "adze :: WithLeaf < Token >");
    } else {
        panic!("expected struct");
    }
}

// ===========================================================================
// 3. Module enum extraction
// ===========================================================================

#[test]
fn enum_variant_count_preserved() {
    let m = parse_mod("mod m { enum Op { Add, Sub, Mul, Div } }");
    let items = module_items(&m);
    if let Item::Enum(e) = &items[0] {
        assert_eq!(e.variants.len(), 4);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn enum_variant_names_preserved_in_order() {
    let m = parse_mod("mod m { enum Dir { North, South, East, West } }");
    let items = module_items(&m);
    if let Item::Enum(e) = &items[0] {
        let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
        assert_eq!(names, vec!["North", "South", "East", "West"]);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn enum_with_tuple_variant_data() {
    let m = parse_mod("mod m { enum Expr { Lit(i32), Ident(String) } }");
    let items = module_items(&m);
    if let Item::Enum(e) = &items[0] {
        assert_eq!(e.variants.len(), 2);
        // First variant has one unnamed field
        assert_eq!(e.variants[0].fields.len(), 1);
        assert_eq!(e.variants[1].fields.len(), 1);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn enum_with_struct_variant() {
    let m = parse_mod("mod m { enum Stmt { Assign { name: String, value: i32 } } }");
    let items = module_items(&m);
    if let Item::Enum(e) = &items[0] {
        let variant = &e.variants[0];
        assert_eq!(variant.ident, "Assign");
        let field_names: Vec<_> = variant
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        assert_eq!(field_names, vec!["name", "value"]);
    } else {
        panic!("expected enum");
    }
}

// ===========================================================================
// 4. Module with mixed items
// ===========================================================================

#[test]
fn mixed_struct_enum_fn_counted() {
    let m = parse_mod(
        "mod m {
            struct A;
            enum B { X }
            fn c() {}
        }",
    );
    let items = module_items(&m);
    assert_eq!(items.len(), 3);
    assert!(matches!(&items[0], Item::Struct(_)));
    assert!(matches!(&items[1], Item::Enum(_)));
    assert!(matches!(&items[2], Item::Fn(_)));
}

#[test]
fn mixed_items_structs_and_enums_filtered() {
    let m = parse_mod(
        "mod m {
            struct S1 { x: i32 }
            enum E1 { A, B }
            struct S2 { y: u8 }
            const C: i32 = 0;
            enum E2 { P, Q }
        }",
    );
    let items = module_items(&m);
    let structs: Vec<_> = items
        .iter()
        .filter_map(|i| match i {
            Item::Struct(s) => Some(s.ident.to_string()),
            _ => None,
        })
        .collect();
    let enums: Vec<_> = items
        .iter()
        .filter_map(|i| match i {
            Item::Enum(e) => Some(e.ident.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(structs, vec!["S1", "S2"]);
    assert_eq!(enums, vec!["E1", "E2"]);
}

#[test]
fn mixed_items_type_extraction_across_structs() {
    let m = parse_mod(
        "mod m {
            struct A { items: Vec<i32> }
            struct B { child: Option<String> }
        }",
    );
    let items = module_items(&m);
    // Extract Vec inner from first struct
    if let Item::Struct(s) = &items[0] {
        let field = s.fields.iter().next().unwrap();
        let (inner, ok) = try_extract_inner_type(&field.ty, "Vec", &skip_set(&[]));
        assert!(ok);
        assert_eq!(type_str(&inner), "i32");
    }
    // Extract Option inner from second struct
    if let Item::Struct(s) = &items[1] {
        let field = s.fields.iter().next().unwrap();
        let (inner, ok) = try_extract_inner_type(&field.ty, "Option", &skip_set(&[]));
        assert!(ok);
        assert_eq!(type_str(&inner), "String");
    }
}

// ===========================================================================
// 5. Module visibility handling
// ===========================================================================

#[test]
fn pub_module_detected() {
    let m = parse_mod("pub mod m { struct A; }");
    assert!(matches!(m.vis, Visibility::Public(_)));
}

#[test]
fn private_module_detected() {
    let m = parse_mod("mod m { struct A; }");
    assert!(matches!(m.vis, Visibility::Inherited));
}

#[test]
fn pub_crate_module_detected() {
    let m = parse_mod("pub(crate) mod m { struct A; }");
    assert!(matches!(m.vis, Visibility::Restricted(_)));
}

#[test]
fn pub_struct_inside_module() {
    let m = parse_mod("mod m { pub struct Foo { pub x: i32 } }");
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        assert!(matches!(s.vis, Visibility::Public(_)));
        let field = s.fields.iter().next().unwrap();
        assert!(matches!(field.vis, Visibility::Public(_)));
    } else {
        panic!("expected struct");
    }
}

#[test]
fn private_struct_inside_pub_module() {
    let m = parse_mod("pub mod m { struct Hidden; }");
    assert!(matches!(m.vis, Visibility::Public(_)));
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        assert!(matches!(s.vis, Visibility::Inherited));
    } else {
        panic!("expected struct");
    }
}

// ===========================================================================
// 6. Module attributes handling
// ===========================================================================

#[test]
fn module_outer_attribute_detected() {
    let m = parse_mod("#[cfg(test)] mod m { struct A; }");
    assert_eq!(m.attrs.len(), 1);
    let path_str = m.attrs[0].path().to_token_stream().to_string();
    assert_eq!(path_str, "cfg");
}

#[test]
fn module_with_multiple_attributes() {
    let m = parse_mod(
        "#[allow(dead_code)]
         #[cfg(feature = \"extra\")]
         mod m { struct A; }",
    );
    assert_eq!(m.attrs.len(), 2);
}

#[test]
fn struct_attribute_inside_module() {
    let m = parse_mod(
        "mod m {
            #[derive(Debug)]
            struct Foo { x: i32 }
        }",
    );
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        assert_eq!(s.attrs.len(), 1);
        let path_str = s.attrs[0].path().to_token_stream().to_string();
        assert_eq!(path_str, "derive");
    } else {
        panic!("expected struct");
    }
}

#[test]
fn field_attribute_inside_module_struct() {
    let m = parse_mod(
        r#"mod m {
            struct Foo {
                #[serde(rename = "val")]
                value: i32,
            }
        }"#,
    );
    let items = module_items(&m);
    if let Item::Struct(s) = &items[0] {
        let field = s.fields.iter().next().unwrap();
        assert_eq!(field.attrs.len(), 1);
    } else {
        panic!("expected struct");
    }
}

// ===========================================================================
// 7. Module item ordering
// ===========================================================================

#[test]
fn item_ordering_preserved_for_five_items() {
    let m = parse_mod(
        "mod m {
            struct First;
            enum Second { A }
            fn third() {}
            const FOURTH: i32 = 0;
            type Fifth = u8;
        }",
    );
    let items = module_items(&m);
    assert_eq!(items.len(), 5);
    assert!(matches!(&items[0], Item::Struct(s) if s.ident == "First"));
    assert!(matches!(&items[1], Item::Enum(e) if e.ident == "Second"));
    assert!(matches!(&items[2], Item::Fn(f) if f.sig.ident == "third"));
    assert!(matches!(&items[3], Item::Const(_)));
    assert!(matches!(&items[4], Item::Type(_)));
}

#[test]
fn struct_names_order_matches_declaration_order() {
    let m = parse_mod(
        "mod m {
            struct Zebra;
            struct Apple;
            struct Mango;
        }",
    );
    let items = module_items(&m);
    let names: Vec<_> = items
        .iter()
        .filter_map(|i| match i {
            Item::Struct(s) => Some(s.ident.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["Zebra", "Apple", "Mango"]);
}

#[test]
fn enum_before_struct_ordering() {
    let m = parse_mod(
        "mod m {
            enum E { X }
            struct S;
        }",
    );
    let items = module_items(&m);
    assert!(matches!(&items[0], Item::Enum(_)));
    assert!(matches!(&items[1], Item::Struct(_)));
}

// ===========================================================================
// 8. Empty module handling
// ===========================================================================

#[test]
fn empty_module_has_zero_items() {
    let m = parse_mod("mod m {}");
    let items = module_items(&m);
    assert!(items.is_empty());
}

#[test]
fn empty_module_preserves_ident() {
    let m = parse_mod("mod empty_grammar {}");
    assert_eq!(m.ident.to_string(), "empty_grammar");
}

#[test]
fn empty_pub_module_visibility() {
    let m = parse_mod("pub mod empty {}");
    assert!(matches!(m.vis, Visibility::Public(_)));
    assert!(module_items(&m).is_empty());
}

// ===========================================================================
// 9. Module with use items
// ===========================================================================

#[test]
fn use_item_extracted_from_module() {
    let m = parse_mod("mod m { use std::collections::HashMap; }");
    let items = module_items(&m);
    assert_eq!(items.len(), 1);
    assert!(matches!(&items[0], Item::Use(_)));
}

#[test]
fn multiple_use_items_counted() {
    let m = parse_mod(
        "mod m {
            use std::fmt;
            use std::io;
            use std::collections::HashSet;
        }",
    );
    let items = module_items(&m);
    let use_count = items.iter().filter(|i| matches!(i, Item::Use(_))).count();
    assert_eq!(use_count, 3);
}

#[test]
fn use_item_with_struct_mixed() {
    let m = parse_mod(
        "mod m {
            use std::fmt;
            struct Formatter { width: u32 }
        }",
    );
    let items = module_items(&m);
    assert_eq!(items.len(), 2);
    assert!(matches!(&items[0], Item::Use(_)));
    assert!(matches!(&items[1], Item::Struct(s) if s.ident == "Formatter"));
}

#[test]
fn use_glob_import_in_module() {
    let m = parse_mod("mod m { use std::collections::*; }");
    let items = module_items(&m);
    assert_eq!(items.len(), 1);
    assert!(matches!(&items[0], Item::Use(_)));
}
