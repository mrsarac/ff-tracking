# The prompt

This is the prompt that produces this video. Paste it into [Claude Code](https://claude.com/claude-code)
with the [`fframes-video`](https://github.com/dmtrKovalenko/fframes/tree/main/skills/fframes-video)
skill installed (`npx skills add dmtrKovalenko/fframes`) on a Mac with Apple silicon.
It holds every decision the original session made along the way, so it needs no Q&A.
Change the parts in the brief, and you get your own version.

````text
Make a 6-second video with fframes (use the fframes-video skill): a virtual camera films an AI
agent's terminal up close while a computer-vision tracker locks onto every glyph it types.

STYLE REFERENCE
Michael Nowak's test: https://x.com/mnowakdesign/status/2108253918086176899
(fetch it through https://api.fxtwitter.com/mnowakdesign/status/2108253918086176899, download the
video, look at a contact sheet). Take the look, not the frames: close-up UI text at an angle, strong
depth of field, LED/CRT pixel grid, bloom, chromatic aberration, hatched tracker boxes with
"x: 725 y: 475" labels and lines between them, a cut every ~0.5 s, a different palette per shot.
Then push it further.

BRIEF
- 1920×1080, 30 fps, 12 shots × 15 frames = 180 frames.
- The agent types one line per shot, in English, monospace (JetBrains Mono):
  1 "›" + blinking cursor (paper white, very close, tilted left)
  2 "thinking…" (thermal purple→yellow)   3 "reading 14 files" (black and white)
  4 "src/render.rs" (b/w, rack focus)      5 "tool_call: render_frame()" (phosphor green, top-down)
  6 "attention → token 4812" (thermal, fast push-in)   7 "not just completing —" (b/w, pan)
  8 "— composing" (amber, pull back)       9 "verifying 180 frames" (navy, shaking)
  10 "✓ 0 problems" (phosphor)            11 fourteen boxes collapse into one (thermal)
  12 that box becomes a "Done" pill with a check icon (neon green), fade to black.
- Behind the line: a dim agent log, gutter icons, a title bar and a status bar, so the tilted
  camera has something to blur everywhere.
- Every shot opens with ~35 % of its line typed, finishes typing by frame 8, and at frame 9 the
  newest box grows over the last word and LOCKS: red for 2 frames, scanline tear, beep.

WHAT MAKES IT MORE THAN A COPY
- Real coordinates: every label shows the box's actual pixel position in the final frame.
- Focus follows the tracker: the focal plane sits on the box being tracked.
- Chain of thought: links run box to box in reading order, a dot runs along every link.
- Two tracker layers: boxes live ON the screen (they get the grid and the blur); labels, links and
  corner brackets belong to the CAMERA and stay sharp. Small labels die in the LED grid otherwise.

ARCHITECTURE (do it this way)
- Cargo workspace with three crates:
  hud: no fframes dependency. Shot list, deterministic tracker (boxes per scene frame, seeded
       jitter), camera per video frame (aim trails the target over 4 frames, ease-out zoom, slow
       sway, shake), and project(): the CPU copy of the shader's camera. Unit tests: every line fits
       the screen, the camera keeps every target inside the frame.
  flat (pass 1): SVG at 3840×2160 (hud's 1920×1080 logical space × 2, the lens magnifies up to 3×).
       White on black only; one <text> per glyph at x = start + column × 0.6 em so positions are
       exact; boxes hatched with a rotated <pattern> in mix-blend-mode:difference; pure #ff0000 is
       the only color and marks a locked box. Encode near-lossless (libx264 crf 8).
  lens (pass 2): binds each synced frame of flat.mp4 as a shader child:
       frame.get_synced_video_frame(ctx, "flat.mp4", &input) → .into_image() →
       ShaderUniforms::image("iChannel0", ..). One SkSL shader, in this order: row tear on lock/cut →
       perspective (ray against a tilted plane, rotation Rz·Ry·Rx) → depth of field (circle of
       confusion from the depth difference to the tracked target, 36 golden-angle taps snapped to
       LED cell centers, bokeh weighting, R/G/B sampled at radially shifted points) → 5-stop palette
       per shot (keep red red) → bloom (16 wide taps) → LED grid (equal-area RGB stripes, fades out
       of focus and when cells get too small) → exposure kick on a cut, vignette, flicker, grain,
       final fade. Add a LENS_STAGE env var (0-3) that stops the shader after a step.
       Then draw the sharp camera tracker as SVG on top, positioned with hud::project.
- A small bin exports cut, lock and keystroke times plus the focus point per frame to
  out/tracks.json. tools/sfx.py (numpy + scipy) reads it and synthesizes all sound:
  screen hum, a bit-crushed glitch per cut, a click per keystroke, a pentatonic beep per lock
  panned to the box, a riser into the collapse, a chime + sub hit on "Done". Measure with ffmpeg
  ebur128, correct to -14 LUFS under a soft clip, true peak below -1 dBTP.
- tools/render.sh runs everything; the lens loads out/pass (flat.mp4, sfx.wav) and media/ (fonts)
  at runtime through MediaDirectory + CombinedMediaProvider.

KNOWN TRAPS (fframes 1.2.0)
- `flat` is a reserved word in SkSL. Name the sampling helper something else.
- Never return Svgr::empty() from Video::render_frame; return a root <svg> (a black rect when the
  source frame is missing). An empty root is an error that surfaces only after the encoders
  drain (fframes#209), which looks like a hang.
- include_media_dir! panics on files it doesn't know: keep media/ to fonts (and audio/images).
- If you can, reuse a built target/ from another fframes 1.2 project (cp -cR on APFS) to skip
  the first Skia build.

LOOP AND DONE
- Spike first: one shot through both passes. Proceed only if the video frame reaches the shader.
- After every change: hud tests, `inspect`, a contact sheet of all 12 shots, full-size frames
  of a lock and of the ending, `audio analyze`. Look at the images before you call anything good.
- Done = out/tracking.mp4, 1920×1080, 180 frames, 6.0 s, -14 LUFS ±0.5, no text clipped, every
  label readable, and a README that shows a preview, the 12 shots and the shader stages.
- Make the decisions yourself; report at the end with the path, a contact sheet and the numbers.
````

## Notes

- The original session took about 126 model calls. The prompt above folds in what that session
  learned (the two tracker layers, the traps), so a fresh run should need fewer.
- The result will not be pixel-identical: the tracker jitter, palettes and camera values are
  tuned by eye. The code in this repo is one result of this prompt.
