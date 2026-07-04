# Golden File Entegrasyon Testleri

TİLK test paketi kararlılığı garanti altına almak için golden file testleri içerir.

## Mantık
Örnek programların ürettiği standart çıktılar (`.stdout` uzantılı dosyalarda) kaydedilir ve testler sırasında VM'in çıktıları bu dosyalarla karşılaştırılır. Herhangi bir sapma testin kalmasına sebep olur.
