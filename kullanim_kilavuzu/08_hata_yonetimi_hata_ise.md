# Hata Yönetimi: hata_fırlat ve hata_ise

TİLK dilinde çalışma zamanı hataları kontrol altında tutulabilir.

## Hata Fırlatma
Bir işlem sırasında hata durumunu belirtmek için `hata_fırlat` kullanılır:
```oz
işlev böl(a, b) {
    b == 0 ise {
        hata_fırlat("Sıfıra bölme hatası!");
    }
    döndür a / b;
}
```

## Hata Yakalama (`hata_ise`)
Bir ifadenin çalışması sırasında hata oluşursa alternatif bir blok (`hata_ise`) çalıştırılabilir:
```oz
sonuç = böl(10, 0) hata_ise {
    yazdır("Hata yakalandı:", hata_mesajı);
    döndür 0;
};
```
