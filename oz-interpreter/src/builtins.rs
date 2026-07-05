use super::val::{Env, TaskState, Val};
use std::cell::RefCell;
use std::rc::Rc;

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
    // Built-in function "boyut" (returns length of array or map)
    env.set(
        "boyut".to_string(),
        Val::Builtin(Rc::new(|args| {
            if args.len() == 1 {
                match &args[0] {
                    Val::Array(arr) => return Val::Number(arr.borrow().len() as f64),
                    Val::Map(map) => return Val::Number(map.borrow().len() as f64),
                    _ => {}
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

    // Built-in function "kanal"
    env.set(
        "kanal".to_string(),
        Val::Builtin(Rc::new(|_args| {
            Val::Channel(Rc::new(RefCell::new(std::collections::VecDeque::new())))
        })),
    );

    let format_val_int = |val: &Val| -> String {
        match val {
            Val::Number(n) => n.to_string(),
            Val::String(s) => s.clone(),
            Val::Boolean(b) => if *b { "doğru".to_string() } else { "yanlış".to_string() },
            Val::Bos => "boş".to_string(),
            _ => format!("{:?}", val),
        }
    };

    env.set("biçimle".to_string(), Val::Builtin(Rc::new(move |args| {
        if args.len() < 1 { return Val::Hata("biçimle() en az 1 parametre alır".to_string()); }
        if let Val::String(template) = &args[0] {
            let mut result = String::new();
            let mut arg_idx = 1;
            let mut chars = template.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '{' && chars.peek() == Some(&'}') {
                    chars.next();
                    if arg_idx < args.len() {
                        result.push_str(&format_val_int(&args[arg_idx]));
                        arg_idx += 1;
                    } else {
                        result.push_str("{}");
                    }
                } else {
                    result.push(c);
                }
            }
            return Val::String(result);
        }
        Val::Hata("İlk argüman metin olmalı".to_string())
    })));

    env.set("uzunluk".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 1 {
            if let Val::String(s) = &args[0] {
                return Val::Number(s.chars().count() as f64);
            }
        }
        Val::Hata("Geçersiz argüman: uzunluk(metin)".to_string())
    })));

    env.set("böl".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 2 {
            if let (Val::String(metin), Val::String(ayirici)) = (&args[0], &args[1]) {
                let parts: Vec<Val> = metin.split(ayirici).map(|s| Val::String(s.to_string())).collect();
                return Val::Array(Rc::new(RefCell::new(parts)));
            }
        }
        Val::Hata("Geçersiz argüman: böl(metin, ayraç)".to_string())
    })));

    env.set("birleştir".to_string(), Val::Builtin(Rc::new(move |args| {
        if args.len() == 2 {
            if let (Val::Array(arr), Val::String(ayirici)) = (&args[0], &args[1]) {
                let format_val_int_inner = |val: &Val| -> String {
                    match val {
                        Val::Number(n) => n.to_string(),
                        Val::String(s) => s.clone(),
                        Val::Boolean(b) => if *b { "doğru".to_string() } else { "yanlış".to_string() },
                        Val::Bos => "boş".to_string(),
                        _ => format!("{:?}", val),
                    }
                };
                let strings: Vec<String> = arr.borrow().iter().map(format_val_int_inner).collect();
                return Val::String(strings.join(ayirici));
            }
        }
        Val::Hata("Geçersiz argüman: birleştir(dizi, ayraç)".to_string())
    })));

    env.set("içerir".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 2 {
            if let (Val::String(metin), Val::String(aranan)) = (&args[0], &args[1]) {
                return Val::Boolean(metin.contains(aranan));
            }
        }
        Val::Hata("Geçersiz argüman: içerir(metin, aranan)".to_string())
    })));

    env.set("büyük_harf".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 1 { if let Val::String(s) = &args[0] { return Val::String(s.to_uppercase()); } }
        Val::Hata("Hata".into())
    })));

    env.set("küçük_harf".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 1 { if let Val::String(s) = &args[0] { return Val::String(s.to_lowercase()); } }
        Val::Hata("Hata".into())
    })));

    env.set("kırp".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 1 { if let Val::String(s) = &args[0] { return Val::String(s.trim().to_string()); } }
        Val::Hata("Hata".into())
    })));

    env.set("tamsayı".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 1 { if let Val::Number(n) = &args[0] { return Val::Number(n.floor()); } }
        Val::Hata("Hata".into())
    })));

    env.set("metne_çevir".to_string(), Val::Builtin(Rc::new(move |args| {
        if args.len() == 1 {
            let format_val_int_inner = |val: &Val| -> String {
                match val {
                    Val::Number(n) => n.to_string(),
                    Val::String(s) => s.clone(),
                    Val::Boolean(b) => if *b { "doğru".to_string() } else { "yanlış".to_string() },
                    Val::Bos => "boş".to_string(),
                    _ => format!("{:?}", val),
                }
            };
            return Val::String(format_val_int_inner(&args[0]));
        }
        Val::Hata("Hata".into())
    })));

    env.set("sayıya_çevir".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 1 {
            if let Val::String(s) = &args[0] {
                if let Ok(n) = s.parse::<f64>() { return Val::Number(n); }
            }
        }
        Val::Hata("Hata".into())
    })));

    env.set("rastgele".to_string(), Val::Builtin(Rc::new(|args| {
        if args.len() == 2 {
            if let (Val::Number(min), Val::Number(max)) = (&args[0], &args[1]) {
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
                let range = max - min;
                if range > 0.0 {
                    let rnd = (now % 10000) as f64 / 10000.0;
                    return Val::Number((min + (rnd * range)).floor());
                }
                return Val::Number(*min);
            }
        }
        Val::Hata("Hata".into())
    })));

    env

}
