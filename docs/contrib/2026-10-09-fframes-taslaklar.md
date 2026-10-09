# fframes katkı taslakları (2026-10-09)

Hedef: github.com/dmtrKovalenko/fframes, hesap @mrsarac. Durum: A gönderildi → https://github.com/dmtrKovalenko/fframes/issues/209 (2026-10-09). B gönderiliyor.
Sıra: önce A (hata), sonra B (özellik önerisi, içinde C örnek teklifi).
C için ff-tracking reposunun public olması ve bir lisans (MIT) alması gerekir.

---

## A. Issue: hata

**Başlık:**
Skia render: an error late in the render surfaces only after unfinished segments drain their x264 lookahead (looks like a hang)

**Gövde:**

### What happens

When `render_frame` fails late in a full render (here: a frame without a root `<svg>`, so
`ParserError(NoRootNode)`), the Skia pipeline stops, but the error is printed only after
`SegmentWriter` is dropped. Dropping every unfinished segment flushes its encoder on the main
thread, and with a slow x264 preset that drains the whole lookahead of each segment, one
segment after another. Meanwhile one core sits at ~99 %, the progress output has stopped and
nothing is printed, so it looks like a hang.

| Case (fframes 1.2.0, Skia Metal, M3 Pro) | Time to exit |
|---|---|
| Repro below, no bad frame | 12.5 s (success) |
| Repro, `BAD_FRAME=290` (of 300) | 22 s, then `ParserError(NoRootNode)` |
| Repro, `BAD_FRAME=30` or `150` | < 1 s (fine) |
| Real project (1080p, heavy SkSL pass, `preset=slow`, `tune=grain`), no bad frame | 28 s (success) |
| Same project, bad frame at 170 of 180 | **86 s**, then `ParserError(NoRootNode)` |

A short range render containing the same frame (`render 160..180`) fails within seconds.

### Repro

```toml
[dependencies]
fframes = { version = "=1.2.0", features = ["cli", "h264", "libav-agree-gpl"] }
fframes_skia_renderer = { version = "=1.2.0", features = ["metal"] }
```

```rust
use fframes::{AudioMap, Color, Duration, EncoderOptions, FFramesContext, Frame, RenderOptions, Svgr, Video, cli};
use fframes_skia_renderer::{SkiaFFramesRenderer, SkiaPipelineConfig, metal::SkiaMetalCtx};

struct Repro { bad_frame: Option<usize> }

impl Video for Repro {
    const FPS: usize = 30;
    const WIDTH: usize = 1920;
    const HEIGHT: usize = 1080;
    const BACKGROUND_COLOR: Color = Color::BLACK;
    fn duration(&self) -> Duration<'_> { Duration::Frames(300) }
    fn audio(&self) -> AudioMap<'_> { AudioMap::none() }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        if self.bad_frame.is_some_and(|n| frame.index >= n) {
            return Svgr::empty(); // no root <svg>
        }
        // busy, changing content so the encoder has real work
        let dots: Vec<Svgr> = (0..1500usize).map(|i| {
            let h = (i * 7919 + frame.index * 104_729) % 99_991;
            let (x, y) = ((h * 31) % 1920, (h * 17) % 1080);
            let fill = format!("#{:06x}", h.wrapping_mul(2_654_435_761) & 0xff_ffff);
            fframes::svgr!(<rect x={x} y={y} width="14" height="14" fill={fill} />)
        }).collect();
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080">
            <rect width="1920" height="1080" fill="#101820" />
            {dots}
        </svg>)
    }
}

fn main() -> std::process::ExitCode {
    let video = Repro { bad_frame: std::env::var("BAD_FRAME").ok().and_then(|v| v.parse().ok()) };
    let options = RenderOptions {
        video_encoder_options: EncoderOptions {
            preferred_encoder: Some("libx264"),
            codec_params: Some(&[("crf", "16"), ("preset", "slow")]),
            ..Default::default()
        },
        ..Default::default()
    };
    let gpu = SkiaMetalCtx::new(1920, 1080).expect("GPU context");
    let backend = SkiaFFramesRenderer::new_metal(&gpu, SkiaPipelineConfig::default()).expect("skia");
    cli::new(&video, options).backend(backend).run()
}
```

