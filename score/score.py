"""The ff-tracking score: 6 s of dark electronic, 120 BPM, F minor, made in Slab.

Slab (https://github.com/nooga/slab) is a macOS DAW whose machines are code;
this script writes a Slab project with its Python kit, slabkit, and renders it
headless. The shots cut every 0.5 s, one beat at 120 BPM. Every event comes
from out/tracks.json (hud's `tracks` bin): a kick on each cut, a crushed
burst on each cut, a tick per keystroke, two blips per lock on random notes
of F minor, a kick roll, glitch stutter and a two-octave bass dive through
the collapse, a boom and a chord on "Done". The camera comes from
out/camera.json (hud's `camera` bin), so the mix moves with the lens:

- the tracked box's x on screen pans the lock blips and the keystrokes
- the slide of the filmed screen across the frame pans the pad, riser,
  glitch, hum and echoes
- each shot's zoom push opens the bass filter and brightens the hats
- the two red lock frames crush the bass and the pad, as the image tears

The bass changes saturation on every shot (tube, tape, transformer, diode,
fuzz, rail, valve, fold), level-matched. The whole mix drops to a crushed
sampler (Slab's Era, 6 down to 3 bits) for the "— composing" shot. A
transformer hum on worn tape runs underneath. Everything below 30 Hz is cut.

    PYTHONPATH=$SLAB/tools python3 score/score.py --render

writes score/ff_tracking.slab and out/pass/score.wav; tools/score.sh does
the whole pipeline. Open the project in Slab to edit it by hand, but this
script rewrites it.
"""
import argparse
import json
import math
import os
import random

from slabkit import Song, fx

ap = argparse.ArgumentParser()
HERE = os.path.dirname(os.path.abspath(__file__))
ap.add_argument("--video", default=os.path.dirname(HERE), help="the ff-tracking checkout, with out/tracks.json and out/camera.json")
ap.add_argument("--render", action="store_true")
ap.add_argument("--wav", default=None, help="where to render (default: out/pass/score.wav)")
ap.add_argument("--stems", action="store_true")
args = ap.parse_args()
ev = json.load(open(os.path.join(args.video, "out", "tracks.json")))
cam = json.load(open(os.path.join(args.video, "out", "camera.json")))["frames"]

FPS = ev["fps"]
B = 2.0                                # beats per second at 120 BPM
NF = len(cam)
SHOT = NF // len(ev["cuts"])           # frames per shot
CUTS = [t * B for t in ev["cuts"]]
LOCKS = [t * B for t in ev["locks"]]
KEYS = [t * B for t in ev["keys"]]
COLLAPSE, DONE = ev["collapse"] * B, ev["done"] * B
rng = random.Random(4812)


def beat(g):
    return g / FPS * B


def clamp(v, lo, hi):
    return max(lo, min(hi, v))


def box_x(g):
    """Where the tracked box sits on screen, -1 (left edge) .. 1 (right)."""
    return (cam[g]["focus_screen"][0] - 960) / 960


def sweep_x(g):
    """Where the centre of the filmed screen sits: it slides across every shot."""
    return (cam[g]["centre_screen"][0] - 960) / 960


def push(g):
    """The zoom relative to the start of its shot: 1 at the cut, up to ~1.9."""
    return cam[g]["zoom"] / cam[g - g % SHOT]["zoom"]


def lane(fn, frm=0, to=NF, step=1):
    return [(beat(g), fn(g)) for g in range(frm, to, step)]


song = Song("FF Tracking", bpm=120, key="F minor")
song.section("BOOT", 1)
song.section("TRACE", 1)
song.section("LOCK", 1)
song.section("TAIL", 1)


def clip(track):
    return track.clip(None, bars=4, at_bar=0)


