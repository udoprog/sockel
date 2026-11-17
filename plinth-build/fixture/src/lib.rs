//! A project with local styles, built through plinth-build as an adopter is.
//!
//! [`class`] holds only the local classes, named without the prefix, so a
//! misspelt one does not compile:
//!
//! ```
//! assert_eq!(plinth_build_fixture::class::PANEL, "fixture-panel");
//! assert!(plinth_build_fixture::STYLESHEET.contains(".plinth-stack {"));
//! ```
//!
//! ```compile_fail
//! let _ = plinth_build_fixture::class::PANLE;
//! ```
//!
//! It depends on plinth directly, so plinth-build reads plinth's features
//! (`DEP_PLINTH_FEATURES`) and its stylesheet starts with plinth's own,
//! the same subset:
//!
//! ```
//! assert!(plinth_build_fixture::STYLESHEET.starts_with(plinth::STYLESHEET));
//! ```
//!
//! plinth's classes are `plinth::class`'s, not the project's:
//!
//! ```compile_fail
//! let _ = plinth_build_fixture::class::STACK;
//! ```

pub mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

pub const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

/// A panel's classes: plinth's stack with the fixture's panel, as its
/// `.plinth-stack.fixture-panel` rule targets them.
pub fn panel_classes() -> String {
    format!("{} {}", plinth::class::STACK, class::PANEL)
}
