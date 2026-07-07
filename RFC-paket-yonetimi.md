# RFC: TİLK Paket Yönetimi ve Ekosistem (TİLK-PKG)

## 1. Özet
TİLK'in bir oyuncak dilden çıkıp gerçek dünya projelerinde kullanılabilmesi için standart, güvenilir ve kullanımı kolay bir paket yöneticisine ihtiyacı vardır. Bu RFC, projelerin yönetilmesi, dış modüllerin ağ üzerinden indirilip kurulması ve versiyonlanması süreçlerini tanımlar.

## 2. Tasarım İlkeleri
- **Basitlik:** Bağımlılıkların tanımlanması ve indirilmesi tamamen Türkçe sözdizimi ile yazılmış, okunabilir konfigürasyon dosyalarına dayanmalıdır.
- **Güvenilirlik:** Bağımlılıkların her makinede aynı şekilde derleneceğini garanti eden bir `lock` dosyası (tilk.lock) mekanizması zorunludur.
- **Yerel ve Uzak Paketler:** Hem yerel dosya sistemindeki dizinlere hem de uzak bir depo (registry) yapısına referans verilebilmelidir.

## 3. Yapılandırma Formatı (`tilk.toml`)

Mevcut Cargo benzeri TOML tabanlı paket yapılandırması temel alınacak ve tamamen Türkçe olacaktır.

```toml
[paket]
ad = "ornek_projem"
sürüm = "0.1.0"
yazar = "Geliştirici <dev@email.com>"
açıklama = "TİLK dilinde yazılmış örnek sunucu projesi"

[bağımlılıklar]
matematik = "1.0.2"               # Sürüm belirterek ağdan (registry) indirme
ag_istek = { sürüm = "2.1.0" }     # Detaylı sürüm
yerel_araclar = { yol = "../araclar" } # Yerel dizinden bağlama
```

## 4. `oz-cli` Entegrasyonu (Komutlar)

- `tilk yeni proje_adi` -> Yeni proje dizini ve temel `tilk.toml` dosyasını oluşturur.
- `tilk yükle` (veya `tilk indir`) -> `tilk.toml` dosyasını okur, paketleri uzak depodan `kitaplik/` klasörüne kopyalar ve `tilk.lock` dosyasını günceller.
- `tilk yayınla` -> Paketi sıkıştırıp genel registry sunucusuna yükler.

## 5. Merkezi Registry (Depo) Mimarisi
İlerleyen süreçte Node.js'in `npm`'i veya Rust'ın `crates.io`'su gibi merkezi bir sunucu yazılması planlanmaktadır. Bu sunucu temel bir REST API aracılığıyla paketlerin tarball (`.tar.gz`) sürümlerini sunacaktır. İlk aşamada doğrudan Github repo URL'lerinden indirme (git url bağımlılığı) desteklenebilir.

## 6. Güvenlik
Kötü niyetli kodlara (supply chain attacks) karşı:
- `tilk.lock` dosyası içinde paket içeriklerinin SHA-256 hash (sağlama) değerleri tutulacaktır.
- İndirilen paketin hash değeri eşleşmezse indirme/kurulum reddedilecektir.