# ---- the whole mix runs through one group: Era sits on it, clean until the
# "— composing" shot drops it to an SP-1200 gone wrong for one beat.
ERA_AT, ERA_END = 7.0, 8.0
mix = song.bus("MIX", fx=[fx("era", "sp1200", mix=0, out=-1.5)])
mix.automate("fx0:mix", (0, 0, "hold"), (ERA_AT, 1, "hold"), (ERA_END, 0, "hold"))
mix.automate("fx0:bits", (0, 12, "hold"), (ERA_AT, 6, "curve", -0.4), (ERA_END - 0.05, 3, "hold"), (ERA_END, 12, "hold"))
mix.automate("fx0:rate", (0, 26040, "hold"), (ERA_AT, 11000, "curve", -0.4), (ERA_END - 0.05, 3500, "hold"), (ERA_END, 26040, "hold"))
mix.automate("fx0:filter", (0, 11000, "hold"), (ERA_AT, 6000), (ERA_END - 0.05, 2500, "hold"), (ERA_END, 11000, "hold"))

# ---- returns ---------------------------------------------------------------
verb = song.bus("VERB", output=mix, fx=[
    fx("verb2", "big-plate-hall", mix=1, decay=2.6, predelay=0.012, lowcut=380,
       tone=9000, damp=6000, width=1.5),
    fx("eq2", hpf_on="ON", hpf_hz=320, p1_hz=600, p1_db=-2, hs_hz=9000, hs_db=-3),
])
# the room opens up as the agent closes in on "Done"
verb.automate("volume", (0, 0.8), (COLLAPSE, 1.0), (DONE, 1.2))
echo = song.bus("ECHO", output=mix, fx=[
    fx("delay2", sync="SYNC", div="1/16", mode="PING", fb=0.35, lowcut=700,
       damp=6500, char="DIGITAL", mix=1, width=1.4),
    fx("eq2", hpf_on="ON", hpf_hz=500, p2_hz=3000, p2_db=-2, hs_hz=7000, hs_db=-4),
])
echo.automate("fx0:fb", (0, 0.3), (COLLAPSE, 0.62), (DONE, 0.25, "hold"))
echo.automate("pan", *lane(lambda g: clamp(-sweep_x(g) * 0.5, -0.5, 0.5), step=2))

# ---- drums -----------------------------------------------------------------
drums = song.bus("DRUM BUS", output=mix, fx=[
    fx("eq2", hpf_on="ON", hpf_hz=30),
    fx("sat2", "tape-slam", drive=11, mix=0.5, out=-2.5),
    fx("bus2", "drum-bus", thresh=-18, makeup=2, color=0.3, mix=0.8),
])
kick = song.track("KICK", "drum2", "909-punch", volume=0.7, output=drums,
    params=dict(kick_tune=53, kick_sweep=10, kick_bend=0.03, kick_decay=0.28,
                kick_click=0.55, kick_drive=3.2, kick_level=1, master_drive=1.3),
    fx=[fx("eq2", hpf_on="ON", hpf_hz=30, p1_hz=330, p1_db=-4, p1_q=0.8,
           p2_hz=3500, p2_db=2)])
kc = clip(kick)
for b in CUTS[1:10]:          # four on the floor from the first typed shot to "✓"
    kc.note("C2", b, 0.2, 124 if b in (1.0, 5.0, 9.0) else 112)
for i in range(8):            # collapse: a 16th roll that swells into Done
    kc.note("C2", COLLAPSE + i * 0.125, 0.1, 70 + i * 7)
# the roll falls in pitch with the boxes
kick.automate("kick_tune", (0, 53, "hold"), (COLLAPSE, 70), (DONE - 0.05, 40, "hold"))

boom = song.track("BOOM", "drum2", "808-boom", volume=0.75, output=mix,
    params=dict(kick_tune=43.65, kick_sweep=12, kick_bend=0.06, kick_decay=1.6,
                kick_click=0.4, kick_drive=4.5, kick_level=1, master_drive=1.6),
    fx=[fx("eq2", hpf_on="ON", hpf_hz=30, p1_hz=900, p1_db=3, p1_q=0.7, hs_hz=6000, hs_db=-3),
        fx("sat2", "iron-bass", drive=10, mix=0.5, out=-2)])
