use super::*;
use logos::Logos;
use oz_lexer::Token;

fn parse_helper(src: &str) -> Result<Vec<Spanned<Statement>>, String> {
    let lexer = Token::lexer(src);
    let mut tokens = Vec::new();
    for (token_res, span) in lexer.spanned() {
        match token_res {
            Ok(token) => tokens.push((token, span)),
            Err(_) => return Err(format!("Lexer hatası: {:?}", span)),
        }
    }
    parse_tokens(tokens, src.len()).map_err(|e| format!("{:?}", e))
}

#[test]
fn test_ornek1_kosul() {
    let src = include_str!("../../examples/ornek1_kosul.oz");
    let res = parse_helper(src);
    assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
}

#[test]
fn test_ornek2_dongu() {
    let src = include_str!("../../examples/ornek2_dongu.oz");
    let res = parse_helper(src);
    assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
}

#[test]
fn test_ornek3_islev() {
    let src = include_str!("../../examples/ornek3_islev.oz");
    let res = parse_helper(src);
    assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
}

#[test]
fn test_ornek4_hesap() {
    let src = include_str!("../../examples/ornek4_hesap.oz");
    let res = parse_helper(src);
    assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
}

#[test]
fn test_ornek5_karma() {
    let src = include_str!("../../examples/ornek5_karma.oz");
    let res = parse_helper(src);
    assert!(res.is_ok(), "Ayrıştırma hatası: {:?}", res);
}

#[test]
fn test_generics_ve_tipler() {
    let src = r#"
        işlev topla<T>(x: T, y: T): T {
            döndür x + y;
        }
    "#;
    let res = parse_helper(src);
    assert!(
        res.is_ok(),
        "Jenerik ve tip belirtimli fonksiyon ayrıştırma hatası: {:?}",
        res
    );
}

#[test]
fn test_null_safety() {
    let src = r#"
        işlev bul_ogrenci(id: Sayı): Metin? {
            döndür boş;
        }
    "#;
    let res = parse_helper(src);
    assert!(
        res.is_ok(),
        "Nullable tip belirtimli fonksiyon ayrıştırma hatası: {:?}",
        res
    );
    let ast = res.unwrap();
    let check_res = typechecker::check_program(&ast);
    assert!(check_res.is_ok(), "Tip denetimi hatası: {:?}", check_res);
}

#[test]
fn test_turkce_ekler() {
    let src = r#"
        ogrenci = {};
        ogrenci["ad"] = "Ali";
        ad_deger1 = ogrenci'yi ad;
        ad_deger2 = ogrenci'de ad;
    "#;
    let res = parse_helper(src);
    assert!(
        res.is_ok(),
        "Türkçe durum ekleri ayrıştırma hatası: {:?}",
        res
    );
    let ast = res.unwrap();
    assert_eq!(ast.len(), 4);
}

fn typecheck_helper(src: &str) -> Result<(), crate::typechecker::types::TypeError> {
    let ast = parse_helper(src).unwrap();
    crate::typechecker::check_program(&ast)
}

#[test]
fn test_error_class_1_undefined_var() {
    let src = "yazdır(olmayan_degisken);";
    let err = typecheck_helper(src).unwrap_err();
    assert!(err.message.contains("Tanımlanamayan değişken"));
}

#[test]
fn test_error_class_2_type_mismatch() {
    let src = "sonuc = 5 + \"metin\";";
    let err = typecheck_helper(src).unwrap_err();
    assert!(err.message.contains("birleştirilemiyor"));
    assert_eq!(err.expected, Some(crate::typechecker::types::Type::Number));
}

#[test]
fn test_error_class_3_func_args_mismatch() {
    let src = "işlev topla(a, b) { döndür a + b; } topla(5);";
    let err = typecheck_helper(src).unwrap_err();
    assert!(err.message.contains("argüman sayısı uyuşmuyor"));
}

#[test]
fn test_error_class_4_indexing_wrong_type() {
    let src = "x = 5; y = x[0];";
    let err = typecheck_helper(src).unwrap_err();
    assert!(err
        .message
        .contains("Sadece diziler, haritalar ve kanallar indekslenebilir"));
}

#[test]
fn test_error_class_5_infinite_type() {
    let src = "dizi = []; dizi = ekle(dizi, dizi);";
    let err = typecheck_helper(src).unwrap_err();
    assert!(err.message.contains("Sonsuz tip"));
}
