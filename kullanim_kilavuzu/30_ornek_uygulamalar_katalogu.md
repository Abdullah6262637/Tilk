# Örnek Uygulamalar Kataloğu

TİLK dilinde yazılmış örnek uygulamalar.

## 1. Fibonacci
```oz
işlev fib(n) {
    n <= 1 ise { döndür n; }
    döndür fib(n - 1) + fib(n - 2);
}
yazdır(fib(10));
```

## 2. Faktöriyel
```oz
işlev fakt(n) {
    n <= 1 ise { döndür 1; }
    döndür n * fakt(n - 1);
}
yazdır(fakt(5));
```
