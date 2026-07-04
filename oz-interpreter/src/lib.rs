use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use oz_parser::ast::{Expr, Statement, BinaryOp, Literal, StepDir};

#[derive(Clone)]
pub enum Val {
    Number(f64),
    String(String),
    Boolean(bool),
    Bos,
    Function {
        params: Vec<String>,
        body: Vec<Statement>,
    },
    Builtin(Rc<dyn Fn(Vec<Val>) -> Val>),
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
            _ => false,
        }
    }
}

pub struct EnvInner {
    bindings: HashMap<String, Val>,
    parent: Option<Rc<RefCell<EnvInner>>>,
}

#[derive(Clone)]
pub struct Env(Rc<RefCell<EnvInner>>);

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

pub fn create_global_env() -> Env {
    let env = Env::new();
    // Default built-in function "yazdır"
    env.set(
        "yazdır".to_string(),
        Val::Builtin(Rc::new(|args| {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                match arg {
                    Val::Number(n) => print!("{}", n),
                    Val::String(s) => print!("{}", s),
                    Val::Boolean(b) => print!("{}", if *b { "doğru" } else { "yanlış" }),
                    Val::Bos => print!("boş"),
                    _ => print!("{:?}", arg),
                }
            }
            println!();
            Val::Bos
        })),
    );
    env
}

pub fn eval_expr(expr: &Expr, env: &Env) -> Result<Val, String> {
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::Number(n) => Ok(Val::Number(*n)),
            Literal::String(s) => Ok(Val::String(s.clone())),
            Literal::Boolean(b) => Ok(Val::Boolean(*b)),
            Literal::Bos => Ok(Val::Bos),
        },
        Expr::Identifier(name) => env
            .get(name)
            .ok_or_else(|| format!("HATA: Tanımlanamayan değişken '{}'", name)),
        Expr::Binary(lhs, op, rhs) => {
            let left = eval_expr(lhs, env)?;
            let right = eval_expr(rhs, env)?;
            match op {
                BinaryOp::Add => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Number(a + b)),
                    (Val::String(a), Val::String(b)) => Ok(Val::String(format!("{}{}", a, b))),
                    _ => Err("HATA: Toplama işlemi geçersiz tipler arasında yapılamaz".to_string()),
                },
                BinaryOp::Sub => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Number(a - b)),
                    _ => Err("HATA: Çıkarma işlemi sadece sayılar arasında yapılabilir".to_string()),
                },
                BinaryOp::Mul => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Number(a * b)),
                    _ => Err("HATA: Çarpma işlemi sadece sayılar arasında yapılabilir".to_string()),
                },
                BinaryOp::Div => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => {
                        if b == 0.0 {
                            Err("HATA: Sıfıra bölme hatası".to_string())
                        } else {
                            Ok(Val::Number(a / b))
                        }
                    }
                    _ => Err("HATA: Bölme işlemi sadece sayılar arasında yapılabilir".to_string()),
                },
                BinaryOp::Mod => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Number(a % b)),
                    _ => Err("HATA: Modül işlemi sadece sayılar arasında yapılabilir".to_string()),
                },
                BinaryOp::Eq => Ok(Val::Boolean(left == right)),
                BinaryOp::Ne => Ok(Val::Boolean(left != right)),
                BinaryOp::Lt => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Boolean(a < b)),
                    _ => Err("HATA: Karşılaştırma sadece sayılar arasında yapılabilir".to_string()),
                },
                BinaryOp::Gt => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Boolean(a > b)),
                    _ => Err("HATA: Karşılaştırma sadece sayılar arasında yapılabilir".to_string()),
                },
                BinaryOp::Le => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Boolean(a <= b)),
                    _ => Err("HATA: Karşılaştırma sadece sayılar arasında yapılabilir".to_string()),
                },
                BinaryOp::Ge => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Boolean(a >= b)),
                    _ => Err("HATA: Karşılaştırma sadece sayılar arasında yapılabilir".to_string()),
                },
                BinaryOp::And => match (left, right) {
                    (Val::Boolean(a), Val::Boolean(b)) => Ok(Val::Boolean(a && b)),
                    _ => Err("HATA: Mantıksal VE sadece boolean tipler arasında yapılabilir".to_string()),
                },
                BinaryOp::Or => match (left, right) {
                    (Val::Boolean(a), Val::Boolean(b)) => Ok(Val::Boolean(a || b)),
                    _ => Err("HATA: Mantıksal VEYA sadece boolean tipler arasında yapılabilir".to_string()),
                },
            }
        }
        Expr::Call(name, args) => {
            let func = env
                .get(name)
                .ok_or_else(|| format!("HATA: Tanımlanamayan işlev '{}'", name))?;

            let mut evaluated_args = Vec::new();
            for arg in args {
                evaluated_args.push(eval_expr(arg, env)?);
            }

            match func {
                Val::Function { params, body } => {
                    if params.len() != evaluated_args.len() {
                        return Err(format!(
                            "HATA: '{}' işlevi {} parametre bekliyor, fakat {} parametre verildi",
                            name,
                            params.len(),
                            evaluated_args.len()
                        ));
                    }
                    let child_env = Env::extend(env);
                    for (param, val) in params.into_iter().zip(evaluated_args) {
                        child_env.set(param, val);
                    }
                    let res = eval_program(&body, &child_env)?;
                    Ok(res.unwrap_or(Val::Bos))
                }
                Val::Builtin(f) => Ok(f(evaluated_args)),
                _ => Err(format!("HATA: '{}' bir işlev değil", name)),
            }
        }
    }
}

