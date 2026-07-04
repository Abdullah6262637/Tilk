# ÖZGÜN Programlama Dili: Hiper-Detaylı Teknik Kılavuz ve Belgelendirme 🚀

ÖZGÜN (`.oz`), Türkçenin sondan eklemeli (aglutinative) dilbilgisi yapısını ve kelime türetme mantığını doğrudan kontrol mekanizması olarak kullanan, başka hiçbir programlama dilinin doğrudan kelime çevirisi olmayan, yüksek performanslı ve modern bir sistem/uygulama programlama dilidir.

Bu belge; dilin tasarım felsefesinden, sözdizimi (syntax) kurallarına, derleyici arka uç (compiler backend) mimarisinden sanal makine (VM) yığın yönetimine kadar her detayını derinlemesine açıklayan hiper-detaylı teknik kılavuzdur.

---

## BÖLÜM 1: DİL TASARIM FELSEFESİ VE DİLBİLİMSEL ÇEKİRDEK

### 1.1 "Kelime Çevirisi" Yaklaşımının Reddi
Bugüne kadar geliştirilen Türkçe programlama dili denemelerinin (örneğin TPD, Karamel vb.) neredeyse tamamı, mevcut İngilizce tabanlı dillerin (C/C++, Java, Python, JavaScript) anahtar kelimelerinin Türkçe kelimelerle yer değiştirilmesinden ibarettir:
- İngilizce: `if (x > 5) { return true; }`
- Sahte Türkçe: `eğer (x > 5) { döndür doğru; }`

Bu yaklaşım, dilin sadece "sözcük düzeyinde yerelleştirilmesi" (localization) egzersizidir. Türkçenin doğal düşünce yapısı ve dilbilgisi kuralları bu sözdiziminde hayat bulamaz; cümle kurgusu hala İngilizce (SVO - Özne, Yüklem, Nesne) olarak kalır.

ÖZGÜN, bu kısıtlamayı aşmak amacıyla **sondan eklemeli dilbilgisi kurallarını ve hal eklerini** dilin kontrol yapılarının kendisi haline getirmiştir.

### 1.2 Türkçe Ek Kurgusu ve Tümce Yapısı
Türkçe, sondan eklemeli bir dildir. Bir kök kelimeye getirilen ekler, kelimenin zamanını, yönünü, kipini ve işlevini belirler. ÖZGÜN, bu mantığı aşağıdaki kurallarla programlama dünyasına aktarır:

#### 1. Koşul Yapısı (`-ise / -se`)
Türkçede şart kipi `-ise` veya `-se` ekiyle kurulur ("yağmur yağıyor **ise** şemsiye al", "sayı 5'ten büyük**se** yazdır"). ÖZGÜN'de bu kural birebir uygulanır:
```oz
sayı > 5 ise {
    yazdır("Sayı 5'ten büyüktür.");
} değilse {
    yazdır("Küçüktür.");
}
```

#### 2. Zarf-Fiil Döngü Yapısı (`-iken`)
Bir eylemin devam ettiği süreyi belirtmek için kullanılan `-iken` zarf-fiil eki, ÖZGÜN'de koşullu döngüyü (`while`) oluşturur:
```oz
sayaç <= 10 iken {
    yazdır(sayaç);
    sayaç = sayaç + 1;
}
```

#### 3. Yönelme ve Ayrılma Ekli Aralık Döngüleri (`-den ... -e dek`)
Sayaçlı döngüler (`for`) Türkçedeki yönelme (`-e / -a`) ve ayrılma (`-den / -dan`) ekleri ile `dek` edatı birleştirilerek kurgulanmıştır. Kesme işaretleri (`'`) lexer seviyesinde yoksayılır, böylece doğal yazım korunur:
```oz
// 'den ve 'e ekleri ile aralık belirtilir
i, 1'den 10'a dek artarak {
    yazdır(i);
}

// Azalan döngüler için 'azalarak' soneki kullanılır
j, 10'dan 1'e dek azalarak {
    yazdır(j);
}
```

