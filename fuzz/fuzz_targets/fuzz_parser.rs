#![no_main]
use libfuzzer_sys::fuzz_target;
use logos::Logos;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let lexer = oz_lexer::Token::lexer(s);
        let mut tokens = Vec::new();
        for (token_res, span) in lexer.spanned() {
            if let Ok(token) = token_res {
                tokens.push((token, span));
            }
        }
        // Parser'a geçersiz token dizileri de verilmeli — parse hatası dönmeli, panic değil
        if !tokens.is_empty() {
            let _ = oz_parser::parse_tokens(tokens, s.len());
        }
    }
});
