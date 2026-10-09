"""Synthesize the whole sound track from out/tracks.json, so no third-party audio is needed.

Run from the workspace root after `cargo run --release -p hud --bin tracks`:
    python3 -I tools/sfx.py   -> out/pass/sfx.wav (stereo, 48 kHz, -14 LUFS)

Layers: screen hum, a glitch on every cut, a click per keystroke, a beep on every lock
(panned to where the box is), a riser into the collapse and a chime on "Done".
"""

import json
import re
import subprocess
import wave
from pathlib import Path

import numpy as np
from scipy.signal import butter, sosfilt

SR = 48_000
ROOT = Path(__file__).resolve().parent.parent
TRACKS = ROOT / "out" / "tracks.json"
OUT = ROOT / "out" / "pass" / "sfx.wav"
TARGET_LUFS = -14.0
rng = np.random.default_rng(11)

data = json.loads(TRACKS.read_text())
SECONDS = data["seconds"]
N = int(SR * SECONDS)
mix = np.zeros((N, 2))


def t(seconds):
    return np.arange(int(SR * seconds)) / SR


def env(n, attack, release):
    x = np.arange(n) / SR
    return np.clip(x / max(attack, 1e-4), 0, 1) * np.exp(-x / max(release, 1e-4))


def band(x, lo, hi, order=2):
    sos = butter(order, [lo, hi], btype="bandpass", fs=SR, output="sos")
    return sosfilt(sos, x)


def low(x, cutoff, order=2):
    return sosfilt(butter(order, cutoff, btype="lowpass", fs=SR, output="sos"), x)


def high(x, cutoff, order=2):
    return sosfilt(butter(order, cutoff, btype="highpass", fs=SR, output="sos"), x)


def place(sound, at, gain_db=0.0, pan=0.0):
    """Add a mono sound at `at` seconds; pan -1 (left) .. 1 (right), equal power."""
    i = int(at * SR)
    if i >= N:
        return
    s = sound[: N - i] * 10 ** (gain_db / 20)
    a = (pan + 1) * np.pi / 4
    mix[i : i + len(s), 0] += s * np.cos(a)
    mix[i : i + len(s), 1] += s * np.sin(a)


# Screen hum: mains hum with harmonics, a faint high whine, filtered air. Stereo by detune.
x = t(SECONDS)
for ch, detune in ((0, 0.0), (1, 0.35)):
    hum = sum(np.sin(2 * np.pi * (50 + detune) * k * x) / k**1.4 for k in range(1, 7))
    whine = np.sin(2 * np.pi * (7_812 + detune * 3) * x) * 0.012
    air = band(rng.standard_normal(N), 400, 3_000) * 0.05
    mix[:, ch] += (hum * 0.05 + whine + air) * 0.7

# Glitch on every cut: bit-crushed noise burst over a click.
for k, at in enumerate(data["cuts"]):
    n = int(SR * 0.05)
    noise = rng.standard_normal(n)
    crushed = np.round(noise * 3) / 3
    hold = max(2, int(rng.integers(6, 22)))
    crushed = np.repeat(crushed[::hold], hold)[:n]
    burst = high(crushed, 900) * env(n, 0.0005, 0.012)
    click = np.zeros(n)
    click[:24] = np.hanning(48)[24:] * 1.0
    pan = float(rng.uniform(-0.6, 0.6))
    place(burst * 0.5 + click, at, gain_db=-4.0 if k else -1.0, pan=pan)

# A tiny click per keystroke.
for at in data["keys"]:
    n = int(SR * 0.012)
    c = band(rng.standard_normal(n), 2_500, 9_000) * env(n, 0.0002, 0.0025)
    place(
        c,
        at + float(rng.uniform(-0.004, 0.004)),
        gain_db=-10.0,
        pan=float(rng.uniform(-0.3, 0.3)),
    )

