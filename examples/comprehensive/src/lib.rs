// Comprehensive example showcasing adze capabilities

#[adze::grammar("mini_lang")]
pub mod grammar {
    /// The root of our mini language
    #[adze::language]
    pub struct Program {
        #[adze::repeat]
        pub items: Vec<Item>,
    }

    /// Top-level items in the program
    #[adze::language]
    pub enum Item {
        Function(Function),
        Struct(Struct),
        Variable(Variable),
    }

    /// Function definition
    #[adze::language]
    pub struct Function {
        #[adze::leaf(text = "fn")]
        _fn: (),
        pub name: Identifier,
        pub params: Parameters,
        pub return_type: Option<ReturnType>,
        pub body: Block,
    }

    /// Function parameters
    #[adze::language]
    pub struct Parameters {
        #[adze::leaf(text = "(")]
        _open: (),
        #[adze::repeat]
        pub params: Vec<Parameter>,
        #[adze::leaf(text = ")")]
        _close: (),
    }

    /// A single parameter
    #[adze::language]
    pub struct Parameter {
        pub name: Identifier,
        #[adze::leaf(text = ":")]
        _colon: (),
        pub type_annotation: Type,
    }

    /// Return type annotation
    #[adze::language]
    pub struct ReturnType {
        #[adze::leaf(text = "->")]
        _arrow: (),
        pub type_annotation: Type,
    }

    /// Struct definition
    #[adze::language]
    pub struct Struct {
        #[adze::leaf(text = "struct")]
        _struct: (),
        pub name: Identifier,
        #[adze::leaf(text = "{")]
        _open: (),
        #[adze::repeat]
        pub fields: Vec<Field>,
        #[adze::leaf(text = "}")]
        _close: (),
    }

    /// Struct field
    #[adze::language]
    pub struct Field {
        pub name: Identifier,
        #[adze::leaf(text = ":")]
        _colon: (),
        pub type_annotation: Type,
        #[adze::leaf(text = ",")]
        _comma: (),
    }

    /// Variable declaration
    #[adze::language]
    pub struct Variable {
        pub kind: VarKind,
        pub name: Identifier,
        pub type_annotation: Option<TypeAnnotation>,
        #[adze::leaf(text = "=")]
        _equals: (),
        pub value: Expression,
        #[adze::leaf(text = ";")]
        _semicolon: (),
    }

    /// Variable kind (let or const)
    #[adze::language]
    pub enum VarKind {
        Let(LetKeyword),
        Const(ConstKeyword),
    }

    #[adze::language]
    pub struct LetKeyword {
        #[adze::leaf(text = "let")]
        _let: (),
    }

    #[adze::language]
    pub struct ConstKeyword {
        #[adze::leaf(text = "const")]
        _const: (),
    }

    /// Optional type annotation
    #[adze::language]
    pub struct TypeAnnotation {
        #[adze::leaf(text = ":")]
        _colon: (),
        pub type_expr: Type,
    }

    /// Type expressions
    #[adze::language]
    pub enum Type {
        Named(NamedType),
        Array(ArrayType),
        Optional(OptionalType),
    }

    #[adze::language]
    pub struct NamedType {
        pub name: Identifier,
    }

    #[adze::language]
    pub struct ArrayType {
        #[adze::leaf(text = "[")]
        _open: (),
        pub element: Box<Type>,
        #[adze::leaf(text = "]")]
        _close: (),
    }

    #[adze::language]
    pub struct OptionalType {
        pub inner: Box<Type>,
        #[adze::leaf(text = "?")]
        _question: (),
    }

    /// Block of statements
    #[adze::language]
    pub struct Block {
        #[adze::leaf(text = "{")]
        _open: (),
        #[adze::repeat]
        pub statements: Vec<Statement>,
        #[adze::leaf(text = "}")]
        _close: (),
    }

    /// Statements
    #[adze::language]
    pub enum Statement {
        Expression(ExpressionStatement),
        Return(ReturnStatement),
        If(IfStatement),
        While(WhileStatement),
        Assignment(AssignmentStatement),
        LocalVariable(Variable),
    }

    #[adze::language]
    pub struct ExpressionStatement {
        pub expression: Expression,
        #[adze::leaf(text = ";")]
        _semicolon: (),
    }

    #[adze::language]
    pub struct ReturnStatement {
        #[adze::leaf(text = "return")]
        _return: (),
        pub value: Option<Expression>,
        #[adze::leaf(text = ";")]
        _semicolon: (),
    }

    #[adze::language]
    pub struct IfStatement {
        #[adze::leaf(text = "if")]
        _if: (),
        pub condition: Expression,
        pub then_block: Block,
        pub else_clause: Option<ElseClause>,
    }

