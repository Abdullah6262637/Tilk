# TİLK'e Katkıda Bulunma Rehberi 🤝

TİLK projesine katkıda bulunmak istediğiniz için teşekkürler! Bu belge, geliştirme ortamının kurulumunu ve katkı sürecini açıklar.

## 🛠️ Geliştirme Ortamı Kurulumu

### Gereksinimler
- **Rust** (stable, 1.75+): [rustup.rs](https://rustup.rs) üzerinden kurulum
- **Git**: Versiyon kontrol sistemi

### Projeyi Klonlama ve Derleme

```bash
git clone https://github.com/Abdullah6262637/Tilk.git
cd Tilk
cargo build
```

### Testleri Çalıştırma

```bash
cargo test --all
```

### Kod Kalitesi Kontrolleri

```bash
# Format kontrolü
cargo fmt --all --check

# Lint kontrolü
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 📋 Kod Standartları

1. **Formatlama**: `cargo fmt` kullanılmalıdır
2. **Lint**: `cargo clippy` uyarısız geçmelidir (`-D warnings`)
3. **Testler**: Yeni özellikler için test yazılmalıdır
4. **Türkçe**: Hata mesajları ve kullanıcıya dönük metinler Türkçe olmalıdır
5. **Yorumlar**: Kod yorumları anlaşılır ve kısa tutulmalıdır

---

## 🔀 Pull Request Süreci

1. `main` dalından yeni bir dal oluşturun:
   ```bash
   git checkout -b ozellik/yeni-ozellik-adi
   ```

2. Değişikliklerinizi yapın ve test edin:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all
   ```

3. Commit'lerinizi anlaşılır mesajlarla yapın:
   ```bash
   git commit -m "feat: yeni özellik açıklaması"
   ```

4. Dalınızı push'layın ve PR açın:
   ```bash
   git push origin ozellik/yeni-ozellik-adi
   ```

---

## 📂 Proje Yapısı

| Dizin | Açıklama |
|-------|----------|
| `oz-lexer/` | Sözcüksel analizci (Logos tabanlı) |
| `oz-parser/` | Ayrıştırıcı ve tip denetimi (Chumsky + HM) |
| `oz-interpreter/` | Ağaç yürüyüşlü yorumlayıcı |
| `oz-vm/` | Yığın tabanlı sanal makine |
| `oz-cli/` | Komut satırı aracı |
| `oz-lsp/` | LSP dil sunucusu |
| `examples/` | Örnek TİLK programları |
| `editors/` | Editör eklentileri |

---

## 🏷️ Commit Mesajı Formatı

- `feat:` — Yeni özellik
- `fix:` — Hata düzeltme
- `docs:` — Belgelendirme
- `refactor:` — Kod yeniden yapılandırma
- `test:` — Test ekleme/güncelleme
- `chore:` — Bakım işleri

---

## ❓ Sorularınız mı var?

GitHub Issues üzerinden soru sorabilir veya tartışma başlatabilirsiniz.
