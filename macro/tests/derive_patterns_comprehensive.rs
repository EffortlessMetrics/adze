// Comprehensive tests for derive macro patterns used in adze.
// Tests the syn/quote/proc_macro2 patterns that underpin proc-macro processing.

use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericParam, ItemEnum, ItemStruct, Lifetime, Type,
    parse_quote, parse2,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn parse_derive_input(tokens: TokenStream) -> DeriveInput {
    parse2::<DeriveInput>(tokens).expect("failed to parse DeriveInput")
}

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2::<ItemStruct>(tokens).expect("failed to parse ItemStruct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2::<ItemEnum>(tokens).expect("failed to parse ItemEnum")
}

fn has_derive(attrs: &[Attribute], derive_name: &str) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            let s = attr.to_token_stream().to_string();
            s.contains(derive_name)
        } else {
            false
        }
    })
}

fn is_path_attr(attr: &Attribute, ns: &str, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == ns && segs[1].ident == name
}

fn field_count(fields: &Fields) -> usize {
    match fields {
        Fields::Named(f) => f.named.len(),
        Fields::Unnamed(f) => f.unnamed.len(),
        Fields::Unit => 0,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Parse derive attribute patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn derive_single_trait() {
    let input = parse_derive_input(quote! {
        #[derive(Clone)]
        struct Foo;
    });
    assert!(has_derive(&input.attrs, "Clone"));
}

#[test]
fn derive_multiple_traits() {
    let input = parse_derive_input(quote! {
        #[derive(Clone, Debug, PartialEq)]
        struct Bar;
    });
    assert!(has_derive(&input.attrs, "Clone"));
    assert!(has_derive(&input.attrs, "Debug"));
    assert!(has_derive(&input.attrs, "PartialEq"));
}

#[test]
fn derive_preserves_ident() {
    let input = parse_derive_input(quote! {
        #[derive(Default)]
        struct MyStruct { x: i32 }
    });
    assert_eq!(input.ident.to_string(), "MyStruct");
}

#[test]
fn derive_with_doc_attr() {
    let input = parse_derive_input(quote! {
        /// Some docs
        #[derive(Debug)]
        struct Documented;
    });
    assert!(has_derive(&input.attrs, "Debug"));
    assert!(input.attrs.iter().any(|a| a.path().is_ident("doc")));
}

#[test]
fn derive_no_attrs() {
    let input = parse_derive_input(quote! {
        struct Plain;
    });
    assert!(input.attrs.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Parse custom attribute patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn custom_attr_two_segment_path() {
    let s = parse_struct(quote! {
        #[adze::language]
        pub struct Lang;
    });
    assert!(is_path_attr(&s.attrs[0], "adze", "language"));
}

#[test]
fn custom_attr_leaf_text() {
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf(text = "+")]
            op: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(is_path_attr(&field.attrs[0], "adze", "leaf"));
}

#[test]
fn custom_attr_leaf_pattern() {
    let s: ItemStruct = parse_quote! {
        struct T {
            #[adze::leaf(pattern = r"\d+")]
            num: String,
        }
    };
    let tokens = s.fields.iter().next().unwrap().attrs[0]
        .to_token_stream()
        .to_string();
    assert!(tokens.contains("pattern"));
}

#[test]
fn custom_attr_extra() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Ws {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(is_path_attr(&s.attrs[0], "adze", "extra"));
}

#[test]
fn custom_attr_prec_left() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, (), Box<Expr>),
        }
    };
    let variant_attr = &e.variants[0].attrs[0];
    assert!(is_path_attr(variant_attr, "adze", "prec_left"));
}

#[test]
fn custom_attr_prec_right() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_right(2)]
            Cons(Box<Expr>, (), Box<Expr>),
        }
    };
    assert!(is_path_attr(&e.variants[0].attrs[0], "adze", "prec_right"));
}

