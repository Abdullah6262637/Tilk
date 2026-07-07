#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::manual_flatten)]
#![allow(clippy::unnecessary_map_or)]
mod c_codegen;

use clap::{Parser as ClapParser, Subcommand};
use logos::Logos;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "tilk")]
#[command(about = "TİLK Dili Araç Zinciri CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// .oz dosyasını sözcüksel analizden geçirip token listesini yazdırır.
    Lex {
        /// Hedef dosya yolu
        file: PathBuf,
    },
    /// TİLK etkileşimli kabuğunu (REPL) başlatır.
    Repl,
    /// .oz dosyasını ayrıştırıp AST (Soyut Sözdizimi Ağacı) yapısını yazdırır.
    Parse {
        /// Hedef dosya yolu
        file: PathBuf,
    },
    /// .oz dosyasını veya mevcut dizindeki projeyi çalıştırır.
    Calistir {
        /// Hedef dosya yolu (isteğe bağlı)
        file: Option<PathBuf>,
    },
    /// Yeni bir ÖZGÜN proje dizini oluşturur.
    Yeni {
        /// Proje adı
        isim: String,
    },
    /// Mevcut projedeki kaynak dosyalarını derler.
    Derle {
        /// Yerel makine koduna (C / LLVM AOT) derler
        #[arg(long, short)]
        yerel: bool,
    },
    /// Projedeki testleri (testler/ altındaki dosyaları) çalıştırır.
    Test,
    /// Projedeki bağımlılıkları kitaplık/ altına indirir.
    Yukle,
    /// .oz dosyasını formatlar (pretty-print).
    Fmt {
        /// Hedef dosya yolu
        file: PathBuf,
        /// Dosyayı yerinde düzenler (üzerine yazar)
        #[arg(long)]
        in_place: bool,
    },
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Dependency {
    Version(String),
    Complex {
        git: Option<String>,
        tag: Option<String>,
        path: Option<String>,
    },
}

#[derive(Deserialize)]
struct PackageConfig {
    paket: PaketDetails,
    bagimliliklar: Option<std::collections::HashMap<String, Dependency>>,
}

#[derive(Deserialize)]
struct PaketDetails {
    ad: String,
    #[allow(dead_code)]
    surum: String,
    giris: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LockDependency {
    surum: Option<String>,
    kaynak: String,
    checksum: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LockFile {
    versiyon: u32,
    bagimliliklar: std::collections::HashMap<String, LockDependency>,
}

fn print_parser_errors(
    errors: Vec<chumsky::error::Simple<oz_lexer::Token>>,
    file_name: &str,
    source: &str,
) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    for err in errors {
        let span = err.span();
        let msg = match err.reason() {
            chumsky::error::SimpleReason::Unexpected => {
                let expected: Vec<String> = err
                    .expected()
                    .map(|tok| match tok {
                        Some(t) => format!("{:?}", t),
                        None => "dosya sonu".to_string(),
                    })
                    .collect();
                if expected.is_empty() {
                    "Beklenmedik sözcük".to_string()
                } else {
                    format!("Beklenen sözcükler: {}", expected.join(", "))
                }
            }
            chumsky::error::SimpleReason::Custom(s) => s.clone(),
            _ => "Ayrıştırma hatası".to_string(),
        };

        let label = if let Some(found) = err.found() {
            format!("Hatalı sözcük: {:?}", found)
        } else {
            "Dosya sonu".to_string()
        };

        Report::build(ReportKind::Error, file_name, span.start)
            .with_message(msg)
            .with_label(
                Label::new((file_name, span))
                    .with_message(label)
                    .with_color(Color::Red),
            )
            .finish()
            .eprint((file_name, Source::from(source)))
            .unwrap();
    }
}

fn print_type_error(err: oz_parser::typechecker::types::TypeError, file_name: &str, source: &str) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    let span = err.span.unwrap_or(0..0);

    let mut builder =
        Report::build(ReportKind::Error, file_name, span.start).with_message(&err.message);

    if span.start != span.end {
        builder = builder.with_label(
            Label::new((file_name, span))
                .with_message("Bu ifade hatalı")
                .with_color(Color::Yellow),
        );
    }

    if let Some(expected) = err.expected {
        builder = builder.with_note(format!("Beklenen tip: {:?}", expected));
    }
    if let Some(found) = err.found {
        builder = builder.with_note(format!("Bulunan tip: {:?}", found));
    }

