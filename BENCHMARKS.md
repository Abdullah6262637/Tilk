# TİLK Benchmark Altyapısı

Bu doküman, TİLK programlama dilinin performans testleri (benchmark) stratejisini ve `criterion` altyapısını açıklar.

## Hedefler

- TİLK Sanal Makinesinin (VM) işlemci (CPU) sürelerini analiz etmek.
- Lexer, Parser ve Typechecker gibi aşamaların süresini mikro-saniyeler cinsinden profilleyerek darboğazları tespit etmek.
- Regresyon (performans düşüşü) durumlarını hızlıca fark etmek (örneğin yeni bir özellik eklendiğinde `cargo bench` ile kıyaslama yapmak).

## Araçlar

Rust ekosisteminin standart endüstri çözümü olan **Criterion** (criterion.rs) kullanılmıştır. Criterion, istatistiksel açıdan anlamlı sonuçlar üretir, aykırı (outlier) değerleri dışlar ve geçmiş koşularla karşılaştırma yaparak (baseline) performans değişimini yüzde olarak raporlar.

## Çalıştırılan Benchmark'lar

Şu anki hedeflenen benchmark suite'leri şunlardır:

1. **`vm_benchmark`:** VM'in temel talimat (instruction) işletim süreleri.
   - Matematiksel işlemler (1'den 1 milyon'a kadar toplama vb.)
   - Fonksiyon çağrıları (Özyinelemeli / Recursive Fibonacci)
   - Bellek ayırma işlemleri (Büyük diziler oluşturma, iterasyon)

2. **`parser_benchmark`:**
   - Çok satırlı (10.000 satır) kaynak kodlarının parçalanma (parsing) hızları.

## Kullanım

Tüm benchmark'ları çalıştırmak için (bu işlem sadece `release` modunda yapılmalıdır):

```bash
cargo bench
```

Sadece belirli bir benchmark'ı (örneğin VM) çalıştırmak için:

```bash
cargo bench -p oz-vm
```

Benchmark sonuçları `target/criterion/` dizini altında HTML raporları ve JSON verileri olarak saklanır. Ayrıntılı raporlar için `target/criterion/report/index.html` dosyasını tarayıcınızda açabilirsiniz.
