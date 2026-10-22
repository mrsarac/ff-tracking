#!/usr/bin/env bash
# Whole pipeline: tracks -> flat pass -> sound -> lens pass -> out/tracking.mp4.
# Run from anywhere: tools/render.sh
set -euo pipefail
cd "$(dirname "$0")/.."
export FF_BACKEND="${FF_BACKEND:-auto}" # auto, gpu (fail if unavailable), or cpu
case "$FF_BACKEND" in
  auto|gpu|cpu) ;;
  *) printf 'FF_BACKEND must be auto, gpu or cpu\n' >&2; exit 1 ;;
esac
command -v ffmpeg >/dev/null
command -v ffprobe >/dev/null
python=("${PYTHON:-python3}" -I)
if ! "${python[@]}" -c 'import numpy, scipy' 2>/dev/null; then
  command -v uv >/dev/null || { printf 'Install numpy/scipy for Python 3, or install uv\n' >&2; exit 1; }
  python=(uv run --no-project --with numpy --with scipy python -I)
fi
bin="${CARGO_TARGET_DIR:-target}/release"
cargo build --release -p hud -p flat -p lens
mkdir -p out/pass
"$bin"/tracks
"$bin"/flat render -o out/pass/flat.mp4
"${python[@]}" tools/sfx.py
"$bin"/lens render -o out/tracking.mp4
ffprobe -v error -show_entries stream=codec_type,width,height,nb_frames,duration -of compact out/tracking.mp4
