"""Build the README images from rendered frames.

Run from the workspace root after tools/render.sh:
    python3 -I tools/readme_media.py
Needs: Pillow, ffmpeg, img2webp (libwebp). Writes docs/media/{stages.jpg, shots.jpg, hero.webp}.
"""

import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "docs" / "media"
WORK = ROOT / "frames" / "readme"
FONT = ImageFont.truetype(str(ROOT / "media" / "JetBrainsMono-Bold.ttf"), 30)
SMALL = ImageFont.truetype(str(ROOT / "media" / "JetBrainsMono-Regular.ttf"), 24)
STAGE_FRAME = 81
STAGES = [
    ("flat", "1  flat pass"),
    ("0", "2  perspective"),
    ("1", "3  + depth of field"),
    ("2", "4  + palette, aberration"),
    ("3", "5  + bloom, LED grid"),
    ("4", "6  + tear, grain, camera tracker"),
]
SHOTS = [
    "›", "thinking…", "reading 14 files", "src/render.rs",
    "tool_call: render_frame()", "attention → token 4812", "not just completing —", "— composing",
    "verifying 180 frames", "✓ 0 problems", "collapse", "Done",
]


def run(cmd, env=None):
    subprocess.run(cmd, check=True, cwd=ROOT, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def label(img, text, font=FONT):
    d = ImageDraw.Draw(img, "RGBA")
    h = font.size + 30
    d.rectangle([0, 0, img.width, h], fill=(0, 0, 0, 150))
    d.text((22, 14), text, font=font, fill=(255, 255, 255, 255))
    return img


def grid(tiles, cols, w, h):
    rows = (len(tiles) + cols - 1) // cols
    out = Image.new("RGB", (cols * w, rows * h), "black")
    for i, t in enumerate(tiles):
        out.paste(t, ((i % cols) * w, (i // cols) * h))
    return out


def stages():
    import os

    tiles = []
    for key, text in STAGES:
        d = WORK / f"stage-{key}"
        if key == "flat":
            run(["target/release/flat", "frame", str(STAGE_FRAME), "-o", str(d)])
        else:
            run(["target/release/lens", "frame", str(STAGE_FRAME), "-o", str(d)], env={**os.environ, "LENS_STAGE": key})
        img = Image.open(d / f"{STAGE_FRAME}.png").convert("RGB").resize((960, 540), Image.LANCZOS)
        tiles.append(label(img, text))
    grid(tiles, 3, 960, 540).save(OUT / "stages.jpg", quality=88)


def shots():
    frames = [s * 15 + 6 for s in range(12)]
    run(["target/release/lens", "frame", ",".join(map(str, frames)), "-o", str(WORK / "shots")])
    tiles = []
    for i, f in enumerate(frames):
        img = Image.open(WORK / "shots" / f"{f}.png").convert("RGB").resize((640, 360), Image.LANCZOS)
        tiles.append(label(img, f"{i + 1:02d}  {SHOTS[i]}", SMALL))
    grid(tiles, 4, 640, 360).save(OUT / "shots.jpg", quality=86)


def hero():
    """Animated WebP (GitHub renders it inline): 960 px, 20 fps, about 4 MB."""
    d = WORK / "hero"
    if d.exists():
        for p in d.iterdir():
            p.unlink()
    d.mkdir(parents=True, exist_ok=True)
    run(["ffmpeg", "-y", "-i", "out/tracking.mp4", "-vf", "fps=20,scale=960:-1:flags=lanczos", str(d / "f%03d.png")])
    frames = sorted(str(p) for p in d.glob("f*.png"))
    run(["img2webp", "-loop", "0", "-lossy", "-q", "72", "-m", "6", "-d", "50", *frames, "-o", str(OUT / "hero.webp")])


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    WORK.mkdir(parents=True, exist_ok=True)
    for step in sys.argv[1:] or ["stages", "shots", "hero"]:
        globals()[step]()
    for p in sorted(OUT.iterdir()):
        print(f"{p.relative_to(ROOT)}  {p.stat().st_size / 1e6:.1f} MB")
