use crate::instruction::{Instruction, Val};
use std::collections::HashMap;

struct Frame {
    return_address: usize,
    locals: HashMap<String, Val>,
}

pub struct VM {
    instructions: Vec<Instruction>,
    ip: usize,
    stack: Vec<Val>,
    globals: HashMap<String, Val>,
    frames: Vec<Frame>,
}

impl VM {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        let mut globals = HashMap::new();
        globals.insert("yazdır".to_string(), Val::Builtin("yazdır".to_string()));

        VM {
            instructions,
            ip: 0,
            stack: Vec::new(),
            globals,
            frames: Vec::new(),
        }
    }

    pub fn get_global(&self, name: &str) -> Option<Val> {
        self.globals.get(name).cloned()
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.ip < self.instructions.len() {
            let inst = &self.instructions[self.ip];
            self.ip += 1;

            match inst {
                Instruction::Constant(val) => {
                    self.stack.push(val.clone());
                }
                Instruction::Load(name) => {
                    // Try locals in current frame first, then globals
                    let mut found = false;
                    if let Some(frame) = self.frames.last() {
                        if let Some(val) = frame.locals.get(name) {
                            self.stack.push(val.clone());
                            found = true;
                        }
                    }
                    if !found {
                        if let Some(val) = self.globals.get(name) {
                            self.stack.push(val.clone());
                        } else {
                            return Err(format!("HATA: Tanımlanamayan değişken '{}'", name));
                        }
                    }
                }
                Instruction::Store(name) => {
                    let val = self.stack.pop().ok_or("HATA: Yığın boş (Store)")?;
                    // If we have frames, store in the current frame's locals
                    if let Some(frame) = self.frames.last_mut() {
                        frame.locals.insert(name.clone(), val);
                    } else {
                        self.globals.insert(name.clone(), val);
                    }
                }
                Instruction::Pop => {
                    self.stack.pop().ok_or("HATA: Yığın boş (Pop)")?;
                }
                Instruction::Add => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Add rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Add lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Number(n1 + n2)),
                        (Val::String(s1), Val::String(s2)) => self.stack.push(Val::String(format!("{}{}", s1, s2))),
                        _ => return Err("HATA: Geçersiz toplama".to_string()),
                    }
                }
                Instruction::Sub => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Sub rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Sub lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Number(n1 - n2)),
                        _ => return Err("HATA: Geçersiz çıkarma".to_string()),
                    }
                }
                Instruction::Mul => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Mul rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Mul lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Number(n1 * n2)),
                        _ => return Err("HATA: Geçersiz çarpma".to_string()),
                    }
                }
                Instruction::Div => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Div rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Div lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => {
                            if n2 == 0.0 {
                                return Err("HATA: Sıfıra bölme".to_string());
                            }
                            self.stack.push(Val::Number(n1 / n2));
                        }
                        _ => return Err("HATA: Geçersiz bölme".to_string()),
                    }
                }
                Instruction::Mod => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Mod rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Mod lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Number(n1 % n2)),
                        _ => return Err("HATA: Geçersiz modül".to_string()),
                    }
                }
                Instruction::Eq => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Eq rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Eq lhs)")?;
                    self.stack.push(Val::Boolean(a == b));
                }
                Instruction::Ne => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Ne rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Ne lhs)")?;
                    self.stack.push(Val::Boolean(a != b));
                }
                Instruction::Lt => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Lt rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Lt lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Boolean(n1 < n2)),
                        _ => return Err("HATA: Geçersiz karşılaştırma".to_string()),
                    }
                }
                Instruction::Gt => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Gt rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Gt lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Boolean(n1 > n2)),
                        _ => return Err("HATA: Geçersiz karşılaştırma".to_string()),
                    }
                }
                Instruction::Le => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Le rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Le lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Boolean(n1 <= n2)),
                        _ => return Err("HATA: Geçersiz karşılaştırma".to_string()),
                    }
                }
                Instruction::Ge => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Ge rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Ge lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Boolean(n1 >= n2)),
                        _ => return Err("HATA: Geçersiz karşılaştırma".to_string()),
                    }
                }
                Instruction::And => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (And rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (And lhs)")?;
                    match (a, b) {
                        (Val::Boolean(b1), Val::Boolean(b2)) => self.stack.push(Val::Boolean(b1 && b2)),
                        _ => return Err("HATA: Geçersiz mantıksal işlem".to_string()),
                    }
                }
                Instruction::Or => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Or rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Or lhs)")?;
                    match (a, b) {
                        (Val::Boolean(b1), Val::Boolean(b2)) => self.stack.push(Val::Boolean(b1 || b2)),
                        _ => return Err("HATA: Geçersiz mantıksal işlem".to_string()),
                    }
                }
                Instruction::Jump(dest) => {
                    self.ip = *dest;
                }
                Instruction::JumpIfFalse(dest) => {
                    let val = self.stack.pop().ok_or("HATA: Yığın boş (JumpIfFalse)")?;
                    match val {
                        Val::Boolean(b) => {
                            if !b {
                                self.ip = *dest;
                            }
                        }
                        _ => return Err("HATA: Mantıksal koşul bekleniyordu".to_string()),
                    }
                }
                Instruction::Call(arg_count) => {
                    let func_val = self.stack.pop().ok_or("HATA: Yığın boş (Call)")?;
                    match func_val {
                        Val::Function { param_count, entry_ip, .. } => {
                            if param_count != *arg_count {
                                return Err(format!("HATA: {} parametre bekleniyor, fakat {} verildi", param_count, arg_count));
                            }
                            // Local variables for function frame
                            let locals = HashMap::new();
                            // Frame is pushed: return address is current ip
                            let frame = Frame {
                                return_address: self.ip,
                                locals,
                            };
                            self.frames.push(frame);
                            self.ip = entry_ip;
                        }
                        Val::Builtin(name) => {
                            if name == "yazdır" {
                                let mut args = Vec::new();
                                for _ in 0..*arg_count {
                                    args.push(self.stack.pop().ok_or("HATA: Yığın boş (yazdır)")?);
                                }
                                args.reverse();

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
                                self.stack.push(Val::Bos);
                            } else {
                                return Err(format!("HATA: Bilinmeyen dahili işlev '{}'", name));
                            }
                        }
                        _ => return Err("HATA: Çağrılabilir bir değer değil".to_string()),
                    }
                }
                Instruction::Return => {
                    let ret_val = self.stack.pop().ok_or("HATA: Yığın boş (Return)")?;
                    let frame = self.frames.pop().ok_or("HATA: Çerçeve yığını boş (Return)")?;
                    self.ip = frame.return_address;
                    self.stack.push(ret_val);
                }
            }
        }
        Ok(())
    }
}
