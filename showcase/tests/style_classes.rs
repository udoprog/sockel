//! Every class the showcase styles is written by its Rust sources.

use plinth_build::{Config, Usage};

#[test]
fn every_styled_class_is_written() {
    Usage::new()
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .assert_clean(
            &Config::new()
                .style(env!("PLINTH_STYLE"))
                .local(concat!(env!("CARGO_MANIFEST_DIR"), "/style"))
                .prefix("showcase-"),
        );
}