#[test]
fn custom_attr_prec_no_assoc() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec(3)]
            Cmp(Box<Expr>, (), Box<Expr>),
        }
    };
    assert!(is_path_attr(&e.variants[0].attrs[0], "adze", "prec"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Parse struct with derives
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn struct_unit() {
    let s = parse_struct(quote! { struct Unit; });
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn struct_named_fields() {
    let s = parse_struct(quote! {
        struct Named { x: i32, y: String }
    });
    assert_eq!(field_count(&s.fields), 2);
}

#[test]
fn struct_unnamed_fields() {
    let s = parse_struct(quote! {
        struct Tuple(i32, String);
    });
    assert_eq!(field_count(&s.fields), 2);
}

#[test]
fn struct_with_pub_visibility() {
    let s = parse_struct(quote! {
        pub struct Public { pub x: i32 }
    });
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
}

#[test]
fn struct_field_types_preserved() {
    let s: ItemStruct = parse_quote! {
        struct S { a: Vec<i32>, b: Option<String> }
    };
    let types: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect();
    assert!(types[0].contains("Vec"));
    assert!(types[1].contains("Option"));
}

#[test]
fn struct_derive_data_variant() {
    let input = parse_derive_input(quote! {
        struct S { x: i32 }
    });
    assert!(matches!(input.data, Data::Struct(_)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Parse enum with derives
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn enum_single_variant() {
    let e = parse_enum(quote! {
        enum Single { One }
    });
    assert_eq!(e.variants.len(), 1);
}

#[test]
fn enum_multiple_variants() {
    let e = parse_enum(quote! {
        enum Multi { A, B, C, D }
    });
    assert_eq!(e.variants.len(), 4);
}

#[test]
fn enum_tuple_variant() {
    let e = parse_enum(quote! {
        enum E { Tup(i32, String) }
    });
    assert_eq!(field_count(&e.variants[0].fields), 2);
}

#[test]
fn enum_named_variant() {
    let e = parse_enum(quote! {
        enum E { Named { x: i32, y: bool } }
    });
    assert_eq!(field_count(&e.variants[0].fields), 2);
}

#[test]
fn enum_unit_variant() {
    let e = parse_enum(quote! {
        enum E { Unit }
    });
    assert_eq!(field_count(&e.variants[0].fields), 0);
}

#[test]
fn enum_mixed_variants() {
    let e = parse_enum(quote! {
        enum Mixed {
            Unit,
            Tuple(i32),
            Named { a: String },
        }
    });
    assert_eq!(field_count(&e.variants[0].fields), 0);
    assert_eq!(field_count(&e.variants[1].fields), 1);
    assert_eq!(field_count(&e.variants[2].fields), 1);
}

#[test]
fn enum_derive_data_variant() {
    let input = parse_derive_input(quote! {
        enum E { A, B }
    });
    assert!(matches!(input.data, Data::Enum(_)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Parse multiple derive macros
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn multiple_derive_attrs_separate() {
    let input = parse_derive_input(quote! {
        #[derive(Clone)]
        #[derive(Debug)]
        struct S;
    });
    let derive_count = input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .count();
    assert_eq!(derive_count, 2);
}

#[test]
fn multiple_derive_attrs_combined() {
    let input = parse_derive_input(quote! {
        #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
        struct S;
    });
    assert!(has_derive(&input.attrs, "Clone"));
    assert!(has_derive(&input.attrs, "Hash"));
}

#[test]
fn derive_plus_custom_attr() {
    let input = parse_derive_input(quote! {
        #[derive(Debug)]
        #[adze::language]
        struct S;
    });
    assert!(has_derive(&input.attrs, "Debug"));
    assert!(
        input
            .attrs
            .iter()
            .any(|a| is_path_attr(a, "adze", "language"))
    );
}

#[test]
fn derive_on_enum() {
    let input = parse_derive_input(quote! {
        #[derive(Clone, Debug)]
        enum E { A, B }
    });
    assert!(has_derive(&input.attrs, "Clone"));
    assert!(matches!(input.data, Data::Enum(_)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Parse nested attribute arguments
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn nested_attr_name_value() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[adze::leaf(text = "+")]
            op: (),
        }
    };
    let attr_str = s.fields.iter().next().unwrap().attrs[0]
        .to_token_stream()
        .to_string();
    assert!(attr_str.contains("text"));
    assert!(attr_str.contains("+"));
}

#[test]
fn nested_attr_multiple_args() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            num: i32,
        }
    };
    let attr_str = s.fields.iter().next().unwrap().attrs[0]
        .to_token_stream()
        .to_string();
    assert!(attr_str.contains("pattern"));
    assert!(attr_str.contains("transform"));
}

#[test]
fn nested_attr_boolean_arg() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[adze::repeat(non_empty = true)]
            items: Vec<i32>,
        }
    };
    let attr_str = s.fields.iter().next().unwrap().attrs[0]
        .to_token_stream()
        .to_string();
    assert!(attr_str.contains("non_empty"));
    assert!(attr_str.contains("true"));
}

#[test]
fn nested_attr_inner_attr_in_delimited() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<i32>,
        }
    };
    let attr_str = s.fields.iter().next().unwrap().attrs[0]
        .to_token_stream()
        .to_string();
    assert!(attr_str.contains("delimited"));
    assert!(attr_str.contains("leaf"));
}

