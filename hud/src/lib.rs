//! Shared data of both passes: what the agent types, where the tracker boxes sit and where
//! the camera looks. `flat` draws it, `lens` films it, `tracks` (bin) exports the sound events.
//!
//! Every position is in the flat pass's logical space, `W` x `H`.

pub const FPS: usize = 30;
pub const W: f32 = 1920.0;
pub const H: f32 = 1080.0;
pub const SCENE_FRAMES: usize = 15;
pub const SCENES: usize = 12;
pub const TOTAL_FRAMES: usize = SCENE_FRAMES * SCENES;

/// Typing runs until this scene frame; the newest box locks right after.
pub const TYPE_END: usize = 8;
pub const LOCK_AT: usize = 9;
pub const LOCK_FRAMES: usize = 2;
/// Share of the line already on screen at the cut, so every shot opens with something to track.
const TYPED_AT_CUT: f32 = 0.35;
/// Boxes on the newest characters.
const CHAIN: usize = 5;

/// JetBrains Mono advances 0.6 em per character.
pub const MONO_ADV: f32 = 0.6;
pub const LOG_SIZE: f32 = 26.0;
pub const LOG_LINE: f32 = 46.0;
pub const LOG_X: f32 = 150.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Cursor,
    Typing,
    Collapse,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Palette {
    Paper,
    Thermal,
    Mono,
    Phosphor,
    Amber,
    Navy,
    Neon,
}

impl Palette {
    /// Five color stops from black (0) to white (1) of the flat pass, sRGB 0..1.
    pub fn stops(self) -> [[f32; 3]; 5] {
        let c = |hex: u32| {
            [
                ((hex >> 16) & 0xff) as f32 / 255.0,
                ((hex >> 8) & 0xff) as f32 / 255.0,
                (hex & 0xff) as f32 / 255.0,
            ]
        };
        let s = |a, b, cc, d, e| [c(a), c(b), c(cc), c(d), c(e)];
        match self {
            Palette::Paper => s(0xecebe6, 0xc9c8c2, 0x77766f, 0x2a2a28, 0x0c0c0c),
            Palette::Thermal => s(0x050310, 0x3a0b6b, 0xb3306a, 0xf6862a, 0xfcf6b4),
            Palette::Mono => s(0x050506, 0x2b2c2e, 0x8a8c90, 0xd9dadc, 0xffffff),
            Palette::Phosphor => s(0x010604, 0x05361b, 0x13a85a, 0x7dffb4, 0xeafff3),
            Palette::Amber => s(0x070300, 0x3d1a02, 0xb35a09, 0xffae3d, 0xfff3d6),
            Palette::Navy => s(0x071451, 0x0d2a8c, 0x4a63d6, 0xb8c4ff, 0xf4f6ff),
            Palette::Neon => s(0x000502, 0x003d1e, 0x00c46a, 0x5dffae, 0xf0fff6),
        }
    }
}

/// Virtual camera filming the flat screen. Angles in radians, zoom 1 shows the whole screen.
#[derive(Clone, Copy, Debug)]
pub struct Cam {
    /// Pitch: positive tips the top of the screen away.
    pub tilt_x: f32,
    /// Yaw: positive turns the right side away.
    pub tilt_y: f32,
    pub roll: f32,
    pub zoom: (f32, f32),
    /// Drift of the aim point over the shot, logical px.
    pub pan: (f32, f32),
    /// Depth of field strength, 1 = normal.
    pub blur: f32,
    /// Hand-held shake, logical px.
    pub shake: f32,
}

pub struct SceneDef {
    pub kind: Kind,
    pub text: &'static str,
    pub palette: Palette,
    pub cam: Cam,
    /// Baseline start of the hero line.
    pub hero: (f32, f32),
    pub size: f32,
}

const fn cam(
    tilt_x: f32,
    tilt_y: f32,
    roll: f32,
    z0: f32,
    z1: f32,
    px: f32,
    py: f32,
    blur: f32,
) -> Cam {
    Cam {
        tilt_x,
        tilt_y,
        roll,
        zoom: (z0, z1),
        pan: (px, py),
        blur,
        shake: 0.0,
    }
}

