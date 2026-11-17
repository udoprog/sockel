fn main() {
    plinth_build::Config::new()
        .local("style/")
        .prefix("fixture-")
        .compile();
}
