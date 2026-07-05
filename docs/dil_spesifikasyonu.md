# TİLK Programlama Dili Spesifikasyonu
v0.1-alpha

TİLK, Türkçenin sondan eklemeli dilbilgisi yapısını temel alan, esnek, hızlı ve modüler bir programlama dilidir. Bu doküman, TİLK'in temel sözdizimini, yapılarını ve derleyici mimarisini açıklar.

## 1. Sözdizimi (Syntax)

TİLK dili, C-benzeri ancak Türkçe anahtar kelimelerin kullanıldığı özgün bir yapıya sahiptir.

### Değişken Tanımlama
Değişkenler `tanım` anahtar kelimesi ile tanımlanır. Tür çıkarımı (type inference) desteklenir. Sabitler için `sabit` kullanılır.
```tilk
tanım yas = 25;
sabit PI = 3.14159;
tanım isim = "Ahmet";
tanım liste = [1, 2, 3];
```

### Kontrol Yapıları

#### ise / değilse
```tilk
ise (yas > 18) {
    yazdir("Reşit");
} değilse {
    yazdir("Reşit değil");
}
```

#### iken (while)
```tilk
tanım i = 0;
iken (i < 10) {
    yazdir(i);
    i = i + 1;
}
```

#### her (foreach)
```tilk
tanım sayilar = [1, 2, 3];
her (sayi icinde sayilar) {
    yazdir(sayi);
}
```

#### den/e dek (for-loop)
```tilk
1'den 10'a dek {
    yazdir("Döngü");
}
```

### İşlevler (Fonksiyonlar)
```tilk
işlev topla(a, b) {
    döndür a + b;
}

tanım sonuc = topla(3, 5);
```

### Veri Yapıları
- **Diziler (Array):** `[1, 2, 3]`
- **Haritalar (Map):** `{"isim": "Ahmet", "yas": 25}`

### Hata Yönetimi
Hataları fırlatmak için `hata_fırlat` kullanılır. Hata yakalamak için `hata_ise` bloğu vardır.
```tilk
hata_ise {
    tanım icerik = dosya_oku("yok.txt");
} tamamlaninca (h) {
    yazdir(h);
}
```

### Asenkron İşlemler
Kanal (`kanal()`) veri yapısı ile asenkron işlemler arası mesajlaşma sağlanır. Yeni bir asenkron görev başlatmak için `asenkron` anahtar kelimesi kullanılır.
```tilk
tanım k = kanal();
asenkron {
    k <- "merhaba";
}
tanım mesaj = <-k;
```

## 2. Mimari

TİLK, klasik bir modern derleyici boru hattını (pipeline) izler ve Rust programlama dili ile yazılmıştır. Workspace modüler bir şekilde aşağıdaki bileşenlerden oluşur:

### Bileşenler
1. **oz-lexer:** Kaynak kodu sözcük (token) birimlerine ayırır.
2. **oz-parser:** Token dizilerini Anlamsal Sözdizim Ağacına (AST) çevirir.
3. **oz-typechecker:** AST üzerinde tip denetimi (type inference & validation) uygular.
4. **oz-interpreter:** AST'yi doğrudan yorumlayan Tree-Walk Interpreter (deneysel/eski).
5. **oz-vm:** Bytecode derleyici ve sanal makine (VM) içerir. Modern ve hızlı çalışma ortamıdır.
6. **oz-cli:** Komut satırı arayüzüdür. (Sözcüksel analiz, ağaç çıkarma, tip denetimi, C transpilation, kod yürütme, paket yükleme).
7. **oz-lsp:** Editörler için dil sunucusu protokolü (Language Server Protocol).

### C Backend Mimarisi
TİLK programları aynı zamanda doğrudan C diline derlenebilir. `oz-cli` içindeki `CCodegen` modülü, AST'yi alır ve yerel C koduna dönüştürür. 

TİLK C Backend iki ana dosyadan oluşur:
- `tilk_runtime.h`: TİLK değerlerinin (`TilkVal`) bellek düzenini ve sanal makine dışı C runtime fonksiyonlarını tanımlayan başlık dosyasıdır. Değerler Box/Tagged Union (`TilkType`) mimarisiyle sarmalanır.
- `tilk_runtime.c`: Çöp toplama gerektirmeyen (mevcut aşamada), basit referans sayımı veya kopyalama mantığı güden temel bellek yönetimi ve standart kütüphane fonksiyonlarını (örn. `yazdir`, `dosya_oku`, diziler, haritalar) gerçekler.

TİLK -> C -> Yerel Çalıştırılabilir (Executable) süreci `gcc` (veya `clang`) tarafından tamamlanır.

## 3. Paket Yönetimi
TİLK'in bağımlılıkları `tilk.toml` dosyasında tanımlanır.
```toml
[paket]
ad = "ornek_proje"
surum = "0.1.0"
giris = "ana.oz"

[bagimliliklar]
matematik = "1.0.0"
harici_git = { git = "https://github.com/user/repo.git", tag = "v1" }
```
`oz-cli yükle` (veya `cargo run -- yukle`) komutu çalıştırıldığında:
1. Paket deposu, yerel dizin veya Git üzerinden paketler klonlanır/indirilir.
2. Kodlar `kitaplik/` dizinine yerleştirilir.
3. Kodların değişmezliğini garanti altına almak için `tilk.lock` adında bir kilit dosyası ve md5/sha checksum değerleri oluşturulur.
