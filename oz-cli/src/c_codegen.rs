#![allow(clippy::single_char_add_str)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::len_zero)]
use oz_parser::ast::{BinaryOp, Expr, Literal, Spanned, Statement, StepDir, UnaryOp};

use std::fs;

pub struct CCodegen {
    code: String,
}

impl CCodegen {
    pub fn new() -> Self {
        CCodegen {
            code: String::new(),
        }
    }

    pub fn transpile(mut self, stmts: &[Spanned<Statement>]) -> Result<String, String> {
        // C runtime header
        self.code.push_str(include_str!("tilk_runtime.h"));
        self.code.push_str("\n");
        self.code.push_str(include_str!("tilk_runtime.c"));
        self.code.push_str("\n");

        // Collect all function declarations
        let fn_decls = collect_function_decls(stmts);

        // Append forward declarations
        self.code.push_str("\n// Forward Declarations\n");
        for decl in &fn_decls {
            if let Statement::FnDecl { name, params, .. } = decl {
                self.code
                    .push_str(&format!("TilkVal {}(", sanitize_identifier(name)));
                for i in 0..params.len() {
                    if i > 0 {
                        self.code.push_str(", ");
                    }
                    self.code.push_str("TilkVal");
                }
                self.code.push_str(");\n");
            }
        }

        // Generate C function bodies for all collected declarations
        for decl in &fn_decls {
            if let Statement::FnDecl {
                name,
                generics: _,
                params,
                return_type: _,
                body,
            } = decl
            {
                let mut fn_code = format!("\nTilkVal {}(", sanitize_identifier(name));
                for (i, (p_name, _)) in params.iter().enumerate() {
                    if i > 0 {
                        fn_code.push_str(", ");
                    }
                    fn_code.push_str(&format!("TilkVal {}", sanitize_identifier(p_name)));
                }
                fn_code.push_str(") {\n");
                for s in body {
                    fn_code.push_str(&self.compile_stmt(s)?);
                }
                fn_code.push_str("    return make_bos();\n}\n");
                self.code.push_str(&fn_code);
            }
        }

        // Generate global main statements
        let mut main_body = String::new();
        for stmt in stmts {
            if let Statement::FnDecl { .. } = &stmt.node {
                continue;
            }
            main_body.push_str(&self.compile_stmt(stmt)?);
        }

        // main function
        self.code.push_str("\nint main() {\n");
        self.code.push_str(&main_body);
        self.code.push_str("    return 0;\n}\n");

        Ok(self.code)
    }

