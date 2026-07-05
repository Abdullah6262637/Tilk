// Örnek 9: Matematik ve Zaman Fonksiyonları
// TİLK'te yerleşik matematik araçları

// Karekök hesaplama
karekok_16 = kök(16);
yazdır("√16 =", karekok_16);

karekok_25 = kök(25);
yazdır("√25 =", karekok_25);

// Üs alma
iki_uzeri_10 = üs(2, 10);
yazdır("2^10 =", iki_uzeri_10);

uc_uzeri_3 = üs(3, 3);
yazdır("3^3 =", uc_uzeri_3);

// Mutlak değer
mutlak_deger = mutlak(0 - 42);
yazdır("|-42| =", mutlak_deger);

// Zaman fonksiyonları
başlangıç = şimdi();
uyku(50);
bitiş = şimdi();
yazdır("Geçen süre (ms):", bitiş - başlangıç);

// Hata durumunda matematik
negatif_kok = kök(0 - 1) hata_ise {
    yazdır("Negatif sayının karekökü alınamaz!");
    0;
};
yazdır("Negatif kök sonucu:", negatif_kok);