#### 4. İşlev Tanımlamaları
İşlevler (fonksiyonlar) `işlev` anahtar kelimesi ile tanımlanır ve gövde bloğu içerisindeki kodları yürütür:
```oz
işlev topla(a, b) {
    döndür a + b;
}
```

### 1.3 Unicode Normalizasyonu ve "Türkçe I/İ" Problemi
Derleyicilerin en sık karşılaştığı kararlılık sorunlarından biri Unicode karakterlerin büyük/küçük harf dönüşümüdür. Türkçe karakter setindeki noktalı `İ` harfinin küçüğü `i`, noktasız `I` harfinin küçüğü ise `ı` karakteridir. Standart Unicode kütüphaneleri (örneğin ASCII tabanlı alt yapılar) `İ` karakterini küçük harfe çevirdiğinde noktasız `i` yapamaz veya hata üretir. Bu da değişken karşılaştırmalarında kararsızlıklara yol açar.

ÖZGÜN derleyicisi, locale-independent (yerel ayarlardan bağımsız) özel bir normalizasyon fonksiyonu (`turkish_lowercase`) barındırır:
```rust
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
```
Bu fonksiyon, işletim sistemi dili veya terminal kodlaması ne olursa olsun, tüm ÖZGÜN derleyicilerinin ve sanal makinelerinin değişken isimlerini tam olarak aynı şekilde hashlemesini ve eşleştirmesini garanti altına alır.

---

## BÖLÜM 2: RESMÎ DİL SÖZDİZİMİ (EBNF GRAMMAR SPECIFICATION)

ÖZGÜN dilinin resmî EBNF (Extended Backus-Naur Form) sözdizimi tanımlaması aşağıdaki gibidir:

```ebnf
Program         ::= Statement*

Statement       ::= VarDecl
                  | Assignment
                  | IfStatement
                  | WhileStatement
                  | ForStatement
                  | FnDeclaration
                  | ReturnStatement
                  | ExprStatement

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

Primary         ::= Identifier
                  | Number
                  | String
                  | Boolean
                  | "boş"
                  | CallExpr
                  | "(" Expr ")"

CallExpr        ::= Identifier "(" ArgList? ")"

ParamList       ::= Identifier ( "," Identifier )*
ArgList         ::= Expr ( "," Expr )*

Boolean         ::= "doğru" | "yanlış"
Number          ::= [0-9]+ ( "." [0-9]+ )?
String          ::= '"' [^"\\]* '"'
Identifier      ::= [a-zA-ZçğıişöüÇĞIİŞÖÜ_] [a-zA-Z0-9çğıişöüÇĞIİŞÖÜ_]*
```

---

## BÖLÜM 3: DERLEYİCİ ARKA UÇ VE ÇALIŞTIRMA MİMARİSİ

ÖZGÜN, kodun analiz edilmesinden çalıştırılmasına kadar 5 aşamalı bir boru hattı (pipeline) kullanır:

```
[Kaynak Kod (.oz)] -> (Lexer) -> [Token Listesi] -> (Parser) -> [AST] -> (Compiler) -> [Bytecode] -> (VM) -> [Çalıştırma]
```

### 3.1 Lexer (`oz-lexer` Modülü)
Sözcüksel analiz aşaması, Rust ekosisteminin en hızlı derleme zamanı DFA (Deterministic Finite Automaton) üreteci olan `logos` kütüphanesini kullanır.

#### Sonek Kesme İşareti (`'`) Yoksayma Stratejisi
Türkçedeki ekler kelimelere kesme işaretiyle bağlanır (`10'a`, `5'ten`, `limit'e`). Bu durumun lexer seviyesinde karmaşıklığa yol açmaması için kesme işareti boşluk karakterleri ile birlikte **yoksayılan karakterler (skip)** sınıfına dahil edilmiştir:
```rust
#[logos(skip r"[ \t\n\f']+")]
```
Bu sayede:
- `10'a` girdisi lexer tarafından ardışık iki bağımsız token olarak okunur: `Token::Number("10")` ve `Token::Identifier("a")`.
- `5'ten` girdisi `Token::Number("5")` ve `Token::Identifier("ten")` olarak okunur.
Bu yaklaşım hem dilin sözdizimini parser seviyesinde aşırı basitleştirir hem de kullanıcının doğal Türkçe imla kurallarıyla kod yazmasına olanak tanır.

