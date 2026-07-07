use super::types::{Scheme, Type, TypeEnv};
use crate::ast::{BinaryOp, Expr, Literal, Spanned, Statement, UnaryOp};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

macro_rules! type_err {
    ($($arg:tt)*) => {
        super::types::TypeError::new(format!($($arg)*))
    }
}

pub struct TypeChecker {
    pub next_var: usize,
    pub substitutions: HashMap<usize, Type>,
    pub recorded_types: HashMap<String, Type>,
    pub loaded_files: HashSet<PathBuf>,
    pub loading_stack: Vec<PathBuf>,
}

impl TypeChecker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TypeChecker {
            next_var: 0,
            substitutions: HashMap::new(),
            recorded_types: HashMap::new(),
            loaded_files: HashSet::new(),
            loading_stack: Vec::new(),
        }
    }

    pub fn new_var(&mut self) -> usize {
        let v = self.next_var;
        self.next_var += 1;
        v
    }

    pub fn infer_expr(
        &mut self,
        expr: &Spanned<Expr>,
        env: &mut TypeEnv,
        current_ret_ty: &Option<Type>,
    ) -> Result<Type, super::types::TypeError> {
        match &expr.node {
            Expr::Literal(lit) => match lit {
                Literal::Number(_) => Ok(Type::Number),
                Literal::String(_) => Ok(Type::String),
                Literal::Boolean(_) => Ok(Type::Boolean),
                Literal::Bos => Ok(Type::Bos),
            },
            Expr::Identifier(prefix, name) => {
                let lookup_name = if let Some(p) = prefix {
                    format!("{}::{}", p.join("::"), name)
                } else {
                    name.clone()
                };
                if let Some(scheme) = env.get(&lookup_name) {
                    let instantiated = self.instantiate(&scheme);
                    let resolved = self.resolve(&instantiated);
                    self.recorded_types.insert(lookup_name.clone(), resolved);
                    Ok(instantiated)
                } else {
                    Err(type_err!(
                        "Tip Hatası: Tanımlanamayan değişken '{}'",
                        lookup_name
                    ))
                }
            }
            Expr::InterpolatedString(parts) => {
                for part in parts {
                    if let crate::ast::InterpolatedPart::Expr(e) = part {
                        self.infer_expr(e, env, current_ret_ty)?;
                    }
                }
                Ok(Type::String)
            }
            Expr::Unary(op, operand) => {
                let t = self.infer_expr(operand, env, current_ret_ty)?;
                match op {
                    UnaryOp::Neg => {
                        self.unify(&t, &Type::Number)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Number)
                    }
                    UnaryOp::Not => {
                        self.unify(&t, &Type::Boolean)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Boolean)
                    }
                }
            }

            Expr::Binary(lhs, op, text_rhs) => {
                let t1 = self.infer_expr(lhs, env, current_ret_ty)?;
                let t2 = self.infer_expr(text_rhs, env, current_ret_ty)?;
                match op {
                    BinaryOp::Add => {
                        self.unify(&t1, &t2)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        let resolved = self.resolve(&t1);
                        if resolved == Type::String || resolved == Type::Number {
                            Ok(resolved)
                        } else {
                            self.unify(&t1, &Type::Number)
                                .map_err(|e| e.with_span(expr.span.clone()))?;
                            Ok(Type::Number)
                        }
                    }
                    BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        self.unify(&t1, &Type::Number)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        self.unify(&t2, &Type::Number)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Number)
                    }
                    BinaryOp::Eq | BinaryOp::Ne => {
                        self.unify(&t1, &t2)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Boolean)
                    }
                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        self.unify(&t1, &Type::Number)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        self.unify(&t2, &Type::Number)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Boolean)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        self.unify(&t1, &Type::Boolean)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        self.unify(&t2, &Type::Boolean)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Boolean)
                    }
                }
            }
            Expr::Call(prefix, name, args) => {
                if prefix.is_none() && name == "dahil_et" {
                    if args.is_empty() {
                        return Err(
                            type_err!("Tip Hatası: dahil_et en az bir argüman almalıdır")
                                .with_span(expr.span.clone()),
                        );
                    }
                    if let Expr::Literal(Literal::String(path_str)) = &args[0].node {
                        let embedded_content = match path_str.as_str() {
                            "std::sonuc" => Some(include_str!("../../../std/sonuc.oz").to_string()),
                            "std::matematik" => {
                                Some(include_str!("../../../std/matematik.oz").to_string())
                            }
                            "std::dosya" => Some(include_str!("../../../std/dosya.oz").to_string()),
                            "std::zaman" => Some(include_str!("../../../std/zaman.oz").to_string()),
                            "std::metin" => Some(include_str!("../../../std/metin.oz").to_string()),
                            "std::dizi" => Some(include_str!("../../../std/dizi.oz").to_string()),
                            _ => None,
                        };

                        let (canonical_path, content) = if let Some(content_str) = embedded_content
                        {
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

                        if self.loading_stack.contains(&canonical_path) {
                            return Err(type_err!(
                                "Tip Hatası: Döngüsel bağımlılık tespit edildi: {}",
                                path_str
                            ));
                        }

                        if self.loaded_files.contains(&canonical_path) {
                            return Ok(Type::Bos);
                        }

                        self.loading_stack.push(canonical_path.clone());

                        use logos::Logos;
                        use oz_lexer::Token;
                        let lexer = Token::lexer(&content);
                        let mut tokens = Vec::new();
                        for (token_res, span) in lexer.spanned() {
                            match token_res {
                                Ok(token) => tokens.push((token, span)),
                                Err(_) => {
                                    return Err(type_err!("Sözcüksel analiz hatası at {:?}", span)
                                        .with_span(expr.span.clone()))
                                }
                            }
                        }

                        let ast = crate::parse_tokens(tokens, content.len())
                            .map_err(|e| format!("Ayrıştırma hatası: {:?}", e))?;

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

                        let mut module_env = create_default_type_env(self);
                        for stmt in &ast {
                            self.infer_stmt(stmt, &mut module_env, current_ret_ty)?;
                        }

                        let mut current_env = Some(Box::new(module_env));
                        while let Some(curr) = current_env {
                            for (name, scheme) in curr.bindings.clone() {
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
                                        | "dahil_et"
                                        | "kanal"
                                        | "biçimle"
                                        | "uzunluk"
                                        | "böl"
                                        | "birleştir"
                                        | "içerir"
                                        | "büyük_harf"
                                        | "küçük_harf"
                                        | "kırp"
                                        | "tamsayı"
                                        | "metne_çevir"
                                        | "sayıya_çevir"
                                        | "rastgele"
                                );
                                if is_builtin {
                                    continue;
                                }
                                if name.contains("::") {
                                    env.set(name, scheme);
                                } else {
                                    let bound_name = if let Some(ref p) = has_namespace_prefix {
                                        format!("{}::{}", p.join("::"), name)
                                    } else {
                                        name
                                    };
                                    env.set(bound_name, scheme);
                                }
                            }
                            current_env = curr.parent;
                        }

                        self.loading_stack.pop();
                        self.loaded_files.insert(canonical_path);

                        return Ok(Type::Bos);
                    } else {
                        return Err(type_err!("Tip Hatası: dahil_et parametresi doğrudan metin (literal string) olmalıdır"));
                    }
                }

                // Variadic builtins — skip unify, just infer each arg
                if prefix.is_none() && (name == "yazdır" || name == "yazdir") {
                    for arg in args {
                        self.infer_expr(arg, env, current_ret_ty)?;
                    }
                    return Ok(Type::Bos);
                }

                if prefix.is_none() && (name == "biçimle" || name == "bicimle") {
                    for arg in args {
                        self.infer_expr(arg, env, current_ret_ty)?;
                    }
                    return Ok(Type::String);
                }

                if prefix.is_none()
                    && (name == "arkaplanda_çalıştır" || name == "arkaplanda_calistir")
                {
                    if args.is_empty() {
                        return Err(type_err!(
                            "Tip Hatası: arkaplanda_çalıştır en az bir argüman almalıdır"
                        )
                        .with_span(expr.span.clone()));
                    }
                    let fn_ty = self.infer_expr(&args[0], env, current_ret_ty)?;
                    let mut param_tys = Vec::new();
                    for arg in &args[1..] {
                        param_tys.push(self.infer_expr(arg, env, current_ret_ty)?);
                    }
                    let ret_var = self.new_var();
                    let expected_fn_ty = Type::Function {
                        params: param_tys,
                        ret: Box::new(Type::Var(ret_var)),
                    };
                    self.unify(&fn_ty, &expected_fn_ty)
                        .map_err(|e| e.with_span(expr.span.clone()))?;
                    return Ok(Type::Task(Box::new(self.resolve(&Type::Var(ret_var)))));
                }

                let lookup_name = if let Some(p) = prefix {
                    format!("{}::{}", p.join("::"), name)
                } else {
                    name.clone()
                };

                let fn_scheme = env
                    .get(&lookup_name)
                    .or_else(|| {
                        // fallback to name without prefix if name starts with standard lib prefix but was parsed without it? No, if name is a builtin
                        let is_builtin = matches!(
                            name.as_str(),
                            "yazdır"
                                | "yazdir"
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
                                | "biçimle"
                                | "uzunluk"
                                | "böl"
                                | "birleştir"
                                | "içerir"
                                | "büyük_harf"
                                | "küçük_harf"
                                | "kırp"
                                | "tamsayı"
                                | "metne_çevir"
                                | "sayıya_çevir"
                                | "rastgele"
                        );
                        if is_builtin {
                            env.get(name)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| format!("Tip Hatası: Tanımlanamayan işlev '{}'", lookup_name))?;
                let fn_ty = self.instantiate(&fn_scheme);

                let mut arg_tys = Vec::new();
                for arg in args {
                    arg_tys.push(self.infer_expr(arg, env, current_ret_ty)?);
                }

                let ret_var = self.new_var();
                let expected_fn_ty = Type::Function {
                    params: arg_tys,
                    ret: Box::new(Type::Var(ret_var)),
                };
                self.unify(&fn_ty, &expected_fn_ty)
                    .map_err(|e| e.with_span(expr.span.clone()))?;
                Ok(self.resolve(&Type::Var(ret_var)))
            }
            Expr::Array(elements) => {
                for el in elements {
                    let _el_ty = self.infer_expr(el, env, current_ret_ty)?;
                }
                Ok(Type::Array(Box::new(Type::Var(self.new_var()))))
            }
            Expr::Map(pairs) => {
                for (key_expr, val_expr) in pairs {
                    let key_ty = self.infer_expr(key_expr, env, current_ret_ty)?;
                    self.unify(&key_ty, &Type::String)
                        .map_err(|e| e.with_span(expr.span.clone()))?;
                    let _val_ty = self.infer_expr(val_expr, env, current_ret_ty)?;
                }
                Ok(Type::Map(Box::new(Type::Var(self.new_var()))))
            }
            Expr::Index(array_expr, index_expr) => {
                let arr_ty = self.infer_expr(array_expr, env, current_ret_ty)?;
                let idx_ty = self.infer_expr(index_expr, env, current_ret_ty)?;
                let resolved_arr = self.resolve(&arr_ty);
                match resolved_arr {
                    Type::Array(_) => {
                        self.unify(&idx_ty, &Type::Number)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Var(self.new_var()))
                    }
                    Type::Map(_) => {
                        self.unify(&idx_ty, &Type::String)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Var(self.new_var()))
                    }
                    Type::Channel(inner) => {
                        self.unify(&idx_ty, &Type::Number)
                            .map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(*inner)
                    }
                    Type::Var(_id) => {
                        let res_var = self.new_var();
                        // Instead of force-constraining to Array immediately, let's keep it flexible
                        // or default to Array if no other type is unified. Wait, a safer way:
                        // Let's create a fresh variable representing the collection type itself
                        // but since we don't have Union types, let's check if the index is a number,
                        // and when unifying Type::Var(id) with Array/Channel, it will resolve.
                        // Let's check: if we do not insert substitution here, then occurs check and unify will handle it when arkaplanda_çalıştır is unified!
                        // Yes! We can just return Ok(Type::Var(res_var)) without inserting substitutions.insert(id, array_ty)!
                        // Wait, if we don't insert, then id remains a Var, and later when unified with Channel(T), it works!
                        // Let's check: self.unify(&idx_ty, &Type::Number).map_err(|e| e.with_span(expr.span.clone()))?;
                        Ok(Type::Var(res_var))
                    }
                    _ => Err(type_err!(
                        "Tip Hatası: Sadece diziler, haritalar ve kanallar indekslenebilir"
                    )
                    .with_span(expr.span.clone())),
                }
            }
            Expr::HataIse(base_expr, body) => {
                let base_ty = self.infer_expr(base_expr, env, current_ret_ty)?;
                let mut child_env = TypeEnv::extend(env);
                child_env.set(
                    "hata_mesajı".to_string(),
                    Scheme {
                        vars: vec![],
                        ty: Type::String,
                    },
                );
                child_env.set(
                    "hata_mesaji".to_string(),
                    Scheme {
                        vars: vec![],
                        ty: Type::String,
                    },
                );

                let mut body_ty = Type::Bos;
                let mut current_env = child_env;
                for stmt in body {
                    if let Statement::Expr(e) = &stmt.node {
                        body_ty = self.infer_expr(e, &mut current_env, current_ret_ty)?;
                    } else {
                        self.infer_stmt(stmt, &mut current_env, current_ret_ty)?;
                        body_ty = Type::Bos;
                    }
                }
                if body_ty != Type::Bos {
                    self.unify(&base_ty, &body_ty)
                        .map_err(|e| e.with_span(expr.span.clone()))?;
                }
                Ok(self.resolve(&base_ty))
            }
            Expr::Lambda { params, body } => {
                let mut param_tys = Vec::new();
                let mut child_env = TypeEnv::extend(env);
                for param in params {
                    let p_var = self.new_var();
                    let p_ty = Type::Var(p_var);
                    param_tys.push(p_ty.clone());
                    child_env.set(
                        param.clone(),
                        Scheme {
                            vars: vec![],
                            ty: p_ty,
                        },
                    );
                }

                let ret_ty_var = self.new_var();
                let expected_ret_ty = Type::Var(ret_ty_var);
                let opt_expected = Some(expected_ret_ty.clone());

                for stmt in body {
                    self.infer_stmt(stmt, &mut child_env, &opt_expected)?;
                }

                Ok(Type::Function {
                    params: param_tys,
                    ret: Box::new(expected_ret_ty),
                })
            }
        }
    }

    pub fn infer_stmt(
        &mut self,
        stmt: &Spanned<Statement>,
        env: &mut TypeEnv,
        current_ret_ty: &Option<Type>,
    ) -> Result<(), super::types::TypeError> {
        match &stmt.node {
            Statement::VarDecl(name, value) | Statement::Assignment(name, value) => {
                let val_ty = self.infer_expr(value, env, current_ret_ty)?;
                if let Some(scheme) = env.get(name) {
                    let instantiated = self.instantiate(&scheme);
                    self.unify(&instantiated, &val_ty)
                        .map_err(|e| e.with_span(stmt.span.clone()))?;
                } else {
                    env.set(
                        name.clone(),
                        Scheme {
                            vars: vec![],
                            ty: val_ty.clone(),
                        },
                    );
                }
                let resolved = self.resolve(&val_ty);
                self.recorded_types.insert(name.clone(), resolved);
            }
            Statement::IndexAssignment(array_expr, index_expr, value_expr) => {
                let arr_ty = self.infer_expr(array_expr, env, current_ret_ty)?;
                let idx_ty = self.infer_expr(index_expr, env, current_ret_ty)?;
                let _val_ty = self.infer_expr(value_expr, env, current_ret_ty)?;
                let resolved_arr = self.resolve(&arr_ty);
                match resolved_arr {
                    Type::Array(_) => {
                        self.unify(&idx_ty, &Type::Number)
                            .map_err(|e| e.with_span(stmt.span.clone()))?;
                    }
                    Type::Map(_) => {
                        self.unify(&idx_ty, &Type::String)
                            .map_err(|e| e.with_span(stmt.span.clone()))?;
                    }
                    Type::Channel(inner) => {
                        self.unify(&idx_ty, &Type::Number)
                            .map_err(|e| e.with_span(stmt.span.clone()))?;
                        self.unify(&_val_ty, &inner)
                            .map_err(|e| e.with_span(stmt.span.clone()))?;
                    }
                    Type::Var(_id) => {
                        // Keep it flexible so it can unify with either Array or Channel
                    }
                    _ => {
                        return Err(type_err!(
                            "Tip Hatası: Sadece diziler, haritalar ve kanallar güncellenebilir"
                        )
                        .with_span(stmt.span.clone()));
                    }
                }
            }
            Statement::If(cond, then_block, else_block) => {
                let cond_ty = self.infer_expr(cond, env, current_ret_ty)?;
                self.unify(&cond_ty, &Type::Boolean)
                    .map_err(|e| e.with_span(stmt.span.clone()))?;

                let mut then_env = TypeEnv::extend(env);
                for s in then_block {
                    self.infer_stmt(s, &mut then_env, current_ret_ty)?;
                }
                if let Some(else_stmts) = else_block {
                    let mut else_env = TypeEnv::extend(env);
                    for s in else_stmts {
                        self.infer_stmt(s, &mut else_env, current_ret_ty)?;
                    }
                }
            }
            Statement::While(cond, body) => {
                let cond_ty = self.infer_expr(cond, env, current_ret_ty)?;
                self.unify(&cond_ty, &Type::Boolean)
                    .map_err(|e| e.with_span(stmt.span.clone()))?;

                let mut body_env = TypeEnv::extend(env);
                for s in body {
                    self.infer_stmt(s, &mut body_env, current_ret_ty)?;
                }
            }
            Statement::For {
                var,
                start,
                end,
                step_dir: _,
                body,
            } => {
                let start_ty = self.infer_expr(start, env, current_ret_ty)?;
                self.unify(&start_ty, &Type::Number)
                    .map_err(|e| e.with_span(stmt.span.clone()))?;
                let end_ty = self.infer_expr(end, env, current_ret_ty)?;
                self.unify(&end_ty, &Type::Number)
                    .map_err(|e| e.with_span(stmt.span.clone()))?;

                let mut body_env = TypeEnv::extend(env);
                body_env.set(
                    var.clone(),
                    Scheme {
                        vars: vec![],
                        ty: Type::Number,
                    },
                );
                for s in body {
                    self.infer_stmt(s, &mut body_env, current_ret_ty)?;
                }
            }
            Statement::ForEach {
                var,
                iterable,
                body,
            } => {
                let iterable_ty = self.infer_expr(iterable, env, current_ret_ty)?;
                let resolved_iterable = self.resolve(&iterable_ty);

                let item_ty = match resolved_iterable {
                    Type::Array(inner) => *inner.clone(),
                    Type::String => Type::String,
                    Type::Var(_) => {
                        let element_ty = self.new_var();
                        self.unify(&iterable_ty, &Type::Array(Box::new(Type::Var(element_ty))))?;
                        Type::Var(element_ty)
                    }
                    _ => {
                        return Err(type_err!("Tip Hatası: For-Each döngüsü sadece diziler ve metinler üzerinde kullanılabilir").with_span(stmt.span.clone()))
                    }
                };

                let mut body_env = TypeEnv::extend(env);
                body_env.set(
                    var.clone(),
                    Scheme {
                        vars: vec![],
                        ty: item_ty,
                    },
                );
                for s in body {
                    self.infer_stmt(s, &mut body_env, current_ret_ty)?;
                }
            }
            Statement::FnDecl {
                name,
                generics,
                params,
                return_type,
                body,
            } => {
                let ret_ty = if let Some(ret_str) = return_type {
                    // Resolve explicit type. For simple parser, parse string type names:
                    // e.g. "Sayı?", "Sayı", "Metin", etc.
                    let is_nullable = ret_str.ends_with('?');
                    let base_name = if is_nullable {
                        &ret_str[0..ret_str.len() - 1]
                    } else {
                        ret_str.as_str()
                    };
                    let base_ty = match base_name {
                        "Sayı" | "sayı" => Type::Number,
                        "Metin" | "metin" => Type::String,
                        "Mantıksal" | "mantıksal" => Type::Boolean,
                        "Boş" | "boş" => Type::Bos,
                        _ => {
                            if generics.contains(&base_name.to_string()) {
                                Type::Generic(base_name.to_string())
                            } else {
                                Type::Var(self.new_var())
                            }
                        }
                    };
                    if is_nullable {
                        Type::Option(Box::new(base_ty))
                    } else {
                        base_ty
                    }
                } else {
                    let ret_var = self.new_var();
                    Type::Var(ret_var)
                };

                let mut param_tys = Vec::new();
                let mut body_env = TypeEnv::extend(env);
                for (p_name, p_type_str_opt) in params {
                    let p_ty = if let Some(p_type_str) = p_type_str_opt {
                        let is_nullable = p_type_str.ends_with('?');
                        let base_name = if is_nullable {
                            &p_type_str[0..p_type_str.len() - 1]
                        } else {
                            p_type_str.as_str()
                        };
                        let base_ty = match base_name {
                            "Sayı" | "sayı" => Type::Number,
                            "Metin" | "metin" => Type::String,
                            "Mantıksal" | "mantıksal" => Type::Boolean,
                            "Boş" | "boş" => Type::Bos,
                            _ => {
                                if generics.contains(&base_name.to_string()) {
                                    Type::Generic(base_name.to_string())
                                } else {
                                    Type::Var(self.new_var())
                                }
                            }
                        };
                        if is_nullable {
                            Type::Option(Box::new(base_ty))
                        } else {
                            base_ty
                        }
                    } else {
                        let p_var = self.new_var();
                        Type::Var(p_var)
                    };
                    param_tys.push(p_ty.clone());
                    body_env.set(
                        p_name.clone(),
                        Scheme {
                            vars: vec![],
                            ty: p_ty,
                        },
                    );
                }

                let fn_ty = Type::Function {
                    params: param_tys.clone(),
                    ret: Box::new(ret_ty.clone()),
                };
                body_env.set(
                    name.clone(),
                    Scheme {
                        vars: vec![],
                        ty: fn_ty.clone(),
                    },
                );

                for s in body {
                    self.infer_stmt(s, &mut body_env, &Some(ret_ty.clone()))?;
                }

                let resolved_fn_ty = self.resolve(&fn_ty);
                self.recorded_types
                    .insert(name.clone(), resolved_fn_ty.clone());
                let generalized = self.generalize(&resolved_fn_ty, env);
                env.set(name.clone(), generalized);
            }
            Statement::Return(opt_expr) => {
                if let Some(expected_ret) = current_ret_ty {
                    let actual_ty = if let Some(expr) = opt_expr {
                        self.infer_expr(expr, env, current_ret_ty)?
                    } else {
                        Type::Bos
                    };
                    self.unify(expected_ret, &actual_ty)
                        .map_err(|e| e.with_span(stmt.span.clone()))?;
                } else {
                    // Outside function body (e.g. inside hata_ise block) —
                    // just typecheck the expression, don't enforce Bos return
                    if let Some(expr) = opt_expr {
                        let _actual_ty = self.infer_expr(expr, env, current_ret_ty)?;
                    }
                }
            }
            Statement::Expr(expr) => {
                self.infer_expr(expr, env, current_ret_ty)?;
            }
            Statement::Tamamlaninca(gorev_expr, body) => {
                let task_ty = self.infer_expr(gorev_expr, env, current_ret_ty)?;
                let task_res_var = self.new_var();
                let expected_task_ty = Type::Task(Box::new(Type::Var(task_res_var)));
                self.unify(&task_ty, &expected_task_ty)
                    .map_err(|e| e.with_span(stmt.span.clone()))?;

                let mut body_env = TypeEnv::extend(env);
                let resolved_res_ty = self.resolve(&Type::Var(task_res_var));
                body_env.set(
                    "sonuç".to_string(),
                    Scheme {
                        vars: vec![],
                        ty: resolved_res_ty.clone(),
                    },
                );
                body_env.set(
                    "sonuc".to_string(),
                    Scheme {
                        vars: vec![],
                        ty: resolved_res_ty,
                    },
                );
                for s in body {
                    self.infer_stmt(s, &mut body_env, current_ret_ty)?;
                }
            }
            Statement::Break | Statement::Continue => {}
        }
        Ok(())
    }
}