pub const SCENE_LIST: [SceneDef; SCENES] = [
    SceneDef {
        kind: Kind::Cursor,
        text: "›",
        palette: Palette::Paper,
        cam: cam(0.18, 0.62, -0.05, 3.3, 3.7, 10.0, 0.0, 1.1),
        hero: (720.0, 560.0),
        size: 120.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "thinking…",
        palette: Palette::Thermal,
        cam: cam(0.38, -0.22, 0.04, 2.5, 2.8, 50.0, 0.0, 1.2),
        hero: (560.0, 540.0),
        size: 104.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "reading 14 files",
        palette: Palette::Mono,
        cam: cam(0.12, -0.55, 0.06, 2.1, 2.3, 60.0, -10.0, 1.0),
        hero: (330.0, 600.0),
        size: 84.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "src/render.rs",
        palette: Palette::Mono,
        cam: cam(-0.12, 0.5, -0.03, 2.7, 2.7, 0.0, 0.0, 1.7),
        hero: (420.0, 480.0),
        size: 92.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "tool_call: render_frame()",
        palette: Palette::Phosphor,
        cam: cam(0.78, 0.1, -0.12, 1.9, 2.2, 30.0, 0.0, 1.2),
        hero: (230.0, 560.0),
        size: 64.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "attention → token 4812",
        palette: Palette::Thermal,
        cam: cam(0.26, -0.36, 0.0, 1.6, 3.1, 0.0, 0.0, 1.0),
        hero: (250.0, 540.0),
        size: 72.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "not just completing —",
        palette: Palette::Mono,
        cam: cam(0.2, 0.32, 0.05, 2.3, 2.3, -170.0, 0.0, 1.1),
        hero: (230.0, 520.0),
        size: 76.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "— composing",
        palette: Palette::Amber,
        cam: cam(0.32, -0.26, -0.02, 3.1, 2.1, 0.0, 0.0, 1.3),
        hero: (560.0, 560.0),
        size: 112.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "verifying 180 frames",
        palette: Palette::Navy,
        cam: Cam {
            tilt_x: 0.15,
            tilt_y: -0.16,
            roll: 0.0,
            zoom: (2.05, 2.1),
            pan: (0.0, 0.0),
            blur: 0.8,
            shake: 4.0,
        },
        hero: (300.0, 520.0),
        size: 74.0,
    },
    SceneDef {
        kind: Kind::Typing,
        text: "✓ 0 problems",
        palette: Palette::Phosphor,
        cam: cam(0.22, 0.62, -0.08, 2.5, 2.7, 20.0, 0.0, 1.2),
        hero: (500.0, 560.0),
        size: 100.0,
    },
    SceneDef {
        kind: Kind::Collapse,
        text: "",
        palette: Palette::Thermal,
        cam: cam(0.3, 0.2, 0.03, 1.35, 2.7, 0.0, 0.0, 1.0),
        hero: (960.0, 540.0),
        size: 80.0,
    },
    SceneDef {
        kind: Kind::Done,
        text: "Done",
        palette: Palette::Neon,
        cam: cam(0.46, -0.32, 0.06, 3.0, 1.9, 0.0, 0.0, 1.2),
        hero: (960.0, 540.0),
        size: 80.0,
    },
];

/// Dim agent log behind the hero line; each scene scrolls it by a few rows.
pub const LOG: [&str; 24] = [
    "› plan  1/4  map the request to scenes",
    "  read  src/lib.rs              412 lines",
    "  read  src/render.rs           188 lines",
    "  grep  \"render_frame\"         9 matches",
    "› think  boxes must follow real glyphs",
    "  tool  measure_text(\"JetBrains Mono\")",
    "  ok    advance 0.6 em",
    "  read  shaders/lens.sksl        96 lines",
    "› plan  2/4  camera, focus, palette",
    "  tool  render_frame(42)        12 ms",
    "  tool  render_frame(43)        11 ms",
    "  warn  focus drifted 3 px, correcting",
    "  ok    lock  x: 1180 y: 512",
    "› think  not just completing — composing",
    "  read  scenes.rs               210 lines",
    "  tool  inspect --all-frames",
    "  ok    0 problems",
    "› plan  3/4  sound on every lock",
    "  tool  sfx.synth(beep, 1760 Hz)",
    "  ok    -14.0 LUFS",
    "› plan  4/4  verify, deliver",
    "  tool  ffprobe out/tracking.mp4",
    "  ok    180 frames  6.0 s",
    "› done",
];

