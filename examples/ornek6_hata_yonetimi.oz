// Örnek 6: Hata Yönetimi
// TİLK'te hata yakalama ve fırlatma mekanizması

işlev güvenli_böl(a, b) {
    b == 0 ise {
        hata_fırlat("Sıfıra bölme yapılamaz!");
    }
    döndür a / b;
}

// Hata yakalama ile güvenli işlem
sonuç = güvenli_böl(10, 2) hata_ise {
    yazdır("Hata oluştu:", hata_mesajı);
    döndür 0;
};
yazdır("10 / 2 =", sonuç);

// Hata fırlatılan durumda yakalama
sonuç2 = güvenli_böl(10, 0) hata_ise {
    yazdır("Hata yakalandı:", hata_mesajı);
    döndür 0;
};
yazdır("10 / 0 sonucu:", sonuç2);

// İç içe hata yönetimi
işlev dosya_işle(yol) {
    içerik = dosya_oku(yol) hata_ise {
        yazdır("Dosya okunamadı:", hata_mesajı);
        döndür "varsayılan içerik";
    };
    döndür içerik;
}

veri = dosya_işle("olmayan_dosya.txt");
yazdır("Sonuç:", veri);
