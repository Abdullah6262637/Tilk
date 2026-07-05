# TİLK Programlama Dili 🚀

[![CI](https://github.com/Abdullah6262637/Tilk/actions/workflows/ci.yml/badge.svg)](https://github.com/Abdullah6262637/Tilk/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org/)

TİLK (`.oz`), Türkçenin sondan eklemeli (aglutinative) dilbilimsel çekirdeğini ve kelime türetme mantığını doğrudan kontrol yapıları olarak kullanan, yüksek performanslı ve modern bir sistem/uygulama programlama dilidir. Başka hiçbir dilin doğrudan kelime çevirisi olmayıp, özgün bir dil tasarımı felsefesine dayanır.

---

## ⚡ Hızlı Başlangıç

### Kurulum

```bash
# Depoyu klonlayın
git clone https://github.com/Abdullah6262637/Tilk.git
cd Tilk

# Derleyin
cargo build --release

# Bir TİLK programı çalıştırın
cargo run -- calistir examples/ornek1_kosul.oz
```

### İlk Programınız

`merhaba.oz` dosyası oluşturun:

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

## 📌 İÇİNDEKİLER

1. [Dil Tasarım Felsefesi](#1-dil-tasarim-felsefesi)
2. [EBNF Sözdizimi Şeması](#2-ebnf-sozdizimi-semasi)
3. [Derleme ve Yürütme Mimarisi (Pipeline)](#3-derleme-ve-yurutme-mimarisi-pipeline)
4. [Sanal Makine (VM) ve Performans Optimizasyonları](#4-sanal-makine-vm-ve-performans-optimizasyonlari)
5. [Span-Tabanlı Görsel Hata Teşhis Sistemi (Ariadne)](#5-span-tabanli-gorsel-hata-teshis-sistemi-ariadne)
6. [CLI Araç Zinciri ve Paket Yönetimi](#6-cli-arac-zinciri-ve-paket-yonetimi)
7. [Editör Desteği](#7-editor-destegi)
8. [Örnek Uygulamalar Kataloğu](#8-ornek-uygulamalar-katalogu)
9. [Katkıda Bulunma](#9-katkida-bulunma)
10. [Gelecek Yol Haritası](#10-gelecek-yol-haritasi)

---

## 1. DİL TASARIM FELSEFESİ

TİLK, sözcük düzeyinde bir Türkçe çeviri (\"TPD\" veya \"Karamel\" gibi sahte yerelleştirmeler) değildir. Dilin kontrol yapıları ve tümce kurgusu, Türkçenin sondan eklemeli yapısına sadık kalınarak oluşturulmuştur.

### 1.1 Koşullar (`-ise` / `-se`)
Türkçedeki şart kipi doğrudan koşul bloklarını yönetir:
```oz
sayı > 5 ise {
    yazdır("Sayı 5'ten büyüktür.");
} değilse {
    yazdır("Sayı 5 veya daha küçüktür.");
}
```

### 1.2 Zarf-Fiil Döngüleri (`-iken`)
Bir eylemin sürdüğü aralığı belirtmek için kullanılan `-iken` eki, koşullu döngüleri (`while`) kurar:
```oz
sayaç <= 3 iken {
    yazdır(sayaç);
    sayaç = sayaç + 1;
}
```

### 1.3 Yönelme ve Ayrılma Ekli Aralık Döngüleri (`-den ... -e dek`)
Sayaçlı döngüler (`for`) Türkçedeki yönelme (`-e / -a`) ve ayrılma (`-den / -dan`) ekleri ile `dek` edatı birleştirilerek kurgulanmıştır:
```oz
i, 1'den 5'e dek artarak {
    yazdır(i);
}
```

### 1.4 Hata Yönetimi (`hata_ise`)
Türkçe deyimsel hata yakalama:
```oz
sonuç = güvenli_böl(10, 0) hata_ise {
    yazdır("Hata:", hata_mesajı);
    döndür 0;
};
```

### 1.5 Asenkron Programlama
```oz
görev = arkaplanda_çalıştır(hesapla, 10, 20);
görev tamamlanınca {
    yazdır("Sonuç:", sonuç);
}
```

---

## 2. EBNF SÖZDİZİMİ ŞEMASI

TİLK dilinin resmî EBNF dilbilgisi kuralları:

```ebnf
Program         ::= Statement*
Statement       ::= VarDecl | Assignment | IfStatement | WhileStatement | ForStatement | ForEachStatement | FnDeclaration | ReturnStatement | ExprStatement
VarDecl         ::= Identifier "=" Expr ";"
Assignment      ::= Identifier "=" Expr ";"
ReturnStatement ::= "döndür" Expr? ";"
IfStatement     ::= Expr ("ise" | "se") Block ( "değilse" Block )?
WhileStatement  ::= Expr "iken" Block
ForStatement    ::= Identifier "," Expr ("dan" | "den" | "tan" | "ten") Expr ("e" | "a" | "ye" | "ya") "dek" ("artarak" | "azalarak")? Block
ForEachStatement::= "her" "(" Identifier "içinde" Expr ")" Block
FnDeclaration   ::= "işlev" Identifier "(" ParamList? ")" Block
Block           ::= "{" Statement* "}"
Expr            ::= LogicalOrExpr
```

---

## 3. DERLEME VE YÜRÜTME MİMARİSİ (PIPELINE)

TİLK, klasik bir derleyici boru hattını takip eder:

```
Kaynak Kod (.oz)
    │
    ▼
┌─────────────┐    ┌─────────────┐    ┌──────────────┐    ┌───────────┐    ┌────────┐
│  oz-lexer   │───▶│  oz-parser  │───▶│ Tip Denetimi │───▶│ Derleyici │───▶│  VM    │
│  (Logos)    │    │ (Chumsky)   │    │    (HM)      │    │(Bytecode) │    │(Stack) │
└─────────────┘    └─────────────┘    └──────────────┘    └───────────┘    └────────┘
```

1. **Sözcüksel Analiz (Lexer)**: `oz-lexer` — Logos tabanlı DFA lexer, Unicode/NFC normalizasyon
2. **Ayrıştırıcı (Parser)**: `oz-parser` — Chumsky parser kombinatörü
3. **Tip Denetimi (Typechecker)**: Hindley-Milner tip çıkarım sistemi
4. **Bayt Kodu Oluşturucu (Compiler)**: AST'yi doğrusal VM talimatlarına derler
5. **Sanal Makine (VM)**: `oz-vm` — Yığın tabanlı yüksek hızlı makine

---

## 4. SANAL MAKİNE (VM) VE OPTİMİZASYONLAR

### 4.1 Slot-Index Tabanlı Lokal Değişken Yönetimi
Her yerel değişken bir `u16` indeks değerine çözümlenir. `slots: Vec<Val>` ile O(1) erişim sağlanır.

### 4.2 Kısa Devre (Short-Circuit) Tasarımı
`JumpIfFalseKeep` ve `JumpIfTrueKeep` opkodları ile `ve`/`veya` operatörlerinde performans korunur.

### 4.3 Kaçış Karakterleri
`\n`, `\t`, `\r`, `\"`, `\\` kaçış dizilimleri lexer aşamasında çözülür.

---

## 5. SPAN-TABANLI GÖRSEL HATA TEŞHİS SİSTEMİ (ARIADNE)

AST düğümlerinin tamamı `Span` bilgisiyle sarmalanır. Hata mesajları `ariadne` ile görselleştirilir:

```
Error: Sayı bekleniyordu
   ╭─[test.oz:1:13]
   │
 1 │ sayı = 5 + * 4 ;
   │             ┬  
   │             ╰── Dosya sonu
───╯
```

---

## 6. CLI ARAÇ ZINCİRİ VE PAKET YÖNETİMİ

`oz-cli`, TİLK ekosisteminin resmi komut satırı aracıdır.

| Komut | Açıklama |
|-------|----------|
| `yeni <ad>` | Yeni bir TİLK projesi şablonu oluşturur |
| `derle` | `tilk.toml` baz alarak tüm projeyi derler |
| `calistir [dosya]` | Projeyi veya bir `.oz` dosyasını çalıştırır |
| `test` | `testler/` dizinindeki tüm dosyaları test eder |

---

## 7. EDİTÖR DESTEĞİ

### VS Code Eklentisi

TİLK, VS Code için sözdizimi renklendirmesi ve LSP desteği sunar:

- 🎨 **Sözdizimi Renklendirmesi**: Anahtar kelimeler, yerleşik fonksiyonlar, metin ve sayılar
- 📝 **Otomatik Tamamlama**: Yerleşik fonksiyonlar ve tanımlı değişkenler
- 🔍 **Anlık Hata Gösterimi**: Sözcüksel, ayrıştırma ve tip hataları
- 💡 **Hover Bilgisi**: Değişken tipleri üzerinde ipucu

Eklenti dosyaları `editors/vscode/` dizininde bulunur.

### LSP Sunucusu

`oz-lsp` modülü, Tower-LSP tabanlı bir dil sunucusu sağlar ve herhangi bir LSP destekli editörle kullanılabilir.

---

## 8. ÖRNEK UYGULAMALAR KATALOĞU

| Dosya | Konu |
|-------|------|
| [`ornek1_kosul.oz`](examples/ornek1_kosul.oz) | Koşul yapıları (`ise`/`değilse`) |
| [`ornek2_dongu.oz`](examples/ornek2_dongu.oz) | Döngüler (`iken`) |
| [`ornek3_islev.oz`](examples/ornek3_islev.oz) | Fonksiyonlar (`işlev`) |
| [`ornek4_hesap.oz`](examples/ornek4_hesap.oz) | Aritmetik hesaplamalar |
| [`ornek5_karma.oz`](examples/ornek5_karma.oz) | Karma yapılar |
| [`ornek6_hata_yonetimi.oz`](examples/ornek6_hata_yonetimi.oz) | Hata yönetimi (`hata_ise`) |
| [`ornek7_diziler_haritalar.oz`](examples/ornek7_diziler_haritalar.oz) | Diziler ve haritalar |
| [`ornek8_asenkron.oz`](examples/ornek8_asenkron.oz) | Asenkron programlama ve kanallar |
| [`ornek9_matematik.oz`](examples/ornek9_matematik.oz) | Matematik ve zaman fonksiyonları |
| [`ornek10_modul.oz`](examples/ornek10_modul.oz) | Modül sistemi (`dahil_et`) |

### Özyinelemeli Faktöriyel Örneği
```oz
işlev faktöriyel(n) {
    n <= 1 ise {
        döndür 1;
    }
    döndür n * faktöriyel(n - 1);
}

yazdır(faktöriyel(5)); // 120
```

---

## 9. KATKIDA BULUNMA

Katkılarınızı bekliyoruz! Detaylar için [CONTRIBUTING.md](CONTRIBUTING.md) dosyasına bakınız.

```bash
# Geliştirme ortamı
git clone https://github.com/Abdullah6262637/Tilk.git
cd Tilk
cargo build
cargo test --all
```

---

## 10. ÖZELLİKLER VE YOL HARİTASI

### ✅ Çalışan Özellikler
- **Modüler Mimari:** Lexer, Parser, HM Tip Denetimi, Bytecode Compiler, VM bağımsız modüller halindedir.
- **Yerel Sözdizimi:** `ise`, `iken`, `den/e dek`, `her` gibi Türkçenin yapısına uygun özgün blok yapıları.
- **Hata Yönetimi:** `hata_ise` ve `tamamlanınca` bloklarıyla Türkçe dilbilgisine uyan istisna yönetimi.
- **Standart Kütüphane:** Dahili modüller `std/` dizini içinde yer alır (örn. `matematik`, `metin`, `zaman`).
- **Paket Yöneticisi:** `tilk.lock` tabanlı sürüm ve checksum (md5) yönetimli bağımlılık kurulum aracı (`oz-cli yükle`).

### 🧪 Deneysel Özellikler
- **C Backend Transpilation:** TİLK kaynak kodunu C'ye çevirerek GCC/Clang ile yerel çalıştırılabilir formata dönüştürür.
- **Asenkron Kanallar:** Go benzeri kanallarla (channels) arka plan görevleri arası iletişim (`asenkron` anahtar kelimesi).
- **Dil Sunucusu (LSP):** Otomatik tamamlama, anlık hata denetimi ve hover yardımı sağlayan `oz-lsp`.

### 🚀 Gelecek Planları (Uzun Vadeli)
- [ ] Closure (kapanış) desteği
- [ ] String interpolasyonu (`"Merhaba {isim}"`)
- [ ] Pattern matching (`eşleştir` anahtar kelimesi)
- [ ] **Cranelift/LLVM Arka Ucu**: Native makine kodu üretimi (AOT/JIT)
- [ ] **WebAssembly (WASM)**: Tarayıcıda native hızda çalışma
- [ ] **Paket Deposu**: Topluluk kütüphaneleri için merkezi kayıt sistemi

---

## 📄 Lisans

Bu proje [MIT Lisansı](LICENSE) altında lisanslanmıştır.

---

## 📚 Kapsamlı Kılavuzlar

Tilk dili hakkında detaylı 30 adet kılavuz belgesi [kullanim_kilavuzu/](kullanim_kilavuzu/) dizini altında yer almaktadır.
