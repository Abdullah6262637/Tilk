use oz_parser::ast::{Spanned, Statement};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub enum Val {
    Number(f64),
    String(String),
    Boolean(bool),
    Bos,
    Function {
        params: Vec<String>,
        body: Vec<Spanned<Statement>>,
    },

    Builtin(Rc<dyn Fn(Vec<Val>) -> Val>),
    Array(Rc<RefCell<Vec<Val>>>),
    Map(Rc<RefCell<HashMap<String, Val>>>),
    Hata(String),
    Task(Rc<RefCell<TaskState>>),
}

#[derive(Clone)]
pub struct TaskState {
    pub completed: bool,
    pub func: Val,
    pub args: Vec<Val>,
    pub result: Val,
}

impl std::fmt::Debug for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Number(n) => write!(f, "Number({})", n),
            Val::String(s) => write!(f, "String({:?})", s),
            Val::Boolean(b) => write!(f, "Boolean({})", b),
            Val::Bos => write!(f, "Bos"),
            Val::Function { params, .. } => write!(f, "Function(params: {:?})", params),
            Val::Builtin(_) => write!(f, "Builtin"),
            Val::Array(arr) => {
                let items = arr.borrow();
                write!(f, "[")?;
                for (i, val) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", val)?;
                }
                write!(f, "]")
            }
            Val::Map(map) => {
                let items = map.borrow();
                write!(f, "{{")?;
                for (i, (key, val)) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}: {:?}", key, val)?;
                }
                write!(f, "}}")
            }
            Val::Hata(msg) => write!(f, "Hata({:?})", msg),
            Val::Task(_) => write!(f, "Task"),
        }
    }
}

impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Val::Number(a), Val::Number(b)) => a == b,
            (Val::String(a), Val::String(b)) => a == b,
            (Val::Boolean(a), Val::Boolean(b)) => a == b,
            (Val::Bos, Val::Bos) => true,
            (Val::Array(a), Val::Array(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (Val::Map(a), Val::Map(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (Val::Hata(a), Val::Hata(b)) => a == b,
            (Val::Task(a), Val::Task(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

pub struct EnvInner {
    pub(crate) bindings: HashMap<String, Val>,
    pub(crate) parent: Option<Rc<RefCell<EnvInner>>>,
}

#[derive(Clone)]
pub struct Env(pub(crate) Rc<RefCell<EnvInner>>);

impl Env {
    pub fn new() -> Self {
        Env(Rc::new(RefCell::new(EnvInner {
            bindings: HashMap::new(),
            parent: None,
        })))
    }

    pub fn extend(parent: &Self) -> Self {
        Env(Rc::new(RefCell::new(EnvInner {
            bindings: HashMap::new(),
            parent: Some(parent.0.clone()),
        })))
    }

    pub fn get(&self, name: &str) -> Option<Val> {
        let inner = self.0.borrow();
        if let Some(val) = inner.bindings.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &inner.parent {
            Env(parent.clone()).get(name)
        } else {
            None
        }
    }

    pub fn set(&self, name: String, val: Val) {
        if self.update_in_parent(&name, &val) {
            return;
        }
        self.0.borrow_mut().bindings.insert(name, val);
    }

    fn update_in_parent(&self, name: &str, val: &Val) -> bool {
        let mut inner = self.0.borrow_mut();
        if inner.bindings.contains_key(name) {
            inner.bindings.insert(name.to_string(), val.clone());
            true
        } else if let Some(parent) = &inner.parent {
            Env(parent.clone()).update_in_parent(name, val)
        } else {
            false
        }
    }
}
