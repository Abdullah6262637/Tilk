# Zaman Yönetimi: şimdi ve uyku

Zaman işlemleri ve yürütmeyi geciktirme fonksiyonları.

## Şimdiki Zaman (`şimdi`)
Unix zaman damgasını (milisaniye cinsinden) döner:
```oz
başlangıç = şimdi();
// işlemler...
bitiş = şimdi();
yazdır("Geçen süre:", bitiş - başlangıç, "ms");
```

## Yürütmeyi Duraklatma (`uyku`)
Belirtilen milisaniye kadar aktif thread'i uyutur:
```oz
yazdır("Bekleniyor...");
uyku(2000); // 2 saniye uyutur
yazdır("Devam ediyor.");
```
