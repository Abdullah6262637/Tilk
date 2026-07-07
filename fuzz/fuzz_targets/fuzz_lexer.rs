#![no_main]
use libfuzzer_sys::fuzz_target;
use logos::Logos;

fuzz_target!(|data: &[u8]| {
    // Geçersiz UTF-8 sessizce atlanmalı, panic olmamalı
    if let Ok(s) = std::str::from_utf8(data) {
        let lexer = oz_lexer::Token::lexer(s);
        for _ in lexer.spanned() {
            // Her token'ı consume et — hiçbir input panic'e yol açmamalı
        }
    }
});
