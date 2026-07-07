# TİLK Programlama Dili

<div align="center">

[![CI](https://github.com/Abdullah6262637/Tilk/actions/workflows/ci.yml/badge.svg)](https://github.com/Abdullah6262637/Tilk/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org/)

*Türkçenin dilbilgisi kuralları üzerine inşa edilmiş, yüksek performanslı ve modern bir sistem programlama dili.*

</div>

---

TİLK (`.oz`), Türkçenin sondan eklemeli (aglutinative) dilbilimsel çekirdeğini ve kelime türetme mantığını doğrudan kontrol yapıları olarak kullanan, Rust ile geliştirilmiş yenilikçi bir programlama dilidir. Klasik programlama dillerinin sözcük bazında çevirisi olmaktan ziyade, tamamen Türkçe düşünme yapısına göre tasarlanmış özgün bir mimariye sahiptir.

## Hızlı Başlangıç

### Kurulum ve Derleme

TİLK derleyicisini sisteminize kurmak ve çalıştırmak için aşağıdaki adımları izleyebilirsiniz:

```bash
# Depoyu sisteminize kopyalayın
git clone https://github.com/Abdullah6262637/Tilk.git
cd Tilk

# Projeyi optimize edilmiş sürümle derleyin
cargo build --release

# Örnek bir TİLK programını çalıştırın
cargo run --release -- calistir examples/ornek1_kosul.oz
```

### İlk Programınız

Aşağıdaki kodu `merhaba.oz` adıyla kaydedin ve çalıştırın:

```oz
yazdır("Merhaba Dünya!");

isim = "TİLK";
yazdır("Ben", isim, "programlama diliyim!");

sayı = 42;
sayı > 10 ise {
    yazdır("Sayı 10'dan büyük!");
} değilse {
    yazdır("Sayı 10 veya daha küçük.");
}
```

---

## İçindekiler

1. [Dil Tasarım Felsefesi](#1-dil-tasarim-felsefesi)
2. [EBNF Sözdizimi Şeması](#2-ebnf-sozdizimi-semasi)
3. [Derleme ve Yürütme Mimarisi](#3-derleme-ve-yurutme-mimarisi)
4. [Sanal Makine (VM) ve Performans Optimizasyonları](#4-sanal-makine-vm-ve-performans-optimizasyonlari)
5. [Span-Tabanlı Görsel Hata Teşhis Sistemi](#5-span-tabanli-gorsel-hata-teshis-sistemi)
6. [CLI Araç Zinciri ve Paket Yönetimi](#6-cli-arac-zinciri-ve-paket-yonetimi)
7. [Editör Desteği](#7-editor-destegi)
8. [Gelecek Yol Haritası](#8-gelecek-yol-haritasi)

---

## 1. Dil Tasarım Felsefesi

TİLK, sözcük düzeyinde basit bir kelime eşleştirme aracı değildir. Dilin kontrol yapıları ve tümce kurgusu, Türkçenin akışına tam uyum sağlayacak şekilde tasarlanmıştır.

### 1.1. Koşullu İfadeler
Türkçedeki şart kipi doğrudan koşul bloklarını yönetir:
```oz
sayı > 5 ise {
    yazdır("Sayı 5'ten büyüktür.");
} değilse {
    yazdır("Sayı 5 veya daha küçüktür.");
}
```

### 1.2. Zarf-Fiil Döngüleri
Bir eylemin sürdüğü aralığı belirtmek için kullanılan `-iken` eki, koşullu döngüleri (`while`) kurar:
```oz
sayaç <= 3 iken {
    yazdır(sayaç);
    sayaç = sayaç + 1;
}
```

### 1.3. Yönelme ve Ayrılma Ekli Aralık Döngüleri
Sayaçlı döngüler (`for`), Türkçedeki yönelme (`-e / -a`) ve ayrılma (`-den / -dan`) ekleri ile `dek` edatı birleştirilerek kurgulanmıştır:
```oz
i, 1'den 5'e dek artarak {
    yazdır(i);
}
```

### 1.4. Türkçe Hata Yönetimi
İstisnai durumları yönetmek için deyimsel bir yaklaşım benimsenmiştir:
```oz
sonuç = (10 / 0) hata_ise {
    yazdır("Hata oluştu:", hata_mesajı);
    döndür 0;
};
```

---

## 2. EBNF Sözdizimi Şeması

TİLK dilinin resmî EBNF dilbilgisi kuralları özeti:

```ebnf
Program         ::= Statement*
Statement       ::= VarDecl | Assignment | IfStatement | WhileStatement 
                  | ForStatement | ForEachStatement | FnDeclaration 
                  | ReturnStatement | ExprStatement
VarDecl         ::= Identifier "=" Expr ";"
Assignment      ::= Identifier "=" Expr ";"
ReturnStatement ::= "döndür" Expr? ";"
IfStatement     ::= Expr ("ise" | "se") Block ( "değilse" Block )?
WhileStatement  ::= Expr "iken" Block
ForStatement    ::= Identifier "," Expr ("dan"|"den") Expr ("e"|"a") "dek" ("artarak"|"azalarak")? Block
ForEachStatement::= "her" "(" Identifier "içinde" Expr ")" Block
FnDeclaration   ::= "işlev" Identifier "(" ParamList? ")" Block
Block           ::= "{" Statement* "}"
```

---

## 3. Derleme ve Yürütme Mimarisi

TİLK, yüksek performans hedefleyen çok katmanlı bir derleyici boru hattını (pipeline) takip eder:

1. **Sözcüksel Analiz (Lexer)**: `oz-lexer` — Logos tabanlı DFA lexer, tam Unicode/NFC desteği.
2. **Ayrıştırıcı (Parser)**: `oz-parser` — Chumsky tabanlı parser kombinatörü.
3. **Tip Denetimi**: Hindley-Milner tabanlı sıkı tip çıkarım sistemi.
4. **Bytecode Derleyici**: Soyut sözdizim ağacını (AST) VM talimatlarına çevirir.
5. **Sanal Makine (VM)**: `oz-vm` — İşletim sistemi kaynaklarını etkin kullanan yığın (stack) tabanlı motor.

---

## 4. Sanal Makine (VM) ve Performans Optimizasyonları

- **Slot-Index Lokal Değişken Yönetimi**: Her yerel değişken bir `u16` indeks değerine çözümlenir ve `O(1)` hızında erişilir.
- **Kısa Devre (Short-Circuit)**: Mantıksal `ve` / `veya` operatörleri akış denetimini optimize ederek gereksiz hesaplamaları önler.
- **WASM Entegrasyonu**: TİLK dilinin tamamı WebAssembly olarak derlenebilmekte ve tarayıcı üzerinde native performansta çalışabilmektedir (`oz-wasm`).

---

## 5. Span-Tabanlı Görsel Hata Teşhis Sistemi

AST düğümlerinin tamamı dosya konumu bilgisi (Span) ile sarmalanır. Bu sayede hatalar, profesyonel araçlarla (`ariadne`) görselleştirilerek kullanıcıya sunulur:

```text
Error: Sayı bekleniyordu
   ╭─[test.oz:1:13]
   │
 1 │ sayı = 5 + * 4 ;
   │             ┬  
   │             ╰── Beklenmeyen operatör
───╯
```

---

## 6. CLI Araç Zinciri ve Paket Yönetimi

`oz-cli`, projenin resmi komut satırı aracıdır. Zengin özellikleri ile modern bir geliştirici deneyimi sunar:

| Komut | Açıklama |
|-------|----------|
| `yeni <ad>` | Standart dosya hiyerarşisiyle yeni bir TİLK projesi oluşturur. |
| `derle` | `tilk.toml` yapılandırmasını okuyarak modülleri derler. |
| `calistir` | Projeyi veya belirtilen bir `.oz` dosyasını çalıştırır. |
| `fmt` | Kodu TİLK standartlarına uygun şekilde otomatik formatlar. |
| `repl` | Etkileşimli kod deneme kabuğunu (Read-Eval-Print Loop) başlatır. |

---

## 7. Editör Desteği

### VS Code Eklentisi
- Sözdizimi renklendirmesi (Syntax Highlighting)
- Otomatik tamamlama (Autocomplete)
- Anlık hata gösterimi (Diagnostics)
- `editors/vscode/` dizininde kaynak kodlarına erişebilirsiniz.

### Dil Sunucusu (LSP)
`oz-lsp` paketi, Tower-LSP üzerine inşa edilmiştir. Standart LSP istemcisine sahip her IDE'de (Neovim, Sublime Text, vb.) tam teşekküllü dil asistanı olarak çalışır.

---

## 8. Özellikler ve Yol Haritası

### Mevcut Yetenekler
- Modüler ve ayrıştırılmış derleyici mimarisi.
- Türkçe dil bilgisine uygun akıcı kod akışı.
- Kapsamlı standart kütüphane desteği (matematik, metin, zaman).
- WebAssembly (WASM) desteği ile tarayıcı ortamında çalışma.
- Tam kapsamlı REPL ve otomatik kod formatlayıcı (`fmt`).

### Gelecek Planları
- Closure (Kapanış) desteği.
- Pattern matching (Eşleştirme) altyapısı.
- Cranelift veya LLVM arka ucu ile native JIT/AOT makine kodu üretimi.
- TİLK projeleri için merkezi bir paket deposu (Registry) oluşturulması.

---

## Lisans

Bu proje [MIT Lisansı](LICENSE) koşulları altında dağıtılmaktadır. Daha fazla bilgi için `LICENSE` dosyasına başvurabilirsiniz.

Kapsamlı eğitim metinleri için [kullanim_kilavuzu/](kullanim_kilavuzu/) dizinini veya [10 Dakikada TİLK](10_dakikada_tilk.md) rehberini inceleyebilirsiniz.
