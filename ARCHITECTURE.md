# TİLK Mimari ve Tasarım (Architecture)

TİLK, Türkçenin sondan eklemeli yapısına ve dilbilgisine sadık kalmak üzere özel olarak tasarlanmış dinamik tipli, sanal makine (VM) tabanlı bir programlama dilidir. Bu belge, TİLK projesinin modüler yapısını, bellek modelini ve eşzamanlılık (concurrency) yaklaşımını özetler.

## 1. Modüler Yapı

Proje, birbirinden bağımsız görevleri üstlenen alt paketlere (`crates`) bölünmüştür:

- **`oz-lexer` (Sözcüksel Analiz):** `logos` kütüphanesi üzerine inşa edilmiştir. Kaynak kodu, dilin dilbilgisine uygun yapısal parçalara (`Token`) ayırır. Türkçe karakter ve unicode büyük/küçük harf dönüşümleri bu aşamada özenle yönetilir.
- **`oz-parser` (Sözdizimsel Analiz):** `chumsky` kütüphanesi kullanılarak yazılmıştır. Jeneratör (combinator) mantığı ile Token dizisini AST'ye (Soyut Sözdizimi Ağacı) çevirir. `typechecker` modülünü de barındırır.
- **`oz-vm` (Sanal Makine ve Derleyici):** TİLK'in çalışma zamanı merkezidir. AST'yi bir bytecode setine derler ve yığın-tabanlı (stack-based) bir VM üzerinde koşturur.
- **`oz-interpreter` (Ağaç-Yürüyücü Yorumlayıcı):** Geliştirme, prototipleme ve hata ayıklama aşamaları için tasarlanmıştır. AST üzerinde doğrudan dolaşarak (tree-walk) kod çalıştırır. (VM'e kıyasla daha yavaştır.)
- **`oz-cli` (Komut Satırı Arayüzü):** Geliştiricinin TİLK projelerini çalıştırdığı, formatladığı (`fmt`), etkileşimli kabuğa (`repl`) eriştiği ana giriş noktasıdır.
- **`oz-lsp` (Dil Sunucusu Protokolü):** IDE ve metin editörleri (VSCode vb.) için sözdizimi renklendirme, hata vurgulama (diagnostics) gibi özellikler sağlar.

## 2. Bellek Modeli ve `Rc<RefCell<T>>` Analizi

TİLK dilinin mevcut bellek modeli, **basit çöp toplayıcısız (non-GC) referans sayımı** mekanizmasına dayanır. Geliştirme hızını artırmak ve dilin iç yapılarını esnek tutmak adına karmaşık (dinamik boyutlu) veri türleri için Rust'ın `Rc<RefCell<T>>` türleri tercih edilmiştir.

### `Val` Enum'u İçerisindeki Tahsisler

TİLK'in çalışma zamanında her değer `Val` enum'u ile ifade edilir. İlkel (primitive) türler (`Number`, `Boolean`, `Bos`) değeri doğrudan yığın (stack) üzerinde taşırken, bellekte büyüyebilen veya paylaşılan yapılar `Rc<RefCell>` ile sarılmıştır:

```rust
pub enum Val {
    Number(f64),
    String(String), // Rust'ın String'i Heap'tedir ancak mülkiyeti Val'a aittir.
    Boolean(bool),
    Bos,
    Array(Rc<RefCell<Vec<Val>>>),
    Map(Rc<RefCell<HashMap<String, Val>>>),
    Task(Rc<RefCell<TaskState>>),
    Channel(Rc<RefCell<VecDeque<Val>>>),
    // ...
}
```

#### Neden `Rc<RefCell>`?
1. **Paylaşımlı Sahiplik (Shared Ownership):** Bir dizi veya harita (map) fonksiyona geçirildiğinde kopyalanmak yerine referansıyla kopyalanır (`Rc`). Değişiklikler tüm referanslarda görünür.
2. **İçsel Değişebilirlik (Interior Mutability):** TİLK değişkenleri varsayılan olarak değişebilirdir. `RefCell`, VM çalışırken veri yapılarının güvenli bir şekilde güncellenmesine olanak tanır.

#### Performans Etkileri ve Gelecek (Optimizasyonlar)
- `Rc<RefCell>`, döngüsel referanslar (circular references) yaratıldığında bellek sızıntısına (memory leak) neden olabilir (çünkü GC yoktur).
- **Planlanan Geliştirme:** Gelecekte gerçek bir Çöp Toplayıcı (Garbage Collector) entegrasyonu (örneğin mark-and-sweep) veya nesne tahsis havuzu (Memory Arena) kullanımı performansı artırabilir. Ancak şu anki prototipleme safhasında `Rc<RefCell>` en verimli çözümdür.

## 3. Çalışma Zamanı (Runtime) Tasarımı

Sanal Makine (VM), bir talimat listesini (`Vec<Instruction>`) ardışık olarak işler.
- **Yığın (Stack):** Ara sonuçlar ve operasyon argümanları yığın üzerinde tutulur.
- **Global Ortam:** Değişkenler, bir `HashMap<String, Val>` içerisinde isimle saklanır ve çağrılır. (Burası, ileride yerel değişkenleri doğrudan array/stack slotlarına dönüştürerek büyük bir hız kazanabilir).

## 4. Eşzamanlılık (Concurrency) Hedefleri

Şu anda VM içerisindeki eşzamanlılık ilkel seviyede (`arkaplanda_çalıştır`, `kanal`) olup ilerleyen fazlarda Rust'ın asenkron (`tokio`) veya iş parçacığı (thread) altyapısıyla güçlendirilecektir. Detaylı eşzamanlılık tasarımı `RFC-concurrency.md` belgesinde açıklanmıştır.
