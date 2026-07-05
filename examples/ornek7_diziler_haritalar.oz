// Örnek 7: Diziler ve Haritalar
// TİLK'te koleksiyon veri yapıları

// Dizi oluşturma ve işlemleri
meyveler = ["elma", "armut", "kiraz"];
yazdır("Meyveler:", meyveler);
yazdır("İlk meyve:", meyveler[0]);

// Eleman ekleme
ekle(meyveler, "portakal");
yazdır("Eleman sayısı:", boyut(meyveler));

// İyelik erişimi (Türkçe sözdizimi)
ikinci = meyveler'nin 1'inci elemanı;
yazdır("İkinci meyve:", ikinci);

// Dizi güncelleme
meyveler[0] = "çilek";
yazdır("Güncellenen ilk meyve:", meyveler[0]);

// Harita (Map) oluşturma
öğrenci = { "ad": "Ali", "yas": 20, "bölüm": "Bilgisayar" };
yazdır("Öğrenci adı:", öğrenci["ad"]);

// Harita güncelleme ve yeni alan ekleme
öğrenci["yas"] = 21;
öğrenci["not_ort"] = 3.5;
yazdır("Güncellenen yaş:", öğrenci["yas"]);
yazdır("Harita boyutu:", boyut(öğrenci));

// İyelik ile harita erişimi
ad = öğrenci'nin ad;
yazdır("İyelik ile ad:", ad);
