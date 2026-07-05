use logos::Logos;
use unicode_normalization::UnicodeNormalization;

#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\r\n\f']+|//[^\n]*")] // Skip whitespace, Turkish suffix apostrophes ('), and comments starting with //
pub enum Token {
    // Core Keywords
    #[token("işlev")]
    Islev,

    #[token("döndür")]
    Dondur,

    #[token("değilse")]
    Degilse,

    #[token("değil")]
    Degil,

    #[token("doğru")]
    Dogru,

    #[token("yanlış")]
    Yanlis,

    #[token("boş")]
    Bos,

    #[token("hata_ise")]
    HataIse,

    #[token("tamamlanınca")]
    #[token("tamamlandığında")]
    Tamamlaninca,

    #[token("her")]
    Her,

    #[token("içinde")]
    #[token("icinde")]
    Icinde,

    // Operators
    #[token("=")]
    Assign,

    #[token("==")]
    Eq,

    #[token("!=")]
    Ne,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[token("<=")]
    Le,

    #[token(">=")]
    Ge,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Mul,

    #[token("/")]
    Div,

    #[token("%")]
    Mod,

    #[token("ve")]
    And,

    #[token("veya")]
    Or,

    // Punctuation
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token("::")]
    DoubleColon,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("?")]
    QuestionMark,

    // Literals & Identifiers
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().to_string())]
    Number(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        let mut res = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    match next {
                        'n' => res.push('\n'),
                        't' => res.push('\t'),
                        'r' => res.push('\r'),
                        '"' => res.push('"'),
                        '\\' => res.push('\\'),
                        _ => {
                            res.push('\\');
                            res.push(next);
                        }
                    }
                } else {
                    res.push('\\');
                }
            } else {
                res.push(c);
            }
        }
        res
    })]
    String(String),

    // Identifier containing Turkish characters
    #[regex(r"[a-zA-ZçğıişöüÇĞIİŞÖÜ_][a-zA-Z0-9çğıişöüÇĞIİŞÖÜ_]*", |lex| {
        turkish_lowercase(lex.slice())
    }, priority = 1)]
    Identifier(String),
}

/// Locale-independent Turkish lowercase normalization to avoid the "Turkish I" bug
pub fn turkish_lowercase(s: &str) -> String {
    let normalized: String = s.nfc().collect();
    let mut result = String::with_capacity(normalized.len());
    for c in normalized.chars() {
        match c {
            'İ' => result.push('i'),
            'I' => result.push('ı'),
            _ => {
                for lc in c.to_lowercase() {
                    result.push(lc);
                }
            }
        }
    }
    result
}
