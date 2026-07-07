# 10 Dakikada TİLK Programlama Dili

TİLK, Türkçenin dilbilgisi yapısına (sondan eklemeli, Özne-Nesne-Yüklem dizilimi) sadık kalmayı hedefleyen dinamik tipli, yenilikçi bir programlama dilidir. Bu rehberde TİLK ile kod yazmanın temellerini 10 dakika gibi kısa bir sürede öğreneceksiniz.

## 1. Temel Değişkenler ve Tipler

TİLK dilinde değişken tanımlamak son derece basittir. Değişkenler varsayılan olarak değişebilirdir (mutable).

```oz
isim = "Ahmet";
yas = 28;
aktif = doğru;

// Değişkenin değerini güncelleme
yas = 29;
```

TİLK'te sayılar, metinler (string) ve mantıksal değerler (`doğru` ve `yanlış`) bulunur. Eğer bir değer atanmamışsa o değer `boş` (null) olarak ifade edilir.

## 2. Ekrana Yazdırma ve Yorum Satırları

Ekrana metin veya değişken yazdırmak için `yazdır()` gömülü işlevini kullanırız.

```oz
// Bu tek satırlık bir yorumdur
/* 
   Bu ise çok satırlı
   bir yorum bloğudur
*/
yazdır("Merhaba, TİLK Dünyası!");
yazdır("Ahmet'in yaşı:", yas);
```

### 2.1 Metin Birleştirme (String Interpolation)
Metinlerin içine değişken yerleştirmek için `f"..."` formatlı stringleri (f-string) kullanabilirsiniz. Değişkenler `{}` içine yazılır:

```oz
yazdır(f"Benim adım {isim} ve {yas} yaşındayım.");
```

## 3. Matematiksel İşlemler

Standart matematiksel operatörlerin tümü kullanılabilir:

```oz
a = 10;
b = 3;

toplam = a + b;
kalan = a % b; // Mod alma işlemi
us = a ** 2;   // Üs alma işlemi
```

## 4. Koşullu İfadeler (If-Else)

TİLK, Türkçenin akışına uymak için koşulları bir önermenin ardından gelen `ise` takısıyla (keyword) kontrol eder.

```oz
puan = 85;

puan > 80 ise {
    yazdır("Başarılı, tebrikler!");
} değilse {
    yazdır("Biraz daha çalışmalısın.");
}
```

## 5. Döngüler

TİLK'te döngüler `iken` (while), sayı aralıkları (for) veya koleksiyon elemanları (for-each) üzerinde dolaşarak oluşturulabilir.

### `iken` Döngüsü (While)
```oz
sayac = 0;
sayac < 5 iken {
    yazdır(sayac);
    sayac = sayac + 1;
}
```

### Aralık Döngüsü (For)
Türkçenin `...den ...e dek` kalıbını kullanarak belirli aralıklarda dolaşabilirsiniz. Artış yönünü (`artarak` / `azalarak`) belirtebilirsiniz:

```oz
i, 1'den 5'e dek artarak {
    yazdır(i);
}
```

## 6. Diziler ve Haritalar (Koleksiyonlar)

Birden çok öğeyi tutmak için `[]` ile dizileri (Array) veya `{}` ile haritaları (Map) kullanabilirsiniz.

```oz
// Dizi (Liste)
meyveler = ["Elma", "Armut", "Muz"];
yazdır("İlk meyve:", meyveler[0]);

// Harita (Sözlük)
bilgiler = {
    "isim": "Zeynep",
    "sehir": "Ankara"
};
yazdır("Kişi:", bilgiler["isim"]);
```

Bir koleksiyon içindeki tüm elemanları sırayla işlemek için `her ... içinde` döngüsünü kullanın:

```oz
her meyve meyveler içinde {
    yazdır("Bugün yediğim meyve:", meyve);
}
```

## 7. İşlevler (Fonksiyonlar)

Kendi komutlarınızı bir araya getirmek için `işlev` kelimesini kullanırsınız. TİLK dilinde değer döndürmek için `döndür` anahtar kelimesi mevcuttur.

```oz
işlev topla(x, y) {
    sonuc = x + y;
    döndür sonuc;
}

cevap = topla(5, 7);
yazdır("5 ve 7'nin toplamı:", cevap);
```

## 8. Hata Yönetimi (Hata_Ise)

Programınızdaki hataların uygulamanın çökmesine neden olmaması için `hata_ise` bloğunu kullanabilirsiniz.

```oz
bolme_sonucu = (10 / 0) hata_ise {
    yazdır("Sıfıra bölme hatası oluştu!");
    döndür 0;
}
```

## 9. Asenkron (Arkaplan) İşlemler

Uzun süren işlemleri arkaplanda (başka bir görev olarak) başlatabilir ve mesajlaşma kanalları aracılığıyla bilgi alışverişi yapabilirsiniz.

```oz
k = kanal();

arkaplanda_çalıştır {
    // Uzun bir işlem...
    k.kanal_gönder("İşlem tamamlandı!");
}

yazdır("Arkaplan işleminden gelen sonuç:", k.kanal_al());
```

---
*Tebrikler, artık TİLK ile ilk programınızı yazmaya hazırsınız! Etkileşimli kabuğa girmek için `cargo run -p oz-cli -- repl` komutunu çalıştırabilirsiniz.*
