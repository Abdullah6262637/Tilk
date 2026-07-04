# CI/CD Pipeline Mimarisi

TİLK projeleri GitHub Actions entegrasyonu ile sürekli olarak test edilir.

## Süreçler
- **Biçimlendirme Kontrolü**: `cargo fmt --check` ile kod stili denetlenir.
- **Linter Kontrolü**: `cargo clippy` ile kod kalitesi denetlenir.
- **Birim ve Entegrasyon Testleri**: Tüm testler koşturulur.
