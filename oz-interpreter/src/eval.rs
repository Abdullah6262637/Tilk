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
        Expr::Identifier(prefix, name) => {
            let lookup_name = if let Some(p) = prefix {
                format!("{}::{}", p.join("::"), name)
            } else {
                name.clone()
            };
            env.get(&lookup_name)
                .ok_or_else(|| format!("HATA: Tanımlanamayan değişken '{}'", lookup_name))
        }
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
                    _ => Err(
                        "HATA: Toplama işlemi sayılar veya metinler arasında yapılabilir"
                            .to_string(),
                    ),
                },
                BinaryOp::Sub => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Number(a - b)),
                    _ => Err("HATA: Çıkarma işlemi sadece sayılarla yapılabilir".to_string()),
                },
                BinaryOp::Mul => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Number(a * b)),
                    _ => Err("HATA: Çarpma işlemi sadece sayılarla yapılabilir".to_string()),
                },
                BinaryOp::Div => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => {
                        if b == 0.0 {
                            Err("HATA: Sıfıra bölme hatası".to_string())
                        } else {
                            Ok(Val::Number(a / b))
                        }
                    }
                    _ => Err("HATA: Bölme işlemi sadece sayılarla yapılabilir".to_string()),
                },
                BinaryOp::Mod => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Number(a % b)),
                    _ => Err("HATA: Kalan işlemi sadece sayılarla yapılabilir".to_string()),
                },
                BinaryOp::Eq => Ok(Val::Boolean(left == right)),
                BinaryOp::Ne => Ok(Val::Boolean(left != right)),
                BinaryOp::Lt => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Boolean(a < b)),
                    _ => Err("HATA: Karşılaştırma işlemi sadece sayılarla yapılabilir".to_string()),
                },
                BinaryOp::Gt => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Boolean(a > b)),
                    _ => Err("HATA: Karşılaştırma işlemi sadece sayılarla yapılabilir".to_string()),
                },
                BinaryOp::Le => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Boolean(a <= b)),
                    _ => Err("HATA: Karşılaştırma işlemi sadece sayılarla yapılabilir".to_string()),
                },
                BinaryOp::Ge => match (left, right) {
                    (Val::Number(a), Val::Number(b)) => Ok(Val::Boolean(a >= b)),
                    _ => Err("HATA: Karşılaştırma işlemi sadece sayılarla yapılabilir".to_string()),
                },
                BinaryOp::And => match (left, right) {
                    (Val::Boolean(a), Val::Boolean(b)) => Ok(Val::Boolean(a && b)),
                    _ => Err(
                        "HATA: Mantıksal VE işlemi sadece mantıksal değerlerle yapılabilir"
                            .to_string(),
                    ),
                },
                BinaryOp::Or => match (left, right) {
                    (Val::Boolean(a), Val::Boolean(b)) => Ok(Val::Boolean(a || b)),
                    _ => Err(
                        "HATA: Mantıksal VEYA işlemi sadece mantıksal değerlerle yapılabilir"
                            .to_string(),
                    ),
                },
            }
        }
        Expr::Call(prefix, name, args) => {
            if prefix.is_none() && name == "dahil_et" {
                if args.len() != 1 {
                    return Err(
                        "HATA: dahil_et tek bir dosya yolu parametresi almalıdır".to_string()
                    );
                }
                let path_val = eval_expr(&args[0], env)?;
                if let Val::String(path_str) = path_val {
                    let embedded_content = match path_str.as_str() {
                        "std::sonuc" => Some("işlev basarili(deger) { r = {}; r[\"tur\"] = \"basarili\"; r[\"deger\"] = deger; döndür r; } işlev hata(mesaj) { r = {}; r[\"tur\"] = \"hata\"; r[\"hata\"] = mesaj; döndür r; }".to_string()),
                        "std::matematik" => Some("işlev karekok(x) { döndür kök(x); } işlev ust(taban, kuvvet) { döndür üs(taban, kuvvet); } işlev mutlak_deger(x) { döndür mutlak(x); }".to_string()),
                        "std::dosya" => Some("dahil_et(\"std::sonuc\"); işlev oku(yol) { r = dosya_oku(yol); (r) hata_ise { döndür std::sonuc::hata(\"Okuma hatası\"); }; döndür std::sonuc::basarili(r); } işlev yaz(yol, icerik) { r = dosya_yaz(yol, icerik); (r) hata_ise { döndür std::sonuc::hata(\"Yazma hatası\"); }; döndür std::sonuc::basarili(boş); } işlev sil(yol) { r = dosya_sil(yol); (r) hata_ise { döndür std::sonuc::hata(\"Silme hatası\"); }; döndür std::sonuc::basarili(boş); }".to_string()),
                        "std::zaman" => Some("işlev simdi() { döndür şimdi(); } işlev bekle(ms) { döndür uyku(ms); }".to_string()),
                        _ => None,
                    };

                    let (canonical_path, content) = if let Some(content_str) = embedded_content {
                        (std::path::PathBuf::from(path_str.clone()), content_str)
                    } else {
                        let path = std::path::Path::new(&path_str);
                        let canonical_path = std::fs::canonicalize(path).map_err(|e| {
                            format!("Modül yolu çözümlenemedi ({}): {}", path_str, e)
                        })?;
                        let read_content = std::fs::read_to_string(&canonical_path)
                            .map_err(|e| format!("Modül yüklenemedi ({}): {}", path_str, e))?;
                        (canonical_path, read_content)
                    };

                    if env.is_loading(&canonical_path) {
                        return Err(format!(
                            "HATA: Döngüsel bağımlılık tespit edildi: {}",
                            path_str
                        ));
                    }
                    if env.is_loaded(&canonical_path) {
                        return Ok(Val::Bos);
                    }
                    env.push_loading(canonical_path.clone());

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

                    let module_env = crate::builtins::create_global_env();
                    // Copy existing bindings from the parent environment into the module env so it can resolve namespaces
                    for (k, v) in env.get_bindings().into_iter() {
                        module_env.set_local(k, v);
                    }
                    let _ = eval_program(&ast, &module_env)?;

                    let has_namespace_prefix = if path_str.starts_with("std::") {
                        Some(
                            path_str
                                .split("::")
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>(),
                        )
                    } else {
                        None
                    };

                    if let Some(ref p) = has_namespace_prefix {
                        let prefix_str = p.join("::");
                        for (k, v) in module_env.get_bindings().into_iter() {
                            let is_builtin = matches!(
                                k.as_str(),
                                "yazdır"
                                    | "boyut"
                                    | "ekle"
                                    | "hata_fırlat"
                                    | "hata_firlat"
                                    | "dosya_oku"
                                    | "dosya_yaz"
                                    | "dosya_sil"
                                    | "arkaplanda_çalıştır"
                                    | "arkaplanda_calistir"
                                    | "kök"
                                    | "karekok"
                                    | "üs"
                                    | "ust"
                                    | "mutlak"
                                    | "şimdi"
                                    | "simdi"
                                    | "uyku"
                                    | "dahil_et"
                                    | "kanal"
                            );
                            if is_builtin {
                                continue;
                            }
                            if k.contains("::") {
                                env.set_local(k, v);
                            } else {
                                env.set_local(format!("{}::{}", prefix_str, k), v);
                            }
                        }
                    } else {
                        for (k, v) in module_env.get_bindings().into_iter() {
                            let is_builtin = matches!(
                                k.as_str(),
                                "yazdır"
                                    | "boyut"
                                    | "ekle"
                                    | "hata_fırlat"
                                    | "hata_firlat"
                                    | "dosya_oku"
                                    | "dosya_yaz"
                                    | "dosya_sil"
                                    | "arkaplanda_çalıştır"
                                    | "arkaplanda_calistir"
                                    | "kök"
                                    | "karekok"
                                    | "üs"
                                    | "ust"
                                    | "mutlak"
                                    | "şimdi"
                                    | "simdi"
                                    | "uyku"
                                    | "dahil_et"
                                    | "kanal"
                            );
                            if is_builtin {
                                continue;
                            }
                            env.set_local(k, v);
                        }
                    }

                    env.pop_loading();
                    env.mark_loaded(canonical_path);

                    return Ok(Val::Bos);
                } else {
                    return Err("HATA: dahil_et parametresi metin (string) olmalıdır".to_string());
                }
            }

            let lookup_name = if let Some(p) = prefix {
                format!("{}::{}", p.join("::"), name)
            } else {
                name.clone()
            };

            let func = env
                .get(&lookup_name)
                .or_else(|| {
                    let is_builtin = matches!(
                        name.as_str(),
                        "yazdır"
                            | "boyut"
                            | "ekle"
                            | "hata_fırlat"
                            | "hata_firlat"
                            | "dosya_oku"
                            | "dosya_yaz"
                            | "dosya_sil"
                            | "arkaplanda_çalıştır"
                            | "arkaplanda_calistir"
                            | "kök"
                            | "karekok"
                            | "üs"
                            | "ust"
                            | "mutlak"
                            | "şimdi"
                            | "simdi"
                            | "uyku"
                            | "kanal"
                    );
                    if is_builtin {
                        env.get(name)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| format!("HATA: Tanımlanamayan işlev '{}'", lookup_name))?;

            let mut evaluated_args = Vec::new();
            for arg in args {
                match eval_expr(arg, env) {
                    Ok(v) => evaluated_args.push(v),
                    Err(e) => return Ok(Val::Hata(e)),
                }
            }

            let is_calistir = prefix.is_none()
                && (name == "arkaplanda_çalıştır" || name == "arkaplanda_calistir");
            if is_calistir && !evaluated_args.is_empty() {
                let func_to_run = evaluated_args[0].clone();
                let task_args = evaluated_args[1..].to_vec();
                let run_res = match &func_to_run {
                    Val::Function { params, body } => {
                        let child_env = Env::extend(env);
                        for (param, val) in params.iter().zip(&task_args) {
                            child_env.set(param.clone(), val.clone());
                        }
                        let ret = eval_program(body, &child_env)?;
                        ret.unwrap_or(Val::Bos)
                    }
                    Val::Builtin(f) => f(task_args.clone()),
                    _ => Val::Bos,
                };
                return Ok(Val::Task(Rc::new(RefCell::new(crate::val::TaskState {
                    completed: true,
                    func: func_to_run,
                    args: task_args,
                    result: run_res,
                }))));
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
                    let unwrapped = res.unwrap_or(Val::Bos);
                    if let Val::Return(inner) = unwrapped {
                        Ok(*inner)
                    } else {
                        Ok(unwrapped)
                    }
                }
                Val::Builtin(f) => {
                    let call_res = f(evaluated_args);
                    Ok(call_res)
                }
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
                Val::Channel(ch) => {
                    let mut items = ch.borrow_mut();
                    if let Some(val) = items.pop_front() {
                        Ok(val)
                    } else {
                        Ok(Val::Bos)
                    }
                }
                _ => Err("HATA: Sadece diziler, haritalar ve kanallar indekslenebilir".to_string()),
            }
        }
        Expr::HataIse(base, body) => {
            let res = eval_expr(base, env);
            let opt_err_msg = match &res {
                Err(msg) => Some(msg.clone()),
                Ok(Val::Hata(msg)) => Some(msg.clone()),
                Ok(Val::Map(map)) => {
                    let m = map.borrow();
                    if m.get("tur") == Some(&Val::String("hata".to_string())) {
                        match m.get("hata") {
                            Some(Val::String(s)) => Some(s.clone()),
                            _ => Some("Bilinmeyen hata".to_string()),
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(msg) = opt_err_msg {
                let child_env = Env::extend(env);
                child_env.set("hata_mesajı".to_string(), Val::String(msg.clone()));
                child_env.set("hata_mesaji".to_string(), Val::String(msg));
                match eval_program(body, &child_env)? {
                    Some(val) => {
                        // Return signal propagated inside Val::Return
                        Ok(Val::Return(Box::new(val)))
                    }
                    None => Ok(Val::Bos),
                }
            } else {
                res
            }
        }
    }
}

pub fn eval_stmt(stmt: &Spanned<Statement>, env: &Env) -> Result<Option<Val>, String> {
    match &stmt.node {
        Statement::VarDecl(name, value) => {
            let val = eval_expr(value, env)?;
            if let Val::Return(_) = val {
                Ok(Some(val))
            } else {
                env.set(name.clone(), val);
                Ok(None)
            }
        }
        Statement::Assignment(name, value) => {
            let val = eval_expr(value, env)?;
            if let Val::Return(_) = val {
                Ok(Some(val))
            } else {
                env.set(name.clone(), val);
                Ok(None)
            }
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
                Val::Channel(ch) => {
                    ch.borrow_mut().push_back(value_val);
                    Ok(None)
                }
                _ => Err("HATA: Sadece diziler, haritalar ve kanallar güncellenebilir".to_string()),
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
        Statement::ForEach {
            var,
            iterable,
            body,
        } => {
            let iter_val = eval_expr(iterable, env)?;
            match iter_val {
                Val::Array(arr) => {
                    let items = arr.borrow().clone();
                    for item in items {
                        let child_env = Env::extend(env);
                        child_env.set(var.clone(), item);
                        if let Some(ret) = eval_program(body, &child_env)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Val::String(s) => {
                    for c in s.chars() {
                        let child_env = Env::extend(env);
                        child_env.set(var.clone(), Val::String(c.to_string()));
                        if let Some(ret) = eval_program(body, &child_env)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                _ => return Err("HATA: For-Each döngüsü sadece diziler ve metinler üzerinde kullanılabilir".to_string()),
            }
            Ok(None)
        }
        Statement::FnDecl {
            name,
            generics: _,
            params,
            return_type: _,
            body,
        } => {
            let param_names: Vec<String> =
                params.iter().map(|(p_name, _)| p_name.clone()).collect();
            env.set(
                name.clone(),
                Val::Function {
                    params: param_names,
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
            let val = eval_expr(expr, env)?;
            if let Val::Return(_) = val {
                Ok(Some(val))
            } else {
                Ok(None)
            }
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
