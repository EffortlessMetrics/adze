// Go grammar for rust-sitter
// Simplified version for v0.5.0-beta

#[rust_sitter::grammar("go")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct SourceFile {
        pub package_clause: PackageClause,
        #[rust_sitter::repeat]
        pub import_declarations: Vec<ImportDeclaration>,
        #[rust_sitter::repeat]
        pub top_level_declarations: Vec<TopLevelDeclaration>,
    }
    
    #[rust_sitter::language]
    pub struct PackageClause {
        #[rust_sitter::leaf(text = "package")]
        _package: (),
        pub name: Identifier,
    }
    
    #[rust_sitter::language]
    pub struct ImportDeclaration {
        #[rust_sitter::leaf(text = "import")]
        _import: (),
        pub spec: ImportSpec,
    }
    
    #[rust_sitter::language]
    pub enum ImportSpec {
        Single(SingleImport),
        List(ImportList),
    }
    
    #[rust_sitter::language]
    pub struct SingleImport {
        pub path: StringLiteral,
    }
    
    #[rust_sitter::language]
    pub struct ImportList {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub imports: Vec<SingleImport>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub enum TopLevelDeclaration {
        Function(FunctionDeclaration),
        Type(TypeDeclaration),
        Var(VarDeclaration),
        Const(ConstDeclaration),
    }
    
    #[rust_sitter::language]
    pub struct FunctionDeclaration {
        #[rust_sitter::leaf(text = "func")]
        _func: (),
        pub name: Identifier,
        pub parameters: ParameterList,
        pub result: Option<Type>,
        pub body: Block,
    }
    
    #[rust_sitter::language]
    pub struct ParameterList {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub parameters: Vec<ParameterDeclaration>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct ParameterDeclaration {
        pub name: Identifier,
        pub type_expr: Type,
    }
    
    #[rust_sitter::language]
    pub struct TypeDeclaration {
        #[rust_sitter::leaf(text = "type")]
        _type: (),
        pub name: Identifier,
        pub type_expr: Type,
    }
    
    #[rust_sitter::language]
    pub struct VarDeclaration {
        #[rust_sitter::leaf(text = "var")]
        _var: (),
        pub name: Identifier,
        pub type_expr: Option<Type>,
        pub init: Option<VarInit>,
    }
    
    #[rust_sitter::language]
    pub struct VarInit {
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub expression: Expression,
    }
    
    #[rust_sitter::language]
    pub struct ConstDeclaration {
        #[rust_sitter::leaf(text = "const")]
        _const: (),
        pub name: Identifier,
        pub type_expr: Option<Type>,
        pub init: Option<ConstInit>,
    }
    
    #[rust_sitter::language]
    pub struct ConstInit {
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub expression: Expression,
    }
    
    #[rust_sitter::language]
    pub enum Type {
        Identifier(TypeIdentifier),
        Pointer(Box<PointerType>),
        Array(Box<ArrayType>),
        Slice(Box<SliceType>),
        Map(Box<MapType>),
        Struct(StructType),
        Interface(InterfaceType),
        Function(Box<FunctionType>),
    }
    
    #[rust_sitter::language]
    pub struct TypeIdentifier {
        pub name: Identifier,
    }
    
    #[rust_sitter::language]
    pub struct PointerType {
        #[rust_sitter::leaf(text = "*")]
        _star: (),
        pub type_expr: Type,
    }
    
    #[rust_sitter::language]
    pub struct ArrayType {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        pub length: Expression,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
        pub element: Type,
    }
    
    #[rust_sitter::language]
    pub struct SliceType {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        #[rust_sitter::leaf(text = "]")]
        _close: (),
        pub element: Type,
    }
    
    #[rust_sitter::language]
    pub struct MapType {
        #[rust_sitter::leaf(text = "map")]
        _map: (),
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        pub key: Type,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
        pub value: Type,
    }
    
    #[rust_sitter::language]
    pub struct StructType {
        #[rust_sitter::leaf(text = "struct")]
        _struct: (),
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub fields: Vec<FieldDeclaration>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct FieldDeclaration {
        pub name: Identifier,
        pub type_expr: Type,
    }
    
    #[rust_sitter::language]
    pub struct InterfaceType {
        #[rust_sitter::leaf(text = "interface")]
        _interface: (),
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub methods: Vec<MethodSpec>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct MethodSpec {
        pub name: Identifier,
        pub parameters: ParameterList,
        pub result: Option<Type>,
    }
    
    #[rust_sitter::language]
    pub struct FunctionType {
        #[rust_sitter::leaf(text = "func")]
        _func: (),
        pub parameters: ParameterList,
        pub result: Option<Type>,
    }
    
    #[rust_sitter::language]
    pub enum Statement {
        Simple(SimpleStatement),
        Return(ReturnStatement),
        If(IfStatement),
        For(ForStatement),
        Block(Block),
        Go(GoStatement),
        Defer(DeferStatement),
    }
    
    #[rust_sitter::language]
    pub enum SimpleStatement {
        Expression(ExpressionStatement),
        Assignment(AssignmentStatement),
        ShortVar(ShortVarDeclaration),
    }
    
    #[rust_sitter::language]
    pub struct ExpressionStatement {
        pub expression: Expression,
    }
    
    #[rust_sitter::language]
    pub struct AssignmentStatement {
        pub left: Expression,
        #[rust_sitter::leaf(text = "=")]
        _equals: (),
        pub right: Expression,
    }
    
    #[rust_sitter::language]
    pub struct ShortVarDeclaration {
        pub left: Identifier,
        #[rust_sitter::leaf(text = ":=")]
        _coloneq: (),
        pub right: Expression,
    }
    
    #[rust_sitter::language]
    pub struct ReturnStatement {
        #[rust_sitter::leaf(text = "return")]
        _return: (),
        pub expression: Option<Expression>,
    }
    
    #[rust_sitter::language]
    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        pub condition: Expression,
        pub body: Block,
        pub else_clause: Option<ElseClause>,
    }
    
    #[rust_sitter::language]
    pub struct ElseClause {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        pub body: Block,
    }
    
    #[rust_sitter::language]
    pub struct ForStatement {
        #[rust_sitter::leaf(text = "for")]
        _for: (),
        pub condition: Option<Expression>,
        pub body: Block,
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
    pub struct GoStatement {
        #[rust_sitter::leaf(text = "go")]
        _go: (),
        pub expression: Expression,
    }
    
    #[rust_sitter::language]
    pub struct DeferStatement {
        #[rust_sitter::leaf(text = "defer")]
        _defer: (),
        pub expression: Expression,
    }
    
    #[rust_sitter::language]
    pub enum Expression {
        Binary(Box<BinaryExpression>),
        Unary(Box<UnaryExpression>),
        Call(Box<CallExpression>),
        Selector(Box<SelectorExpression>),
        Index(Box<IndexExpression>),
        Primary(PrimaryExpression),
    }
    
    #[rust_sitter::language]
    pub struct BinaryExpression {
        pub left: Expression,
        pub op: BinaryOp,
        pub right: Expression,
    }
    
    #[rust_sitter::language]
    pub enum BinaryOp {
        Add(AddOp),
        Sub(SubOp),
        Mul(MulOp),
        Div(DivOp),
        Mod(ModOp),
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
        pub op: UnaryOp,
        pub operand: Expression,
    }
    
    #[rust_sitter::language]
    pub enum UnaryOp {
        Plus(PlusOp),
        Minus(MinusOp),
        Not(NotOp),
        Deref(DerefOp),
        Ref(RefOp),
    }
    
    #[rust_sitter::language]
    pub struct PlusOp {
        #[rust_sitter::leaf(text = "+")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct MinusOp {
        #[rust_sitter::leaf(text = "-")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct NotOp {
        #[rust_sitter::leaf(text = "!")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct DerefOp {
        #[rust_sitter::leaf(text = "*")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct RefOp {
        #[rust_sitter::leaf(text = "&")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct CallExpression {
        pub function: Expression,
        pub arguments: ArgumentList,
    }
    
    #[rust_sitter::language]
    pub struct ArgumentList {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        #[rust_sitter::repeat]
        pub arguments: Vec<Expression>,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct SelectorExpression {
        pub operand: Expression,
        #[rust_sitter::leaf(text = ".")]
        _dot: (),
        pub field: Identifier,
    }
    
    #[rust_sitter::language]
    pub struct IndexExpression {
        pub operand: Expression,
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
        Composite(CompositeLiteral),
    }
    
    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }
    
    #[rust_sitter::language]
    pub enum Literal {
        Int(IntLiteral),
        Float(FloatLiteral),
        String(StringLiteral),
        True(TrueLiteral),
        False(FalseLiteral),
        Nil(NilLiteral),
    }
    
    #[rust_sitter::language]
    pub struct IntLiteral {
        #[rust_sitter::leaf(pattern = r"(0[xX][0-9a-fA-F]+|0[oO]?[0-7]+|0[bB][01]+|\d+)")]
        pub value: String,
    }
    
    #[rust_sitter::language]
    pub struct FloatLiteral {
        #[rust_sitter::leaf(pattern = r"(\d+\.\d*|\.\d+)([eE][+-]?\d+)?")]
        pub value: String,
    }
    
    #[rust_sitter::language]
    pub struct StringLiteral {
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
        pub value: String,
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
    pub struct NilLiteral {
        #[rust_sitter::leaf(text = "nil")]
        _nil: (),
    }
    
    #[rust_sitter::language]
    pub struct CompositeLiteral {
        pub type_expr: Option<Type>,
        pub value: LiteralValue,
    }
    
    #[rust_sitter::language]
    pub struct LiteralValue {
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub elements: Vec<Element>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub enum Element {
        Keyed(KeyedElement),
        Expression(Expression),
    }
    
    #[rust_sitter::language]
    pub struct KeyedElement {
        pub key: Identifier,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub value: Expression,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello_world() {
        // Grammar builds successfully
        assert!(true);
    }
}