pub fn scene(i: usize) -> &'static SceneDef {
    &SCENE_LIST[i.min(SCENES - 1)]
}

/// Scene index and frame inside it for a video frame.
pub fn split(global: usize) -> (usize, usize) {
    let g = global.min(TOTAL_FRAMES - 1);
    (g / SCENE_FRAMES, g % SCENE_FRAMES)
}

/// Deterministic noise in 0..1.
pub fn rnd(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77) ^ 0x27d4_eb2f;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    h as f32 / u32::MAX as f32
}

fn chars(s: &SceneDef) -> Vec<char> {
    s.text.chars().collect()
}

/// Characters of the hero line on screen at scene frame `f`.
pub fn visible(si: usize, f: usize) -> usize {
    let s = scene(si);
    let n = chars(s).len();
    match s.kind {
        Kind::Typing => {
            let p = (f as f32 / TYPE_END as f32).min(1.0);
            ((n as f32) * (TYPED_AT_CUT + (1.0 - TYPED_AT_CUT) * p)).ceil() as usize
        }
        _ => n,
    }
    .min(n)
}

/// Cell of character `i` of the hero line: x, y (top), w, h.
pub fn cell(si: usize, i: usize) -> (f32, f32, f32, f32) {
    let s = scene(si);
    let adv = s.size * MONO_ADV;
    (
        s.hero.0 + i as f32 * adv,
        s.hero.1 - s.size * 0.78,
        adv,
        s.size * 0.98,
    )
}

/// Rows of the log for a scene: (text, x, baseline y). Rows close to the hero line are left out.
pub fn log_rows(si: usize) -> Vec<(&'static str, f32, f32)> {
    let s = scene(si);
    let top = 70.0;
    let mut rows = vec![];
    for r in 0..24usize {
        let y = top + r as f32 * LOG_LINE;
        if y > H - 40.0 {
            break;
        }
        if s.kind != Kind::Collapse
            && (y - s.hero.1).abs() < s.size * 0.95 + 10.0
            && y < s.hero.1 + 30.0
        {
            continue;
        }
        if s.kind != Kind::Collapse && y > s.hero.1 && y - s.hero.1 < 40.0 {
            continue;
        }
        let line = LOG[(r + si * 2) % LOG.len()];
        rows.push((line, LOG_X, y));
    }
    rows
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub hatch: bool,
    pub locked: bool,
}

impl TrackBox {
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w * 0.5, self.y + self.h * 0.5)
    }
}

pub struct Track {
    /// Boxes in chain order; the last one is the current target.
    pub boxes: Vec<TrackBox>,
    /// Extra lines from a box to a far point (off the screen in most shots).
    pub tails: Vec<((f32, f32), (f32, f32))>,
    pub focus: (f32, f32),
    pub lock: bool,
}

fn jitter(si: usize, f: usize, k: usize, amp: f32) -> (f32, f32) {
    let a = (si * 1000 + k * 37) as u32;
    (
        (rnd(a, f as u32) - 0.5) * 2.0 * amp,
        (rnd(a + 1, f as u32 + 999) - 0.5) * 2.0 * amp,
    )
}

fn boxed(x: f32, y: f32, w: f32, h: f32, pad: f32, j: (f32, f32), hatch: bool) -> TrackBox {
    TrackBox {
        x: x - pad + j.0,
        y: y - pad + j.1,
        w: w + 2.0 * pad,
        h: h + 2.0 * pad,
        hatch,
        locked: false,
    }
}

/// Icon squares in the log gutter, as anchors for the tracker: (x, y, size).
pub fn icons(si: usize) -> Vec<(f32, f32, f32)> {
    log_rows(si)
        .iter()
        .enumerate()
        .filter(|(r, _)| rnd(si as u32 + 50, *r as u32) > 0.62)
        .map(|(_, (_, _, y))| (LOG_X - 64.0, y - 24.0, 30.0))
        .collect()
}