bc = clip(boom)
bc.note("C2", 0.0, 1.0, 70)      # the cursor shot: a low knock to open
bc.note("C2", DONE, 2.0, 127)    # "Done"
boom.automate("kick_decay", (0, 0.45, "hold"), (DONE - 0.25, 1.4, "hold"))

clap = song.track("CLAP", "drum2", "gated-snare-kit", volume=0.85, output=drums,
    params=dict(snare_tune=210, snare_decay=0.16, snare_snap=0.9, snare_tone=3200,
                clap_level=0.8, clap_tone=1400, clap_decay=0.2, master_drive=2.2),
    fx=[fx("eq2", hpf_on="ON", hpf_hz=180, p1_hz=500, p1_db=-3, p2_hz=2200, p2_db=2,
           hs_hz=9000, hs_db=-4),
        fx("char2", "smush-1200", mix=0.35)])
cc = clip(clap)
for b in (3.0, 5.0, 7.0, 9.0):
    cc.note("D2", b, 0.2, 118)
    cc.note("D#2", b, 0.2, 100)
cc.note("D2", 8.75, 0.1, 70)
clap.send(verb, -11)
# the crush grows over the piece
clap.automate("fx1:mix", (0, 0.2), (COLLAPSE, 0.6))

hats = song.track("HATS", "drum2", "crisp-hat-kit", volume=1.2, output=drums,
    params=dict(hat_tone=0.8, hat_tune=1.2, hat_chdec=0.03, hat_ohdec=0.16, hat_level=1.1),
    fx=[fx("eq2", hpf_on="ON", hpf_hz=900, p2_hz=6000, p2_db=-2, hs_hz=9000, hs_db=-5)])
hc = clip(hats)
for i in range(4, 40):           # 16ths from beat 1 to the collapse
    hc.note("F#2", i * 0.25, 0.05, 92 if i % 4 == 2 else 58 + (i % 2) * 10)
for b in (5.5, 6.5, 7.5, 8.5, 9.5):
    hc.note("A#2", b, 0.2, 84)
# ping-pong, leaning the way the camera slides
hats.automate("pan", *[(i * 0.25 - 0.01, clamp((0.45 if i % 2 else -0.45) - sweep_x(round(i * 0.25 / B * FPS)) * 0.3, -0.8, 0.8), "hold")
                       for i in range(4, 40)])
# each zoom push brightens and tightens them
hats.automate("hat_tone", *lane(lambda g: clamp(0.75 + (push(g) - 1) * 0.9, 0.6, 1.4), SHOT, 10 * SHOT, 2))

# ---- glitch ----------------------------------------------------------------
# One per keystroke: a crushed tick, panned to the glyph being typed.
ticks = song.track("KEYS", "drum2", "lofi-kit", volume=1.2, output=drums,
    params=dict(hat_tune=1.6, hat_tone=1.4, hat_chdec=0.02, snare_tune=420,
                snare_decay=0.05, snare_sdec=0.03, snare_snap=1, snare_tone=5000,
                master_drive=2.5),
    fx=[fx("char2", "smush-1200", ltype="80s", lofi=0.85, noise=0, mix=1, makeup=0),
        fx("eq2", hpf_on="ON", hpf_hz=1000, p2_hz=4000, p2_db=-2, hs_hz=10000, hs_db=-6),
        fx("comp2", thresh=-24, ratio=10, atk=0.0002, rel=0.04, makeup=9)])
tc = clip(ticks)
for i, b in enumerate(KEYS):
    tc.note("F#2" if i % 3 else "D2", b, 0.04, rng.randint(70, 110))
ticks.send(echo, -20)
ticks.automate("pan", *lane(lambda g: clamp(box_x(g) * 1.6, -0.85, 0.85)))
# the crush gets coarser as the agent speeds up
ticks.automate("fx0:lofi", (0, 0.6), (COLLAPSE, 1.0))

