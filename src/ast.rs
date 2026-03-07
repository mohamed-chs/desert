use std::ops::Range;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Simple(String),
    Generic(String, Vec<Type>),
    Tuple(Vec<Type>),
    SharedRef(Box<Type>), // &
    UniqueRef(Box<Type>), // ~
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Expression>), // [1, 2, 3]
    Dict(Vec<(Expression, Expression)>), // {"a": 1, "b": 2}
    Set(Vec<Expression>), // {1, 2, 3}
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    And,
    Or,
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
    Not(Box<Expression>),
    Question(Box<Expression>),                        // ?
    Unwrap(Box<Expression>),                          // !!
    Index(Box<Expression>, Box<Expression>),          // expr[index]
    Tuple(Vec<Expression>),                           // (a, b, c)
    Range(Box<Expression>, Box<Expression>),          // a..b
    RangeInclusive(Box<Expression>, Box<Expression>), // a..=b
    Lambda {
        params: Vec<Param>,
        body: Box<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Name(String),
    Tuple(Vec<Pattern>),
}

impl Pattern {
    pub fn as_name(&self) -> Option<&str> {
        match self {
            Pattern::Name(name) => Some(name),
            _ => None,
        }
    }

    pub fn names(&self) -> Vec<&str> {
        match self {
            Pattern::Name(name) => vec![name.as_str()],
            Pattern::Tuple(pats) => pats.iter().flat_map(|p| p.names()).collect(),
        }
    }

    pub fn format_desert(&self) -> String {
        match self {
            Pattern::Name(name) => name.clone(),
            Pattern::Tuple(pats) => format!(
                "({})",
                pats.iter()
                    .map(|p| p.format_desert())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
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
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
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
        pattern: Pattern,
        ty: Option<Type>,
        value: Expression,
    },
    Mut {
        pattern: Pattern,
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
        pattern: Pattern,
        iterable: Expression,
        body: Vec<Statement>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Struct {
        name: String,
        fields: Vec<Param>,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
    Protocol {
        name: String,
        methods: Vec<Statement>,
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
    PyImport(String),
    Return(Option<Expression>),
    Break,
    Continue,
    Expr(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}
