use super::*;
use instruction::Val;
use std::cell::RefCell;
use std::rc::Rc;

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
    assert_eq!(
        vm.get_global("icerik_res"),
        Some(Val::String("Tilk Dosya Sistemi".to_string()))
    );
    assert_eq!(vm.get_global("silindi_res"), Some(Val::Boolean(true)));
    assert_eq!(
        vm.get_global("hata_res"),
        Some(Val::String("yakalandi".to_string()))
    );
    assert_eq!(
        vm.get_global("hata_icerik_res"),
        Some(Val::String("ok".to_string()))
    );

    let msg = vm.get_global("hata_mesaji_res").unwrap();
    if let Val::String(s) = msg {
        assert!(
            s.contains("okunamadı")
                || s.contains("okunamadi")
                || s.contains("bulunamadı")
                || s.contains("bulunamadi")
        );
    } else {
        panic!("Hata mesajı string olmalı!");
    }
}

#[test]
fn test_asenkron_tamamlaninca() {
    let src = r#"
        işlev hesapla(x, y) {
            döndür x + y;
        }
        
        gorev = arkaplanda_çalıştır(hesapla, 10, 20);
        
        yakalanan_sonuc = 0;
        gorev tamamlanınca {
            yakalanan_sonuc = sonuç;
        }
    "#;
    let res = run_bytecode(src);
    assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
    let (_, vm) = res.unwrap();
    assert_eq!(vm.get_global("yakalanan_sonuc"), Some(Val::Number(30.0)));
}

#[test]
fn test_math_time() {
    let src = r#"
        işlev test_matematik() {
            karekok_deger = kök(16);
            us_deger = üs(2, 3);
            mutlak_deger = mutlak(0 - 42);
            simdi_zaman = şimdi();
            uyku(10);
            hata_deger = 0;
            temp_hata = kök(0 - 1) hata_ise {
                hata_deger = 999;
            };
            döndür [karekok_deger, us_deger, mutlak_deger, simdi_zaman, hata_deger];
        }
        sonuclar = test_matematik();
        karekok_res = sonuclar[0];
        us_res = sonuclar[1];
        mutlak_res = sonuclar[2];
        simdi_res = sonuclar[3];
        hata_res = sonuclar[4];
    "#;
    let res = run_bytecode(src);
    assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
    let (_, vm) = res.unwrap();
    assert_eq!(vm.get_global("karekok_res"), Some(Val::Number(4.0)));
    assert_eq!(vm.get_global("us_res"), Some(Val::Number(8.0)));
    assert_eq!(vm.get_global("mutlak_res"), Some(Val::Number(42.0)));
    assert!(vm.get_global("simdi_res").is_some());
    assert_eq!(vm.get_global("hata_res"), Some(Val::Number(999.0)));
}

#[test]
fn test_haritalar_ve_mutasyon() {
    let src = r#"
        işlev test_harita() {
            haritamız = { "ad": "Tilk", "yas": 3 };
            boyut_ilk = boyut(haritamız);
            ad_deger = haritamız["ad"];
            haritamız["yas"] = 4;
            haritamız["sehir"] = "Bozkır";
            
            yas_yeni = haritamız["yas"];
            sehir_yeni = haritamız["sehir"];
            boyut_son = boyut(haritamız);
            
            dizi = [10, 20, 30];
            dizi[0] = 99;
            dizi_ilk = dizi[0];
            
            döndür [boyut_ilk, ad_deger, yas_yeni, sehir_yeni, boyut_son, dizi_ilk];
        }
        res = test_harita();
        res_boyut_ilk = res[0];
        res_ad = res[1];
        res_yas_yeni = res[2];
        res_sehir_yeni = res[3];
        res_boyut_son = res[4];
        res_dizi_ilk = res[5];
    "#;
    let res = run_bytecode(src);
    assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
    let (_, vm) = res.unwrap();
    assert_eq!(vm.get_global("res_boyut_ilk"), Some(Val::Number(2.0)));
    assert_eq!(
        vm.get_global("res_ad"),
        Some(Val::String("Tilk".to_string()))
    );
    assert_eq!(vm.get_global("res_yas_yeni"), Some(Val::Number(4.0)));
    assert_eq!(
        vm.get_global("res_sehir_yeni"),
        Some(Val::String("Bozkır".to_string()))
    );
    assert_eq!(vm.get_global("res_boyut_son"), Some(Val::Number(3.0)));
    assert_eq!(vm.get_global("res_dizi_ilk"), Some(Val::Number(99.0)));
}

#[test]
fn test_dahil_et() {
    use std::fs;
    let module_content = r#"
        işlev ek_topla(a, b) {
            döndür a + b + 10;
        }
        sabit_deger = 42;
    "#;
    let module_path = "test_modul_vm.oz";
    fs::write(module_path, module_content).unwrap();

    let src = r#"
        dahil_et("test_modul_vm.oz");
        sonuc_islev = ek_topla(5, 5);
        sonuc_sabit = sabit_deger;
    "#;
    let res = run_bytecode(src);

    let _ = fs::remove_file(module_path);

    let (_, vm) = res.unwrap();
    assert_eq!(vm.get_global("sonuc_islev"), Some(Val::Number(20.0)));
    assert_eq!(vm.get_global("sonuc_sabit"), Some(Val::Number(42.0)));
}