### 3.2 Parser (`oz-parser` Modülü)
ÖZGÜN parser'ı, güçlü bir parser kombinatörü olan `chumsky` kütüphanesini kullanır. Chumsky, sözdizimi hatalarında durmak yerine hatayı izole edip parse işlemine devam edebilen (error recovery) PEG-benzeri bir altyapı sunar.

#### Soneklerin Dinamik Çözümlenmesi (Contextual Suffixes)
Çakışmaları önlemek için dilbilgisi ekleri (`ise`, `iken`, `dan/den`, `a/e/ya/ye`, `artarak`, `azalarak`) lexer seviyesinde katı anahtar kelimeler (keywords) olarak tanımlanmamıştır. Eğer bu kelimeler token seviyesinde kilitlenseydi, geliştirici `a`, `e` veya `se` isimli bir değişken tanımlayamazdı (`a = 15;` yazıldığında lexer `a` karakterini `Token::AE` olarak algılar ve derleme hatası verirdi).

Bu sorunu çözmek için sonekler parser seviyesinde **dinamik tanımlayıcı filtrelemesi** ile çözümlenir:
```rust
fn suffix_parser(values: &'static [&'static str]) -> impl Parser<Token, String, Error = Simple<Token>> + Clone {
    ident_parser().try_map(move |s, span| {
        if values.contains(&s.as_str()) {
            Ok(s)
        } else {
            Err(Simple::custom(span, "Beklenen ek bulunamadı"))
        }
    })
}
```
Bu sayede:
- `a = 15;` satırındaki `a`, standart bir tanımlayıcı (`Identifier`) olarak çözümlenir.
- `i, 1'den 10'a dek` döngüsündeki `a` (kesme işareti atılınca `a` olarak kalan yönelme eki), `suffix_parser(&["a", "e", "ya", "ye"])` filtresine girerek döngünün yönelme eki olarak kabul edilir.

---

## BÖLÜM 4: BAYT KODU DERLEYİCİSİ VE YIĞIN TABANLI SANAL MAKİNE (VM)

ÖZGÜN, Faz 4 itibarıyla AST'yi doğrusal talimatlara derleyip bunları kendi yığın tabanlı Sanal Makinesinde (VM) çalıştıracak şekilde optimize edilmiştir.

### 4.1 VM Talimat Kümesi (Instruction Set)
Sanal makine, aşağıdaki minimal ve optimize edilmiş opkodları (Opcode) işler:

```rust
pub enum Instruction {
    Constant(Val),     // Yığına sabit değer yükler (Sayı, Metin, Boolean, Boş)
    Load(String),      // Belirtilen değişkenin değerini yığına yükler (local/global)
    Store(String),     // Yığının tepesindeki değeri değişkene kaydeder
    Pop,               // Yığının tepesindeki değeri atar
    Add, Sub, Mul, Div, Mod, // Aritmetik işlemler
    Eq, Ne, Lt, Gt, Le, Ge,  // Karşılaştırma işlemleri
    And, Or,           // Mantıksal işlemler
    Jump(usize),       // Program sayacını (IP) doğrudan hedef adrese taşır
    JumpIfFalse(usize),// Yığının tepesindeki değer yanlış (false) ise hedefe dallanır
    Call(usize),       // Yığındaki argüman sayısı ile fonksiyonu çağırır
    Return,            // Fonksiyon çağrısından geri döner
}
```

### 4.2 Yığın ve Çerçeve (Stack & Call Frame) Yönetimi
VM, her fonksiyon çağrısı için yeni bir yürütme çerçevesi (`Frame`) oluşturur. Bu çerçeveler çağrı yığınında (call stack) tutulur:

```rust
struct Frame {
    return_address: usize,          // Fonksiyon bittiğinde dönülecek program sayacı (IP)
    locals: HashMap<String, Val>,  // Fonksiyon içi lokal değişken tablosu
}
```