# Lock beep: two partials and a falling data chirp, pitched along a minor pentatonic,
# panned to the locked box.
notes = [
    1318.5,
    1568.0,
    1760.0,
    2093.0,
    2349.3,
    1760.0,
    1568.0,
    2093.0,
    2637.0,
    1760.0,
    1046.5,
    2093.0,
]
focus = data["focus"]
for k, at in enumerate(data["locks"]):
    n = int(SR * 0.16)
    xx = t(0.16)
    f0 = notes[k % len(notes)]
    beep = (
        np.sin(2 * np.pi * f0 * xx) + 0.35 * np.sin(2 * np.pi * f0 * 2.01 * xx)
    ) * env(n, 0.001, 0.045)
    sweep = f0 * 2.5 * np.exp(-xx * 40)
    chirp = np.sin(2 * np.pi * np.cumsum(sweep) / SR) * env(n, 0.0005, 0.018) * 0.4
    frame = min(int(round(at * data["fps"])), len(focus) - 1)
    pan = float(np.clip((focus[frame][0] / 1920.0 - 0.5) * 1.4, -0.8, 0.8))
    place(beep + chirp, at, gain_db=-7.0, pan=pan)

# Riser into the collapse: band-passed noise sweeping up plus a rising sine, ends at "Done".
rise_len = data["done"] - data["collapse"]
n = int(SR * rise_len)
xx = t(rise_len)
p = xx / rise_len
noise = rng.standard_normal(n)
riser = np.zeros(n)
for i0 in range(0, n, 1_200):
    seg = slice(i0, min(n, i0 + 1_200))
    centre = 500 + 6_000 * p[i0] ** 2
    riser[seg] = band(noise, centre * 0.7, min(centre * 1.4, 20_000))[seg]
tone = np.sin(2 * np.pi * np.cumsum(180 + 900 * p**2) / SR) * 0.25
place((riser * 0.6 + tone) * p**1.5, data["collapse"], gain_db=-6.0)

# "Done": soft two-note chime over a sub thump; the tail runs past the fade to black.
n = int(SR * 1.4)
xx = t(1.4)
chime = (
    np.sin(2 * np.pi * 880 * xx)
    + 0.6 * np.sin(2 * np.pi * 1318.5 * xx)
    + 0.2 * np.sin(2 * np.pi * 2637 * xx)
)
chime *= env(n, 0.004, 0.35)
thump = np.sin(2 * np.pi * np.cumsum(90 * np.exp(-xx * 9) + 38) / SR) * env(
    n, 0.002, 0.16
)
place(chime * 0.55, data["done"] + 0.06, gain_db=-5.0, pan=-0.15)
place(chime * 0.55, data["done"] + 0.075, gain_db=-7.0, pan=0.2)
place(thump, data["done"], gain_db=-2.0)

# Fade the last 0.25 s with the picture.
fade = int(SR * 0.25)
mix[-fade:] *= np.linspace(1, 0, fade)[:, None]


def write(path, x):
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes((np.clip(x, -1, 1) * 32767).astype("<i2").tobytes())


def lufs(path):
    out = subprocess.run(
        ["ffmpeg", "-nostats", "-i", str(path), "-af", "ebur128", "-f", "null", "-"],
        capture_output=True,
        text=True,
    ).stderr
    return float(re.findall(r"I:\s+(-?[\d.]+) LUFS", out)[-1])


# Loudness: measure, apply the gain, then a soft clip keeps peaks under -1 dBFS.
mix /= np.max(np.abs(mix)) + 1e-9
mix *= 10 ** (-12 / 20)
write(OUT, mix)
ceiling = 10 ** (-1.6 / 20)
base = mix.copy()
gain = 10 ** ((TARGET_LUFS - lufs(OUT)) / 20)
for _ in range(3):  # the soft clip eats some loudness: correct and measure again
    mix = np.tanh(base * gain / ceiling) * ceiling
    write(OUT, mix)
    gain *= 10 ** ((TARGET_LUFS - lufs(OUT)) / 20)
print(f"wrote {OUT.relative_to(ROOT)}  {SECONDS:.2f} s  {lufs(OUT):.1f} LUFS")
