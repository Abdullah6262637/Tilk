# Değişken Tanımlama ve Veri Tipleri

TİLK dilinde değişkenler dinamik tiplidir ancak statik tip kontrol mekanizması ile derleme zamanında doğrulanırlar.

## Değişken Tanımlama
Bir değişkene değer atandığı an tanımlanmış olur:
```oz
sayı = 42;
ad = "Tilk";
durum = doğru;
değer = boş;
```

## Veri Tipleri
1. **Sayı (`Number`)**: 64-bit kayan noktalı sayılardır (örn: `10`, `3.14`).
2. **Metin (`String`)**: Çift tırnakla çevrelenen karakter dizileridir (örn: `"Merhaba"`).
3. **Boolean**: Mantıksal `doğru` ve `yanlış` değerleridir.
4. **Boş (`Bos`)**: Değersizliği ifade eden `boş` sabitidir.
5. **Dizi (`Array`)**: `[1, 2, 3]` şeklinde tanımlanan dinamik listelerdir.
6. **Harita (`Map`)**: `{"anahtar": "değer"}` şeklindeki key-value koleksiyonlarıdır.