#### Fonksiyon Çağrı Algoritması:
1. `Call(arg_count)` talimatı geldiğinde:
   - Yığından çağrılacak işlev (`Val::Function`) pop edilir.
   - Yığından sırasıyla argümanlar pop edilerek yeni oluşturulan çerçevenin `locals` haritasına parametre adlarıyla eşleştirilerek yazılır.
   - Mevcut `ip` (Instruction Pointer) adresi `return_address` olarak kaydedilerek yeni çerçeve `frames` yığınına push edilir.
   - `ip` değeri fonksiyonun giriş adresi olan `entry_ip`'ye setlenir.
2. `Return` talimatı geldiğinde:
   - Yığının tepesindeki dönüş değeri pop edilir.
   - Aktif çerçeve `frames` yığınından pop edilerek kaldırılır.
   - `ip` program sayacı, çerçevenin `return_address` değerine geri döndürülür.
   - Dönüş değeri tekrar yığına push edilir.

---

## BÖLÜM 5: PAKET YÖNETİCİSİ VE CLI KULLANIMI

`oz-cli` aracı, ÖZGÜN ekosisteminin resmi derleme, test ve paket yönetim aracıdır.

### 5.1 Proje Manifestosu (`ozgun.toml`)
Her ÖZGÜN projesinin kök dizininde projenin meta verilerini tutan bir `ozgun.toml` dosyası bulunur:
```toml
[paket]
ad = "hesaplama_modulu"
surum = "0.1.0"
giris = "kaynak/ana.oz"
```

### 5.2 CLI Komut Referansı

#### 1. Yeni Proje Oluşturma (`yeni`)
Belirtilen isimde şablon bir ÖZGÜN projesi ve klasör yapısı oluşturur:
```bash
cargo run --bin oz-cli -- yeni <proje_adı>
```
**Oluşan Yapı:**
```
proje_adı/
├── ozgun.toml
├── kaynak/
│   └── ana.oz
└── testler/
    └── test_ana.oz
```

#### 2. Proje Derleme (`derle`)
`ozgun.toml` içindeki `giris` dosyasını baz alarak tüm projeyi derler ve olası sözdizimi hatalarını raporlar:
```bash
cargo run --bin oz-cli -- derle
```

#### 3. Proje Çalıştırma (`calistir`)
Projeyi varsayılan olarak bayt kodu derleyicisi ile derleyip Sanal Makine (VM) üzerinde yürütür. Eğer parametre verilirse doğrudan tek bir dosyayı çalıştırır:
```bash
# Projeyi ozgun.toml üzerinden çalıştırır
cargo run --bin oz-cli -- calistir

# Belirli bir dosyayı doğrudan çalıştırır
cargo run --bin oz-cli -- calistir <dosya_yolu.oz>
```

#### 4. Entegrasyon Testlerini Koşturma (`test`)
`testler/` dizini altındaki tüm `.oz` uzantılı test dosyalarını tarar, VM üzerinde çalıştırır ve testlerin geçip geçmediğini raporlar:
```bash
cargo run --bin oz-cli -- test
```

---

## BÖLÜM 6: ZENGİN GÖRSEL WEB PLAYGROUND

ÖZGÜN dili, herhangi bir derleyici kurulumu gerektirmeden doğrudan web tarayıcısında çalışabilen istemci taraflı (client-side) bir **Oyun Alanına (Playground)** sahiptir.

### 6.1 Tasarım Dili ve Temel Bileşenler
Playground, modern web tasarım trendlerine uygun olarak **koyu tema ve cammorfizmi (glassmorphism)** temel alarak tasarlanmıştır:
- **Renk Paleti:** Gece siyahı arka plan (`#0a0c10`), neon turkuaz (`#00e5ff`) ve neon turuncu (`#ff5722`) vurgular.
- **Yazı Tipleri:** Başlıklar ve metinler için premium `Outfit`, kod alanları için ise `Fira Code` monospace yazı tipi kullanılmıştır.
- **Dinamik Satır Numaraları:** Kod yazıldıkça otomatik güncellenen satır numarası sütunu.

