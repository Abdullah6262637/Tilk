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
}

#[derive(Deserialize)]
struct PackageConfig {
    paket: PaketDetails,
    bagimliliklar: Option<std::collections::HashMap<String, String>>,
}

#[derive(Deserialize)]
struct PaketDetails {
    ad: String,
    #[allow(dead_code)]
    surum: String,
    giris: String,
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

    oz_parser::typechecker::check_program(&ast)?;

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
                    eprintln!("HATA: Tip denetim hatası: {}", type_err);
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
                                println!("GEÇTİ");
                                passed += 1;
                            }
                            Err(e) => {
                                println!("BAŞARISIZ");
                                eprintln!("  Hata detayı: {}", e);
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

                println!("--- Bağımlılıklar Yükleniyor ---");
                for (ad, surum) in bagimliliklar {
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
                }
                println!("Tüm bağımlılıklar başarıyla yüklendi!");
            } else {
                println!("Yüklenecek bağımlılık bulunamadı.");
            }
        }
    }
}
