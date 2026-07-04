# C Kod Üretimi ve Makine Kodu (AOT) Derleme

TİLK kodları doğrudan yerel makine koduna (C transpile) derlenebilir.

## Transpile Süreci
`oz-cli` derleme aracı, AST'yi optimize edilmiş standart bir C koduna dönüştürür. Ardından yerel makinedeki C derleyicisi (gcc, clang, msvc) ile doğrudan native `.exe` veya ELF formatına dönüştürür.
