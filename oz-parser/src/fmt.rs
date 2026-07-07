use crate::ast::{BinaryOp, Expr, InterpolatedPart, Literal, Spanned, Statement, StepDir, UnaryOp};

pub struct Formatter {
    indent_level: usize,
    indent_size: usize,
}

impl Default for Formatter {
    fn default() -> Self {
        Self {
            indent_level: 0,
            indent_size: 4,
        }
    }
}

impl Formatter {
    pub fn new() -> Self {
        Self::default()
    }

    fn indent_str(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    pub fn format_program(&mut self, program: &[Spanned<Statement>]) -> String {
        let mut output = String::new();
        for (i, stmt) in program.iter().enumerate() {
            output.push_str(&self.format_statement(stmt));
            if i < program.len() - 1 {
                output.push('\n');
            }
        }
        output
    }

    fn format_statement(&mut self, stmt: &Spanned<Statement>) -> String {
        let ind = self.indent_str();
        match &stmt.node {
            Statement::VarDecl(name, expr) => {
                format!("{}{} = {};\n", ind, name, self.format_expr(expr))
            }
            Statement::Assignment(name, expr) => {
                format!("{}{} = {};\n", ind, name, self.format_expr(expr))
            }
            Statement::IndexAssignment(target, index, val) => {
                format!(
                    "{}{}[{}] = {};\n",
                    ind,
                    self.format_expr(target),
                    self.format_expr(index),
                    self.format_expr(val)
                )
            }
            Statement::Expr(expr) => {
                format!("{}{};\n", ind, self.format_expr(expr))
            }
            Statement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    format!("{}döndür {};\n", ind, self.format_expr(expr))
                } else {
                    format!("{}döndür;\n", ind)
                }
            }
            Statement::Break => format!("{}kır;\n", ind),
            Statement::Continue => format!("{}devam;\n", ind),
            Statement::If(cond, then_branch, else_branch) => {
                let mut out = format!("{}{} ise {{\n", ind, self.format_expr(cond));
                self.indent_level += 1;
                for s in then_branch {
                    out.push_str(&self.format_statement(s));
                }
                self.indent_level -= 1;
                if let Some(else_branch) = else_branch {
                    out.push_str(&format!("{}}} değilse {{\n", ind));
                    self.indent_level += 1;
                    for s in else_branch {
                        out.push_str(&self.format_statement(s));
                    }
                    self.indent_level -= 1;
                }
                out.push_str(&format!("{}}}\n", ind));
                out
            }
            Statement::While(cond, body) => {
                let mut out = format!("{}{} iken {{\n", ind, self.format_expr(cond));
                self.indent_level += 1;
                for s in body {
                    out.push_str(&self.format_statement(s));
                }
                self.indent_level -= 1;
                out.push_str(&format!("{}}}\n", ind));
                out
            }
            Statement::For {
                var,
                start,
                end,
                step_dir,
                body,
            } => {
                let dir_str = match step_dir {
                    StepDir::Artarak => "artarak",
                    StepDir::Azalarak => "azalarak",
                };
                let mut out = format!(
                    "{}{}, {}'den {}'e dek {} {{\n",
                    ind,
                    var,
                    self.format_expr(start),
                    self.format_expr(end),
                    dir_str
                );
                self.indent_level += 1;
                for s in body {
                    out.push_str(&self.format_statement(s));
                }
                self.indent_level -= 1;
                out.push_str(&format!("{}}}\n", ind));
                out
            }
            Statement::ForEach {
                var,
                iterable,
                body,
            } => {
                let mut out = format!(
                    "{}her {} {} içinde {{\n",
                    ind,
                    var,
                    self.format_expr(iterable)
                );
                self.indent_level += 1;
                for s in body {
                    out.push_str(&self.format_statement(s));
                }
                self.indent_level -= 1;
                out.push_str(&format!("{}}}\n", ind));
                out
            }
            Statement::FnDecl {
                name,
                generics: _, // TODO
                params,
                return_type: _, // TODO
                body,
            } => {
                let params_str = params
                    .iter()
                    .map(|(p_name, _)| p_name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut out = format!("{}işlev {}({}) {{\n", ind, name, params_str);
                self.indent_level += 1;
                for s in body {
                    out.push_str(&self.format_statement(s));
                }
                self.indent_level -= 1;
                out.push_str(&format!("{}}}\n", ind));
                out
            }
            Statement::Tamamlaninca(expr, body) => {
                let mut out = format!("{}{} tamamlanınca {{\n", ind, self.format_expr(expr));
                self.indent_level += 1;
                for s in body {
                    out.push_str(&self.format_statement(s));
                }
                self.indent_level -= 1;
                out.push_str(&format!("{}}}\n", ind));
                out
            }
        }
    }

