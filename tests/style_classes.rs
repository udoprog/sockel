//! Every class sockel styles is written by its own sources or the showcase's,
//! or is a hook an adopter's markup writes.

use sockel_build::{Config, Usage};

/// Utilities and hooks documented in the Sass for an app's own markup.
const ADOPTER_HOOKS: [&str; 12] = [
    "sockel-basis-lg",
    "sockel-basis-md",
    "sockel-basis-sm",
    "sockel-dense",
    "sockel-grow",
    "sockel-head-end",
    "sockel-inset",
    "sockel-library-page",
    "sockel-push-end",
    "sockel-reading",
    "sockel-self-start",
    "sockel-start",
];

#[test]
fn every_styled_class_is_written() {
    let showcase = concat!(env!("CARGO_MANIFEST_DIR"), "/showcase/src");

    // The showcase is not part of the published package, and without its
    // sources the classes only it writes would read as unused.
    if !std::path::Path::new(showcase).is_dir() {
        eprintln!("skipping: {showcase} is missing");
        return;
    }

    Usage::new()
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .source(showcase)
        .foreign(ADOPTER_HOOKS)
        .assert_clean(&Config::new().style(env!("SOCKEL_STYLE")));
}