```bash
cargo run --release -- render -o ok.mp4                  # 12.5 s
BAD_FRAME=290 cargo run --release -- render -o bad.mp4   # 22 s, then the error
```

### Where the time goes

`sample` of the stuck process (main thread, 2462/2462 samples):

```
fframes_skia_renderer::skia_pipeline::render
  core::ptr::drop_in_place<fframes::renderer::segment_writer::SegmentWriter>
    alloc::sync::Arc<T,A>::drop_slow
      core::ptr::drop_in_place<UnsafeCell<Option<segment_writer::Segment>>>
        libx264.165.dylib  (encoding)
```

`Encoder::drop` (`fframes/src/renderer/encoder.rs`) calls `avcodec_flush_buffers` on the video
encoder. For libx264 that drains every delayed frame. A segment that finished normally was
already drained by `flush_stream` in `SegmentWriter::submit_with`, so the drop is cheap there.
Only a segment that never finished, whose file is thrown away anyway, pays for the drain.

### Possible fix

On the error path, discard open segments without draining them, for example a
`SegmentWriter::abort()` that drops the open encoders with a flag that skips
`avcodec_flush_buffers` and `av_write_trailer` and removes the temp files. Then
`skia_pipeline::render` calls it before returning `Err`. I'm happy to send a PR if this
direction is OK.

### Related docs note

`skills/fframes-video/references/api.md` shows
`let Some(clip) = frame.get_synced_video_frame(..) else { return Svgr::empty() };`.
That's fine inside a `Scene`, but at the `Video::render_frame` level it produces exactly
this `NoRootNode` error. A one-line hint ("at the video level return a root `<svg>`, e.g.
a black rect") would save the next person the search.

Environment: fframes 1.2.0, fframes_skia_renderer 1.2.0 (Metal), macOS 26.6.2, Apple M3
Pro, rustc 1.91.1, Homebrew FFmpeg 9.0.1, libx264.165. On 1.0.0 I saw the same stop with
0 % CPU and no exit after 10 min; I did not re-check 1.0.0 after finding the cause.

---

## B. Issue: özellik önerisi (içinde C)

**Başlık:**
Feature idea: feed a shader with an SVG subtree rendered in the same frame (multi-pass in one video)

**Gövde:**

### Motivation

I recreated @mnowakdesign's tracking-HUD look (the same one you posted) with a camera that
films the screen: perspective tilt, depth of field focused on the tracked glyph, LED grid,
bloom, chromatic aberration, per-scene palette, all in one SkSL pass.

The shader needs the flat screen as its input, so today it takes two fframes projects:

1. `flat` renders the terminal + tracker at 3840×2160 to an mp4 (crf 8).
2. `lens` reads it with `get_synced_video_frame(..).into_image()` and binds it as `iChannel0`.

That works (thanks for the video-frames-in-shaders support), but it costs a 4K h264 round
trip per frame (disk, decode, compression artefacts under 3× magnification), two binaries,
and a shared crate just so both passes agree on the timeline.

### Idea

Let a shader take an SVG subtree of the same frame as a child image, rendered offscreen
first:

```rust
let screen = fframes::svgr!(<g>{terminal}{tracker}</g>);
let layer = self.lens.draw(&frame, ShaderUniforms::new()
    .layer("iChannel0", screen, 3840, 2160)   // rendered offscreen, bound as `uniform shader`
    .float("uZoom", zoom));
fframes::svgr!(<image href={layer.href()} width="1920" height="1080" />)
```

On Skia this could stay on the GPU: draw the subtree into an offscreen surface and bind its
image snapshot as the child shader, with no CPU readback. On the CPU backend it could fall
back to rasterizing the subtree. Is this something you'd want in fframes? I can help with
the API, an example or tests.

### Example (C)

The project is at https://github.com/mrsarac/ff-tracking (MIT): 6 s, 12 scenes, a two-pass
render plus a synthesized sound track (~40 s end to end on an M3 Pro). If it's useful, I can
contribute a trimmed version as `examples/screen-camera`, now as a two-pass example or later
as the single-pass demo of the feature above. Video: https://x.com/0xsarac/status/2108515856544317489