pub fn track(si: usize, f: usize) -> Track {
    let s = scene(si);
    let lock = (LOCK_AT..LOCK_AT + LOCK_FRAMES).contains(&f);
    let mut boxes = vec![];
    let mut tails = vec![];

    // Two anchors on gutter icons near the hero line start every chain.
    let mut anchors: Vec<(f32, f32, f32)> = icons(si);
    anchors.sort_by(|a, b| (a.1 - s.hero.1).abs().total_cmp(&(b.1 - s.hero.1).abs()));
    anchors.truncate(2);
    if matches!(s.kind, Kind::Cursor | Kind::Typing) {
        for (k, (x, y, sz)) in anchors.iter().enumerate() {
            boxes.push(boxed(
                *x,
                *y,
                *sz,
                *sz,
                5.0,
                jitter(si, f, 90 + k, 1.2),
                k == 0,
            ));
        }
    }

    match s.kind {
        Kind::Typing => {
            let ch = chars(s);
            let n = visible(si, f);
            let idx: Vec<usize> = (0..n).filter(|&i| !ch[i].is_whitespace()).collect();
            let from = idx.len().saturating_sub(CHAIN);
            for (k, &i) in idx[from..].iter().enumerate() {
                let (x, y, w, h) = cell(si, i);
                let span = if rnd(si as u32, i as u32 + 7) > 0.7 && i + 1 < n {
                    2.0
                } else {
                    1.0
                };
                let pad = 2.0 + rnd(si as u32 + 3, i as u32) * 8.0;
                let hatch = rnd(si as u32 + 9, i as u32) > 0.35;
                boxes.push(boxed(x, y, w * span, h, pad, jitter(si, f, k, 1.6), hatch));
            }
            if f >= LOCK_AT {
                // The newest box grows over the last word and locks on it.
                let last_word_start = (0..n)
                    .rev()
                    .take_while(|&i| !ch[i].is_whitespace())
                    .last()
                    .unwrap_or(0);
                let (x0, y, _, h) = cell(si, last_word_start);
                let (x1, _, w1, _) = cell(si, n.saturating_sub(1));
                if let Some(b) = boxes.last_mut() {
                    *b = boxed(x0, y, x1 + w1 - x0, h, 10.0, jitter(si, f, 77, 0.8), true);
                    b.locked = lock;
                }
            }
        }
        Kind::Cursor => {
            let (x, y, w, h) = cell(si, 0);
            boxes.push(boxed(x, y, w, h, 6.0, jitter(si, f, 1, 1.5), false));
            let (cx, cy, cw, ch) = cell(si, 2);
            let mut b = boxed(cx, cy, cw * 0.8, ch, 8.0, jitter(si, f, 2, 1.5), true);
            b.locked = lock;
            boxes.push(b);
        }
        Kind::Collapse => {
            // Fourteen boxes spread over the log fall into one point and lock as a single box.
            let p = ease_in_out((f as f32 / 11.0).min(1.0));
            let c = s.hero;
            for k in 0..14usize {
                let sx = 120.0 + rnd(77, k as u32) * (W - 240.0);
                let sy = 90.0 + rnd(78, k as u32) * (H - 180.0);
                let sw = 30.0 + rnd(79, k as u32) * 90.0;
                let sh = 26.0 + rnd(80, k as u32) * 60.0;
                let x = sx + (c.0 - 40.0 - sx) * p;
                let y = sy + (c.1 - 40.0 - sy) * p;
                let w = sw + (80.0 - sw) * p;
                let h = sh + (80.0 - sh) * p;
                let mut b = boxed(
                    x,
                    y,
                    w,
                    h,
                    0.0,
                    jitter(si, f, k, 2.5 * (1.0 - p)),
                    rnd(81, k as u32) > 0.4,
                );
                b.locked = lock;
                boxes.push(b);
            }
            if p >= 1.0 {
                boxes.truncate(1);
                boxes[0] = boxed(
                    c.0 - 40.0,
                    c.1 - 40.0,
                    80.0,
                    80.0,
                    0.0,
                    jitter(si, f, 3, 1.0),
                    true,
                );
                boxes[0].locked = lock;
            }
        }
        Kind::Done => {
            let (ix, iy) = done_icon();
            boxes.push(boxed(
                ix - 38.0,
                iy - 32.0,
                76.0,
                64.0,
                0.0,
                jitter(si, f, 1, 1.2),
                true,
            ));
            let (dx, dy) = done_label();
            let mut b = boxed(dx + 1.0, dy - 62.0, 44.0, 70.0, 3.0, jitter(si, f, 2, 1.2), false);
            b.locked = lock;
            boxes.push(b);
        }
    }

    if let Some(first) = boxes.first() {
        let c = first.center();
        let a = rnd(si as u32 + 200, 1) * std::f32::consts::TAU;
        tails.push((c, (c.0 + a.cos() * 2600.0, c.1 + a.sin() * 2600.0)));
    }
    if let Some(last) = boxes.last() {
        let c = last.center();
        let a = rnd(si as u32 + 201, 2) * std::f32::consts::TAU;
        tails.push((c, (c.0 + a.cos() * 2600.0, c.1 + a.sin() * 2600.0)));
    }

    let focus = boxes.last().map(|b| b.center()).unwrap_or(s.hero);
    Track {
        boxes,
        tails,
        focus,
        lock,
    }
}

