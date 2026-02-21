use std::collections::HashSet;

use crate::errors::IteratorExt as _;
use adze_common::*;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{parse::Parse, punctuated::Punctuated, *};

fn is_sitter_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .iter()
        .next()
        .map(|segment| segment.ident == "adze")
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
        .find(|attr| attr.path() == &syn::parse_quote!(adze::leaf));

    let leaf_params = leaf_attr.and_then(|a| {
        a.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
            .ok()
    });

    let transform_param = leaf_params.as_ref().and_then(|p| {
        p.iter()
            .find(|param| param.path == "transform")
            .map(|p| p.expr.clone())
    });

    // ❌ CRITICAL BUG: Transform functions are captured but never executed! (Issue #74)
    //
    // PROBLEM: This code generates the macro expansion for transform functions like:
    //   #[adze::leaf(pattern = r"\d+", transform = |s| s.parse::<i32>().unwrap())]
    // But the generated code only captures the closure, it doesn't actually CALL it!
    //
    // CURRENT FLOW:
    // 1. User writes: transform = |s| s.parse::<i32>().unwrap()
    // 2. This code captures it as: Some(&#closure)
    // 3. Generated parser code gets closure but never executes it
    // 4. Result: Raw text is used instead of transformed value
    //
    // IMPACT:
    // - Numbers remain as strings instead of i32
    // - Custom transforms for dates, enums, etc. don't work
    // - Grammar definitions look like they support transforms but silently fail
    // - Combined with lexer issue #74, creates completely broken parsing
    //
    // TODO (HIGH PRIORITY - Issue #74):
    // Generate code that actually EXECUTES the transform:
    // ```rust
    // let raw_text = extract_raw_text(cursor, source);
    // match transform_fn(raw_text) {
    //     Ok(transformed) => transformed,
    //     Err(e) => return Err(ParseError::TransformFailed(e))
    // }
    // ```
    //
    // REQUIRED CHANGES:
    // 1. Generate transform execution code instead of just capturing closure
    // 2. Add proper error handling for transform failures
    // 3. Ensure type safety between pattern match and transform result
    // 4. Test with actual transform functions to verify execution
    //
    let (leaf_type, closure_expr): (Type, Expr) = match transform_param {
        Some(closure) => {
            let mut non_leaf = HashSet::new();
            non_leaf.insert("Spanned");
            non_leaf.insert("Box");
            non_leaf.insert("Option");
            non_leaf.insert("Vec");
            let wrapped_leaf_type = wrap_leaf_type(&leaf_type, &non_leaf);
            // ❌ BUG: This just captures the closure, doesn't execute it!
            (wrapped_leaf_type, syn::parse_quote!(Some(&#closure)))
        }
        None => (leaf_type, syn::parse_quote!(None)),
    };

    syn::parse_quote!({
        ::adze::__private::extract_field::<#leaf_type,_>(cursor, source, last_idx, #ident_str, #closure_expr)
    })
}

fn gen_struct_or_variant(
    fields: Fields,
    variant_ident: Option<Ident>,
    containing_type: Ident,
    container_attrs: Vec<Attribute>,
) -> Result<Expr> {
    // Special handling for single-field enum variants that might be leaf nodes
    if let (Some(variant_name), Fields::Unnamed(unnamed_fields)) = (&variant_ident, &fields)
        && unnamed_fields.unnamed.len() == 1
    {
        let field = &unnamed_fields.unnamed[0];
        // Check if this field has a leaf attribute
        let is_leaf = field
            .attrs
            .iter()
            .any(|attr| attr.path() == &syn::parse_quote!(adze::leaf));
        if is_leaf {
            // For leaf variants, extract directly from the node without navigating to children
            let leaf_type = &field.ty;
            let leaf_attr = field
                .attrs
                .iter()
                .find(|attr| attr.path() == &syn::parse_quote!(adze::leaf));

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
                    let wrapped_leaf_type = wrap_leaf_type(leaf_type, &non_leaf);
                    (wrapped_leaf_type, syn::parse_quote!(Some(&#closure)))
                }
                None => (leaf_type.clone(), syn::parse_quote!(None)),
            };

            let construct_name = quote! {
                #containing_type::#variant_name
            };

            return Ok(syn::parse_quote!({
                let value = <#leaf_type as ::adze::Extract<_>>::extract(Some(node), source, 0, #closure_expr);
                #construct_name(value)
            }));
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
                    .find(|attr| attr.path() == &syn::parse_quote!(adze::skip))
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
        syn::parse_quote!(::adze::__private::extract_struct_or_variant(node, move |cursor, last_idx| #construct_expr)),
    )
}

pub fn expand_grammar(input: ItemMod) -> Result<ItemMod> {
    let grammar_name = input
        .attrs
        .iter()
        .find_map(|a| {
            if a.path() == &syn::parse_quote!(adze::grammar) {
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
                    .any(|attr| attr.path() == &syn::parse_quote!(adze::language))
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
                "Each parser must have the root type annotated with `#[adze::language]`",
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
                        let extract_expr = gen_struct_or_variant(
                            v.fields.clone(),
                            Some(v.ident.clone()),
                            e.ident.clone(),
                            v.attrs.clone(),
                        )?;

                        // Generate detection logic using symbol names
                        let enum_name = e.ident.to_string();
                        let variant_name = v.ident.to_string();
                        let expected_symbol = format!("{}_{}", enum_name, variant_name);

                        let detection_expr = quote! {
                            if node.kind() == #expected_symbol {
                                return #extract_expr;
                            }
                        };

                        Ok(detection_expr)
                    }).collect::<Result<Vec<_>>>()?;

                    // Generate separate detection logic for leaf variants on terminal nodes
                    let leaf_variant_detection = e.variants.iter().filter_map(|v| {
                        // Check if this is a leaf variant
                        let is_leaf_variant = if let Fields::Unnamed(ref unnamed) = v.fields {
                            unnamed.unnamed.len() == 1 && unnamed.unnamed[0]
                                .attrs
                                .iter()
                                .any(|attr| attr.path() == &syn::parse_quote!(adze::leaf))
                        } else {
                            false
                        };

                        if is_leaf_variant {
                            let extract_expr = gen_struct_or_variant(
                                v.fields.clone(),
                                Some(v.ident.clone()),
                                e.ident.clone(),
                                v.attrs.clone(),
                            ).ok()?;

                            // For Number variant specifically, handle numeric terminals
                            if v.ident == "Number" {
                                Some(quote! {
                                    // Number terminals often have generated names like _1, _2, etc
                                    return #extract_expr;
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>();

                    e.attrs.retain(|a| !is_sitter_attr(a));
                    e.variants.iter_mut().for_each(|v| {
                        v.attrs.retain(|a| !is_sitter_attr(a));
                        v.fields.iter_mut().for_each(|f| {
                            f.attrs.retain(|a| !is_sitter_attr(a));
                        });
                    });

                    // Generate separate detection logic for standard and pure-rust modes
                    let variant_detection_logic_std = e.variants.iter().map(|v| {
                        let extract_expr = gen_struct_or_variant(
                            v.fields.clone(),
                            Some(v.ident.clone()),
                            e.ident.clone(),
                            v.attrs.clone(),
                        )?;

                        // Generate detection logic using symbol names
                        let enum_name = e.ident.to_string();
                        let variant_name = v.ident.to_string();
                        let expected_symbol = format!("{}_{}", enum_name, variant_name);

                        let detection_expr = quote! {
                            if child_node.kind() == #expected_symbol {
                                return #extract_expr;
                            }
                        };

                        Ok(detection_expr)
                    }).collect::<Result<Vec<_>>>()?;

                    let enum_name = &e.ident;
                    let extract_impl: Item = syn::parse_quote! {
                        impl ::adze::Extract<#enum_name> for #enum_name {
                            type LeafFn = ();

                            #[cfg(feature = "pure-rust")]
                            const GRAMMAR_NAME: &'static str = GRAMMAR_NAME;

                            #[allow(non_snake_case)]
                            #[cfg(not(feature = "pure-rust"))]
                            fn extract(node: Option<::adze::tree_sitter::Node>, source: &[u8], _last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
                                let node = node.unwrap();

                                // Recursively unwrap hidden rules and wrapper nodes
                                fn unwrap_hidden_rules(node: ::adze::tree_sitter::Node) -> ::adze::tree_sitter::Node {
                                    // If this is a hidden rule (starts with '_') or has a single child, unwrap it
                                    if (node.kind().starts_with('_') || node.child_count() == 1) && node.child_count() > 0 {
                                        if let Some(child) = node.child(0) {
                                            return unwrap_hidden_rules(child);
                                        }
                                    }
                                    node
                                }

                                let unwrapped_node = unwrap_hidden_rules(node);

                                // Check if this is an enum wrapper node (e.g., "Expression" wrapping "Expression_Number")
                                if unwrapped_node.kind() == stringify!(#enum_name) && unwrapped_node.child_count() == 1 {
                                    // This is a wrapper node, extract from its child
                                    let child = unwrapped_node.child(0);
                                    return Self::extract(child, source, _last_idx, _leaf_fn);
                                }

                                // Check the unwrapped node structure to determine variant
                                let child_node = unwrapped_node;
                                #(#variant_detection_logic_std)*

                                panic!("Could not determine enum variant from tree structure: node kind='{}', child_count={}",
                                    unwrapped_node.kind(), unwrapped_node.child_count())
                            }

                            #[allow(non_snake_case)]
                            #[cfg(feature = "pure-rust")]
                            fn extract(node: Option<&::adze::pure_parser::ParsedNode>, source: &[u8], _last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
                                let node = node.unwrap();

                                // Recursively unwrap hidden rules and wrapper nodes
                                fn unwrap_hidden_rules<'a>(node: &'a ::adze::pure_parser::ParsedNode) -> &'a ::adze::pure_parser::ParsedNode {
                                    // If this is a hidden rule (starts with '_') or has a single child, unwrap it
                                    if (node.kind().starts_with('_') || node.children.len() == 1) && node.children.len() > 0 {
                                        return unwrap_hidden_rules(&node.children[0]);
                                    }
                                    node
                                }

                                let unwrapped_node = unwrap_hidden_rules(node);

                                // Check if this is an enum wrapper node (e.g., "Expression" wrapping "Expression_Number")
                                if unwrapped_node.kind() == stringify!(#enum_name) && unwrapped_node.children.len() == 1 {
                                    // This is a wrapper node, extract from its child
                                    let child_node = &unwrapped_node.children[0];
                                    return Self::extract(Some(child_node), source, _last_idx, _leaf_fn);
                                }

                                // Check if this node directly matches one of our variants
                                let node = unwrapped_node;
                                #(#variant_detection_logic)*

                                // Special handling for terminal nodes that represent leaf variants
                                if unwrapped_node.children.is_empty() {
                                    // This is a terminal node, it might be a leaf variant
                                    let node = unwrapped_node;
                                    #(#leaf_variant_detection)*
                                }

                                panic!("Could not determine enum variant from tree structure: node kind='{}', symbol={}, child_count={}",
                                    unwrapped_node.kind(), unwrapped_node.symbol, unwrapped_node.children.len())
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


                    // Generate different extract expressions for pure-rust vs non-pure-rust
                    // because pure-rust uses &ParsedNode while non-pure-rust uses Node by value
                    let extract_expr_std = extract_expr.clone();

                    // Always generate both versions - the cfg will be evaluated when the generated code is compiled
                    let extract_impl: Item = syn::parse_quote! {
                        impl ::adze::Extract<#struct_name> for #struct_name {
                            type LeafFn = ();

                            #[cfg(feature = "pure-rust")]
                            const GRAMMAR_NAME: &'static str = GRAMMAR_NAME;

                            #[allow(non_snake_case)]
                            #[cfg(not(feature = "pure-rust"))]
                            fn extract(node: Option<::adze::tree_sitter::Node>, source: &[u8], last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
                                let node = node.unwrap();
                                #extract_expr_std
                            }

                            #[allow(non_snake_case)]
                            #[cfg(feature = "pure-rust")]
                            fn extract(node: Option<&::adze::pure_parser::ParsedNode>, source: &[u8], last_idx: usize, _leaf_fn: Option<&Self::LeafFn>) -> Self {
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

    // Generate both backend implementations with cfg attributes in the output
    let tree_sitter_ident = Ident::new(&format!("tree_sitter_{grammar_name}"), Span::call_site());

    // For C backend compatibility
    transformed.push(syn::parse_quote! {
        #[cfg(not(feature = "pure-rust"))]
        unsafe extern "C" {
            fn #tree_sitter_ident() -> ::adze::tree_sitter::Language;
        }
    });

    transformed.push(syn::parse_quote! {
        #[cfg(not(feature = "pure-rust"))]
        pub fn language() -> ::adze::tree_sitter::Language {
            unsafe { #tree_sitter_ident() }
        }
    });

    // For pure-rust backend
    transformed.push(syn::parse_quote! {
        #[cfg(feature = "pure-rust")]
        include!(concat!(env!("OUT_DIR"), "/grammar_", #grammar_name, "/parser_", #grammar_name, ".rs"));
    });

    transformed.push(syn::parse_quote! {
        #[cfg(feature = "pure-rust")]
        pub fn language() -> &'static ::adze::pure_parser::TSLanguage {
            unsafe { &LANGUAGE }
        }
    });

    let root_type_docstr = format!("[`{root_type}`]");

    // Generate parse function for non-pure-rust
    transformed.push(syn::parse_quote! {
        /// Parse an input string according to the grammar. Returns either any parsing errors that happened, or a
        #[doc = #root_type_docstr]
        /// instance containing the parsed structured data.
        #[cfg(not(feature = "pure-rust"))]
        pub fn parse(input: &str) -> core::result::Result<#root_type, Vec<::adze::errors::ParseError>> {
            ::adze::__private::parse::<#root_type>(input, language)
        }
    });

    // Generate parse function for pure-rust
    transformed.push(syn::parse_quote! {
        /// Parse an input string according to the grammar. Returns either any parsing errors that happened, or a
        #[doc = #root_type_docstr]
        /// instance containing the parsed structured data.
        #[cfg(feature = "pure-rust")]
        pub fn parse(input: &str) -> core::result::Result<#root_type, Vec<::adze::errors::ParseError>> {
            ::adze::__private::parse::<#root_type>(input, language)
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
