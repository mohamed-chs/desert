use std::ops::Range;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Simple(String),
    Generic(String, Vec<Type>),
    SharedRef(Box<Type>), // &
    UniqueRef(Box<Type>), // ~
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Expression>), // [1, 2, 3]
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Assign,
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    MatMul,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),
    Ident(String),
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    MacroCall(String, Vec<Expression>),
    MemberAccess(Box<Expression>, String),
    GenericCall(Box<Expression>, Vec<Type>, Vec<Expression>), // .collect[List[i32]]()
    Move(Box<Expression>),
    SharedRef(Box<Expression>),
    UniqueRef(Box<Expression>),
    Question(Box<Expression>),               // ?
    Unwrap(Box<Expression>),                 // !!
    Index(Box<Expression>, Box<Expression>), // expr[index]
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub is_mut: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Range<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StatementKind {
    Import {
        path: String,
        alias: Option<String>,
    },
    FromImport {
        path: String,
        items: Vec<ImportItem>,
    },
    Let {
        name: String,
        ty: Option<Type>,
        value: Expression,
    },
    Mut {
        name: String,
        ty: Option<Type>,
        value: Expression,
    },
    Def {
        name: String,
        params: Vec<Param>,
        return_ty: Option<Type>,
        body: Vec<Statement>,
    },
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    For {
        var: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Struct {
        name: String,
        fields: Vec<Param>,
    },
    Protocol {
        name: String,
        methods: Vec<Statement>, // These will likely be Def with empty bodies or signatures
    },
    Impl {
        protocol: Option<String>,
        for_type: String,
        methods: Vec<Statement>,
    },
    Match {
        expression: Expression,
        arms: Vec<(Expression, Vec<Statement>)>, // (pattern, body)
    },
    PyImport(String), // The whole block as a string for now
    Return(Option<Expression>),
    Expr(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}
