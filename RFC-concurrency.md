# RFC: TİLK Eşzamanlılık (Concurrency) ve Asenkron Model

## 1. Özet
TİLK'in şu anki sürümünde `arkaplanda_çalıştır` (Task) ve `kanal` (Channel) yapıları ile ilkel bir asenkron model prototipi oluşturulmuştur. Bu RFC, dilin üretim ortamına hazır (production-ready) bir eşzamanlılık modeline geçişi için Rust'ın `tokio` çalışma zamanını merkeze alan **"Yeşil İş Parçacıkları (Green Threads) ve Mesajlaşma"** mimarisini önermektedir.

## 2. Motivasyon
Türkçenin akıcı dilbilgisi kurallarına uyan bir dilin, Go ve Erlang gibi dillerde görülen "paylaşımsız durum ve mesajlaşma" (Share by communicating) prensiplerini benimsemesi hem güvenlik (Data Race önleme) hem de anlaşılabilirlik açısından en mantıklı yoldur.

## 3. Önerilen Tasarım

### 3.1. `arkaplanda_çalıştır` (Yeşil İş Parçacıkları - Green Threads)
TİLK Sanal Makinesi (VM), işletim sistemi iş parçacıklarını (OS Threads) bloklamak yerine her bir `arkaplanda_çalıştır` çağrısını bir **Tokio Task** olarak haritalayacaktır. 

```oz
görev = arkaplanda_çalıştır {
    uzun_süren_hesaplama();
}
```

- **İzolasyon:** Her görev kendi yığınına (stack) ve VM ortamına sahip olacaktır.
- **Paylaşımlı Durum Yok:** Görevler arası değişken paylaşımı (global mutable state) derleyici veya çalışma zamanı seviyesinde yasaklanacaktır.

### 3.2. Kanallar (Channels) ve Mesajlaşma
Görevler arası iletişim yalnızca Kanallar üzerinden yapılacaktır. 

```oz
mesaj_kanalı = kanal();

arkaplanda_çalıştır {
    mesaj_kanalı.kanal_gönder("Merhaba Dünya");
}

gelen_mesaj = mesaj_kanalı.kanal_al();
```

- **Kanal Türleri:** Rust'ın `tokio::sync::mpsc` (Çoklu Üretici, Tekli Tüketici) kanalları kullanılarak TİLK'in `kanal()` fonksiyonu genişletilecektir.
- **Engelleme (Blocking):** `kanal_al()` çağrısı, veri gelene kadar geçerli görevi asenkron olarak bekletir, ancak işletim sistemi iş parçacığını dondurmaz.

### 3.3. Gelecek (Future) ve Bekleme (Await) Semantiği
Dilin ilerleyen versiyonlarında `bekle` (await) anahtar kelimesi eklenecektir.

```oz
veri = ağdan_veri_çek() bekle;
```

## 4. Teknik Zorluklar
- **`Rc<RefCell<T>>` Uyumluluğu:** Mevcut bellek modelimiz iş parçacıkları arası taşınabilir (Send) değildir. Tokio ile çalışabilmek için `Arc<RwLock<T>>` veya `Arc<Mutex<T>>` yapısına geçilmesi gerekmektedir.
- **VM Mimarisi Revizyonu:** VM döngüsü asenkron (async fn) hale getirilmelidir.

## 5. Uygulama Planı
1. `oz-vm`'deki `Val` tahsislerinin `Rc<RefCell>` yerine `Arc<RwLock>` kullanacak şekilde güncellenmesi.
2. `kanal()` yapısının `tokio::sync::mpsc` ile desteklenmesi.
3. `arkaplanda_çalıştır` komutunun `tokio::spawn` ile entegre edilmesi.
