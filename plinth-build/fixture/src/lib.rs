//! A project with local styles, built through plinth-build as an adopter is.
//!
//! [`class`] holds only the local classes, named without the prefix, so a
//! misspelt one does not compile:
//!
//! ```
//! assert_eq!(plinth_build_fixture::class::PANEL, "fixture-panel");
//! assert!(plinth_build_fixture::STYLESHEET.contains(".stack {"));
//! ```
//!
//! ```compile_fail
//! let _ = plinth_build_fixture::class::PANLE;
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
