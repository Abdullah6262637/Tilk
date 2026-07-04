use crate::instruction::{Instruction, Val};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

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
        globals.insert("boyut".to_string(), Val::Builtin("boyut".to_string()));
        globals.insert("ekle".to_string(), Val::Builtin("ekle".to_string()));
        globals.insert("hata_fırlat".to_string(), Val::Builtin("hata_fırlat".to_string()));
        globals.insert("dosya_oku".to_string(), Val::Builtin("dosya_oku".to_string()));
        globals.insert("dosya_yaz".to_string(), Val::Builtin("dosya_yaz".to_string()));
        globals.insert("dosya_sil".to_string(), Val::Builtin("dosya_sil".to_string()));

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
                                        Val::Array(arr) => {
                                            let items = arr.borrow();
                                            print!("[");
                                            for (idx, item) in items.iter().enumerate() {
                                                if idx > 0 {
                                                    print!(", ");
                                                }
                                                match item {
                                                    Val::Number(n) => print!("{}", n),
                                                    Val::String(s) => print!("{}", s),
                                                    Val::Boolean(b) => print!("{}", if *b { "doğru" } else { "yanlış" }),
                                                    Val::Bos => print!("boş"),
                                                    _ => print!("{:?}", item),
                                                }
                                            }
                                            print!("]");
                                        }
                                        _ => print!("{:?}", arg),
                                    }
                                }
                                println!();
                                self.stack.push(Val::Bos);
                            } else if name == "boyut" {
                                if *arg_count != 1 {
                                    return Err("HATA: boyut() tek bir parametre alır".to_string());
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (boyut)")?;
                                if let Val::Array(arr) = arg {
                                    self.stack.push(Val::Number(arr.borrow().len() as f64));
                                } else {
                                    self.stack.push(Val::Number(0.0));
                                }
                            } else if name == "ekle" {
                                if *arg_count != 2 {
                                    return Err("HATA: ekle() iki parametre alır".to_string());
                                }
                                let val = self.stack.pop().ok_or("HATA: Yığın boş (ekle val)")?;
                                let arr_val = self.stack.pop().ok_or("HATA: Yığın boş (ekle arr)")?;
                                if let Val::Array(arr) = arr_val {
                                    arr.borrow_mut().push(val);
                                }
                                self.stack.push(Val::Bos);
                            } else if name == "hata_fırlat" {
                                if *arg_count >= 1 {
                                    let arg = self.stack.pop().ok_or("HATA: Yığın boş (hata_fırlat)")?;
                                    let msg = match arg {
                                        Val::String(s) => s,
                                        _ => format!("{:?}", arg),
                                    };
                                    self.stack.push(Val::Hata(msg));
                                } else {
                                    self.stack.push(Val::Hata("Bilinmeyen hata".to_string()));
                                }
                            } else if name == "dosya_oku" {
                                if *arg_count != 1 {
                                    return Err("HATA: dosya_oku() tek bir parametre alır".to_string());
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (dosya_oku)")?;
                                if let Val::String(path) = arg {
                                    match std::fs::read_to_string(path) {
                                        Ok(content) => self.stack.push(Val::String(content)),
                                        Err(e) => self.stack.push(Val::Hata(format!("Dosya okunamadı: {}", e))),
                                    }
                                } else {
                                    self.stack.push(Val::Hata("Geçersiz argüman: dosya_oku(yol)".to_string()));
                                }
                            } else if name == "dosya_yaz" {
                                if *arg_count != 2 {
                                    return Err("HATA: dosya_yaz() iki parametre alır".to_string());
                                }
                                let content_val = self.stack.pop().ok_or("HATA: Yığın boş (dosya_yaz content)")?;
                                let path_val = self.stack.pop().ok_or("HATA: Yığın boş (dosya_yaz path)")?;
                                if let (Val::String(path), Val::String(content)) = (path_val, content_val) {
                                    match std::fs::write(path, content) {
                                        Ok(_) => self.stack.push(Val::Boolean(true)),
                                        Err(e) => self.stack.push(Val::Hata(format!("Dosya yazılamadı: {}", e))),
                                    }
                                } else {
                                    self.stack.push(Val::Hata("Geçersiz argüman: dosya_yaz(yol, içerik)".to_string()));
                                }
                            } else if name == "dosya_sil" {
                                if *arg_count != 1 {
                                    return Err("HATA: dosya_sil() tek bir parametre alır".to_string());
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (dosya_sil)")?;
                                if let Val::String(path) = arg {
                                    match std::fs::remove_file(path) {
                                        Ok(_) => self.stack.push(Val::Boolean(true)),
                                        Err(e) => self.stack.push(Val::Hata(format!("Dosya silinemedi: {}", e))),
                                    }
                                } else {
                                    self.stack.push(Val::Hata("Geçersiz argüman: dosya_sil(yol)".to_string()));
                                }
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
                Instruction::Array(size) => {
                    let mut elements = Vec::new();
                    for _ in 0..*size {
                        elements.push(self.stack.pop().ok_or("HATA: Yığın boş (Array)")?);
                    }
                    elements.reverse();
                    self.stack.push(Val::Array(Rc::new(RefCell::new(elements))));
                }
                Instruction::Index => {
                    let index_val = self.stack.pop().ok_or("HATA: Yığın boş (Index idx)")?;
                    let array_val = self.stack.pop().ok_or("HATA: Yığın boş (Index arr)")?;
                    match array_val {
                        Val::Array(arr) => {
                            match index_val {
                                Val::Number(n) => {
                                    let idx = n as usize;
                                    let items = arr.borrow();
                                    if idx < items.len() {
                                        self.stack.push(items[idx].clone());
                                    } else {
                                        return Err(format!("HATA: Dizi sınırları dışında: indeks {}, boyut {}", idx, items.len()));
                                    }
                                }
                                _ => return Err("HATA: Dizi indeksi sayı olmalıdır".to_string()),
                            }
                        }
                        _ => return Err("HATA: Sadece diziler indekslenebilir".to_string()),
                    }
                }
                Instruction::JumpIfError(dest) => {
                    let val = self.stack.pop().ok_or("HATA: Yığın boş (JumpIfError)")?;
                    match val {
                        Val::Hata(msg) => {
                            self.stack.push(Val::String(msg));
                        }
                        _ => {
                            self.stack.push(val);
                            self.ip = *dest;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
