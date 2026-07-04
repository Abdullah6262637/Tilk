# Diziler ve Haritalar

TİLK, dinamik koleksiyonlar olan dizileri (Arrays) ve haritaları (Maps) yerleşik olarak destekler.

## Diziler
Diziler köşeli parantez içinde tanımlanır:
```oz
liste = [10, 20, 30];
yazdır(liste[0]); // 10

// Eleman güncelleme
liste[1] = 99;

// Eleman ekleme ve boyut alma
ekle(liste, 40);
yazdır(boyut(liste)); // 4
```

## Haritalar
Anahtar-değer çiftlerinden oluşur:
```oz
sözlük = {"ad": "Tilk", "yas": 4};
yazdır(sözlük["ad"]); // Tilk

sözlük["yas"] = 5;
```