#[test]
fn nested_attr_integer_arg() {
    let e: ItemEnum = parse_quote! {
        enum E {
            #[adze::prec_left(42)]
            V(i32),
        }
    };
    let attr_str = e.variants[0].attrs[0].to_token_stream().to_string();
    assert!(attr_str.contains("42"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Attribute path parsing
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn attr_path_single_segment() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        struct S;
    };
    assert!(s.attrs[0].path().is_ident("derive"));
}

#[test]
fn attr_path_two_segments() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct S;
    };
    let segs: Vec<_> = s.attrs[0].path().segments.iter().collect();
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].ident.to_string(), "adze");
    assert_eq!(segs[1].ident.to_string(), "language");
}

#[test]
fn attr_path_segments_iteration() {
    let s: ItemStruct = parse_quote! {
        #[some::nested::path]
        struct S;
    };
    let names: Vec<_> = s.attrs[0]
        .path()
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
    assert_eq!(names, vec!["some", "nested", "path"]);
}

#[test]
fn attr_path_is_ident_check() {
    let s: ItemStruct = parse_quote! {
        #[cfg(test)]
        struct S;
    };
    assert!(s.attrs[0].path().is_ident("cfg"));
}

#[test]
fn attr_path_not_matching() {
    let s: ItemStruct = parse_quote! {
        #[serde(rename = "x")]
        struct S;
    };
    assert!(!s.attrs[0].path().is_ident("derive"));
    assert!(s.attrs[0].path().is_ident("serde"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Attribute token parsing
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn attr_tokens_to_string() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "hello")]
        struct S;
    };
    let tok_str = s.attrs[0].to_token_stream().to_string();
    assert!(tok_str.contains("hello"));
}

#[test]
fn attr_meta_path_style() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct S;
    };
    let meta = &s.attrs[0].meta;
    assert!(matches!(meta, syn::Meta::Path(_)));
}

#[test]
fn attr_meta_list_style() {
    let s: ItemStruct = parse_quote! {
        #[derive(Clone, Debug)]
        struct S;
    };
    let meta = &s.attrs[0].meta;
    assert!(matches!(meta, syn::Meta::List(_)));
}

