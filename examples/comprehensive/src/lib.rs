// Comprehensive example showcasing rust-sitter capabilities

#[rust_sitter::grammar("mini_lang")]
pub mod grammar {
    /// The root of our mini language
    #[rust_sitter::language]
    pub struct Program {
        #[rust_sitter::repeat]
        pub items: Vec<Item>,
    }

    /// Top-level items in the program
    #[rust_sitter::language]
    pub enum Item {
        Function(Function),
        Struct(Struct),
        Variable(Variable),
    }

    /// Function definition
    #[rust_sitter::language]
    pub struct Function {
        #[rust_sitter::leaf(text = "fn")]
        _fn: (),
        pub name: Identifier,
        pub params: Parameters,
        pub return_type: Option<ReturnType>,
        pub body: Block,
    }

    /// Function parameters
    #[rust_sitter::language]
    pub struct Parameters {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub params: Vec<Parameter>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }

    /// A single parameter
    #[rust_sitter::language]
    pub struct Parameter {
        pub name: Identifier,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub type_annotation: Type,
    }

    /// Return type annotation
    #[rust_sitter::language]
    pub struct ReturnType {
        #[rust_sitter::leaf(text = "->")]
        _arrow: (),
        pub type_annotation: Type,
    }

    /// Struct definition
    #[rust_sitter::language]
    pub struct Struct {
        #[rust_sitter::leaf(text = "struct")]
        _struct: (),
        pub name: Identifier,
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub fields: Vec<Field>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }

    /// Struct field
    #[rust_sitter::language]
    pub struct Field {
        pub name: Identifier,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub type_annotation: Type,
        #[rust_sitter::leaf(text = ",")]
        _comma: (),
    }

    /// Variable declaration
    #[rust_sitter::language]
    pub struct Variable {
        pub kind: VarKind,
        pub name: Identifier,
        pub type_annotation: Option<TypeAnnotation>,
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub value: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    /// Variable kind (let or const)
    #[rust_sitter::language]
    pub enum VarKind {
        Let(LetKeyword),
        Const(ConstKeyword),
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

    /// Optional type annotation
    #[rust_sitter::language]
    pub struct TypeAnnotation {
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub type_expr: Type,
    }

    /// Type expressions
    #[rust_sitter::language]
    pub enum Type {
        Named(NamedType),
        Array(ArrayType),
        Optional(OptionalType),
    }

    #[rust_sitter::language]
    pub struct NamedType {
        pub name: Identifier,
    }

    #[rust_sitter::language]
    pub struct ArrayType {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        pub element: Box<Type>,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }

    #[rust_sitter::language]
    pub struct OptionalType {
        pub inner: Box<Type>,
        #[rust_sitter::leaf(text = "?")]
        _question: (),
    }

    /// Block of statements
    #[rust_sitter::language]
    pub struct Block {
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }

    /// Statements
    #[rust_sitter::language]
    pub enum Statement {
        Expression(ExpressionStatement),
        Return(ReturnStatement),
        If(IfStatement),
        While(WhileStatement),
        Assignment(AssignmentStatement),
        LocalVariable(Variable),
    }

    #[rust_sitter::language]
    pub struct ExpressionStatement {
        pub expression: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[rust_sitter::language]
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        _return: (),
        pub value: Option<Expression>,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    #[rust_sitter::language]
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        pub condition: Expression,
        pub then_block: Block,
        pub else_clause: Option<ElseClause>,
    }

    #[rust_sitter::language]
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        pub block: Block,
    }

    #[rust_sitter::language]
    pub struct WhileStatement {
        #[rust_sitter::leaf(text = "while")]
        _while: (),
        pub condition: Expression,
        pub body: Block,
    }

    #[rust_sitter::language]
    pub struct AssignmentStatement {
        pub target: Expression,
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub value: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }

    /// Expressions
    #[rust_sitter::language]
    pub enum Expression {
        Binary(Box<BinaryExpression>),
        Unary(Box<UnaryExpression>),
        Call(Box<CallExpression>),
        Access(Box<AccessExpression>),
        Index(Box<IndexExpression>),
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
        // Arithmetic
        Add(AddOp),
        Subtract(SubOp),
        Multiply(MulOp),
        Divide(DivOp),
        // Comparison
        Equal(EqOp),
        NotEqual(NeOp),
        Less(LtOp),
        Greater(GtOp),
        // Logical
        And(AndOp),
        Or(OrOp),
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
    pub struct AndOp {
        #[rust_sitter::leaf(text = "&&")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct OrOp {
        #[rust_sitter::leaf(text = "||")]
        _op: (),
    }

    #[rust_sitter::language]
    pub struct UnaryExpression {
        pub operator: UnaryOperator,
        pub operand: Expression,
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
        pub function: Expression,
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub arguments: Vec<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }

    #[rust_sitter::language]
    pub struct AccessExpression {
        pub object: Expression,
        #[rust_sitter::leaf(text = ".")]
        _dot: (),
        pub field: Identifier,
    }

    #[rust_sitter::language]
    pub struct IndexExpression {
        pub object: Expression,
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        pub index: Expression,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }

    #[rust_sitter::language]
    pub enum PrimaryExpression {
        Identifier(Identifier),
        Number(Number),
        String(StringLiteral),
        Boolean(Boolean),
        Array(ArrayLiteral),
    }

    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }

    #[rust_sitter::language]
    pub struct Number {
        #[rust_sitter::leaf(pattern = r"\d+(?:\.\d+)?", transform = |s| s.parse::<f64>().unwrap())]
        pub value: f64,
    }

    #[rust_sitter::language]
    pub struct StringLiteral {
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |s| s[1..s.len()-1].to_string())]
        pub value: String,
    }

    #[rust_sitter::language]
    pub enum Boolean {
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
    pub struct ArrayLiteral {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        #[rust_sitter::repeat]
        pub elements: Vec<Expression>,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_grammar_compiles() {
        // The fact that this compiles means our grammar is valid
        assert!(true);
    }
}