### 6.2 Tarayıcı İçi Yorumlama Motoru
Playground, Rust derleyicimiz ile %100 uyumlu çalışan saf bir **JavaScript Lexer, Parser ve Interpreter** omurgası barındırır. Bu sayede tarayıcıda yazılan kod hiçbir sunucu çağrısı (backend API request) yapmadan anında istemci tarafında derlenip çalıştırılır.

#### Sunulan Görsel Paneller:
1. **Konsol Çıktısı (Console):** Kodun ürettiği standart çıktıları (`yazdır`) ve olası çalışma zamanı hatalarını anlık gösterir.
2. **AST Görselleştirici:** Kodun derleyici tarafından oluşturulan Soyut Sözdizimi Ağacı (AST) yapısını canlı olarak JSON formatında gösterir. Geliştiricinin dilin derleme mantığını anlamasını kolaylaştırır.
3. **Token Akış Tablosu:** Lexer aşamasında kodun hangi kelimelere (tokens) ayrıştırıldığını gösteren dinamik analiz tablosu.

---

## BÖLÜM 7: KAPSAMLI ÖRNEK PROGRAMLAR

### 7.1 Özyinelemeli (Recursive) Faktöriyel Hesaplama
Dildeki fonksiyon tanımlama, koşul yapıları, return deyimleri ve rekürsif çağrı mekanizmasını doğrular:
```oz
işlev faktöriyel(n) {
    n <= 1 ise {
        döndür 1;
    }
    döndür n * faktöriyel(n - 1);
}

limit = 5;
i, 1'den limit'e dek artarak {
    yazdır(faktöriyel(i));
}
```

### 7.2 Sayaçlı ve Koşullu Döngülerin Birlikte Kullanımı
Döngüler, zarf-fiiller ve aritmetik operasyonların doğruluğunu test eder:
```oz
sayaç = 1;
sayaç <= 3 iken {
    yazdır("sayaç değeri:", sayaç);
    sayaç = sayaç + 1;
}

i, 1'den 5'e dek artarak {
    yazdır("döngü i:", i);
}
```

### 7.3 Karmaşık Mantıksal ve Aritmetik İşlemler
Operatör önceliklerini (precedence) ve mantıksal VE/VEYA işlemlerinin doğruluğunu test eder:
```oz
a = 15;
b = 4;
toplam = a + b;
kalan = a % b;

doğru veya yanlış ve doğru ise {
    yazdır("Aritmetik Sonuçlar:");
    yazdır("Toplam:", toplam);
    yazdır("Modül Kalanı:", kalan);
}
```

---

## BÖLÜM 8: GELECEK YOL HARİTASI VE 500x ÖLÇEKLENDİRME PLANI

ÖZGÜN dilinin bir eğitim prototipinden, endüstriyel standartlarda bir genel amaçlı yazılım diline dönüşmesi için planlanan 500 kat büyüme stratejisi:

### 8.1 Güçlü Statik Tip Sistemi (Tip Çıkarımı)
- **Hindley-Milner Algoritması:** Geliştiricinin tip yazmasına gerek kalmadan derleme zamanında katı tip denetimi yapılması.
- **Koleksiyon Sonekleri:** Türkçe iyelik ekleriyle dizi erişimleri (örn. `dizi'nin 1'inci elemanı`).

### 8.2 Performans Odaklı Derleme Arka Uçları
- **Cranelift JIT:** Geliştirme modunda kodun anında makine koduna derlenerek gecikmesiz çalışması.
- **LLVM (`inkwell`):** Üretim modunda native `.exe` veya WebAssembly çıktılarının yüksek optimizasyonla (O3) derlenmesi.

### 8.3 Geliştirici Ekosistemi (LSP & Paket Deposu)
- **LSP (Language Server):** `tower-lsp` tabanlı dil sunucusu ile VS Code, JetBrains ve Neovim editörleri için anlık autocomplete ve syntax highlighting desteği.
- **Çevrimiçi Paket Kaydı:** `crates.io` benzeri merkezi bir kütüphane havuzunun kurulması ve `oz-cli` üzerinden bağımlılıkların yönetilmesi.
