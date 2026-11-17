//! A project with local styles, built through sockel-build as an adopter is.
//!
//! [`class`] holds only the local classes, named without the prefix, so a
//! misspelt one does not compile:
//!
//! ```
//! assert_eq!(sockel_build_fixture::class::PANEL, "fixture-panel");
//! assert!(sockel_build_fixture::STYLESHEET.contains(".sockel-stack {"));
//! ```
//!
//! ```compile_fail
//! let _ = sockel_build_fixture::class::PANLE;
//! ```
//!
//! It depends on sockel directly, so sockel-build reads sockel's features
//! (`DEP_SOCKEL_FEATURES`) and its stylesheet starts with sockel's own,
//! the same subset:
//!
//! ```
//! assert!(sockel_build_fixture::STYLESHEET.starts_with(sockel::STYLESHEET));
//! ```
//!
//! sockel's classes are `sockel::class`'s, not the project's:
//!
//! ```compile_fail
//! let _ = sockel_build_fixture::class::STACK;
//! ```

pub mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

pub const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

/// A panel's classes: sockel's stack with the fixture's panel, as its
/// `.sockel-stack.fixture-panel` rule targets them.
pub fn panel_classes() -> String {
    format!("{} {}", sockel::class::STACK, class::PANEL)
}