/// Pill of the last scene: center x, y, width, height.
pub fn done_pill() -> (f32, f32, f32, f32) {
    (960.0, 540.0, 420.0, 132.0)
}

pub fn done_icon() -> (f32, f32) {
    let (cx, cy, w, _) = done_pill();
    (cx - w * 0.5 + 92.0, cy)
}

/// Baseline start of "Done".
pub fn done_label() -> (f32, f32) {
    let (cx, cy, _, _) = done_pill();
    (cx - 40.0, cy + 28.0)
}

pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Camera state for one video frame.
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub aim: (f32, f32),
    pub focus: (f32, f32),
    pub zoom: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub roll: f32,
    pub blur: f32,
    pub palette: Palette,
    pub lock: bool,
    /// Exposure kick right after a cut, 0..1.
    pub cut: f32,
    /// Fade to black at the very end, 0..1 (1 = black).
    pub out: f32,
}

pub fn shot(global: usize) -> Shot {
    let (si, f) = split(global);
    let s = scene(si);
    let t = f as f32 / (SCENE_FRAMES - 1) as f32;
    // The aim trails the target over the last four frames and leans to the hero line.
    let mut sum = (0.0, 0.0);
    for k in 0..4 {
        let p = track(si, f.saturating_sub(k)).focus;
        sum.0 += p.0;
        sum.1 += p.1;
    }
    let lag = (sum.0 / 4.0, sum.1 / 4.0);
    let hero_mid = match s.kind {
        Kind::Typing | Kind::Cursor => {
            let (x, y, w, h) = cell(si, visible(si, f).saturating_sub(1) / 2);
            (x + w * 0.5, y + h * 0.5)
        }
        _ => s.hero,
    };
    let sh = (
        (rnd(si as u32 + 400, f as u32) - 0.5) * 2.0 * s.cam.shake,
        (rnd(si as u32 + 401, f as u32) - 0.5) * 2.0 * s.cam.shake,
    );
    let aim = (
        lag.0 * 0.72 + hero_mid.0 * 0.28 + s.cam.pan.0 * (t - 0.5) + sh.0,
        lag.1 * 0.72 + hero_mid.1 * 0.28 + s.cam.pan.1 * (t - 0.5) + sh.1,
    );
    let zt = ease_out(t);
    // A slow sway keeps every shot hand-held.
    let sway = (si as f32 * 1.7 + t * 1.3).sin() * 0.015;
    Shot {
        aim,
        focus: track(si, f).focus,
        zoom: s.cam.zoom.0 + (s.cam.zoom.1 - s.cam.zoom.0) * zt,
        tilt_x: s.cam.tilt_x + sway,
        tilt_y: s.cam.tilt_y - sway * 0.7,
        roll: s.cam.roll + sway * 0.4,
        blur: s.cam.blur,
        palette: s.palette,
        lock: track(si, f).lock,
        cut: match f {
            0 => 1.0,
            1 => 0.45,
            2 => 0.15,
            _ => 0.0,
        },
        out: if si == SCENES - 1 {
            ((f as f32 - 11.0) / 3.0).clamp(0.0, 1.0)
        } else {
            0.0
        },
    }
}

/// Must match `FOCAL` in lens.sksl.
pub const FOCAL: f32 = 1.7;

type V3 = [f32; 3];

