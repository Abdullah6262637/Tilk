use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\n\f']+")] // Skip whitespace and Turkish suffix apostrophes (')
pub enum Token {
    // Core Keywords
    #[token("işlev")]
    Islev,

    #[token("döndür")]
    Dondur,

    #[token("değilse")]
    Degilse,

    #[token("doğru")]
    Dogru,

    #[token("yanlış")]
    Yanlis,

    #[token("boş")]
    Bos,

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

    // Literals & Identifiers
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().to_string())]
    Number(String),

    #[regex(r#""[^"\\]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
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
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
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