fn run_golden_test_file(src_path: &str, expected_stdout_path: &str) {
    let src = std::fs::read_to_string(src_path).expect("Kaynak dosya okunamadı");
    let expected =
        std::fs::read_to_string(expected_stdout_path).expect("Beklenen çıktı dosyası okunamadı");

    use logos::Logos;
    use oz_lexer::Token;

    let lexer = Token::lexer(&src);
    let mut tokens = Vec::new();
    for (token_res, span) in lexer.spanned() {
        match token_res {
            Ok(token) => tokens.push((token, span)),
            Err(_) => panic!("Lexer hatası at {:?}", span),
        }
    }

    let ast = oz_parser::parse_tokens(tokens, src.len()).expect("Ayrıştırma hatası");
    let compiler = compiler::Compiler::new();
    let insts = compiler.compile_program(&ast).expect("Derleme hatası");

    let mut vm = vm::VM::new(insts);
    let stdout_buffer = Rc::new(RefCell::new(Vec::new()));
    vm.stdout = Some(stdout_buffer.clone());

    vm.run().expect("VM çalışma hatası");

    let actual_output = String::from_utf8(stdout_buffer.borrow().clone()).unwrap();
    let actual_norm = actual_output.replace("\r\n", "\n").trim_end().to_string();
    let expected_norm = expected.replace("\r\n", "\n").trim_end().to_string();

    assert_eq!(
        actual_norm, expected_norm,
        "Golden test başarısız! Dosya: {}",
        src_path
    );
}

#[test]
fn test_golden_ornekler() {
    run_golden_test_file(
        "../examples/ornek1_kosul.oz",
        "../tests/golden/ornek1_kosul.stdout",
    );
    run_golden_test_file(
        "../examples/ornek2_dongu.oz",
        "../tests/golden/ornek2_dongu.stdout",
    );
    run_golden_test_file(
        "../examples/ornek3_islev.oz",
        "../tests/golden/ornek3_islev.stdout",
    );
    run_golden_test_file(
        "../examples/ornek4_hesap.oz",
        "../tests/golden/ornek4_hesap.stdout",
    );
    run_golden_test_file(
        "../examples/ornek5_karma.oz",
        "../tests/golden/ornek5_karma.stdout",
    );
}

#[test]
fn test_dahil_et_cift_yukleme() {
    use std::fs;
    let module_content = r#"
        sayac_mod = 1;
    "#;
    let module_path = "test_modul_cift_vm.oz";
    fs::write(module_path, module_content).unwrap();

    let src = r#"
        dahil_et("test_modul_cift_vm.oz");
        dahil_et("test_modul_cift_vm.oz");
        sonuc_sayac = sayac_mod;
    "#;
    let res = run_bytecode(src);
    let _ = fs::remove_file(module_path);

    assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
    let (_, vm) = res.unwrap();
    assert_eq!(vm.get_global("sonuc_sayac"), Some(Val::Number(1.0)));
}

#[test]
fn test_dahil_et_dongusel() {
    use std::fs;
    let mod_a_content = r#"
        dahil_et("test_mod_a_vm.oz");
    "#;
    fs::write("test_mod_a_vm.oz", mod_a_content).unwrap();

    let src = r#"
        dahil_et("test_mod_a_vm.oz");
    "#;
    let res = run_bytecode(src);

    let _ = fs::remove_file("test_mod_a_vm.oz");

    assert!(res.is_err(), "Döngüsel bağımlılık hata fırlatmalıydı!");
    let err_msg = res.err().unwrap();
    assert!(
        err_msg.contains("Döngüsel bağımlılık"),
        "Hata mesajı: {}",
        err_msg
    );
}

#[test]
fn test_dahil_et_std() {
    let src = r#"
        dahil_et("std::matematik");
        dahil_et("std::zaman");
        dahil_et("std::dosya");
        
        kok_degeri = karekok(9);
        ust_degeri = ust(2, 3);
    "#;
    let res = run_bytecode(src);
    assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
    let (_, vm) = res.unwrap();
    assert_eq!(vm.get_global("kok_degeri"), Some(Val::Number(3.0)));
    assert_eq!(vm.get_global("ust_degeri"), Some(Val::Number(8.0)));
}

#[test]
fn test_harita_iyelik_erisim() {
    let src = r#"
        ogrenci = {};
        ogrenci["ad"] = "Ali";
        ogrenci["yas"] = 20;

        ogrenci_adi = ogrenci'nin ad;
        ogrenci_yasi = ogrenci'nin yas;
    "#;
    let res = run_bytecode(src);
    assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
    let (_, vm) = res.unwrap();
    assert_eq!(
        vm.get_global("ogrenci_adi"),
        Some(Val::String("Ali".to_string()))
    );
    assert_eq!(vm.get_global("ogrenci_yasi"), Some(Val::Number(20.0)));
}

#[test]
fn test_zarf_fiil_kontrol_akisi() {
    let src = r#"
        sayac = 0;
        toplam = 0;
        
        sayac < 5 oldukça {
            toplam = toplam + sayac;
            sayac = sayac + 1;
        }

        durum_kontrol = yanlış;
        sayac == 5 olunca {
            durum_kontrol = doğru;
        } değilse {
            durum_kontrol = yanlış;
        }
    "#;
    let res = run_bytecode(src);
    assert!(res.is_ok(), "Hata: {:?}", res.as_ref().err());
    let (_, vm) = res.unwrap();
    assert_eq!(vm.get_global("toplam"), Some(Val::Number(10.0)));
    assert_eq!(vm.get_global("durum_kontrol"), Some(Val::Boolean(true)));
}
