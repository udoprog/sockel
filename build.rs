fn main() {
    let enabled = plinth_build::FEATURES.iter().filter(|(name, _)| {
        let var = format!("CARGO_FEATURE_{}", name.to_uppercase().replace('-', "_"));
        std::env::var_os(var).is_some()
    });
    plinth_build::Config::new()
        .features(enabled.map(|(name, _)| *name))
        .compile();
}