    fn compile_stmt(&self, stmt: &Spanned<Statement>) -> Result<String, String> {
        let mut out = String::new();
        match &stmt.node {
            Statement::VarDecl(name, expr) => {
                let expr_str = self.compile_expr(expr)?;
                out.push_str(&format!(
                    "    TilkVal {} = {};\n",
                    sanitize_identifier(name),
                    expr_str
                ));
            }
            Statement::Assignment(name, expr) => {
                let expr_str = self.compile_expr(expr)?;
                out.push_str(&format!(
                    "    {} = {};\n",
                    sanitize_identifier(name),
                    expr_str
                ));
            }
            Statement::IndexAssignment(target, idx, val) => {
                let target_str = self.compile_expr(target)?;
                let idx_str = self.compile_expr(idx)?;
                let val_str = self.compile_expr(val)?;
                out.push_str(&format!(
                    "    index_assign({}, {}, {});\n",
                    target_str, idx_str, val_str
                ));
            }
            Statement::If(cond, then_branch, else_branch) => {
                let cond_str = self.compile_expr(cond)?;
                out.push_str(&format!("    if ({}.val.boolean) {{\n", cond_str));
                for s in then_branch {
                    out.push_str(&self.compile_stmt(s)?);
                }
                out.push_str("    }");
                if let Some(eb) = else_branch {
                    out.push_str(" else {\n");
                    for s in eb {
                        out.push_str(&self.compile_stmt(s)?);
                    }
                    out.push_str("    }");
                }
                out.push_str("\n");
            }
            Statement::While(cond, body) => {
                let cond_str = self.compile_expr(cond)?;
                out.push_str(&format!("    while ({}.val.boolean) {{\n", cond_str));
                for s in body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                out.push_str("    }\n");
            }
            Statement::For {
                var,
                start,
                end,
                step_dir,
                body,
            } => {
                let start_str = self.compile_expr(start)?;
                let end_str = self.compile_expr(end)?;
                let s_var = sanitize_identifier(var);
                let start_var = format!("{}_start", s_var);
                let end_var = format!("{}_end", s_var);
                out.push_str(&format!("    TilkVal {} = {};\n", start_var, start_str));
                out.push_str(&format!("    TilkVal {} = {};\n", end_var, end_str));

                out.push_str(&format!(
                    "    for (double {} = {}.val.number; ",
                    s_var, start_var
                ));
                match step_dir {
                    StepDir::Artarak => {
                        out.push_str(&format!("{} <= {}.val.number; {}++", s_var, end_var, s_var))
                    }
                    StepDir::Azalarak => {
                        out.push_str(&format!("{} >= {}.val.number; {}--", s_var, end_var, s_var))
                    }
                }
                out.push_str(") {\n");
                out.push_str(&format!(
                    "        TilkVal {}_val = make_number({});\n",
                    s_var, s_var
                ));
                out.push_str(&format!("        TilkVal {} = {}_val;\n", s_var, s_var));
                for s in body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                out.push_str("    }\n");
            }
            Statement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let expr_str = self.compile_expr(expr)?;
                    out.push_str(&format!("    return {};\n", expr_str));
                } else {
                    out.push_str("    return make_bos();\n");
                }
            }
            Statement::Expr(expr) => {
                let expr_str = self.compile_expr(expr)?;
                out.push_str(&format!("    {};\n", expr_str));
            }
            Statement::Tamamlaninca(gorev, body) => {
                let gorev_str = self.compile_expr(gorev)?;
                out.push_str("    {\n");
                out.push_str(&format!("        TilkVal sonuc = {};\n", gorev_str));
                out.push_str("        TilkVal sonuc_val = sonuc;\n");
                out.push_str("        TilkVal sonuc = sonuc_val;\n");
                out.push_str("        TilkVal sonuç = sonuc_val;\n");
                for s in body {
                    out.push_str(&self.compile_stmt(s)?);
                }
                out.push_str("    }\n");
            }
            _ => {}
        }
        Ok(out)
    }

    fn compile_expr(&self, expr: &Spanned<Expr>) -> Result<String, String> {
        match &expr.node {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => Ok(format!("make_number({})", n)),
                Literal::String(s) => Ok(format!("make_string(\"{}\")", escape_string(s))),
                Literal::Boolean(b) => Ok(format!("make_boolean({})", b)),
                Literal::Bos => Ok("make_bos()".to_string()),
            },
            Expr::Identifier(prefix, name) => {
                let lookup_name = if let Some(p) = prefix {
                    format!("{}::{}", p.join("::"), name)
                } else {
                    name.clone()
                };
                Ok(sanitize_identifier(&lookup_name))
            }
            Expr::Unary(op, operand) => {
                let op_str = self.compile_expr(operand)?;
                let helper = match op {
                    UnaryOp::Neg => "neg_val",
                    UnaryOp::Not => "not_val",
                };
                Ok(format!("{}({})", helper, op_str))
            }
            Expr::Binary(lhs, op, rhs) => {
                let lhs_str = self.compile_expr(lhs)?;
                let rhs_str = self.compile_expr(rhs)?;
                let helper = match op {
                    BinaryOp::Add => "add_val",
                    BinaryOp::Sub => "sub_val",
                    BinaryOp::Mul => "mul_val",
                    BinaryOp::Div => "div_val",
                    BinaryOp::Mod => "mod_val",
                    BinaryOp::Eq => "eq_val",
                    BinaryOp::Ne => "ne_val",
                    BinaryOp::Lt => "lt_val",
                    BinaryOp::Gt => "gt_val",
                    BinaryOp::Le => "le_val",
                    BinaryOp::Ge => "ge_val",
                    BinaryOp::And => "and_val",
                    BinaryOp::Or => "or_val",
                };
                Ok(format!("{}({}, {})", helper, lhs_str, rhs_str))
            }
            Expr::Call(prefix, name, args) => {
                let lookup_name = if let Some(p) = prefix {
                    format!("{}::{}", p.join("::"), name)
                } else {
                    name.clone()
                };
                if lookup_name == "dahil_et" {
                    if let Some(Expr::Literal(Literal::String(path))) =
                        args.first().map(|s| &s.node)
                    {
                        let content = fs::read_to_string(path)
                            .map_err(|e| format!("Modül yüklenemedi ({}): {}", path, e))?;

                        use logos::Logos;
                        use oz_lexer::Token;
                        let lexer = Token::lexer(&content);
                        let mut tokens = Vec::new();
                        for (token_res, _) in lexer.spanned() {
                            if let Ok(token) = token_res {
                                tokens.push((token, 0..0));
                            }
                        }

                        let ast = oz_parser::parse_tokens(tokens, content.len())
                            .map_err(|e| format!("Ayrıştırma hatası: {:?}", e))?;

                        let mut inline_code = String::new();
                        for stmt in &ast {
                            inline_code.push_str(&self.compile_stmt(stmt)?);
                        }
                        return Ok(format!("({{\n{} make_bos();\n}})", inline_code));
                    }
                }

                if name == "arkaplanda_çalıştır" || name == "arkaplanda_calistir" {
                    if args.len() >= 1 {
                        let inner_call = self.compile_expr(&args[0])?;
                        return Ok(inner_call);
                    }
                }

                let mut args_str = String::new();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        args_str.push_str(", ");
                    }
                    args_str.push_str(&self.compile_expr(arg)?);
                }
                Ok(format!("{}({})", sanitize_identifier(name), args_str))
            }
            Expr::Array(elements) => {
                let mut args_str = String::new();
                for el in elements {
                    args_str.push_str(", ");
                    args_str.push_str(&self.compile_expr(el)?);
                }
                Ok(format!("create_array({}{})", elements.len(), args_str))
            }
            Expr::Map(elements) => {
                let mut args_str = String::new();
                for (k, v) in elements {
                    args_str.push_str(", ");
                    args_str.push_str(&self.compile_expr(k)?);
                    args_str.push_str(", ");
                    args_str.push_str(&self.compile_expr(v)?);
                }
                Ok(format!("create_map({}{})", elements.len(), args_str))
            }
            Expr::Index(array, idx) => {
                let array_str = self.compile_expr(array)?;
                let idx_str = self.compile_expr(idx)?;
                Ok(format!("index_val({}, {})", array_str, idx_str))
            }
            Expr::HataIse(base, body) => {
                let base_str = self.compile_expr(base)?;
                let mut body_code = String::new();
                for stmt in body {
                    body_code.push_str(&self.compile_stmt(stmt)?);
                }
                Ok(format!(
                    r#"({{
                    TilkVal base = {};
                    if (base.type == VAL_HATA) {{
                        TilkVal hata_mesajı = make_string(base.val.error);
                        TilkVal hata_mesaji = hata_mesajı;
                        {}
                    }}
                    base;
                }})"#,
                    base_str, body_code
                ))
            }
        }
    }
}

