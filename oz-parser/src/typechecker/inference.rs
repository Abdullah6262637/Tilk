use super::types::{Scheme, Type, TypeEnv};
use crate::ast::{BinaryOp, Expr, Literal, Spanned, Statement, UnaryOp};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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
    ) -> Result<Type, String> {
        match &expr.node {
            Expr::Literal(lit) => match lit {
                Literal::Number(_) => Ok(Type::Number),
                Literal::String(_) => Ok(Type::String),
                Literal::Boolean(_) => Ok(Type::Boolean),
                Literal::Bos => Ok(Type::Bos),
            },
            Expr::Identifier(name) => {
                if let Some(scheme) = env.get(name) {
                    let instantiated = self.instantiate(&scheme);
                    let resolved = self.resolve(&instantiated);
                    self.recorded_types.insert(name.clone(), resolved);
                    Ok(instantiated)
                } else {
                    Err(format!("Tip Hatası: Tanımlanamayan değişken '{}'", name))
                }
            }
            Expr::Unary(op, operand) => {
                let t = self.infer_expr(operand, env, current_ret_ty)?;
                match op {
                    UnaryOp::Neg => {
                        self.unify(&t, &Type::Number)?;
                        Ok(Type::Number)
                    }
                    UnaryOp::Not => {
                        self.unify(&t, &Type::Boolean)?;
                        Ok(Type::Boolean)
                    }
                }
            }

            Expr::Binary(lhs, op, text_rhs) => {
                let t1 = self.infer_expr(lhs, env, current_ret_ty)?;
                let t2 = self.infer_expr(text_rhs, env, current_ret_ty)?;
                match op {
                    BinaryOp::Add => {
                        self.unify(&t1, &t2)?;
                        let resolved = self.resolve(&t1);
                        match resolved {
                            Type::Number => Ok(Type::Number),
                            Type::String => Ok(Type::String),
                            Type::Var(id) => {
                                self.substitutions.insert(id, Type::Number);
                                Ok(Type::Number)
                            }
                            _ => Err("Tip Hatası: Toplama işlemi sadece sayılar veya metinler arasında yapılabilir".to_string()),
                        }
                    }
                    BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        self.unify(&t1, &Type::Number)?;
                        self.unify(&t2, &Type::Number)?;
                        Ok(Type::Number)
                    }
                    BinaryOp::Eq | BinaryOp::Ne => {
                        self.unify(&t1, &t2)?;
                        Ok(Type::Boolean)
                    }
                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        self.unify(&t1, &Type::Number)?;
                        self.unify(&t2, &Type::Number)?;
                        Ok(Type::Boolean)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        self.unify(&t1, &Type::Boolean)?;
                        self.unify(&t2, &Type::Boolean)?;
                        Ok(Type::Boolean)
                    }
                }
            }
            Expr::Call(name, args) => {
                if name == "dahil_et" {
                    if args.is_empty() {
                        return Err("Tip Hatası: dahil_et en az bir argüman almalıdır".to_string());
                    }
                    if let Expr::Literal(Literal::String(path_str)) = &args[0].node {
                        let embedded_content = match path_str.as_str() {
                            "std::sonuc" => Some("işlev basarili(deger) { r = {}; r[\"tur\"] = \"basarili\"; r[\"deger\"] = deger; döndür r; } işlev hata(mesaj) { r = {}; r[\"tur\"] = \"hata\"; r[\"hata\"] = mesaj; döndür r; }".to_string()),
                            "std::matematik" => Some("işlev karekok(x) { döndür kök(x); } işlev ust(taban, kuvvet) { döndür üs(taban, kuvvet); } işlev mutlak_deger(x) { döndür mutlak(x); }".to_string()),
                            "std::dosya" => Some("dahil_et(\"std::sonuc\"); işlev oku(yol) { döndür (basarili(dosya_oku(yol))) hata_ise { döndür hata(\"Okuma hatası\"); }; } işlev yaz(yol, icerik) { döndür (basarili(dosya_yaz(yol, icerik))) hata_ise { döndür hata(\"Yazma hatası\"); }; } işlev sil(yol) { döndür (basarili(dosya_sil(yol))) hata_ise { döndür hata(\"Silme hatası\"); }; }".to_string()),
                            "std::zaman" => Some("işlev simdi() { döndür şimdi(); } işlev bekle(ms) { döndür uyku(ms); }".to_string()),
                            _ => None,
                        };

                        let (canonical_path, content) = if let Some(content_str) = embedded_content
                        {
                            (std::path::PathBuf::from(path_str.clone()), content_str)
                        } else {
                            let path = std::path::Path::new(path_str);
                            let canonical_path = std::fs::canonicalize(path).map_err(|e| {
                                format!("Modül yolu çözümlenemedi ({}): {}", path_str, e)
                            })?;
                            let read_content = std::fs::read_to_string(&canonical_path)
                                .map_err(|e| format!("Modül yüklenemedi ({}): {}", path_str, e))?;
                            (canonical_path, read_content)
                        };

                        // 1. Döngüsel Bağımlılık Kontrolü
                        if self.loading_stack.contains(&canonical_path) {
                            return Err(format!(
                                "Tip Hatası: Döngüsel bağımlılık tespit edildi: {}",
                                path_str
                            ));
                        }

                        // 2. Çift Dahil Etme Kontrolü (Include Guard)
                        if self.loaded_files.contains(&canonical_path) {
                            return Ok(Type::Bos);
                        }

                        // Yükleme stack'ine ekle
                        self.loading_stack.push(canonical_path.clone());

                        use logos::Logos;
                        use oz_lexer::Token;
                        let lexer = Token::lexer(&content);
                        let mut tokens = Vec::new();
                        for (token_res, span) in lexer.spanned() {
                            match token_res {
                                Ok(token) => tokens.push((token, span)),
                                Err(_) => {
                                    return Err(format!("Sözcüksel analiz hatası at {:?}", span))
                                }
                            }
                        }

                        let ast = crate::parse_tokens(tokens, content.len())
                            .map_err(|e| format!("Ayrıştırma hatası: {:?}", e))?;

                        for stmt in &ast {
                            self.infer_stmt(stmt, env, current_ret_ty)?;
                        }

                        // Yükleme tamamlandı, stack'ten çıkar ve loaded_files'a ekle
                        self.loading_stack.pop();
                        self.loaded_files.insert(canonical_path);

                        return Ok(Type::Bos);
                    } else {
                        return Err("Tip Hatası: dahil_et parametresi doğrudan metin (literal string) olmalıdır".to_string());
                    }
                }

                if name == "arkaplanda_çalıştır" || name == "arkaplanda_calistir" {
                    if args.is_empty() {
                        return Err(
                            "Tip Hatası: arkaplanda_çalıştır en az bir argüman almalıdır"
                                .to_string(),
                        );
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
                    self.unify(&fn_ty, &expected_fn_ty)?;
                    return Ok(Type::Task(Box::new(self.resolve(&Type::Var(ret_var)))));
                }

                let fn_scheme = env
                    .get(name)
                    .ok_or_else(|| format!("Tip Hatası: Tanımlanamayan işlev '{}'", name))?;
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
                self.unify(&fn_ty, &expected_fn_ty)?;
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
                    self.unify(&key_ty, &Type::String)?;
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
                        self.unify(&idx_ty, &Type::Number)?;
                        Ok(Type::Var(self.new_var()))
                    }
                    Type::Map(_) => {
                        self.unify(&idx_ty, &Type::String)?;
                        Ok(Type::Var(self.new_var()))
                    }
                    Type::Var(id) => {
                        let res_var = self.new_var();
                        let array_ty = Type::Array(Box::new(Type::Var(res_var)));
                        self.substitutions.insert(id, array_ty);
                        self.unify(&idx_ty, &Type::Number)?;
                        Ok(Type::Var(res_var))
                    }
                    _ => Err("Tip Hatası: Sadece diziler ve haritalar indekslenebilir".to_string()),
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
                    self.unify(&base_ty, &body_ty)?;
                }
                Ok(self.resolve(&base_ty))
            }
        }
    }

    pub fn infer_stmt(
        &mut self,
        stmt: &Spanned<Statement>,
        env: &mut TypeEnv,
        current_ret_ty: &Option<Type>,
    ) -> Result<(), String> {
        match &stmt.node {
            Statement::VarDecl(name, value) | Statement::Assignment(name, value) => {
                let val_ty = self.infer_expr(value, env, current_ret_ty)?;
                if let Some(scheme) = env.get(name) {
                    let instantiated = self.instantiate(&scheme);
                    self.unify(&instantiated, &val_ty)?;
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
                        self.unify(&idx_ty, &Type::Number)?;
                    }
                    Type::Map(_) => {
                        self.unify(&idx_ty, &Type::String)?;
                    }
                    Type::Var(id) => {
                        let res_var = self.new_var();
                        let array_ty = Type::Array(Box::new(Type::Var(res_var)));
                        self.substitutions.insert(id, array_ty);
                        self.unify(&idx_ty, &Type::Number)?;
                    }
                    _ => {
                        return Err(
                            "Tip Hatası: Sadece diziler ve haritalar güncellenebilir".to_string()
                        )
                    }
                }
            }
            Statement::If(cond, then_block, else_block) => {
                let cond_ty = self.infer_expr(cond, env, current_ret_ty)?;
                self.unify(&cond_ty, &Type::Boolean)?;

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
                self.unify(&cond_ty, &Type::Boolean)?;

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
                self.unify(&start_ty, &Type::Number)?;
                let end_ty = self.infer_expr(end, env, current_ret_ty)?;
                self.unify(&end_ty, &Type::Number)?;

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
            Statement::FnDecl { name, params, body } => {
                let ret_var = self.new_var();
                let ret_ty = Type::Var(ret_var);

                let mut param_tys = Vec::new();
                let mut body_env = TypeEnv::extend(env);
                for p in params {
                    let p_var = self.new_var();
                    let p_ty = Type::Var(p_var);
                    param_tys.push(p_ty.clone());
                    body_env.set(
                        p.clone(),
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
                    self.unify(expected_ret, &actual_ty)?;
                } else {
                    if let Some(expr) = opt_expr {
                        let actual_ty = self.infer_expr(expr, env, current_ret_ty)?;
                        self.unify(&Type::Bos, &actual_ty)?;
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
                self.unify(&task_ty, &expected_task_ty)?;

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

    env
}

pub fn check_program(stmts: &[Spanned<Statement>]) -> Result<(), String> {
    let mut checker = TypeChecker::new();
    let mut env = create_default_type_env(&mut checker);
    for stmt in stmts {
        checker.infer_stmt(stmt, &mut env, &None)?;
    }
    Ok(())
}
