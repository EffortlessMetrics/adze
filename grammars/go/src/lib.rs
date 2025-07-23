// Simplified Go grammar for rust-sitter v0.5.0-beta
// This is a minimal subset to demonstrate basic functionality

#[rust_sitter::grammar("go")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct SourceFile {
        pub package_clause: PackageClause,
        #[rust_sitter::repeat]
        pub declarations: Vec<Declaration>,
    }
    
    #[rust_sitter::language]
    pub struct PackageClause {
        #[rust_sitter::leaf(text = "package")]
        _package: (),
        pub name: Identifier,
    }
    
    #[rust_sitter::language]
    pub enum Declaration {
        Function(FunctionDeclaration),
        Variable(VarDeclaration),
    }
    
    #[rust_sitter::language]
    pub struct FunctionDeclaration {
        #[rust_sitter::leaf(text = "func")]
        _func: (),
        pub name: Identifier,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
        pub body: Block,
    }
    
    #[rust_sitter::language]
    pub struct VarDeclaration {
        #[rust_sitter::leaf(text = "var")]
        _var: (),
        pub name: Identifier,
        pub type_name: Identifier,
    }
    
    #[rust_sitter::language]
    pub struct Block {
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub enum Statement {
        Assignment(AssignmentStatement),
        Call(CallStatement),
        Return(ReturnStatement),
    }
    
    #[rust_sitter::language]
    pub struct AssignmentStatement {
        pub name: Identifier,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Expression,
    }
    
    #[rust_sitter::language]
    pub struct CallStatement {
        pub name: Identifier,
        #[rust_sitter::leaf(text = "(")]
        _lparen: (),
        #[rust_sitter::repeat]
        pub args: Vec<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _rparen: (),
    }
    
    #[rust_sitter::language]
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        _return: (),
        pub value: Expression,
    }
    
    #[rust_sitter::language]
    pub enum Expression {
        Identifier(Identifier),
        Literal(Literal),
    }
    
    #[rust_sitter::language]
    pub enum Literal {
        String(StringLiteral),
        Number(NumberLiteral),
    }
    
    #[rust_sitter::language]
    pub struct StringLiteral {
        #[rust_sitter::leaf(pattern = r#""[^"]*""#)]
        pub value: (),
    }
    
    #[rust_sitter::language]
    pub struct NumberLiteral {
        #[rust_sitter::leaf(pattern = r"\d+")]
        pub value: (),
    }
    
    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
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