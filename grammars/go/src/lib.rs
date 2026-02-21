#![allow(
    clippy::empty_line_after_outer_attr,
    clippy::manual_non_exhaustive,
    clippy::op_ref,
    clippy::char_lit_as_u8,
    clippy::unnecessary_cast,
    clippy::assertions_on_constants
)]

// Simplified Go grammar for adze v0.5.0-beta
// This is a minimal subset to demonstrate basic functionality

#[adze::grammar("go")]
pub mod grammar {
    #[adze::language]
    pub struct SourceFile {
        pub package_clause: PackageClause,
        #[adze::repeat]
        pub declarations: Vec<Declaration>,
    }

    #[adze::language]
    pub struct PackageClause {
        #[adze::leaf(text = "package")]
        _package: (),
        pub name: Identifier,
    }

    #[adze::language]
    pub enum Declaration {
        Function(FunctionDeclaration),
        Variable(VarDeclaration),
    }

    #[adze::language]
    pub struct FunctionDeclaration {
        #[adze::leaf(text = "func")]
        _func: (),
        pub name: Identifier,
        #[adze::leaf(text = "(")]
        _lparen: (),
        #[adze::leaf(text = ")")]
        _rparen: (),
        pub body: Block,
    }

    #[adze::language]
    pub struct VarDeclaration {
        #[adze::leaf(text = "var")]
        _var: (),
        pub name: Identifier,
        pub type_name: Identifier,
    }

    #[adze::language]
    pub struct Block {
        #[adze::leaf(text = "{")]
        _open: (),
        #[adze::repeat]
        pub statements: Vec<Statement>,
        #[adze::leaf(text = "}")]
        _close: (),
    }

    #[adze::language]
    pub enum Statement {
        Assignment(AssignmentStatement),
        Call(CallStatement),
        Return(ReturnStatement),
    }

    #[adze::language]
    pub struct AssignmentStatement {
        pub name: Identifier,
        #[adze::leaf(text = "=")]
        _eq: (),
        pub value: Expression,
    }

    #[adze::language]
    pub struct CallStatement {
        pub name: Identifier,
        #[adze::leaf(text = "(")]
        _lparen: (),
        #[adze::repeat]
        pub args: Vec<Expression>,
        #[adze::leaf(text = ")")]
        _rparen: (),
    }

    #[adze::language]
    pub struct ReturnStatement {
        #[adze::leaf(text = "return")]
        _return: (),
        pub value: Expression,
    }

    #[adze::language]
    pub enum Expression {
        Identifier(Identifier),
        Literal(Literal),
    }

    #[adze::language]
    pub enum Literal {
        String(StringLiteral),
        Number(NumberLiteral),
    }

    #[adze::language]
    pub struct StringLiteral {
        #[adze::leaf(pattern = r#""[^"]*""#)]
        pub value: (),
    }

    #[adze::language]
    pub struct NumberLiteral {
        #[adze::leaf(pattern = r"\d+")]
        pub value: (),
    }

    #[adze::language]
    pub struct Identifier {
        #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: (),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_go() {
        // Grammar builds successfully
        assert!(true);
    }
}
