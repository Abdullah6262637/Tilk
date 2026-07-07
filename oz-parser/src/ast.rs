pub type Span = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Bos,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedPart {
    Text(String),
    Expr(Box<Spanned<Expr>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(Option<Vec<String>>, String),
    Binary(Box<Spanned<Expr>>, BinaryOp, Box<Spanned<Expr>>),
    Unary(UnaryOp, Box<Spanned<Expr>>),
    Call(Option<Vec<String>>, String, Vec<Spanned<Expr>>),
    Array(Vec<Spanned<Expr>>),
    Map(Vec<(Spanned<Expr>, Spanned<Expr>)>),
    Index(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    HataIse(Box<Spanned<Expr>>, Vec<Spanned<Statement>>),
    InterpolatedString(Vec<InterpolatedPart>),
    Lambda {
        params: Vec<String>,
        body: Vec<Spanned<Statement>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepDir {
    Artarak,
    Azalarak,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Simple(String),
    Generic(String, Vec<Spanned<TypeAnnotation>>),
    Tuple(Vec<Spanned<TypeAnnotation>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl(String, Option<Spanned<TypeAnnotation>>, Spanned<Expr>),
    Assignment(String, Spanned<Expr>),
    IndexAssignment(Spanned<Expr>, Spanned<Expr>, Spanned<Expr>),
    If(
        Spanned<Expr>,
        Vec<Spanned<Statement>>,
        Option<Vec<Spanned<Statement>>>,
    ),
    While(Spanned<Expr>, Vec<Spanned<Statement>>),
    For {
        var: String,
        start: Spanned<Expr>,
        end: Spanned<Expr>,
        step_dir: StepDir,
        body: Vec<Spanned<Statement>>,
    },
    ForEach {
        var: String,
        iterable: Spanned<Expr>,
        body: Vec<Spanned<Statement>>,
    },
    FnDecl {
        name: String,
        generics: Vec<String>,
        params: Vec<(String, Option<Spanned<TypeAnnotation>>)>,
        return_type: Option<Spanned<TypeAnnotation>>,
        body: Vec<Spanned<Statement>>,
    },
    Return(Option<Spanned<Expr>>),
    Expr(Spanned<Expr>),
    Tamamlaninca(Spanned<Expr>, Vec<Spanned<Statement>>),
    Break,
    Continue,
}
