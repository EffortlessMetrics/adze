use std::collections::HashSet;

use rust_sitter_common::*;
use serde_json::{Map, Value, json};
use syn::{parse::Parse, punctuated::Punctuated, *};

fn gen_field(
    path: String,
    leaf_type: Type,
    leaf_attrs: Vec<Attribute>,
    word_rule: &mut Option<String>,
    out: &mut Map<String, Value>,
) -> (Value, bool) {
    let leaf_attr = leaf_attrs
        .iter()
        .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::leaf));

    if leaf_attrs
        .iter()
        .any(|attr| attr.path() == &syn::parse_quote!(rust_sitter::word))
    {
        if word_rule.is_some() {
            panic!("Multiple `word` rules specified");
        }

        *word_rule = Some(path.clone());
    }

    let leaf_params = leaf_attr.and_then(|a| {
        a.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
            .ok()
    });

    let pattern_param = leaf_params.as_ref().and_then(|p| {
        p.iter()
            .find(|param| param.path == "pattern")
            .map(|p| p.expr.clone())
    });

    let text_param = leaf_params.as_ref().and_then(|p| {
        p.iter()
            .find(|param| param.path == "text")
            .map(|p| p.expr.clone())
    });

    let mut skip_over = HashSet::new();
    skip_over.insert("Spanned");
    skip_over.insert("Box");

    let (inner_type_vec, is_vec) = try_extract_inner_type(&leaf_type, "Vec", &skip_over);
    let (inner_type_option, is_option) = try_extract_inner_type(&leaf_type, "Option", &skip_over);

    if !is_vec && !is_option {
        if let Some(Expr::Lit(lit)) = pattern_param {
            if let Lit::Str(s) = &lit.lit {
                // Validate that the pattern is not empty
                if s.value().is_empty() {
                    panic!(
                        "Empty patterns are not supported. Token '{}' has an empty pattern value.",
                        path
                    );
                }

                out.insert(
                    path.clone(),
                    json!({
                        "type": "PATTERN",
                        "value": s.value(),
                    }),
                );

                (
                    json!({
                        "type": "SYMBOL",
                        "name": path
                    }),
                    is_option,
                )
            } else {
                panic!("Expected string literal for pattern");
            }
        } else if let Some(Expr::Lit(lit)) = text_param {
            if let Lit::Str(s) = &lit.lit {
                // Allow empty strings for now - they may be used in some grammars
                // to avoid the EmptyString error

                out.insert(
                    path.clone(),
                    json!({
                        "type": "STRING",
                        "value": s.value(),
                    }),
                );

                (
                    json!({
                        "type": "SYMBOL",
                        "name": path
                    }),
                    is_option,
                )
            } else {
                panic!("Expected string literal for text");
            }
        } else {
            let symbol_name = match filter_inner_type(&leaf_type, &skip_over) {
                Type::Path(p) => {
                    if p.path.segments.len() == 1 {
                        p.path.segments[0].ident.to_string()
                    } else {
                        panic!("Expected a single segment path");
                    }
                }
                Type::Tuple(t) if t.elems.is_empty() => {
                    // Unit type () - generate a synthetic name
                    format!("{path}_unit")
                }
                _ => panic!("Expected a path or unit type"),
            };

            (
                json!({
                    "type": "SYMBOL",
                    "name": symbol_name,
                }),
                false,
            )
        }
    } else if is_vec {
        // Check if we need to pass the inner element type name
        let element_path = if path.ends_with("_vec_contents") {
            // This is a recursive call - use the path as-is
            path.clone()
        } else {
            // This is the initial call - we'll generate a _vec_contents rule
            path.clone()
        };

        let (field_json, field_optional) = gen_field(
            element_path,
            inner_type_vec,
            leaf_attrs.clone(),
            word_rule,
            out,
        );

        let delimited_attr = leaf_attrs
            .iter()
            .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::delimited));

        let delimited_params =
            delimited_attr.and_then(|a| a.parse_args_with(FieldThenParams::parse).ok());

        let delimiter_json = delimited_params.map(|p| {
            gen_field(
                format!("{path}_vec_delimiter"),
                p.field.ty,
                p.field.attrs,
                word_rule,
                out,
            )
        });

        let repeat_attr = leaf_attrs
            .iter()
            .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::repeat));

        let repeat_params = repeat_attr.and_then(|a| {
            a.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
                .ok()
        });

        let repeat_non_empty = repeat_params
            .and_then(|p| {
                p.iter()
                    .find(|param| param.path == "non_empty")
                    .map(|p| p.expr.clone())
            })
            .map(|e| e == syn::parse_quote!(true))
            .unwrap_or(false);

        let field_rule_non_optional = json!({
            "type": "FIELD",
            "name": format!("{path}_vec_element"),
            "content": field_json
        });

        let field_rule = if field_optional {
            json!({
                "type": "CHOICE",
                "members": [
                    {
                        "type": "BLANK"
                    },
                    field_rule_non_optional
                ]
            })
        } else {
            field_rule_non_optional
        };

        let vec_contents = if let Some((delimiter_json, delimiter_optional)) = delimiter_json {
            let delim_made_optional = if delimiter_optional {
                json!({
                    "type": "CHOICE",
                    "members": [
                        {
                            "type": "BLANK"
                        },
                        delimiter_json
                    ]
                })
            } else {
                delimiter_json
            };

            json!({
                "type": "SEQ",
                "members": [
                    field_rule,
                    {
                        "type": if field_optional {
                            "REPEAT1"
                        } else {
                            "REPEAT"
                        },
                        "content": {
                            "type": "SEQ",
                            "members": [
                                delim_made_optional,
                                field_rule,
                            ]
                        }
                    }
                ]
            })
        } else {
            // Always use REPEAT1 for the rule definition to avoid empty string issues
            // The empty case is handled by wrapping the reference in CHOICE
            json!({
                "type": "REPEAT1",
                "content": field_rule
            })
        };

        // Always create a named rule with REPEAT1 (never empty)
        let contents_ident = format!("{path}_vec_contents");
        out.insert(contents_ident.clone(), vec_contents);
        
        // Return a reference to the named rule
        // If the Vec can be empty, wrap in CHOICE to make it optional
        let reference = if !repeat_non_empty {
            // Vec can be empty, so make the reference optional
            json!({
                "type": "CHOICE",
                "members": [
                    {
                        "type": "BLANK"
                    },
                    {
                        "type": "SYMBOL",
                        "name": contents_ident,
                    }
                ]
            })
        } else {
            // Vec must have at least one element
            json!({
                "type": "SYMBOL",
                "name": contents_ident,
            })
        };
        
        (reference, false) // Never mark as optional since we handle it in the reference
    } else {
        // is_option
        let (field_json, field_optional) =
            gen_field(path, inner_type_option, leaf_attrs, word_rule, out);

        if field_optional {
            panic!("Option<Option<_>> is not supported");
        }

        (field_json, true)
    }
}

