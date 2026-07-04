# Unicode ve NFC Normalizasyon Kuralları

Türkçe büyük/küçük harf dönüşümleri ve unicode normalizasyon kuralları.

## NFC Normalizasyonu
Lowersacing işlemleri sırasında unicode decomposed karakterlerin uyuşmazlık yaratmaması için tüm metinler `unicode-normalization` kütüphanesi ile NFC formatına dönüştürülür.
