# TİLK Diline Giriş ve Tasarım Felsefesi

TİLK (Türkçe İşlevsel Logotetik Kod), Türkçenin sondan eklemeli (aglütinatif) dilbilimsel çekirdeğini doğrudan programlama mantığına entegre eden modern bir programlama dilidir.

## 1. Tasarım Felsefesi
Çoğu yerel programlama dili girişimi, İngilizce anahtar kelimelerin Türkçe karşılıklarıyla değiştirilmesinden (kelime çevirisinden) öteye gidememiştir. TİLK ise dilbilimsel kuralları sözdizimsel denetim yapılarına dönüştürür.

- **Sözcük Düzeyinde Değil, Dilbilgisi Düzeyinde Türkçe**: Koşullar (`ise/se`), döngüler (`iken`, `dek`) ve asenkron kalıplar (`tamamlanınca`) Türkçe tümce yapısına uygun olarak kurgulanmıştır.
- **Doğal Düşünce Akışı**: Türkçe düşünen bir yazılımcının mantık silsilesine en uygun sözdizimini sunar.

## 2. Dilbilimsel Ekler
Dildeki anahtar yapılar Türkçe ekler yardımıyla kurulur:
1. **Koşul**: `sayı > 5 ise { ... }`
2. **Döngü**: `sayaç < 10 iken { ... }`
3. **Aralık**: `i, 1'den 10'a dek artarak { ... }`
