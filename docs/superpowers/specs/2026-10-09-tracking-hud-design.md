# ff-tracking: tracking HUD stil denemesi (tasarım)

Tarih: 2026-10-09
Durum: onaylandı (brainstorming), uygulama planı bekliyor

## Amaç

Michael Nowak'ın X paylaşımındaki görsel stili fframes ile yeniden üretmek ve geliştirmek.
Referans: <https://x.com/mnowakdesign/status/2108253918086176899> (yerel kopya `reference/`, git dışı).

Bu bir **stil denemesidir**. İçerik ikinci planda. Başarı ölçütü: görünüm referans kadar
"yakın çekim fiziksel ekran" hissi verir ve referansın ötesinde en az bir yeni fikir taşır.

## Referansın analizi

- 5,5 sn, 1920×1080, sesli.
- Arayüz metni çok yakından, eğik açıyla çekilmiş gibi. Güçlü alan derinliği (odak dışı bulanıklık).
- LED/CRT piksel ızgarası, parlama (bloom), renk kayması (chromatic aberration).
- Bilgisayarlı görü katmanı: taralı kutular, `x: 725 y: 475` etiketleri, kutuları bağlayan çizgiler.
- Yaklaşık 0,5 sn'de bir kesme. Her sahnede farklı palet: kâğıt beyazı, termal mor, siyah-beyaz,
  lacivert, neon yeşil.

## Kararlar

| Konu | Karar |
|---|---|
| Kullanım | Stil denemesi |
| Katmanlar | Takip katmanı, piksel ızgarası + parlama, eğik açı + bulanıklık, hızlı kurgu + palet (hepsi) |
| Ana görüntü | Terminal / ajan ekranı; kutular yazılan karakterleri takip eder |
| Metin | Ajan düşünce akışı, İngilizce |
| Süre | 6 sn, 30 fps (180 kare), 12 sahne × 0,5 sn (15 kare) |
| Ses | Kodla üretilen senkron efekt sesleri, hedef -14 LUFS |
| Yaklaşım | İki geçiş, ikisi de fframes; ikinci geçiş Skia Metal shader |

## Sahne akışı

| # | Terminal metni | Palet | Kamera |
|---|---|---|---|
| 1 | `▍` (yanıp sönen imleç) | kâğıt beyazı | çok yakın, sola eğik |
| 2 | `thinking…` | termal (mor→sarı) | yavaş kayma |
| 3 | `reading 14 files` | siyah-beyaz | sağa eğik |
| 4 | `src/render.rs` | siyah-beyaz | odak kayması |
| 5 | `tool_call: render_frame()` | fosfor yeşili | yukarıdan bakış |
| 6 | `attention → token 4812` | termal | hızlı yakınlaşma |
| 7 | `not just completing —` | siyah-beyaz | yatay kayma |
| 8 | `— composing` | kehribar | geri çekilme |
| 9 | `verifying 180 frames` | lacivert | sabit, titreşimli |
| 10 | `✓ 0 problems` | fosfor yeşili | eğik |
| 11 | (bütün kutular tek noktaya toplanır) | termal | yakınlaşma |
| 12 | `Done` | neon yeşil hap şekli | geri çekilme, siyaha kesme |

### Referansın ötesindeki eklemeler

1. **Gerçek koordinatlar.** Etiketlerdeki x/y değerleri, karakterin düz (geçiş 1) görüntüdeki
   gerçek piksel konumudur. Rastgele sayı yoktur.
2. **Düşünce zinciri çizgileri.** Bağlantı çizgileri, ajanın okuduğu sırayla kutudan kutuya ilerler.
3. **Kilitlenen odak.** Netlik düzlemi her sahnede o an takip edilen kutuya kayar (rack focus).
4. **Kilitlenme anı.** Kutu hedefe kilitlenince 2 kare kırmızı olur; tarama çizgisi yırtılır; bip sesi çalar.
5. **Final.** Sahne 11'de bütün kutular tek kutuya toplanır; sahne 12'de o kutu "Done" hapına dönüşür.

## Mimari

```
scenes.json ──► flat (geçiş 1) ──► out/flat.mp4 + out/tracks.json
                                          │
scenes.json ──────────────────────► lens (geçiş 2, Skia Metal, lens.sksl) ──► out/tracking.mp4
                                          ▲
               sfx (WAV üretici) ─► out/sfx/*.wav (tracks.json olaylarına göre)
```

### `scenes.json` (tek kaynak)

Her sahne için: `text`, `palette` (ad), `camera` (eğim açıları, yakınlık, kayma yönü/hızı),
`frames` (varsayılan 15). Paletler aynı dosyada ad → renk durakları (LUT) olarak tanımlıdır.

### Geçiş 1: `flat` (fframes projesi, CPU veya Skia backend)

- Siyah zemin, beyaz, tek genişlikli yazı tipi. Harf harf yazım animasyonu.
- Karakter konumu: `x = sol_kenar + sütun × karakter_genişliği`. Tek genişlikli yazı tipi
  olduğu için kesin.
