fn main() {
    let enabled: Vec<_> = plinth_build::FEATURES
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| {
            let var = format!("CARGO_FEATURE_{}", name.to_uppercase().replace('-', "_"));
            std::env::var_os(var).is_some()
        })
        .collect();
    // A dependent's build.rs sees this as DEP_PLINTH_FEATURES, which
    // plinth-build reads, so its stylesheet has the same subset.
    println!("cargo::metadata=features={}", enabled.join(","));
    plinth_build::Config::new().features(enabled).compile();
}
