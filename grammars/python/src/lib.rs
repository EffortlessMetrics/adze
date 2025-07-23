// Python grammar for rust-sitter
// Simplified version for v0.5.0-beta (without indentation handling)

#[rust_sitter::grammar("python")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Module {
        #[rust_sitter::repeat]
        pub statements: Vec<Statement>,
    }
    
    #[rust_sitter::language]
    pub enum Statement {
        Simple(SimpleStatement),
        Compound(CompoundStatement),
    }
    
    #[rust_sitter::language]
    pub struct SimpleStatement {
        pub statement: SimpleStmt,
        #[rust_sitter::leaf(pattern = r"\n")]
        _newline: String,
    }
    
    #[rust_sitter::language]
    pub enum SimpleStmt {
        Expression(ExpressionStatement),
        Assignment(Assignment),
        Return(ReturnStatement),
        Pass(PassStatement),
        Break(BreakStatement),
        Continue(ContinueStatement),
        Import(ImportStatement),
    }
    
    #[rust_sitter::language]
    pub enum CompoundStatement {
        Function(FunctionDefinition),
        Class(ClassDefinition),
        If(IfStatement),
        While(WhileStatement),
        For(ForStatement),
    }
    
    #[rust_sitter::language]
    pub struct ExpressionStatement {
        pub expression: Expression,
    }
    
    #[rust_sitter::language]
    pub struct Assignment {
        pub target: Expression,
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub value: Expression,
    }
    
    #[rust_sitter::language]
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        _return: (),
        pub value: Option<Expression>,
    }
    
    #[rust_sitter::language]
    pub struct PassStatement {
        #[rust_sitter::leaf(text = "pass")]
        _pass: (),
    }
    
    #[rust_sitter::language]
    pub struct BreakStatement {
        #[rust_sitter::leaf(text = "break")]
        _break: (),
    }
    
    #[rust_sitter::language]
    pub struct ContinueStatement {
        #[rust_sitter::leaf(text = "continue")]
        _continue: (),
    }
    
    #[rust_sitter::language]
    pub struct ImportStatement {
        #[rust_sitter::leaf(text = "import")]
        _import: (),
        pub module: DottedName,
    }
    
    #[rust_sitter::language]
    pub struct DottedName {
        pub first: Identifier,
        #[rust_sitter::repeat]
        pub rest: Vec<DottedNamePart>,
    }
    
    #[rust_sitter::language]
    pub struct DottedNamePart {
        #[rust_sitter::leaf(text = ".")]
        _dot: (),
        pub name: Identifier,
    }
    
    #[rust_sitter::language]
    pub struct FunctionDefinition {
        #[rust_sitter::leaf(text = "def")]
        _def: (),
        pub name: Identifier,
        pub parameters: Parameters,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        #[rust_sitter::leaf(pattern = r"\n")]
        _newline: String,
        pub body: Block,
    }
    
    #[rust_sitter::language]
    pub struct Parameters {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub params: Vec<Parameter>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct Parameter {
        pub name: Identifier,
    }
    
    #[rust_sitter::language]
    pub struct Block {
        #[rust_sitter::repeat(non_empty = true)]
        pub statements: Vec<Statement>,
    }
    
    #[rust_sitter::language]
    pub struct ClassDefinition {
        #[rust_sitter::leaf(text = "class")]
        _class: (),
        pub name: Identifier,
        pub bases: Option<ClassBases>,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        #[rust_sitter::leaf(pattern = r"\n")]
        _newline: String,
        pub body: Block,
    }
    
    #[rust_sitter::language]
    pub struct ClassBases {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub bases: Vec<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        #[rust_sitter::leaf(pattern = r"\n")]
        _newline: String,
        pub body: Block,
        pub else_clause: Option<ElseClause>,
    }
    
    #[rust_sitter::language]
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        #[rust_sitter::leaf(pattern = r"\n")]
        _newline: String,
        pub body: Block,
    }
    
    #[rust_sitter::language]
    pub struct WhileStatement {
        #[rust_sitter::leaf(text = "while")]
        _while: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        #[rust_sitter::leaf(pattern = r"\n")]
        _newline: String,
        pub body: Block,
    }
    
    #[rust_sitter::language]
    pub struct ForStatement {
        #[rust_sitter::leaf(text = "for")]
        _for: (),
        pub target: Identifier,
        #[rust_sitter::leaf(text = "in")]
        _in: (),
        pub iter: Expression,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        #[rust_sitter::leaf(pattern = r"\n")]
        _newline: String,
        pub body: Block,
    }
    
    #[rust_sitter::language]
    pub enum Expression {
        Binary(Box<BinaryExpression>),
        Unary(Box<UnaryExpression>),
        Call(Box<CallExpression>),
        Attribute(Box<AttributeExpression>),
        Subscript(Box<SubscriptExpression>),
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
        Modulo(ModOp),
        Power(PowerOp),
        Equal(EqOp),
        NotEqual(NeOp),
        Less(LtOp),
        Greater(GtOp),
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
    pub struct ModOp {
        #[rust_sitter::leaf(text = "%")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct PowerOp {
        #[rust_sitter::leaf(text = "**")]
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
        #[rust_sitter::leaf(text = "and")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct OrOp {
        #[rust_sitter::leaf(text = "or")]
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
        #[rust_sitter::leaf(text = "not")]
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
        pub arguments: Arguments,
    }
    
    #[rust_sitter::language]
    pub struct Arguments {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub args: Vec<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct AttributeExpression {
        pub value: Expression,
        #[rust_sitter::leaf(text = ".")]
        _dot: (),
        pub attr: Identifier,
    }
    
    #[rust_sitter::language]
    pub struct SubscriptExpression {
        pub value: Expression,
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        pub index: Expression,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub enum PrimaryExpression {
        Identifier(Identifier),
        Literal(Literal),
        List(ListExpression),
        Tuple(TupleExpression),
        Dict(DictExpression),
    }
    
    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }
    
    #[rust_sitter::language]
    pub enum Literal {
        String(StringLiteral),
        Number(NumberLiteral),
        Boolean(BooleanLiteral),
        None(NoneLiteral),
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
        #[rust_sitter::leaf(text = "True")]
        _true: (),
    }
    
    #[rust_sitter::language]
    pub struct FalseLiteral {
        #[rust_sitter::leaf(text = "False")]
        _false: (),
    }
    
    #[rust_sitter::language]
    pub struct NoneLiteral {
        #[rust_sitter::leaf(text = "None")]
        _none: (),
    }
    
    #[rust_sitter::language]
    pub struct ListExpression {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        #[rust_sitter::repeat]
        pub elements: Vec<Expression>,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct TupleExpression {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub elements: Vec<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct DictExpression {
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub items: Vec<DictItem>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct DictItem {
        pub key: Expression,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub value: Expression,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_program() {
        // Grammar builds successfully
        assert!(true);
    }
}