    builder
        .finish()
        .eprint((file_name, Source::from(source)))
        .unwrap();
}

fn run_file(file: &PathBuf) -> Result<(), String> {
    let content = fs::read_to_string(file)
        .map_err(|e| format!("Dosya okunamadı {}: {}", file.display(), e))?;

    let lexer = oz_lexer::Token::lexer(&content);
    let mut tokens = Vec::new();
    let file_name = file.to_string_lossy().to_string();
    for (token_res, span) in lexer.spanned() {
        match token_res {
            Ok(token) => tokens.push((token, span)),
            Err(_) => {
                use ariadne::{Color, Label, Report, ReportKind, Source};
                Report::build(ReportKind::Error, file_name.clone(), span.start)
                    .with_message("Sözcüksel analiz hatası: Tanımlanamayan karakter")
                    .with_label(
                        Label::new((file_name.clone(), span))
                            .with_message("Geçersiz karakter")
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint((file_name.clone(), Source::from(&content)))
                    .unwrap();
                return Err("Sözcüksel analiz hatası".to_string());
            }
        }
    }

    let len = content.len();
    let ast = match oz_parser::parse_tokens(tokens, len) {
        Ok(ast) => ast,
        Err(errors) => {
            print_parser_errors(errors, &file_name, &content);
            return Err("Sözdizimi ayrıştırma hatası".to_string());
        }
    };

    if let Err(type_err) = oz_parser::typechecker::check_program(&ast) {
        print_type_error(type_err, &file_name, &content);
        return Err("Tip denetimi hatası".to_string());
    }

    let compiler = oz_vm::compiler::Compiler::new();
    let insts = compiler.compile_program(&ast)?;

    let mut vm = oz_vm::vm::VM::new(insts);
    vm.run()?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lex { file } => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Dosya okunamadı: {}", e);
                    std::process::exit(1);
                }
            };

            let lexer = oz_lexer::Token::lexer(&content);
            println!("--- Sözcüksel Analiz (Tokens) ---");
            for (token_res, span) in lexer.spanned() {
                match token_res {
                    Ok(token) => println!("{:?} at {:?}", token, span),
                    Err(_) => eprintln!("HATA: Tanımlanamayan karakter at {:?}", span),
                }
            }
        }
        Commands::Repl => {
            println!("TİLK Etkileşimli Kabuk (REPL) - Çıkmak için 'cikis' yazın veya Ctrl+C / Ctrl+D tuşlarına basın.");
            let mut rl = rustyline::DefaultEditor::new().unwrap();

            let mut tc = oz_parser::typechecker::TypeChecker::new();
            let mut env = oz_parser::typechecker::create_default_type_env(&mut tc);
            let mut vm = oz_vm::vm::VM::new(vec![]);

            loop {
                let compiler = oz_vm::compiler::Compiler::new();
                let readline = rl.readline(">> ");
                match readline {
                    Ok(line) => {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        if line == "cikis" || line == "çıkış" {
                            break;
                        }

                        let _ = rl.add_history_entry(line);

                        let lexer = oz_lexer::Token::lexer(line);
                        let mut tokens = Vec::new();
                        let mut lex_error = false;
                        for (token_res, span) in lexer.spanned() {
                            match token_res {
                                Ok(token) => tokens.push((token, span)),
                                Err(_) => {
                                    eprintln!("HATA: Sözcüksel analiz hatası at {:?}", span);
                                    lex_error = true;
                                    break;
                                }
                            }
                        }
                        if lex_error {
                            continue;
                        }

                        let ast = match oz_parser::parse_tokens(tokens, line.len()) {
                            Ok(ast) => ast,
                            Err(errors) => {
                                print_parser_errors(errors, "<repl>", line);
                                continue;
                            }
                        };

                        let mut type_err = false;
                        for stmt in &ast {
                            if let Err(err) = tc.infer_stmt(stmt, &mut env, &None) {
                                print_type_error(err, "<repl>", line);
                                type_err = true;
                                break;
                            }
                        }
                        if type_err {
                            continue;
                        }

                        let insts = match compiler.compile_program(&ast) {
                            Ok(insts) => insts,
                            Err(e) => {
                                eprintln!("Derleme hatası: {}", e);
                                continue;
                            }
                        };

                        if let Err(e) = vm.run_instructions(insts) {
                            eprintln!("Çalışma zamanı hatası: {}", e);
                        }
                    }
                    Err(rustyline::error::ReadlineError::Interrupted)
                    | Err(rustyline::error::ReadlineError::Eof) => {
                        break;
                    }
                    Err(err) => {
                        println!("Hata: {:?}", err);
                        break;
                    }
                }
            }
        }
        Commands::Parse { file } => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Dosya okunamadı: {}", e);
                    std::process::exit(1);
                }
            };

            let lexer = oz_lexer::Token::lexer(&content);
            let mut tokens = Vec::new();
            let file_name = file.to_string_lossy().to_string();
            for (token_res, span) in lexer.spanned() {
                match token_res {
                    Ok(token) => tokens.push((token, span)),
                    Err(_) => {
                        use ariadne::{Color, Label, Report, ReportKind, Source};
                        Report::build(ReportKind::Error, file_name.clone(), span.start)
                            .with_message("Sözcüksel analiz hatası: Tanımlanamayan karakter")
                            .with_label(
                                Label::new((file_name.clone(), span))
                                    .with_message("Geçersiz karakter")
                                    .with_color(Color::Red),
                            )
                            .finish()
                            .eprint((file_name.clone(), Source::from(&content)))
                            .unwrap();
                        std::process::exit(1);
                    }
                }
            }

            let len = content.len();
            match oz_parser::parse_tokens(tokens, len) {
                Ok(ast) => {
                    println!("--- Soyut Sözdizimi Ağacı (AST) ---");
                    println!("{:#?}", ast);
                }
                Err(errors) => {
                    print_parser_errors(errors, &file_name, &content);
                    std::process::exit(1);
                }
            }
        }
        Commands::Calistir { file } => {
            let target_file = match file {
                Some(f) => f,
                None => {
                    // Look for tilk.toml
                    let toml_path = PathBuf::from("tilk.toml");
                    if toml_path.exists() {
                        let toml_str = match fs::read_to_string(&toml_path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("tilk.toml okunamadı: {}", e);
                                std::process::exit(1);
                            }
                        };
                        let config: PackageConfig = match toml::from_str(&toml_str) {
                            Ok(c) => c,
                            Err(e) => {
                                eprintln!("tilk.toml ayrıştırılamadı: {}", e);
                                std::process::exit(1);
                            }
                        };
                        PathBuf::from(config.paket.giris)
                    } else {
                        eprintln!(
                            "HATA: Çalıştırılacak dosya belirtilmedi ve tilk.toml bulunamadı."
                        );
                        std::process::exit(1);
                    }
                }
            };

            match run_file(&target_file) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Yeni { isim } => {
            let root_dir = PathBuf::from(&isim);
            if root_dir.exists() {
                eprintln!("HATA: '{}' dizini zaten mevcut.", isim);
                std::process::exit(1);
            }

            let kaynak_dir = root_dir.join("kaynak");
            let testler_dir = root_dir.join("testler");

            fs::create_dir_all(&kaynak_dir).unwrap();
            fs::create_dir_all(&testler_dir).unwrap();

            // Create tilk.toml
            let toml_content = format!(
                r#"[paket]
ad = "{}"
surum = "0.1.0"
giris = "kaynak/ana.oz"
"#,
                isim
            );
            fs::write(root_dir.join("tilk.toml"), toml_content).unwrap();

            // Create kaynak/ana.oz
            let main_content = r#"yazdır("Merhaba Dünya!");
"#;
            fs::write(kaynak_dir.join("ana.oz"), main_content).unwrap();

            // Create testler/test_ana.oz
            let test_content = r#"doğru ise {
    yazdır("Test geçti");
}
"#;
            fs::write(testler_dir.join("test_ana.oz"), test_content).unwrap();

            println!(
                "Başarılı: '{}' adında yeni bir TİLK projesi oluşturuldu.",
                isim
            );
        }
        Commands::Derle { yerel } => {
            let toml_path = PathBuf::from("tilk.toml");
            if !toml_path.exists() {
                eprintln!("HATA: tilk.toml bulunamadı. TİLK projesi dizininde değilsiniz.");
                std::process::exit(1);
            }

            let toml_str = fs::read_to_string(&toml_path).unwrap();
            let config: PackageConfig = toml::from_str(&toml_str).unwrap();
            let entry_file = PathBuf::from(config.paket.giris);

            println!("Derleniyor: {}...", config.paket.ad);

            if yerel {
                let content = match fs::read_to_string(&entry_file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("HATA: Giriş dosyası okunamadı: {}", e);
                        std::process::exit(1);
                    }
                };

                let lexer = oz_lexer::Token::lexer(&content);
                let mut tokens = Vec::new();
                for (token_res, span) in lexer.spanned() {
                    match token_res {
                        Ok(token) => tokens.push((token, span)),
                        Err(_) => {
                            eprintln!("HATA: Sözcüksel analiz hatası at {:?}", span);
                            std::process::exit(1);
                        }
                    }
                }

                let ast = match oz_parser::parse_tokens(tokens, content.len()) {
                    Ok(a) => a,
                    Err(errors) => {
                        eprintln!("HATA: Sözdizimi ayrıştırma hataları: {:?}", errors);
                        std::process::exit(1);
                    }
                };

                if let Err(type_err) = oz_parser::typechecker::check_program(&ast) {
                    let file_name = entry_file.to_string_lossy().to_string();
                    print_type_error(type_err, &file_name, &content);
                    std::process::exit(1);
                }

                let codegen = c_codegen::CCodegen::new();
                let c_code = match codegen.transpile(&ast) {
                    Ok(code) => code,
                    Err(e) => {
                        eprintln!("HATA: C kod üretimi başarısız oldu: {}", e);
                        std::process::exit(1);
                    }
                };

                let c_file_path = PathBuf::from("program.c");
                fs::write(&c_file_path, c_code).unwrap();
                println!("  -> C kaynak kodu üretildi: {}", c_file_path.display());

                println!("Yerel binary'ye derleniyor...");
                let exe_name = if cfg!(target_os = "windows") {
                    "program.exe"
                } else {
                    "program"
                };

                let mut compiled = false;
                let output = std::process::Command::new("gcc")
                    .args(&["-O3", "program.c", "-o", exe_name, "-lm"])
                    .output();
                if let Ok(out) = output {
                    if out.status.success() {
                        println!(
                            "Başarılı: GCC kullanılarak yerel binary derlendi -> {}",
                            exe_name
                        );
                        compiled = true;
                    }
                }

                if !compiled {
                    let output = std::process::Command::new("clang")
                        .args(&["-O3", "program.c", "-o", exe_name, "-lm"])
                        .output();
                    if let Ok(out) = output {
                        if out.status.success() {
                            println!(
                                "Başarılı: Clang kullanılarak yerel binary derlendi -> {}",
                                exe_name
                            );
                            compiled = true;
                        }
                    }
                }

                if !compiled {
                    let output = std::process::Command::new("cl.exe")
                        .args(&["/O2", "program.c", "/Fe:", exe_name])
                        .output();
                    if let Ok(out) = output {
                        if out.status.success() {
                            println!(
                                "Başarılı: MSVC kullanılarak yerel binary derlendi -> {}",
                                exe_name
                            );
                            compiled = true;
                        }
                    }
                }

                if !compiled {
                    println!(
                        "UYARI: Sistemde yüklü bir C derleyicisi (gcc, clang, cl) bulunamadı."
                    );
                    println!("Tilk kodundan üretilen C kaynak kodunu manuel derlemek için:");
                    println!("  gcc -O3 program.c -o {} -lm", exe_name);
                }
            } else {
                match run_file(&entry_file) {
                    Ok(_) => {
                        println!("Derleme başarılı! Herhangi bir sözdizimi hatası bulunamadı.")
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Test => {
            let toml_path = PathBuf::from("tilk.toml");
            if !toml_path.exists() {
                eprintln!("HATA: tilk.toml bulunamadı. TİLK projesi dizininde değilsiniz.");
                std::process::exit(1);
            }

            let testler_dir = PathBuf::from("testler");
            if !testler_dir.exists() {
                println!("Çalıştırılacak test bulunamadı (testler/ dizini mevcut değil).");
                return;
            }

            let entries = match fs::read_dir(testler_dir) {
                Ok(e) => e,
                Err(_) => {
                    eprintln!("HATA: testler/ dizini okunamadı.");
                    std::process::exit(1);
                }
            };

            let mut passed = 0;
            let mut failed = 0;

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "oz") {
                        print!("Test çalıştırılıyor: {}... ", path.display());
                        match run_file(&path) {
                            Ok(_) => {
                                println!("\x1b[32m[GEÇTİ]\x1b[0m");
                                passed += 1;
                            }
                            Err(e) => {
                                println!("\x1b[31m[KALDI]\x1b[0m: {}", e);
                                failed += 1;
                            }
                        }
                    }
                }
            }

            println!("\nTest Sonucu: {} geçti, {} başarısız.", passed, failed);
            if failed > 0 {
                std::process::exit(1);
            }
        }
        Commands::Yukle => {
            let toml_path = PathBuf::from("tilk.toml");
            if !toml_path.exists() {
                eprintln!("HATA: tilk.toml bulunamadı. TİLK projesi dizininde değilsiniz.");
                std::process::exit(1);
            }

            let toml_str = fs::read_to_string(&toml_path).unwrap();
            let config: PackageConfig = toml::from_str(&toml_str).unwrap();

            if let Some(bagimliliklar) = config.bagimliliklar {
                let kitaplik_dir = PathBuf::from("kitaplik");
                if !kitaplik_dir.exists() {
                    fs::create_dir_all(&kitaplik_dir).unwrap();
                }

                let mut lock_deps = std::collections::HashMap::new();

                println!("--- Bağımlılıklar Yükleniyor ---");
                for (ad, dep) in bagimliliklar {
                    match dep {
                        Dependency::Version(surum) => {
                            println!("Paket deposu taranıyor: '{}' (sürüm {})...", ad, surum);
                            let code = match ad.as_str() {
                                "matematik" => {
                                    r#"
işlev topla(a, b) {
    döndür a + b;
}
işlev carp(a, b) {
    döndür a * b;
}
işlev kare(a) {
    döndür üs(a, 2);
}
"#
                                }
                                "dizi_araclari" => {
                                    r#"
işlev ilk_eleman(d) {
    döndür d[0];
}
işlev son_eleman(d) {
    döndür d[boyut(d) - 1];
}
"#
                                }
                                _ => {
                                    eprintln!("HATA: '{}' paketi depoda bulunamadı.", ad);
                                    std::process::exit(1);
                                }
                            };

                            let target_path = kitaplik_dir.join(format!("{}.oz", ad));
                            fs::write(&target_path, code).unwrap();
                            println!("  -> İndirildi ve kaydedildi: {}", target_path.display());

                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                            std::hash::Hash::hash(&code, &mut hasher);
                            let hash = std::hash::Hasher::finish(&hasher);
                            lock_deps.insert(
                                ad.clone(),
                                LockDependency {
                                    surum: Some(surum),
                                    kaynak: "kayit_defteri".to_string(),
                                    checksum: Some(format!("{:x}", hash)),
                                },
                            );
                        }
                        Dependency::Complex { git, tag, path } => {
                            if let Some(git_url) = git {
                                println!("Git deposu klonlanıyor: '{}' ({:?})...", git_url, tag);
                                let target_repo_dir = kitaplik_dir.join(&ad);
                                if target_repo_dir.exists() {
                                    let _ = fs::remove_dir_all(&target_repo_dir);
                                }
                                let mut git_cmd = std::process::Command::new("git")
                                    .args(&["clone", &git_url, target_repo_dir.to_str().unwrap()])
                                    .spawn()
                                    .map_err(|e| format!("git clone başarısız oldu: {}", e))
                                    .unwrap();

                                let status = git_cmd
                                    .wait()
                                    .map_err(|e| {
                                        format!("git clone beklenirken hata oluştu: {}", e)
                                    })
                                    .unwrap();
                                if !status.success() {
                                    eprintln!("HATA: git clone başarısız oldu.");
                                    std::process::exit(1);
                                }

                                if let Some(tag_str) = tag.clone() {
                                    let mut checkout_cmd = std::process::Command::new("git")
                                        .args(&["checkout", &tag_str])
                                        .current_dir(&target_repo_dir)
                                        .spawn()
                                        .map_err(|e| format!("git checkout başarısız oldu: {}", e))
                                        .unwrap();
                                    checkout_cmd.wait().unwrap();
                                }
                                println!(
                                    "  -> Git deposu başarıyla yüklendi: {}",
                                    target_repo_dir.display()
                                );

                                lock_deps.insert(
                                    ad.clone(),
                                    LockDependency {
                                        surum: tag,
                                        kaynak: format!("git: {}", git_url),
                                        checksum: None,
                                    },
                                );
                            } else if let Some(local_path) = path {
                                println!("Yerel kütüphane kopyalanıyor: '{}'...", local_path);
                                let src_path = PathBuf::from(local_path.clone());
                                let target_path = kitaplik_dir.join(format!("{}.oz", ad));
                                if src_path.is_file() {
                                    fs::copy(&src_path, &target_path).unwrap();
                                    println!("  -> Kopyalandı: {}", target_path.display());
                                } else {
                                    let target_dir = kitaplik_dir.join(&ad);
                                    if target_dir.exists() {
                                        let _ = fs::remove_dir_all(&target_dir);
                                    }
                                    fs::create_dir_all(&target_dir).unwrap();
                                    if let Ok(entries) = fs::read_dir(src_path) {
                                        for entry in entries.flatten() {
                                            let ep = entry.path();
                                            if ep.is_file()
                                                && ep.extension().map_or(false, |ext| ext == "oz")
                                            {
                                                fs::copy(
                                                    &ep,
                                                    target_dir.join(ep.file_name().unwrap()),
                                                )
                                                .unwrap();
                                            }
                                        }
                                    }
                                    println!("  -> Klasör kopyalandı: {}", target_dir.display());
                                }

                                lock_deps.insert(
                                    ad.clone(),
                                    LockDependency {
                                        surum: None,
                                        kaynak: format!("yerel: {}", local_path),
                                        checksum: None,
                                    },
                                );
                            }
                        }
                    }
                }

                let lock_file = LockFile {
                    versiyon: 1,
                    bagimliliklar: lock_deps,
                };
                let lock_toml = toml::to_string(&lock_file).unwrap();
                fs::write("tilk.lock", lock_toml).unwrap();
                println!("  -> tilk.lock güncellendi");

                println!("Tüm bağımlılıklar başarıyla yüklendi!");
            } else {
                println!("Yüklenecek bağımlılık bulunamadı.");
            }
        }
        Commands::Fmt { file, in_place } => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("HATA: Dosya okunamadı: {}", e);
                    std::process::exit(1);
                }
            };

            let lexer = oz_lexer::Token::lexer(&content);
            let mut tokens = Vec::new();
            for (token_res, span) in lexer.spanned() {
                match token_res {
                    Ok(token) => tokens.push((token, span)),
                    Err(_) => {
                        eprintln!("HATA: Sözcüksel analiz hatası at {:?}", span);
                        std::process::exit(1);
                    }
                }
            }

            match oz_parser::parse_tokens(tokens, content.len()) {
                Ok(ast) => {
                    let mut formatter = oz_parser::fmt::Formatter::new();
                    let formatted_code = formatter.format_program(&ast);
                    if in_place {
                        if let Err(e) = fs::write(&file, formatted_code) {
                            eprintln!("HATA: Dosya yazılamadı: {}", e);
                            std::process::exit(1);
                        }
                        println!("{} başarıyla formatlandı.", file.display());
                    } else {
                        println!("{}", formatted_code);
                    }
                }
                Err(errors) => {
                    let file_name = file.to_string_lossy().to_string();
                    eprintln!("HATA: Formatlanacak dosyada sözdizimi hataları var:");
                    print_parser_errors(errors, &file_name, &content);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_deserialization() {
        let toml_content = r#"
            [paket]
            ad = "test_proje"
            surum = "0.1.0"
            giris = "ana.oz"

            [bagimliliklar]
            matematik = "1.0.0"
            harici_git = { git = "https://github.com/user/repo.git", tag = "v1" }
            yerel_paket = { path = "./yerel" }
        "#;
        let config: PackageConfig = toml::from_str(toml_content).unwrap();
        let bagimliliklar = config.bagimliliklar.unwrap();

        assert!(matches!(
            bagimliliklar.get("matematik").unwrap(),
            Dependency::Version(_)
        ));
        if let Dependency::Complex { git, tag, path: _ } = bagimliliklar.get("harici_git").unwrap()
        {
            assert_eq!(git.as_deref(), Some("https://github.com/user/repo.git"));
            assert_eq!(tag.as_deref(), Some("v1"));
        } else {
            panic!("harici_git complex olmalı");
        }

        if let Dependency::Complex {
            git: _,
            tag: _,
            path,
        } = bagimliliklar.get("yerel_paket").unwrap()
        {
            assert_eq!(path.as_deref(), Some("./yerel"));
        } else {
            panic!("yerel_paket complex olmalı");
        }
    }
}
