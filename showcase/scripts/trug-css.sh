#!/bin/sh
# Trunk post_build hook: builds the stylesheet with `trug css` (on PATH, or $TRUG)
# and checks it against the one build.rs wrote. Without trug the check is skipped.
set -e
cd "$(dirname "$0")/.."
trug="${TRUG:-trug}"
if ! command -v "$trug" >/dev/null 2>&1; then
    echo "trug-css: '$trug' not found, skipping the trug CSS check (cargo install trug)" >&2
    exit 0
fi
out=../target/trug-css/style.css
mkdir -p "$(dirname "$out")"
"$trug" css trug/app.scss -I trug -o "$out" --offline
if ! cmp -s "$out" ../target/showcase-css/style.css; then
    echo "trug-css: trug's CSS ($out) differs from build.rs's (../target/showcase-css/style.css)" >&2
    exit 1
fi
