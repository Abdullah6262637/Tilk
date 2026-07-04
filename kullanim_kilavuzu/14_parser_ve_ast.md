# Chumsky Parser ve AST Yapısı

Ayrıştırma (parsing) aşaması Chumsky parser combinator altyapısını kullanır.

## Soyut Sözdizimi Ağacı (AST)
Her bir ifade (`Expr`) ve deyim (`Statement`), kaynak kodundaki konumunu belirten `Span` bilgisiyle sarmalanır:
```rust
pub type Span = std::ops::Range<usize>;

pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}
```
Bu sayede hata raporlamalarında hatanın tam koordinatları verilebilir.
