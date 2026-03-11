//! Traversal utilities for discovering `#[adze::grammar]` modules in a syntax tree.

#![forbid(unsafe_op_in_unsafe_fn)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use syn::{Attribute, Item, ItemMod};

/// Return every inline module annotated with `#[adze::grammar]` under `items`, recursively.
pub fn collect_grammar_modules(items: &[Item]) -> Vec<&ItemMod> {
    let mut modules = Vec::new();
    for item in items {
        collect_from_item(item, &mut modules);
    }
    modules
}

fn collect_from_item<'a>(item: &'a Item, out: &mut Vec<&'a ItemMod>) {
    let Item::Mod(module) = item else {
        return;
    };

    if has_adze_grammar_attr(&module.attrs) {
        out.push(module);
    }

    if let Some((_, nested)) = &module.content {
        for nested_item in nested {
            collect_from_item(nested_item, out);
        }
    }
}

fn has_adze_grammar_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        path.segments.len() == 2
            && path.segments[0].ident == "adze"
            && path.segments[1].ident == "grammar"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn finds_nested_grammar_modules() {
        let items: Vec<Item> = vec![parse_quote! {
            mod outer {
                #[adze::grammar]
                mod g1 {}

                mod inner {
                    #[adze::grammar]
                    mod g2 {}
                }
            }
        }];

        let modules = collect_grammar_modules(&items);
        let names: Vec<_> = modules.iter().map(|m| m.ident.to_string()).collect();

        assert_eq!(names, vec!["g1", "g2"]);
    }

    #[test]
    fn ignores_non_grammar_modules() {
        let items: Vec<Item> = vec![parse_quote! {
            mod outer {
                #[adze::language]
                mod not_grammar {}
            }
        }];

        assert!(collect_grammar_modules(&items).is_empty());
    }
}
