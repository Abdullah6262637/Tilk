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
    Channel(Box<Type>),
    Var(usize),
    Hata,
    Option(Box<Type>),
    Generic(String),
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

#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub message: String,
    pub span: Option<std::ops::Range<usize>>,
    pub expected: Option<Type>,
    pub found: Option<Type>,
}

impl TypeError {
    pub fn new(message: impl Into<String>) -> Self {
        TypeError {
            message: message.into(),
            span: None,
            expected: None,
            found: None,
        }
    }

    pub fn with_span(mut self, span: std::ops::Range<usize>) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_expected(mut self, expected: Type) -> Self {
        self.expected = Some(expected);
        self
    }

    pub fn with_found(mut self, found: Type) -> Self {
        self.found = Some(found);
        self
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<String> for TypeError {
    fn from(msg: String) -> Self {
        TypeError::new(msg)
    }
}

impl From<&str> for TypeError {
    fn from(msg: &str) -> Self {
        TypeError::new(msg)
    }
}

impl From<TypeError> for String {
    fn from(val: TypeError) -> Self {
        val.message
    }
}