pub fn create_default_type_env(checker: &mut TypeChecker) -> TypeEnv {
    let mut env = TypeEnv::new();

    let a_var = checker.new_var();
    let yazdir_ty = Type::Function {
        params: vec![Type::Var(a_var)],
        ret: Box::new(Type::Bos),
    };
    env.set(
        "yazdır".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: yazdir_ty.clone(),
        },
    );
    env.set(
        "yazdir".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: yazdir_ty,
        },
    );

    let a_var = checker.new_var();
    let boyut_ty = Type::Function {
        params: vec![Type::Var(a_var)],
        ret: Box::new(Type::Number),
    };
    env.set(
        "boyut".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: boyut_ty,
        },
    );

    let a_var = checker.new_var();
    let ekle_ty = Type::Function {
        params: vec![Type::Array(Box::new(Type::Var(a_var))), Type::Var(a_var)],
        ret: Box::new(Type::Bos),
    };
    env.set(
        "ekle".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: ekle_ty,
        },
    );

    let kok_ty = Type::Function {
        params: vec![Type::Number],
        ret: Box::new(Type::Number),
    };
    env.set(
        "kök".to_string(),
        Scheme {
            vars: vec![],
            ty: kok_ty.clone(),
        },
    );
    env.set(
        "karekok".to_string(),
        Scheme {
            vars: vec![],
            ty: kok_ty,
        },
    );

    let us_ty = Type::Function {
        params: vec![Type::Number, Type::Number],
        ret: Box::new(Type::Number),
    };
    env.set(
        "üs".to_string(),
        Scheme {
            vars: vec![],
            ty: us_ty.clone(),
        },
    );
    env.set(
        "ust".to_string(),
        Scheme {
            vars: vec![],
            ty: us_ty,
        },
    );

    let mutlak_ty = Type::Function {
        params: vec![Type::Number],
        ret: Box::new(Type::Number),
    };
    env.set(
        "mutlak".to_string(),
        Scheme {
            vars: vec![],
            ty: mutlak_ty,
        },
    );

    let simdi_ty = Type::Function {
        params: vec![],
        ret: Box::new(Type::Number),
    };
    env.set(
        "şimdi".to_string(),
        Scheme {
            vars: vec![],
            ty: simdi_ty.clone(),
        },
    );
    env.set(
        "simdi".to_string(),
        Scheme {
            vars: vec![],
            ty: simdi_ty,
        },
    );

    let uyku_ty = Type::Function {
        params: vec![Type::Number],
        ret: Box::new(Type::Bos),
    };
    env.set(
        "uyku".to_string(),
        Scheme {
            vars: vec![],
            ty: uyku_ty,
        },
    );

    let a_var = checker.new_var();
    let b_var = checker.new_var();
    let arkaplanda_ty = Type::Function {
        params: vec![
            Type::Function {
                params: vec![Type::Var(a_var)],
                ret: Box::new(Type::Var(b_var)),
            },
            Type::Var(a_var),
        ],
        ret: Box::new(Type::Task(Box::new(Type::Var(b_var)))),
    };
    env.set(
        "arkaplanda_çalıştır".to_string(),
        Scheme {
            vars: vec![a_var, b_var],
            ty: arkaplanda_ty.clone(),
        },
    );
    env.set(
        "arkaplanda_calistir".to_string(),
        Scheme {
            vars: vec![a_var, b_var],
            ty: arkaplanda_ty,
        },
    );

    let dosya_oku_ty = Type::Function {
        params: vec![Type::String],
        ret: Box::new(Type::String),
    };
    env.set(
        "dosya_oku".to_string(),
        Scheme {
            vars: vec![],
            ty: dosya_oku_ty,
        },
    );

    let dosya_yaz_ty = Type::Function {
        params: vec![Type::String, Type::String],
        ret: Box::new(Type::Boolean),
    };
    env.set(
        "dosya_yaz".to_string(),
        Scheme {
            vars: vec![],
            ty: dosya_yaz_ty,
        },
    );

    let dosya_sil_ty = Type::Function {
        params: vec![Type::String],
        ret: Box::new(Type::Boolean),
    };
    env.set(
        "dosya_sil".to_string(),
        Scheme {
            vars: vec![],
            ty: dosya_sil_ty,
        },
    );

    let a_var = checker.new_var();
    let hata_firlat_ty = Type::Function {
        params: vec![Type::Var(a_var)],
        ret: Box::new(Type::Var(checker.new_var())),
    };
    env.set(
        "hata_fırlat".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: hata_firlat_ty.clone(),
        },
    );
    env.set(
        "hata_firlat".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: hata_firlat_ty,
        },
    );

    let dahil_et_ty = Type::Function {
        params: vec![Type::String],
        ret: Box::new(Type::Bos),
    };
    env.set(
        "dahil_et".to_string(),
        Scheme {
            vars: vec![],
            ty: dahil_et_ty,
        },
    );

    let ch_var = checker.new_var();
    let kanal_ty = Type::Function {
        params: vec![],
        ret: Box::new(Type::Channel(Box::new(Type::Var(ch_var)))),
    };
    env.set(
        "kanal".to_string(),
        Scheme {
            vars: vec![ch_var],
            ty: kanal_ty,
        },
    );

    // --- Yeni eklenen yerleşik fonksiyonlar ---

    // biçimle: variadic, handled specially in infer_expr but needs env entry
    let a_var = checker.new_var();
    let bicimle_ty = Type::Function {
        params: vec![Type::Var(a_var)],
        ret: Box::new(Type::String),
    };
    env.set(
        "biçimle".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: bicimle_ty,
        },
    );

    // uzunluk (metin uzunluğu)
    let uzunluk_ty = Type::Function {
        params: vec![Type::String],
        ret: Box::new(Type::Number),
    };
    env.set(
        "uzunluk".to_string(),
        Scheme {
            vars: vec![],
            ty: uzunluk_ty,
        },
    );

    // böl(metin, ayraç) -> Dizi
    let bol_ty = Type::Function {
        params: vec![Type::String, Type::String],
        ret: Box::new(Type::Array(Box::new(Type::String))),
    };
    env.set(
        "böl".to_string(),
        Scheme {
            vars: vec![],
            ty: bol_ty,
        },
    );

    // birleştir(dizi, ayraç) -> Metin
    let a_var = checker.new_var();
    let birlestir_ty = Type::Function {
        params: vec![Type::Array(Box::new(Type::Var(a_var))), Type::String],
        ret: Box::new(Type::String),
    };
    env.set(
        "birleştir".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: birlestir_ty,
        },
    );

    // içerir(metin, aranan) -> Mantıksal
    let icerir_ty = Type::Function {
        params: vec![Type::String, Type::String],
        ret: Box::new(Type::Boolean),
    };
    env.set(
        "içerir".to_string(),
        Scheme {
            vars: vec![],
            ty: icerir_ty,
        },
    );

    // büyük_harf(metin) -> Metin
    let buyuk_harf_ty = Type::Function {
        params: vec![Type::String],
        ret: Box::new(Type::String),
    };
    env.set(
        "büyük_harf".to_string(),
        Scheme {
            vars: vec![],
            ty: buyuk_harf_ty,
        },
    );

    // küçük_harf(metin) -> Metin
    let kucuk_harf_ty = Type::Function {
        params: vec![Type::String],
        ret: Box::new(Type::String),
    };
    env.set(
        "küçük_harf".to_string(),
        Scheme {
            vars: vec![],
            ty: kucuk_harf_ty,
        },
    );

    // kırp(metin) -> Metin
    let kirp_ty = Type::Function {
        params: vec![Type::String],
        ret: Box::new(Type::String),
    };
    env.set(
        "kırp".to_string(),
        Scheme {
            vars: vec![],
            ty: kirp_ty,
        },
    );

    // tamsayı(sayı) -> Sayı
    let tamsayi_ty = Type::Function {
        params: vec![Type::Number],
        ret: Box::new(Type::Number),
    };
    env.set(
        "tamsayı".to_string(),
        Scheme {
            vars: vec![],
            ty: tamsayi_ty,
        },
    );

    // metne_çevir(değer) -> Metin
    let a_var = checker.new_var();
    let metne_cevir_ty = Type::Function {
        params: vec![Type::Var(a_var)],
        ret: Box::new(Type::String),
    };
    env.set(
        "metne_çevir".to_string(),
        Scheme {
            vars: vec![a_var],
            ty: metne_cevir_ty,
        },
    );

    // sayıya_çevir(metin) -> Sayı
    let sayiya_cevir_ty = Type::Function {
        params: vec![Type::String],
        ret: Box::new(Type::Number),
    };
    env.set(
        "sayıya_çevir".to_string(),
        Scheme {
            vars: vec![],
            ty: sayiya_cevir_ty,
        },
    );

    // rastgele(min, max) -> Sayı
    let rastgele_ty = Type::Function {
        params: vec![Type::Number, Type::Number],
        ret: Box::new(Type::Number),
    };
    env.set(
        "rastgele".to_string(),
        Scheme {
            vars: vec![],
            ty: rastgele_ty,
        },
    );

    env
}

pub fn check_program(stmts: &[Spanned<Statement>]) -> Result<(), super::types::TypeError> {
    let mut checker = TypeChecker::new();
    let mut env = create_default_type_env(&mut checker);
    for stmt in stmts {
        checker.infer_stmt(stmt, &mut env, &None)?;
    }
    Ok(())
}
