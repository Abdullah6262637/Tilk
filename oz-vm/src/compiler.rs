use crate::instruction::{Instruction, Val};
use oz_parser::ast::{BinaryOp, Expr, Literal, Spanned, Statement, StepDir, UnaryOp};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum VarRef {
    Local(u16),
    Global(String),
}

pub struct Compiler {
    instructions: Vec<Instruction>,
    scopes: Vec<HashMap<String, u16>>,
    next_local: u16,
    loaded_files: HashSet<PathBuf>,
    loading_stack: Vec<PathBuf>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: Vec::new(),
            scopes: Vec::new(),
            next_local: 0,
            loaded_files: HashSet::new(),
            loading_stack: Vec::new(),
        }
    }

    fn get_variable(&self, name: &str) -> VarRef {
        for scope in self.scopes.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return VarRef::Local(slot);
            }
        }
        VarRef::Global(name.to_string())
    }

    fn declare_variable(&mut self, name: &str) -> VarRef {
        for scope in self.scopes.iter().rev() {
            if let Some(&slot) = scope.get(name) {
                return VarRef::Local(slot);
            }
        }
        if !self.scopes.is_empty() {
            let slot = self.next_local;
            self.next_local += 1;
            self.scopes
                .last_mut()
                .unwrap()
                .insert(name.to_string(), slot);
            VarRef::Local(slot)
        } else {
            VarRef::Global(name.to_string())
        }
    }

    pub fn compile_program(
        mut self,
        stmts: &[Spanned<Statement>],
    ) -> Result<Vec<Instruction>, String> {
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        Ok(self.instructions)
    }

    fn compile_expr(&mut self, expr: &Spanned<Expr>) -> Result<(), String> {
        match &expr.node {
            Expr::Literal(lit) => match lit {
                Literal::Number(n) => self
                    .instructions
                    .push(Instruction::Constant(Val::Number(*n))),
                Literal::String(s) => self
                    .instructions
                    .push(Instruction::Constant(Val::String(s.clone()))),
                Literal::Boolean(b) => self
                    .instructions
                    .push(Instruction::Constant(Val::Boolean(*b))),
                Literal::Bos => self.instructions.push(Instruction::Constant(Val::Bos)),
            },
            Expr::Identifier(name) => match self.get_variable(name) {
                VarRef::Local(slot) => self.instructions.push(Instruction::LoadLocal(slot)),
                VarRef::Global(slot) => self.instructions.push(Instruction::LoadGlobal(slot)),
            },
            Expr::Unary(op, operand) => {
                self.compile_expr(operand)?;
                match op {
                    UnaryOp::Neg => self.instructions.push(Instruction::Neg),
                    UnaryOp::Not => self.instructions.push(Instruction::Not),
                }
            }
            Expr::Binary(lhs, op, rhs) => match op {
                BinaryOp::And => {
                    self.compile_expr(lhs)?;
                    let jump_idx = self.instructions.len();
                    self.instructions.push(Instruction::JumpIfFalseKeep(0));
                    self.instructions.push(Instruction::Pop);
                    self.compile_expr(rhs)?;
                    let end_idx = self.instructions.len();
                    self.instructions[jump_idx] = Instruction::JumpIfFalseKeep(end_idx);
                }
                BinaryOp::Or => {
                    self.compile_expr(lhs)?;
                    let jump_idx = self.instructions.len();
                    self.instructions.push(Instruction::JumpIfTrueKeep(0));
                    self.instructions.push(Instruction::Pop);
                    self.compile_expr(rhs)?;
                    let end_idx = self.instructions.len();
                    self.instructions[jump_idx] = Instruction::JumpIfTrueKeep(end_idx);
                }
                _ => {
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
                        BinaryOp::And | BinaryOp::Or => unreachable!(),
                    };
                    self.instructions.push(inst);
                }
            },

            Expr::Call(name, args) => {
                if name == "dahil_et" {
                    if args.len() != 1 {
                        return Err(
                            "HATA: dahil_et tek bir dosya yolu parametresi almalıdır".to_string()
                        );
                    }
                    if let Expr::Literal(Literal::String(path_str)) = &args[0].node {
                        let embedded_content = match path_str.as_str() {
                            "std::sonuc" => Some("işlev basarili(deger) { r = {}; r[\"tur\"] = \"basarili\"; r[\"deger\"] = deger; döndür r; } işlev hata(mesaj) { r = {}; r[\"tur\"] = \"hata\"; r[\"hata\"] = mesaj; döndür r; }".to_string()),
                            "std::matematik" => Some("işlev karekok(x) { döndür kök(x); } işlev ust(taban, kuvvet) { döndür üs(taban, kuvvet); } işlev mutlak_deger(x) { döndür mutlak(x); }".to_string()),
                            "std::dosya" => Some("dahil_et(\"std::sonuc\"); işlev oku(yol) { döndür (basarili(dosya_oku(yol))) hata_ise { döndür hata(\"Okuma hatası\"); }; } işlev yaz_yardimci(yol, icerik) { dosya_yaz(yol, icerik); döndür basarili(boş); } işlev yaz(yol, icerik) { döndür (yaz_yardimci(yol, icerik)) hata_ise { döndür hata(\"Yazma hatası\"); }; } işlev sil_yardimci(yol) { dosya_sil(yol); döndür basarili(boş); } işlev sil(yol) { döndür (sil_yardimci(yol)) hata_ise { döndür hata(\"Silme hatası\"); }; }".to_string()),
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
                                "HATA: Döngüsel bağımlılık tespit edildi: {}",
                                path_str
                            ));
                        }

                        // 2. Çift Dahil Etme Kontrolü (Include Guard)
                        if self.loaded_files.contains(&canonical_path) {
                            self.instructions.push(Instruction::Constant(Val::Bos));
                            return Ok(());
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

                        let ast = oz_parser::parse_tokens(tokens, content.len())
                            .map_err(|e| format!("Ayrıştırma hatası: {:?}", e))?;

                        for stmt in &ast {
                            self.compile_stmt(stmt)?;
                        }

                        // Yükleme tamamlandı, stack'ten çıkar ve loaded_files'a ekle
                        self.loading_stack.pop();
                        self.loaded_files.insert(canonical_path);

                        self.instructions.push(Instruction::Constant(Val::Bos));
                        return Ok(());
                    } else {
                        return Err("HATA: Derleme zamanı dahil_et parametresi doğrudan metin (literal string) olmalıdır".to_string());
                    }
                }

                for arg in args {
                    self.compile_expr(arg)?;
                }
                match self.get_variable(name) {
                    VarRef::Local(slot) => self.instructions.push(Instruction::LoadLocal(slot)),
                    VarRef::Global(slot) => self.instructions.push(Instruction::LoadGlobal(slot)),
                }
                self.instructions.push(Instruction::Call(args.len()));
            }
            Expr::Array(elements) => {
                for el in elements {
                    self.compile_expr(el)?;
                }
                self.instructions.push(Instruction::Array(elements.len()));
            }
            Expr::Map(pairs) => {
                for (key_expr, val_expr) in pairs {
                    self.compile_expr(key_expr)?;
                    self.compile_expr(val_expr)?;
                }
                self.instructions.push(Instruction::Map(pairs.len()));
            }
            Expr::Index(array_expr, index_expr) => {
                self.compile_expr(array_expr)?;
                self.compile_expr(index_expr)?;
                self.instructions.push(Instruction::Index);
            }
            Expr::HataIse(base, body) => {
                self.compile_expr(base)?;
                let jump_error_idx = self.instructions.len();
                self.instructions.push(Instruction::JumpIfError(0));

                let err_var = self.declare_variable("hata_mesajı");
                match &err_var {
                    VarRef::Local(slot) => {
                        self.instructions.push(Instruction::StoreLocal(*slot));
                        self.instructions.push(Instruction::LoadLocal(*slot));
                    }
                    VarRef::Global(slot) => {
                        self.instructions.push(Instruction::StoreGlobal(slot.clone()));
                        self.instructions.push(Instruction::LoadGlobal(slot.clone()));
                    }
                }
                let err_var_ascii = self.declare_variable("hata_mesaji");
                match &err_var_ascii {
                    VarRef::Local(slot) => self.instructions.push(Instruction::StoreLocal(*slot)),
                    VarRef::Global(slot) => self.instructions.push(Instruction::StoreGlobal(slot.clone())),
                }
                for stmt in body {
                    self.compile_stmt(stmt)?;
                }
                self.instructions.push(Instruction::Constant(Val::Bos));
                let jump_end_idx = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));
                let else_start = self.instructions.len();
                self.instructions[jump_error_idx] = Instruction::JumpIfError(else_start);
                let end_idx = self.instructions.len();
                self.instructions[jump_end_idx] = Instruction::Jump(end_idx);
            }
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Spanned<Statement>) -> Result<(), String> {
        match &stmt.node {
            Statement::VarDecl(name, value) | Statement::Assignment(name, value) => {
                self.compile_expr(value)?;
                match self.declare_variable(name) {
                    VarRef::Local(slot) => self.instructions.push(Instruction::StoreLocal(slot)),
                    VarRef::Global(slot) => self.instructions.push(Instruction::StoreGlobal(slot)),
                }
            }

            Statement::IndexAssignment(array, index, value) => {
                self.compile_expr(array)?;
                self.compile_expr(index)?;
                self.compile_expr(value)?;
                self.instructions.push(Instruction::IndexStore);
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
                let var_ref = self.declare_variable(var);
                match &var_ref {
                    VarRef::Local(slot) => self.instructions.push(Instruction::StoreLocal(*slot)),
                    VarRef::Global(slot) => self
                        .instructions
                        .push(Instruction::StoreGlobal(slot.clone())),
                }

                let loop_start = self.instructions.len();

                match &var_ref {
                    VarRef::Local(slot) => self.instructions.push(Instruction::LoadLocal(*slot)),
                    VarRef::Global(slot) => self
                        .instructions
                        .push(Instruction::LoadGlobal(slot.clone())),
                }
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

                match &var_ref {
                    VarRef::Local(slot) => self.instructions.push(Instruction::LoadLocal(*slot)),
                    VarRef::Global(slot) => self
                        .instructions
                        .push(Instruction::LoadGlobal(slot.clone())),
                }
                self.instructions
                    .push(Instruction::Constant(Val::Number(1.0)));
                match step_dir {
                    StepDir::Artarak => self.instructions.push(Instruction::Add),
                    StepDir::Azalarak => self.instructions.push(Instruction::Sub),
                }
                match &var_ref {
                    VarRef::Local(slot) => self.instructions.push(Instruction::StoreLocal(*slot)),
                    VarRef::Global(slot) => self
                        .instructions
                        .push(Instruction::StoreGlobal(slot.clone())),
                }

                self.instructions.push(Instruction::Jump(loop_start));

                let loop_end = self.instructions.len();
                self.instructions[jump_end_idx] = Instruction::JumpIfFalse(loop_end);
            }
            Statement::FnDecl { name, params, body } => {
                let jump_over_idx = self.instructions.len();
                self.instructions.push(Instruction::Jump(0));

                let fn_start = self.instructions.len();

                // local scope
                self.scopes.push(HashMap::new());
                let old_next_local = self.next_local;
                self.next_local = 0;

                for param in params.iter().rev() {
                    let param_ref = self.declare_variable(param);
                    match &param_ref {
                        VarRef::Local(slot) => {
                            self.instructions.push(Instruction::StoreLocal(*slot))
                        }
                        VarRef::Global(slot) => self
                            .instructions
                            .push(Instruction::StoreGlobal(slot.clone())),
                    }
                }

                for s in body {
                    self.compile_stmt(s)?;
                }
                self.instructions.push(Instruction::Constant(Val::Bos));
                self.instructions.push(Instruction::Return);

                self.scopes.pop();
                self.next_local = old_next_local;

                let fn_end = self.instructions.len();
                self.instructions[jump_over_idx] = Instruction::Jump(fn_end);

                self.instructions.push(Instruction::Constant(Val::Function {
                    name: name.clone(),
                    param_count: params.len(),
                    entry_ip: fn_start,
                }));

                let fn_var = self.declare_variable(name);
                match &fn_var {
                    VarRef::Local(slot) => self.instructions.push(Instruction::StoreLocal(*slot)),
                    VarRef::Global(slot) => self
                        .instructions
                        .push(Instruction::StoreGlobal(slot.clone())),
                }
            }
            Statement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    self.compile_expr(expr)?;
                } else {
                    self.instructions.push(Instruction::Constant(Val::Bos));
                }
                self.instructions.push(Instruction::Return);
            }
            Statement::Tamamlaninca(gorev_expr, body) => {
                self.compile_expr(gorev_expr)?;
                self.instructions.push(Instruction::AwaitTask);

                let sonuc_ref = self.declare_variable("sonuç");
                match &sonuc_ref {
                    VarRef::Local(slot) => self.instructions.push(Instruction::StoreLocal(*slot)),
                    VarRef::Global(slot) => self
                        .instructions
                        .push(Instruction::StoreGlobal(slot.clone())),
                }

                match &sonuc_ref {
                    VarRef::Local(slot) => self.instructions.push(Instruction::LoadLocal(*slot)),
                    VarRef::Global(slot) => self
                        .instructions
                        .push(Instruction::LoadGlobal(slot.clone())),
                }

                let sonuc_ref_ascii = self.declare_variable("sonuc");
                match &sonuc_ref_ascii {
                    VarRef::Local(slot) => self.instructions.push(Instruction::StoreLocal(*slot)),
                    VarRef::Global(slot) => self
                        .instructions
                        .push(Instruction::StoreGlobal(slot.clone())),
                }

                for s in body {
                    self.compile_stmt(s)?;
                }
            }
        }
        Ok(())
    }
}
