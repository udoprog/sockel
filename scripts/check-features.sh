#!/usr/bin/env bash
# Checks that sockel builds with each of its Cargo features alone, and with
# none: `cargo check --no-default-features --features <f>` per feature in
# Cargo.toml's [features]. That each feature's CSS holds no other feature's
# rules is sockel-build's test `features::each_feature_s_css_is_its_own`, run
# here too. The fixture, a direct dependent on `layout` alone, is tested on
# its own, so its sockel features are its own: its doc test checks that
# sockel-build gave it sockel's subset (DEP_SOCKEL_FEATURES). Run from
# anywhere in the repository; set CARGO_BUILD_JOBS as the board's build
# policy asks.
set -euo pipefail
cd "$(dirname "$0")/.."

features=$(sed -n '/^\[features\]/,/^\[/{s/^\([a-z][a-z-]*\) = .*/\1/p}' Cargo.toml |
    grep -vx -e default -e full)
failed=()
for feature in "" $features; do
    echo "== ${feature:-(none)}"
    if ! cargo check --quiet -p sockel --no-default-features --features "$feature"; then
        failed+=("${feature:-(none)}")
    fi
done
cargo test --quiet -p sockel-build features:: || failed+=("sockel-build features::")
cargo test --quiet -p sockel-build-fixture || failed+=("sockel-build-fixture")
if ((${#failed[@]})); then
    echo "failed: ${failed[*]}" >&2
    exit 1
fi
echo "every feature builds alone"
