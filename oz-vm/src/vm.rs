use crate::instruction::{Instruction, TaskState, Val};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct Frame {
    return_address: usize,
    slots: Vec<Val>,
}

pub struct VM {
    instructions: Vec<Instruction>,
    ip: usize,
    stack: Vec<Val>,
    globals: HashMap<String, Val>,
    frames: Vec<Frame>,
    pub stdout: Option<Rc<RefCell<Vec<u8>>>>,
}

impl VM {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        let mut globals = HashMap::new();
        globals.insert("yazdır".to_string(), Val::Builtin("yazdır".to_string()));
        globals.insert("boyut".to_string(), Val::Builtin("boyut".to_string()));
        globals.insert("ekle".to_string(), Val::Builtin("ekle".to_string()));
        globals.insert(
            "hata_fırlat".to_string(),
            Val::Builtin("hata_fırlat".to_string()),
        );
        globals.insert(
            "dosya_oku".to_string(),
            Val::Builtin("dosya_oku".to_string()),
        );
        globals.insert(
            "dosya_yaz".to_string(),
            Val::Builtin("dosya_yaz".to_string()),
        );
        globals.insert(
            "dosya_sil".to_string(),
            Val::Builtin("dosya_sil".to_string()),
        );
        globals.insert(
            "arkaplanda_çalıştır".to_string(),
            Val::Builtin("arkaplanda_çalıştır".to_string()),
        );
        globals.insert(
            "arkaplanda_calistir".to_string(),
            Val::Builtin("arkaplanda_çalıştır".to_string()),
        );
        globals.insert("kök".to_string(), Val::Builtin("kök".to_string()));
        globals.insert("karekok".to_string(), Val::Builtin("kök".to_string()));
        globals.insert("üs".to_string(), Val::Builtin("üs".to_string()));
        globals.insert("ust".to_string(), Val::Builtin("üs".to_string()));
        globals.insert("mutlak".to_string(), Val::Builtin("mutlak".to_string()));
        globals.insert("şimdi".to_string(), Val::Builtin("şimdi".to_string()));
        globals.insert("simdi".to_string(), Val::Builtin("şimdi".to_string()));
        globals.insert("uyku".to_string(), Val::Builtin("uyku".to_string()));

