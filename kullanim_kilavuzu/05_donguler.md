# Döngüler: iken ve dek Döngüleri

TİLK dilinde iki ana döngü çeşidi mevcuttur: koşullu döngüler (`iken`) ve aralık döngüleri (`dek`).

## 1. Koşullu Döngü (`iken`)
Belirli bir koşul doğru olduğu sürece çalışır:
```oz
sayaç = 1;
sayaç <= 5 iken {
    yazdır("Sayı:", sayaç);
    sayaç = sayaç + 1;
}
```

## 2. Aralık Döngüsü (`dek`)
Belirli bir aralıktaki sayıları yönelme ve ayrılma ekleri kullanarak döner:
```oz
// 1'den 5'e kadar artarak
i, 1'den 5'e dek artarak {
    yazdır(i);
}

// 5'ten 1'e kadar azalarak
j, 5'ten 1'e dek azalarak {
    yazdır(j);
}
```
