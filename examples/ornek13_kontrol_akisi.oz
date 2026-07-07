// kır ve devam_et testi
toplam = 0;
i, 1'den 10'a dek artarak {
    i == 5 ise { kır; }
    toplam = toplam + i;
}
yazdır("kır sonucu:", toplam);

cift_toplam = 0;
j, 1'den 10'a dek artarak {
    j % 2 == 1 ise { devam_et; }
    cift_toplam = cift_toplam + j;
}
yazdır("devam_et sonucu:", cift_toplam);
