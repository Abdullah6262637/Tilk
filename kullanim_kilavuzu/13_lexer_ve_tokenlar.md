# Logos Lexer ve Token Yapısı

TİLK sözcüksel analizcisi Rust'ın `logos` crate'ini kullanır.

## Kesme İşareti Yoksayma
Türkçe kesme işaretleri (`'`) lexer seviyesinde otomatik olarak yok sayılır:
```rust
#[logos(skip r"[ \t\n\f']+")]
```
Bu sayede `5'ten` yazıldığında `5` ve `ten` tokenları elde edilir.

## String Escape Karakterleri
Çift tırnak içindeki `\n`, `\t`, `\r`, `\"` ve `\\` kaçış dizilimleri lexer tarafından çözülerek String token'ına dönüştürülür.
