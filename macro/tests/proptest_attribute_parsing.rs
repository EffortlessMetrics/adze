use proc_macro2::TokenStream;
use proptest::prelude::*;
use quote::quote;
use syn::{self, DeriveInput, Fields, ItemStruct, Visibility};

// --- Strategies ---

fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("not a Rust keyword", |s| {
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
                | "gen"
        )
    })
}

fn struct_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,10}"
}

fn type_name_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("u8"),
        Just("u16"),
        Just("u32"),
        Just("i32"),
        Just("String"),
        Just("bool"),
        Just("f64"),
    ]
}

fn visibility_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just(""), Just("pub "), Just("pub(crate) "),]
}

fn attribute_strategy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just(""),
        Just("#[allow(dead_code)] "),
        Just("#[cfg(test)] "),
        Just("#[doc = \"hello\"] "),
    ]
}

fn generic_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("Vec<u8>".to_string()),
        Just("Option<String>".to_string()),
        Just("Box<i32>".to_string()),
        Just("Result<u32, String>".to_string()),
        Just("Vec<Vec<u8>>".to_string()),
        Just("Option<bool>".to_string()),
        Just("(u8, u16)".to_string()),
    ]
}

// --- proptest! macro tests ---

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 1. Any valid Rust identifier can be used as a struct field name
    #[test]
    fn valid_identifier_as_field_name(name in field_name_strategy()) {
        let code = format!("struct Foo {{ {name}: u32 }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok(), "Failed to parse field name: {name}");
    }

    // 2. syn::parse_str::<ItemStruct>() on valid struct code succeeds
    #[test]
    fn parse_valid_struct(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ {fname}: {ty} }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok(), "Failed to parse: {code}");
    }

    // 3. Simple struct definitions always have the expected field count
    #[test]
    fn struct_field_count(
        sname in struct_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
        t1 in type_name_strategy(),
        t2 in type_name_strategy(),
    ) {
        prop_assume!(f1 != f2);
        let code = format!("struct {sname} {{ {f1}: {t1}, {f2}: {t2} }}");
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        prop_assert_eq!(parsed.fields.len(), 2);
    }

    // 4. Visibility modifiers parse correctly
    #[test]
    fn visibility_modifier_parses(
        vis in visibility_strategy(),
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("{vis}struct {sname} {{ {vis}{fname}: {ty} }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok(), "Failed to parse with vis '{vis}': {code}");
    }

    // 5. Generic types parse correctly
    #[test]
    fn generic_type_parses(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        gty in generic_type_strategy(),
    ) {
        let code = format!("struct {sname} {{ {fname}: {gty} }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok(), "Failed to parse generic type: {code}");
    }

    // 6. Attributes on struct fields parse
    #[test]
    fn attribute_on_field_parses(
        attr in attribute_strategy(),
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ {attr}{fname}: {ty} }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok(), "Failed to parse with attribute: {code}");
    }

    // 7. Random valid Rust type strings parse as syn::Type
    #[test]
    fn type_string_parses(ty in type_name_strategy()) {
        let parsed = syn::parse_str::<syn::Type>(ty);
        prop_assert!(parsed.is_ok(), "Failed to parse type: {ty}");
    }

    // 8. DeriveInput from valid code has correct ident
    #[test]
    fn derive_input_ident(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ {fname}: {ty} }}");
        let parsed: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), sname);
    }

    // 9. Token stream from quote! is never empty for any valid code
    #[test]
    fn quote_token_stream_not_empty(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let sname_ident: syn::Ident = syn::parse_str(&sname).unwrap();
        let fname_ident: syn::Ident = syn::parse_str(&fname).unwrap();
        let ty_parsed: syn::Type = syn::parse_str(ty).unwrap();
        let tokens: TokenStream = quote! {
            struct #sname_ident {
                #fname_ident: #ty_parsed
            }
        };
        prop_assert!(!tokens.is_empty(), "Token stream was empty");
    }

    // 10. Multiple attributes on same field parse correctly
    #[test]
    fn multiple_attributes_on_field(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!(
            "struct {sname} {{ #[allow(dead_code)] #[cfg(test)] {fname}: {ty} }}"
        );
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        if let Fields::Named(ref fields) = parsed.fields {
            let field = fields.named.first().unwrap();
            prop_assert_eq!(field.attrs.len(), 2);
        } else {
            prop_assert!(false, "Expected named fields");
        }
    }

    // 11. Empty struct parses correctly
    #[test]
    fn empty_struct_parses(sname in struct_name_strategy()) {
        let code = format!("struct {sname} {{}}");
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        prop_assert_eq!(parsed.fields.len(), 0);
    }

    // 12. Unit struct parses correctly
    #[test]
    fn unit_struct_parses(sname in struct_name_strategy()) {
        let code = format!("struct {sname};");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok());
    }

    // 13. Tuple struct parses
    #[test]
    fn tuple_struct_parses(sname in struct_name_strategy(), ty in type_name_strategy()) {
        let code = format!("struct {sname}({ty});");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok());
    }

    // 14. Field name survives quote roundtrip
    #[test]
    fn field_name_quote_roundtrip(fname in field_name_strategy()) {
        let ident: syn::Ident = syn::parse_str(&fname).unwrap();
        let tokens: TokenStream = quote! { #ident };
        let reparsed: syn::Ident = syn::parse2(tokens).unwrap();
        prop_assert_eq!(ident, reparsed);
    }

    // 15. Struct name survives quote roundtrip
    #[test]
    fn struct_name_quote_roundtrip(sname in struct_name_strategy()) {
        let ident: syn::Ident = syn::parse_str(&sname).unwrap();
        let tokens: TokenStream = quote! { #ident };
        let reparsed: syn::Ident = syn::parse2(tokens).unwrap();
        prop_assert_eq!(ident.to_string(), reparsed.to_string());
    }

    // 16. Type survives quote roundtrip
    #[test]
    fn type_quote_roundtrip(ty in type_name_strategy()) {
        let parsed: syn::Type = syn::parse_str(ty).unwrap();
        let tokens: TokenStream = quote! { #parsed };
        let reparsed: syn::Type = syn::parse2(tokens).unwrap();
        prop_assert_eq!(quote!(#parsed).to_string(), quote!(#reparsed).to_string());
    }

    // 17. Generic type survives quote roundtrip
    #[test]
    fn generic_type_quote_roundtrip(gty in generic_type_strategy()) {
        let parsed: syn::Type = syn::parse_str(&gty).unwrap();
        let tokens: TokenStream = quote! { #parsed };
        let reparsed: syn::Type = syn::parse2(tokens).unwrap();
        prop_assert_eq!(quote!(#parsed).to_string(), quote!(#reparsed).to_string());
    }

    // 18. pub field visibility is detected
    #[test]
    fn pub_field_detected(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ pub {fname}: {ty} }}");
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        if let Fields::Named(ref fields) = parsed.fields {
            let field = fields.named.first().unwrap();
            prop_assert!(matches!(field.vis, Visibility::Public(_)));
        }
    }

    // 19. Non-pub field has inherited visibility
    #[test]
    fn private_field_detected(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ {fname}: {ty} }}");
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        if let Fields::Named(ref fields) = parsed.fields {
            let field = fields.named.first().unwrap();
            prop_assert!(matches!(field.vis, Visibility::Inherited));
        }
    }

    // 20. Struct with many fields parses
    #[test]
    fn many_fields_struct(
        sname in struct_name_strategy(),
        count in 1usize..8,
        ty in type_name_strategy(),
    ) {
        let fields_str: String = (0..count)
            .map(|i| format!("f{i}: {ty}"))
            .collect::<Vec<_>>()
            .join(", ");
        let code = format!("struct {sname} {{ {fields_str} }}");
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        prop_assert_eq!(parsed.fields.len(), count);
    }

    // 21. Derive attribute on struct parses
    #[test]
    fn derive_on_struct(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("#[derive(Debug, Clone)] struct {sname} {{ {fname}: {ty} }}");
        let parsed: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert_eq!(parsed.attrs.len(), 1);
    }

    // 22. Multiple derives on struct
    #[test]
    fn multiple_derives_on_struct(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!(
            "#[derive(Debug)] #[derive(Clone)] struct {sname} {{ {fname}: {ty} }}"
        );
        let parsed: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert_eq!(parsed.attrs.len(), 2);
    }

    // 23. doc attribute on struct parses
    #[test]
    fn doc_attribute_on_struct(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("#[doc = \"some doc\"] struct {sname} {{ {fname}: {ty} }}");
        let parsed: DeriveInput = syn::parse_str(&code).unwrap();
        prop_assert_eq!(parsed.attrs.len(), 1);
    }

    // 24. Struct with Option type
    #[test]
    fn option_field_parses(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ {fname}: Option<{inner}> }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok());
    }

    // 25. Struct with Vec type
    #[test]
    fn vec_field_parses(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ {fname}: Vec<{inner}> }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok());
    }

    // 26. pub(crate) struct visibility
    #[test]
    fn pub_crate_struct_visibility(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("pub(crate) struct {sname} {{ {fname}: {ty} }}");
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        prop_assert!(matches!(parsed.vis, Visibility::Restricted(_)));
    }

    // 27. Full struct quote roundtrip
    #[test]
    fn full_struct_quote_roundtrip(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ {fname}: {ty} }}");
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        let tokens: TokenStream = quote! { #parsed };
        let reparsed: ItemStruct = syn::parse2(tokens).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), reparsed.ident.to_string());
        prop_assert_eq!(parsed.fields.len(), reparsed.fields.len());
    }

    // 28. Nested generic types
    #[test]
    fn nested_generic_types(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
    ) {
        let code = format!("struct {sname} {{ {fname}: Vec<Option<String>> }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok());
    }

    // 29. Reference type fields
    #[test]
    fn reference_type_field(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
        ty in type_name_strategy(),
    ) {
        let code = format!("struct {sname}<'a> {{ {fname}: &'a {ty} }}");
        let parsed = syn::parse_str::<ItemStruct>(&code);
        prop_assert!(parsed.is_ok());
    }

    // 30. Generic struct with type param
    #[test]
    fn generic_struct_with_type_param(
        sname in struct_name_strategy(),
        fname in field_name_strategy(),
    ) {
        let code = format!("struct {sname}<T> {{ {fname}: T }}");
        let parsed: ItemStruct = syn::parse_str(&code).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 1);
    }
}

