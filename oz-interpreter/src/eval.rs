use super::val::{Env, Val};
use oz_parser::ast::{BinaryOp, Expr, Literal, Spanned, Statement, StepDir, UnaryOp};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn eval_expr(expr: &Spanned<Expr>, env: &Env) -> Result<Val, String> {
    match &expr.node {
        Expr::Literal(lit) => match lit {
            Literal::Number(n) => Ok(Val::Number(*n)),
            Literal::String(s) => Ok(Val::String(s.clone())),
            Literal::Boolean(b) => Ok(Val::Boolean(*b)),
            Literal::Bos => Ok(Val::Bos),
        },
        Expr::Identifier(name) => env
            .get(name)
            .ok_or_else(|| format!("HATA: Tanımlanamayan değişken '{}'", name)),
        Expr::Unary(op, operand) => {
            let val = eval_expr(operand, env)?;
            match op {
                UnaryOp::Neg => match val {
                    Val::Number(n) => Ok(Val::Number(-n)),
                    _ => Err("HATA: Negatif işlem sadece sayılarla yapılabilir".to_string()),
                },
                UnaryOp::Not => match val {
                    Val::Boolean(b) => Ok(Val::Boolean(!b)),
                    _ => Err(
                        "HATA: Mantıksal değil işlemi sadece mantıksal değerlerle yapılabilir"
                            .to_string(),
                    ),
                },
            }
        }
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
                    _ => {
                        Err("HATA: Çıkarma işlemi sadece sayılar arasında yapılabilir".to_string())
                    }
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
                    _ => Err(
                        "HATA: Mantıksal VE sadece boolean tipler arasında yapılabilir".to_string(),
                    ),
                },
                BinaryOp::Or => match (left, right) {
                    (Val::Boolean(a), Val::Boolean(b)) => Ok(Val::Boolean(a || b)),
                    _ => Err(
                        "HATA: Mantıksal VEYA sadece boolean tipler arasında yapılabilir"
                            .to_string(),
                    ),
                },
            }
        }
        Expr::Call(name, args) => {
            if name == "dahil_et" {
                if args.len() != 1 {
                    return Err(
                        "HATA: dahil_et tek bir dosya yolu parametresi almalıdır".to_string()
                    );
                }
                let path_val = eval_expr(&args[0], env)?;
                if let Val::String(path) = path_val {
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| format!("Modül yüklenemedi ({}): {}", path, e))?;

                    use logos::Logos;
                    use oz_lexer::Token;
                    let lexer = Token::lexer(&content);
                    let mut tokens = Vec::new();
                    for (token_res, span) in lexer.spanned() {
                        match token_res {
                            Ok(token) => tokens.push((token, span)),
                            Err(_) => return Err(format!("Sözcüksel analiz hatası at {:?}", span)),
                        }
                    }

                    let ast = oz_parser::parse_tokens(tokens, content.len())
                        .map_err(|e| format!("Ayrıştırma hatası: {:?}", e))?;

                    let _ = eval_program(&ast, env)?;
                    return Ok(Val::Bos);
                } else {
                    return Err("HATA: dahil_et parametresi metin (string) olmalıdır".to_string());
                }
            }

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
        Expr::Array(elements) => {
            let mut vals = Vec::new();
            for el in elements {
                vals.push(eval_expr(el, env)?);
            }
            Ok(Val::Array(Rc::new(RefCell::new(vals))))
        }
        Expr::Map(pairs) => {
            let mut map = HashMap::new();
            for (key_expr, val_expr) in pairs {
                let key_val = eval_expr(key_expr, env)?;
                let val_val = eval_expr(val_expr, env)?;
                match key_val {
                    Val::String(s) => {
                        map.insert(s, val_val);
                    }
                    _ => {
                        return Err(
                            "HATA: Harita anahtarı metin (string) olmak zorundadır".to_string()
                        )
                    }
                }
            }
            Ok(Val::Map(Rc::new(RefCell::new(map))))
        }
        Expr::Index(array_expr, index_expr) => {
            let arr_val = eval_expr(array_expr, env)?;
            let idx_val = eval_expr(index_expr, env)?;
            match arr_val {
                Val::Array(arr) => match idx_val {
                    Val::Number(n) => {
                        let idx = n as usize;
                        let items = arr.borrow();
                        if idx < items.len() {
                            Ok(items[idx].clone())
                        } else {
                            Err(format!(
                                "HATA: Dizi sınırları dışında erişim: indeks {}, boyut {}",
                                idx,
                                items.len()
                            ))
                        }
                    }
                    _ => Err("HATA: Dizi indeksi sayı olmak zorundadır".to_string()),
                },
                Val::Map(map) => match idx_val {
                    Val::String(s) => {
                        let items = map.borrow();
                        if let Some(val) = items.get(&s) {
                            Ok(val.clone())
                        } else {
                            Ok(Val::Bos)
                        }
                    }
                    _ => Err("HATA: Harita indeksi metin (string) olmak zorundadır".to_string()),
                },
                _ => Err("HATA: Sadece diziler ve haritalar indekslenebilir".to_string()),
            }
        }
        Expr::HataIse(base, body) => {
            let res = eval_expr(base, env)?;
            if let Val::Hata(msg) = res {
                let child_env = Env::extend(env);
                child_env.set("hata_mesajı".to_string(), Val::String(msg));
                let body_res = eval_program(body, &child_env)?;
                Ok(body_res.unwrap_or(Val::Bos))
            } else {
                Ok(res)
            }
        }
    }
}

