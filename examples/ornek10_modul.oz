// Örnek 10: Modül Sistemi
// TİLK'te dahil_et ile modül yükleme

// Standart kütüphane modülleri
dahil_et("std::matematik");
dahil_et("std::zaman");

// Standart kütüphane fonksiyonlarını namespace ile kullanma
kok_degeri = std::matematik::karekok(144);
yazdır("√144 =", kok_degeri);

ust_degeri = std::matematik::ust(2, 8);
yazdır("2^8 =", ust_degeri);

mutlak_degeri = std::matematik::mutlak(0 - 100);
yazdır("|-100| =", mutlak_degeri);

// Zaman modülü
simdi_zaman = std::zaman::simdi();
yazdır("Şu anki zaman (ms):", simdi_zaman);
