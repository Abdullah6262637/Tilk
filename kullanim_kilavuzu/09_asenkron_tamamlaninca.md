# Asenkron Görevler: arkaplanda_çalıştır ve tamamlanınca

TİLK dili, arka planda çalışan görevleri (asynchronous tasks) yerleşik olarak destekler.

## Görev Başlatma
`arkaplanda_çalıştır` ile bir işlev asenkron bir göreve dönüştürülür:
```oz
işlev uzun_süren_iş() {
    uyku(1000); // 1 saniye bekle
    döndür 42;
}

görev = arkaplanda_çalıştır(uzun_süren_iş);
```

## Tamamlanınca Bloğu (`tamamlaninca`)
Görev bittiğinde elde edilen sonuç ile tetiklenen bloktur:
```oz
görev tamamlanınca {
    yazdır("Görev sonucu geldi:", sonuç);
}
```
