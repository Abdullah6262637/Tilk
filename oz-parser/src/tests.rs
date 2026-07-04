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
