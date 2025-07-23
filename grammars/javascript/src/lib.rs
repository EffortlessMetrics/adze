// JavaScript grammar for rust-sitter
// Based on tree-sitter-javascript

#[rust_sitter::grammar("javascript")]
pub mod grammar {
    use rust_sitter::Spanned;
    
    #[rust_sitter::language]
    pub struct Program {
        #[rust_sitter::field("body")]
        pub statements: Vec<Statement>,
    }
    
    #[rust_sitter::extra]
    pub enum Extra {
        Whitespace(#[rust_sitter::leaf(pattern = r"\s+")] ()),
        LineComment(#[rust_sitter::leaf(pattern = r"//[^\n]*")] ()),
        BlockComment(#[rust_sitter::leaf(pattern = r"/\*([^*]|\*[^/])*\*/")] ()),
    }
    
    pub enum Statement {
        Expression(ExpressionStatement),
        Variable(VariableDeclaration),
        Function(FunctionDeclaration),
        Return(ReturnStatement),
        If(IfStatement),
        While(WhileStatement),
        For(ForStatement),
        Block(BlockStatement),
        Empty(EmptyStatement),
        Break(BreakStatement),
        Continue(ContinueStatement),
        Throw(ThrowStatement),
        Try(TryStatement),
    }
    
    pub struct ExpressionStatement {
        pub expression: Expression,
        pub semicolon: Option<#[rust_sitter::leaf(text = ";")] ()>,
    }
    
    pub struct VariableDeclaration {
        pub kind: VarKind,
        pub declarations: Vec<VariableDeclarator>,
        pub semicolon: Option<#[rust_sitter::leaf(text = ";")] ()>,
    }
    
    pub enum VarKind {
        #[rust_sitter::leaf(text = "var")]
        Var,
        #[rust_sitter::leaf(text = "let")]
        Let,
        #[rust_sitter::leaf(text = "const")]
        Const,
    }
    
    pub struct VariableDeclarator {
        #[rust_sitter::field("name")]
        pub id: BindingPattern,
        pub init: Option<VariableInit>,
    }
    
    pub struct VariableInit {
        #[rust_sitter::leaf(text = "=")]
        pub equals: (),
        #[rust_sitter::field("value")]
        pub expression: Expression,
    }
    
    pub enum BindingPattern {
        Identifier(Identifier),
        Object(ObjectPattern),
        Array(ArrayPattern),
    }
    
    pub struct ObjectPattern {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub properties: Vec<ObjectPatternProperty>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub struct ObjectPatternProperty {
        pub key: PropertyKey,
        pub pattern: Option<ObjectPatternValue>,
    }
    
    pub struct ObjectPatternValue {
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub pattern: BindingPattern,
    }
    
    pub struct ArrayPattern {
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub elements: Vec<Option<BindingPattern>>,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub struct FunctionDeclaration {
        #[rust_sitter::leaf(text = "function")]
        pub function_keyword: (),
        #[rust_sitter::field("name")]
        pub name: Identifier,
        #[rust_sitter::field("parameters")]
        pub parameters: FormalParameters,
        #[rust_sitter::field("body")]
        pub body: BlockStatement,
    }
    
    pub struct FormalParameters {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub params: Vec<FormalParameter>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub enum FormalParameter {
        Identifier(Identifier),
        Pattern(BindingPattern),
        Rest(RestParameter),
    }
    
    pub struct RestParameter {
        #[rust_sitter::leaf(text = "...")]
        pub dots: (),
        pub pattern: BindingPattern,
    }
    
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        pub return_keyword: (),
        pub expression: Option<Expression>,
        pub semicolon: Option<#[rust_sitter::leaf(text = ";")] ()>,
    }
    
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        pub if_keyword: (),
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        #[rust_sitter::field("condition")]
        pub condition: Expression,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
        #[rust_sitter::field("consequence")]
        pub then_statement: Box<Statement>,
        pub else_clause: Option<ElseClause>,
    }
    
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        pub else_keyword: (),
        #[rust_sitter::field("alternative")]
        pub statement: Box<Statement>,
    }
    
    pub struct WhileStatement {
        #[rust_sitter::leaf(text = "while")]
        pub while_keyword: (),
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
        pub body: Box<Statement>,
    }
    
    pub struct ForStatement {
        #[rust_sitter::leaf(text = "for")]
        pub for_keyword: (),
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub init: Option<ForInit>,
        #[rust_sitter::leaf(text = ";")]
        pub semi1: (),
        pub test: Option<Expression>,
        #[rust_sitter::leaf(text = ";")]
        pub semi2: (),
        pub update: Option<Expression>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
        pub body: Box<Statement>,
    }
    
    pub enum ForInit {
        Variable(VariableDeclaration),
        Expression(Expression),
    }
    
    pub struct BlockStatement {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub struct EmptyStatement {
        #[rust_sitter::leaf(text = ";")]
        pub semicolon: (),
    }
    
    pub struct BreakStatement {
        #[rust_sitter::leaf(text = "break")]
        pub break_keyword: (),
        pub label: Option<Identifier>,
        pub semicolon: Option<#[rust_sitter::leaf(text = ";")] ()>,
    }
    
    pub struct ContinueStatement {
        #[rust_sitter::leaf(text = "continue")]
        pub continue_keyword: (),
        pub label: Option<Identifier>,
        pub semicolon: Option<#[rust_sitter::leaf(text = ";")] ()>,
    }
    
    pub struct ThrowStatement {
        #[rust_sitter::leaf(text = "throw")]
        pub throw_keyword: (),
        pub expression: Expression,
        pub semicolon: Option<#[rust_sitter::leaf(text = ";")] ()>,
    }
    
    pub struct TryStatement {
        #[rust_sitter::leaf(text = "try")]
        pub try_keyword: (),
        pub block: BlockStatement,
        pub handler: Option<CatchClause>,
        pub finalizer: Option<FinallyClause>,
    }
    
    pub struct CatchClause {
        #[rust_sitter::leaf(text = "catch")]
        pub catch_keyword: (),
        pub param: Option<CatchParameter>,
        pub body: BlockStatement,
    }
    
    pub struct CatchParameter {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub pattern: BindingPattern,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub struct FinallyClause {
        #[rust_sitter::leaf(text = "finally")]
        pub finally_keyword: (),
        pub block: BlockStatement,
    }
    
    pub enum Expression {
        #[rust_sitter::prec(1)]
        Assignment(Box<AssignmentExpression>),
        #[rust_sitter::prec(2)]
        Conditional(Box<ConditionalExpression>),
        #[rust_sitter::prec(3)]
        LogicalOr(Box<BinaryExpression>),
        #[rust_sitter::prec(4)]
        LogicalAnd(Box<BinaryExpression>),
        #[rust_sitter::prec(5)]
        BitwiseOr(Box<BinaryExpression>),
        #[rust_sitter::prec(6)]
        BitwiseXor(Box<BinaryExpression>),
        #[rust_sitter::prec(7)]
        BitwiseAnd(Box<BinaryExpression>),
        #[rust_sitter::prec(8)]
        Equality(Box<BinaryExpression>),
        #[rust_sitter::prec(9)]
        Relational(Box<BinaryExpression>),
        #[rust_sitter::prec(10)]
        Shift(Box<BinaryExpression>),
        #[rust_sitter::prec_left(11)]
        Additive(Box<BinaryExpression>),
        #[rust_sitter::prec_left(12)]
        Multiplicative(Box<BinaryExpression>),
        #[rust_sitter::prec(13)]
        Unary(Box<UnaryExpression>),
        #[rust_sitter::prec(14)]
        Update(Box<UpdateExpression>),
        #[rust_sitter::prec_left(15)]
        Call(Box<CallExpression>),
        #[rust_sitter::prec_left(15)]
        Member(Box<MemberExpression>),
        #[rust_sitter::prec(16)]
        Primary(PrimaryExpression),
    }
    
    pub struct AssignmentExpression {
        #[rust_sitter::field("left")]
        pub left: Expression,
        #[rust_sitter::field("operator")]
        pub operator: AssignmentOperator,
        #[rust_sitter::field("right")]
        pub right: Expression,
    }
    
    pub enum AssignmentOperator {
        #[rust_sitter::leaf(text = "=")]
        Assign,
        #[rust_sitter::leaf(text = "+=")]
        AddAssign,
        #[rust_sitter::leaf(text = "-=")]
        SubAssign,
        #[rust_sitter::leaf(text = "*=")]
        MulAssign,
        #[rust_sitter::leaf(text = "/=")]
        DivAssign,
        #[rust_sitter::leaf(text = "%=")]
        ModAssign,
        #[rust_sitter::leaf(text = "<<=")]
        LeftShiftAssign,
        #[rust_sitter::leaf(text = ">>=")]
        RightShiftAssign,
        #[rust_sitter::leaf(text = ">>>=")]
        UnsignedRightShiftAssign,
        #[rust_sitter::leaf(text = "&=")]
        BitAndAssign,
        #[rust_sitter::leaf(text = "^=")]
        BitXorAssign,
        #[rust_sitter::leaf(text = "|=")]
        BitOrAssign,
    }
    
    pub struct ConditionalExpression {
        pub test: Expression,
        #[rust_sitter::leaf(text = "?")]
        pub question: (),
        pub consequent: Expression,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub alternate: Expression,
    }
    
    pub struct BinaryExpression {
        #[rust_sitter::field("left")]
        pub left: Expression,
        #[rust_sitter::field("operator")]
        pub operator: BinaryOperator,
        #[rust_sitter::field("right")]
        pub right: Expression,
    }
    
    pub enum BinaryOperator {
        #[rust_sitter::leaf(text = "||")]
        LogicalOr,
        #[rust_sitter::leaf(text = "&&")]
        LogicalAnd,
        #[rust_sitter::leaf(text = "|")]
        BitwiseOr,
        #[rust_sitter::leaf(text = "^")]
        BitwiseXor,
        #[rust_sitter::leaf(text = "&")]
        BitwiseAnd,
        #[rust_sitter::leaf(text = "==")]
        Equal,
        #[rust_sitter::leaf(text = "!=")]
        NotEqual,
        #[rust_sitter::leaf(text = "===")]
        StrictEqual,
        #[rust_sitter::leaf(text = "!==")]
        StrictNotEqual,
        #[rust_sitter::leaf(text = "<")]
        Less,
        #[rust_sitter::leaf(text = ">")]
        Greater,
        #[rust_sitter::leaf(text = "<=")]
        LessEqual,
        #[rust_sitter::leaf(text = ">=")]
        GreaterEqual,
        #[rust_sitter::leaf(text = "<<")]
        LeftShift,
        #[rust_sitter::leaf(text = ">>")]
        RightShift,
        #[rust_sitter::leaf(text = ">>>")]
        UnsignedRightShift,
        #[rust_sitter::leaf(text = "+")]
        Add,
        #[rust_sitter::leaf(text = "-")]
        Subtract,
        #[rust_sitter::leaf(text = "*")]
        Multiply,
        #[rust_sitter::leaf(text = "/")]
        Divide,
        #[rust_sitter::leaf(text = "%")]
        Modulo,
    }
    
    pub struct UnaryExpression {
        pub operator: UnaryOperator,
        pub argument: Expression,
    }
    
    pub enum UnaryOperator {
        #[rust_sitter::leaf(text = "!")]
        Not,
        #[rust_sitter::leaf(text = "~")]
        BitwiseNot,
        #[rust_sitter::leaf(text = "+")]
        Plus,
        #[rust_sitter::leaf(text = "-")]
        Minus,
        #[rust_sitter::leaf(text = "typeof")]
        Typeof,
        #[rust_sitter::leaf(text = "void")]
        Void,
        #[rust_sitter::leaf(text = "delete")]
        Delete,
    }
    
    pub struct UpdateExpression {
        pub operator: UpdateOperator,
        pub argument: Expression,
        pub prefix: bool,
    }
    
    pub enum UpdateOperator {
        #[rust_sitter::leaf(text = "++")]
        Increment,
        #[rust_sitter::leaf(text = "--")]
        Decrement,
    }
    
    pub struct CallExpression {
        #[rust_sitter::field("function")]
        pub callee: Expression,
        #[rust_sitter::field("arguments")]
        pub arguments: Arguments,
    }
    
    pub struct Arguments {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub args: Vec<Argument>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub enum Argument {
        Expression(Expression),
        Spread(SpreadElement),
    }
    
    pub struct SpreadElement {
        #[rust_sitter::leaf(text = "...")]
        pub dots: (),
        pub expression: Expression,
    }
    
    pub struct MemberExpression {
        #[rust_sitter::field("object")]
        pub object: Expression,
        pub property: MemberProperty,
    }
    
    pub enum MemberProperty {
        Dot(DotProperty),
        Bracket(BracketProperty),
    }
    
    pub struct DotProperty {
        #[rust_sitter::leaf(text = ".")]
        pub dot: (),
        #[rust_sitter::field("property")]
        pub property: Identifier,
    }
    
    pub struct BracketProperty {
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        #[rust_sitter::field("property")]
        pub property: Expression,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub enum PrimaryExpression {
        Identifier(Identifier),
        Literal(Literal),
        Array(ArrayExpression),
        Object(ObjectExpression),
        Function(FunctionExpression),
        ArrowFunction(ArrowFunction),
        This(#[rust_sitter::leaf(text = "this")] ()),
        Parenthesized(ParenthesizedExpression),
    }
    
    #[rust_sitter::word]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_$][a-zA-Z0-9_$]*")]
        pub name: String,
    }
    
    pub enum Literal {
        String(StringLiteral),
        Number(NumberLiteral),
        Boolean(BooleanLiteral),
        Null(#[rust_sitter::leaf(text = "null")] ()),
        Undefined(#[rust_sitter::leaf(text = "undefined")] ()),
    }
    
    pub enum StringLiteral {
        SingleQuoted(
            #[rust_sitter::leaf(pattern = r#"'([^'\\]|\\.)*'"#)]
            String
        ),
        DoubleQuoted(
            #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
            String
        ),
    }
    
    pub struct NumberLiteral {
        #[rust_sitter::leaf(pattern = r"(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?")]
        pub value: String,
    }
    
    pub enum BooleanLiteral {
        #[rust_sitter::leaf(text = "true")]
        True,
        #[rust_sitter::leaf(text = "false")]
        False,
    }
    
    pub struct ArrayExpression {
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub elements: Vec<Option<ArrayElement>>,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub enum ArrayElement {
        Expression(Expression),
        Spread(SpreadElement),
    }
    
    pub struct ObjectExpression {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub properties: Vec<Property>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub enum Property {
        Property(PropertyDefinition),
        Method(MethodDefinition),
        Spread(SpreadElement),
    }
    
    pub struct PropertyDefinition {
        pub key: PropertyKey,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub value: Expression,
    }
    
    pub struct MethodDefinition {
        pub key: PropertyKey,
        pub params: FormalParameters,
        pub body: BlockStatement,
    }
    
    pub enum PropertyKey {
        Identifier(Identifier),
        String(StringLiteral),
        Number(NumberLiteral),
        Computed(ComputedPropertyKey),
    }
    
    pub struct ComputedPropertyKey {
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        pub expression: Expression,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub struct FunctionExpression {
        #[rust_sitter::leaf(text = "function")]
        pub function_keyword: (),
        pub name: Option<Identifier>,
        pub parameters: FormalParameters,
        pub body: BlockStatement,
    }
    
    pub struct ArrowFunction {
        pub params: ArrowParameters,
        #[rust_sitter::leaf(text = "=>")]
        pub arrow: (),
        pub body: ArrowBody,
    }
    
    pub enum ArrowParameters {
        Identifier(Identifier),
        Formal(FormalParameters),
    }
    
    pub enum ArrowBody {
        Expression(Expression),
        Block(BlockStatement),
    }
    
    pub struct ParenthesizedExpression {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub expression: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
}

#[cfg(test)]
mod tests {
    use super::grammar::*;
    
    #[test]
    fn test_simple_program() {
        let input = "let x = 42;";
        let result = parse(input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_function_declaration() {
        let input = "function add(a, b) { return a + b; }";
        let result = parse(input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_arrow_function() {
        let input = "const add = (a, b) => a + b;";
        let result = parse(input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_object_literal() {
        let input = "const obj = { foo: 'bar', baz: 42 };";
        let result = parse(input);
        assert!(result.is_ok());
    }
}