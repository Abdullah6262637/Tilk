# Dosya Giriş/Çıkış (I/O) İşlemleri

TİLK dili dosya sistemine erişim için dahili fonksiyonlar sunar.

## Dosya Yazma
`dosya_yaz(yol, içerik)` fonksiyonu belirtilen yola dosya yazar:
```oz
dosya_yaz("test.txt", "TİLK dili dosya yazma testi.");
```

## Dosya Okuma
`dosya_oku(yol)` fonksiyonu dosya içeriğini metin olarak okur:
```oz
içerik = dosya_oku("test.txt");
yazdır(içerik);
```

## Dosya Silme
`dosya_sil(yol)` fonksiyonu dosyayı siler:
```oz
dosya_sil("test.txt");
```
