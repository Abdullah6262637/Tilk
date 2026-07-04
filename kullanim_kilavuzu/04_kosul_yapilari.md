# Koşul Yapıları: ise, se ve değilse

TİLK dilinde karar mekanizmaları Türkçenin şart kipi ekleri (`-ise` / `-se`) kullanılarak inşa edilir.

## Temel Kullanım
Koşul ifadesinden hemen sonra `ise` veya `se` anahtar kelimesi gelir:
```oz
yaş = 20;
yaş >= 18 ise {
    yazdır("Ehliyet alabilirsiniz.");
} değilse {
    yazdır("Yaşınız yetersiz.");
}
```

## Çoklu Koşullar
Koşullar `değilse` blokları ile zincirlenebilir:
```oz
notu = 85;
notu >= 90 ise {
    yazdır("Harika!");
} notu >= 70 ise {
    yazdır("Başarılı.");
} değilse {
    yazdır("Kaldı.");
}
```
