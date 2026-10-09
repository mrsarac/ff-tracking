# ff-tracking: tracking-HUD style test (design)

Date: 2026-10-09
Status: approved (brainstorming), implemented

## Goal

Recreate and extend the visual style of Michael Nowak's X post with fframes.
Reference: <https://x.com/mnowakdesign/status/2108253918086176899> (local copy in `reference/`, not in git).

This is a **style test**; the content is secondary. Success: the result feels as much like
"a close-up of a physical screen" as the reference and adds at least one idea beyond it.

## Reference analysis

- 5.5 s, 1920×1080, with sound.
- UI text filmed very close and at an angle. Strong depth of field.
- LED/CRT pixel grid, bloom, chromatic aberration.
- Computer-vision overlay: hatched boxes, `x: 725 y: 475` labels, lines linking the boxes.
- A cut about every 0.5 s. Each shot has its own palette: paper white, thermal purple,
  black and white, navy, neon green.

## Decisions

| Topic | Decision |
|---|---|
| Use | Style test |
| Layers | Tracker overlay, pixel grid + bloom, tilt + blur, fast cuts + palettes (all of them) |
| Subject | An agent's terminal; the boxes track the characters as they are typed |
| Text | Agent thought stream, in English |
| Length | 6 s, 30 fps (180 frames), 12 shots × 0.5 s (15 frames) |
| Sound | Synthesized, synced sound effects, target -14 LUFS |
| Approach | Two passes, both fframes; the second is a Skia Metal shader |

## Shot list

| # | Terminal text | Palette | Camera |
|---|---|---|---|
| 1 | `›` and a blinking cursor | paper white | very close, tilted left |
| 2 | `thinking…` | thermal (purple→yellow) | slow drift |
| 3 | `reading 14 files` | black and white | tilted right |
| 4 | `src/render.rs` | black and white | rack focus |
| 5 | `tool_call: render_frame()` | phosphor green | top-down |
| 6 | `attention → token 4812` | thermal | fast push-in |
| 7 | `not just completing —` | black and white | horizontal pan |
| 8 | `— composing` | amber | pull back |
| 9 | `verifying 180 frames` | navy | locked off, shaking |
| 10 | `✓ 0 problems` | phosphor green | tilted |
| 11 | (every box collapses into one point) | thermal | push-in |
| 12 | `Done` | neon green pill | pull back, fade to black |

### Beyond the reference

1. **Real coordinates.** The x/y in each label is the box's real pixel position, not a random number.
2. **Chain-of-thought lines.** The links run from box to box in the order the agent reads.
3. **Locking focus.** The focal plane moves to the box being tracked in every shot (rack focus).
4. **Lock moment.** When a box locks on, it turns red for 2 frames, the scanlines tear and a beep plays.
5. **Ending.** In shot 11 every box collapses into one; in shot 12 that box becomes the "Done" pill.

## Architecture (as built)

```
hud (shared crate: shots, tracker boxes, camera, projection)
 ├─► flat   (pass 1, 3840×2160) ──► out/pass/flat.mp4
 ├─► tracks (bin) ──► out/tracks.json ──► tools/sfx.py ──► out/pass/sfx.wav
 └─► lens   (pass 2, Skia Metal, lens.sksl + camera overlay) ──► out/tracking.mp4
```

### `hud` (single source of truth)

`hud/src/lib.rs` holds `SCENE_LIST` (text, palette, camera per shot), the tracker
(`track(scene, frame)`), the camera per video frame (`shot(frame)`) and `project()`, the CPU
copy of the shader's camera. Both passes call the same functions, so they agree on every frame.

### Pass 1: `flat`

- Black background, white monospace text (JetBrains Mono, 0.6 em advance), typed glyph by glyph.
- One `<text>` per glyph at `x = hero.x + column × advance`, so the tracker positions are exact.
- Tracker boxes on the screen itself: stroked rects, some hatched with a `<pattern>` in
  `mix-blend-mode: difference` (the stripes turn dark over a glyph, like the reference).
- Only color: pure red (#ff0000) on a locked box; the lens keeps it red in every palette.
- Rendered at 2× (3840×2160) because the lens magnifies it up to 3×.

### Pass 2: `lens`

- Binds the synced `flat.mp4` frame as `iChannel0` of `lens/shaders/lens.sksl`.
- Shader order: tear → perspective (ray / tilted plane) → depth of field (circle of confusion
  from the depth difference to the tracked target, 36 taps, chromatic aberration per channel) →
  palette (5 stops) → bloom (16 wide taps) → LED grid (equal-area RGB stripes, fades out of
  focus) → exposure kick on a cut, vignette, flicker, grain, final fade.
- On top: a sharp camera-space tracker (links, a dot running along each link, coordinate
  labels, brackets on the target), projected with `hud::project`. Label x/y = the box's
  real pixel in the output. Reason: small labels were unreadable through the LED grid.

### Sound: `tools/sfx.py`

Reads `out/tracks.json` and synthesizes everything: screen hum, a glitch on each cut, a
click per keystroke, a pentatonic beep on each lock (panned to the box), a riser into the
collapse and a chime on "Done". Loudness is measured with ffmpeg `ebur128` and corrected
to -14 LUFS under a soft clip.

## Checks

1. After every render: contact sheets, `inspect`, `audio analyze`.
2. Result: `out/tracking.mp4`, 1920×1080, 6.0 s, 180 frames, -14.0 LUFS, true peak -0.9 dBTP
   (fframes limiter at -1 dBFS).

## Out of scope

- Motion tracking on real camera footage.
- Versions longer than 6 s, music, voice-over.

## Notes from the build

- **The risk did not happen.** `get_synced_video_frame(..).into_image()` binds as a shader
  child, so the ffmpeg fallback was not needed.
- **SkSL:** `flat` is a reserved word (GLSL ES interpolation qualifier).
- **fframes 1.0.0:** the decoder did not hand out the last frames of `flat.mp4`; the
  workaround was 10 padding frames. A frame without a root `<svg>` stalled the render.
- **fframes 1.2.0 (current):** the last frames arrive (#167, #172), so the padding is gone;
  `MaxPerformance` works (lens render 33 s → 28-30 s). A frame without a root `<svg>`
  still delays the error: the unfinished segments drain their x264 lookahead on drop first
  (86 s instead of 28 s here), reported as fframes #209. That is why the lens draws a black
  frame when the source frame is missing.
- 1.0 and 1.2 output look the same (SSIM 0.95; the difference is grain and encoder noise).

## Run

`tools/render.sh` → `out/tracking.mp4` (about 40 s on an M3 Pro).