# On every cut: a bit-crushed noise burst through sample-and-hold filter
# steps, then a stutter roll through the collapse.
glitch = song.track("GLITCH", "concoction", "noise-sweep", volume=1.25, output=mix,
    params=dict(n_level=1, n_color=0.3, f_mode="BP", f_cut=2400, f_res=0.55,
                f_key=1, e1_a=0.0005, e1_d=0.07, e1_s=0, e1_r=0.02,
                e3_a=0.0005, e3_d=0.1, e3_s=0, m1_src="OFF",
                l1_shape="S&H", l1_sync="1/32", l1_mode="RETRIG",
                m2_src="LFO1", m2_dst="CUTOFF", m2_amt=0.8, level=0.8),
    fx=[fx("char2", "smush-1200", ltype="80s", lofi=1, noise=0, mix=1, makeup=6),
        fx("sat2", "fold-wreck", drive=18, mix=0.5, out=-1),
        fx("eq2", hpf_on="ON", hpf_hz=500, p2_hz=3200, p2_db=-3, hs_hz=11000, hs_db=-4)])
gc = clip(glitch)
for k, b in enumerate(CUTS[:10]):
    gc.note(rng.choice(["F5", "C6", "Ab5", "Eb6"]), b, 0.15, 124 if k == 0 else 112)
b, step, p = COLLAPSE, 0.125, 72  # collapse: 32nds accelerating to 64ths, climbing
while b < DONE - 0.02:
    gc.note(p, b, step * 0.7, min(127, 96 + int(40 * (b - COLLAPSE))))
    b += step
    step = max(0.0625, step * 0.9)
    p += 1
glitch.send(echo, -12)
# each burst flies the opposite way to the screen's slide, the roll spins
glitch.automate("pan", *[(max(0.0, b - 0.02), clamp(-sweep_x(round(b / B * FPS) + 1) * 1.2, -0.85, 0.85), "hold")
                         for b in CUTS[:10]],
                *[(COLLAPSE + i / 16, 0.75 * math.sin(i * 1.9), "hold") for i in range(16)])

# ---- bass ------------------------------------------------------------------
# Rolling reese: three 16ths after each kick; sub an octave down, growl in the
# 700-1000 Hz band, ducked by the kick.
bass = song.track("REESE", "concoction", "reese", volume=0.8, output=mix,
    params=dict(sub_on="ON", sub_shape="SINE", sub_oct=-1, sub_level=0.3,
                a_pos=0.6667, b_pos=0.6667, a_fine=-18, b_fine=18,
                f_mode="LP24", f_cut=420, f_res=0.4, f_drive=0.9, f_env=0.45,
                e2_a=0.0005, e2_d=0.11, e2_s=0.1, e2_r=0.05,
                e1_a=0.002, e1_d=0.2, e1_s=0.7, e1_r=0.04, level=0.7),
    fx=[fx("eq2", hpf_on="ON", hpf_hz=32, ls_hz=80, ls_db=-5, p1_hz=850, p1_db=6, p1_q=0.7,
           p2_hz=2500, p2_db=2, hs_hz=6000, hs_db=-6),
        fx("sat2", "iron-bass", drive=16, mix=0.75, out=0),
        fx("char2", "smush-1200", ltype="90s", lofi=0.5, noise=0, mix=0),
        fx("comp2", key=kick, thresh=-26, ratio=6, knee=4, atk=0.001, rel=0.11),
        fx("geq8", "flat", t1="LC48", f1=30, t5="BELL", f5=1200, b5=2, q5=0.8,
           t8="HC12", f8=6500)])
# Each shot gets its own saturation, its output trimmed so the level holds.
GRIT = {1: ("TUBE", 20, 0), 2: ("TAPE", 24, -3), 3: ("XFMR", 26, 3), 4: ("DIODE", 18, 2),
        5: ("FUZZ", 16, -5), 6: ("RAIL", 22, -4), 7: ("VALVE", 28, -5.5), 8: ("FOLD", 14, 1),
        9: ("FUZZ", 26, -6), 10: ("FOLD", 32, -2), 11: ("TUBE", 12, 0)}
