// Standart Dosya Modülü
dahil_et("std::sonuc");

işlev oku(yol) {
    r = dosya_oku(yol) hata_ise {
        döndür std::sonuc::hata("Okuma hatası");
    };
    döndür std::sonuc::basarili(r);
}

işlev yaz(yol, icerik) {
    r = dosya_yaz(yol, icerik) hata_ise {
        döndür std::sonuc::hata("Yazma hatası");
    };
    döndür std::sonuc::basarili(boş);
}

işlev sil(yol) {
    r = dosya_sil(yol) hata_ise {
        döndür std::sonuc::hata("Silme hatası");
    };
    döndür std::sonuc::basarili(boş);
}
