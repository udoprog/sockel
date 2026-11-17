use std::{env, fs, path::Path};

fn main() {
    sockel_build::Config::new()
        .local("style/")
        .prefix("showcase-")
        .compile();

    // Where the trunk post_build hook (scripts/trug-css.sh) finds this build's CSS.
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("style.css");
    let known = Path::new("../target/showcase-css/style.css");
    fs::create_dir_all(known.parent().unwrap()).unwrap();
    fs::copy(out, known).unwrap();
}