bass.automate("fx1:mode", *[(b, m, "hold") for b, (m, _, _) in GRIT.items()])
bass.automate("fx1:drive", *[(b, d, "hold") for b, (_, d, _) in GRIT.items()])
bass.automate("fx1:out", *[(b, o, "hold") for b, (_, _, o) in GRIT.items()])
roots = {1: "F1", 2: "F1", 3: "F1", 4: "F1", 5: "Db1", 6: "Db1", 7: "Eb1", 8: "Eb1", 9: "C1"}
bcl = clip(bass)
for bt, root in roots.items():
    for j, off in enumerate((0.25, 0.5, 0.75)):
        bcl.note(root, bt + off, 0.2, 112 if j == 0 else 96)
        if bt in (4, 8) and j == 2:          # an octave flick into each change
            bcl.notes[-1]["pitch"] += 12
# the collapse: one note that dives two octaves into the Done boom
bcl.note("C2", COLLAPSE, DONE - COLLAPSE, 118)
dive = bcl.notes[-1]
bcl.bend([(0.0, 0.0), (0.95, -24.0, "curve", 0.5)], notes=lambda n: n is dive)
bcl.note("F1", DONE, 1.0, 120)


def reese_cut(g):
    b = beat(g)
    if b >= DONE:
        return 260
    if b >= COLLAPSE:
        return 2600 * (180 / 2600) ** ((b - COLLAPSE) / (DONE - COLLAPSE)) ** 0.7
    base = 520 * (2400 / 520) ** clamp((b - 1) / 8, 0, 1) ** 1.4
    return clamp(base * push(g) ** 1.6, 200, 6000)  # each zoom push opens it


bass.automate("f_cut", *lane(reese_cut))


def lock_tear(g):
    """1 on the two red lock frames of every shot, else 0."""
    return 1.0 if cam[g]["lock"] else 0.0


# the lock frames tear the reese into bits, then the collapse crushes it whole
bass.automate("fx2:mix", *[(beat(g), 0.7 * lock_tear(g), "hold") for g in range(0, 10 * SHOT)],
              (COLLAPSE, 0.0), (COLLAPSE + 0.6, 1.0, "hold"), (DONE, 0.0, "hold"))
bass.automate("fx2:lofi", (0, 0.5, "hold"), (COLLAPSE, 0.3), (DONE - 0.05, 1.0, "hold"), (DONE, 0.4, "hold"))

# ---- harmony ---------------------------------------------------------------
pad = song.track("PAD", "concoction", "vowel-pad", volume=1.25, output=mix,
    params=dict(e1_a=0.25, e1_r=0.6, f_mode="LP24", f_cut=600, f_res=0.25, level=1.0),
    fx=[fx("eq2", hpf_on="ON", hpf_hz=180, p1_hz=400, p1_db=-3, hs_hz=10000, hs_db=-2),
        fx("tape2", "vhs-rental", wear=0.4, hiss=0.15, mix=0.7),
        fx("chorus2", "juno-ii-deep", mix=0.6),
        fx("char2", "smush-1200", ltype="80s", lofi=0.7, noise=0, mix=0),
        fx("comp2", key=kick, thresh=-30, ratio=8, atk=0.002, rel=0.16)])
pc = clip(pad)
pc.chord(["F3", "Ab3", "C4", "Eb4", "G4"], 0.0, 5.0, 84)       # Fm9
pc.chord(["Db3", "F3", "Ab3", "C4", "G4"], 5.0, 2.0, 84)       # Dbmaj7#11
pc.chord(["Eb3", "G3", "Bb3", "C4", "F4"], 7.0, 2.0, 84)       # Eb6/9
pc.chord(["C3", "E3", "G3", "Bb3", "Db4"], 9.0, 1.0, 88)       # C7b9
pc.chord(["C3", "E3", "G3", "Bb3", "Db4"], COLLAPSE, DONE - COLLAPSE, 84)
pc.converge("C2", COLLAPSE, DONE, tension=0.4)                 # 14 boxes fall into one
pc.chord(["F3", "C4", "Eb4", "G4", "Ab4"], DONE, 1.5, 100)     # Fm(add9) on Done
pad.ramp("f_cut", 0, 10, 500, 3600, tension=-0.5)
pad.automate("pan", *lane(lambda g: clamp(sweep_x(g) * 0.55, -0.6, 0.6), step=2))
pad.automate("fx3:mix", *[(beat(g), 0.6 * lock_tear(g), "hold") for g in range(0, NF)])
pad.send(verb, -5)

