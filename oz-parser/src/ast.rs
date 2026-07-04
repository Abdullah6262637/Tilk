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

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepDir {
    Artarak,
    Azalarak,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl(String, Expr),
    Assignment(String, Expr),
    If(Expr, Vec<Statement>, Option<Vec<Statement>>),
    While(Expr, Vec<Statement>),
    For {
        var: String,
        start: Expr,
        end: Expr,
        step_dir: StepDir,
        body: Vec<Statement>,
    },
    FnDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    Return(Option<Expr>),
    Expr(Expr),
}
