use crate::instruction::{Instruction, Val};
use oz_parser::ast::{Expr, Statement, BinaryOp, Literal, StepDir};

pub struct Compiler {
    instructions: Vec<Instruction>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: Vec::new(),
        }
    }

    pub fn compile_program(mut self, stmts: &[Statement]) -> Result<Vec<Instruction>, String> {
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        Ok(self.instructions)
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => self.instructions.push(Instruction::Constant(Val::Number(*n))),
                Literal::String(s) => self.instructions.push(Instruction::Constant(Val::String(s.clone()))),
                Literal::Boolean(b) => self.instructions.push(Instruction::Constant(Val::Boolean(*b))),
                Literal::Bos => self.instructions.push(Instruction::Constant(Val::Bos)),
            },
            Expr::Identifier(name) => {
                self.instructions.push(Instruction::Load(name.clone()));
            }
            Expr::Binary(lhs, op, rhs) => {
                self.compile_expr(lhs)?;
                self.compile_expr(rhs)?;
                let inst = match op {
                    BinaryOp::Add => Instruction::Add,
                    BinaryOp::Sub => Instruction::Sub,
                    BinaryOp::Mul => Instruction::Mul,
                    BinaryOp::Div => Instruction::Div,
                    BinaryOp::Mod => Instruction::Mod,
                    BinaryOp::Eq => Instruction::Eq,
                    BinaryOp::Ne => Instruction::Ne,
                    BinaryOp::Lt => Instruction::Lt,
                    BinaryOp::Gt => Instruction::Gt,
                    BinaryOp::Le => Instruction::Le,
                    BinaryOp::Ge => Instruction::Ge,
                    BinaryOp::And => Instruction::And,
                    BinaryOp::Or => Instruction::Or,
                };
                self.instructions.push(inst);
            }
            Expr::Call(name, args) => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.instructions.push(Instruction::Load(name.clone()));
                self.instructions.push(Instruction::Call(args.len()));
            }
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::VarDecl(name, value) | Statement::Assignment(name, value) => {
                self.compile_expr(value)?;
                self.instructions.push(Instruction::Store(name.clone()));
            }
            Statement::Expr(expr) => {
                self.compile_expr(expr)?;
                self.instructions.push(Instruction::Pop);
            }
            Statement::If(cond, then_block, else_block) => {
                self.compile_expr(cond)?;
                let jump_false_idx = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(0));

                for s in then_block {
                    self.compile_stmt(s)?;
                }

                if let Some(else_block) = else_block {
                    let jump_end_idx = self.instructions.len();
                    self.instructions.push(Instruction::Jump(0));

                    let else_start = self.instructions.len();
                    self.instructions[jump_false_idx] = Instruction::JumpIfFalse(else_start);

                    for s in else_block {
                        self.compile_stmt(s)?;
                    }

                    let end_idx = self.instructions.len();
                    self.instructions[jump_end_idx] = Instruction::Jump(end_idx);
                } else {
                    let end_idx = self.instructions.len();
                    self.instructions[jump_false_idx] = Instruction::JumpIfFalse(end_idx);
                }
            }
            Statement::While(cond, body) => {
                let start_idx = self.instructions.len();
                self.compile_expr(cond)?;
                let jump_false_idx = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(0));

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.instructions.push(Instruction::Jump(start_idx));
                let end_idx = self.instructions.len();
                self.instructions[jump_false_idx] = Instruction::JumpIfFalse(end_idx);
            }
            Statement::For {
                var,
                start,
                end,
                step_dir,
                body,
            } => {
                self.compile_expr(start)?;
                self.instructions.push(Instruction::Store(var.clone()));

                let loop_start = self.instructions.len();

                self.instructions.push(Instruction::Load(var.clone()));
                self.compile_expr(end)?;
                match step_dir {
                    StepDir::Artarak => self.instructions.push(Instruction::Le),
                    StepDir::Azalarak => self.instructions.push(Instruction::Ge),
                }

                let jump_end_idx = self.instructions.len();
                self.instructions.push(Instruction::JumpIfFalse(0));

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.instructions.push(Instruction::Load(var.clone()));
                self.instructions.push(Instruction::Constant(Val::Number(1.0)));
                match step_dir {
                    StepDir::Artarak => self.instructions.push(Instruction::Add),
                    StepDir::Azalarak => self.instructions.push(Instruction::Sub),
                }
                self.instructions.push(Instruction::Store(var.clone()));

                self.instructions.push(Instruction::Jump(loop_start));

                let loop_end = self.instructions.len();
                self.instructions[jump_end_idx] = Instruction::JumpIfFalse(loop_end);
            }
            Statement::FnDecl { name, params, body } => {
                let jump_over_idx = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));

                let fn_start = self.instructions.len();
                for param in params.iter().rev() {
                    self.instructions.push(Instruction::Store(param.clone()));
                }

                for s in body {
                    self.compile_stmt(s)?;
                }
                self.instructions.push(Instruction::Constant(Val::Bos));
                self.instructions.push(Instruction::Return);

                let fn_end = self.instructions.len();
                self.instructions[jump_over_idx] = Instruction::Jump(fn_end);

                self.instructions.push(Instruction::Constant(Val::Function {
                    name: name.clone(),
                    param_count: params.len(),
                    entry_ip: fn_start,
                }));
                self.instructions.push(Instruction::Store(name.clone()));
            }
            Statement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    self.compile_expr(expr)?;
                } else {
                    self.instructions.push(Instruction::Constant(Val::Bos));
                }
                self.instructions.push(Instruction::Return);
            }
        }
        Ok(())
    }
}
