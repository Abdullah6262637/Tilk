# Lokal Değişken Slot-Index Optimizasyonu

VM performansını artırmak amacıyla lokal değişken erişimleri optimize edilmiştir.

## Optimizasyon Detayları
- **Eski Yapı**: Her yerel değişken string anahtarlı bir HashMap'te aranıyordu.
- **Yeni Yapı**: Yerel değişkenler derleme zamanında `u16` tipinde indekslere çözümlenir ve Frame içindeki `slots` dizisinde O(1) hızında erişilir.