fn gen_struct_or_variant(
    path: String,
    attrs: Vec<Attribute>,
    fields: Fields,
    out: &mut Map<String, Value>,
    word_rule: &mut Option<String>,
) -> Option<Value> {
    // Check if this is a single-leaf variant (enum variant with a single leaf field)
    if let Fields::Unnamed(fields_unnamed) = &fields {
        if fields_unnamed.unnamed.len() == 1 {
            let field = &fields_unnamed.unnamed[0];
            if let Some(leaf_attrs) = field
                .attrs
                .iter()
                .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::leaf))
            {
                // This is a single-leaf variant - return the token directly
                let params = leaf_attrs
                    .parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
                    .ok();
                if let Some(params) = params {
                    if let Some(pattern) = params
                        .iter()
                        .find(|param| param.path == "pattern")
                        .map(|p| p.expr.clone())
                    {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) = pattern
                        {
                            // For single-leaf variants, create a rule with the pattern
                            // Don't return inline - we want a named rule for proper AST nodes
                            out.insert(path, json!({
                                "type": "PATTERN",
                                "value": s.value(),
                            }));
                            return None;
                        }
                    } else if let Some(text) = params
                        .iter()
                        .find(|param| param.path == "text")
                        .map(|p| p.expr.clone())
                    {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) = text
                        {
                            // For single-leaf variants, create a rule with the string
                            // Don't return inline - we want a named rule for proper AST nodes
                            out.insert(path, json!({
                                "type": "STRING",
                                "value": s.value(),
                            }));
                            return None;
                        }
                    }
                }
            }
        }
    }

    fn gen_field_optional(
        path: &str,
        field: &Field,
        word_rule: &mut Option<String>,
        out: &mut Map<String, Value>,
        ident_str: String,
    ) -> Value {
        let (field_contents, is_option) = gen_field(
            format!("{path}_{ident_str}"),
            field.ty.clone(),
            field.attrs.clone(),
            word_rule,
            out,
        );

        let core = json!({
            "type": "FIELD",
            "name": ident_str,
            "content": field_contents
        });

        if is_option {
            json!({
                "type": "CHOICE",
                "members": [
                    {
                        "type": "BLANK"
                    },
                    core
                ]
            })
        } else {
            core
        }
    }

    let children = fields
        .iter()
        .enumerate()
        .filter_map(|(i, field)| {
            if field
                .attrs
                .iter()
                .any(|attr| attr.path() == &syn::parse_quote!(rust_sitter::skip))
            {
                None
            } else {
                // Check for #[rust_sitter::field("name")] attribute
                let field_name = field
                    .attrs
                    .iter()
                    .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::field))
                    .and_then(|attr| attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value()));

                let ident_str = field_name.unwrap_or_else(|| {
                    field
                        .ident
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| {
                            // Generate a deterministic name based on the path and field index
                            // This ensures consistent naming across builds
                            format!("{path}_{i}")
                        })
                });

                Some(gen_field_optional(&path, field, word_rule, out, ident_str))
            }
        })
        .collect::<Vec<Value>>();

    let prec_attr = attrs
        .iter()
        .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::prec));

    let prec_param = prec_attr.and_then(|a| a.parse_args_with(Expr::parse).ok());

    let prec_left_attr = attrs
        .iter()
        .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::prec_left));

    let prec_left_param = prec_left_attr.and_then(|a| a.parse_args_with(Expr::parse).ok());

    let prec_right_attr = attrs
        .iter()
        .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::prec_right));

    let prec_right_param = prec_right_attr.and_then(|a| a.parse_args_with(Expr::parse).ok());

    let base_rule = match fields {
        Fields::Unit => {
            let dummy_field = Field {
                attrs: attrs.clone(),
                vis: Visibility::Inherited,
                mutability: FieldMutability::None,
                ident: None,
                colon_token: None,
                ty: Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: Punctuated::new(),
                }),
            };
            gen_field_optional(&path, &dummy_field, word_rule, out, "unit".to_owned())
        }
        _ => {
            // If all children are optional, we need at least one to be present
            // to avoid the EmptyString error
            if children.is_empty() {
                panic!("Struct {} has no non-skipped fields", path);
            } else if children.len() == 1 {
                // Single field - use it directly
                children.into_iter().next().unwrap()
            } else {
                json!({
                    "type": "SEQ",
                    "members": children
                })
            }
        }
    };

    // Check if this rule could be empty (single optional field)
    let potentially_empty = match &base_rule {
        Value::Object(obj) => {
            obj.get("type").and_then(|t| t.as_str()) == Some("FIELD") &&
            obj.get("content").and_then(|c| c.as_object()).map(|c| {
                c.get("type").and_then(|t| t.as_str()) == Some("CHOICE") &&
                c.get("members").and_then(|m| m.as_array()).map(|m| {
                    m.iter().any(|member| {
                        member.as_object().and_then(|o| o.get("type")).and_then(|t| t.as_str()) == Some("BLANK")
                    })
                }).unwrap_or(false)
            }).unwrap_or(false)
        },
        _ => false
    };
    
    if potentially_empty {
        eprintln!("Warning: Rule '{}' can match empty input. Tree-sitter requires all named rules to match at least one character. Consider adding at least one required field or using 'non_empty = true' for Vec fields.", path);
    }
    
    let rule = if let Some(Expr::Lit(lit)) = prec_param {
        if prec_left_attr.is_some() || prec_right_attr.is_some() {
            panic!("only one of prec, prec_left, and prec_right can be specified");
        }

        if let Lit::Int(i) = &lit.lit {
            json!({
                "type": "PREC",
                "value": i.base10_parse::<u32>().unwrap(),
                "content": base_rule
            })
        } else {
            panic!("Expected integer literal for precedence");
        }
    } else if let Some(Expr::Lit(lit)) = prec_left_param {
        if prec_right_attr.is_some() {
            panic!("only one of prec, prec_left, and prec_right can be specified");
        }

        if let Lit::Int(i) = &lit.lit {
            json!({
                "type": "PREC_LEFT",
                "value": i.base10_parse::<u32>().unwrap(),
                "content": base_rule
            })
        } else {
            panic!("Expected integer literal for precedence");
        }
    } else if let Some(Expr::Lit(lit)) = prec_right_param {
        if let Lit::Int(i) = &lit.lit {
            json!({
                "type": "PREC_RIGHT",
                "value": i.base10_parse::<u32>().unwrap(),
                "content": base_rule
            })
        } else {
            panic!("Expected integer literal for precedence");
        }
    } else {
        base_rule
    };

    out.insert(path, rule);
    None // Return None for non-single-leaf variants
}

