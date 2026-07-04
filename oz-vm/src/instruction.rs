use std::cell::RefCell;
use std::rc::Rc;

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
    Array(Rc<RefCell<Vec<Val>>>),
    Map(Rc<RefCell<std::collections::HashMap<String, Val>>>),
    Hata(String),
    Task(Rc<RefCell<TaskState>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskState {
    pub completed: bool,
    pub func: Val,
    pub args: Vec<Val>,
    pub result: Val,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Constant(Val),
    LoadLocal(u16),
    StoreLocal(u16),
    LoadGlobal(String),
    StoreGlobal(String),

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
    JumpIfError(usize),
    Call(usize),
    Return,
    Array(usize),
    Map(usize),
    Index,
    IndexStore,
    AwaitTask,
    Neg,
    Not,
    JumpIfFalseKeep(usize),
    JumpIfTrueKeep(usize),
}
