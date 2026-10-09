//! Pass 1: the agent's screen, flat and colorless, with the tracker drawn on top.
//!
//! White on black only. The lens pass maps brightness to each scene's palette, so the one
//! color here is pure red (#ff0000): it marks a locked box and survives the palette.
//! Drawn in `hud`'s 1920x1080 logical space at 2x, so the lens can magnify it.
use fframes::{AudioMap, Color, Duration, FFramesContext, Frame, Svgr, Video, include_media_dir};
use hud::{Kind, LOG_SIZE, TrackBox};

include_media_dir!(pub struct FlatMedia, "media");

pub const WIDTH: usize = 3840;
pub const HEIGHT: usize = 2160;
const SCALE: f32 = 2.0;

const MONO: &str = "JetBrains Mono";
const WHITE: &str = "#ffffff";
const LOG_GRAY: &str = "#5c5c5c";
const UI_GRAY: &str = "#3a3a3a";
const RED: &str = "#ff0000";

#[derive(Default)]
pub struct FlatVideo;

impl FlatVideo {
    pub fn new() -> Self {
        Self
    }
}

impl Video for FlatVideo {
    const FPS: usize = hud::FPS;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Frames(hud::TOTAL_FRAMES)
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::none()
    }

    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let (si, f) = hud::split(frame.index);
        let s = hud::scene(si);
        let content = match s.kind {
            Kind::Typing | Kind::Cursor => hero(si, f),
            Kind::Collapse => Svgr::empty(),
            Kind::Done => done(f),
        };
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT}>
                <defs>
                    <pattern id="hatch" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
                        <rect x="0" y="0" width="2.6" height="7" fill="#ffffff" />
                    </pattern>
                    <pattern id="hatch-red" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
                        <rect x="0" y="0" width="3.2" height="7" fill="#ff0000" />
                    </pattern>
                </defs>
                <rect width={WIDTH} height={HEIGHT} fill="#000000" />
                <g transform={format!("scale({SCALE})")}>
                    {chrome(si, f)}
                    {log(si)}
                    {content}
                    {tracker(si, f)}
                </g>
            </svg>
        )
    }
}

/// Window title and status bar: tiny, out of focus most of the time, but they give the
/// tilted camera something to blur at the edges.
fn chrome<'a>(si: usize, f: usize) -> Svgr<'a> {
    let g = si * hud::SCENE_FRAMES + f;
    let tokens = 3_120 + g * 9 + (hud::rnd(5, g as u32) * 7.0) as usize;
    let status = format!(
        "tokens {}  ·  {:>2} ms/frame  ·  frame {:03}/{}  ·  scene {:02}/{}",
        group(tokens),
        10 + (hud::rnd(6, g as u32) * 5.0) as usize,
        g + 1,
        hud::TOTAL_FRAMES,
        si + 1,
        hud::SCENES
    );
    let progress = 1720.0 * (g + 1) as f32 / hud::TOTAL_FRAMES as f32;
    fframes::svgr!(
        <g font-family={MONO} font-weight="400" font-size="18" fill={LOG_GRAY}>
            <circle cx="104" cy="30" r="7" fill={UI_GRAY} />
            <circle cx="128" cy="30" r="7" fill={UI_GRAY} />
            <circle cx="152" cy="30" r="7" fill={UI_GRAY} />
            <text x="190" y="36">"agent — session 0x4f2a  ·  ~/ff-tracking"</text>
            <rect x="100" y="48" width="1720" height="1" fill={UI_GRAY} />
            <rect x="100" y="1016" width="1720" height="1" fill={UI_GRAY} />
            <rect x="100" y="1016" width={progress} height="3" fill={LOG_GRAY} />
            <text x="100" y="1050">{status}</text>
        </g>
    )
}

