# ff-tracking

> **English:** a 6-second tracking-HUD style test made with [fframes](https://github.com/dmtrKovalenko/fframes).
> Pass 1 (`flat`) renders an agent terminal and tracker boxes at 3840×2160. Pass 2 (`lens`) films it
> through one SkSL shader (`lens/shaders/lens.sksl`): perspective tilt, depth of field focused on the
> tracked glyph, LED grid, bloom, chromatic aberration, per-scene palette. A sharp camera-space
> tracker overlay is projected with the same camera (`hud::project`), so the label coordinates are the
> boxes' real output pixels. Sound is synthesized (`tools/sfx.py`). Run `tools/render.sh`.
> Style inspired by [@mnowakdesign](https://x.com/mnowakdesign/status/2108253918086176899). MIT.

6 saniyelik tracking-HUD stil denemesi, [fframes](https://github.com/dmtrKovalenko/fframes) ile.
Bir ajan terminalde düşünür; ekrandaki takip kutuları yazılan harfleri izler, bir sanal kamera
ekranı eğik açıdan, odak bulanıklığı, LED ızgarası ve sahne başına paletle çeker.

Stil ilhamı: [Michael Nowak (@mnowakdesign)](https://x.com/mnowakdesign/status/2108253918086176899).

## Çalıştırma

```bash
tools/render.sh        # -> out/tracking.mp4 (yaklaşık 40 sn)
```

Gerekenler: Rust, ffmpeg, Python 3 + numpy + scipy, macOS (Skia Metal).

## Yapı

| Klasör | Ne yapar |
|---|---|
| `hud/` | Sahneler, takip kutuları, kamera, projeksiyon; iki geçişin ortak kaynağı |
| `flat/` | Geçiş 1: düz ekran + kutular, 3840×2160 → `out/pass/flat.mp4` |
| `lens/` | Geçiş 2: `shaders/lens.sksl` ile kamera + net takip katmanı → `out/tracking.mp4` |
| `tools/sfx.py` | Bütün sesleri sentezler → `out/pass/sfx.wav` |

Sahne metni, palet ya da kamera değiştirmek için: `hud/src/lib.rs` → `SCENE_LIST`.

Tasarım: `docs/superpowers/specs/2026-10-09-tracking-hud-design.md`. Promptlar ve kararlar: `docs/PROMPTS.md`.

## Lisans

Kod: MIT (`LICENSE`). Yazı tipi: JetBrains Mono, SIL Open Font License 1.1 (`media/JetBrainsMono-OFL.txt`).
Sesler `tools/sfx.py` ile sentezlenir; üçüncü taraf ses yoktur.