- Takip katmanı: `<pattern>` ile taralı kutular, etiketler, çizgiler. Kutular yeni yazılan
  karakterleri ve kelimeleri hedefler.
- Çıktılar: `out/flat.mp4` (renksiz, efektsiz) ve `out/tracks.json`:
  her kare için `focus: [x, y]`, olay listesi `locks: [{frame, x, y}]`, `cuts: [frame]`.

### Geçiş 2: `lens` (fframes, Skia Metal backend)

- `out/flat.mp4` karesini `iChannel0` olarak `lens.sksl` shader'ına verir.
- Shader sırası: perspektif (homografi) → odak noktasına uzaklığa göre bulanıklık →
  palet LUT → parlama → renk kayması → LED ızgara maskesi → kilitlenmede yırtılma.
- Uniform değerleri: kamera (`scenes.json`), odak ve kilitlenme (`tracks.json`), sahne içi zaman.
- SkSL kısıtları: sabit döngü sınırları, dinamik dizi indeksi yok. Bulanıklık sabit örnek sayılı
  disk örneklemesiyle yapılır.

### Ses: `sfx`

- Küçük bir script, efekt seslerini sentezler: kesmede tık/glitch, kilitlenmede kısa bip,
  sürekli düşük uğultu.
- fframes ses haritasına `tracks.json` olay karelerine göre yerleştirilir. Hedef -14 LUFS, limiter -1 dBFS.

## Riskler ve geri dönüş

- **Ana risk:** geçiş 2'de video karesinin shader'a girdi olarak verilememesi.
  **Teknik test (ilk iş):** tek sahnelik `flat.mp4` → `lens` → kontrol karesi.
  Test başarısız olursa geçiş 2'yi ffmpeg filtreleriyle yaparım (`perspective`, `gblur`,
  `rgbashift`, ızgara bindirmesi) ve Patron'a bildiririm. Sahne başına palet LUT ile korunur.
- **Performans:** shader'daki bulanıklık örnek sayısı render süresini belirler. Taslaklar yarım
  çözünürlükte render edilir.

## Kontrol

1. Her renderdan sonra fframes CLI ile kare özet görüntüsü (contact sheet), otomatik hata taraması
   ve ses ölçümü.
2. Tam videodan önce Patron'a 2 sahnelik taslak gösterilir (sahne 2 ve 6: termal palet, en zor efektler).
3. Son çıktı: `out/tracking.mp4`, 1920×1080, 6,0 sn, -14 LUFS ±1.

## Kapsam dışı

- Gerçek kamera görüntüsü üzerinde hareket takibi.
- 6 sn'den uzun sürüm, müzik, seslendirme.
- X'te yayınlama.

## Uygulamada değişenler (2026-10-09)

- **`scenes.json` yerine `hud` crate'i.** Sahne verisi, takip kutuları ve kamera ortak bir Rust
  crate'inde (`hud/src/lib.rs`). İki geçiş aynı fonksiyonları çağırır; `tracks.json` yalnızca ses
  script'i için `hud`'dan üretilir (`target/release/tracks`).
- **İki katmanlı takip.** Kutular ve tarama deseni ekranda kalır (LED ızgarası ve bulanıklık alır).
  Etiketler, bağlantı çizgileri ve hedef köşelikleri kameranın net katmanıdır; `lens` bunları
  `hud::project` ile çizer. Etiketteki x/y, kutunun çıktı görüntüsündeki gerçek pikselidir.
  Sebep: LED ızgarasında küçük etiketler okunamıyordu.
- **Teknik test başarılı.** `get_synced_video_frame(..).into_image()` shader'a `iChannel0` olarak
  veriliyor; ffmpeg yedeğine gerek kalmadı.
- **Render ayarı.** `lens` tek GPU hattıyla (`OnePipeline`) çalışır. `flat` sonuna 10 yedek kare
  eklenir, çünkü çözücü son kareleri vermiyor; kare gelmezse `lens` siyah kare çizer.
- **Ses.** Gerçekleşen değer: -14,0 LUFS, gerçek tepe -0,9 dBTP (fframes limiter -1 dBFS).

## Çalıştırma

`tools/render.sh` → `out/tracking.mp4` (yaklaşık 40 sn).

## fframes 1.2.0'a geçiş (2026-10-09)

- 1.2.0 son video karelerini veriyor (#167, #172): `flat` sonundaki 10 yedek kare kaldırıldı.
- `MaxPerformance` 1.2.0'da kilitlenmiyor: tekrar açıldı (lens render 33 sn → 28-30 sn).
- Eski kilitlenmenin asıl sebebi: kökünde `<svg>` olmayan bir kare (`Svgr::empty()`) tam
  render'ı hata vermeden kilitliyor. Aralık render'ı (`render 160..180`) doğru hata veriyor.
  1.2.0'da iki modda da tekrarlandı; siyah kare yedeği bu yüzden kalıyor.
- 1.0 ile 1.2 çıktısı gözle aynı (SSIM 0,95, fark kumlanma ve kodlama gürültüsü).
