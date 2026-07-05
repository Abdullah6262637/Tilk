// Örnek 8: Asenkron Programlama
// TİLK'te görev yönetimi ve kanallar

// Basit asenkron görev
işlev hesapla(x, y) {
    döndür x + y;
}

görev = arkaplanda_çalıştır(hesapla, 100, 200);

yakalanan = 0;
görev tamamlanınca {
    yakalanan = sonuç;
}
yazdır("Asenkron sonuç:", yakalanan);

// Kanal ile veri iletişimi
iletim = kanal();

işlev üretici(k) {
    k[0] = 42;
    k[0] = 99;
    k[0] = 7;
    döndür boş;
}

görev2 = arkaplanda_çalıştır(üretici, iletim);

// Kanaldan veri okuma
ilk = iletim[0];
ikinci = iletim[0];
üçüncü = iletim[0];

yazdır("Kanaldan okunan:", ilk, ikinci, üçüncü);
