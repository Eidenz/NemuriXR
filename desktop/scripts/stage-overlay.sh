#!/usr/bin/env bash
# Build the overlay and stage it as a Tauri sidecar so `tauri build`/`tauri dev`
# bundle it alongside the desktop app. Usage: stage-overlay.sh [debug|release].
set -euo pipefail
cd "$(dirname "$0")/.."

profile="${1:-release}"
triple="$(rustc -vV | sed -n 's/^host: //p')"

flag=""
outdir="debug"
if [ "$profile" = "release" ]; then
  flag="--release"
  outdir="release"
fi

echo "Building overlay ($profile) for ${triple}…"
cargo build $flag -p nemurixr-overlay --manifest-path ../Cargo.toml

mkdir -p src-tauri/binaries
cp "../target/${outdir}/nemurixr-overlay" "src-tauri/binaries/nemurixr-overlay-${triple}"
echo "Staged src-tauri/binaries/nemurixr-overlay-${triple} ($profile)"
