// JavaScript grammar for rust-sitter
// Simplified version for v0.5.0-beta

// Allow clippy::manual_non_exhaustive since underscore fields are semantic tokens in rust-sitter grammar,
// not actual non-exhaustive pattern implementations
#![allow(clippy::manual_non_exhaustive)]

#[rust_sitter::grammar("javascript")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        #[rust_sitter::repeat(non_empty = true)]
        pub statements: Vec<Statement>,
    }

    #[rust_sitter::language]
    pub enum Statement {
        Expression(ExpressionStatement),
        Variable(VariableDeclaration),
        Function(FunctionDeclaration),
        Return(ReturnStatement),
        If(IfStatement),
        Block(BlockStatement),
        Empty(EmptyStatement),
    }

    #[rust_sitter::language]
    pub struct ExpressionStatement {
        pub expression: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[rust_sitter::language]
    pub struct Semicolon {
        #[rust_sitter::leaf(text = ";")]
        _semi: (),
    }

    #[rust_sitter::language]
    pub struct VariableDeclaration {
        pub kind: VarKind,
        #[rust_sitter::repeat(non_empty = true)]
        pub declarations: Vec<VariableDeclarator>,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[rust_sitter::language]
    pub enum VarKind {
        Var(VarKeyword),
        Let(LetKeyword),
        Const(ConstKeyword),
    }

    #[rust_sitter::language]
    pub struct VarKeyword {
        #[rust_sitter::leaf(text = "var")]
        _var: (),
    }

    #[rust_sitter::language]
    pub struct LetKeyword {
        #[rust_sitter::leaf(text = "let")]
        _let: (),
    }

    #[rust_sitter::language]
    pub struct ConstKeyword {
        #[rust_sitter::leaf(text = "const")]
        _const: (),
    }

    #[rust_sitter::language]
    pub struct VariableDeclarator {
        pub id: Identifier,
        pub init: Option<VariableInit>,
    }

    #[rust_sitter::language]
    pub struct VariableInit {
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub expression: Expression,
    }

    #[rust_sitter::language]
    pub struct FunctionDeclaration {
        #[rust_sitter::leaf(text = "function")]
        _function: (),
        pub name: Identifier,
        pub parameters: FormalParameters,
        pub body: BlockStatement,
    }

    #[rust_sitter::language]
    pub struct FormalParameters {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub params: Vec<FormalParameter>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }

    #[rust_sitter::language]
    pub struct FormalParameter {
        pub name: Identifier,
    }

    #[rust_sitter::language]
    pub struct Comma {
        #[rust_sitter::leaf(text = ",")]
        _comma: (),
    }

    #[rust_sitter::language]
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        _return: (),
        pub expression: Option<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[rust_sitter::language]
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
        pub then_statement: Box<Statement>,
        pub else_clause: Option<ElseClause>,
    }

    #[rust_sitter::language]
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        pub statement: Box<Statement>,
    }

    #[rust_sitter::language]
    pub struct BlockStatement {
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }

    #[rust_sitter::language]
    pub struct EmptyStatement {
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[rust_sitter::language]
    pub enum Expression {
        Binary(Box<BinaryExpression>),
        Unary(Box<UnaryExpression>),
        Call(Box<CallExpression>),
        Member(Box<MemberExpression>),
        Primary(PrimaryExpression),
    }

    #[rust_sitter::language]
    pub struct BinaryExpression {
        pub left: Expression,
        pub operator: BinaryOperator,
        pub right: Expression,
    }

    #[rust_sitter::language]
    pub enum BinaryOperator {
        Add(AddOp),
        Subtract(SubOp),
        Multiply(MulOp),
        Divide(DivOp),
        Equal(EqOp),
        NotEqual(NeOp),
        Less(LtOp),
        Greater(GtOp),
    }

    #[rust_sitter::language]
    pub struct AddOp {
        #[rust_sitter::leaf(text = "+")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct SubOp {
        #[rust_sitter::leaf(text = "-")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct MulOp {
        #[rust_sitter::leaf(text = "*")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct DivOp {
        #[rust_sitter::leaf(text = "/")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct EqOp {
        #[rust_sitter::leaf(text = "==")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct NeOp {
        #[rust_sitter::leaf(text = "!=")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct LtOp {
        #[rust_sitter::leaf(text = "<")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct GtOp {
        #[rust_sitter::leaf(text = ">")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct UnaryExpression {
        pub operator: UnaryOperator,
        pub argument: Expression,
    }

    #[rust_sitter::language]
    pub enum UnaryOperator {
        Not(NotOp),
        Minus(MinusOp),
    }

    #[rust_sitter::language]
    pub struct NotOp {
        #[rust_sitter::leaf(text = "!")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct MinusOp {
        #[rust_sitter::leaf(text = "-")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct CallExpression {
        pub callee: Expression,
        pub arguments: Arguments,
    }

    #[rust_sitter::language]
    pub struct Arguments {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub args: Vec<Argument>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }

    #[rust_sitter::language]
    pub struct Argument {
        pub expression: Expression,
    }

    #[rust_sitter::language]
    pub struct MemberExpression {
        pub object: Expression,
        pub property: MemberProperty,
    }

    #[rust_sitter::language]
    pub enum MemberProperty {
        Dot(DotProperty),
        Bracket(BracketProperty),
    }

    #[rust_sitter::language]
    pub struct DotProperty {
        #[rust_sitter::leaf(text = ".")]
        _dot: (),
        pub property: Identifier,
    }

    #[rust_sitter::language]
    pub struct BracketProperty {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        pub property: Expression,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }

    #[rust_sitter::language]
    pub enum PrimaryExpression {
        Identifier(Identifier),
        Literal(Literal),
        Parenthesized(ParenthesizedExpression),
    }

    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_$][a-zA-Z0-9_$]*")]
        pub name: String,
    }

    #[rust_sitter::language]
    pub enum Literal {
        String(StringLiteral),
        Number(NumberLiteral),
        Boolean(BooleanLiteral),
        Null(NullLiteral),
    }

    #[rust_sitter::language]
    pub enum StringLiteral {
        SingleQuoted(SingleQuotedString),
        DoubleQuoted(DoubleQuotedString),
    }

    #[rust_sitter::language]
    pub struct SingleQuotedString {
        #[rust_sitter::leaf(pattern = r"'([^'\\]|\\.)*'")]
        pub value: String,
    }

    #[rust_sitter::language]
    pub struct DoubleQuotedString {
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
        pub value: String,
    }

    #[rust_sitter::language]
    pub struct NumberLiteral {
        #[rust_sitter::leaf(pattern = r"(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?")]
        pub value: String,
    }

    #[rust_sitter::language]
    pub enum BooleanLiteral {
        True(TrueLiteral),
        False(FalseLiteral),
    }

    #[rust_sitter::language]
    pub struct TrueLiteral {
        #[rust_sitter::leaf(text = "true")]
        _true: (),
    }

    #[rust_sitter::language]
    pub struct FalseLiteral {
        #[rust_sitter::leaf(text = "false")]
        _false: (),
    }

    #[rust_sitter::language]
    pub struct NullLiteral {
        #[rust_sitter::leaf(text = "null")]
        _null: (),
    }

    #[rust_sitter::language]
    pub struct ParenthesizedExpression {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        pub expression: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_program() {
        // Grammar builds successfully - this test ensures the grammar compiles without issues
    }
}
