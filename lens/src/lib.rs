//! Pass 2: a virtual camera films the flat pass through `shaders/lens.sksl`.
//!
//! The shader does the physical part: perspective, depth of field focused on the tracker's
//! target, palette, bloom, chromatic aberration, LED grid and the tear on a lock. Every
//! per-frame value comes from `hud::shot`.
use fframes::{
    AudioMap, AudioTimestamp::*, AudioTrack, Color, Duration, FFramesContext,
    FFramesSyncedVideoFrame, Frame, Shader, ShaderUniforms, Svgr, SyncVideoFrameInput, Video,
};

pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;

pub struct LensVideo {
    shader: Shader,
}

impl LensVideo {
    pub fn new() -> Self {
        Self {
            shader: Shader::sksl(include_str!("../shaders/lens.sksl")),
        }
    }
}

impl Default for LensVideo {
    fn default() -> Self {
        Self::new()
    }
}

impl Video for LensVideo {
    const FPS: usize = hud::FPS;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::BLACK;

    fn duration(&self) -> Duration<'_> {
        Duration::Frames(hud::TOTAL_FRAMES)
    }

    fn audio(&self) -> AudioMap<'_> {
        AudioMap::from([AudioTrack::new("sfx.wav", Second(0.)..Eof)])
    }

    fn render_frame<'a>(&'a self, frame: Frame, ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let (si, f) = hud::split(frame.index);
        let s = hud::shot(frame.index);
        let input = SyncVideoFrameInput {
            start_from: 0.0,
            looping: false,
            editor_fallback_image: None,
        };
        // A root without <svg> makes a full render hang instead of failing (fframes 1.2.0),
        // so a missing frame draws black.
        let Some(flat) = frame.get_synced_video_frame(ctx, "flat.mp4", &input) else {
            return fframes::svgr!(
                <svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT}>
                    <rect width={WIDTH} height={HEIGHT} fill="#000000" />
                </svg>
            );
        };
        let flat = flat.into_image();
        let p = s.palette.stops();
        let uniforms = ShaderUniforms::new()
            .image("iChannel0", &flat)
            .float2("uAim", s.aim.0, s.aim.1)
            .float("uZoom", s.zoom)
            .float3("uRot", s.tilt_x, s.tilt_y, s.roll)
            .float2("uFocus", s.focus.0, s.focus.1)
            .float("uBlur", s.blur)
            .float3("uP0", p[0][0], p[0][1], p[0][2])
            .float3("uP1", p[1][0], p[1][1], p[1][2])
            .float3("uP2", p[2][0], p[2][1], p[2][2])
            .float3("uP3", p[3][0], p[3][1], p[3][2])
            .float3("uP4", p[4][0], p[4][1], p[4][2])
            .float("uLock", if s.lock { 1.0 } else { 0.0 })
            .float("uCut", s.cut)
            .float("uOut", s.out)
            .float("uSeed", (si * 31 + f) as f32);
        let layer = self.shader.draw(&frame, uniforms);
        fframes::svgr!(
            <svg xmlns="http://www.w3.org/2000/svg" width={WIDTH} height={HEIGHT}>
                <image href={layer.href()} x="0" y="0" width={WIDTH} height={HEIGHT} />
                {camera_tracker(&s, si, f)}
            </svg>
        )
    }
}

const MONO: &str = "JetBrains Mono";
const LOCK_RED: &str = "#ff3b30";
const RW: f32 = WIDTH as f32;
const RH: f32 = HEIGHT as f32;

