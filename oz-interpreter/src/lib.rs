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
    Array(Rc<RefCell<Vec<Val>>>),
    Hata(String),
    Task(Rc<RefCell<TaskState>>),
}

#[derive(Clone)]
pub struct TaskState {
    pub completed: bool,
    pub func: Val,
    pub args: Vec<Val>,
    pub result: Val,
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
            Val::Array(arr) => {
                let items = arr.borrow();
                write!(f, "[")?;
                for (i, val) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", val)?;
                }
                write!(f, "]")
            }
            Val::Hata(msg) => write!(f, "Hata({:?})", msg),
            Val::Task(_) => write!(f, "Task"),
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
            (Val::Array(a), Val::Array(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (Val::Hata(a), Val::Hata(b)) => a == b,
            (Val::Task(a), Val::Task(b)) => Rc::ptr_eq(a, b),
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
    // Built-in function "boyut" (returns length of array)
    env.set(
        "boyut".to_string(),
        Val::Builtin(Rc::new(|args| {
            if args.len() == 1 {
                if let Val::Array(arr) = &args[0] {
                    return Val::Number(arr.borrow().len() as f64);
                }
            }
            Val::Number(0.0)
        })),
    );
    // Built-in function "ekle" (appends element to array)
    env.set(
        "ekle".to_string(),
        Val::Builtin(Rc::new(|args| {
            if args.len() == 2 {
                if let Val::Array(arr) = &args[0] {
                    arr.borrow_mut().push(args[1].clone());
                }
            }
            Val::Bos
        })),
    );
    // Built-in function "hata_fırlat" (raises an error value)
    env.set(
        "hata_fırlat".to_string(),
        Val::Builtin(Rc::new(|args| {
            let msg = if args.len() >= 1 {
                match &args[0] {
                    Val::String(s) => s.clone(),
                    _ => format!("{:?}", args[0]),
                }
            } else {
                "Bilinmeyen hata".to_string()
            };
            Val::Hata(msg)
        })),
    );
    // Built-in function "dosya_oku"
    env.set(
        "dosya_oku".to_string(),
        Val::Builtin(Rc::new(|args| {
            if args.len() == 1 {
                if let Val::String(path) = &args[0] {
                    match std::fs::read_to_string(path) {
                        Ok(content) => return Val::String(content),
                        Err(e) => return Val::Hata(format!("Dosya okunamadı: {}", e)),
                    }
                }
            }
            Val::Hata("Geçersiz argüman: dosya_oku(yol)".to_string())
        })),
    );
    // Built-in function "dosya_yaz"
    env.set(
        "dosya_yaz".to_string(),
        Val::Builtin(Rc::new(|args| {
            if args.len() == 2 {
                if let (Val::String(path), Val::String(content)) = (&args[0], &args[1]) {
                    match std::fs::write(path, content) {
                        Ok(_) => return Val::Boolean(true),
                        Err(e) => return Val::Hata(format!("Dosya yazılamadı: {}", e)),
                    }
                }
            }
            Val::Hata("Geçersiz argüman: dosya_yaz(yol, içerik)".to_string())
        })),
    );
    // Built-in function "dosya_sil"
    env.set(
        "dosya_sil".to_string(),
        Val::Builtin(Rc::new(|args| {
            if args.len() == 1 {
                if let Val::String(path) = &args[0] {
                    match std::fs::remove_file(path) {
                        Ok(_) => return Val::Boolean(true),
                        Err(e) => return Val::Hata(format!("Dosya silinemedi: {}", e)),
                    }
                }
            }
            Val::Hata("Geçersiz argüman: dosya_sil(yol)".to_string())
        })),
    );
    // Built-in function "arkaplanda_çalıştır" / "calistir"
    let calistir_builtin = Val::Builtin(Rc::new(|args| {
        if args.len() >= 1 {
            let func = args[0].clone();
            let remaining_args = args[1..].to_vec();
            return Val::Task(Rc::new(RefCell::new(TaskState {
                completed: false,
                func,
                args: remaining_args,
                result: Val::Bos,
            })));
        }
        Val::Hata("Geçersiz argüman: arkaplanda_çalıştır(işlev, ...)".to_string())
    }));
    env.set("arkaplanda_çalıştır".to_string(), calistir_builtin.clone());
    env.set("arkaplanda_calistir".to_string(), calistir_builtin);

    // Built-in function "kök" / "karekok"
    let kok_builtin = Val::Builtin(Rc::new(|args| {
        if args.len() == 1 {
            if let &Val::Number(n) = &args[0] {
                if n >= 0.0 {
                    return Val::Number(n.sqrt());
                } else {
                    return Val::Hata("Negatif sayının karekökü alınamaz".to_string());
                }
            }
        }
        Val::Hata("Geçersiz argüman: kök(sayı)".to_string())
    }));
    env.set("kök".to_string(), kok_builtin.clone());
    env.set("karekok".to_string(), kok_builtin);

    // Built-in function "üs" / "ust"
    let us_builtin = Val::Builtin(Rc::new(|args| {
        if args.len() == 2 {
            if let (&Val::Number(base), &Val::Number(exponent)) = (&args[0], &args[1]) {
                return Val::Number(base.powf(exponent));
            }
        }
        Val::Hata("Geçersiz argüman: üs(taban, üs)".to_string())
    }));
    env.set("üs".to_string(), us_builtin.clone());
    env.set("ust".to_string(), us_builtin);

    // Built-in function "mutlak"
    env.set(
        "mutlak".to_string(),
        Val::Builtin(Rc::new(|args| {
            if args.len() == 1 {
                if let &Val::Number(n) = &args[0] {
                    return Val::Number(n.abs());
                }
            }
            Val::Hata("Geçersiz argüman: mutlak(sayı)".to_string())
        })),
    );

    // Built-in function "şimdi" / "simdi"
    let simdi_builtin = Val::Builtin(Rc::new(|_args| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Val::Number(now.as_secs_f64())
    }));
    env.set("şimdi".to_string(), simdi_builtin.clone());
    env.set("simdi".to_string(), simdi_builtin);

    // Built-in function "uyku"
    env.set(
        "uyku".to_string(),
        Val::Builtin(Rc::new(|args| {
            if args.len() == 1 {
                if let &Val::Number(ms) = &args[0] {
                    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
                    return Val::Bos;
                }
            }
            Val::Hata("Geçersiz argüman: uyku(milisaniye)".to_string())
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
        Expr::Array(elements) => {
            let mut vals = Vec::new();
            for el in elements {
                vals.push(eval_expr(el, env)?);
            }
            Ok(Val::Array(Rc::new(RefCell::new(vals))))
        }
        Expr::Index(array_expr, index_expr) => {
            let arr_val = eval_expr(array_expr, env)?;
            let idx_val = eval_expr(index_expr, env)?;
            match arr_val {
                Val::Array(arr) => {
                    match idx_val {
                        Val::Number(n) => {
                            let idx = n as usize;
                            let items = arr.borrow();
                            if idx < items.len() {
                                Ok(items[idx].clone())
                            } else {
                                Err(format!("HATA: Dizi sınırları dışında erişim: indeks {}, boyut {}", idx, items.len()))
                            }
                        }
                        _ => Err("HATA: Dizi indeksi sayı olmak zorundadır".to_string()),
                    }
                }
                _ => Err("HATA: Sadece diziler indekslenebilir".to_string()),
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
                            Val::Builtin(f) => {
                                f(task.args.clone())
                            }
                            _ => return Err("HATA: Görev çağrılabilir bir işlev içermiyor".to_string()),
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

    #[test]
    fn test_diziler() {
        let src = r#"
            dizi = [10, 20, 30];
            ekle(dizi, 40);
            birinci = dizi[0];
            ikinci = dizi'nin 1'inci elemanı;
            eleman_sayisi = boyut(dizi);
        "#;
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("birinci"), Some(Val::Number(10.0)));
        assert_eq!(env.get("ikinci"), Some(Val::Number(20.0)));
        assert_eq!(env.get("eleman_sayisi"), Some(Val::Number(4.0)));
    }

    #[test]
    fn test_hata_ise() {
        let src = r#"
            işlev test_hata(hata_var) {
                hata_var ise {
                    res = hata_fırlat("baglanti koptu") hata_ise {
                        döndür 500;
                    };
                    döndür res;
                } değilse {
                    res = 100 hata_ise {
                        döndür 0;
                    };
                    döndür res;
                }
            }
            sonuc_basarili = test_hata(yanlış);
            sonuc_hatali = test_hata(doğru);
        "#;
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("sonuc_basarili"), Some(Val::Number(100.0)));
        assert_eq!(env.get("sonuc_hatali"), Some(Val::Number(500.0)));
    }

    #[test]
    fn test_dosya_io() {
        let src = r#"
            işlev test_dosya() {
                yazildi = dosya_yaz("test_cikti.txt", "Tilk Dosya Sistemi");
                hata_icerik = "ok";
                icerik = dosya_oku("test_cikti.txt") hata_ise {
                    hata_icerik = "hata";
                };
                silindi = dosya_sil("test_cikti.txt");
                hata_var = "ok";
                hata_mesaji_var = "";
                temp = dosya_oku("olmayan_dosya.txt") hata_ise {
                    hata_var = "yakalandi";
                    hata_mesaji_var = hata_mesajı;
                };
                döndür [yazildi, icerik, silindi, hata_var, hata_icerik, hata_mesaji_var];
            }
            sonuclar = test_dosya();
            yazildi_res = sonuclar[0];
            icerik_res = sonuclar[1];
            silindi_res = sonuclar[2];
            hata_res = sonuclar[3];
            hata_icerik_res = sonuclar[4];
            hata_mesaji_res = sonuclar[5];
        "#;
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("yazildi_res"), Some(Val::Boolean(true)));
        assert_eq!(env.get("icerik_res"), Some(Val::String("Tilk Dosya Sistemi".to_string())));
        assert_eq!(env.get("silindi_res"), Some(Val::Boolean(true)));
        assert_eq!(env.get("hata_res"), Some(Val::String("yakalandi".to_string())));
        assert_eq!(env.get("hata_icerik_res"), Some(Val::String("ok".to_string())));
        
        let msg = env.get("hata_mesaji_res").unwrap();
        if let Val::String(s) = msg {
            assert!(s.contains("okunamadı") || s.contains("okunamadi") || s.contains("bulunamadı") || s.contains("bulunamadi"));
        } else {
            panic!("Hata mesajı string olmalı!");
        }
    }

    #[test]
    fn test_asenkron_tamamlaninca() {
        let src = r#"
            işlev hesapla(x, y) {
                döndür x + y;
            }
            
            gorev = arkaplanda_çalıştır(hesapla, 10, 20);
            
            yakalanan_sonuc = 0;
            gorev tamamlanınca {
                yakalanan_sonuc = sonuç;
            }
        "#;
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("yakalanan_sonuc"), Some(Val::Number(30.0)));
    }

    #[test]
    fn test_math_time() {
        let src = r#"
            işlev test_matematik() {
                karekok_deger = kök(16);
                us_deger = üs(2, 3);
                mutlak_deger = mutlak(0 - 42);
                simdi_zaman = şimdi();
                uyku(10);
                hata_deger = 0;
                temp_hata = kök(0 - 1) hata_ise {
                    hata_deger = 999;
                };
                döndür [karekok_deger, us_deger, mutlak_deger, simdi_zaman, hata_deger];
            }
            sonuclar = test_matematik();
            karekok_res = sonuclar[0];
            us_res = sonuclar[1];
            mutlak_res = sonuclar[2];
            simdi_res = sonuclar[3];
            hata_res = sonuclar[4];
        "#;
        let res = run_src(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, env) = res.unwrap();
        assert_eq!(env.get("karekok_res"), Some(Val::Number(4.0)));
        assert_eq!(env.get("us_res"), Some(Val::Number(8.0)));
        assert_eq!(env.get("mutlak_res"), Some(Val::Number(42.0)));
        assert!(env.get("simdi_res").is_some());
        assert_eq!(env.get("hata_res"), Some(Val::Number(999.0)));
    }
}

