
#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Number(f64),
    String(String),
    Boolean(bool),
    Bos,
    Function {
        name: String,
        param_count: usize,
        entry_ip: usize,
    },
    Builtin(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Constant(Val),
    Load(String),
    Store(String),
    Pop,
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
    Jump(usize),
    JumpIfFalse(usize),
    Call(usize),
    Return,
}