    #[adze::language]
    pub struct ElseClause {
        #[adze::leaf(text = "else")]
        _else: (),
        pub block: Block,
    }

    #[adze::language]
    pub struct WhileStatement {
        #[adze::leaf(text = "while")]
        _while: (),
        pub condition: Expression,
        pub body: Block,
    }

    #[adze::language]
    pub struct AssignmentStatement {
        pub target: Expression,
        #[adze::leaf(text = "=")]
        _equals: (),
        pub value: Expression,
        #[adze::leaf(text = ";")]
        _semicolon: (),
    }

    /// Expressions
    #[adze::language]
    pub enum Expression {
        Binary(Box<BinaryExpression>),
        Unary(Box<UnaryExpression>),
        Call(Box<CallExpression>),
        Access(Box<AccessExpression>),
        Index(Box<IndexExpression>),
        Primary(PrimaryExpression),
    }

    #[adze::language]
    pub struct BinaryExpression {
        pub left: Expression,
        pub operator: BinaryOperator,
        pub right: Expression,
    }

    #[adze::language]
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

    #[adze::language]
    pub struct AddOp {
        #[adze::leaf(text = "+")]
        _op: (),
    }

    #[adze::language]
    pub struct SubOp {
        #[adze::leaf(text = "-")]
        _op: (),
    }

    #[adze::language]
    pub struct MulOp {
        #[adze::leaf(text = "*")]
        _op: (),
    }

    #[adze::language]
    pub struct DivOp {
        #[adze::leaf(text = "/")]
        _op: (),
    }

    #[adze::language]
    pub struct EqOp {
        #[adze::leaf(text = "==")]
        _op: (),
    }

    #[adze::language]
    pub struct NeOp {
        #[adze::leaf(text = "!=")]
        _op: (),
    }

    #[adze::language]
    pub struct LtOp {
        #[adze::leaf(text = "<")]
        _op: (),
    }

    #[adze::language]
    pub struct GtOp {
        #[adze::leaf(text = ">")]
        _op: (),
    }

    #[adze::language]
    pub struct AndOp {
        #[adze::leaf(text = "&&")]
        _op: (),
    }

    #[adze::language]
    pub struct OrOp {
        #[adze::leaf(text = "||")]
        _op: (),
    }

    #[adze::language]
    pub struct UnaryExpression {
        pub operator: UnaryOperator,
        pub operand: Expression,
    }

    #[adze::language]
    pub enum UnaryOperator {
        Not(NotOp),
        Minus(MinusOp),
    }

    #[adze::language]
    pub struct NotOp {
        #[adze::leaf(text = "!")]
        _op: (),
    }

    #[adze::language]
    pub struct MinusOp {
        #[adze::leaf(text = "-")]
        _op: (),
    }

    #[adze::language]
    pub struct CallExpression {
        pub function: Expression,
        #[adze::leaf(text = "(")]
        _open: (),
        #[adze::repeat]
        pub arguments: Vec<Expression>,
        #[adze::leaf(text = ")")]
        _close: (),
    }

    #[adze::language]
    pub struct AccessExpression {
        pub object: Expression,
        #[adze::leaf(text = ".")]
        _dot: (),
        pub field: Identifier,
    }

    #[adze::language]
    pub struct IndexExpression {
        pub object: Expression,
        #[adze::leaf(text = "[")]
        _open: (),
        pub index: Expression,
        #[adze::leaf(text = "]")]
        _close: (),
    }

    #[adze::language]
    pub enum PrimaryExpression {
        Identifier(Identifier),
        Number(Number),
        String(StringLiteral),
        Boolean(Boolean),
        Array(ArrayLiteral),
    }

    #[adze::language]
    pub struct Identifier {
        #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }

    #[adze::language]
    pub struct Number {
        #[adze::leaf(pattern = r"\d+(?:\.\d+)?", transform = |s| s.parse::<f64>().unwrap())]
        pub value: f64,
    }

    #[adze::language]
    pub struct StringLiteral {
        #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |s| s[1..s.len()-1].to_string())]
        pub value: String,
    }

    #[adze::language]
    pub enum Boolean {
        True(TrueLiteral),
        False(FalseLiteral),
    }

    #[adze::language]
    pub struct TrueLiteral {
        #[adze::leaf(text = "true")]
        _true: (),
    }

    #[adze::language]
    pub struct FalseLiteral {
        #[adze::leaf(text = "false")]
        _false: (),
    }

    #[adze::language]
    pub struct ArrayLiteral {
        #[adze::leaf(text = "[")]
        _open: (),
        #[adze::repeat]
        pub elements: Vec<Expression>,
        #[adze::leaf(text = "]")]
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