pub fn generate_grammar(module: &ItemMod) -> Value {
    let mut rules_map = Map::new();
    // for some reason, source_file must be the first key for things to work
    // We'll insert it after we find the root type

    let mut extras_list = vec![];
    let mut externals_list = vec![];

    let grammar_name = module
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
                    Some(s.value())
                } else {
                    panic!("Expected string literal for grammar name");
                }
            } else {
                None
            }
        })
        .expect("Each grammar must have a name");

    let (_, contents) = module.content.as_ref().unwrap();

    let root_type = contents
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
        .expect("Each parser must have the root type annotated with `#[rust_sitter::language]`")
        .to_string();

    // Insert source_file rule that references the root type
    rules_map.insert(
        "source_file".to_string(),
        json!({
            "type": "SYMBOL",
            "name": root_type.to_string()
        }),
    );

    // Optionally locate the rule annotated with `#[rust_sitter::word]`.
    let mut word_rule = None;
    contents.iter().for_each(|c| {
        let (symbol, attrs) = match c {
            Item::Enum(e) => {
                let mut members: Vec<Value> = vec![];

                e.variants.iter().for_each(|v| {
                    let variant_path = format!("{}_{}", e.ident, v.ident);

                    // Generate the variant rule
                    let _variant_result = gen_struct_or_variant(
                        variant_path.clone(),
                        v.attrs.clone(),
                        v.fields.clone(),
                        &mut rules_map,
                        &mut word_rule,
                    );
                    
                    
                    // Always reference the variant by name, even for single-leaf variants
                    // This ensures we get proper node names in the parse tree
                    let variant_ref = json!({
                        "type": "SYMBOL",
                        "name": variant_path.clone()
                    });
                    
                    // Check if this variant has precedence
                    let prec_attr = v.attrs
                        .iter()
                            .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::prec));
                        let prec_left_attr = v.attrs
                            .iter()
                            .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::prec_left));
                        let prec_right_attr = v.attrs
                            .iter()
                            .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::prec_right));
                        
                        // Apply precedence if specified on the variant
                        let member = if let Some(attr) = prec_attr {
                            if let Ok(Expr::Lit(expr_lit)) = attr.parse_args_with(Expr::parse) {
                                if let Lit::Int(i) = &expr_lit.lit {
                                    json!({
                                        "type": "PREC",
                                        "value": i.base10_parse::<i32>().unwrap(),
                                        "content": variant_ref.clone()
                                    })
                                } else {
                                    variant_ref.clone()
                                }
                            } else {
                                variant_ref.clone()
                            }
                        } else if let Some(attr) = prec_left_attr {
                            if let Ok(Expr::Lit(expr_lit)) = attr.parse_args_with(Expr::parse) {
                                if let Lit::Int(i) = &expr_lit.lit {
                                    json!({
                                        "type": "PREC_LEFT",
                                        "value": i.base10_parse::<i32>().unwrap(),
                                        "content": variant_ref.clone()
                                    })
                                } else {
                                    variant_ref.clone()
                                }
                            } else {
                                variant_ref.clone()
                            }
                        } else if let Some(attr) = prec_right_attr {
                            if let Ok(Expr::Lit(expr_lit)) = attr.parse_args_with(Expr::parse) {
                                if let Lit::Int(i) = &expr_lit.lit {
                                    json!({
                                        "type": "PREC_RIGHT",
                                        "value": i.base10_parse::<i32>().unwrap(),
                                        "content": variant_ref.clone()
                                    })
                                } else {
                                    variant_ref.clone()
                                }
                            } else {
                                variant_ref.clone()
                            }
                        } else {
                            variant_ref.clone()
                        };
                        
                        members.push(member);
                });

                // For enums, we want the choice to be transparent in the parse tree.
                // The variants should appear directly without a wrapper node.
                // Tree-sitter convention: rules starting with _ are hidden.
                
                // Create a hidden rule for the enum CHOICE
                let hidden_rule_name = format!("_{}", e.ident);
                let rule = json!({
                    "type": "CHOICE",
                    "members": members
                });
                
                // Insert the hidden CHOICE rule
                rules_map.insert(hidden_rule_name.clone(), rule);
                
                // Create a visible rule that references the hidden one
                // This allows the enum to be referenced in the grammar while keeping it transparent
                rules_map.insert(e.ident.to_string(), json!({
                    "type": "SYMBOL",
                    "name": hidden_rule_name
                }));

                (e.ident.to_string(), e.attrs.clone())
            }

            Item::Struct(s) => {
                // Check if this is an external token first
                let is_external = s
                    .attrs
                    .iter()
                    .any(|a| a.path() == &syn::parse_quote!(rust_sitter::external));

                // Check if this is an extra token
                let is_extra = s
                    .attrs
                    .iter()
                    .any(|a| a.path() == &syn::parse_quote!(rust_sitter::extra));

                // Check if this is the word token
                let is_word = s
                    .attrs
                    .iter()
                    .any(|a| a.path() == &syn::parse_quote!(rust_sitter::word));

                if is_word {
                    if word_rule.is_some() {
                        panic!("Multiple `word` rules specified");
                    }
                    word_rule = Some(s.ident.to_string());
                }

                // Generate rules for non-external structs AND extra structs (even if they're not referenced)
                if !is_external || is_extra {
                    let _ = gen_struct_or_variant(
                        s.ident.to_string(),
                        s.attrs.clone(),
                        s.fields.clone(),
                        &mut rules_map,
                        &mut word_rule,
                    );
                }

                (s.ident.to_string(), s.attrs.clone())
            }

            _ => return,
        };

        if attrs
            .iter()
            .any(|a| a.path() == &syn::parse_quote!(rust_sitter::extra))
        {
            // For extras, we want to reference the generated rule directly
            // The Whitespace struct generates a rule like "Whitespace" which contains the pattern
            extras_list.push(json!({
                "type": "SYMBOL",
                "name": symbol
            }));
        }

        if attrs
            .iter()
            .any(|a| a.path() == &syn::parse_quote!(rust_sitter::external))
        {
            externals_list.push(json!({
                "type": "SYMBOL",
                "name": symbol
            }));
        }
    });

    // source_file rule already inserted above - don't overwrite it!

    let mut grammar = json!({
        "name": grammar_name,
        "word": word_rule,
        "rules": rules_map,
        "extras": extras_list
    });

    // Only include externals if there are any
    if !externals_list.is_empty() {
        grammar
            .as_object_mut()
            .unwrap()
            .insert("externals".to_string(), json!(externals_list));
    }

    
    grammar
}
