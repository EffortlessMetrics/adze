//! Example demonstrating external scanner and word token attributes

#[allow(dead_code)]
#[rust_sitter::grammar("python_like")]
mod grammar {
    #[rust_sitter::language]
    pub enum Statement {
        If(IfStatement),
        Expression(Expression),
    }

    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        condition: Expression,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        body: Vec<Statement>,
    }

    pub enum Expression {
        Identifier(Identifier),
        Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
    }

    // Word token - helps distinguish keywords from identifiers
    #[rust_sitter::word]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_]\w*")]
        name: String,
    }

    // External scanner tokens for indentation-based parsing
    #[rust_sitter::external]
    pub struct Indent;

    #[rust_sitter::external]
    pub struct Dedent;

    #[rust_sitter::external]
    pub struct Newline;

    // Regular extras
    #[rust_sitter::extra]
    pub struct Whitespace {
        #[rust_sitter::leaf(pattern = r"[ \t]+")]
        _ws: (),
    }
}

// Note: In a real implementation, you would need to provide an external scanner
// implementation that handles the Indent/Dedent/Newline tokens based on
// indentation levels, similar to how Python parsers work.

#[cfg(test)]
mod tests {

    #[test]
    fn test_word_token() {
        // This test would work once the full parser is generated
        // The word token helps ensure "if" is parsed as a keyword, not an identifier
    }
}
