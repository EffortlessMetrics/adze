use std::collections::HashSet;

use rust_sitter_common::*;
use serde_json::{json, Map, Value};
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
        let (field_json, field_optional) = gen_field(
            path.clone(),
            inner_type_vec,
            leaf_attr.iter().cloned().cloned().collect(),
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
            json!({
                "type": "REPEAT1",
                "content": field_rule
            })
        };

        let contents_ident = format!("{path}_vec_contents");
        out.insert(contents_ident.clone(), vec_contents);

        (
            json!({
                "type": "SYMBOL",
                "name": contents_ident,
            }),
            !repeat_non_empty,
        )
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
    eprintln!("DEBUG gen_struct_or_variant: path={}, fields={:?}", path, fields);
    
    // Check if this is a single-leaf variant (enum variant with a single leaf field)
    if let Fields::Unnamed(fields_unnamed) = &fields {
        if fields_unnamed.unnamed.len() == 1 {
            let field = &fields_unnamed.unnamed[0];
            if let Some(leaf_attrs) = field.attrs.iter().find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::leaf)) {
                // This is a single-leaf variant - return the token directly
                let params = leaf_attrs.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated).ok();
                if let Some(params) = params {
                    if let Some(pattern) = params.iter()
                        .find(|param| param.path == "pattern")
                        .map(|p| p.expr.clone()) {
                        if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = pattern {
                            // Return a PATTERN rule directly
                            return Some(json!({
                                "type": "PATTERN",
                                "value": s.value(),
                            }));
                        }
                    } else if let Some(text) = params.iter()
                        .find(|param| param.path == "text")
                        .map(|p| p.expr.clone()) {
                        if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = text {
                            // Return a STRING rule directly
                            return Some(json!({
                                "type": "STRING",
                                "value": s.value(),
                            }));
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
                let field_name = field.attrs.iter()
                    .find(|attr| attr.path() == &syn::parse_quote!(rust_sitter::field))
                    .and_then(|attr| {
                        attr.parse_args::<syn::LitStr>().ok()
                            .map(|lit| lit.value())
                    });
                
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
        _ => json!({
            "type": "SEQ",
            "members": children
        }),
    };

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
    None  // Return None for non-single-leaf variants
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
    rules_map.insert("source_file".to_string(), json!({
        "type": "SYMBOL",
        "name": root_type.to_string()
    }));

    // Optionally locate the rule annotated with `#[rust_sitter::word]`.
    let mut word_rule = None;
    contents.iter().for_each(|c| {
        let (symbol, attrs) = match c {
            Item::Enum(e) => {
                let mut members: Vec<Value> = vec![];
                
                e.variants.iter().for_each(|v| {
                    let variant_path = format!("{}_{}", e.ident, v.ident);
                    
                    // Try to inline single-leaf variants
                    if let Some(inlined_rule) = gen_struct_or_variant(
                        variant_path.clone(),
                        v.attrs.clone(),
                        v.fields.clone(),
                        &mut rules_map,
                        &mut word_rule,
                    ) {
                        // This is a single-leaf variant - use the token directly
                        members.push(inlined_rule);
                    } else {
                        // This is a complex variant - reference the generated rule
                        members.push(json!({
                            "type": "SYMBOL",
                            "name": variant_path
                        }));
                    }
                });

                let rule = json!({
                    "type": "CHOICE",
                    "members": members
                });

                rules_map.insert(e.ident.to_string(), rule);

                (e.ident.to_string(), e.attrs.clone())
            }

            Item::Struct(s) => {
                // Check if this is an external token first
                let is_external = s.attrs
                    .iter()
                    .any(|a| a.path() == &syn::parse_quote!(rust_sitter::external));
                
                // Check if this is an extra token
                let is_extra = s.attrs
                    .iter()
                    .any(|a| a.path() == &syn::parse_quote!(rust_sitter::extra));
                
                // Check if this is the word token
                let is_word = s.attrs
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
            eprintln!("DEBUG expansion: Found rust_sitter::extra on symbol '{}'", symbol);
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

    eprintln!("DEBUG expansion: extras_list = {:?}", extras_list);
    eprintln!("DEBUG expansion: rules_map keys = {:?}", rules_map.keys().collect::<Vec<_>>());
    let mut grammar = json!({
        "name": grammar_name,
        "word": word_rule,
        "rules": rules_map,
        "extras": extras_list
    });

    // Only include externals if there are any
    if !externals_list.is_empty() {
        grammar.as_object_mut().unwrap().insert(
            "externals".to_string(),
            json!(externals_list)
        );
    }

    grammar
}
