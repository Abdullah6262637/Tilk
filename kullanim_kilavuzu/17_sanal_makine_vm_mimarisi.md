# Sanal Makine (VM) Yığın Mimarisi

TİLK sanal makinesi (VM) yığın tabanlı çalışan yüksek performanslı bir motordur.

## Yığın (Stack) Mantığı
Tüm aritmetik ve mantıksal işlemler yığın tepesindeki değerleri pop ederek yürütülür ve sonuç tekrar yığına push edilir.

## Call Frame (Yürütme Çerçeveleri)
Her fonksiyon çağrısı yeni bir call frame oluşturur. Bu frame çağrı bittiğinde yığından temizlenir.
