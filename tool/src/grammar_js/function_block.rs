//! Function-block parsing helpers for grammar.js arrow-function rules.
//!
//! The top-level parser is responsible for parsing grammar rule syntax. This
//! module only understands the JavaScript block wrapper around a returned rule:
//! it records simple inline helper declarations, finds the returned expression,
//! and expands calls to those helpers.

use anyhow::{Result, bail};
use indexmap::IndexMap;

#[cfg(not(debug_assertions))]
macro_rules! eprintln {
    ($($arg:tt)*) => {};
}

#[cfg(debug_assertions)]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        if std::env::var("RUST_LOG")
            .ok()
            .unwrap_or_default()
            .contains("debug")
        {
            std::eprintln!($($arg)*);
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InlineHelper {
    param_name: String,
    body: String,
}

/// Extracts a grammar rule expression from a JavaScript function block.
pub(super) fn extract_return_rule_expression(block: &str) -> Result<Option<String>> {
    if is_table_definition(block) {
        eprintln!("Warning: Complex JavaScript table definition found, returning placeholder");
        return Ok(None);
    }

    let helpers = collect_inline_helpers(block);
    let mut return_expr = extract_return_expression(block)?;
    expand_inline_helpers(&mut return_expr, &helpers);
    Ok(Some(return_expr))
}

fn is_table_definition(block: &str) -> bool {
    block.contains("const table = [")
}

fn collect_inline_helpers(block: &str) -> IndexMap<String, InlineHelper> {
    block
        .lines()
        .filter_map(parse_inline_helper_declaration)
        .collect()
}

fn parse_inline_helper_declaration(line: &str) -> Option<(String, InlineHelper)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("const ") {
        return None;
    }

    let eq_pos = trimmed.find('=')?;
    let helper_name = trimmed[6..eq_pos].trim();
    let rhs = trimmed[eq_pos + 1..].trim();
    let arrow_pos = rhs.find("=>")?;
    let params = rhs[..arrow_pos].trim();
    let body = rhs[arrow_pos + 2..].trim();
    let param_name = params.trim_matches(|c: char| c == '(' || c == ')' || c.is_whitespace());
    let body = body.trim_end_matches(';').trim();

    eprintln!(
        "Debug: Registered helper '{}' with param '{}' and body '{}'",
        helper_name, param_name, body
    );

    Some((
        helper_name.to_string(),
        InlineHelper {
            param_name: param_name.to_string(),
            body: body.to_string(),
        },
    ))
}

fn extract_return_expression(block: &str) -> Result<String> {
    let Some(return_pos) = block.rfind("return ") else {
        bail!("Function block must contain a return statement");
    };

    let return_content = &block[return_pos + 7..];
    let end_pos = find_return_statement_end(return_content);
    Ok(return_content[..end_pos].trim().to_string())
}

fn find_return_statement_end(return_content: &str) -> usize {
    let mut end_pos = return_content.len();
    let mut depth = 0;
    let mut in_string = false;
    let mut in_regex = false;
    let mut escape_next = false;

    for (i, ch) in return_content.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
            continue;
        }

        if !in_regex && (ch == '"' || ch == '\'') {
            in_string = !in_string;
        }

        if !in_string && ch == '/' {
            in_regex = !in_regex;
        }

        if !in_string && !in_regex {
            match ch {
                '(' | '{' | '[' => depth += 1,
                ')' | '}' | ']' => depth -= 1,
                ';' if depth == 0 => {
                    end_pos = i;
                    break;
                }
                _ => {}
            }
        }

        if depth < 0 {
            end_pos = i;
            break;
        }
    }

    end_pos
}

fn expand_inline_helpers(return_expr: &mut String, helpers: &IndexMap<String, InlineHelper>) {
    for (helper_name, helper) in helpers {
        expand_inline_helper(return_expr, helper_name, helper);
    }
}

fn expand_inline_helper(return_expr: &mut String, helper_name: &str, helper: &InlineHelper) {
    let call_pattern = format!("{}(", helper_name);
    if !return_expr.contains(&call_pattern) {
        return;
    }

    eprintln!(
        "Debug: Expanding helper '{}' in return expression",
        helper_name
    );

    let Some(call_start) = return_expr.find(&call_pattern) else {
        return;
    };

    let args_start = call_start + call_pattern.len();
    let Some(arg_end) = find_call_argument_end(return_expr, args_start) else {
        return;
    };

    let arg = &return_expr[args_start..arg_end];
    eprintln!("Debug: Helper argument: '{}'", arg);

    let expanded = helper.body.replace(&helper.param_name, arg);
    eprintln!("Debug: Expanded body: '{}'", expanded);

    let call_expr = return_expr[call_start..=arg_end].to_string();
    *return_expr = return_expr.replace(&call_expr, &expanded);
    eprintln!("Debug: New return expression: '{}'", return_expr);
}

fn find_call_argument_end(return_expr: &str, args_start: usize) -> Option<usize> {
    let mut depth = 1;
    let chars: Vec<char> = return_expr.chars().collect();

    for (i, ch) in chars.iter().enumerate().skip(args_start) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::extract_return_rule_expression;

    #[test]
    fn extracts_return_expression_until_top_level_semicolon() {
        let block = r#"{
            return seq($.left, choice(';', /a;b/), $.right);
        }"#;

        let expr = extract_return_rule_expression(block).unwrap().unwrap();

        assert_eq!(expr, "seq($.left, choice(';', /a;b/), $.right)");
    }

    #[test]
    fn expands_simple_inline_helper_call() {
        let block = r#"{
            const commaSep = (rule) => seq(rule, repeat(seq(',', rule)));
            return commaSep($.item);
        }"#;

        let expr = extract_return_rule_expression(block).unwrap().unwrap();

        assert_eq!(expr, "seq($.item, repeat(seq(',', $.item)))");
    }

    #[test]
    fn reports_table_definitions_as_unsupported_expressions() {
        let block = r#"{
            const table = [[$.a, $.b]];
            return choice(...table.map(([left, right]) => seq(left, right)));
        }"#;

        let expr = extract_return_rule_expression(block).unwrap();

        assert!(expr.is_none());
    }
}
