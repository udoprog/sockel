#!/usr/bin/env bash
# Builds and tests a minimal app against sockel and sockel-build as packaged
# (`cargo package`), the way an app depending on them from a registry gets
# them. The app's files are the README's getting-started blocks, each
# marked `<!-- check: FILE -->`; blocks marked with the same file are joined
# in order. A block marked `<!-- check-variant: FILE -->` stands in for FILE
# in a build of its own. The app's `[patch.crates-io]` points at the
# unpacked packages, each in a directory of its own, so a path reaching
# outside a package fails.
# Usage: check-packaged.sh [README]; the blocks come from README.md unless
# another file is given.
# Works offline from cargo's cache, in target/package-check/. Set
# CARGO_BUILD_JOBS as the board's build policy asks.
set -euo pipefail
cd "$(dirname "$0")/.."
root=$PWD
readme=$(realpath "${1:-README.md}")
scratch=$root/target/package-check
rm -rf "$scratch/crates" "$scratch/app" "$scratch/variants"
mkdir -p "$scratch/crates/sockel" "$scratch/crates/sockel-build" "$scratch/app"

cargo package --offline --no-verify --allow-dirty --exclude-lockfile --quiet -p sockel-build -p sockel
for name in sockel sockel-build; do
    version=$(cargo pkgid --offline -p "$name" | sed 's/.*[@#]//')
    # Fresh mtimes, so a changed package rebuilds in the kept target.
    tar -xzmf "target/package/$name-$version.crate" -C "$scratch/crates/$name"
done

awk -v app="$scratch/app" -v variants="$scratch/variants" '
    /^<!-- check: [^ ]+ -->$/ { file = app "/" $3; next }
    /^<!-- check-variant: [^ ]+ -->$/ { file = variants "/" ++n "/" $3; next }
    file != "" && /^```/ {
        if (inside) { inside = 0; file = ""; next }
        inside = 1; next
    }
    inside {
        dir = file
        sub(/\/[^\/]*$/, "", dir)
        system("mkdir -p \"" dir "\"")
        print > file
    }
' "$readme"

cd "$scratch/app"
for file in Cargo.toml style.rs build.rs src/main.rs style/_index.scss tests/style_classes.rs; do
    [[ -s $file ]] || { echo "README has no check block for $file" >&2; exit 1; }
done
sed -i "1i [package]\nname = \"packaged-app\"\nedition = \"2024\"\npublish = false\n" Cargo.toml
cat >> Cargo.toml <<EOF

[patch.crates-io]
sockel = { path = "$(echo "$scratch"/crates/sockel/sockel-*)" }
sockel-build = { path = "$(echo "$scratch"/crates/sockel-build/sockel-build-*)" }

[workspace]
EOF
cp "$root/Cargo.lock" .

# What the README cannot show: the stylesheet is the packaged sockel's, with
# the app's feature subset, then the app's own rules.
cat > tests/packaged.rs <<'EOF'
const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

#[test]
fn builds_from_the_packages() {
    assert!(env!("SOCKEL_STYLE").contains("/target/package-check/crates/sockel/"));
    assert!(STYLESHEET.starts_with(sockel::STYLESHEET));
    assert!(STYLESHEET.contains(".sockel-stack {"));
    assert!(STYLESHEET.contains(".app-sidebar {"));
    assert!(!STYLESHEET.contains(".sockel-table"));
}
EOF

export CARGO_TARGET_DIR=$scratch/target
failed=()
cargo test --offline --quiet || failed+=("test")
cargo check --offline --quiet --target wasm32-unknown-unknown || failed+=("wasm32 check")
for variant in "$scratch"/variants/*/; do
    [[ -d $variant ]] || continue
    cp -r "$variant". .
    cargo check --offline --quiet || failed+=("variant $(basename "$variant")")
done
if ((${#failed[@]})); then
    echo "failed: ${failed[*]}" >&2
    exit 1
fi
echo "the packaged crates build an app"