fn mat_mul_vec(cols: &[V3; 3], v: V3) -> V3 {
    [
        cols[0][0] * v[0] + cols[1][0] * v[1] + cols[2][0] * v[2],
        cols[0][1] * v[0] + cols[1][1] * v[1] + cols[2][1] * v[2],
        cols[0][2] * v[0] + cols[1][2] * v[1] + cols[2][2] * v[2],
    ]
}

/// Same rotation as `rotation()` in lens.sksl (SkSL matrices are column-major): Rz * Ry * Rx.
fn rotate(s: &Shot, v: V3) -> V3 {
    let (cx, sx) = (s.tilt_x.cos(), s.tilt_x.sin());
    let (cy, sy) = (s.tilt_y.cos(), s.tilt_y.sin());
    let (cz, sz) = (s.roll.cos(), s.roll.sin());
    let rx = [[1.0, 0.0, 0.0], [0.0, cx, sx], [0.0, -sx, cx]];
    let ry = [[cy, 0.0, -sy], [0.0, 1.0, 0.0], [sy, 0.0, cy]];
    let rz = [[cz, sz, 0.0], [-sz, cz, 0.0], [0.0, 0.0, 1.0]];
    mat_mul_vec(&rz, mat_mul_vec(&ry, mat_mul_vec(&rx, v)))
}

/// Where a logical point of the flat screen lands in the lens output (`res_w` x `res_h`
/// pixels). `None` when it is behind the camera.
pub fn project(s: &Shot, q: (f32, f32), res_w: f32, res_h: f32) -> Option<(f32, f32)> {
    let k = FOCAL * H / s.zoom;
    let (lx, ly) = ((q.0 - s.aim.0) / k, (q.1 - s.aim.1) / k);
    let eu = rotate(s, [1.0, 0.0, 0.0]);
    let ev = rotate(s, [0.0, 1.0, 0.0]);
    let p = [
        eu[0] * lx + ev[0] * ly,
        eu[1] * lx + ev[1] * ly,
        1.0 + eu[2] * lx + ev[2] * ly,
    ];
    if p[2] <= 0.05 {
        return None;
    }
    let sx = FOCAL * p[0] / p[2];
    let sy = FOCAL * p[1] / p[2];
    Some((sx * res_h + 0.5 * res_w, sy * res_h + 0.5 * res_h))
}

/// Sound events, in video frames.
pub struct Events {
    pub cuts: Vec<usize>,
    pub locks: Vec<usize>,
    pub keys: Vec<usize>,
    pub collapse: usize,
    pub done: usize,
}

pub fn events() -> Events {
    let mut e = Events {
        cuts: vec![],
        locks: vec![],
        keys: vec![],
        collapse: 0,
        done: 0,
    };
    for si in 0..SCENES {
        let g0 = si * SCENE_FRAMES;
        e.cuts.push(g0);
        e.locks.push(g0 + LOCK_AT);
        if scene(si).kind == Kind::Typing {
            for f in 1..=TYPE_END {
                if visible(si, f) > visible(si, f - 1) {
                    e.keys.push(g0 + f);
                }
            }
        }
        match scene(si).kind {
            Kind::Collapse => e.collapse = g0,
            Kind::Done => e.done = g0,
            _ => {}
        }
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_frame_has_a_target_on_the_canvas() {
        for g in 0..TOTAL_FRAMES {
            let s = shot(g);
            assert!(
                s.focus.0 > 0.0 && s.focus.0 < W && s.focus.1 > 0.0 && s.focus.1 < H,
                "frame {g}: {:?}",
                s.focus
            );
            assert!(!track(split(g).0, split(g).1).boxes.is_empty());
        }
    }

    #[test]
    fn the_camera_keeps_the_target_in_frame() {
        for g in 0..TOTAL_FRAMES {
            let s = shot(g);
            let (x, y) = project(&s, s.focus, 1920.0, 1080.0).expect("in front of the camera");
            assert!(
                x > 200.0 && x < 1720.0 && y > 150.0 && y < 930.0,
                "frame {g}: target at {x:.0},{y:.0}"
            );
        }
    }

    #[test]
    fn hero_lines_fit_the_canvas() {
        for si in 0..SCENES {
            let n = scene(si).text.chars().count();
            if n > 0 {
                let (x, _, w, _) = cell(si, n - 1);
                assert!(x + w < W - 60.0, "scene {si} too wide");
            }
        }
    }
}
