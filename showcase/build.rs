fn main() {
    plinth_build::Config::new()
        .local("style/")
        .prefix("showcase-")
        .compile();
}
