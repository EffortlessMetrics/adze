use anyhow::{Result, bail};
use indexmap::IndexMap;

/// Parsed result for JavaScript block-bodied grammar rules.
pub(super) enum ParsedFunctionBlock {
    /// Complex executable JavaScript that cannot be represented statically yet.
    PlaceholderChoice,
    /// A statically extracted return expression ready for rule parsing.
    ReturnExpression(String),
}

#[derive(Debug)]
struct InlineHelper {
    param_name: String,
    body: String,
}

/// Extracts the static rule expression from a JavaScript function block.
pub(super) fn parse(block: &str) -> Result<ParsedFunctionBlock> {
    eprintln!("Debug: parse_function_block called with block:\n{}", block);

    if has_table_driven_definition(block) {
        eprintln!("Warning: Complex JavaScript table definition found, returning placeholder");
        return Ok(ParsedFunctionBlock::PlaceholderChoice);
    }

    let helpers = extract_inline_helpers(block);
    let return_expr = extract_return_expression(block)?;
    Ok(ParsedFunctionBlock::ReturnExpression(
        expand_inline_helpers(return_expr, &helpers),
    ))
}

fn has_table_driven_definition(block: &str) -> bool {
    block.contains("const table = [")
}

fn extract_inline_helpers(block: &str) -> IndexMap<String, InlineHelper> {
    let mut helpers = IndexMap::new();

    for line in block.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("const ") {
            continue;
        }

        let Some(eq_pos) = trimmed.find('=') else {
            continue;
        };

        let helper_name = trimmed[6..eq_pos].trim();
        let rhs = trimmed[eq_pos + 1..].trim();
        let Some(arrow_pos) = rhs.find("=>") else {
            continue;
        };

        let params = rhs[..arrow_pos].trim();
        let body = rhs[arrow_pos + 2..].trim().trim_end_matches(';').trim();
        let param_name = params.trim_matches(|c: char| c == '(' || c == ')' || c.is_whitespace());

        helpers.insert(
            helper_name.to_string(),
            InlineHelper {
                param_name: param_name.to_string(),
                body: body.to_string(),
            },
        );
        eprintln!(
            "Debug: Registered helper '{}' with param '{}' and body '{}'",
            helper_name, param_name, body
        );
    }

    helpers
}

fn extract_return_expression(block: &str) -> Result<&str> {
    let Some(return_pos) = block.rfind("return ") else {
        bail!("Function block must contain a return statement")
    };

    let return_content = &block[return_pos + 7..];
    let end_pos = find_return_expression_end(return_content);
    Ok(return_content[..end_pos].trim())
}

fn find_return_expression_end(return_content: &str) -> usize {
    let mut depth = 0;
    let mut in_string = false;
    let mut in_regex = false;
    let mut escape_next = false;

    for (i, ch) in return_content.char_indices() {
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
                ';' if depth == 0 => return i,
                _ => {}
            }
        }

        if depth < 0 {
            return i;
        }
    }

    return_content.len()
}

fn expand_inline_helpers(return_expr: &str, helpers: &IndexMap<String, InlineHelper>) -> String {
    let mut expanded = return_expr.to_string();

    for (helper_name, helper) in helpers {
        let call_pattern = format!("{}(", helper_name);
        if !expanded.contains(&call_pattern) {
            continue;
        }

        eprintln!(
            "Debug: Expanding helper '{}' in return expression",
            helper_name
        );

        if let Some(next_expr) = expand_first_helper_call(&expanded, &call_pattern, helper) {
            expanded = next_expr;
            eprintln!("Debug: New return expression: '{}'", expanded);
        }
    }

    expanded
}

fn expand_first_helper_call(
    return_expr: &str,
    call_pattern: &str,
    helper: &InlineHelper,
) -> Option<String> {
    let call_start = return_expr.find(call_pattern)?;
    let args_start = call_start + call_pattern.len();
    let arg_end = find_helper_argument_end(return_expr, args_start)?;
    let arg = &return_expr[args_start..arg_end];
    eprintln!("Debug: Helper argument: '{}'", arg);

    let expanded_body = helper.body.replace(&helper.param_name, arg);
    eprintln!("Debug: Expanded body: '{}'", expanded_body);

    let mut result = String::new();
    result.push_str(&return_expr[..call_start]);
    result.push_str(&expanded_body);
    result.push_str(&return_expr[arg_end + 1..]);
    Some(result)
}

fn find_helper_argument_end(expr: &str, args_start: usize) -> Option<usize> {
    let mut depth = 1;

    for (i, ch) in expr.char_indices().skip_while(|(i, _)| *i < args_start) {
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
    use super::{ParsedFunctionBlock, parse};

    #[test]
    fn extracts_return_expression_until_top_level_semicolon() {
        let block = r#"{
            return seq($.left, choice(';', /a;b/), $.right);
        }"#;

        let parsed = parse(block).unwrap();

        match parsed {
            ParsedFunctionBlock::ReturnExpression(expr) => {
                assert_eq!(expr, "seq($.left, choice(';', /a;b/), $.right)");
            }
            ParsedFunctionBlock::PlaceholderChoice => panic!("expected return expression"),
        }
    }

    #[test]
    fn expands_simple_inline_helper_call() {
        let block = r#"{
            const commaSep = (rule) => seq(rule, repeat(seq(',', rule)));
            return commaSep($.item);
        }"#;

        let parsed = parse(block).unwrap();

        match parsed {
            ParsedFunctionBlock::ReturnExpression(expr) => {
                assert_eq!(expr, "seq($.item, repeat(seq(',', $.item)))");
            }
            ParsedFunctionBlock::PlaceholderChoice => panic!("expected return expression"),
        }
    }

    #[test]
    fn reports_table_definitions_as_placeholder_choices() {
        let block = r#"{
            const table = [[$.a, $.b]];
            return choice(...table.map(([left, right]) => seq(left, right)));
        }"#;

        let parsed = parse(block).unwrap();

        assert!(matches!(parsed, ParsedFunctionBlock::PlaceholderChoice));
    }
}
