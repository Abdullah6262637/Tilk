# ÖZGÜN Programlama Dili 🚀

ÖZGÜN, Türkçenin sondan eklemeli (aglutinative) dilbilgisi yapısına uygun olarak tasarlanmış, başka hiçbir dilin doğrudan kelime çevirisi olmayan, özgün sözdizimli ve modern altyapılı bir programlama dilidir. 

Proje, yüksek performans, bellek güvenliği ve esneklik sağlamak amacıyla sıfırdan **Rust** dili ile geliştirilmiş bir derleyici ve sanal makine (VM) çekirdeğine sahiptir.

---

## 💡 Dil Felsefesi ve Özgün Yapı

Çoğu yerelleştirilmiş dil denemesi, İngilizce anahtar kelimelerin (`if` $\to$ `eğer`, `for` $\to$ `için` gibi) birebir çevrilmesinden ibarettir. ÖZGÜN ise Türkçedeki **fiil çekimlerini ve hal eklerini** kontrol yapıları olarak kullanır:

- **Koşul Suffixleri (`-ise / -se`):** `koşul ise { ... } değilse { ... }`
- **Zarf-Fiil Döngüleri (`-iken`):** `sayaç < 5 iken { ... }`
- **Aralık Döngüleri (`-den ... -e dek`):** `i, 1'den 10'a dek artarak { ... }`
- **Tanımlamalar:** `işlev topla(a, b) { döndür a + b; }`
- **Türkçe Harf Normalizasyonu:** Derleyici seviyesinde locale-indepedent harf normalizasyonu sayesinde "Türkçe I/İ" harfi uyumsuzluk sorunları yaşanmaz.

---

## 🛠️ Proje Mimarisi (Rust Workspace)

Proje, modüler bir yapı sunan Cargo Workspace olarak kurgulanmıştır:

- **`oz-lexer`:** `logos` tabanlı, Unicode destekli hızlı sözcüksel analiz (tokenizer) motoru. Kesme işaretlerini (`'`) kaldırarak soneklerin parse edilmesini kolaylaştırır.
- **`oz-parser`:** `chumsky` tabanlı, hata geri kazanımlı parser. Sonekleri (`ise`, `iken`, `dan/den`, `a/e`, `artarak`) dinamik olarak çözümleyerek değişken isimleriyle çakışmalarını önler.
- **`oz-interpreter`:** Kapsam zincirlemesini (scope/environment) ve fonksiyon yürütmelerini kontrol eden AST tabanlı yorumlayıcı.
- **`oz-vm`:** Komut kodlarını (`Instruction`) doğrusal bayt koduna derleyen derleyici ve bu komutları koşturan hızlı, yığın tabanlı (stack-based) Sanal Makine.
- **`oz-cli`:** ÖZGÜN projelerini oluşturma, derleme, test etme ve çalıştırma işlemlerini tek elden yöneten komut satırı arayüzü ve paket yöneticisi.
- **`playground`:** Kurulum veya sunucu gerektirmeksizin dilin doğrudan tarayıcı üzerinden test edilebilmesini sağlayan, zengin görselliğe sahip Web Oyun Alanı.

---

## 🚀 Kurulum ve Çalıştırma

### Gereksinimler
- Bilgisayarınızda **Rust** ve **Cargo** araç zincirinin yüklü olması gerekmektedir.

### 1. Testleri Yürütme
Tüm modüllere ait test senaryolarını çalıştırmak için projenin ana dizininde:
```bash
cargo test
```

### 2. Yeni Bir Proje Oluşturma (Paket Yöneticisi)
CLI aracını kullanarak şablon bir ÖZGÜN projesi oluşturabilirsiniz:
```bash
cargo run --bin oz-cli -- yeni deneme_projesi
```
Bu komut, projeniz için `ozgun.toml` manifestosunu, kaynak kod şablonunu (`kaynak/ana.oz`) ve test şablonunu (`testler/test_ana.oz`) otomatik hazırlar.

### 3. Proje Yürütme (Sanal Makine Üzerinde)
Oluşturulan proje dizinine geçin ve projeyi doğrudan çalıştırın:
```bash
cd deneme_projesi
cargo run --manifest-path ../Cargo.toml --bin oz-cli -- calistir
```

### 4. Projeyi Derleme / Test Etme
Projeyi derleyip hata analizi yapmak veya testleri çalıştırmak için:
```bash
# Hata denetimi yapar
cargo run --manifest-path ../Cargo.toml --bin oz-cli -- derle

# testler/ klasöründeki test senaryolarını çalıştırır
cargo run --manifest-path ../Cargo.toml --bin oz-cli -- test
```

---

## 🌐 Web Oyun Alanı (Playground)

Herhangi bir kurulum yapmadan veya komut çalıştırmadan dili görsel olarak denemek için `playground/index.html` dosyasını tarayıcınızda açabilirsiniz:

👉 **[ÖZGÜN Playground Arayüzünü Aç (playground/index.html)](./playground/index.html)**

Playground ile:
- Hazır kod şablonlarını (Faktöriyel, Döngüler, Koşullar) tek tıkla yükleyip çalıştırabilirsiniz.
- Kodun arka planda nasıl parse edildiğini gösteren **Soyut Sözdizimi Ağacı (AST)** panelini inceleyebilirsiniz.
- Sözcüklerin analiz edildiği **Token Akış Tablosu**'nu görüntüleyebilirsiniz.

---

## 📜 Lisans

Bu proje **MIT Lisansı** ile lisanslanmıştır. Detaylar için [LICENSE](./LICENSE) dosyasına göz atabilirsiniz.
