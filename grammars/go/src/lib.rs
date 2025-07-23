// Go grammar for rust-sitter
// Based on tree-sitter-go

#[rust_sitter::grammar("go")]
pub mod grammar {
    use rust_sitter::Spanned;
    
    #[rust_sitter::language]
    pub struct SourceFile {
        pub package_clause: PackageClause,
        pub import_declarations: Vec<ImportDeclaration>,
        pub top_level_declarations: Vec<TopLevelDeclaration>,
    }
    
    #[rust_sitter::extra]
    pub enum Extra {
        Whitespace(#[rust_sitter::leaf(pattern = r"\s+")] ()),
        LineComment(#[rust_sitter::leaf(pattern = r"//[^\n]*")] ()),
        BlockComment(#[rust_sitter::leaf(pattern = r"/\*([^*]|\*[^/])*\*/")] ()),
    }
    
    pub struct PackageClause {
        #[rust_sitter::leaf(text = "package")]
        pub package_keyword: (),
        pub name: Identifier,
    }
    
    pub struct ImportDeclaration {
        #[rust_sitter::leaf(text = "import")]
        pub import_keyword: (),
        pub specs: ImportSpec,
    }
    
    pub enum ImportSpec {
        Single(SingleImport),
        List(ImportList),
    }
    
    pub struct SingleImport {
        pub alias: Option<ImportAlias>,
        pub path: StringLiteral,
    }
    
    pub struct ImportList {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub imports: Vec<SingleImport>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub enum ImportAlias {
        Identifier(Identifier),
        Dot(#[rust_sitter::leaf(text = ".")] ()),
    }
    
    pub enum TopLevelDeclaration {
        Function(FunctionDeclaration),
        Method(MethodDeclaration),
        Type(TypeDeclaration),
        Var(VarDeclaration),
        Const(ConstDeclaration),
    }
    
    pub struct FunctionDeclaration {
        #[rust_sitter::leaf(text = "func")]
        pub func_keyword: (),
        #[rust_sitter::field("name")]
        pub name: Identifier,
        #[rust_sitter::field("parameters")]
        pub parameters: ParameterList,
        #[rust_sitter::field("result")]
        pub result: Option<Result>,
        #[rust_sitter::field("body")]
        pub body: Option<Block>,
    }
    
    pub struct MethodDeclaration {
        #[rust_sitter::leaf(text = "func")]
        pub func_keyword: (),
        #[rust_sitter::field("receiver")]
        pub receiver: ParameterList,
        #[rust_sitter::field("name")]
        pub name: Identifier,
        #[rust_sitter::field("parameters")]
        pub parameters: ParameterList,
        #[rust_sitter::field("result")]
        pub result: Option<Result>,
        #[rust_sitter::field("body")]
        pub body: Option<Block>,
    }
    
    pub struct ParameterList {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub parameters: Vec<ParameterDeclaration>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub struct ParameterDeclaration {
        pub identifiers: Vec<Identifier>,
        pub variadic: Option<#[rust_sitter::leaf(text = "...")] ()>,
        pub type_expr: Type,
    }
    
    pub enum Result {
        Type(Type),
        Parameters(ParameterList),
    }
    
    pub struct TypeDeclaration {
        #[rust_sitter::leaf(text = "type")]
        pub type_keyword: (),
        pub specs: TypeSpec,
    }
    
    pub enum TypeSpec {
        Single(SingleTypeSpec),
        List(TypeSpecList),
    }
    
    pub struct SingleTypeSpec {
        pub name: Identifier,
        pub type_expr: Type,
    }
    
    pub struct TypeSpecList {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub specs: Vec<SingleTypeSpec>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub struct VarDeclaration {
        #[rust_sitter::leaf(text = "var")]
        pub var_keyword: (),
        pub specs: VarSpec,
    }
    
    pub enum VarSpec {
        Single(SingleVarSpec),
        List(VarSpecList),
    }
    
    pub struct SingleVarSpec {
        pub identifiers: Vec<Identifier>,
        pub type_expr: Option<Type>,
        pub init: Option<VarInit>,
    }
    
    pub struct VarInit {
        #[rust_sitter::leaf(text = "=")]
        pub equals: (),
        pub expressions: Vec<Expression>,
    }
    
    pub struct VarSpecList {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub specs: Vec<SingleVarSpec>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub struct ConstDeclaration {
        #[rust_sitter::leaf(text = "const")]
        pub const_keyword: (),
        pub specs: ConstSpec,
    }
    
    pub enum ConstSpec {
        Single(SingleConstSpec),
        List(ConstSpecList),
    }
    
    pub struct SingleConstSpec {
        pub identifiers: Vec<Identifier>,
        pub type_expr: Option<Type>,
        pub init: Option<ConstInit>,
    }
    
    pub struct ConstInit {
        #[rust_sitter::leaf(text = "=")]
        pub equals: (),
        pub expressions: Vec<Expression>,
    }
    
    pub struct ConstSpecList {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub specs: Vec<SingleConstSpec>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub enum Type {
        #[rust_sitter::prec(1)]
        Identifier(TypeIdentifier),
        #[rust_sitter::prec(2)]
        Qualified(QualifiedType),
        #[rust_sitter::prec(3)]
        Pointer(Box<PointerType>),
        #[rust_sitter::prec(4)]
        Array(Box<ArrayType>),
        #[rust_sitter::prec(5)]
        Slice(Box<SliceType>),
        #[rust_sitter::prec(6)]
        Map(Box<MapType>),
        #[rust_sitter::prec(7)]
        Channel(Box<ChannelType>),
        #[rust_sitter::prec(8)]
        Function(Box<FunctionType>),
        #[rust_sitter::prec(9)]
        Interface(InterfaceType),
        #[rust_sitter::prec(10)]
        Struct(StructType),
        #[rust_sitter::prec(11)]
        Parenthesized(Box<ParenthesizedType>),
    }
    
    pub struct TypeIdentifier {
        pub name: Identifier,
    }
    
    pub struct QualifiedType {
        pub package: Identifier,
        #[rust_sitter::leaf(text = ".")]
        pub dot: (),
        pub name: Identifier,
    }
    
    pub struct PointerType {
        #[rust_sitter::leaf(text = "*")]
        pub star: (),
        pub type_expr: Type,
    }
    
    pub struct ArrayType {
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        pub length: Expression,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
        pub element: Type,
    }
    
    pub struct SliceType {
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
        pub element: Type,
    }
    
    pub struct MapType {
        #[rust_sitter::leaf(text = "map")]
        pub map_keyword: (),
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        pub key: Type,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
        pub value: Type,
    }
    
    pub struct ChannelType {
        pub dir: Option<ChannelDirection>,
        #[rust_sitter::leaf(text = "chan")]
        pub chan_keyword: (),
        pub element: Type,
    }
    
    pub enum ChannelDirection {
        Send(#[rust_sitter::leaf(text = "<-")] ()),
        Receive(ChannelReceive),
    }
    
    pub struct ChannelReceive {
        #[rust_sitter::leaf(text = "chan")]
        pub chan_keyword: (),
        #[rust_sitter::leaf(text = "<-")]
        pub arrow: (),
    }
    
    pub struct FunctionType {
        #[rust_sitter::leaf(text = "func")]
        pub func_keyword: (),
        pub parameters: ParameterList,
        pub result: Option<Result>,
    }
    
    pub struct InterfaceType {
        #[rust_sitter::leaf(text = "interface")]
        pub interface_keyword: (),
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        pub methods: Vec<InterfaceMethod>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub enum InterfaceMethod {
        Method(MethodSpec),
        Embedded(Type),
    }
    
    pub struct MethodSpec {
        pub name: Identifier,
        pub parameters: ParameterList,
        pub result: Option<Result>,
    }
    
    pub struct StructType {
        #[rust_sitter::leaf(text = "struct")]
        pub struct_keyword: (),
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        pub fields: Vec<FieldDeclaration>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub struct FieldDeclaration {
        pub names: Option<FieldNames>,
        pub type_expr: Type,
        pub tag: Option<StringLiteral>,
    }
    
    pub struct FieldNames {
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub names: Vec<Identifier>,
    }
    
    pub struct ParenthesizedType {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub type_expr: Type,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub enum Statement {
        Declaration(Declaration),
        Simple(SimpleStatement),
        Return(ReturnStatement),
        Break(BreakStatement),
        Continue(ContinueStatement),
        Goto(GotoStatement),
        Fallthrough(FallthroughStatement),
        Block(Block),
        If(IfStatement),
        Switch(SwitchStatement),
        Select(SelectStatement),
        For(ForStatement),
        Go(GoStatement),
        Defer(DeferStatement),
        Labeled(LabeledStatement),
        Empty(EmptyStatement),
    }
    
    pub enum Declaration {
        Const(ConstDeclaration),
        Type(TypeDeclaration),
        Var(VarDeclaration),
    }
    
    pub enum SimpleStatement {
        Expression(ExpressionStatement),
        Send(SendStatement),
        IncDec(IncDecStatement),
        Assignment(AssignmentStatement),
        ShortVar(ShortVarDeclaration),
    }
    
    pub struct ExpressionStatement {
        pub expression: Expression,
    }
    
    pub struct SendStatement {
        pub channel: Expression,
        #[rust_sitter::leaf(text = "<-")]
        pub arrow: (),
        pub value: Expression,
    }
    
    pub struct IncDecStatement {
        pub expression: Expression,
        pub op: IncDecOp,
    }
    
    pub enum IncDecOp {
        #[rust_sitter::leaf(text = "++")]
        Inc,
        #[rust_sitter::leaf(text = "--")]
        Dec,
    }
    
    pub struct AssignmentStatement {
        pub left: Vec<Expression>,
        pub op: AssignmentOp,
        pub right: Vec<Expression>,
    }
    
    pub enum AssignmentOp {
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
        #[rust_sitter::leaf(text = "&=")]
        AndAssign,
        #[rust_sitter::leaf(text = "|=")]
        OrAssign,
        #[rust_sitter::leaf(text = "^=")]
        XorAssign,
        #[rust_sitter::leaf(text = "<<=")]
        ShlAssign,
        #[rust_sitter::leaf(text = ">>=")]
        ShrAssign,
        #[rust_sitter::leaf(text = "&^=")]
        AndNotAssign,
    }
    
    pub struct ShortVarDeclaration {
        pub left: Vec<Expression>,
        #[rust_sitter::leaf(text = ":=")]
        pub op: (),
        pub right: Vec<Expression>,
    }
    
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        pub return_keyword: (),
        pub expressions: Vec<Expression>,
    }
    
    pub struct BreakStatement {
        #[rust_sitter::leaf(text = "break")]
        pub break_keyword: (),
        pub label: Option<Identifier>,
    }
    
    pub struct ContinueStatement {
        #[rust_sitter::leaf(text = "continue")]
        pub continue_keyword: (),
        pub label: Option<Identifier>,
    }
    
    pub struct GotoStatement {
        #[rust_sitter::leaf(text = "goto")]
        pub goto_keyword: (),
        pub label: Identifier,
    }
    
    pub struct FallthroughStatement {
        #[rust_sitter::leaf(text = "fallthrough")]
        pub fallthrough_keyword: (),
    }
    
    pub struct Block {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        pub statements: Vec<Statement>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        pub if_keyword: (),
        pub init: Option<Box<SimpleStatement>>,
        pub condition: Expression,
        pub consequence: Block,
        pub alternative: Option<ElseClause>,
    }
    
    pub enum ElseClause {
        Block(ElseBlock),
        If(Box<ElseIf>),
    }
    
    pub struct ElseBlock {
        #[rust_sitter::leaf(text = "else")]
        pub else_keyword: (),
        pub block: Block,
    }
    
    pub struct ElseIf {
        #[rust_sitter::leaf(text = "else")]
        pub else_keyword: (),
        pub if_statement: IfStatement,
    }
    
    pub struct SwitchStatement {
        #[rust_sitter::leaf(text = "switch")]
        pub switch_keyword: (),
        pub init: Option<Box<SimpleStatement>>,
        pub value: Option<Expression>,
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        pub cases: Vec<CaseClause>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub enum CaseClause {
        Case(ExpressionCase),
        Default(DefaultCase),
    }
    
    pub struct ExpressionCase {
        #[rust_sitter::leaf(text = "case")]
        pub case_keyword: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub expressions: Vec<Expression>,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub statements: Vec<Statement>,
    }
    
    pub struct DefaultCase {
        #[rust_sitter::leaf(text = "default")]
        pub default_keyword: (),
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub statements: Vec<Statement>,
    }
    
    pub struct SelectStatement {
        #[rust_sitter::leaf(text = "select")]
        pub select_keyword: (),
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        pub cases: Vec<CommClause>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub enum CommClause {
        Case(CommunicationCase),
        Default(DefaultCase),
    }
    
    pub struct CommunicationCase {
        #[rust_sitter::leaf(text = "case")]
        pub case_keyword: (),
        pub comm: Communication,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub statements: Vec<Statement>,
    }
    
    pub enum Communication {
        Send(SendStatement),
        Receive(ReceiveStatement),
    }
    
    pub struct ReceiveStatement {
        pub expressions: Option<Vec<Expression>>,
        pub op: Option<ReceiveOp>,
        pub channel: UnaryExpression,
    }
    
    pub enum ReceiveOp {
        #[rust_sitter::leaf(text = "=")]
        Assign,
        #[rust_sitter::leaf(text = ":=")]
        ShortAssign,
    }
    
    pub struct ForStatement {
        #[rust_sitter::leaf(text = "for")]
        pub for_keyword: (),
        pub clause: Option<ForClause>,
        pub body: Block,
    }
    
    pub enum ForClause {
        Condition(Expression),
        Traditional(TraditionalFor),
        Range(RangeFor),
    }
    
    pub struct TraditionalFor {
        pub init: Option<Box<SimpleStatement>>,
        #[rust_sitter::leaf(text = ";")]
        pub semi1: (),
        pub condition: Option<Expression>,
        #[rust_sitter::leaf(text = ";")]
        pub semi2: (),
        pub update: Option<Box<SimpleStatement>>,
    }
    
    pub struct RangeFor {
        pub left: Vec<Expression>,
        pub op: RangeOp,
        #[rust_sitter::leaf(text = "range")]
        pub range_keyword: (),
        pub right: Expression,
    }
    
    pub enum RangeOp {
        #[rust_sitter::leaf(text = "=")]
        Assign,
        #[rust_sitter::leaf(text = ":=")]
        ShortAssign,
    }
    
    pub struct GoStatement {
        #[rust_sitter::leaf(text = "go")]
        pub go_keyword: (),
        pub expression: Expression,
    }
    
    pub struct DeferStatement {
        #[rust_sitter::leaf(text = "defer")]
        pub defer_keyword: (),
        pub expression: Expression,
    }
    
    pub struct LabeledStatement {
        pub label: Identifier,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub statement: Box<Statement>,
    }
    
    pub struct EmptyStatement {
        #[rust_sitter::leaf(text = ";")]
        pub semicolon: (),
    }
    
    pub enum Expression {
        #[rust_sitter::prec_left(1)]
        Binary(Box<BinaryExpression>),
        #[rust_sitter::prec(14)]
        Unary(Box<UnaryExpression>),
        #[rust_sitter::prec_left(15)]
        Selector(Box<SelectorExpression>),
        #[rust_sitter::prec_left(15)]
        Index(Box<IndexExpression>),
        #[rust_sitter::prec_left(15)]
        Slice(Box<SliceExpression>),
        #[rust_sitter::prec_left(15)]
        Call(Box<CallExpression>),
        #[rust_sitter::prec_left(15)]
        TypeAssertion(Box<TypeAssertionExpression>),
        #[rust_sitter::prec(16)]
        Primary(PrimaryExpression),
    }
    
    pub struct BinaryExpression {
        pub left: Expression,
        pub op: BinaryOp,
        pub right: Expression,
    }
    
    pub enum BinaryOp {
        #[rust_sitter::prec_left(2)]
        #[rust_sitter::leaf(text = "||")]
        LogicalOr,
        #[rust_sitter::prec_left(3)]
        #[rust_sitter::leaf(text = "&&")]
        LogicalAnd,
        #[rust_sitter::prec_left(4)]
        #[rust_sitter::leaf(text = "==")]
        Equal,
        #[rust_sitter::prec_left(4)]
        #[rust_sitter::leaf(text = "!=")]
        NotEqual,
        #[rust_sitter::prec_left(4)]
        #[rust_sitter::leaf(text = "<")]
        Less,
        #[rust_sitter::prec_left(4)]
        #[rust_sitter::leaf(text = "<=")]
        LessEqual,
        #[rust_sitter::prec_left(4)]
        #[rust_sitter::leaf(text = ">")]
        Greater,
        #[rust_sitter::prec_left(4)]
        #[rust_sitter::leaf(text = ">=")]
        GreaterEqual,
        #[rust_sitter::prec_left(5)]
        #[rust_sitter::leaf(text = "+")]
        Add,
        #[rust_sitter::prec_left(5)]
        #[rust_sitter::leaf(text = "-")]
        Sub,
        #[rust_sitter::prec_left(5)]
        #[rust_sitter::leaf(text = "|")]
        Or,
        #[rust_sitter::prec_left(5)]
        #[rust_sitter::leaf(text = "^")]
        Xor,
        #[rust_sitter::prec_left(6)]
        #[rust_sitter::leaf(text = "*")]
        Mul,
        #[rust_sitter::prec_left(6)]
        #[rust_sitter::leaf(text = "/")]
        Div,
        #[rust_sitter::prec_left(6)]
        #[rust_sitter::leaf(text = "%")]
        Mod,
        #[rust_sitter::prec_left(6)]
        #[rust_sitter::leaf(text = "<<")]
        Shl,
        #[rust_sitter::prec_left(6)]
        #[rust_sitter::leaf(text = ">>")]
        Shr,
        #[rust_sitter::prec_left(6)]
        #[rust_sitter::leaf(text = "&")]
        And,
        #[rust_sitter::prec_left(6)]
        #[rust_sitter::leaf(text = "&^")]
        AndNot,
    }
    
    pub struct UnaryExpression {
        pub op: UnaryOp,
        pub operand: Expression,
    }
    
    pub enum UnaryOp {
        #[rust_sitter::leaf(text = "+")]
        Plus,
        #[rust_sitter::leaf(text = "-")]
        Minus,
        #[rust_sitter::leaf(text = "!")]
        Not,
        #[rust_sitter::leaf(text = "^")]
        Xor,
        #[rust_sitter::leaf(text = "*")]
        Deref,
        #[rust_sitter::leaf(text = "&")]
        Ref,
        #[rust_sitter::leaf(text = "<-")]
        Receive,
    }
    
    pub struct SelectorExpression {
        pub operand: Expression,
        #[rust_sitter::leaf(text = ".")]
        pub dot: (),
        pub field: Identifier,
    }
    
    pub struct IndexExpression {
        pub operand: Expression,
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        pub index: Expression,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub struct SliceExpression {
        pub operand: Expression,
        #[rust_sitter::leaf(text = "[")]
        pub open: (),
        pub start: Option<Expression>,
        #[rust_sitter::leaf(text = ":")]
        pub colon1: (),
        pub end: Option<Expression>,
        pub capacity: Option<SliceCapacity>,
        #[rust_sitter::leaf(text = "]")]
        pub close: (),
    }
    
    pub struct SliceCapacity {
        #[rust_sitter::leaf(text = ":")]
        pub colon2: (),
        pub capacity: Expression,
    }
    
    pub struct CallExpression {
        #[rust_sitter::field("function")]
        pub function: Expression,
        #[rust_sitter::field("arguments")]
        pub arguments: ArgumentList,
    }
    
    pub struct ArgumentList {
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub arguments: Vec<Argument>,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub enum Argument {
        Expression(Expression),
        Variadic(VariadicArgument),
    }
    
    pub struct VariadicArgument {
        pub expression: Expression,
        #[rust_sitter::leaf(text = "...")]
        pub dots: (),
    }
    
    pub struct TypeAssertionExpression {
        pub operand: Expression,
        #[rust_sitter::leaf(text = ".")]
        pub dot: (),
        #[rust_sitter::leaf(text = "(")]
        pub open: (),
        pub type_expr: Type,
        #[rust_sitter::leaf(text = ")")]
        pub close: (),
    }
    
    pub enum PrimaryExpression {
        Identifier(Identifier),
        Literal(Literal),
        Composite(CompositeLiteral),
        Function(FunctionLiteral),
        Parenthesized(ParenthesizedExpression),
    }
    
    #[rust_sitter::word]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }
    
    pub enum Literal {
        Int(IntLiteral),
        Float(FloatLiteral),
        Imaginary(ImaginaryLiteral),
        Rune(RuneLiteral),
        String(StringLiteral),
        RawString(RawStringLiteral),
        True(#[rust_sitter::leaf(text = "true")] ()),
        False(#[rust_sitter::leaf(text = "false")] ()),
        Nil(#[rust_sitter::leaf(text = "nil")] ()),
    }
    
    pub struct IntLiteral {
        #[rust_sitter::leaf(pattern = r"(0[xX][0-9a-fA-F]+|0[oO]?[0-7]+|0[bB][01]+|\d+)")]
        pub value: String,
    }
    
    pub struct FloatLiteral {
        #[rust_sitter::leaf(pattern = r"(\d+\.\d*|\.\d+)([eE][+-]?\d+)?")]
        pub value: String,
    }
    
    pub struct ImaginaryLiteral {
        #[rust_sitter::leaf(pattern = r"(\d+|\d+\.\d*|\.\d+)([eE][+-]?\d+)?i")]
        pub value: String,
    }
    
    pub struct RuneLiteral {
        #[rust_sitter::leaf(pattern = r"'([^'\\]|\\[\\/'\"abfnrtv]|\\[0-7]{3}|\\x[0-9a-fA-F]{2}|\\u[0-9a-fA-F]{4}|\\U[0-9a-fA-F]{8})'")]
        pub value: String,
    }
    
    pub struct StringLiteral {
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
        pub value: String,
    }
    
    pub struct RawStringLiteral {
        #[rust_sitter::leaf(pattern = r"`[^`]*`")]
        pub value: String,
    }
    
    pub struct CompositeLiteral {
        pub type_expr: Option<Type>,
        pub value: LiteralValue,
    }
    
    pub struct LiteralValue {
        #[rust_sitter::leaf(text = "{")]
        pub open: (),
        #[rust_sitter::delimited(#[rust_sitter::leaf(text = ",")] ())]
        pub elements: Vec<Element>,
        #[rust_sitter::leaf(text = "}")]
        pub close: (),
    }
    
    pub enum Element {
        Keyed(KeyedElement),
        Expression(Expression),
    }
    
    pub struct KeyedElement {
        pub key: FieldKey,
        #[rust_sitter::leaf(text = ":")]
        pub colon: (),
        pub value: Expression,
    }
    
    pub enum FieldKey {
        Name(Identifier),
        Expression(Expression),
    }
    
    pub struct FunctionLiteral {
        #[rust_sitter::leaf(text = "func")]
        pub func_keyword: (),
        pub parameters: ParameterList,
        pub result: Option<Result>,
        pub body: Block,
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
    fn test_hello_world() {
        let input = r#"
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
"#;
        let result = parse(input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_struct_definition() {
        let input = r#"
package main

type Person struct {
    Name string
    Age  int
}
"#;
        let result = parse(input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_interface() {
        let input = r#"
package main

type Writer interface {
    Write([]byte) (int, error)
}
"#;
        let result = parse(input);
        assert!(result.is_ok());
    }
}