# İşlevler ve Özyinelemeli (Recursive) Yapılar

İşlevler, tekrar kullanılabilir kod blokları tanımlamamızı sağlar.

## İşlev Tanımlama
`işlev` anahtar kelimesi ile bildirilir ve değer döndürmek için `döndür` kullanılır:
```oz
işlev çarp(a, b) {
    döndür a * b;
}

sonuç = çarp(5, 6);
yazdır(sonuç); // 30
```

## Özyinelemeli (Recursive) İşlevler
Bir işlev kendi kendini çağırabilir. Klasik fibonacci örneği:
```oz
işlev fib(n) {
    n <= 1 ise {
        döndür n;
    }
    döndür fib(n - 1) + fib(n - 2);
}

yazdır(fib(7)); // 13
```