// --- Regular #[test] functions ---

#[test]
fn test_parse_empty_struct() {
    let code = "struct Empty {}";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(parsed.ident, "Empty");
    assert_eq!(parsed.fields.len(), 0);
}

#[test]
fn test_parse_unit_struct() {
    let code = "struct Unit;";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(parsed.ident, "Unit");
    assert!(matches!(parsed.fields, Fields::Unit));
}

#[test]
fn test_parse_tuple_struct() {
    let code = "struct Pair(u32, String);";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(parsed.ident, "Pair");
    assert_eq!(parsed.fields.len(), 2);
}

#[test]
fn test_derive_input_ident_simple() {
    let code = "struct MyStruct { x: i32 }";
    let parsed: DeriveInput = syn::parse_str(code).unwrap();
    assert_eq!(parsed.ident, "MyStruct");
}

#[test]
fn test_multiple_field_types() {
    let types = ["u8", "u16", "u32", "i32", "String", "bool", "f64"];
    for ty in &types {
        let code = format!("struct Foo {{ val: {ty} }}");
        assert!(syn::parse_str::<ItemStruct>(&code).is_ok(), "type {ty}");
    }
}

#[test]
fn test_generic_option_field() {
    let code = "struct Foo { x: Option<u32> }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(parsed.fields.len(), 1);
}

