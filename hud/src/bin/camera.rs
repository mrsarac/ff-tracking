//! Writes `out/camera.json`: per-frame camera state and where the tracked target lands
//! in the 1920x1080 output, for scoring (pan, sweeps) in Slab.
use std::fmt::Write as _;

fn main() -> std::io::Result<()> {
    let mut rows = String::new();
    for g in 0..hud::TOTAL_FRAMES {
        let s = hud::shot(g);
        let fp = hud::project(&s, s.focus, hud::W, hud::H).unwrap_or((960.0, 540.0));
        // a fixed point of the flat screen (its centre) shows how the whole image drifts
        let cp = hud::project(&s, (hud::W * 0.5, hud::H * 0.5), hud::W, hud::H).unwrap_or((960.0, 540.0));
        let _ = write!(
            rows,
            "{}{{\"aim\":[{:.1},{:.1}],\"zoom\":{:.3},\"tilt_x\":{:.4},\"tilt_y\":{:.4},\"roll\":{:.4},\"lock\":{},\"focus_screen\":[{:.1},{:.1}],\"centre_screen\":[{:.1},{:.1}]}}",
            if g == 0 { "" } else { ",\n" },
            s.aim.0, s.aim.1, s.zoom, s.tilt_x, s.tilt_y, s.roll, s.lock, fp.0, fp.1, cp.0, cp.1
        );
    }
    std::fs::create_dir_all("out")?;
    std::fs::write("out/camera.json", format!("{{\"fps\": {}, \"frames\": [\n{}\n]}}\n", hud::FPS, rows))?;
    println!("wrote out/camera.json");
    Ok(())
}