        VM {
            instructions,
            ip: 0,
            stack: Vec::new(),
            globals,
            frames: Vec::new(),
            stdout: None,
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
                Instruction::LoadLocal(slot) => {
                    if let Some(frame) = self.frames.last() {
                        if let Some(val) = frame.slots.get(*slot as usize) {
                            self.stack.push(val.clone());
                        } else {
                            self.stack.push(Val::Bos);
                        }
                    } else {
                        return Err(
                            "HATA: Yerel değişken yerel yığın dışında kullanıldı".to_string()
                        );
                    }
                }
                Instruction::StoreLocal(slot) => {
                    let val = self.stack.pop().ok_or("HATA: Yığın boş (StoreLocal)")?;
                    if let Some(frame) = self.frames.last_mut() {
                        let idx = *slot as usize;
                        if idx >= frame.slots.len() {
                            frame.slots.resize(idx + 1, Val::Bos);
                        }
                        frame.slots[idx] = val;
                    } else {
                        return Err(
                            "HATA: Yerel değişken yerel yığın dışında kullanıldı".to_string()
                        );
                    }
                }
                Instruction::LoadGlobal(name) => {
                    if let Some(val) = self.globals.get(name) {
                        self.stack.push(val.clone());
                    } else {
                        return Err(format!("HATA: Tanımlanamayan değişken '{}'", name));
                    }
                }
                Instruction::StoreGlobal(name) => {
                    let val = self.stack.pop().ok_or("HATA: Yığın boş (StoreGlobal)")?;
                    self.globals.insert(name.clone(), val);
                }
                Instruction::Pop => {
                    self.stack.pop().ok_or("HATA: Yığın boş (Pop)")?;
                }

                Instruction::Add => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Add rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Add lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => self.stack.push(Val::Number(n1 + n2)),
                        (Val::String(s1), Val::String(s2)) => {
                            self.stack.push(Val::String(format!("{}{}", s1, s2)))
                        }
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
                        (Val::Number(n1), Val::Number(n2)) => {
                            self.stack.push(Val::Boolean(n1 < n2))
                        }
                        _ => return Err("HATA: Geçersiz karşılaştırma".to_string()),
                    }
                }
                Instruction::Gt => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Gt rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Gt lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => {
                            self.stack.push(Val::Boolean(n1 > n2))
                        }
                        _ => return Err("HATA: Geçersiz karşılaştırma".to_string()),
                    }
                }
                Instruction::Le => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Le rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Le lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => {
                            self.stack.push(Val::Boolean(n1 <= n2))
                        }
                        _ => return Err("HATA: Geçersiz karşılaştırma".to_string()),
                    }
                }
                Instruction::Ge => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Ge rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Ge lhs)")?;
                    match (a, b) {
                        (Val::Number(n1), Val::Number(n2)) => {
                            self.stack.push(Val::Boolean(n1 >= n2))
                        }
                        _ => return Err("HATA: Geçersiz karşılaştırma".to_string()),
                    }
                }
                Instruction::And => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (And rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (And lhs)")?;
                    match (a, b) {
                        (Val::Boolean(b1), Val::Boolean(b2)) => {
                            self.stack.push(Val::Boolean(b1 && b2))
                        }
                        _ => return Err("HATA: Geçersiz mantıksal işlem".to_string()),
                    }
                }
                Instruction::Or => {
                    let b = self.stack.pop().ok_or("HATA: Yığın boş (Or rhs)")?;
                    let a = self.stack.pop().ok_or("HATA: Yığın boş (Or lhs)")?;
                    match (a, b) {
                        (Val::Boolean(b1), Val::Boolean(b2)) => {
                            self.stack.push(Val::Boolean(b1 || b2))
                        }
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
                Instruction::JumpIfFalseKeep(dest) => {
                    let val = self
                        .stack
                        .last()
                        .ok_or("HATA: Yığın boş (JumpIfFalseKeep)")?;
                    match val {
                        Val::Boolean(b) => {
                            if !*b {
                                self.ip = *dest;
                            }
                        }
                        _ => return Err("HATA: Mantıksal koşul bekleniyordu".to_string()),
                    }
                }
                Instruction::JumpIfTrueKeep(dest) => {
                    let val = self
                        .stack
                        .last()
                        .ok_or("HATA: Yığın boş (JumpIfTrueKeep)")?;
                    match val {
                        Val::Boolean(b) => {
                            if *b {
                                self.ip = *dest;
                            }
                        }
                        _ => return Err("HATA: Mantıksal koşul bekleniyordu".to_string()),
                    }
                }
                Instruction::Neg => {
                    let val = self.stack.pop().ok_or("HATA: Yığın boş (Neg)")?;
                    match val {
                        Val::Number(n) => self.stack.push(Val::Number(-n)),
                        _ => {
                            return Err(
                                "HATA: Negatif işlem sadece sayılarla yapılabilir".to_string()
                            )
                        }
                    }
                }
                Instruction::Not => {
                    let val = self.stack.pop().ok_or("HATA: Yığın boş (Not)")?;
                    match val {
                        Val::Boolean(b) => self.stack.push(Val::Boolean(!b)),
                        _ => return Err(
                            "HATA: Mantıksal değil işlemi sadece mantıksal değerlerle yapılabilir"
                                .to_string(),
                        ),
                    }
                }

                Instruction::Call(arg_count) => {
                    let func_val = self.stack.pop().ok_or("HATA: Yığın boş (Call)")?;
                    match func_val {
                        Val::Function {
                            param_count,
                            entry_ip,
                            ..
                        } => {
                            if param_count != *arg_count {
                                return Err(format!(
                                    "HATA: {} parametre bekleniyor, fakat {} verildi",
                                    param_count, arg_count
                                ));
                            }
                            let frame = Frame {
                                return_address: self.ip,
                                slots: Vec::new(),
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

                                let mut output = String::new();
                                for (i, arg) in args.iter().enumerate() {
                                    if i > 0 {
                                        output.push(' ');
                                    }
                                    output.push_str(&format_val(arg));
                                }
                                output.push('\n');

                                if let Some(stdout_ref) = &self.stdout {
                                    stdout_ref.borrow_mut().extend_from_slice(output.as_bytes());
                                } else {
                                    print!("{}", output);
                                }
                                self.stack.push(Val::Bos);
                            } else if name == "boyut" {
                                if *arg_count != 1 {
                                    return Err("HATA: boyut() tek bir parametre alır".to_string());
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (boyut)")?;
                                match arg {
                                    Val::Array(arr) => {
                                        self.stack.push(Val::Number(arr.borrow().len() as f64))
                                    }
                                    Val::Map(map) => {
                                        self.stack.push(Val::Number(map.borrow().len() as f64))
                                    }
                                    _ => self.stack.push(Val::Number(0.0)),
                                }
                            } else if name == "ekle" {
                                if *arg_count != 2 {
                                    return Err("HATA: ekle() iki parametre alır".to_string());
                                }
                                let val = self.stack.pop().ok_or("HATA: Yığın boş (ekle val)")?;
                                let arr_val =
                                    self.stack.pop().ok_or("HATA: Yığın boş (ekle arr)")?;
                                if let Val::Array(arr) = arr_val {
                                    arr.borrow_mut().push(val);
                                }
                                self.stack.push(Val::Bos);
                            } else if name == "hata_fırlat" {
                                if *arg_count >= 1 {
                                    let arg =
                                        self.stack.pop().ok_or("HATA: Yığın boş (hata_fırlat)")?;
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
                                    return Err(
                                        "HATA: dosya_oku() tek bir parametre alır".to_string()
                                    );
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (dosya_oku)")?;
                                if let Val::String(path) = arg {
                                    match std::fs::read_to_string(path) {
                                        Ok(content) => self.stack.push(Val::String(content)),
                                        Err(e) => self
                                            .stack
                                            .push(Val::Hata(format!("Dosya okunamadı: {}", e))),
                                    }
                                } else {
                                    self.stack.push(Val::Hata(
                                        "Geçersiz argüman: dosya_oku(yol)".to_string(),
                                    ));
                                }
                            } else if name == "dosya_yaz" {
                                if *arg_count != 2 {
                                    return Err("HATA: dosya_yaz() iki parametre alır".to_string());
                                }
                                let content_val = self
                                    .stack
                                    .pop()
                                    .ok_or("HATA: Yığın boş (dosya_yaz content)")?;
                                let path_val =
                                    self.stack.pop().ok_or("HATA: Yığın boş (dosya_yaz path)")?;
                                if let (Val::String(path), Val::String(content)) =
                                    (path_val, content_val)
                                {
                                    match std::fs::write(path, content) {
                                        Ok(_) => self.stack.push(Val::Boolean(true)),
                                        Err(e) => self
                                            .stack
                                            .push(Val::Hata(format!("Dosya yazılamadı: {}", e))),
                                    }
                                } else {
                                    self.stack.push(Val::Hata(
                                        "Geçersiz argüman: dosya_yaz(yol, içerik)".to_string(),
                                    ));
                                }
                            } else if name == "dosya_sil" {
                                if *arg_count != 1 {
                                    return Err(
                                        "HATA: dosya_sil() tek bir parametre alır".to_string()
                                    );
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (dosya_sil)")?;
                                if let Val::String(path) = arg {
                                    match std::fs::remove_file(path) {
                                        Ok(_) => self.stack.push(Val::Boolean(true)),
                                        Err(e) => self
                                            .stack
                                            .push(Val::Hata(format!("Dosya silinemedi: {}", e))),
                                    }
                                } else {
                                    self.stack.push(Val::Hata(
                                        "Geçersiz argüman: dosya_sil(yol)".to_string(),
                                    ));
                                }
                            } else if name == "arkaplanda_çalıştır" {
                                let mut call_args = Vec::new();
                                for _ in 0..*arg_count {
                                    call_args.push(
                                        self.stack
                                            .pop()
                                            .ok_or("HATA: Yığın boş (calistir args)")?,
                                    );
                                }
                                call_args.reverse();
                                if call_args.len() >= 1 {
                                    let func = call_args[0].clone();
                                    let func_args = call_args[1..].to_vec();
                                    self.stack.push(Val::Task(Rc::new(RefCell::new(TaskState {
                                        completed: false,
                                        func,
                                        args: func_args,
                                        result: Val::Bos,
                                    }))));
                                } else {
                                    self.stack.push(Val::Hata(
                                        "Geçersiz argüman: arkaplanda_çalıştır(işlev, ...)"
                                            .to_string(),
                                    ));
                                }
                            } else if name == "kök" {
                                if *arg_count != 1 {
                                    return Err("HATA: kök() tek bir parametre alır".to_string());
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (kök)")?;
                                if let Val::Number(n) = arg {
                                    if n >= 0.0 {
                                        self.stack.push(Val::Number(n.sqrt()));
                                    } else {
                                        self.stack.push(Val::Hata(
                                            "Negatif sayının karekökü alınamaz".to_string(),
                                        ));
                                    }
                                } else {
                                    self.stack
                                        .push(Val::Hata("Geçersiz argüman: kök(sayı)".to_string()));
                                }
                            } else if name == "üs" {
                                if *arg_count != 2 {
                                    return Err("HATA: üs() iki parametre alır".to_string());
                                }
                                let exp_val = self.stack.pop().ok_or("HATA: Yığın boş (üs exp)")?;
                                let base_val =
                                    self.stack.pop().ok_or("HATA: Yığın boş (üs base)")?;
                                if let (Val::Number(base), Val::Number(exponent)) =
                                    (base_val, exp_val)
                                {
                                    self.stack.push(Val::Number(base.powf(exponent)));
                                } else {
                                    self.stack.push(Val::Hata(
                                        "Geçersiz argüman: üs(taban, üs)".to_string(),
                                    ));
                                }
                            } else if name == "mutlak" {
                                if *arg_count != 1 {
                                    return Err("HATA: mutlak() tek bir parametre alır".to_string());
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (mutlak)")?;
                                if let Val::Number(n) = arg {
                                    self.stack.push(Val::Number(n.abs()));
                                } else {
                                    self.stack.push(Val::Hata(
                                        "Geçersiz argüman: mutlak(sayı)".to_string(),
                                    ));
                                }
                            } else if name == "şimdi" {
                                if *arg_count != 0 {
                                    return Err("HATA: şimdi() parametre almaz".to_string());
                                }
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default();
                                self.stack.push(Val::Number(now.as_secs_f64()));
                            } else if name == "uyku" {
                                if *arg_count != 1 {
                                    return Err("HATA: uyku() tek bir parametre alır".to_string());
                                }
                                let arg = self.stack.pop().ok_or("HATA: Yığın boş (uyku)")?;
                                if let Val::Number(ms) = arg {
                                    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
                                    self.stack.push(Val::Bos);
                                } else {
                                    self.stack.push(Val::Hata(
                                        "Geçersiz argüman: uyku(milisaniye)".to_string(),
                                    ));
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
                    let frame = self
                        .frames
                        .pop()
                        .ok_or("HATA: Çerçeve yığını boş (Return)")?;
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
                Instruction::Map(size) => {
                    let mut map = HashMap::new();
                    for _ in 0..*size {
                        let val = self.stack.pop().ok_or("HATA: Yığın boş (Map val)")?;
                        let key = self.stack.pop().ok_or("HATA: Yığın boş (Map key)")?;
                        if let Val::String(s) = key {
                            map.insert(s, val);
                        } else {
                            return Err("HATA: Harita anahtarı metin olmak zorundadır".to_string());
                        }
                    }
                    self.stack.push(Val::Map(Rc::new(RefCell::new(map))));
                }
                Instruction::Index => {
                    let index_val = self.stack.pop().ok_or("HATA: Yığın boş (Index idx)")?;
                    let array_val = self.stack.pop().ok_or("HATA: Yığın boş (Index arr)")?;
                    match array_val {
                        Val::Array(arr) => match index_val {
                            Val::Number(n) => {
                                let idx = n as usize;
                                let items = arr.borrow();
                                if idx < items.len() {
                                    self.stack.push(items[idx].clone());
                                } else {
                                    return Err(format!(
                                        "HATA: Dizi sınırları dışında: indeks {}, boyut {}",
                                        idx,
                                        items.len()
                                    ));
                                }
                            }
                            _ => return Err("HATA: Dizi indeksi sayı olmalıdır".to_string()),
                        },
                        Val::Map(map) => match index_val {
                            Val::String(s) => {
                                let items = map.borrow();
                                if let Some(v) = items.get(&s) {
                                    self.stack.push(v.clone());
                                } else {
                                    self.stack.push(Val::Bos);
                                }
                            }
                            _ => {
                                return Err(
                                    "HATA: Harita indeksi metin olmak zorundadır".to_string()
                                )
                            }
                        },
                        _ => {
                            return Err(
                                "HATA: Sadece diziler ve haritalar indekslenebilir".to_string()
                            )
                        }
                    }
                }
                Instruction::IndexStore => {
                    let val = self.stack.pop().ok_or("HATA: Yığın boş (IndexStore val)")?;
                    let index_val = self.stack.pop().ok_or("HATA: Yığın boş (IndexStore idx)")?;
                    let target_val = self
                        .stack
                        .pop()
                        .ok_or("HATA: Yığın boş (IndexStore target)")?;
                    match target_val {
                        Val::Array(arr) => match index_val {
                            Val::Number(n) => {
                                let idx = n as usize;
                                let mut items = arr.borrow_mut();
                                if idx < items.len() {
                                    items[idx] = val;
                                } else {
                                    return Err(format!("HATA: Dizi sınırları dışında güncelleme: indeks {}, boyut {}", idx, items.len()));
                                }
                            }
                            _ => return Err("HATA: Dizi indeksi sayı olmalıdır".to_string()),
                        },
                        Val::Map(map) => match index_val {
                            Val::String(s) => {
                                map.borrow_mut().insert(s, val);
                            }
                            _ => {
                                return Err(
                                    "HATA: Harita indeksi metin olmak zorundadır".to_string()
                                )
                            }
                        },
                        _ => {
                            return Err(
                                "HATA: Sadece diziler ve haritalar güncellenebilir".to_string()
                            )
                        }
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
                Instruction::AwaitTask => {
                    let task_val = self.stack.pop().ok_or("HATA: Yığın boş (AwaitTask)")?;
                    match task_val {
                        Val::Task(task_cell) => {
                            let mut task = task_cell.borrow_mut();
                            if !task.completed {
                                match &task.func {
                                    Val::Function {
                                        name: _,
                                        param_count: _,
                                        entry_ip,
                                    } => {
                                        let mut sub_vm = VM::new(self.instructions.clone());
                                        sub_vm.globals = self.globals.clone();
                                        for arg in &task.args {
                                            sub_vm.stack.push(arg.clone());
                                        }
                                        sub_vm.frames.push(Frame {
                                            return_address: self.instructions.len(),
                                            slots: Vec::new(),
                                        });
                                        sub_vm.ip = *entry_ip;
                                        sub_vm.run()?;
                                        let res = sub_vm.stack.pop().unwrap_or(Val::Bos);
                                        self.globals = sub_vm.globals;
                                        task.result = res;
                                        task.completed = true;
                                    }
                                    Val::Builtin(name) => {
                                        let result = if name == "yazdır" {
                                            for arg in &task.args {
                                                print!("{:?} ", arg);
                                            }
                                            println!();
                                            Val::Bos
                                        } else if name == "boyut" {
                                            if task.args.len() == 1 {
                                                match &task.args[0] {
                                                    Val::Array(arr) => {
                                                        Val::Number(arr.borrow().len() as f64)
                                                    }
                                                    Val::Map(map) => {
                                                        Val::Number(map.borrow().len() as f64)
                                                    }
                                                    _ => Val::Number(0.0),
                                                }
                                            } else {
                                                Val::Number(0.0)
                                            }
                                        } else if name == "ekle" {
                                            if task.args.len() == 2 {
                                                if let Val::Array(arr) = &task.args[0] {
                                                    arr.borrow_mut().push(task.args[1].clone());
                                                }
                                            }
                                            Val::Bos
                                        } else if name == "hata_fırlat" {
                                            let msg = if task.args.len() >= 1 {
                                                match &task.args[0] {
                                                    Val::String(s) => s.clone(),
                                                    _ => format!("{:?}", task.args[0]),
                                                }
                                            } else {
                                                "Bilinmeyen hata".to_string()
                                            };
                                            Val::Hata(msg)
                                        } else if name == "dosya_oku" {
                                            if task.args.len() == 1 {
                                                if let Val::String(path) = &task.args[0] {
                                                    match std::fs::read_to_string(path) {
                                                        Ok(content) => Val::String(content),
                                                        Err(e) => Val::Hata(format!(
                                                            "Dosya okunamadı: {}",
                                                            e
                                                        )),
                                                    }
                                                } else {
                                                    Val::Hata(
                                                        "Geçersiz argüman: dosya_oku(yol)"
                                                            .to_string(),
                                                    )
                                                }
                                            } else {
                                                Val::Hata(
                                                    "dosya_oku() tek bir parametre alır"
                                                        .to_string(),
                                                )
                                            }
                                        } else if name == "dosya_yaz" {
                                            if task.args.len() == 2 {
                                                if let (Val::String(path), Val::String(content)) =
                                                    (&task.args[0], &task.args[1])
                                                {
                                                    match std::fs::write(path, content) {
                                                        Ok(_) => Val::Boolean(true),
                                                        Err(e) => Val::Hata(format!(
                                                            "Dosya yazılamadı: {}",
                                                            e
                                                        )),
                                                    }
                                                } else {
                                                    Val::Hata(
                                                        "Geçersiz argüman: dosya_yaz(yol, içerik)"
                                                            .to_string(),
                                                    )
                                                }
                                            } else {
                                                Val::Hata(
                                                    "dosya_yaz() iki parametre alır".to_string(),
                                                )
                                            }
                                        } else if name == "dosya_sil" {
                                            if task.args.len() == 1 {
                                                if let Val::String(path) = &task.args[0] {
                                                    match std::fs::remove_file(path) {
                                                        Ok(_) => Val::Boolean(true),
                                                        Err(e) => Val::Hata(format!(
                                                            "Dosya silinemedi: {}",
                                                            e
                                                        )),
                                                    }
                                                } else {
                                                    Val::Hata(
                                                        "Geçersiz argüman: dosya_sil(yol)"
                                                            .to_string(),
                                                    )
                                                }
                                            } else {
                                                Val::Hata(
                                                    "dosya_sil() tek bir parametre alır"
                                                        .to_string(),
                                                )
                                            }
                                        } else if name == "kök" {
                                            if task.args.len() == 1 {
                                                if let &Val::Number(n) = &task.args[0] {
                                                    if n >= 0.0 {
                                                        Val::Number(n.sqrt())
                                                    } else {
                                                        Val::Hata(
                                                            "Negatif sayının karekökü alınamaz"
                                                                .to_string(),
                                                        )
                                                    }
                                                } else {
                                                    Val::Hata(
                                                        "Geçersiz argüman: kök(sayı)".to_string(),
                                                    )
                                                }
                                            } else {
                                                Val::Hata(
                                                    "kök() tek bir parametre alır".to_string(),
                                                )
                                            }
                                        } else if name == "üs" {
                                            if task.args.len() == 2 {
                                                if let (
                                                    &Val::Number(base),
                                                    &Val::Number(exponent),
                                                ) = (&task.args[0], &task.args[1])
                                                {
                                                    Val::Number(base.powf(exponent))
                                                } else {
                                                    Val::Hata(
                                                        "Geçersiz argüman: üs(taban, üs)"
                                                            .to_string(),
                                                    )
                                                }
                                            } else {
                                                Val::Hata("üs() iki parametre alır".to_string())
                                            }
                                        } else if name == "mutlak" {
                                            if task.args.len() == 1 {
                                                if let &Val::Number(n) = &task.args[0] {
                                                    Val::Number(n.abs())
                                                } else {
                                                    Val::Hata(
                                                        "Geçersiz argüman: mutlak(sayı)"
                                                            .to_string(),
                                                    )
                                                }
                                            } else {
                                                Val::Hata(
                                                    "mutlak() tek bir parametre alır".to_string(),
                                                )
                                            }
                                        } else if name == "şimdi" {
                                            if task.args.is_empty() {
                                                let now = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default();
                                                Val::Number(now.as_secs_f64())
                                            } else {
                                                Val::Hata("şimdi() parametre almaz".to_string())
                                            }
                                        } else if name == "uyku" {
                                            if task.args.len() == 1 {
                                                if let &Val::Number(ms) = &task.args[0] {
                                                    std::thread::sleep(
                                                        std::time::Duration::from_millis(ms as u64),
                                                    );
                                                    Val::Bos
                                                } else {
                                                    Val::Hata(
                                                        "Geçersiz argüman: uyku(milisaniye)"
                                                            .to_string(),
                                                    )
                                                }
                                            } else {
                                                Val::Hata(
                                                    "uyku() tek bir parametre alır".to_string(),
                                                )
                                            }
                                        } else {
                                            Val::Bos
                                        };
                                        task.result = result;
                                        task.completed = true;
                                    }
                                    _ => {
                                        return Err("HATA: Görev çağrılabilir bir işlev içermiyor"
                                            .to_string())
                                    }
                                }
                            }
                            self.stack.push(task.result.clone());
                        }
                        other => {
                            self.stack.push(other);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn format_val(val: &Val) -> String {
    match val {
        Val::Number(n) => format!("{}", n),
        Val::String(s) => s.clone(),
        Val::Boolean(b) => (if *b { "doğru" } else { "yanlış" }).to_string(),
        Val::Bos => "boş".to_string(),
        Val::Array(arr) => {
            let items = arr.borrow();
            let mut s = "[".to_string();
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    s.push_str(", ");
                }
                s.push_str(&format_val(item));
            }
            s.push(']');
            s
        }
        Val::Map(map) => {
            let items = map.borrow();
            let mut s = "{".to_string();
            for (idx, (k, v)) in items.iter().enumerate() {
                if idx > 0 {
                    s.push_str(", ");
                }
                s.push_str(&format!("{}: {}", k, format_val(v)));
            }
            s.push('}');
            s
        }
        _ => format!("{:?}", val),
    }
}