# ---- background: transformer hum through a worn tape ----------------------
# A buzzing F (87 Hz and its harmonics), a slow LFO through the filter, then
# the tape's wow, flutter and dropouts; it drifts with the screen.
hum = song.track("HUM", "concoction", None, volume=0.55, output=mix,
    params=dict(a_table="DRIVE", a_pos=0.7, b_on="ON", b_table="BASIC", b_pos=0.3,
                b_oct=1, b_fine=7, b_level=0.35, f_mode="LP12", f_cut=700, f_res=0.35,
                e1_a=0.15, e1_s=1, e1_r=0.4, l1_shape="SINE", l1_rate=0.35, l1_mode="FREE",
                m1_src="LFO1", m1_dst="CUTOFF", m1_amt=0.35,
                l2_shape="SINE", l2_rate=5.5, l2_mode="FREE", m2_src="LFO2", m2_dst="A LVL", m2_amt=0.12,
                level=0.8),
    fx=[fx("tape2", "chewed", hum=0.5, hiss=0.25, drops=0.35, wow=0.75, flutter=0.5, mix=1),
        fx("eq2", hpf_on="ON", hpf_hz=100, p1_hz=300, p1_db=-2, p2_hz=1800, p2_db=1.5, hs_hz=7000, hs_db=-6),
        fx("chorus2", "wide-slow", mix=0.5),
        fx("comp2", key=kick, thresh=-30, ratio=4, atk=0.002, rel=0.2)])
hc2 = clip(hum)
hc2.note("F2", 0.0, DONE + 0.5, 90)
hc2.note("C3", 0.0, DONE + 0.5, 60)
hum.automate("pan", *lane(lambda g: clamp(sweep_x(g) * 0.4, -0.4, 0.4), step=3))
# it swells under the collapse and cuts out with the Done boom
hum.automate("volume", (0, 0.35), (1.0, 0.55), (COLLAPSE, 0.55), (DONE - 0.1, 0.8), (DONE, 0.0, "hold"))

# ---- the lock: a pitched data blip, panned to the box ----------------------
# Two quick hits per lock (the two red frames), each a random note of F minor
# in the middle register: a square-ish FM zap with a pitch drop, crushed.
blip = song.track("LOCK", "concoction", None, volume=1.0, output=mix,
    params=dict(a_table="PWM", a_pos=0.35, b_on="ON", b_table="FM", b_pos=0.45,
                b_oct=-1, b_level=0.5, f_mode="LP12", f_cut=1800, f_res=0.45, f_env=0.5,
                e2_a=0.0005, e2_d=0.05, e2_s=0, p_amt=12, p_time=0.012,
                e1_a=0.0005, e1_d=0.08, e1_s=0, e1_r=0.03, level=1.0),
    fx=[fx("era", "sp1200", bits=7, rate=14000, filter=9000, mix=1),
        fx("sat2", "diode-drive", drive=14, mix=0.6, out=3),
        fx("eq2", hpf_on="ON", hpf_hz=220, p1_hz=500, p1_db=-2, p2_hz=2500, p2_db=2, hs_hz=9000, hs_db=-4)])
lc = clip(blip)
SCALE = ["F3", "G3", "Ab3", "Bb3", "C4", "Db4", "Eb4", "F4", "G4", "Ab4", "C5"]
lock_rng = random.Random(180)
for k, b in enumerate(LOCKS[:11]):
    lc.note(lock_rng.choice(SCALE), b, 0.1, 118)
    lc.note(lock_rng.choice(SCALE), b + 2 / FPS * B, 0.1, 100)
