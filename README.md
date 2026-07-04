# TİLK Programlama Dili: Kapsamlı Teknik El Kitabı ve Belgelendirme 🚀

TİLK (`.oz`), Türkçenin sondan eklemeli (aglutinative) dilbilimsel çekirdeğini ve kelime türetme mantığını doğrudan kontrol yapıları olarak kullanan, yüksek performanslı ve modern bir sistem/uygulama programlama dilidir. Başka hiçbir dilin doğrudan kelime çevirisi olmayıp, özgün bir dil tasarımı felsefesine dayanır.

---

## 📌 İÇİNDEKİLER

1. [Dil Tasarım Felsefesi](#1-dil-tasarim-felsefesi)
2. [EBNF Sözdizimi Şeması](#2-ebnf-sozdizimi-semasi)
3. [Derleme ve Yürütme Mimarisi (Pipeline)](#3-derleme-ve-yurutme-mimarisi-pipeline)
4. [Sanal Makine (VM) ve Performans Optimizasyonları](#4-sanal-makine-vm-ve-performans-optimizasyonlari)
5. [Span-Tabanlı Görsel Hata Teşhis Sistemi (Ariadne)](#5-span-tabanli-gorsel-hata-teshis-sistemi-ariadne)
6. [CLI Araç Zinciri ve Paket Yönetimi](#6-cli-arac-zinciri-ve-paket-yonetimi)
7. [Örnek Uygulamalar Kataloğu](#7-ornek-uygulamalar-katalogu)
8. [Kapsamlı Kullanım Kılavuzu İndeksi (30 Adet Belge)](#8-kapsamli-kullanim-kilavuzu-indeksi)
9. [Gelecek Yol Haritası (500x Ölçeklendirme)](#9-gelecek-yol-haritasi)

---

## 1. DİL TASARIM FELSEFESİ

TİLK, sözcük düzeyinde bir Türkçe çeviri ("TPD" veya "Karamel" gibi sahte yerelleştirmeler) değildir. Dilin kontrol yapıları ve tümce kurgusu, Türkçenin sondan eklemeli yapısına sadık kalınarak oluşturulmuştur.

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
Sayaçlı döngüler (`for`) Türkçedeki yönelme (`-e / -a`) ve ayrılma (`-den / -dan`) ekleri ile `dek` edatı birleştirilerek kurgulanmıştır. Kesme işaretleri (`'`) lexer seviyesinde yoksayılır, böylece doğal yazım korunur:
```oz
i, 1'den 5'e dek artarak {
    yazdır(i);
}
```

---

## 2. EBNF SÖZDİZİMİ ŞEMASI

TİLK dilinin resmî EBNF dilbilgisi kuralları:

```ebnf
Program         ::= Statement*
Statement       ::= VarDecl | Assignment | IfStatement | WhileStatement | ForStatement | FnDeclaration | ReturnStatement | ExprStatement
VarDecl         ::= Identifier "=" Expr ";"
Assignment      ::= Identifier "=" Expr ";"
ReturnStatement ::= "döndür" Expr? ";"
IfStatement     ::= Expr ("ise" | "se") Block ( "değilse" Block )?
WhileStatement  ::= Expr "iken" Block
ForStatement    ::= Identifier "," Expr ("dan" | "den" | "tan" | "ten") Expr ("e" | "a" | "ye" | "ya") "dek" ("artarak" | "azalarak")? Block
FnDeclaration   ::= "işlev" Identifier "(" ParamList? ")" Block
Block           ::= "{" Statement* "}"
Expr            ::= LogicalOrExpr
LogicalOrExpr   ::= LogicalAndExpr ( "veya" LogicalAndExpr )*
LogicalAndExpr  ::= EqualityExpr ( "ve" EqualityExpr )*
EqualityExpr    ::= ComparisonExpr ( ( "==" | "!=" ) ComparisonExpr )*
ComparisonExpr  ::= Term ( ( "<" | ">" | "<=" | ">=" ) Term )*
Term            ::= Factor ( ( "+" | "-" ) Factor )*
Factor          ::= Primary ( ( "*" | "/" | "%" ) Primary )*
Primary         ::= Identifier | Number | String | Boolean | "boş" | CallExpr | "(" Expr ")"
CallExpr        ::= Identifier "(" ArgList? ")"
```

---

## 3. DERLEME VE YÜRÜTME MİMARİSİ (PIPELINE)

TİLK, klasik bir derleyici boru hattını (compiler pipeline) takip eder:

1. **Sözcüksel Analiz (Lexer)**: `oz-lexer` modülü, Rust'ın en hızlı DFA lexer üreteci olan `logos` kütüphanesini kullanır. Türkçe harflerin büyük-küçük uyuşmazlığı (`İ`/`i`, `I`/`ı`) yerel ayarlardan bağımsız `turkish_lowercase` fonksiyonu ve `unicode-normalization` (NFC) ile normalize edilir.
2. **Ayrıştırıcı (Parser)**: `oz-parser` modülü, güçlü bir parser kombinatörü olan `chumsky` kütüphanesini kullanır.
3. **Tip Denetimi (Typechecker)**: Hindley-Milner tip çıkarım sistemi ile dildeki tiplerin uyuşup uyuşmadığı derleme anında doğrulanır.
4. **Bayt Kodu Oluşturucu (Compiler)**: AST'yi doğrusal VM talimatlarına derler.
5. **Sanal Makine (VM)**: `oz-vm` modülü, bayt kodlarını yüksek hızlı bir yığın makinesinde yürütür.

---

## 4. SANAL MAKİNE (VM) VE OPTİMİZASYONLAR

### 4.1 Slot-Index Tabanlı Lokal Değişken Yönetimi
Yerel değişkenlerin HashMap'te string anahtarla aranıp yavaş çalışması yerine, derleyici düzeyinde her yerel değişken bir `u16` indeks değerine çözümlenir. VM Frame yapısı güncellenerek `locals: HashMap` yerine `slots: Vec<Val>` O(1) tabanlı vektöre geçilmiştir.

### 4.2 Kısa Devre (Short-Circuit) Tasarımı
Mantıksal `ve` ile `veya` operatörleri için VM'e `JumpIfFalseKeep` ve `JumpIfTrueKeep` opkodları eklenmiştir. Sol tarafın durumuna göre sağ tarafın değerlendirilmesi tamamen atlanarak performans korunur.

### 4.3 Kaçış Karakterleri Desteği
Metinlerde (String literals) kullanılan `\n` (satır atlama), `\t` (sekme), `\r` (satır başı), `\"` (çift tırnak) ve `\\` (ters eğik çizgi) kaçış dizilimleri lexer aşamasında çözülür.

---

## 5. SPAN-TABANLI GÖRSEL HATA TEŞHİS SİSTEMİ (ARIADNE)

AST düğümlerinin tamamı, kaynak kodundaki karakter koordinatlarını belirten `Span` bilgisiyle sarmalanır. Derleme veya ayrıştırma hatası alındığında `ariadne` yardımıyla hata görselleştirilir:

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

- `yeni <ad>`: Yeni bir TİLK projesi şablonu oluşturur.
- `derle`: Manifest dosyasını (`tilk.toml`) baz alarak tüm projeyi derler.
- `calistir [dosya]`: Projeyi veya belirli bir `.oz` dosyasını VM üzerinde çalıştırır.
- `test`: `testler/` dizini altındaki tüm dosyaları tarayıp koşturur.

---

## 7. ÖRNEK UYGULAMALAR KATALOĞU

### 7.1 Özyinelemeli Faktöriyel
```oz
işlev faktöriyel(n) {
    n <= 1 ise {
        döndür 1;
    }
    döndür n * faktöriyel(n - 1);
}

yazdır(faktöriyel(5)); // 120
```

### 7.2 Hata Yakalama ve Aritmetik
```oz
işlev güvenli_böl(a, b) {
    b == 0 ise {
        hata_fırlat("Sıfıra bölme yapılamaz!");
    }
    döndür a / b;
}

sonuç = güvenli_böl(10, 0) hata_ise {
    yazdır("Hata oluştu:", hata_mesajı);
    döndür 0;
};
```

---

## 8. KAPSAMLI KULLANIM KILAVUZU İNDEKSİ

Tilk dili hakkında her ayrıntıyı teker teker açıklayan 30 adet detaylı kılavuz belgesi [kullanim_kilavuzu/](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu) dizini altında yer almaktadır:

1. [01_giris_ve_tasarim.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/01_giris_ve_tasarim.md): Dilin dilbilimsel çekirdeği ve tasarım felsefesi.
2. [02_sozdizimi_kurallari.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/02_sozdizimi_kurallari.md): Dilbilgisinin EBNF spesifikasyonları.
3. [03_degiskenler_ve_tipler.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/03_degiskenler_ve_tipler.md): Dinamik tipler ve değişken atamaları.
4. [04_kosul_yapilari.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/04_kosul_yapilari.md): `ise`, `se` ve `değilse` koşul blokları.
5. [05_donguler.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/05_donguler.md): `iken` ve `-den ... -e dek` döngü yapıları.
6. [06_islevler_ve_recursion.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/06_islevler_ve_recursion.md): Fonksiyon tanımlama ve özyineleme mantığı.
7. [07_diziler_ve_haritalar.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/07_diziler_ve_haritalar.md): Diziler, haritalar ve mutasyon işlemleri.
8. [08_hata_yonetimi_hata_ise.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/08_hata_yonetimi_hata_ise.md): Hata fırlatma ve `hata_ise` ile yakalama.
9. [09_asenkron_tamamlaninca.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/09_asenkron_tamamlaninca.md): Görevler ve asenkron programlama.
10. [10_dosya_io_islemleri.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/10_dosya_io_islemleri.md): Dosya sistemi okuma, yazma ve silme yardımcıları.
11. [11_zaman_ve_uyku.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/11_zaman_ve_uyku.md): `şimdi` ve thread duraklatan `uyku` fonksiyonu.
12. [12_matematik_fonksiyonlari.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/12_matematik_fonksiyonlari.md): `kök`, `üs` ve `mutlak` gibi yerleşik matematik araçları.
13. [13_lexer_ve_tokenlar.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/13_lexer_ve_tokenlar.md): logos lexer mimarisi ve token skip kuralları.
14. [14_parser_ve_ast.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/14_parser_ve_ast.md): Chumsky parser mimarisi ve AST span tasarımı.
15. [15_tip_kontrol_sistemi.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/15_tip_kontrol_sistemi.md): HM tip çıkarımı ve unification.
16. [16_derleyici_ve_bytecode.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/16_derleyici_ve_bytecode.md): AST'den bayt koduna derleme aşaması.
17. [17_sanal_makine_vm_mimarisi.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/17_sanal_makine_vm_mimarisi.md): Yığın makinesi, call frame ve execution döngüsü.
18. [18_slot_index_duzenleme.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/18_slot_index_duzenleme.md): HashMap yerine O(1) slot vektör optimizasyonu.
19. [19_short_circuit_mantigi.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/19_short_circuit_mantigi.md): Mantıksal operatörlerde kısa devre tasarımı.
20. [20_escape_dizileri.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/20_escape_dizileri.md): Metinlerde kaçış karakterleri desteği.
21. [21_ariadne_ve_hata_teshisi.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/21_ariadne_ve_hata_teshisi.md): Görsel, konum işaretçili hata raporlama sistemi.
22. [22_golden_file_testler.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/22_golden_file_testler.md): Golden stdout tabanlı entegrasyon test harness'ı.
23. [23_cicd_pipeline.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/23_cicd_pipeline.md): GitHub Actions test, clippy ve fmt otomasyonu.
24. [24_c_codegen_makine_kodu.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/24_c_codegen_makine_kodu.md): Transpile ve yerel derleme altyapısı.
25. [25_proje_manifestosu_tilk_toml.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/25_proje_manifestosu_tilk_toml.md): `tilk.toml` bildirim yapılandırması.
26. [26_cli_arac_zinciri.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/26_cli_arac_zinciri.md): `oz-cli` araç setinin komutları ve kullanımı.
27. [27_web_playground_arayuzu.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/27_web_playground_arayuzu.md): Web arayüzü ve tarayıcı içi yorumlama motoru.
28. [28_unicode_ve_nfc_normalizasyonu.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/28_unicode_ve_nfc_normalizasyonu.md): Unicode normalizasyon ve Türkçe I/İ harf çözümleri.
29. [29_gelecek_yol_haritasi.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/29_gelecek_yol_haritasi.md): Native derleyici ve 500x ölçeklendirme planı.
30. [30_ornek_uygulamalar_katalogu.md](file:///c:/Users/HP/Desktop/Tilk/kullanim_kilavuzu/30_ornek_uygulamalar_katalogu.md): Dilde yazılmış zengin kod örnekleri.

---

## 9. GELECEK YOL HARİTASI

- **Cranelift/LLVM Arka Ucu**: Bayt kodu yorumlaması yerine doğrudan yerel makine kodu (native binary) üreten AOT/JIT derleyici altyapısı.
- **Language Server Protocol (LSP)**: Tower-LSP tabanlı, VS Code ve benzeri editörlerde anlık hata gösterimi ve otomatik tamamlama.
- **Gelişmiş Tip Sistemi**: Türkçe iyelik sonekleriyle tip eşleşmeleri ve generikler.