#[test]
fn attr_tokens_roundtrip() {
    let original: ItemStruct = parse_quote! {
        #[adze::leaf(pattern = r"\w+")]
        struct S;
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(original.ident, reparsed.ident);
    assert_eq!(original.attrs.len(), reparsed.attrs.len());
}

#[test]
fn attr_multiple_on_same_field() {
    let s: ItemStruct = parse_quote! {
        struct S {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<i32>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.attrs.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 9. TokenStream combination patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn token_stream_extend() {
    let a = quote! { fn foo() {} };
    let b = quote! { fn bar() {} };
    let mut combined = TokenStream::new();
    combined.extend(a);
    combined.extend(b);
    let s = combined.to_string();
    assert!(s.contains("foo"));
    assert!(s.contains("bar"));
}

#[test]
fn token_stream_interpolation() {
    let name = format_ident!("MyStruct");
    let field_ty = quote! { i32 };
    let result = quote! {
        struct #name {
            value: #field_ty,
        }
    };
    let parsed: ItemStruct = parse2(result).unwrap();
    assert_eq!(parsed.ident.to_string(), "MyStruct");
}

#[test]
fn token_stream_iteration_interpolation() {
    let names: Vec<Ident> = vec![
        format_ident!("Alpha"),
        format_ident!("Beta"),
        format_ident!("Gamma"),
    ];
    let result = quote! {
        enum Variants { #(#names),* }
    };
    let e: ItemEnum = parse2(result).unwrap();
    assert_eq!(e.variants.len(), 3);
}

#[test]
fn token_stream_nested_quote() {
    let inner = quote! { x: i32 };
    let outer = quote! {
        struct Wrapper { #inner }
    };
    let s: ItemStruct = parse2(outer).unwrap();
    assert_eq!(field_count(&s.fields), 1);
}

#[test]
fn token_stream_conditional_generation() {
    let include_field = true;
    let extra = if include_field {
        quote! { extra: bool, }
    } else {
        quote! {}
    };
    let result = quote! {
        struct S { base: i32, #extra }
    };
    let s: ItemStruct = parse2(result).unwrap();
    assert_eq!(field_count(&s.fields), 2);
}

#[test]
fn token_stream_empty() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
}

#[test]
fn token_stream_from_string() {
    let ts: TokenStream = "struct S;".parse().unwrap();
    let s: ItemStruct = parse2(ts).unwrap();
    assert_eq!(s.ident.to_string(), "S");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. Ident manipulation for derive expansion
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn ident_format_suffix() {
    let base = format_ident!("Expr");
    let derived = format_ident!("{base}Visitor");
    assert_eq!(derived.to_string(), "ExprVisitor");
}

#[test]
fn ident_format_prefix() {
    let base = format_ident!("Token");
    let derived = format_ident!("parse_{base}");
    assert_eq!(derived.to_string(), "parse_Token");
}

#[test]
fn ident_from_string() {
    let name = String::from("dynamic_name");
    let ident = Ident::new(&name, Span::call_site());
    assert_eq!(ident.to_string(), "dynamic_name");
}

#[test]
fn ident_in_quote() {
    let ty_name = format_ident!("Foo");
    let tokens = quote! { impl #ty_name {} };
    let s = tokens.to_string();
    assert!(s.contains("Foo"));
}

#[test]
fn ident_equality() {
    let a = format_ident!("same");
    let b = format_ident!("same");
    assert_eq!(a, b);
}

#[test]
fn ident_lowercase_conversion() {
    let variant = "MyVariant";
    let snake = variant
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_lowercase().to_string()
            }
        })
        .collect::<String>();
    let ident = format_ident!("{snake}");
    assert_eq!(ident.to_string(), "my_variant");
}

#[test]
fn ident_numbered_generation() {
    let idents: Vec<Ident> = (0..3).map(|i| format_ident!("field_{i}")).collect();
    assert_eq!(idents[0].to_string(), "field_0");
    assert_eq!(idents[2].to_string(), "field_2");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 11. Generic struct derive patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn generic_single_type_param() {
    let input = parse_derive_input(quote! {
        struct Wrapper<T> { inner: T }
    });
    assert_eq!(input.generics.params.len(), 1);
    assert!(matches!(&input.generics.params[0], GenericParam::Type(_)));
}

#[test]
fn generic_multiple_type_params() {
    let input = parse_derive_input(quote! {
        struct Pair<A, B> { first: A, second: B }
    });
    assert_eq!(input.generics.params.len(), 2);
}

#[test]
fn generic_with_bound() {
    let input = parse_derive_input(quote! {
        struct Bounded<T: Clone + Send> { val: T }
    });
    if let GenericParam::Type(tp) = &input.generics.params[0] {
        assert!(!tp.bounds.is_empty());
    } else {
        panic!("expected type param");
    }
}

#[test]
fn generic_with_where_clause() {
    let input = parse_derive_input(quote! {
        struct WithWhere<T> where T: Default { val: T }
    });
    assert!(input.generics.where_clause.is_some());
}

#[test]
fn generic_const_param() {
    let input = parse_derive_input(quote! {
        struct Array<const N: usize> { data: [u8; N] }
    });
    assert!(matches!(&input.generics.params[0], GenericParam::Const(_)));
}

#[test]
fn generic_split_for_impl() {
    let input = parse_derive_input(quote! {
        struct G<T: Clone> { val: T }
    });
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    let tokens = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn get(&self) -> &T { &self.val }
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("impl"));
    assert!(s.contains("Clone"));
}

#[test]
fn generic_quote_interpolation() {
    let name = format_ident!("Container");
    let ty_param = format_ident!("T");
    let result = quote! {
        struct #name<#ty_param> { inner: #ty_param }
    };
    let parsed: ItemStruct = parse2(result).unwrap();
    assert_eq!(parsed.generics.params.len(), 1);
}

#[test]
fn generic_default_type() {
    let input = parse_derive_input(quote! {
        struct WithDefault<T = String> { val: T }
    });
    if let GenericParam::Type(tp) = &input.generics.params[0] {
        assert!(tp.default.is_some());
    } else {
        panic!("expected type param");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 12. Lifetime struct derive patterns
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn lifetime_single() {
    let input = parse_derive_input(quote! {
        struct Ref<'a> { data: &'a str }
    });
    assert_eq!(input.generics.params.len(), 1);
    assert!(matches!(
        &input.generics.params[0],
        GenericParam::Lifetime(_)
    ));
}

#[test]
fn lifetime_multiple() {
    let input = parse_derive_input(quote! {
        struct Multi<'a, 'b> { x: &'a str, y: &'b str }
    });
    assert_eq!(input.generics.params.len(), 2);
}

#[test]
fn lifetime_with_type_param() {
    let input = parse_derive_input(quote! {
        struct Mixed<'a, T> { data: &'a T }
    });
    assert!(matches!(
        &input.generics.params[0],
        GenericParam::Lifetime(_)
    ));
    assert!(matches!(&input.generics.params[1], GenericParam::Type(_)));
}

#[test]
fn lifetime_bound() {
    let input = parse_derive_input(quote! {
        struct Bounded<'a, 'b: 'a> { x: &'a str, y: &'b str }
    });
    if let GenericParam::Lifetime(lt) = &input.generics.params[1] {
        assert!(!lt.bounds.is_empty());
    } else {
        panic!("expected lifetime param");
    }
}

#[test]
fn lifetime_in_quote() {
    let lt = Lifetime::new("'a", Span::call_site());
    let tokens = quote! {
        struct Ref<#lt> { data: &#lt str }
    };
    let parsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(parsed.generics.params.len(), 1);
}

#[test]
fn lifetime_split_for_impl() {
    let input = parse_derive_input(quote! {
        struct Ref<'a> { data: &'a str }
    });
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    let tokens = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            fn data(&self) -> &str { self.data }
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("'a"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional pattern tests (roundtrips, edge cases, combinations)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn roundtrip_struct_parse_quote_reparse() {
    let original: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(original.ident, reparsed.ident);
    assert_eq!(field_count(&original.fields), field_count(&reparsed.fields));
}

#[test]
fn roundtrip_enum_parse_quote_reparse() {
    let original: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            Add(Box<Expr>, Box<Expr>),
        }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(original.variants.len(), reparsed.variants.len());
}

#[test]
fn field_type_box_extraction() {
    let s: ItemStruct = parse_quote! {
        struct S { child: Box<Expr> }
    };
    let ty_str = s
        .fields
        .iter()
        .next()
        .unwrap()
        .ty
        .to_token_stream()
        .to_string();
    assert!(ty_str.contains("Box"));
    assert!(ty_str.contains("Expr"));
}

#[test]
fn field_type_vec_extraction() {
    let s: ItemStruct = parse_quote! {
        struct S { items: Vec<Item> }
    };
    let ty_str = s
        .fields
        .iter()
        .next()
        .unwrap()
        .ty
        .to_token_stream()
        .to_string();
    assert!(ty_str.contains("Vec"));
    assert!(ty_str.contains("Item"));
}

#[test]
fn field_type_option_extraction() {
    let s: ItemStruct = parse_quote! {
        struct S { maybe: Option<i32> }
    };
    let ty_str = s
        .fields
        .iter()
        .next()
        .unwrap()
        .ty
        .to_token_stream()
        .to_string();
    assert!(ty_str.contains("Option"));
}

#[test]
fn enum_variant_with_boxed_self_reference() {
    let e: ItemEnum = parse_quote! {
        enum Tree {
            Leaf(i32),
            Branch(Box<Tree>, Box<Tree>),
        }
    };
    assert_eq!(field_count(&e.variants[0].fields), 1);
    assert_eq!(field_count(&e.variants[1].fields), 2);
}

#[test]
fn derive_input_data_fields_extraction() {
    let input = parse_derive_input(quote! {
        struct S { a: i32, b: String, c: bool }
    });
    if let Data::Struct(ds) = &input.data {
        assert_eq!(field_count(&ds.fields), 3);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn derive_input_enum_variant_names() {
    let input = parse_derive_input(quote! {
        enum Color { Red, Green, Blue }
    });
    if let Data::Enum(de) = &input.data {
        let names: Vec<_> = de.variants.iter().map(|v| v.ident.to_string()).collect();
        assert_eq!(names, vec!["Red", "Green", "Blue"]);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn quote_generate_impl_block() {
    let ty_name = format_ident!("Foo");
    let method_name = format_ident!("new");
    let tokens = quote! {
        impl #ty_name {
            fn #method_name() -> Self {
                Self {}
            }
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("impl Foo"));
    assert!(s.contains("fn new"));
}

#[test]
fn quote_generate_match_arms() {
    let variants = vec![
        (format_ident!("A"), quote! { 1 }),
        (format_ident!("B"), quote! { 2 }),
        (format_ident!("C"), quote! { 3 }),
    ];
    let arms = variants.iter().map(|(name, val)| {
        quote! { Self::#name => #val, }
    });
    let tokens = quote! {
        fn to_int(&self) -> i32 {
            match self {
                #(#arms)*
            }
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("Self :: A => 1"));
    assert!(s.contains("Self :: C => 3"));
}

#[test]
fn parse_type_from_string() {
    let ty: Type = syn::parse_str("Vec<Option<i32>>").unwrap();
    let s = ty.to_token_stream().to_string();
    assert!(s.contains("Vec"));
    assert!(s.contains("Option"));
    assert!(s.contains("i32"));
}

#[test]
fn struct_with_multiple_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(is_path_attr(&s.attrs[0], "adze", "language"));
    assert!(is_path_attr(&s.attrs[1], "adze", "word"));
}

#[test]
fn enum_all_unit_leaf_variants() {
    let e: ItemEnum = parse_quote! {
        enum Kw {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
            #[adze::leaf(text = "while")]
            While,
            #[adze::leaf(text = "for")]
            For,
        }
    };
    assert_eq!(e.variants.len(), 4);
    for v in &e.variants {
        assert_eq!(field_count(&v.fields), 0);
        assert!(is_path_attr(&v.attrs[0], "adze", "leaf"));
    }
}

#[test]
fn complex_enum_with_all_adze_attrs() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::leaf(pattern = r"\d+")]
            Num(String),
            #[adze::prec_left(1)]
            Add(Box<Expr>, (), Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, (), Box<Expr>),
            #[adze::prec_right(3)]
            Pow(Box<Expr>, (), Box<Expr>),
            #[adze::prec(0)]
            Cmp(Box<Expr>, (), Box<Expr>),
        }
    };
    assert_eq!(e.variants.len(), 5);
    assert!(is_path_attr(&e.variants[0].attrs[0], "adze", "leaf"));
    assert!(is_path_attr(&e.variants[1].attrs[0], "adze", "prec_left"));
    assert!(is_path_attr(&e.variants[3].attrs[0], "adze", "prec_right"));
    assert!(is_path_attr(&e.variants[4].attrs[0], "adze", "prec"));
}