pub fn eval_stmt(stmt: &Spanned<Statement>, env: &Env) -> Result<Option<Val>, String> {
    match &stmt.node {
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
        Statement::IndexAssignment(array_expr, index_expr, value_expr) => {
            let target_val = eval_expr(array_expr, env)?;
            let index_val = eval_expr(index_expr, env)?;
            let value_val = eval_expr(value_expr, env)?;

            match target_val {
                Val::Array(arr) => match index_val {
                    Val::Number(n) => {
                        let idx = n as usize;
                        let mut items = arr.borrow_mut();
                        if idx < items.len() {
                            items[idx] = value_val;
                            Ok(None)
                        } else {
                            Err(format!(
                                "HATA: Dizi sınırları dışında güncelleme: indeks {}, boyut {}",
                                idx,
                                items.len()
                            ))
                        }
                    }
                    _ => Err("HATA: Dizi indeksi sayı olmak zorundadır".to_string()),
                },
                Val::Map(map) => match index_val {
                    Val::String(s) => {
                        map.borrow_mut().insert(s, value_val);
                        Ok(None)
                    }
                    _ => Err("HATA: Harita indeksi metin (string) olmak zorundadır".to_string()),
                },
                _ => Err("HATA: Sadece diziler ve haritalar güncellenebilir".to_string()),
            }
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
                _ => {
                    Err("HATA: Koşul ifadesi doğru/yanlış (boolean) değer üretmelidir".to_string())
                }
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
                    _ => {
                        return Err(
                            "HATA: Koşul ifadesi doğru/yanlış (boolean) değer üretmelidir"
                                .to_string(),
                        )
                    }
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
        Statement::Tamamlaninca(gorev_expr, body) => {
            let val = eval_expr(gorev_expr, env)?;
            let result_val = match val {
                Val::Task(task_cell) => {
                    let mut task = task_cell.borrow_mut();
                    if !task.completed {
                        let res = match &task.func {
                            Val::Function { params, body } => {
                                let child_env = Env::extend(env);
                                for (param, val) in params.iter().zip(&task.args) {
                                    child_env.set(param.clone(), val.clone());
                                }
                                let ret = eval_program(body, &child_env)?;
                                ret.unwrap_or(Val::Bos)
                            }
                            Val::Builtin(f) => f(task.args.clone()),
                            _ => {
                                return Err(
                                    "HATA: Görev çağrılabilir bir işlev içermiyor".to_string()
                                )
                            }
                        };
                        task.result = res;
                        task.completed = true;
                    }
                    task.result.clone()
                }
                other => other,
            };

            let child_env = Env::extend(env);
            child_env.set("sonuç".to_string(), result_val.clone());
            child_env.set("sonuc".to_string(), result_val);
            eval_program(body, &child_env)
        }
    }
}

pub fn eval_program(stmts: &[Spanned<Statement>], env: &Env) -> Result<Option<Val>, String> {
    for stmt in stmts {
        if let Some(ret) = eval_stmt(stmt, env)? {
            return Ok(Some(ret));
        }
    }
    Ok(None)
}
