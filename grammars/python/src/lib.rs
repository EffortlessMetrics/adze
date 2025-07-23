// Python grammar for rust-sitter
// Based on tree-sitter-python with indentation handling

#[rust_sitter::grammar("python")]
pub mod grammar {
    use rust_sitter::Spanned;
    
    #[rust_sitter::language]
    pub struct Module {
        pub statements: Vec<Statement>,
    }
    
    #[rust_sitter::extra]
    pub enum Extra {
        Whitespace(#[rust_sitter::leaf(pattern = r"[ \t]+")] ()),
        Comment(#[rust_sitter::leaf(pattern = r"#[^\n]*")] ()),
        LineContinuation(#[rust_sitter::leaf(pattern = r"\\\n")] ()),
    }
    
    // External tokens for indentation
    #[rust_sitter::external]
    pub enum ExternalToken {
        Newline,
        Indent,
        Dedent,
        StringStart,
        StringEnd,
    }
    
    pub enum Statement {
        Simple(SimpleStatement),
        Compound(CompoundStatement),
    }
    
    pub struct SimpleStatement {
        pub statement: SimpleStmt,
        pub newline: Newline,
    }
    
    pub enum SimpleStmt {
        Expression(ExpressionStatement),
        Assignment(Assignment),
        Return(ReturnStatement),
        Pass(PassStatement),
        Break(BreakStatement),
        Continue(ContinueStatement),
        Import(ImportStatement),
        Raise(RaiseStatement),
        Assert(AssertStatement),
        Del(DelStatement),
        Global(GlobalStatement),
        Nonlocal(NonlocalStatement),
    }
    
    pub enum CompoundStatement {
        Function(FunctionDefinition),
        Class(ClassDefinition),
        If(IfStatement),
        While(WhileStatement),
        For(ForStatement),
        Try(TryStatement),
        With(WithStatement),
    }
    
    pub struct ExpressionStatement {
        pub expression: Expression,
    }
    
    pub struct Assignment {
        pub targets: Vec<Expression>,
        #[rust_sitter::leaf(text = "=")]
        pub equals: (),
        pub value: Expression,
    }
    
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        pub return_keyword: (),
        pub value: Option<Expression>,
    }
    
    pub struct PassStatement {
        #[rust_sitter::leaf(text = "pass")]
        pub pass_keyword: (),
    }
    
    pub struct BreakStatement {
        #[rust_sitter::leaf(text = "break")]
        pub break_keyword: (),
    }
    
    pub struct ContinueStatement {
        #[rust_sitter::leaf(text = "continue")]
        pub continue_keyword: (),
    }
    
    pub struct ImportStatement {
        #[rust_sitter::leaf(text = "import")]
        pub import_keyword: (),
        pub modules: ImportList,
    }
    
    pub struct ImportList {
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub imports: Vec<ImportSpec>,
    }
    
    pub struct ImportSpec {
        pub module: DottedName,
        pub alias: Option<ImportAlias>,
    }
    
    pub struct ImportAlias {
        #[rust_sitter::leaf(text = "as")]
        pub as_keyword: (),
        pub name: Identifier,
    }
    
    pub struct DottedName {
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ".")] ())]
        pub parts: Vec<Identifier>,
    }
    
    pub struct RaiseStatement {
        #[rust_sitter::leaf(text = "raise")]
        pub raise_keyword: (),
        pub exception: Option<Expression>,
        pub cause: Option<RaiseCause>,
    }
    
    pub struct RaiseCause {
        #[rust_sitter::leaf(text = "from")]
        pub from_keyword: (),
        pub expression: Expression,
    }
    
    pub struct AssertStatement {
        #[rust_sitter::leaf(text = "assert")]
        pub assert_keyword: (),
        pub test: Expression,
        pub message: Option<AssertMessage>,
    }
    
    pub struct AssertMessage {
        #[rust_sitter::leaf(text = ",")]
        pub comma: (),
        pub expression: Expression,
    }
    
    pub struct DelStatement {
        #[rust_sitter::leaf(text = "del")]
        pub del_keyword: (),
        pub targets: Vec<Expression>,
    }
    
    pub struct GlobalStatement {
        #[rust_sitter::leaf(text = "global")]
        pub global_keyword: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub names: Vec<Identifier>,
    }
    
    pub struct NonlocalStatement {
        #[rust_sitter::leaf(text = "nonlocal")]
        pub nonlocal_keyword: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub names: Vec<Identifier>,
    }
    
    pub struct FunctionDefinition {
        pub decorators: Vec<Decorator>,
        #[rust_sitter::leaf(text = "def")]
        pub def_keyword: (),
        #[rust_sitter::field("name")]
        pub name: Identifier,
        #[rust_sitter::field("parameters")]
        pub parameters: Parameters,
        pub return_annotation: Option<TypeAnnotation>,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        #[rust_sitter::field("body")]
        pub body: Suite,
    }
    
    pub struct Decorator {
        #[rust_sitter::leaf(text = "@")]
        pub at: (),
        pub expression: Expression,
        pub newline: Newline,
    }
    
    pub struct Parameters {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub params: Vec<Parameter>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub enum Parameter {
        Simple(SimpleParameter),
        Default(DefaultParameter),
        Args(StarParameter),
        Kwargs(DoubleStarParameter),
    }
    
    pub struct SimpleParameter {
        pub name: Identifier,
        pub annotation: Option<ParameterAnnotation>,
    }
    
    pub struct DefaultParameter {
        pub name: Identifier,
        pub annotation: Option<ParameterAnnotation>,
        #[rust_sitter::leaf(text = "=")]
        pub equals: (),
        pub default: Expression,
    }
    
    pub struct StarParameter {
        #[rust_sitter::leaf(text = "*")]
        pub star: (),
        pub name: Option<Identifier>,
    }
    
    pub struct DoubleStarParameter {
        #[rust_sitter::leaf(text = "**")]
        pub stars: (),
        pub name: Identifier,
    }
    
    pub struct ParameterAnnotation {
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub type_expr: Expression,
    }
    
    pub struct TypeAnnotation {
        #[rust_sitter::leaf(text = "->")]
        pub arrow: (),
        pub type_expr: Expression,
    }
    
    pub struct Suite {
        pub newline: Newline,
        pub indent: Indent,
        pub statements: Vec<Statement>,
        pub dedent: Dedent,
    }
    
    pub struct ClassDefinition {
        pub decorators: Vec<Decorator>,
        #[rust_sitter::leaf(text = "class")]
        pub class_keyword: (),
        #[rust_sitter::field("name")]
        pub name: Identifier,
        pub bases: Option<ClassBases>,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        #[rust_sitter::field("body")]
        pub body: Suite,
    }
    
    pub struct ClassBases {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub arguments: Vec<Argument>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        pub if_keyword: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
        pub elif_clauses: Vec<ElifClause>,
        pub else_clause: Option<ElseClause>,
    }
    
    pub struct ElifClause {
        #[rust_sitter::leaf(text = "elif")]
        pub elif_keyword: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
    }
    
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        pub else_keyword: (),
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
    }
    
    pub struct WhileStatement {
        #[rust_sitter::leaf(text = "while")]
        pub while_keyword: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
        pub else_clause: Option<ElseClause>,
    }
    
    pub struct ForStatement {
        #[rust_sitter::leaf(text = "for")]
        pub for_keyword: (),
        pub target: Expression,
        #[rust_sitter::leaf(text = "in")]
        pub in_keyword: (),
        pub iter: Expression,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
        pub else_clause: Option<ElseClause>,
    }
    
    pub struct TryStatement {
        #[rust_sitter::leaf(text = "try")]
        pub try_keyword: (),
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
        pub except_clauses: Vec<ExceptClause>,
        pub else_clause: Option<ElseClause>,
        pub finally_clause: Option<FinallyClause>,
    }
    
    pub struct ExceptClause {
        #[rust_sitter::leaf(text = "except")]
        pub except_keyword: (),
        pub exception: Option<ExceptionSpec>,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
    }
    
    pub struct ExceptionSpec {
        pub type_expr: Expression,
        pub name: Option<ExceptionAlias>,
    }
    
    pub struct ExceptionAlias {
        #[rust_sitter::leaf(text = "as")]
        pub as_keyword: (),
        pub name: Identifier,
    }
    
    pub struct FinallyClause {
        #[rust_sitter::leaf(text = "finally")]
        pub finally_keyword: (),
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
    }
    
    pub struct WithStatement {
        #[rust_sitter::leaf(text = "with")]
        pub with_keyword: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub items: Vec<WithItem>,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Suite,
    }
    
    pub struct WithItem {
        pub expression: Expression,
        pub alias: Option<WithAlias>,
    }
    
    pub struct WithAlias {
        #[rust_sitter::leaf(text = "as")]
        pub as_keyword: (),
        pub target: Expression,
    }
    
    pub enum Expression {
        #[rust_sitter::prec(1)]
        Lambda(Box<LambdaExpression>),
        #[rust_sitter::prec(2)]
        Conditional(Box<ConditionalExpression>),
        #[rust_sitter::prec_left(3)]
        Or(Box<BinaryExpression>),
        #[rust_sitter::prec_left(4)]
        And(Box<BinaryExpression>),
        #[rust_sitter::prec(5)]
        Not(Box<UnaryExpression>),
        #[rust_sitter::prec(6)]
        Comparison(Box<ComparisonExpression>),
        #[rust_sitter::prec_left(7)]
        BitwiseOr(Box<BinaryExpression>),
        #[rust_sitter::prec_left(8)]
        BitwiseXor(Box<BinaryExpression>),
        #[rust_sitter::prec_left(9)]
        BitwiseAnd(Box<BinaryExpression>),
        #[rust_sitter::prec_left(10)]
        Shift(Box<BinaryExpression>),
        #[rust_sitter::prec_left(11)]
        Add(Box<BinaryExpression>),
        #[rust_sitter::prec_left(12)]
        Multiply(Box<BinaryExpression>),
        #[rust_sitter::prec(13)]
        Unary(Box<UnaryExpression>),
        #[rust_sitter::prec_right(14)]
        Power(Box<BinaryExpression>),
        #[rust_sitter::prec_left(15)]
        Call(Box<CallExpression>),
        #[rust_sitter::prec_left(15)]
        Subscript(Box<SubscriptExpression>),
        #[rust_sitter::prec_left(15)]
        Attribute(Box<AttributeExpression>),
        #[rust_sitter::prec(16)]
        Primary(PrimaryExpression),
    }
    
    pub struct LambdaExpression {
        #[rust_sitter::leaf(text = "lambda")]
        pub lambda_keyword: (),
        pub parameters: Option<LambdaParameters>,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub body: Expression,
    }
    
    pub struct LambdaParameters {
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub params: Vec<Identifier>,
    }
    
    pub struct ConditionalExpression {
        pub body: Expression,
        #[rust_sitter::leaf(text = "if")]
        pub if_keyword: (),
        pub condition: Expression,
        #[rust_sitter::leaf(text = "else")]
        pub else_keyword: (),
        pub orelse: Expression,
    }
    
    pub struct BinaryExpression {
        pub left: Expression,
        pub operator: BinaryOperator,
        pub right: Expression,
    }
    
    pub enum BinaryOperator {
        #[rust_sitter::leaf(text = "or")]
        Or,
        #[rust_sitter::leaf(text = "and")]
        And,
        #[rust_sitter::leaf(text = "|")]
        BitwiseOr,
        #[rust_sitter::leaf(text = "^")]
        BitwiseXor,
        #[rust_sitter::leaf(text = "&")]
        BitwiseAnd,
        #[rust_sitter::leaf(text = "<<")]
        LeftShift,
        #[rust_sitter::leaf(text = ">>")]
        RightShift,
        #[rust_sitter::leaf(text = "+")]
        Add,
        #[rust_sitter::leaf(text = "-")]
        Subtract,
        #[rust_sitter::leaf(text = "*")]
        Multiply,
        #[rust_sitter::leaf(text = "/")]
        Divide,
        #[rust_sitter::leaf(text = "//")]
        FloorDivide,
        #[rust_sitter::leaf(text = "%")]
        Modulo,
        #[rust_sitter::leaf(text = "**")]
        Power,
        #[rust_sitter::leaf(text = "@")]
        MatMul,
    }
    
    pub struct UnaryExpression {
        pub operator: UnaryOperator,
        pub operand: Expression,
    }
    
    pub enum UnaryOperator {
        #[rust_sitter::leaf(text = "not")]
        Not,
        #[rust_sitter::leaf(text = "+")]
        Plus,
        #[rust_sitter::leaf(text = "-")]
        Minus,
        #[rust_sitter::leaf(text = "~")]
        BitwiseNot,
    }
    
    pub struct ComparisonExpression {
        pub left: Expression,
        pub comparisons: Vec<Comparison>,
    }
    
    pub struct Comparison {
        pub operator: ComparisonOperator,
        pub right: Expression,
    }
    
    pub enum ComparisonOperator {
        #[rust_sitter::leaf(text = "<")]
        Less,
        #[rust_sitter::leaf(text = ">")]
        Greater,
        #[rust_sitter::leaf(text = "<=")]
        LessEqual,
        #[rust_sitter::leaf(text = ">=")]
        GreaterEqual,
        #[rust_sitter::leaf(text = "==")]
        Equal,
        #[rust_sitter::leaf(text = "!=")]
        NotEqual,
        #[rust_sitter::leaf(text = "is")]
        Is,
        #[rust_sitter::leaf(text = "is not")]
        IsNot,
        #[rust_sitter::leaf(text = "in")]
        In,
        #[rust_sitter::leaf(text = "not in")]
        NotIn,
    }
    
    pub struct CallExpression {
        pub function: Expression,
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
        Positional(Expression),
        Keyword(KeywordArgument),
        Star(StarArgument),
        DoubleStar(DoubleStarArgument),
    }
    
    pub struct KeywordArgument {
        pub name: Identifier,
        #[rust_sitter::leaf(text = "=")]
        pub equals: (),
        pub value: Expression,
    }
    
    pub struct StarArgument {
        #[rust_sitter::leaf(text = "*")]
        pub star: (),
        pub expression: Expression,
    }
    
    pub struct DoubleStarArgument {
        #[rust_sitter::leaf(text = "**")]
        pub stars: (),
        pub expression: Expression,
    }
    
    pub struct SubscriptExpression {
        pub value: Expression,
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        pub slice: Slice,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub enum Slice {
        Index(Expression),
        Slice(SliceExpression),
    }
    
    pub struct SliceExpression {
        pub lower: Option<Expression>,
        #[rust_sitter::leaf(text = ":")]
        pub colon1: (),
        pub upper: Option<Expression>,
        pub step: Option<SliceStep>,
    }
    
    pub struct SliceStep {
        #[rust_sitter::leaf(text = ":")]
        pub colon2: (),
        pub step: Expression,
    }
    
    pub struct AttributeExpression {
        pub value: Expression,
        #[rust_sitter::leaf(text = ".")]
        pub dot: (),
        pub attr: Identifier,
    }
    
    pub enum PrimaryExpression {
        Identifier(Identifier),
        Literal(Literal),
        List(ListExpression),
        Tuple(TupleExpression),
        Dict(DictExpression),
        Set(SetExpression),
        Comprehension(Comprehension),
        Parenthesized(ParenthesizedExpression),
    }
    
    #[rust_sitter::word]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }
    
    pub enum Literal {
        String(StringLiteral),
        Integer(IntegerLiteral),
        Float(FloatLiteral),
        Boolean(BooleanLiteral),
        None(#[rust_sitter::leaf(text = "None")] ()),
    }
    
    pub enum StringLiteral {
        Simple(SimpleString),
        Format(FormatString),
        Raw(RawString),
        Bytes(BytesLiteral),
    }
    
    pub enum SimpleString {
        SingleQuoted(#[rust_sitter::leaf(pattern = r#"'([^'\\]|\\.)*'"#)] String),
        DoubleQuoted(#[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)] String),
        TripleSingleQuoted(#[rust_sitter::leaf(pattern = r#"'''(.|\\n)*?'''"#)] String),
        TripleDoubleQuoted(#[rust_sitter::leaf(pattern = r#"\"\"\"(.|\\n)*?\"\"\""#)] String),
    }
    
    pub struct FormatString {
        pub prefix: FormatPrefix,
        pub content: SimpleString,
    }
    
    pub enum FormatPrefix {
        #[rust_sitter::leaf(text = "f")]
        F,
        #[rust_sitter::leaf(text = "F")]
        CapitalF,
    }
    
    pub struct RawString {
        pub prefix: RawPrefix,
        pub content: SimpleString,
    }
    
    pub enum RawPrefix {
        #[rust_sitter::leaf(text = "r")]
        R,
        #[rust_sitter::leaf(text = "R")]
        CapitalR,
    }
    
    pub struct BytesLiteral {
        pub prefix: BytesPrefix,
        pub content: SimpleString,
    }
    
    pub enum BytesPrefix {
        #[rust_sitter::leaf(text = "b")]
        B,
        #[rust_sitter::leaf(text = "B")]
        CapitalB,
    }
    
    pub struct IntegerLiteral {
        #[rust_sitter::leaf(pattern = r"(0[xX][0-9a-fA-F]+|0[oO][0-7]+|0[bB][01]+|\d+)")]
        pub value: String,
    }
    
    pub struct FloatLiteral {
        #[rust_sitter::leaf(pattern = r"(\d+\.\d*|\.\d+)([eE][+-]?\d+)?")]
        pub value: String,
    }
    
    pub enum BooleanLiteral {
        #[rust_sitter::leaf(text = "True")]
        True,
        #[rust_sitter::leaf(text = "False")]
        False,
    }
    
    pub struct ListExpression {
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub elements: Vec<Expression>,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub struct TupleExpression {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub elements: Vec<Expression>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub struct DictExpression {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub items: Vec<DictItem>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub enum DictItem {
        Pair(DictPair),
        DoubleStar(DoubleStarDict),
    }
    
    pub struct DictPair {
        pub key: Expression,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub value: Expression,
    }
    
    pub struct DoubleStarDict {
        #[rust_sitter::leaf(text = "**")]
        pub stars: (),
        pub expression: Expression,
    }
    
    pub struct SetExpression {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub elements: Vec<Expression>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub enum Comprehension {
        List(ListComprehension),
        Set(SetComprehension),
        Dict(DictComprehension),
        Generator(GeneratorExpression),
    }
    
    pub struct ListComprehension {
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        pub element: Expression,
        pub generators: Vec<ComprehensionGenerator>,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub struct SetComprehension {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        pub element: Expression,
        pub generators: Vec<ComprehensionGenerator>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub struct DictComprehension {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        pub key: Expression,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub value: Expression,
        pub generators: Vec<ComprehensionGenerator>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub struct GeneratorExpression {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub element: Expression,
        pub generators: Vec<ComprehensionGenerator>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub struct ComprehensionGenerator {
        #[rust_sitter::leaf(text = "for")]
        pub for_keyword: (),
        pub target: Expression,
        #[rust_sitter::leaf(text = "in")]
        pub in_keyword: (),
        pub iter: Expression,
        pub conditions: Vec<ComprehensionCondition>,
    }
    
    pub struct ComprehensionCondition {
        #[rust_sitter::leaf(text = "if")]
        pub if_keyword: (),
        pub test: Expression,
    }
    
    pub struct ParenthesizedExpression {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub expression: Box<Expression>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    // External token types handled by scanner
    pub struct Newline;
    pub struct Indent;
    pub struct Dedent;
}