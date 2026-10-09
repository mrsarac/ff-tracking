//! Writes `out/tracks.json`: cut, lock and keystroke times for `tools/sfx.py`, plus the focus
//! point of every frame. Run from the workspace root: `cargo run -p hud --bin tracks`.
use std::fmt::Write as _;

fn list(v: &[usize]) -> String {
    let secs: Vec<String> = v
        .iter()
        .map(|f| format!("{:.4}", *f as f32 / hud::FPS as f32))
        .collect();
    format!("[{}]", secs.join(", "))
}

fn main() -> std::io::Result<()> {
    let e = hud::events();
    let mut focus = String::new();
    for g in 0..hud::TOTAL_FRAMES {
        let s = hud::shot(g);
        let _ = write!(
            focus,
            "{}[{:.1}, {:.1}]",
            if g == 0 { "" } else { ", " },
            s.focus.0,
            s.focus.1
        );
    }
    let json = format!(
        "{{\n  \"fps\": {},\n  \"frames\": {},\n  \"seconds\": {:.4},\n  \"cuts\": {},\n  \"locks\": {},\n  \"keys\": {},\n  \"collapse\": {:.4},\n  \"done\": {:.4},\n  \"focus\": [{}]\n}}\n",
        hud::FPS,
        hud::TOTAL_FRAMES,
        hud::TOTAL_FRAMES as f32 / hud::FPS as f32,
        list(&e.cuts),
        list(&e.locks),
        list(&e.keys),
        e.collapse as f32 / hud::FPS as f32,
        e.done as f32 / hud::FPS as f32,
        focus
    );
    std::fs::create_dir_all("out")?;
    std::fs::write("out/tracks.json", json)?;
    println!("wrote out/tracks.json");
    Ok(())
}
