use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Bos,
    Array(Box<Type>),
    Map(Box<Type>),
    Function { params: Vec<Type>, ret: Box<Type> },
    Task(Box<Type>),
    Var(usize),
    Hata,
}

#[derive(Clone, Debug)]
pub struct Scheme {
    pub vars: Vec<usize>,
    pub ty: Type,
}

#[derive(Clone)]
pub struct TypeEnv {
    pub(crate) bindings: HashMap<String, Scheme>,
    pub(crate) parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TypeEnv {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn extend(parent: &TypeEnv) -> Self {
        TypeEnv {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent.clone())),
        }
    }

    pub fn get(&self, name: &str) -> Option<Scheme> {
        if let Some(scheme) = self.bindings.get(name) {
            Some(scheme.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, scheme: Scheme) {
        self.bindings.insert(name, scheme);
    }
}
