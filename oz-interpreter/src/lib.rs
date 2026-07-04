#![allow(clippy::new_without_default)]
#![allow(clippy::len_zero)]
pub mod builtins;

pub mod eval;
pub mod val;

#[cfg(test)]
mod tests;

pub use builtins::create_global_env;
pub use eval::{eval_expr, eval_program, eval_stmt};
pub use val::{Env, TaskState, Val};
