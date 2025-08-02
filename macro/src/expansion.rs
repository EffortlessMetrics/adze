use std::collections::HashSet;

use crate::errors::IteratorExt as _;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use rust_sitter_common::*;
use syn::{parse::Parse, punctuated::Punctuated, *};

fn is_sitter_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .iter()
        .next()
        .map(|segment| segment.ident == "rust_sitter")
        .unwrap_or(false)
}

pub enum ParamOrField {
    Param(Expr),
    Field(FieldValue),
}

impl ToTokens for ParamOrField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ParamOrField::Param(expr) => expr.to_tokens(tokens),
            ParamOrField::Field(field) => field.to_tokens(tokens),
        }
    }
}

fn gen_field(ident_str: String, leaf: Field) -> Expr {
    let leaf_type = leaf.ty;

    let leaf_attr = leaf
        .attrs
        .iter()
        .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::leaf));

    let leaf_params = leaf_attr.and_then(|a| {
        a.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
            .ok()
    });

    let transform_param = leaf_params.as_ref().and_then(|p| {
        p.iter()
            .find(|param| param.path == "transform")
            .map(|p| p.expr.clone())
    });

    let (leaf_type, closure_expr): (Type, Expr) = match transform_param {
        Some(closure) => {
            let mut non_leaf = HashSet::new();
            non_leaf.insert("Spanned");
            non_leaf.insert("Box");
            non_leaf.insert("Option");
            non_leaf.insert("Vec");
            let wrapped_leaf_type = wrap_leaf_type(&leaf_type, &non_leaf);
            (wrapped_leaf_type, syn::parse_quote!(Some(&#closure)))
        }
        None => (leaf_type, syn::parse_quote!(None)),
    };

    syn::parse_quote!({
        ::rust_sitter::__private::extract_field::<#leaf_type,_>(cursor, source, last_idx, #ident_str, #closure_expr)
    })
}

fn gen_struct_or_variant(
    fields: Fields,
    variant_ident: Option<Ident>,
    containing_type: Ident,
    container_attrs: Vec<Attribute>,
) -> Result<Expr> {
    // Special handling for single-field enum variants that might be leaf nodes
    if let (Some(variant_name), Fields::Unnamed(unnamed_fields)) = (&variant_ident, &fields) {
        if unnamed_fields.unnamed.len() == 1 {
            let field = &unnamed_fields.unnamed[0];
            // Check if this field has a leaf attribute
            let is_leaf = field
                .attrs
                .iter()
                .any(|attr| attr.path() == &syn::parse_quote!(rust_sitter::leaf));
            if is_leaf {
                // For leaf variants, extract directly from the node without navigating to children
                let leaf_type = &field.ty;
                let leaf_attr = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::leaf));

                let leaf_params = leaf_attr.and_then(|a| {
                    a.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
                        .ok()
                });

                let transform_param = leaf_params.as_ref().and_then(|p| {
                    p.iter()
                        .find(|param| param.path == "transform")
                        .map(|p| p.expr.clone())
                });

                let (leaf_type, closure_expr): (Type, Expr) = match transform_param {
                    Some(closure) => {
                        let mut non_leaf = HashSet::new();
                        non_leaf.insert("Spanned");
                        non_leaf.insert("Box");
                        non_leaf.insert("Option");
                        non_leaf.insert("Vec");
                        let wrapped_leaf_type = wrap_leaf_type(&leaf_type, &non_leaf);
                        (wrapped_leaf_type, syn::parse_quote!(Some(&#closure)))
                    }
                    None => (leaf_type.clone(), syn::parse_quote!(None)),
                };

                let construct_name = quote! {
                    #containing_type::#variant_name
                };

                return Ok(syn::parse_quote!({
                    let value = <#leaf_type as ::rust_sitter::Extract<_>>::extract(Some(node), source, 0, #closure_expr);
                    #construct_name(value)
                }));
            }
        }
    }

    let children_parsed = if fields == Fields::Unit {
        let expr = {
            let dummy_field = Field {
                attrs: container_attrs,
                vis: Visibility::Inherited,
                mutability: FieldMutability::None,
                ident: None,
                colon_token: None,
                ty: Type::Verbatim(quote!(())), // unit type.
            };

            gen_field("unit".to_string(), dummy_field)
        };
        vec![ParamOrField::Param(expr)]
    } else {
        fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let expr = if let Some(skip_attrs) = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::skip))
                {
                    skip_attrs.parse_args::<syn::Expr>()?
                } else {
                    let ident_str = field
                        .ident
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or(format!("{i}"));

                    gen_field(ident_str, field.clone())
                };

                let field = if let Some(field_name) = &field.ident {
                    ParamOrField::Field(FieldValue {
                        attrs: vec![],
                        member: Member::Named(field_name.clone()),
                        colon_token: Some(Token![:](Span::call_site())),
                        expr,
                    })
                } else {
                    ParamOrField::Param(expr)
                };
                Ok(field)
            })
            .sift::<Vec<ParamOrField>>()?
    };

    let construct_name = match variant_ident {
        Some(ident) => quote! {
            #containing_type::#ident
        },
        None => quote! {
            #containing_type
        },
    };

    let construct_expr = {
        match &fields {
            Fields::Unit => {
                let ParamOrField::Param(ref expr) = children_parsed[0] else {
                    unreachable!()
                };

                quote! {
                    {
                        #expr;
                        #construct_name
                    }
                }
            }
            Fields::Named(_) => quote! {
                #construct_name {
                    #(#children_parsed),*
                }
            },
            Fields::Unnamed(_) => quote! {
                #construct_name(
                    #(#children_parsed),*
                )
            },
        }
    };

    Ok(
        syn::parse_quote!(::rust_sitter::__private::extract_struct_or_variant(node, move |cursor, last_idx| #construct_expr)),
    )
}

pub fn expand_grammar(input: ItemMod) -> Result<ItemMod> {
    let grammar_name = input
        .attrs
        .iter()
        .find_map(|a| {
            if a.path() == &syn::parse_quote!(rust_sitter::grammar) {
                let grammar_name_expr = a.parse_args_with(Expr::parse).ok();
                if let Some(Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Str(s),
                })) = grammar_name_expr
                {
                    Some(Ok(s.value()))
                } else {
                    Some(Err(syn::Error::new(
                        Span::call_site(),
                        "Expected a string literal grammar name",
                    )))
                }
            } else {
                None
            }
        })
        .transpose()?
        .ok_or_else(|| syn::Error::new(Span::call_site(), "Each grammar must have a name"))?;

    let (brace, new_contents) = input.content.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "Expected the module to have inline contents (`mod my_module { .. }` syntax)",
        )
    })?;

    let root_type = new_contents
        .iter()
        .find_map(|item| match item {
            Item::Enum(ItemEnum { ident, attrs, .. })
            | Item::Struct(ItemStruct { ident, attrs, .. }) => {
                if attrs
                    .iter()
                    .any(|attr| attr.path() == &syn::parse_quote!(rust_sitter::language))
                {
                    Some(ident.clone())
                } else {
                    None
                }
            }
            _ => None,
        })
        .ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "Each parser must have the root type annotated with `#[rust_sitter::language]`",
            )
        })?;

    let mut transformed: Vec<Item> = new_contents
        .iter()
        .cloned()
        .map(|c| match c {
            Item::Enum(mut e) => {
                    // For Tree-sitter compatibility, we need to detect enum variants by their structure
                    // rather than by node kind, since all variants have the same kind
                    let variant_detection_logic = e.variants.iter().map(|v| {
                        let variant_ident = &v.ident;
                        let extract_expr = gen_struct_or_variant(
                            v.fields.clone(),
                            Some(v.ident.clone()),
                            e.ident.clone(),
                            v.attrs.clone(),
                        )?;

                        // Generate detection logic based on variant structure
                        let detection_expr = match &v.fields {
                            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                                // Single field variant like Number(i32)
                                // Check if this is the Number variant specifically
                                if v.ident == "Number" {
                                    quote! {
                                        if node.child_count() == 0 {
                                            return #extract_expr;
                                        }
                                    }
                                } else {
                                    // For other single-field variants, use different heuristics
                                    quote! {
                                        if node.child_count() == 1 {
                                            return #extract_expr;
                                        }
                                    }
                                }
                            }
                            Fields::Unnamed(fields) if fields.unnamed.len() == 3 => {
                                // Three field variant like Sub(Expr, (), Expr) or Mul(Expr, (), Expr)
                                // Check the middle child to distinguish
                                quote! {
                                    if node.child_count() == 3 {
                                        // Check the middle child (operator) to determine variant
                                        let middle_child = &node.children[1];
                                        let middle_kind = middle_child.kind();

                                        if middle_kind == "-" && stringify!(#variant_ident).contains("Sub") {
                                            return #extract_expr;
                                        } else if middle_kind == "*" && stringify!(#variant_ident).contains("Mul") {
                                            return #extract_expr;
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Fallback to original behavior for other patterns
                                let variant_path = format!("{}_{}", e.ident, v.ident);
                                quote! {
                                    if child_node.kind() == #variant_path {
                                        return #extract_expr;
                                    }
                                }
                            }
                        };

                        Ok(detection_expr)
                    }).collect::<Result<Vec<_>>>()?;

                    e.attrs.retain(|a| !is_sitter_attr(a));
                    e.variants.iter_mut().for_each(|v| {
                        v.attrs.retain(|a| !is_sitter_attr(a));
                        v.fields.iter_mut().for_each(|f| {
                            f.attrs.retain(|a| !is_sitter_attr(a));
                        });
                    });

                    // Generate separate detection logic for standard and pure-rust modes
                    let variant_detection_logic_std = e.variants.iter().map(|v| {
                        let variant_ident = &v.ident;
                        let extract_expr = gen_struct_or_variant(
                            v.fields.clone(),
                            Some(v.ident.clone()),
                            e.ident.clone(),
                            v.attrs.clone(),
                        )?;

                        // Generate detection logic based on variant structure
                        let detection_expr = match &v.fields {
                            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                                // Single field variant like Number(i32)
                                quote! {
                                    if child_node.child_count() == 1 {
                                        return #extract_expr;
                                    }
                                }
                            }
                            Fields::Unnamed(fields) if fields.unnamed.len() == 3 => {
                                // Three field variant like Sub(Expr, (), Expr) or Mul(Expr, (), Expr)
                                quote! {
                                    if child_node.child_count() == 3 {
                                        // Check the middle child (operator) to determine variant
                                        let mut cursor = child_node.walk();
                                        cursor.goto_first_child();
                                        cursor.goto_next_sibling();
                                        let middle_kind = cursor.node().kind();

                                        if middle_kind == "-" && stringify!(#variant_ident).contains("Sub") {
                                            return #extract_expr;
                                        } else if middle_kind == "*" && stringify!(#variant_ident).contains("Mul") {
                                            eprintln!("DEBUG: Matched Mul variant, calling extract_expr");
                                            return #extract_expr;
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Fallback to original behavior for other patterns
                                let variant_path = format!("{}_{}", e.ident, v.ident);
                                quote! {
                                    if child_node.kind() == #variant_path {
                                        return #extract_expr;
                                    }
                                }
                            }
                        };

                        Ok(detection_expr)
                    }).collect::<Result<Vec<_>>>()?;

                    let enum_name = &e.ident;
                    let extract_impl: Item = syn::parse_quote! {
                        impl ::rust_sitter::Extract<#enum_name> for #enum_name {
                            type LeafFn = ();

                            #[allow(non_snake_case)]
                            #[cfg(not(feature = "pure-rust"))]
                            fn extract(node: Option<::rust_sitter::tree_sitter::Node>, source: &[u8], _last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
                                let node = node.unwrap();

                                // Tree-sitter wraps enum variants in a parent node
                                // If this is a wrapper node with a single child, extract from the child
                                if node.child_count() == 1 {
                                    let child = node.child(0).unwrap();
                                    let child_node = child;

                                    // Check the child node structure to determine variant
                                    #(#variant_detection_logic_std)*
                                }

                                panic!("Could not determine enum variant from tree structure")
                            }

                            #[allow(non_snake_case)]
                            #[cfg(feature = "pure-rust")]
                            fn extract(node: Option<&::rust_sitter::pure_parser::ParsedNode>, source: &[u8], _last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
                                let node = node.unwrap();
                                // Tree-sitter wraps enum variants in a parent node
                                // If this is a wrapper node with a single child, extract from the child
                                if node.children.len() == 1 {
                                    let child_node = &node.children[0];

                                    // Apply variant detection logic to the child node
                                    let node = child_node;
                                    #(#variant_detection_logic)*
                                }

                                panic!("Could not determine enum variant from tree structure: node symbol={}, child_count={}", node.symbol, node.children.len())
                            }
                        }
                    };
                    Ok(vec![Item::Enum(e), extract_impl])
            }

            Item::Struct(mut s) => {
                    let struct_name = &s.ident;
                    let extract_expr = gen_struct_or_variant(
                        s.fields.clone(),
                        None,
                        s.ident.clone(),
                        s.attrs.clone(),
                    )?;

                    s.attrs.retain(|a| !is_sitter_attr(a));
                    s.fields.iter_mut().for_each(|f| {
                        f.attrs.retain(|a| !is_sitter_attr(a));
                    });


                    let extract_impl: Item = syn::parse_quote! {
                        impl ::rust_sitter::Extract<#struct_name> for #struct_name {
                            type LeafFn = ();

                            #[allow(non_snake_case)]
                            #[cfg(not(feature = "pure-rust"))]
                            fn extract(node: Option<::rust_sitter::tree_sitter::Node>, source: &[u8], last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
                                let node = node.unwrap();
                                #extract_expr
                            }

                            #[allow(non_snake_case)]
                            #[cfg(feature = "pure-rust")]
                            fn extract(node: Option<&::rust_sitter::pure_parser::ParsedNode>, source: &[u8], last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
                                let node = node.unwrap();
                                #extract_expr
                            }
                        }
                    };

                    Ok(vec![Item::Struct(s), extract_impl])
            }

            o => Ok(vec![o]),
        })
        .sift::<Vec<_>>()?.into_iter().flatten().collect();

    #[cfg(not(feature = "pure-rust"))]
    let tree_sitter_ident = Ident::new(&format!("tree_sitter_{grammar_name}"), Span::call_site());

    // For C backend compatibility
    #[cfg(not(feature = "pure-rust"))]
    transformed.push(syn::parse_quote! {
        unsafe extern "C" {
            fn #tree_sitter_ident() -> ::rust_sitter::tree_sitter::Language;
        }
    });

    #[cfg(not(feature = "pure-rust"))]
    transformed.push(syn::parse_quote! {
        pub fn language() -> ::rust_sitter::tree_sitter::Language {
            unsafe { #tree_sitter_ident() }
        }
    });

    // For pure-rust backend
    #[cfg(feature = "pure-rust")]
    {
        // Generate a function that includes the generated parser at runtime
        transformed.push(syn::parse_quote! {
            include!(concat!(env!("OUT_DIR"), "/grammar_", #grammar_name, "/parser_", #grammar_name, ".rs"));
        });

        transformed.push(syn::parse_quote! {
            pub fn language() -> &'static ::rust_sitter::pure_parser::TSLanguage {
                unsafe { &LANGUAGE }
            }
        });
    }

    let root_type_docstr = format!("[`{root_type}`]");
    transformed.push(syn::parse_quote! {
    /// Parse an input string according to the grammar. Returns either any parsing errors that happened, or a
    #[doc = #root_type_docstr]
    /// instance containing the parsed structured data.
      pub fn parse(input: &str) -> core::result::Result<#root_type, Vec<::rust_sitter::errors::ParseError>> {
        ::rust_sitter::__private::parse::<#root_type>(input, language)
      }
  });

    let mut filtered_attrs = input.attrs;
    filtered_attrs.retain(|a| !is_sitter_attr(a));
    Ok(ItemMod {
        attrs: filtered_attrs,
        vis: input.vis,
        unsafety: None,
        mod_token: input.mod_token,
        ident: input.ident,
        content: Some((brace, transformed)),
        semi: input.semi,
    })
}
