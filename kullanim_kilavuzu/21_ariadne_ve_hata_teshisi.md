# Ariadne ile Görsel Hata Teşhis Sistemi

TİLK, geliştirici dostu renkli hata teşhislerine sahiptir.

## Çalışma Prensibi
Hata oluştuğunda, hata yapan kod satırı terminale görsel olarak basılır ve hatanın yeri oklarla işaretlenir.

## Örnek Çıktı
```
Error: Sayı bekleniyordu
   ╭─[test.oz:1:13]
   │
 1 │ sayı = 5 + * 4 ;
   │             ┬  
   │             ╰── Dosya sonu
───╯
```
