pub mod instruction;
pub mod compiler;
pub mod vm;

use instruction::Val;

pub fn run_bytecode(src: &str) -> Result<(Option<Val>, vm::VM), String> {
    use oz_lexer::Token;
    use logos::Logos;

    let lexer = Token::lexer(src);
    let mut tokens = Vec::new();
    for (token_res, span) in lexer.spanned() {
        match token_res {
            Ok(token) => tokens.push((token, span)),
            Err(_) => return Err(format!("Lexer hatası: {:?}", span)),
        }
    }

    let ast = oz_parser::parse_tokens(tokens, src.len()).map_err(|e| format!("{:?}", e))?;
    let compiler = compiler::Compiler::new();
    let insts = compiler.compile_program(&ast)?;

    let mut vm = vm::VM::new(insts);
    vm.run()?;
    Ok((None, vm))
}

#[cfg(test)]
mod tests {
    use super::*;
    use instruction::Val;

    #[test]
    fn test_kosul() {
        let src = include_str!("../../examples/ornek1_kosul.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("sayı"), Some(Val::Number(8.0)));
    }

    #[test]
    fn test_dongu() {
        let src = include_str!("../../examples/ornek2_dongu.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("sayaç"), Some(Val::Number(4.0)));
    }

    #[test]
    fn test_islev() {
        let src = include_str!("../../examples/ornek3_islev.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("sonuç"), Some(Val::Number(30.0)));
    }

    #[test]
    fn test_hesap() {
        let src = include_str!("../../examples/ornek4_hesap.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("toplam"), Some(Val::Number(19.0)));
        assert_eq!(vm.get_global("kalan"), Some(Val::Number(3.0)));
    }

    #[test]
    fn test_karma() {
        let src = include_str!("../../examples/ornek5_karma.oz");
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("limit"), Some(Val::Number(5.0)));
    }

    #[test]
    fn test_diziler() {
        let src = r#"
            dizi = [10, 20, 30];
            ekle(dizi, 40);
            birinci = dizi[0];
            ikinci = dizi'nin 1'inci elemanı;
            eleman_sayisi = boyut(dizi);
        "#;
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("birinci"), Some(Val::Number(10.0)));
        assert_eq!(vm.get_global("ikinci"), Some(Val::Number(20.0)));
        assert_eq!(vm.get_global("eleman_sayisi"), Some(Val::Number(4.0)));
    }

    #[test]
    fn test_hata_ise() {
        let src = r#"
            işlev test_hata(hata_var) {
                hata_var ise {
                    res = hata_fırlat("baglanti koptu") hata_ise {
                        döndür 500;
                    };
                    döndür res;
                } değilse {
                    res = 100 hata_ise {
                        döndür 0;
                    };
                    döndür res;
                }
            }
            sonuc_basarili = test_hata(yanlış);
            sonuc_hatali = test_hata(doğru);
        "#;
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("sonuc_basarili"), Some(Val::Number(100.0)));
        assert_eq!(vm.get_global("sonuc_hatali"), Some(Val::Number(500.0)));
    }

    #[test]
    fn test_dosya_io() {
        let src = r#"
            işlev test_dosya() {
                yazildi = dosya_yaz("test_cikti.txt", "Tilk Dosya Sistemi");
                hata_icerik = "ok";
                icerik = dosya_oku("test_cikti.txt") hata_ise {
                    hata_icerik = "hata";
                };
                silindi = dosya_sil("test_cikti.txt");
                hata_var = "ok";
                hata_mesaji_var = "";
                temp = dosya_oku("olmayan_dosya.txt") hata_ise {
                    hata_var = "yakalandi";
                    hata_mesaji_var = hata_mesajı;
                };
                döndür [yazildi, icerik, silindi, hata_var, hata_icerik, hata_mesaji_var];
            }
            sonuclar = test_dosya();
            yazildi_res = sonuclar[0];
            icerik_res = sonuclar[1];
            silindi_res = sonuclar[2];
            hata_res = sonuclar[3];
            hata_icerik_res = sonuclar[4];
            hata_mesaji_res = sonuclar[5];
        "#;
        let res = run_bytecode(src);
        assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
        let (_, vm) = res.unwrap();
        assert_eq!(vm.get_global("yazildi_res"), Some(Val::Boolean(true)));
        assert_eq!(vm.get_global("icerik_res"), Some(Val::String("Tilk Dosya Sistemi".to_string())));
        assert_eq!(vm.get_global("silindi_res"), Some(Val::Boolean(true)));
        assert_eq!(vm.get_global("hata_res"), Some(Val::String("yakalandi".to_string())));
        assert_eq!(vm.get_global("hata_icerik_res"), Some(Val::String("ok".to_string())));
        
        let msg = vm.get_global("hata_mesaji_res").unwrap();
        if let Val::String(s) = msg {
            assert!(s.contains("okunamadı") || s.contains("okunamadi") || s.contains("bulunamadı") || s.contains("bulunamadi"));
        } else {
            panic!("Hata mesajı string olmalı!");
        }
    }
}
