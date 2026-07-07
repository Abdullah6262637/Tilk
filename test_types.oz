işlev kimlik<T>(a: T): T {
    döndür a;
}

Sayi x = 5;
Metin y = "selam";

// kimlik(3)'ün türü Sayi olmalıdır. Eğer metne atamaya çalışırsak hata vermelidir.
Metin z = kimlik(3);
