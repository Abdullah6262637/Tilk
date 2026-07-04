# Kısa Devre (Short-Circuit) Değerlendirmesi

Mantıksal `ve` ile `veya` operatörlerinin kısa devre tasarımı.

## Çalışma Mantığı
- **VE (`ve`)**: Sol taraf `yanlış` ise sağ taraf hiç çalıştırılmaz, doğrudan yanlış döner.
- **VEYA (`veya`)**: Sol taraf `doğru` ise sağ taraf hiç çalıştırılmaz, doğrudan doğru döner.
Bu mekanizma VM'e eklenen `JumpIfFalseKeep` ve `JumpIfTrueKeep` opkodları ile sağlanır.