lc.note("F4", LOCKS[11], 0.12, 110)  # after Done: home
blip.automate("pan", *lane(lambda g: clamp(box_x(g) * 1.8, -0.85, 0.85)))
blip.automate("fx0:bits", (0, 8), (COLLAPSE, 4), (DONE, 9, "hold"))
blip.send(echo, -6)
blip.send(verb, -14)

# ---- sweeps ----------------------------------------------------------------
riser = song.track("RISER", "concoction", "noise-sweep", volume=0.5, output=mix,
    params=dict(e3_a=2.5, f_res=0.7, e1_a=0.3, e1_r=0.3),
    fx=[fx("sat2", "fold-buzz", drive=6, mix=0.25, out=-2),
        fx("eq2", hpf_on="ON", hpf_hz=350, p2_hz=4000, p2_db=-2, hs_hz=12000, hs_db=-3)])
rc = clip(riser)
rc.note("C5", 0.3, 0.7, 100)          # a quick inhale into the drop
rc.note("F4", 6.0, DONE - 6.0, 110)   # and the long one into Done
riser.automate("f_cut", (0.3, 400, "curve", -0.5), (1.0, 6000, "hold"),
               (6.0, 300, "curve", -0.6), (DONE, 7000))
riser.automate("volume", (0, 0.4), (6.0, 0.4), (COLLAPSE, 0.5), (DONE, 0.8, "hold"))
riser.automate("pan", *lane(lambda g: clamp(sweep_x(g) * 0.7, -0.7, 0.7), 0, 11 * SHOT, 2))
riser.send(verb, -10)

# The first frame: a filtered noise fall, white screen to dark.
fall = song.track("FALL", "concoction", "noise-sweep", volume=0.45, output=mix,
    params=dict(e1_a=0.001, e1_d=1.2, e1_s=0, e1_r=0.4, m1_src="OFF",
                f_mode="BP", f_cut=8000, f_res=0.5, e3_a=0.0005),
    fx=[fx("eq2", hpf_on="ON", hpf_hz=200, hs_hz=12000, hs_db=-3)])
fc = clip(fall)
fc.note("F4", 0.0, 1.2, 110)
fall.ramp("f_cut", 0.0, 1.2, 9000, 250, tension=0.5)
fall.automate("pan", *lane(lambda g: clamp(sweep_x(g) * 0.8, -0.8, 0.8), 0, 2 * SHOT))
fall.send(verb, -6)

# ---- "Done" ----------------------------------------------------------------
chime = song.track("DONE", "fm86", "slab/bells", volume=1.15, output=mix,
    fx=[fx("eq2", hpf_on="ON", hpf_hz=500, p2_hz=2800, p2_db=-2, hs_hz=12000, hs_db=1.5),
        fx("chorus2", "bell-widener")])
dc = clip(chime)
dc.chord(["F5", "C6", "Ab6"], DONE + 0.12, 1.5, 100, strum=0.03)
chime.send(verb, -6)
chime.send(echo, -16)

# Master: subsonics gone below 30 Hz (the built-in 24 dB/oct filter and a
# 48 dB/oct cut on the GEQ), then glue, tape and a limiter.
song.master(subsonic=True, fx=[
    fx("geq8", "flat", t1="LC48", f1=30, t2="BELL", f2=250, b2=-1.5, q2=0.8,
       t8="HSHLF", f8=12000, b8=1.5),
    fx("bus2", "mix-glue", thresh=-16, makeup=2, color=0.25),
    fx("sat2", "tape-glue", drive=7, mix=0.35, out=-1.5),
    fx("limiter2", gain=5.5, ceil=-2.2),
])

song.save(os.path.join(HERE, "ff_tracking.slab"))
if args.render:
    wav = args.wav or os.path.join(args.video, "out", "pass", "score.wav")
    os.makedirs(os.path.dirname(wav), exist_ok=True)
    song.render(wav=wav, stems=args.stems)
