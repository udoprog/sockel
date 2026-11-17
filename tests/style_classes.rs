//! Every class plinth styles is written by its own sources or the showcase's,
//! or is a hook an adopter's markup writes.

use plinth_build::{Config, Usage};

/// Utilities and hooks documented in the Sass for an app's own markup.
const ADOPTER_HOOKS: [&str; 12] = [
    "plinth-basis-lg",
    "plinth-basis-md",
    "plinth-basis-sm",
    "plinth-dense",
    "plinth-grow",
    "plinth-head-end",
    "plinth-inset",
    "plinth-library-page",
    "plinth-push-end",
    "plinth-reading",
    "plinth-self-start",
    "plinth-start",
];

#[test]
fn every_styled_class_is_written() {
    Usage::new()
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/showcase/src"))
        .foreign(ADOPTER_HOOKS)
        .assert_clean(&Config::new().style(env!("PLINTH_STYLE")));
}
