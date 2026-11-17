fn main() {
    sockel_build::Config::new()
        .local("style/")
        .prefix("showcase-")
        .compile();
}
