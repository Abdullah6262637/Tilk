pub mod inference;
pub mod types;
pub mod unify;

pub use inference::{check_program, create_default_type_env, TypeChecker};
pub use types::{Scheme, Type, TypeEnv};
