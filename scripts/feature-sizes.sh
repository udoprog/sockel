#!/usr/bin/env bash
# Measures each feature's CSS: sockel-build's `Config::features` with only
# that feature and the features it brings in (Cargo.toml's graph), the
# stylesheet's bytes / 1000, as `<feature> <KB>` rows, with `(none)` and
# `full` among them. The measuring is sockel-build's ignored test
# `features::feature_sizes`. Offline.
#
# Run from anywhere in the repository; set CARGO_BUILD_JOBS as the board's
# build policy asks.
set -euo pipefail
cd "$(dirname "$0")/.."

sizes=$(cargo test --quiet --offline -p sockel-build --lib features::feature_sizes \
    -- --ignored --nocapture --exact |
    awk 'NF == 2 && $2 ~ /^[0-9]+$/ { print }')
if [[ -z $sizes ]]; then
    echo "no sizes measured" >&2
    exit 1
fi

awk '{ printf "%s %.1f\n", $1, $2 / 1000 }' <<<"$sizes"
