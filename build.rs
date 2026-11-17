use std::path::Path;

fn main() {
    let style = Path::new("style");
    println!("cargo:rerun-if-changed={}", style.display());
    let mut sources: Vec<_> = std::fs::read_dir(style)
        .expect("read style/")
        .map(|entry| entry.expect("style/ entry").path())
        .collect();
    sources.sort();
    for source in &sources {
        println!("cargo:rerun-if-changed={}", source.display());
    }

    let css = grass::from_path(style.join("_index.scss"), &grass::Options::default())
        .unwrap_or_else(|error| panic!("compile style/_index.scss: {error}"));
    let out = Path::new(&std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("plinth.css");
    std::fs::write(&out, css).expect("write plinth.css");
}
