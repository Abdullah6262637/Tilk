use clap::{Parser as ClapParser, Subcommand};
use logos::Logos;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "oz")]
#[command(about = "ÖZGÜN Dili Araç Zinciri CLI", long_about = None)]
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
    Derle,
    /// Projedeki testleri (testler/ altındaki dosyaları) çalıştırır.
    Test,
}

#[derive(Deserialize)]
struct PackageConfig {
    paket: PaketDetails,
}

#[derive(Deserialize)]
struct PaketDetails {
    ad: String,
    #[allow(dead_code)]
    surum: String,
    giris: String,
}

fn run_file(file: &PathBuf) -> Result<(), String> {
    let content = fs::read_to_string(file)
        .map_err(|e| format!("Dosya okunamadı {}: {}", file.display(), e))?;

    let lexer = oz_lexer::Token::lexer(&content);
    let mut tokens = Vec::new();
    for (token_res, span) in lexer.spanned() {
        match token_res {
            Ok(token) => tokens.push((token, span)),
            Err(_) => return Err(format!("Sözcüksel analiz hatası: Tanımlanamayan karakter at {:?}", span)),
        }
    }

    let len = content.len();
    let ast = oz_parser::parse_tokens(tokens, len)
        .map_err(|errors| format!("Sözdizimi ayrıştırma hataları: {:?}", errors))?;

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
            for (token_res, span) in lexer.spanned() {
                match token_res {
                    Ok(token) => tokens.push((token, span)),
                    Err(_) => {
                        eprintln!("Lexer hatası: Tanımlanamayan karakter at {:?}", span);
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
                    eprintln!("Sözdizimi ayrıştırma hataları:");
                    for err in errors {
                        eprintln!("{:?}", err);
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Calistir { file } => {
            let target_file = match file {
                Some(f) => f,
                None => {
                    // Look for ozgun.toml
                    let toml_path = PathBuf::from("ozgun.toml");
                    if toml_path.exists() {
                        let toml_str = match fs::read_to_string(&toml_path) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("ozgun.toml okunamadı: {}", e);
                                std::process::exit(1);
                            }
                        };
                        let config: PackageConfig = match toml::from_str(&toml_str) {
                            Ok(c) => c,
                            Err(e) => {
                                eprintln!("ozgun.toml ayrıştırılamadı: {}", e);
                                std::process::exit(1);
                            }
                        };
                        PathBuf::from(config.paket.giris)
                    } else {
                        eprintln!("HATA: Çalıştırılacak dosya belirtilmedi ve ozgun.toml bulunamadı.");
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

            // Create ozgun.toml
            let toml_content = format!(
                r#"[paket]
ad = "{}"
surum = "0.1.0"
giris = "kaynak/ana.oz"
"#,
                isim
            );
            fs::write(root_dir.join("ozgun.toml"), toml_content).unwrap();

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

            println!("Başarılı: '{}' adında yeni bir ÖZGÜN projesi oluşturuldu.", isim);
        }
        Commands::Derle => {
            let toml_path = PathBuf::from("ozgun.toml");
            if !toml_path.exists() {
                eprintln!("HATA: ozgun.toml bulunamadı. ÖZGÜN projesi dizininde değilsiniz.");
                std::process::exit(1);
            }

            let toml_str = fs::read_to_string(&toml_path).unwrap();
            let config: PackageConfig = toml::from_str(&toml_str).unwrap();
            let entry_file = PathBuf::from(config.paket.giris);

            println!("Derleniyor: {}...", config.paket.ad);
            match run_file(&entry_file) {
                Ok(_) => println!("Derleme başarılı! Herhangi bir sözdizimi hatası bulunamadı."),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Test => {
            let toml_path = PathBuf::from("ozgun.toml");
            if !toml_path.exists() {
                eprintln!("HATA: ozgun.toml bulunamadı. ÖZGÜN projesi dizininde değilsiniz.");
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
    }
}