#[test]
fn test_generic_vec_field() {
    let code = "struct Foo { items: Vec<String> }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(parsed.fields.len(), 1);
}

#[test]
fn test_generic_result_field() {
    let code = "struct Foo { res: Result<u32, String> }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(parsed.fields.len(), 1);
}

#[test]
fn test_pub_struct_visibility() {
    let code = "pub struct Foo { x: u32 }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert!(matches!(parsed.vis, Visibility::Public(_)));
}

#[test]
fn test_pub_crate_visibility() {
    let code = "pub(crate) struct Foo { x: u32 }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert!(matches!(parsed.vis, Visibility::Restricted(_)));
}

#[test]
fn test_inherited_visibility() {
    let code = "struct Foo { x: u32 }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert!(matches!(parsed.vis, Visibility::Inherited));
}

#[test]
fn test_field_with_allow_attribute() {
    let code = "struct Foo { #[allow(unused)] x: u32 }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    if let Fields::Named(ref fields) = parsed.fields {
        assert_eq!(fields.named.first().unwrap().attrs.len(), 1);
    }
}

#[test]
fn test_field_with_cfg_attribute() {
    let code = "struct Foo { #[cfg(test)] x: u32 }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    if let Fields::Named(ref fields) = parsed.fields {
        assert_eq!(fields.named.first().unwrap().attrs.len(), 1);
    }
}

#[test]
fn test_field_with_doc_attribute() {
    let code = "struct Foo { #[doc = \"field doc\"] x: u32 }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    if let Fields::Named(ref fields) = parsed.fields {
        assert_eq!(fields.named.first().unwrap().attrs.len(), 1);
    }
}

#[test]
fn test_field_with_three_attributes() {
    let code = "struct Foo { #[allow(unused)] #[cfg(test)] #[doc = \"hi\"] x: u32 }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    if let Fields::Named(ref fields) = parsed.fields {
        assert_eq!(fields.named.first().unwrap().attrs.len(), 3);
    }
}

#[test]
fn test_quote_produces_nonempty_tokens() {
    let tokens: TokenStream = quote! { struct Foo { x: u32 } };
    assert!(!tokens.is_empty());
}

#[test]
fn test_quote_empty_struct_nonempty() {
    let tokens: TokenStream = quote! { struct Empty {} };
    assert!(!tokens.is_empty());
}

#[test]
fn test_struct_with_lifetime() {
    let code = "struct Foo<'a> { s: &'a str }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(parsed.generics.params.len(), 1);
}

#[test]
fn test_struct_with_type_param() {
    let code = "struct Foo<T> { val: T }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert_eq!(parsed.generics.params.len(), 1);
}

#[test]
fn test_struct_with_where_clause() {
    let code = "struct Foo<T> where T: Clone { val: T }";
    let parsed: ItemStruct = syn::parse_str(code).unwrap();
    assert!(parsed.generics.where_clause.is_some());
}

#[test]
fn test_invalid_struct_fails() {
    let code = "struct { x: u32 }";
    assert!(syn::parse_str::<ItemStruct>(code).is_err());
}

#[test]
fn test_invalid_type_fails() {
    let code = "not a type !!";
    assert!(syn::parse_str::<syn::Type>(code).is_err());
}

#[test]
fn test_derive_debug_attribute() {
    let code = "#[derive(Debug)] struct Foo { x: u32 }";
    let parsed: DeriveInput = syn::parse_str(code).unwrap();
    assert_eq!(parsed.attrs.len(), 1);
}