fn group(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn log<'a>(si: usize) -> Svgr<'a> {
    let rows: Vec<Svgr> = hud::log_rows(si)
        .into_iter()
        .map(|(text, x, y)| {
            let fill = if text.starts_with('›') {
                "#7a7a7a"
            } else {
                LOG_GRAY
            };
            fframes::svgr!(<text x={x} y={y} fill={fill}>{text}</text>)
        })
        .collect();
    let icons: Vec<Svgr> = hud::icons(si)
        .into_iter()
        .enumerate()
        .map(|(k, (x, y, sz))| {
            // Three kinds of gutter icon: dots, a page, a ring.
            let inner = match k % 3 {
                0 => fframes::svgr!(<g fill="#8a8a8a">
                    <circle cx={x + 9.0} cy={y + 15.0} r="2.6" />
                    <circle cx={x + 15.0} cy={y + 15.0} r="2.6" />
                    <circle cx={x + 21.0} cy={y + 15.0} r="2.6" />
                </g>),
                1 => fframes::svgr!(<g fill="none" stroke="#8a8a8a" stroke-width="2">
                    <rect x={x + 9.0} y={y + 6.0} width="12" height="18" rx="1.5" />
                    <line x1={x + 12.0} y1={y + 12.0} x2={x + 18.0} y2={y + 12.0} />
                    <line x1={x + 12.0} y1={y + 17.0} x2={x + 18.0} y2={y + 17.0} />
                </g>),
                _ => fframes::svgr!(<circle cx={x + 15.0} cy={y + 15.0} r="7" fill="none" stroke="#8a8a8a" stroke-width="2.4" />),
            };
            fframes::svgr!(<g>
                <rect x={x} y={y} width={sz} height={sz} rx="7" fill="#151515" stroke="#4a4a4a" stroke-width="1.5" />
                {inner}
            </g>)
        })
        .collect();
    fframes::svgr!(
        <g font-family={MONO} font-weight="400" font-size={LOG_SIZE}>
            {rows}
            {icons}
        </g>
    )
}

/// The hero line, one glyph per cell so the tracker's coordinates are exact.
fn hero<'a>(si: usize, f: usize) -> Svgr<'a> {
    let s = hud::scene(si);
    let n = hud::visible(si, f);
    let glyphs: Vec<Svgr> = s
        .text
        .chars()
        .take(n)
        .enumerate()
        .map(|(i, c)| {
            let (x, _, _, _) = hud::cell(si, i);
            // The newest glyph arrives overexposed and settles.
            let fresh = n - 1 - i < 2 && f <= hud::TYPE_END && s.kind == Kind::Typing;
            let fill = if fresh { "#ffffff" } else { "#e9e9e9" };
            fframes::svgr!(<text x={x} y={s.hero.1} fill={fill}>{c.to_string()}</text>)
        })
        .collect();
    let col = if s.kind == Kind::Cursor { 2 } else { n };
    let (cx, cy, cw, ch) = hud::cell(si, col);
    let blink = f <= hud::TYPE_END || (f / 3) % 2 == 0;
    let cursor = if blink {
        fframes::svgr!(<rect x={cx + cw * 0.12} y={cy} width={cw * 0.62} height={ch} fill={WHITE} />)
    } else {
        Svgr::empty()
    };
    fframes::svgr!(
        <g font-family={MONO} font-weight="700" font-size={s.size}>
            {glyphs}
            {cursor}
        </g>
    )
}

fn done<'a>(f: usize) -> Svgr<'a> {
    let (cx, cy, w, h) = hud::done_pill();
    let (ix, iy) = hud::done_icon();
    let (lx, ly) = hud::done_label();
    let grow = hud::ease_out((f as f32 / 5.0).min(1.0));
    let pw = 80.0 + (w - 80.0) * grow;
    let ph = 80.0 + (h - 80.0) * grow;
    let label_opacity = ((f as f32 - 2.0) / 3.0).clamp(0.0, 1.0);
    fframes::svgr!(
        <g>
            <rect x={cx - pw * 0.5} y={cy - ph * 0.5} width={pw} height={ph} rx={ph * 0.5}
                fill="#1d1d1d" stroke={WHITE} stroke-width="4" />
            <rect x={cx - pw * 0.5 - 14.0} y={cy - ph * 0.5 - 14.0} width={pw + 28.0} height={ph + 28.0} rx={ph * 0.5 + 14.0}
                fill="none" stroke="#6a6a6a" stroke-width="1.5" />
            <g opacity={label_opacity}>
                <circle cx={ix} cy={iy} r="30" fill="#f2f2f2" />
                <path d={format!("M {} {} l 10 11 l 20 -24", ix - 14.0, iy + 1.0)} fill="none" stroke="#111111"
                    stroke-width="7" stroke-linecap="round" stroke-linejoin="round" />
                <text x={lx} y={ly} font-family={MONO} font-weight="700" font-size="80" fill={WHITE}>"Done"</text>
            </g>
        </g>
    )
}

/// The tracker's boxes, on the screen itself so they take the LED grid and the blur.
/// Labels, links and the crosshair belong to the camera and are drawn by the lens pass.
fn tracker<'a>(si: usize, f: usize) -> Svgr<'a> {
    let boxes: Vec<Svgr> = hud::track(si, f).boxes.iter().map(tracker_box).collect();
    fframes::svgr!(<g>{boxes}</g>)
}

fn tracker_box<'a>(b: &TrackBox) -> Svgr<'a> {
    let stroke = if b.locked { RED } else { WHITE };
    // Hatch in difference mode: stripes turn dark where they cross a glyph, like the reference.
    let fill = if !b.hatch {
        Svgr::empty()
    } else if b.locked {
        fframes::svgr!(<rect x={b.x} y={b.y} width={b.w} height={b.h} fill="url(#hatch-red)" />)
    } else {
        fframes::svgr!(<g style="mix-blend-mode:difference">
            <rect x={b.x} y={b.y} width={b.w} height={b.h} fill="url(#hatch)" />
        </g>)
    };
    fframes::svgr!(
        <g>
            {fill}
            <rect x={b.x} y={b.y} width={b.w} height={b.h} fill="none" stroke={stroke} stroke-width="2.2" />
        </g>
    )
}