pub fn eval_stmt(stmt: &Statement, env: &Env) -> Result<Option<Val>, String> {
    match stmt {
        Statement::VarDecl(name, value) => {
            let val = eval_expr(value, env)?;
            env.set(name.clone(), val);
            Ok(None)
        }
        Statement::Assignment(name, value) => {
            let val = eval_expr(value, env)?;
            env.set(name.clone(), val);
            Ok(None)
        }
        Statement::If(cond, then_block, else_block) => {
            let condition = eval_expr(cond, env)?;
            match condition {
                Val::Boolean(b) => {
                    if b {
                        eval_program(then_block, env)
                    } else if let Some(else_block) = else_block {
                        eval_program(else_block, env)
                    } else {
                        Ok(None)
                    }
                }
                _ => Err("HATA: Koşul ifadesi doğru/yanlış (boolean) değer üretmelidir".to_string()),
            }
        }
        Statement::While(cond, body) => {
            loop {
                let condition = eval_expr(cond, env)?;
                match condition {
                    Val::Boolean(b) => {
                        if !b {
                            break;
                        }
                        if let Some(ret) = eval_program(body, env)? {
                            return Ok(Some(ret));
                        }
                    }
                    _ => return Err("HATA: Koşul ifadesi doğru/yanlış (boolean) değer üretmelidir".to_string()),
                }
            }
            Ok(None)
        }
        Statement::For {
            var,
            start,
            end,
            step_dir,
            body,
        } => {
            let s_val = eval_expr(start, env)?;
            let e_val = eval_expr(end, env)?;

            match (s_val, e_val) {
                (Val::Number(start_n), Val::Number(end_n)) => {
                    let mut i = start_n;
                    let target = end_n;
                    loop {
                        match step_dir {
                            StepDir::Artarak => {
                                if i > target {
                                    break;
                                }
                            }
                            StepDir::Azalarak => {
                                if i < target {
                                    break;
                                }
                            }
                        }

                        let child_env = Env::extend(env);
                        child_env.set(var.clone(), Val::Number(i));
                        if let Some(ret) = eval_program(body, &child_env)? {
                            return Ok(Some(ret));
                        }

                        match step_dir {
                            StepDir::Artarak => i += 1.0,
                            StepDir::Azalarak => i -= 1.0,
                        }
                    }
                }
                _ => return Err("HATA: Sayaç sınırları sayı olmalıdır".to_string()),
            }
            Ok(None)
        }
        Statement::FnDecl { name, params, body } => {
            env.set(
                name.clone(),
                Val::Function {
                    params: params.clone(),
                    body: body.clone(),
                },
            );
            Ok(None)
        }
        Statement::Return(opt_expr) => {
            if let Some(expr) = opt_expr {
                let val = eval_expr(expr, env)?;
                Ok(Some(val))
            } else {
                Ok(Some(Val::Bos))
            }
        }
        Statement::Expr(expr) => {
            eval_expr(expr, env)?;
            Ok(None)
        }
    }
}

pub fn eval_program(stmts: &[Statement], env: &Env) -> Result<Option<Val>, String> {
    for stmt in stmts {
        if let Some(ret) = eval_stmt(stmt, env)? {
            return Ok(Some(ret));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oz_lexer::Token;
    use logos::Logos;

    fn run_src(src: &str) -> Result<(Option<Val>, Env), String> {
        let lexer = Token::lexer(src);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            match token_res {
                Ok(token) => tokens.push((token, span)),
                Err(_) => return Err(format!("Lexer hatası: {:?}", span)),
            }
        }
        let ast = oz_parser::parse_tokens(tokens, src.len()).map_err(|e| format!("{:?}", e))?;
        let env = create_global_env();
        let res = eval_program(&ast, &env)?;
        Ok((res, env))
    }

    #[test]
    fn test_kosul() {
        let src = include_str!("../../examples/ornek1_kosul.oz");
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("sayı"), Some(Val::Number(8.0)));
    }

    #[test]
    fn test_dongu() {
        let src = include_str!("../../examples/ornek2_dongu.oz");
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("sayaç"), Some(Val::Number(4.0)));
    }

    #[test]
    fn test_islev() {
        let src = include_str!("../../examples/ornek3_islev.oz");
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("sonuç"), Some(Val::Number(30.0)));
    }

    #[test]
    fn test_hesap() {
        let src = include_str!("../../examples/ornek4_hesap.oz");
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("toplam"), Some(Val::Number(19.0)));
        assert_eq!(env.get("kalan"), Some(Val::Number(3.0)));
    }

    #[test]
    fn test_karma() {
        let src = include_str!("../../examples/ornek5_karma.oz");
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("limit"), Some(Val::Number(5.0)));
    }
}

