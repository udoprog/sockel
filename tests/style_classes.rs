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
    Usage::new()
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/showcase/src"))
        .foreign(ADOPTER_HOOKS)
        .assert_clean(&Config::new().style(env!("SOCKEL_STYLE")));
}
