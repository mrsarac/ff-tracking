#!/usr/bin/env bash
# Whole pipeline: tracks -> flat pass -> sound -> lens pass -> out/tracking.mp4.
# Run from anywhere: tools/render.sh
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release -p hud -p flat -p lens
mkdir -p out/pass
target/release/tracks
target/release/flat render -o out/pass/flat.mp4
python3 -I tools/sfx.py
target/release/lens render -o out/tracking.mp4
ffprobe -v error -show_entries stream=codec_type,width,height,nb_frames,duration -of compact out/tracking.mp4