/// The camera's own tracker, drawn sharp over the filmed screen: links between the boxes,
/// a dot running along each link, coordinate labels and brackets on the target. The
/// numbers are the real pixel positions of the boxes in this frame.
fn camera_tracker<'a>(s: &hud::Shot, si: usize, f: usize) -> Svgr<'a> {
    let t = hud::track(si, f);
    let ink = if s.palette == hud::Palette::Paper {
        "#141414"
    } else {
        "#ffffff"
    };
    let proj = |q: (f32, f32)| hud::project(s, q, RW, RH);
    let centers: Vec<Option<(f32, f32)>> = t.boxes.iter().map(|b| proj(b.center())).collect();

    let mut items: Vec<Svgr> = vec![];

    // Tails run off the frame: aim at a point on the screen plane, then extend on screen.
    for (a, b) in &t.tails {
        let mid = (a.0 + (b.0 - a.0) * 0.12, a.1 + (b.1 - a.1) * 0.12);
        if let (Some(pa), Some(pm)) = (proj(*a), proj(mid)) {
            let (dx, dy) = (pm.0 - pa.0, pm.1 - pa.1);
            let len = (dx * dx + dy * dy).sqrt().max(1e-3);
            let (ex, ey) = (pa.0 + dx / len * 3000.0, pa.1 + dy / len * 3000.0);
            items.push(fframes::svgr!(<line x1={pa.0} y1={pa.1} x2={ex} y2={ey} stroke={ink} stroke-width="1.1" opacity="0.7" />));
        }
    }

    for k in 1..centers.len() {
        if let (Some(a), Some(b)) = (centers[k - 1], centers[k]) {
            let p = (f as f32 * 0.17 + k as f32 * 0.31).fract();
            let (mx, my) = (a.0 + (b.0 - a.0) * p, a.1 + (b.1 - a.1) * p);
            items.push(fframes::svgr!(<g>
                <line x1={a.0} y1={a.1} x2={b.0} y2={b.1} stroke={ink} stroke-width="1.3" />
                <circle cx={mx} cy={my} r="3" fill={ink} />
            </g>));
        }
    }

    // Labels alternate above and below their boxes and skip a slot that is already taken.
    let mut taken: Vec<(f32, f32)> = vec![];
    for (k, (b, c)) in t.boxes.iter().zip(&centers).enumerate() {
        let below = k % 2 == 1 && !b.locked;
        let anchor = if below { (b.x, b.y + b.h) } else { (b.x, b.y) };
        let (Some(c), Some(corner)) = (*c, proj(anchor)) else {
            continue;
        };
        let y = if below { corner.1 + 24.0 } else { corner.1 - 9.0 };
        let free = b.locked
            || taken.iter().all(|(tx, ty)| (tx - corner.0).abs() > 190.0 || (ty - y).abs() > 26.0);
        let color = if b.locked { LOCK_RED } else { ink };
        items.push(fframes::svgr!(<circle cx={c.0} cy={c.1} r="3.2" fill={color} />));
        if !free {
            continue;
        }
        taken.push((corner.0, y));
        let label = if b.locked {
            format!("LOCK x: {:.0} y: {:.0}", c.0, c.1)
        } else {
            format!("x: {:.0} y: {:.0}", c.0, c.1)
        };
        items.push(fframes::svgr!(<text x={corner.0} y={y} font-family={MONO} font-weight="400" font-size="19" fill={color}>{label}</text>));
    }

    // Brackets around the target box, in screen space.
    if let Some(b) = t.boxes.last() {
        let corners = [
            (b.x, b.y),
            (b.x + b.w, b.y),
            (b.x + b.w, b.y + b.h),
            (b.x, b.y + b.h),
        ];
        let pts: Vec<(f32, f32)> = corners.iter().filter_map(|q| proj(*q)).collect();
        if pts.len() == 4 {
            let color = if t.lock { LOCK_RED } else { ink };
            let grow = if t.lock { 10.0 } else { 6.0 };
            let (cx, cy) = (
                pts.iter().map(|p| p.0).sum::<f32>() / 4.0,
                pts.iter().map(|p| p.1).sum::<f32>() / 4.0,
            );
            for (i, p) in pts.iter().enumerate() {
                let (ox, oy) = (p.0 - cx, p.1 - cy);
                let l = (ox * ox + oy * oy).sqrt().max(1e-3);
                let q = (p.0 + ox / l * grow, p.1 + oy / l * grow);
                let n1 = pts[(i + 1) % 4];
                let n0 = pts[(i + 3) % 4];
                let arm = |n: (f32, f32)| {
                    let (dx, dy) = (n.0 - p.0, n.1 - p.1);
                    let l = (dx * dx + dy * dy).sqrt().max(1e-3);
                    let a = 18.0_f32.min(l * 0.4);
                    (q.0 + dx / l * a, q.1 + dy / l * a)
                };
                let (a1, a0) = (arm(n1), arm(n0));
                items.push(fframes::svgr!(<path d={format!("M {:.1} {:.1} L {:.1} {:.1} L {:.1} {:.1}", a0.0, a0.1, q.0, q.1, a1.0, a1.1)}
                    fill="none" stroke={color} stroke-width="2.4" />));
            }
        }
    }

    fframes::svgr!(<g opacity={0.92 * (1.0 - s.out)}>{items}</g>)
}
