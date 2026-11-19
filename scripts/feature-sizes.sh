#!/usr/bin/env bash
# Measures the "CSS alone" column of the feature table in src/lib.rs's crate
# docs: sockel-build's `Config::features` with only that feature and the
# features it brings in (Cargo.toml's graph), the stylesheet's bytes / 1000.
# It also sets the none and `full` figures under the table. The measuring is
# sockel-build's ignored test `features::feature_sizes`. Offline.
#
#   scripts/feature-sizes.sh          rewrite the table and the line under it
#   scripts/feature-sizes.sh --print  print `<feature> <KB>` rows instead
#
# Run from anywhere in the repository; set CARGO_BUILD_JOBS as the board's
# build policy asks. A second run changes nothing.
set -euo pipefail
cd "$(dirname "$0")/.."

sizes=$(cargo test --quiet --offline -p sockel-build --lib features::feature_sizes \
    -- --ignored --nocapture --exact |
    awk 'NF == 2 && $2 ~ /^[0-9]+$/ { print }')
if [[ -z $sizes ]]; then
    echo "no sizes measured" >&2
    exit 1
fi

if [[ ${1:-} == --print ]]; then
    awk '{ printf "%s %.1f\n", $1, $2 / 1000 }' <<<"$sizes"
    exit 0
fi

tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
awk -v sizes="$sizes" '
BEGIN {
    n = split(sizes, lines, "\n")
    for (i = 1; i <= n; i++) {
        split(lines[i], f, " ")
        kb[f[1]] = f[2] / 1000
    }
}
/^\/\/! \| `[a-z-]+` \|.* [0-9.]+ KB \|$/ {
    name = $3
    gsub(/`/, "", name)
    if (name in kb) {
        sub(/[0-9.]+ KB \|$/, sprintf("%.1f KB |", kb[name]))
    } else {
        print "unknown feature " name > "/dev/stderr"
        failed = 1
    }
}
/with none it is [0-9.]+ KB, and with `full` [0-9.]+ KB/ {
    sub(/with none it is [0-9.]+ KB/, sprintf("with none it is %.1f KB", kb["(none)"]))
    sub(/with `full` [0-9.]+ KB/, sprintf("with `full` %.0f KB", kb["full"]))
}
{ print }
END { exit failed }
' src/lib.rs >"$tmp"
cat "$tmp" >src/lib.rs
