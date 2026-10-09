#!/usr/bin/env bash
# The whole pipeline with the Slab score (score/score.py) in place of tools/sfx.py:
# tracks + camera -> flat pass -> Slab render -> trim to 6 s -> lens pass -> out/tracking.mp4.
# With a built Slab checkout (Apple silicon, zig >= 0.16) it renders the score:
#   SLAB=/path/to/slab tools/score.sh
# Without SLAB it uses the committed render, score/score.wav.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -p hud -p flat -p lens
mkdir -p out/pass
target/release/tracks
target/release/camera
target/release/flat render -o out/pass/flat.mp4
if [[ -n "${SLAB:-}" ]]; then
  PYTHONPATH="$SLAB/tools" python3 score/score.py --render
  # The render runs past the picture for the reverb tail: cut at 6 s, fade with the fade to black.
  ffmpeg -v error -y -i out/pass/score.wav -af "atrim=0:6,afade=t=out:st=5.75:d=0.25" \
    -ar 48000 -c:a pcm_s16le out/pass/sfx.wav
else
  cp score/score.wav out/pass/sfx.wav
fi
ffmpeg -nostats -i out/pass/sfx.wav -af ebur128=peak=true -f null - 2>&1 | grep -E "I:|Peak:" | tail -2
target/release/lens render -o out/tracking.mp4
ffprobe -v error -show_entries stream=codec_type,width,height,nb_frames,duration -of compact out/tracking.mp4