fn collect_function_decls(stmts: &[Spanned<Statement>]) -> Vec<Statement> {
    let mut decls = Vec::new();
    for stmt in stmts {
        match &stmt.node {
            Statement::FnDecl { .. } => {
                decls.push(stmt.node.clone());
            }
            Statement::If(_, then_branch, else_branch) => {
                decls.extend(collect_function_decls(then_branch));
                if let Some(eb) = else_branch {
                    decls.extend(collect_function_decls(eb));
                }
            }
            Statement::While(_, body) => {
                decls.extend(collect_function_decls(body));
            }
            Statement::For { body, .. } => {
                decls.extend(collect_function_decls(body));
            }
            Statement::Tamamlaninca(_, body) => {
                decls.extend(collect_function_decls(body));
            }
            Statement::Expr(spanned_expr) => {
                if let Expr::Call(prefix, name, args) = &spanned_expr.node {
                    if prefix.is_none() && name == "dahil_et" {
                        if let Some(spanned_arg) = args.first() {
                            if let Expr::Literal(oz_parser::ast::Literal::String(path)) =
                                &spanned_arg.node
                            {
                                if let Ok(content) = std::fs::read_to_string(path) {
                                    use logos::Logos;
                                    use oz_lexer::Token;
                                    let lexer = Token::lexer(&content);
                                    let mut tokens = Vec::new();
                                    for (token_res, _) in lexer.spanned() {
                                        if let Ok(token) = token_res {
                                            tokens.push((token, 0..0));
                                        }
                                    }
                                    if let Ok(ast) = oz_parser::parse_tokens(tokens, content.len())
                                    {
                                        decls.extend(collect_function_decls(&ast));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    decls
}

fn escape_string(s: &str) -> String {
    s.replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
}

fn sanitize_identifier(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ç' | 'Ç' => 'c',
            'ğ' | 'Ğ' => 'g',
            'ı' | 'İ' => 'i',
            'ö' | 'Ö' => 'o',
            'ş' | 'Ş' => 's',
            'ü' | 'Ü' => 'u',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;
    use oz_lexer::Token;

    #[test]
    fn test_codegen_basic() {
        let src = r#"
            sayı = 42;
            işlev topla(a, b) {
                döndür a + b;
            }
            sonuç = topla(sayı, 8);
        "#;
        let lexer = Token::lexer(src);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            if let Ok(token) = token_res {
                tokens.push((token, span));
            }
        }
        let ast = oz_parser::parse_tokens(tokens, src.len()).unwrap();
        let codegen = CCodegen::new();
        let c_code = codegen.transpile(&ast).unwrap();
        assert!(c_code.contains("TilkVal sayi = make_number(42);"));
        assert!(c_code.contains("TilkVal topla(TilkVal a, TilkVal b)"));
        assert!(c_code.contains("sonuc = topla(sayi, make_number(8))"));
    }

    #[test]
    fn test_codegen_channels() {
        let src = r#"
            iletim = kanal();
            iletim[0] = 42;
            deger = iletim[0];
        "#;
        let lexer = Token::lexer(src);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            if let Ok(token) = token_res {
                tokens.push((token, span));
            }
        }
        let ast = oz_parser::parse_tokens(tokens, src.len()).unwrap();
        let codegen = CCodegen::new();
        let c_code = codegen.transpile(&ast).unwrap();
        assert!(c_code.contains("TilkVal iletim = kanal();"));
        assert!(c_code.contains("index_assign(iletim, make_number(0), make_number(42));"));
        assert!(c_code.contains("TilkVal deger = index_val(iletim, make_number(0));"));
    }
}