    fn format_expr(&self, expr: &Spanned<Expr>) -> String {
        match &expr.node {
            Expr::Literal(Literal::Number(n)) => n.to_string(),
            Expr::Literal(Literal::String(s)) => format!("\"{}\"", s.escape_default()),
            Expr::Literal(Literal::Boolean(b)) => {
                if *b {
                    "doğru".to_string()
                } else {
                    "yanlış".to_string()
                }
            }
            Expr::Literal(Literal::Bos) => "boş".to_string(),
            Expr::Identifier(_, name) => name.clone(),
            Expr::Binary(left, op, right) => {
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::Ne => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::Le => "<=",
                    BinaryOp::Ge => ">=",
                    BinaryOp::And => "ve",
                    BinaryOp::Or => "veya",
                };
                format!(
                    "{} {} {}",
                    self.format_expr(left),
                    op_str,
                    self.format_expr(right)
                )
            }
            Expr::Unary(op, inner) => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "değil ",
                };
                format!("{}{}", op_str, self.format_expr(inner))
            }
            Expr::Call(_, name, args) => {
                let args_str = args
                    .iter()
                    .map(|a| self.format_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name, args_str)
            }
            Expr::Array(items) => {
                let items_str = items
                    .iter()
                    .map(|a| self.format_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items_str)
            }
            Expr::Map(pairs) => {
                let pairs_str = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", self.format_expr(k), self.format_expr(v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", pairs_str)
            }
            Expr::Index(target, index) => {
                format!("{}[{}]", self.format_expr(target), self.format_expr(index))
            }
            Expr::HataIse(target, body) => {
                // Actually HataIse is part of an expression? Wait, HataIse is typically attached to an expr like `do_sth() hata_ise { ... }`
                let mut out = format!("{} hata_ise {{", self.format_expr(target));
                // since it's an expr, inline or block depending on size. Let's do simple inline for now or block if there are statements.
                if body.is_empty() {
                    out.push('}');
                } else {
                    out.push('\n');
                    // formatting body in expression is tricky for indentation since expr doesn't keep mut self.
                    // we'll just format it simply.
                    // To do it correctly, format_expr would need `&mut self` to track indent. Let's assume we can create a temporary formatter.
                    let mut temp_fmt = Formatter {
                        indent_level: self.indent_level + 1,
                        indent_size: self.indent_size,
                    };
                    for s in body {
                        out.push_str(&temp_fmt.format_statement(s));
                    }
                    out.push_str(&format!("{}}}", self.indent_str()));
                }
                out
            }
            Expr::InterpolatedString(parts) => {
                let mut out = String::from("f\"");
                for part in parts {
                    match part {
                        InterpolatedPart::Text(t) => out.push_str(t),
                        InterpolatedPart::Expr(e) => {
                            out.push('{');
                            out.push_str(&self.format_expr(e));
                            out.push('}');
                        }
                    }
                }
                out.push('"');
                out
            }
            Expr::Lambda { params, body } => {
                let params_str = params.join(", ");
                let mut out = format!("|{}| {{", params_str);
                if body.is_empty() {
                    out.push('}');
                } else {
                    out.push('\n');
                    let mut temp_fmt = Formatter {
                        indent_level: self.indent_level + 1,
                        indent_size: self.indent_size,
                    };
                    for s in body {
                        out.push_str(&temp_fmt.format_statement(s));
                    }
                    out.push_str(&format!("{}}}", self.indent_str()));
                }
                out
            }
        }
    }
}
