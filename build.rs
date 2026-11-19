use std::path::Path;

fn main() {
    let enabled: Vec<_> = sockel_build::FEATURES
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| {
            let var = format!("CARGO_FEATURE_{}", name.to_uppercase().replace('-', "_"));
            std::env::var_os(var).is_some()
        })
        .collect();
    // The test-id attribute is read with `option_env!` (see src/testid.rs).
    println!("cargo::rerun-if-env-changed=SOCKEL_TEST_ID_ATTR");
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let style = Path::new(&manifest).join("style");
    // A dependent's build.rs sees these as DEP_SOCKEL_FEATURES and
    // DEP_SOCKEL_STYLE, which sockel-build reads, so its stylesheet has the
    // same subset from the same sources.
    println!("cargo::metadata=features={}", enabled.join(","));
    println!("cargo::metadata=style={}", style.display());
    sockel_build::Config::new()
        .style(style)
        .features(enabled)
        .compile();
}
