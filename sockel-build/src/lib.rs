//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/sockel-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/sockel)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/sockel-build.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/sockel-build)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sockel--build-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/sockel-build)
//! Builds an app's one stylesheet and its class constants from a `build.rs`:
//! sockel's, and a project's own Sass beside it ([`Config::local`]).
//!
//! sockel's Sass sources are read from the sockel package a build depends on
//! ([`STYLE_VAR`]); local Sass loads them as `@use "sockel" as p;`.
//! [`Config::compile`] writes `style.css` and `classes.rs` (a `pub const` per
//! class) to `OUT_DIR`.
//!
//! <br>
//!
//! ## `build.rs`
//!
//! Add sockel-build as a build dependency, then in the `main` of the package's
//! `build.rs`:
//!
//! ```no_run
//! sockel_build::Config::new()
//!     .local("style/")
//!     .prefix("app-")
//!     .compile();
//! ```
//!
//! The app's Sass lives in `style/`, with an `_index.scss` (or `index.scss`)
//! that `@use`s its partials and loads sockel's tokens and mixins as
//! `"sockel"`:
//!
//! ```scss
//! @use "sockel" as p;
//!
//! .app-sidebar {
//!     padding: var(--space-4);
//!
//!     @include p.below("phone") {
//!         padding: var(--space-2);
//!     }
//! }
//! ```
//!
//! The build reruns when `build.rs`, anything under `style/` or sockel's Sass
//! changes. It emits no second copy of sockel's CSS: `style.css` holds
//! sockel's and the app's together, so the page links no other stylesheet.
//!
//! Every class the app's Sass defines that is not sockel's must start with the
//! prefix (`app-`); one that does not is a build error. Classes a
//! dependency's markup writes are allowed with [`Config::foreign`] (a trailing
//! `*` is a prefix), and get no constant. Tokens are configured on the build
//! with [`Config::with`], not `@use "sockel" with (...)`.
//!
//! <br>
//!
//! ## The class constants and the stylesheet
//!
//! `classes.rs` holds the app's classes, named without the prefix. For the
//! Sass above it is:
//!
//! ```
//! /// `.app-sidebar`
//! pub const SIDEBAR: &str = "app-sidebar";
//! ```
//!
//! Include it in a module, and the stylesheet as a string:
//!
//! ```ignore
//! mod class {
//!     include!(concat!(env!("OUT_DIR"), "/classes.rs"));
//! }
//!
//! const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));
//! ```
//!
//! Pass the constants to a sockel component's `class` prop, so a class the
//! Sass does not define does not compile, and mount the stylesheet once with
//! `<sockel::Styles css={STYLESHEET} />`.
//!
//! <br>
//!
//! ## How it finds sockel
//!
//! sockel's own `build.rs` passes two values to the `build.rs` of every
//! package depending on it directly (Cargo's `links` metadata):
//! [`FEATURES_VAR`], its enabled features, and [`STYLE_VAR`], the `style/`
//! directory of its package. [`Config`] builds those features from those
//! sources, so the stylesheet matches the sockel the app links. A package that
//! reaches sockel only through another crate names them with
//! [`Config::features`] and [`Config::style`].
//!
//! <br>
//!
//! ## A guard against dead classes
//!
//! A class the Sass defines and no Rust writes is dead weight. [`Usage`] finds
//! them in a test:
//!
//! ```no_run
//! use sockel_build::{Config, Usage};
//!
//! #[test]
//! fn every_styled_class_is_written() {
//!     Usage::new()
//!         .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
//!         .assert_clean(
//!             &Config::new()
//!                 .style(env!("SOCKEL_STYLE"))
//!                 .local("style/")
//!                 .prefix("app-"),
//!         );
//! }
//! ```
//!
//! `SOCKEL_STYLE` ([`TEST_STYLE_VAR`]) is sockel's style directory, which
//! `compile()` hands the package's crates. A class counts as written when a
//! source names its constant (`class::SIDEBAR`) or the class in a string
//! literal; comments and `#[cfg(test)]` items do not count. The test fails on
//! a dead class and on an allowed pattern that matches no class.

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use sasso::Importer;

#[cfg(test)]
mod features;

/// sockel's sources in this repository.
#[cfg(test)]
const STYLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../style");

#[cfg(test)]
fn sockel() -> Config {
    Config::new().style(STYLE)
}

#[cfg(test)]
fn test_sources() -> Sources {
    Sources {
        style: STYLE.into(),
        index: None,
    }
}

mod usage;
use usage::matches;
pub use usage::{Report, Usage};

/// sockel's Cargo features and the partials (`style/_<name>.scss`) each
/// adds to the stylesheet. Partials no feature names (tokens, hooks, the
/// utilities and what components share) are always in.
pub const FEATURES: &[(&str, &[&str])] = &[
    ("action-group", &["actions"]),
    ("badge", &["badges"]),
    ("button", &["buttons"]),
    ("card-grid", &["card-grid"]),
    ("color-select", &["color-select"]),
    ("composer", &["composer"]),
    ("confirm", &["confirm"]),
    ("disclosure", &["disclosure"]),
    ("drop-zone", &["drop"]),
    ("field", &["fields"]),
    ("head", &["heads"]),
    ("icons", &["icons"]),
    ("input", &["inputs"]),
    ("lanes", &["lanes"]),
    ("layout", &["layout"]),
    ("library-toolbar", &[]),
    ("lightbox", &["lightbox"]),
    ("loadable", &[]),
    ("log-view", &["log-view"]),
    ("media", &["media"]),
    ("menu", &["menu"]),
    ("notice", &["notices"]),
    ("overlay", &["popover", "dialog"]),
    ("pager", &["pager"]),
    ("panel", &["panel"]),
    ("picker", &["picker", "filter-picker"]),
    ("property-list", &["property-list"]),
    ("reorder", &["reorder"]),
    ("rows", &["rows"]),
    ("search-select", &["search-select"]),
    ("seam", &["seam"]),
    ("select", &["listbox", "select"]),
    ("shell", &["shell"]),
    ("shortcuts", &["shortcuts"]),
    ("skeleton", &["skeleton"]),
    ("split-view", &["split-view"]),
    ("stepper", &["stepper"]),
    ("table", &["table"]),
    ("tabs", &["tabs"]),
    ("time", &["time"]),
    ("toast", &["toast"]),
    ("toggles", &["toggles"]),
    ("tooltip", &["tooltip"]),
];

/// The `$omit` parts (style/_tokens.scss) that are a feature's, each with
/// the features that read it: a subset of features leaves a part out when
/// none of them is on.
pub const FEATURE_PARTS: &[(&str, &[&str])] = &[
    ("skeleton.loadable", &["loadable"]),
    ("tokens.actions", &["rows"]),
    ("tokens.card-grid", &["card-grid"]),
    ("tokens.color-select", &["color-select"]),
    ("tokens.drop", &["drop-zone"]),
    ("tokens.fields", &["field"]),
    ("tokens.filter-picker", &["picker"]),
    ("tokens.media", &["media"]),
    ("tokens.panel", &["panel"]),
    ("tokens.picker", &["picker", "search-select"]),
    ("tokens.seam", &["seam"]),
    ("tokens.table", &["table"]),
    ("tokens.toast", &["toast"]),
    ("tokens.toggles", &["toggles"]),
    ("tokens.tooltip", &["tooltip"]),
];

/// The features sockel was built with, comma-separated, as a `build.rs` of a
/// package depending on sockel directly sees them (sockel's `links` metadata).
pub const FEATURES_VAR: &str = "DEP_SOCKEL_FEATURES";

/// The directory of sockel's Sass sources (`style/` in its package), as a
/// `build.rs` of a package depending on sockel directly sees it.
pub const STYLE_VAR: &str = "DEP_SOCKEL_STYLE";

/// The variable [`Config::compile`] sets to sockel's style directory for
/// the package's own crates, so its tests can pass it to [`Config::style`].
pub const TEST_STYLE_VAR: &str = "SOCKEL_STYLE";

/// Styles each hook class of `_hooks.scss`, so its class is found like any other.
const HOOKS_PROBE: &str =
    "@use \"sockel/hooks\";\n@each $class in hooks.$classes { .#{$class} { --hook: 0; } }\n";

/// The directory sockel's sources are served under.
const ROOT: &str = "sockel";

/// What sasso starts a stylesheet with when it holds non-ASCII text.
const CHARSET: &str = "@charset \"UTF-8\";\n";

/// sockel's sources at `sockel` and `sockel/...`, the disk at absolute paths
/// and relative to a file on disk: a project's files are always loaded by
/// absolute path, so nothing else on disk can stand in for sockel's sources.
#[derive(Debug)]
struct Sources {
    style: PathBuf,
    /// Stands in for `_index.scss`: the index of a subset of features.
    index: Option<String>,
}

impl Sources {
    fn disk(
        url: &str,
        ctx: &sasso::CanonicalizeContext<'_>,
    ) -> Result<Option<sasso::CanonicalUrl>, sasso::ImporterError> {
        sasso::FsImporter::new(Vec::new()).canonicalize(url, ctx)
    }

    /// `url`'s file in sockel's style directory.
    fn sockel(&self, url: &str) -> Result<Option<sasso::CanonicalUrl>, sasso::ImporterError> {
        let rest = match url.strip_prefix(ROOT) {
            Some("") => "_index.scss",
            Some(rest) => match rest.strip_prefix('/') {
                Some(rest) => rest,
                None => return Ok(None),
            },
            None => return Ok(None),
        };
        // Resolves `rest` relative to a file in the style directory.
        let anchor = sasso::CanonicalUrl::new(self.style.join("_").to_string_lossy());
        let ctx = sasso::CanonicalizeContext {
            from_import: false,
            containing_url: Some(&anchor),
        };
        Self::disk(rest, &ctx)
    }

    fn is_index(&self, canonical: &sasso::CanonicalUrl) -> bool {
        let index = self.style.join("_index.scss");
        let ctx = sasso::CanonicalizeContext {
            from_import: false,
            containing_url: None,
        };
        matches!(Self::disk(&index.to_string_lossy(), &ctx), Ok(Some(found)) if found == *canonical)
    }
}

impl Importer for Sources {
    fn canonicalize(
        &self,
        url: &str,
        ctx: &sasso::CanonicalizeContext<'_>,
    ) -> Result<Option<sasso::CanonicalUrl>, sasso::ImporterError> {
        if (Path::new(url).is_absolute() || ctx.containing_url.is_some())
            && let Some(found) = Self::disk(url, ctx)?
        {
            return Ok(Some(found));
        }
        self.sockel(url)
    }

    fn load(
        &self,
        canonical: &sasso::CanonicalUrl,
    ) -> Result<Option<sasso::ImporterResult>, sasso::ImporterError> {
        match &self.index {
            Some(index) if self.is_index(canonical) => Ok(Some(sasso::ImporterResult {
                contents: index.clone(),
                syntax: sasso::Syntax::Scss,
                source_map_url: None,
            })),
            _ => sasso::FsImporter::new(Vec::new()).load(canonical),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Sass(sasso::Error),
    Io(io::Error),
    /// A class whose name makes no Rust identifier.
    Name(String),
    /// Two classes that make the same constant.
    Collision(String, String),
    /// Local styles without [`Config::prefix`].
    NoPrefix,
    /// A local class without the project's prefix.
    Unprefixed {
        class: String,
        prefix: String,
    },
    /// A local directory with no `_index.scss` or `index.scss`.
    NoIndex(PathBuf),
    /// sockel's CSS did not come out first and unchanged.
    NotVerbatim,
    /// A name [`FEATURES`] does not list.
    Feature(String),
    /// No [`Config::style`] and no [`STYLE_VAR`].
    NoStyle,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Sass(error) => write!(f, "compile Sass: {error}"),
            Error::Io(error) => write!(f, "{error}"),
            Error::Name(class) => write!(f, "class .{class} makes no constant name"),
            Error::Collision(a, b) => write!(f, "classes .{a} and .{b} make the same constant"),
            Error::NoPrefix => write!(
                f,
                "local styles need a prefix: Config::prefix(\"<project>-\")"
            ),
            Error::Unprefixed { class, prefix } => write!(
                f,
                "local class .{class} is neither sockel's nor prefixed `{prefix}`"
            ),
            Error::NoIndex(dir) => {
                write!(f, "{} has no _index.scss or index.scss", dir.display())
            }
            Error::NotVerbatim => write!(f, "sockel's CSS did not come first and unchanged"),
            Error::Feature(name) => write!(f, "sockel has no feature `{name}`"),
            Error::NoStyle => write!(
                f,
                "sockel's style directory is unknown: {STYLE_VAR} is set only in the \
                 build.rs of a package depending on sockel directly; elsewhere use \
                 Config::style"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<sasso::Error> for Error {
    fn from(error: sasso::Error) -> Self {
        Error::Sass(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

/// A build's settings.
#[derive(Clone, Debug, Default)]
pub struct Config {
    out_dir: Option<PathBuf>,
    style: Option<PathBuf>,
    local: Option<PathBuf>,
    prefix: Option<String>,
    with: Vec<(String, String)>,
    modules: Vec<(PathBuf, String)>,
    features: Option<Vec<String>>,
    foreign: Vec<String>,
    layer: Option<String>,
    layer_order: Vec<String>,
}

/// A build's results.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The app's one stylesheet: sockel's CSS (in [`Config::layer`]'s layer,
    /// after [`Config::layer_order`]'s statement), then the local styles' CSS.
    pub css: String,
    /// sockel's classes: every class its selectors name and every hook class, sorted.
    pub sockel_classes: Vec<String>,
    /// The local styles' own classes (each with the prefix), sorted.
    pub local_classes: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// Where [`Config::write`] puts its files, instead of `OUT_DIR`.
    pub fn out_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.out_dir = Some(dir.into());
        self
    }

    /// sockel's Sass sources, instead of [`STYLE_VAR`]'s directory: in a test
    /// `env!("SOCKEL_STYLE")` ([`TEST_STYLE_VAR`]), set by the package's
    /// `build.rs`.
    pub fn style(mut self, dir: impl Into<PathBuf>) -> Self {
        self.style = Some(dir.into());
        self
    }

    /// The project's own Sass: `dir/_index.scss` (or `index.scss`), which can
    /// `@use "sockel" as p;` for sockel's tokens and mixins. A relative
    /// `dir` is taken from the current directory, a `build.rs`'s package root.
    pub fn local(mut self, dir: impl Into<PathBuf>) -> Self {
        self.local = Some(dir.into());
        self
    }

    /// The prefix every local class must start with (`"army-"`).
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Local classes a dependency's markup writes (`"auth-*"`, a trailing `*`
    /// is a prefix): styled without the prefix, and given no constant.
    pub fn foreign<S: Into<String>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.foreign.extend(classes.into_iter().map(Into::into));
        self
    }

    /// Sets a sockel token (`"text-md"` or `"$text-md"`) to a Sass value
    /// (`"1.0625rem"`), as `@use "sockel" with (...)` would. Only values
    /// change; the classes stay the same.
    pub fn with(mut self, variable: impl AsRef<str>, value: impl Into<String>) -> Self {
        let name = variable.as_ref().trim_start_matches('$').to_owned();
        self.with.push((name, value.into()));
        self
    }

    /// Loads a local Sass module before sockel (`@use "path" as namespace`),
    /// so [`Config::with`]'s values can name its variables (`v.$font-sm`)
    /// instead of restating them. The module must emit no CSS. A relative
    /// `path` is taken from the current directory, as [`Config::local`]'s.
    pub fn with_module(mut self, path: impl Into<PathBuf>, namespace: impl Into<String>) -> Self {
        self.modules.push((path.into(), namespace.into()));
        self
    }

    /// Builds sockel's CSS from only these [`FEATURES`] (each with the
    /// features it depends on already in), instead of all of them, and
    /// leaves out the [`FEATURE_PARTS`] none of them reads. Without it, the
    /// features are [`FEATURES_VAR`]'s when it is set (in the `build.rs` of
    /// a package depending on sockel directly), so the app's stylesheet has
    /// sockel's subset, and all of them otherwise.
    pub fn features<I>(mut self, features: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.features = Some(features.into_iter().map(Into::into).collect());
        self
    }

    /// The features to build, `None` for all of them.
    fn enabled(&self) -> Option<Vec<String>> {
        self.features.clone().or_else(|| {
            let list = std::env::var(FEATURES_VAR).ok()?;
            let names = list.split(',').filter(|name| !name.is_empty());
            Some(names.map(str::to_owned).collect())
        })
    }

    fn style_dir(&self) -> Result<PathBuf, Error> {
        let dir = match &self.style {
            Some(dir) => dir.clone(),
            None => std::env::var_os(STYLE_VAR)
                .map(PathBuf::from)
                .ok_or(Error::NoStyle)?,
        };
        Ok(std::path::absolute(dir)?)
    }

    /// `_index.scss` without the `@use` of each partial whose feature is off.
    fn index(style: &Path, enabled: Option<&[String]>) -> Result<Option<String>, Error> {
        let Some(enabled) = enabled else {
            return Ok(None);
        };
        if let Some(unknown) = enabled
            .iter()
            .find(|name| !FEATURES.iter().any(|(feature, _)| feature == name))
        {
            return Err(Error::Feature(unknown.clone()));
        }
        let off = |partial: &str| {
            FEATURES.iter().any(|(feature, partials)| {
                partials.contains(&partial) && !enabled.iter().any(|name| name == feature)
            })
        };
        let index = std::fs::read_to_string(style.join("_index.scss"))?;
        let lines = index.lines().filter(|line| {
            let partial = line
                .strip_prefix("@use \"")
                .and_then(|rest| rest.strip_suffix("\";"));
            !partial.is_some_and(off)
        });
        Ok(Some(lines.map(|line| format!("{line}\n")).collect()))
    }

    fn sockel_use(&self, enabled: Option<&[String]>) -> Result<String, Error> {
        let on =
            |feature: &&str| enabled.is_none_or(|enabled| enabled.iter().any(|e| e == *feature));
        let folded: Vec<_> = FEATURE_PARTS
            .iter()
            .filter(|(_, features)| !features.iter().any(on))
            .map(|(part, _)| format!("\"{part}\""))
            .collect();
        let mut pairs: Vec<_> = self
            .with
            .iter()
            .filter(|(name, _)| folded.is_empty() || name != "omit")
            .map(|(name, value)| format!("${name}: {value}"))
            .collect();
        let mut head = String::new();
        for (path, namespace) in &self.modules {
            let path = sass_string(&std::path::absolute(path)?);
            head.push_str(&format!("@use \"{path}\" as {namespace};\n"));
        }
        if !folded.is_empty() {
            let folded = format!("({},)", folded.join(", "));
            let omit = match self.with.iter().rfind(|(name, _)| name == "omit") {
                Some((_, value)) => {
                    head.push_str("@use \"sass:list\";\n");
                    format!("list.join({value}, {folded}, comma)")
                }
                None => folded,
            };
            pairs.push(format!("$omit: {omit}"));
        }
        if pairs.is_empty() {
            return Ok(format!("{head}@use \"sockel\";\n"));
        }
        Ok(format!(
            "{head}@use \"sockel\" with ({});\n",
            pairs.join(", ")
        ))
    }

    /// Every class of sockel's enabled features, nothing omitted.
    fn all_classes(options: &sasso::Options<'_>) -> Result<Vec<String>, Error> {
        let mut classes = class_names(&sasso::compile("@use \"sockel\";\n", options)?);
        classes.sort();
        Ok(classes)
    }

    /// Puts sockel's CSS in the cascade layer `name` (`@layer name { ... }`),
    /// for an app whose own styles are in layers: unlayered, sockel's rules
    /// would win over every one of them.
    pub fn layer(mut self, name: impl Into<String>) -> Self {
        self.layer = Some(name.into());
        self
    }

    /// Declares the app's cascade layers in this order (`@layer a, b;`)
    /// before sockel's CSS, so a layer the app puts below
    /// [`Config::layer`]'s comes first.
    pub fn layer_order<I>(mut self, names: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.layer_order = names.into_iter().map(Into::into).collect();
        self
    }

    /// sockel's CSS as it starts the stylesheet: in its layer, after the
    /// layer order.
    fn layered(&self, sockel_css: &str) -> String {
        // `@charset` only counts as the stylesheet's first rule.
        let (mut css, sockel_css) = match sockel_css.strip_prefix(CHARSET) {
            Some(rest) => (CHARSET.to_owned(), rest),
            None => (String::new(), sockel_css),
        };
        if !self.layer_order.is_empty() {
            css.push_str(&format!("@layer {};\n", self.layer_order.join(", ")));
        }
        match &self.layer {
            Some(name) => {
                css.push_str(&format!("@layer {name} {{\n{sockel_css}"));
                if !sockel_css.ends_with('\n') {
                    css.push('\n');
                }
                css.push_str("}\n");
            }
            None => css.push_str(sockel_css),
        }
        css
    }

    pub fn build(&self) -> Result<Output, Error> {
        let enabled = self.enabled();
        let style = self.style_dir()?;
        let sources = Sources {
            index: Self::index(&style, enabled.as_deref())?,
            style,
        };
        let options = sasso::Options::new().with_importer(&sources);
        let sockel_use = self.sockel_use(enabled.as_deref())?;
        let sockel_css = sasso::compile(&sockel_use, &options)?;
        let hooks = sasso::compile(HOOKS_PROBE, &options)?;
        let mut sockel_classes = class_names(&sockel_css);
        sockel_classes.extend(class_names(&hooks));
        sockel_classes.sort();
        sockel_classes.dedup();

        let Some(dir) = &self.local else {
            return Ok(Output {
                css: self.layered(&sockel_css),
                sockel_classes,
                local_classes: Vec::new(),
            });
        };
        let prefix = self.prefix.as_deref().ok_or(Error::NoPrefix)?;
        let index = local_index(dir)?;
        // One compilation, sockel first, so a local `@use "sockel"` reuses
        // the module loaded here and emits nothing again.
        let entry = format!("{sockel_use}@use \"{}\";\n", sass_string(&index));
        let whole = sasso::compile(&entry, &options)?;
        let local_css = whole.strip_prefix(&sockel_css).ok_or(Error::NotVerbatim)?;
        let css = format!("{}{local_css}", self.layered(&sockel_css));
        let mut unconfigured = None;
        let mut local_classes = Vec::new();
        for class in class_names(local_css) {
            if sockel_classes.binary_search(&class).is_ok()
                || self.foreign.iter().any(|pattern| matches(pattern, &class))
            {
                continue;
            }
            if matches!(class.strip_prefix(prefix), Some(rest) if !rest.is_empty()) {
                local_classes.push(class);
                continue;
            }
            // A sockel class `$omit` leaves out, which the app styles itself.
            if class.starts_with("sockel-") {
                let all = match &mut unconfigured {
                    Some(all) => all,
                    None => unconfigured.insert(Self::all_classes(&options)?),
                };
                if all.binary_search(&class).is_ok() {
                    continue;
                }
            }
            return Err(Error::Unprefixed {
                class,
                prefix: prefix.to_owned(),
            });
        }
        Ok(Output {
            css,
            sockel_classes,
            local_classes,
        })
    }

    /// Builds and writes `style.css` (the whole stylesheet) and `classes.rs`:
    /// sockel's constants, or with local styles only the local ones, each
    /// named without its prefix (`sockel-stack` is `STACK`, `army-sidebar`
    /// is `SIDEBAR`).
    pub fn write(&self) -> Result<Output, Error> {
        let output = self.build()?;
        let dir = match &self.out_dir {
            Some(dir) => dir.clone(),
            None => std::env::var_os("OUT_DIR")
                .map(PathBuf::from)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OUT_DIR is not set"))?,
        };
        let classes = match (&self.local, &self.prefix) {
            (Some(_), Some(prefix)) => prefixed_classes_rs(&output.local_classes, prefix)?,
            _ => prefixed_classes_rs(&output.sockel_classes, "sockel-")?,
        };
        std::fs::write(dir.join("style.css"), &output.css)?;
        std::fs::write(dir.join("classes.rs"), classes)?;
        Ok(output)
    }

    /// [`Config::write`] for a `build.rs`: reruns when `build.rs`, a local
    /// file or one of sockel's changes, sets [`TEST_STYLE_VAR`] for the
    /// package's crates, and panics with the error.
    pub fn compile(&self) -> Output {
        let fail = |error: &dyn fmt::Display| -> ! { panic!("sockel-build: {error}") };
        println!("cargo:rerun-if-changed=build.rs");
        let style = self.style_dir().unwrap_or_else(|error| fail(&error));
        let dirs = [Some(&style), self.local.as_ref()];
        for dir in dirs.into_iter().flatten() {
            for path in local_files(dir).unwrap_or_else(|error| fail(&error)) {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
        println!("cargo:rustc-env={TEST_STYLE_VAR}={}", style.display());
        self.write().unwrap_or_else(|error| fail(&error))
    }
}

fn local_index(dir: &Path) -> Result<PathBuf, Error> {
    let dir = std::path::absolute(dir)?;
    ["_index.scss", "index.scss"]
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.is_file())
        .ok_or(Error::NoIndex(dir))
}

/// `dir` and everything under it, directories included (so an added file
/// reruns the build too), sorted.
pub fn local_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut found = vec![dir.to_path_buf()];
    let mut next = 0;
    while let Some(path) = found.get(next).cloned() {
        next += 1;
        if path.is_dir() {
            for entry in std::fs::read_dir(&path)? {
                found.push(entry?.path());
            }
        }
    }
    found.sort();
    Ok(found)
}

fn sass_string(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

/// A class's constant: upper case, `-` as `_` (`page-head-title` is
/// `PAGE_HEAD_TITLE`).
pub fn const_name(class: &str) -> Result<String, Error> {
    let name: String = class
        .chars()
        .map(|c| match c {
            '-' => Some('_'),
            c if c.is_ascii_alphanumeric() || c == '_' => Some(c.to_ascii_uppercase()),
            _ => None,
        })
        .collect::<Option<_>>()
        .ok_or_else(|| Error::Name(class.to_owned()))?;
    match name.chars().next() {
        Some(first) if (first.is_ascii_alphabetic() || first == '_') && name != "_" => Ok(name),
        _ => Err(Error::Name(class.to_owned())),
    }
}

/// The Rust source of one `pub const` per class, its value the class.
pub fn classes_rs(classes: &[String]) -> Result<String, Error> {
    let mut consts = BTreeMap::new();
    for class in classes {
        let name = const_name(class)?;
        if let Some(other) = consts.insert(name, class) {
            return Err(Error::Collision(other.clone(), class.clone()));
        }
    }
    Ok(consts_rs(consts))
}

/// [`classes_rs`] for local classes: each constant named without `prefix`.
pub fn prefixed_classes_rs(classes: &[String], prefix: &str) -> Result<String, Error> {
    let mut consts = BTreeMap::new();
    for class in classes {
        let name = class
            .strip_prefix(prefix)
            .ok_or_else(|| Error::Unprefixed {
                class: class.clone(),
                prefix: prefix.to_owned(),
            })
            .and_then(const_name)?;
        if let Some(other) = consts.insert(name, class) {
            return Err(Error::Collision(other.clone(), class.clone()));
        }
    }
    Ok(consts_rs(consts))
}

fn consts_rs(consts: BTreeMap<String, &String>) -> String {
    consts
        .into_iter()
        .map(|(name, class)| format!("/// `.{class}`\npub const {name}: &str = \"{class}\";\n"))
        .collect()
}

/// The classes a stylesheet's selectors name, sorted and once each; at-rule
/// preludes, declarations, strings and comments are skipped.
pub fn class_names(css: &str) -> Vec<String> {
    let mut classes = std::collections::BTreeSet::new();
    let mut prelude = String::new();
    let mut chars = css.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut last = '\0';
                for c in chars.by_ref() {
                    if last == '*' && c == '/' {
                        break;
                    }
                    last = c;
                }
            }
            '"' | '\'' => {
                let mut escaped = false;
                for next in chars.by_ref() {
                    match next {
                        _ if escaped => escaped = false,
                        '\\' => escaped = true,
                        _ if next == c => break,
                        _ => {}
                    }
                }
                prelude.push(' ');
            }
            '{' => {
                if !prelude.trim_start().starts_with('@') {
                    selector_classes(&prelude, &mut classes);
                }
                prelude.clear();
            }
            ';' | '}' => prelude.clear(),
            c => prelude.push(c),
        }
    }
    classes.into_iter().collect()
}

fn selector_classes(selector: &str, classes: &mut std::collections::BTreeSet<String>) {
    let mut rest = selector;
    while let Some(dot) = rest.find('.') {
        rest = &rest[dot + 1..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(rest.len());
        let class = &rest[..end];
        if class
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '-')
        {
            classes.insert(class.to_owned());
        }
        rest = &rest[end..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_sockel_s_sources() {
        let output = sockel().build().unwrap();
        assert!(output.css.contains(".sockel-stack {"));
        for class in ["sockel-page-head-title", "sockel-crumb", "sockel-trailing"] {
            assert!(
                output.sockel_classes.iter().any(|found| found == class),
                "{class}"
            );
        }
        assert!(!output.css.contains("crumb {"));
        let style = Path::new(env!("CARGO_MANIFEST_DIR")).join("../style/_index.scss");
        let source = std::fs::read_to_string(&style).unwrap();
        let fs = sasso::FsImporter::new(Vec::new());
        let url = style.to_string_lossy();
        let options = sasso::Options::new().with_importer(&fs).with_url(&url);
        let on_disk = sasso::compile(&source, &options).unwrap();
        assert_eq!(output.css, on_disk);
    }

    #[test]
    fn configures_token_values_not_classes() {
        let plain = sockel().build().unwrap();
        let config = sockel()
            .with("$text-md", "1.0625rem")
            .with("radius-lg", "12px");
        for output in [
            config.build().unwrap(),
            config
                .clone()
                .local(fixture("fixture/style"))
                .prefix("fixture-")
                .build()
                .unwrap(),
        ] {
            assert!(
                output.css.contains("--text-md: 1.0625rem"),
                "{}",
                output.css
            );
            assert!(output.css.contains("--radius-lg: 12px"));
            assert!(!output.css.contains("--text-md: 0.875rem"));
            assert_eq!(output.sockel_classes, plain.sockel_classes);
        }
    }

    #[test]
    fn names_a_local_module_in_token_values() {
        let plain = sockel().build().unwrap();
        let config = sockel()
            .with_module(fixture("tests/module/_vars.scss"), "v")
            .with("text-md", "v.$text-md");
        for output in [
            config.build().unwrap(),
            config
                .clone()
                .local(fixture("fixture/style"))
                .prefix("fixture-")
                .build()
                .unwrap(),
        ] {
            assert!(
                output.css.contains("--text-md: 1.0625rem"),
                "{}",
                output.css
            );
            assert_eq!(output.sockel_classes, plain.sockel_classes);
        }
    }

    #[test]
    fn finds_the_classes_of_selectors_only() {
        let css = "/* .comment */\n@media (width < 40rem) { .a-b > .c:not(.d) { width: 0.5rem; } }\n\
                   .e[data-x=\".f\"], .e.g { background: url(x.png); }\n:root { --x: 1.5; }";
        assert_eq!(class_names(css), ["a-b", "c", "d", "e", "g"]);
    }

    #[test]
    fn names_a_constant_after_its_class() {
        assert_eq!(const_name("page-head-title").unwrap(), "PAGE_HEAD_TITLE");
        assert_eq!(const_name("stack").unwrap(), "STACK");
        assert!(const_name("a:b").is_err());
        assert!(const_name("-").is_err());
        let classes = ["a-b".to_owned(), "a_b".to_owned()];
        assert!(matches!(classes_rs(&classes), Err(Error::Collision(..))));
        assert_eq!(
            classes_rs(&["stack".to_owned()]).unwrap(),
            "/// `.stack`\npub const STACK: &str = \"stack\";\n"
        );
    }

    /// The declarations of `selector`'s rules in `css`, one per line,
    /// trimmed: those of its first rule, and of the top-level rules after it
    /// that Sass splits off around nested rules.
    fn declarations(css: &str, selector: &str) -> Vec<String> {
        let head = format!("{selector} {{");
        let start = css
            .find(&head)
            .unwrap_or_else(|| panic!("no rule {selector} in {css}"));
        let top = format!("\n{head}\n");
        let rest = css[start + head.len()..].match_indices(&top);
        let starts = rest.map(|(at, _)| start + head.len() + at + top.len() - 1);
        std::iter::once(start + head.len())
            .chain(starts)
            .flat_map(|at| {
                let body = &css[at..];
                body[..body.find('}').unwrap()].split(';')
            })
            .map(|line| line.trim().to_owned())
            .filter(|line| !line.is_empty())
            .collect()
    }

    fn compile_probe(probe: &str) -> Result<String, sasso::Error> {
        let sources = test_sources();
        let options = sasso::Options::new().with_importer(&sources);
        sasso::compile(&format!("@use \"sockel\" as p;\n{probe}"), &options)
    }

    #[test]
    fn the_jitter_mixins_emit_their_rules() {
        let css = compile_probe(
            ".x-ring { @include p.focus-ring; }\n\
             .x-border { @include p.reserved-border; }\n\
             .x-numeric { @include p.numeric; }\n\
             .x-scroll { @include p.scroll-area; }\n\
             .x-truncate { @include p.truncate; }\n\
             .x-clamp { @include p.clamp(3); }\n\
             .x-hidden { @include p.visually-hidden; }\n\
             .x-phone { @include p.phone-only; }\n\
             .x-desktop { @include p.desktop-only; }\n\
             .x-bg { @include p.current; }\n\
             .x-color { @include p.current(color); }\n\
             .x-line { @include p.current(underline); }\n",
        )
        .unwrap();
        let probe = &css[css.find(".x-ring").unwrap()..];

        assert_eq!(
            declarations(probe, ".x-ring:focus-visible"),
            [
                "outline: var(--focus-ring-width) solid var(--focus-ring-color)",
                "outline-offset: var(--focus-ring-offset)"
            ]
        );
        assert_eq!(
            declarations(probe, ".x-border"),
            ["border: var(--space-px) solid transparent"]
        );
        assert_eq!(
            declarations(probe, ".x-numeric"),
            ["font-variant-numeric: tabular-nums"]
        );
        assert!(declarations(probe, ".x-scroll").contains(&"scrollbar-gutter: stable".to_owned()));
        assert!(declarations(probe, ".x-scroll").contains(&"overflow: auto".to_owned()));
        assert!(declarations(probe, ".x-truncate").contains(&"text-overflow: ellipsis".to_owned()));
        assert!(declarations(probe, ".x-clamp").contains(&"-webkit-line-clamp: 3".to_owned()));
        assert!(declarations(probe, ".x-hidden").contains(&"clip-path: inset(50%)".to_owned()));
        assert!(probe.contains("@media (width >= 40rem) {\n  .x-phone {\n    display: none;"));
        assert!(probe.contains("@media (width < 40rem) {\n  .x-desktop {\n    display: none;"));
        let current = "[aria-current]:not([aria-current=false]), ";
        let also = "[aria-checked=true], .x-bg[aria-pressed=true]";
        assert_eq!(
            declarations(
                probe,
                &format!(".x-bg{current}.x-bg[aria-selected=true], .x-bg{also}")
            ),
            ["background: var(--accent)", "color: var(--on-accent)"]
        );
        assert_eq!(
            declarations(
                probe,
                &format!(
                    ".x-color{current}.x-color[aria-selected=true], .x-color[aria-checked=true], .x-color[aria-pressed=true]"
                )
            ),
            ["color: var(--accent)"]
        );
        assert!(
            !probe.contains("font-weight"),
            "a current item keeps its weight"
        );
        assert!(compile_probe(".x { @include p.current(bold); }").is_err());
    }

    #[test]
    fn the_recipe_mixins_are_their_classes_rules() {
        let css = compile_probe(
            ".x-stack { @include p.stack; }\n\
             .x-cluster { @include p.cluster; }\n\
             .x-line { @include p.cluster($wrap: false); }\n\
             .x-card { @include p.card; }\n\
             .x-field { @include p.field; }\n\
             .x-label { @include p.field-label; }\n\
             .x-button { @include p.button; }\n\
             .x-input { @include p.input; }\n\
             .x-primary { @include p.button(primary); }\n",
        )
        .unwrap();
        let (sockel, probe) = css.split_at(css.find(".x-stack").unwrap());
        for (probe_rule, class) in [
            (".x-stack", ".sockel-stack"),
            (".x-cluster", ".sockel-cluster"),
            (".x-card", ".sockel-card"),
            (".x-field", ".sockel-field"),
            (".x-label", ".sockel-field-label"),
            (".x-button", ".sockel-button"),
            (".x-input", ".sockel-input"),
            (".x-button:hover", ".sockel-button:hover"),
            (
                ".x-button:disabled, .x-button[aria-disabled=true]",
                ".sockel-button:disabled, .sockel-button[aria-disabled=true]",
            ),
        ] {
            assert_eq!(
                declarations(probe, probe_rule),
                declarations(sockel, class),
                "{class}"
            );
        }
        assert!(declarations(probe, ".x-line").contains(&"flex-wrap: nowrap".to_owned()));
        let primary = declarations(probe, ".x-primary");
        assert!(primary.contains(&"--button-fill: var(--accent)".to_owned()));
        assert!(primary.contains(&"height: var(--control-height)".to_owned()));
        assert!(compile_probe(".x { @include p.button(loud); }").is_err());
    }

    #[test]
    fn the_colour_contract_defaults_and_configures() {
        let plain = sockel().build().unwrap();
        let root = declarations(&plain.css, ":root");
        for token in [
            "--danger: color-mix(in srgb, #d92d20 70%, var(--text))",
            "--on-accent: var(--bg)",
            "--surface-raised: var(--surface)",
            "--field-border: var(--border)",
            "--field-bg: var(--surface)",
            "--card-border: var(--border)",
            "--card-border-width: var(--space-px)",
            "--band-chrome-ground: var(--surface)",
            "--band-search-text: var(--text-sm)",
            "--action-overlay-radius: var(--radius-sm)",
            "--action-overlay-padding: var(--space-3) var(--space-2)",
            "--panel-border: var(--border)",
            "--panel-tint-text: var(--text)",
            "--menu-item-icon-ink: currentColor",
            "--focus-ring-color: var(--accent)",
        ] {
            assert!(root.iter().any(|line| line == token), "{token} in {root:?}");
        }
        for token in [
            "--success",
            "--warning",
            "--scrim",
            "--layer-sticky",
            "--layer-overlay",
            "--layer-toast",
            "--layer-tooltip",
        ] {
            assert!(
                root.iter()
                    .any(|line| line.starts_with(&format!("{token}: "))),
                "{token}"
            );
        }
        let tuned = sockel().with("danger", "#c00").build().unwrap();
        assert!(declarations(&tuned.css, ":root").contains(&"--danger: #c00".to_owned()));
    }

    #[test]
    fn a_button_takes_its_padding_and_primary_weight_from_settings() {
        let plain = sockel().build().unwrap();
        assert!(
            declarations(&plain.css, ".sockel-button")
                .contains(&"padding-inline: var(--button-pad-x, var(--space-3))".to_owned())
        );
        assert!(
            !declarations(&plain.css, ".sockel-button.sockel-primary")
                .iter()
                .any(|line| line.starts_with("font-weight"))
        );
        let tuned = sockel()
            .with("button-padding", "var(--space-4)")
            .with("button-padding-small", "var(--space-3)")
            .with("button-primary-weight", "600")
            .build()
            .unwrap();
        assert!(
            declarations(&tuned.css, ".sockel-button")
                .contains(&"padding-inline: var(--button-pad-x, var(--space-4))".to_owned())
        );
        assert!(
            declarations(&tuned.css, ".sockel-button.sockel-small")
                .contains(&"padding-inline: var(--button-pad-x, var(--space-3))".to_owned())
        );
        assert!(
            declarations(&tuned.css, ".sockel-button.sockel-primary")
                .contains(&"font-weight: 600".to_owned())
        );
    }

    #[test]
    fn a_dialog_takes_its_padding_body_gap_and_shadow_from_settings() {
        let plain = sockel().build().unwrap();
        assert!(
            declarations(&plain.css, ".sockel-dialog")
                .contains(&"--dialog-pad: var(--space-5)".to_owned())
        );
        assert!(
            declarations(&plain.css, ".sockel-dialog")
                .contains(&"box-shadow: var(--shadow-1)".to_owned())
        );
        let body_gap = |css: &str| {
            css.match_indices(".sockel-dialog-body {")
                .flat_map(|(at, _)| declarations(&css[at..], ".sockel-dialog-body"))
                .find(|line| line.starts_with("gap"))
        };
        assert_eq!(body_gap(&plain.css), None);
        let tuned = sockel()
            .with("dialog-padding", "var(--space-6)")
            .with("dialog-padding-phone", "var(--space-4)")
            .with("dialog-body-gap", "var(--space-4)")
            .with("dialog-shadow", "var(--shadow-2)")
            .build()
            .unwrap();
        assert!(
            declarations(&tuned.css, ".sockel-dialog")
                .contains(&"--dialog-pad: var(--space-6)".to_owned())
        );
        assert!(
            declarations(&tuned.css, ".sockel-dialog.sockel-sheet")
                .contains(&"--dialog-pad: var(--space-4)".to_owned())
        );
        assert_eq!(body_gap(&tuned.css).as_deref(), Some("gap: var(--space-4)"));
        assert!(
            declarations(&tuned.css, ".sockel-dialog")
                .contains(&"box-shadow: var(--shadow-2)".to_owned())
        );
    }

    #[test]
    fn a_dialog_head_and_body_take_their_padding_rule_and_title_from_settings() {
        let plain = sockel().build().unwrap();
        let head = declarations(&plain.css, ".sockel-dialog-head");
        assert!(
            head.contains(&"padding: var(--space-3) var(--space-3) 0 var(--dialog-pad)".to_owned())
        );
        assert!(
            !head
                .iter()
                .any(|l| l.starts_with("border-bottom") || l.starts_with("--section-title-size"))
        );
        assert!(
            !plain
                .css
                .contains(".sockel-dialog-head > .sockel-head-heading")
        );
        let tuned = sockel()
            .with("dialog-head-padding", "var(--space-3) var(--space-4)")
            .with("dialog-head-rule", "1px solid var(--border)")
            .with("dialog-body-padding", "0")
            .with("dialog-title-size", "var(--text-sm)")
            .with("dialog-title-weight", "500")
            .build()
            .unwrap();
        let head = declarations(&tuned.css, ".sockel-dialog-head");
        assert!(head.contains(&"padding: var(--space-3) var(--space-4)".to_owned()));
        assert!(head.contains(&"border-bottom: 1px solid var(--border)".to_owned()));
        assert!(head.contains(&"--section-title-size: var(--text-sm)".to_owned()));
        assert!(declarations(&tuned.css, ".sockel-dialog-body").contains(&"padding: 0".to_owned()));
        assert!(
            declarations(
                &tuned.css,
                ".sockel-dialog-head > .sockel-head-heading > .sockel-section-head-title"
            )
            .contains(&"font-weight: 500".to_owned())
        );
    }

    #[test]
    fn a_menu_and_its_rows_take_their_look_from_settings() {
        let plain = sockel().build().unwrap();
        let menu = declarations(&plain.css, ".sockel-menu");
        assert!(menu.contains(&"background: var(--surface-raised)".to_owned()));
        assert!(!menu.iter().any(|l| l.starts_with("font-size")));
        let tuned = sockel()
            .with("menu-edge", "var(--edge)")
            .with("menu-ground", "var(--ground)")
            .with("menu-shadow", "var(--lift)")
            .with("menu-padding", "6px")
            .with("menu-text", "13px")
            .with("menu-item-height", "28px")
            .with("menu-item-padding", "0 8px")
            .with("menu-item-gap", "8px")
            .with("menu-item-radius", "4px")
            .with("menu-item-hover-fill", "var(--hover)")
            .build()
            .unwrap();
        let menu = declarations(&tuned.css, ".sockel-menu");
        for line in [
            "border-color: var(--edge)",
            "padding: 6px",
            "background: var(--ground)",
            "box-shadow: var(--lift)",
            "font-size: 13px",
        ] {
            assert!(menu.contains(&line.to_owned()), "{line} in {menu:?}");
        }
        let item = declarations(&tuned.css, ".sockel-menu-item");
        for line in [
            "gap: 8px",
            "min-height: 28px",
            "padding: 0 8px",
            "border-radius: 4px",
        ] {
            assert!(item.contains(&line.to_owned()), "{line} in {item:?}");
        }
        assert!(tuned.css.contains("background: var(--hover)"));
    }

    #[test]
    fn a_pager_takes_its_look_from_settings() {
        let tuned = sockel()
            .with("pager-gap", "2px")
            .with("pager-button-radius", "999px")
            .with("pager-button-ink", "var(--muted)")
            .with("pager-button-text", "14px")
            .with("pager-button-hover-fill", "var(--hover)")
            .with("pager-button-hover-ink", "var(--text)")
            .with("pager-current-fill", "var(--chosen)")
            .with("pager-current-ink", "var(--on-chosen)")
            .with("pager-step-ink", "var(--text)")
            .with("pager-phone-height", "44px")
            .build()
            .unwrap();
        assert!(declarations(&tuned.css, ".sockel-pager").contains(&"gap: 2px".to_owned()));
        let button = declarations(&tuned.css, ".sockel-pager-button");
        for line in [
            "border-radius: 999px",
            "color: var(--muted)",
            "font-size: 14px",
        ] {
            assert!(button.contains(&line.to_owned()), "{line} in {button:?}");
        }
        let hover = declarations(&tuned.css, ".sockel-pager-button:hover:not(:disabled)");
        assert!(
            hover.contains(&"color: var(--text)".to_owned()),
            "{hover:?}"
        );
        assert!(tuned.css.contains("background: var(--chosen)"));
        assert!(tuned.css.contains("--control-height: 44px"));
        let mark = declarations(&tuned.css, ".sockel-pager-mark");
        assert!(mark.contains(&"color: var(--text)".to_owned()), "{mark:?}");
    }

    #[test]
    fn a_skeleton_takes_its_look_from_settings() {
        let plain = sockel().build().unwrap();
        assert!(
            !declarations(&plain.css, ".sockel-skeleton")
                .iter()
                .any(|l| l.starts_with("box-shadow"))
        );
        let tuned = sockel()
            .with("skeleton-fill", "var(--glass)")
            .with("skeleton-edge", "var(--soft)")
            .with("skeleton-radius", "0")
            .with("skeleton-line-height", "24px")
            .with("skeleton-line-inset", "0")
            .with("skeleton-phone-height", "44px")
            .build()
            .unwrap();
        let block = declarations(&tuned.css, ".sockel-skeleton");
        for line in [
            "border-radius: 0",
            "background-color: var(--glass)",
            "box-shadow: inset 0 0 0 1px var(--soft)",
        ] {
            assert!(block.contains(&line.to_owned()), "{line} in {block:?}");
        }
        let line = declarations(&tuned.css, ".sockel-skeleton.sockel-line");
        for decl in ["--skeleton-height: 24px", "--skeleton-inset: 0"] {
            assert!(line.contains(&decl.to_owned()), "{decl} in {line:?}");
        }
        assert!(tuned.css.contains("--skeleton-height: 44px"));
    }

    #[test]
    fn a_pick_row_takes_its_look_from_settings() {
        let tuned = sockel()
            .with("pick-row-height", "36px")
            .with("pick-row-padding", "0 6px")
            .with("pick-row-gap", "8px")
            .with("pick-row-radius", "8px")
            .with("pick-row-text", "13px")
            .with("pick-row-hover-fill", "var(--hover)")
            .with("pick-row-detail-ink", "var(--hint)")
            .with("pick-row-detail-text", "11px")
            .build()
            .unwrap();
        let row = declarations(&tuned.css, ".sockel-pick-row");
        for line in [
            "gap: 8px",
            "min-height: 36px",
            "padding: 0 6px",
            "border-radius: 8px",
            "font-size: 13px",
        ] {
            assert!(row.contains(&line.to_owned()), "{line} in {row:?}");
        }
        assert!(tuned.css.contains("--pick-fill: var(--hover)"));
        let detail = declarations(&tuned.css, ".sockel-pick-row-detail");
        for line in ["color: var(--hint)", "font-size: 11px"] {
            assert!(detail.contains(&line.to_owned()), "{line} in {detail:?}");
        }
    }

    #[test]
    fn a_selection_bar_takes_its_look_from_settings() {
        let plain = sockel().build().unwrap();
        let bar = declarations(&plain.css, ".sockel-selection-bar");
        assert!(bar.contains(&"border-top: var(--space-px) solid var(--border)".to_owned()));
        assert!(!bar.iter().any(|l| l.starts_with("background")));
        let tuned = sockel()
            .with("selection-bar-ground", "var(--tint)")
            .with("selection-bar-rule", "none")
            .with("selection-bar-padding", "3px")
            .with("selection-bar-min-height", "0")
            .with("selection-bar-radius", "8px")
            .with("selection-bar-margin", "0 0 4px")
            .with("selection-bar-text", "13px")
            .with("selection-bar-inset-margin", "0 6px 6px")
            .with("selection-bar-count-ink", "var(--ink)")
            .with("selection-bar-count-weight", "500")
            .with("selection-bar-count-padding", "10px")
            .build()
            .unwrap();
        let bar = declarations(&tuned.css, ".sockel-selection-bar");
        for line in [
            "min-height: 0",
            "margin: 0 0 4px",
            "padding: 3px",
            "border-top: none",
            "border-radius: 8px",
            "background: var(--tint)",
            "font-size: 13px",
        ] {
            assert!(bar.contains(&line.to_owned()), "{line} in {bar:?}");
        }
        assert!(tuned.css.contains("margin: 0 6px 6px"));
        let count = declarations(&tuned.css, ".sockel-selection-bar-count");
        for line in [
            "padding-inline-start: 10px",
            "color: var(--ink)",
            "font-weight: 500",
        ] {
            assert!(count.contains(&line.to_owned()), "{line} in {count:?}");
        }
    }

    #[test]
    fn a_menu_rows_edge_is_a_border_unless_set() {
        // The focus rule of its own, not the one it shares with hover.
        let focus_of = |css: &str| {
            let at = css.find("\n.sockel-menu-item:focus-visible {").unwrap();
            declarations(&css[at..], ".sockel-menu-item:focus-visible")
        };
        let plain = sockel().build().unwrap();
        let focus = focus_of(&plain.css);
        assert!(focus.contains(&"outline: none".to_owned()), "{focus:?}");
        let row = declarations(&plain.css, ".sockel-menu-item");
        assert!(!row.contains(&"border-width: 0".to_owned()));
        let tuned = sockel().with("menu-item-edge", "outline").build().unwrap();
        let row = declarations(&tuned.css, ".sockel-menu-item");
        assert!(row.contains(&"border-width: 0".to_owned()), "{row:?}");
        let focus = focus_of(&tuned.css);
        assert!(
            focus.contains(&"outline-offset: calc(-1 * var(--space-px))".to_owned()),
            "{focus:?}"
        );
    }

    #[test]
    fn a_buttons_line_height_is_a_setting() {
        let plain = sockel().build().unwrap();
        let base = declarations(&plain.css, ".sockel-button");
        assert!(base.contains(&"line-height: 1".to_owned()));
        let tuned = sockel()
            .with("button-line-height", "inherit")
            .build()
            .unwrap();
        let base = declarations(&tuned.css, ".sockel-button");
        assert!(
            base.contains(&"line-height: inherit".to_owned()),
            "{base:?}"
        );
    }

    #[test]
    fn a_switch_takes_its_fill_thumb_and_inset_from_settings() {
        let plain = sockel().build().unwrap();
        let switch = declarations(&plain.css, ".sockel-switch");
        assert!(switch.contains(&"--switch-fill: var(--surface)".to_owned()));
        let track = declarations(&plain.css, ".sockel-switch-track");
        assert!(track.contains(&"padding: var(--space-px)".to_owned()));
        let tuned = sockel()
            .with("switch-fill", "var(--sunken)")
            .with("switch-on-thumb", "#fff")
            .with("switch-thumb-inset", "3px")
            .build()
            .unwrap();
        let switch = declarations(&tuned.css, ".sockel-switch");
        assert!(
            switch.contains(&"--switch-fill: var(--sunken)".to_owned()),
            "{switch:?}"
        );
        assert!(tuned.css.contains("--switch-thumb: #fff"));
        let track = declarations(&tuned.css, ".sockel-switch-track");
        assert!(
            track.contains(&"padding: calc(3px - var(--space-px))".to_owned()),
            "{track:?}"
        );
        let thumb = declarations(&tuned.css, ".sockel-switch-thumb");
        assert!(
            thumb.contains(&"width: calc(var(--switch-height) - 2 * 3px)".to_owned()),
            "{thumb:?}"
        );
    }

    #[test]
    fn a_selection_bar_count_wraps_only_when_set() {
        let plain = sockel().build().unwrap();
        let count = declarations(&plain.css, ".sockel-selection-bar-count");
        assert!(count.contains(&"white-space: nowrap".to_owned()));
        let tuned = sockel()
            .with("selection-bar-count-wrap", "true")
            .build()
            .unwrap();
        let count = declarations(&tuned.css, ".sockel-selection-bar-count");
        for line in ["flex: 1 1 0", "white-space: normal", "overflow: visible"] {
            assert!(count.contains(&line.to_owned()), "{line} in {count:?}");
        }
    }

    #[test]
    fn a_menu_keeps_its_scrollbar_room_unless_set() {
        let plain = sockel().build().unwrap();
        let menu = declarations(&plain.css, ".sockel-menu");
        assert!(menu.contains(&"scrollbar-gutter: stable".to_owned()));
        assert!(!menu.contains(&"scrollbar-gutter: auto".to_owned()));
        let tuned = sockel().with("menu-scroll-gutter", "auto").build().unwrap();
        let menu = declarations(&tuned.css, ".sockel-menu");
        assert!(
            menu.contains(&"scrollbar-gutter: auto".to_owned()),
            "{menu:?}"
        );
    }

    #[test]
    fn a_button_takes_its_look_and_colours_per_variant_from_settings() {
        let plain = sockel().build().unwrap();
        let base = declarations(&plain.css, ".sockel-button");
        assert!(base.contains(&"border-radius: var(--radius-inner, var(--radius-sm))".to_owned()));
        assert!(
            !base
                .iter()
                .any(|line| line.starts_with("gap") || line.starts_with("transition"))
        );
        assert!(!plain.css.contains(":where(:active"));
        assert!(
            !declarations(&plain.css, ".sockel-button.sockel-danger")
                .iter()
                .any(|line| line.starts_with("--button-fill"))
        );

        let tuned = sockel()
            .with("button-radius", "3px")
            .with("button-gap", "4px")
            .with("button-transition", "background-color 80ms ease")
            .with("button-press-shade", "rgba(0, 0, 0, 0.2)")
            .with(
                "button-colours",
                "(secondary: (fill: var(--raised), hover-fill: var(--raised-high), hover-ink: var(--text-high)), danger: (edge: var(--border)))",
            )
            .build()
            .unwrap();
        let base = declarations(&tuned.css, ".sockel-button");
        for line in [
            "border-radius: 3px",
            "gap: 4px",
            "transition: background-color 80ms ease",
            "--button-fill: var(--raised)",
            "--button-hover-fill: var(--raised-high)",
            "--button-hover-ink: var(--text-high)",
        ] {
            assert!(base.contains(&line.to_owned()), "{line} in {base:?}");
        }
        assert!(
            declarations(&tuned.css, ".sockel-button.sockel-danger")
                .contains(&"--button-edge: var(--border)".to_owned())
        );
        assert!(
            tuned.css.contains(
                "background-image: linear-gradient(rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.2))"
            )
        );

        assert!(
            sockel()
                .with("button-colours", "(secondary: (glow: red))")
                .build()
                .is_err()
        );
    }

    #[test]
    fn a_button_colour_setting_replaces_sockel_s_in_place() {
        let tuned = sockel()
            .with(
                "button-colours",
                "(danger: (fill: var(--danger-bg), edge: transparent))",
            )
            .build()
            .unwrap();
        let danger = declarations(&tuned.css, ".sockel-button.sockel-danger");
        assert_eq!(
            danger,
            [
                "--button-ink: var(--danger)",
                "--button-edge: transparent",
                "--button-hover-edge: var(--text)",
                "--button-fill: var(--danger-bg)",
            ]
        );
    }

    #[test]
    fn omitted_parts_are_left_out() {
        let plain = sockel().build().unwrap();
        let lean = sockel()
            .with(
                "omit",
                r#"("layout.toolbar", "menu.context", "tokens.seam")"#,
            )
            .build()
            .unwrap();
        for part in [".sockel-toolbar", ".sockel-context-target", "--seam-grip:"] {
            assert!(plain.css.contains(part), "{part}");
            assert!(!lean.css.contains(part), "{part}");
        }
        assert!(lean.css.len() < plain.css.len());
        assert!(sockel().with("omit", r#"("seam")"#).build().is_err());
    }

    #[test]
    fn a_popover_takes_its_padding_and_shadow_from_settings() {
        let plain = sockel().build().unwrap();
        assert!(
            declarations(&plain.css, ".sockel-popover-body")
                .contains(&"padding: var(--space-3)".to_owned())
        );
        assert!(
            declarations(&plain.css, ".sockel-popover")
                .contains(&"box-shadow: var(--shadow-1)".to_owned())
        );
        let tuned = sockel()
            .with("popover-padding", "var(--space-3) var(--space-4)")
            .with("popover-shadow", "var(--shadow-2)")
            .build()
            .unwrap();
        assert!(
            declarations(&tuned.css, ".sockel-popover-body")
                .contains(&"padding: var(--space-3) var(--space-4)".to_owned())
        );
        assert!(
            declarations(&tuned.css, ".sockel-popover")
                .contains(&"box-shadow: var(--shadow-2)".to_owned())
        );
    }

    #[test]
    fn a_dialog_shares_the_overlay_level_and_its_old_variable_still_compiles() {
        let plain = sockel().build().unwrap();
        assert!(
            declarations(&plain.css, ".sockel-dialog-layer")
                .contains(&"z-index: var(--layer-overlay)".to_owned())
        );
        assert!(!plain.css.contains("--layer-dialog"));
        assert!(sockel().with("layer-dialog", "250").build().is_ok());
    }

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(name)
    }

    #[test]
    fn puts_sockel_first_and_verbatim_then_the_local_styles() {
        let base = sockel().build().unwrap();
        let output = sockel()
            .local(fixture("fixture/style"))
            .prefix("fixture-")
            .build()
            .unwrap();
        let local = output.css.strip_prefix(&base.css).unwrap();
        assert_eq!(output.sockel_classes, base.sockel_classes);
        assert_eq!(output.css.matches(".sockel-stack {").count(), 1);
        assert!(local.contains(".fixture-panel {"));
        assert!(local.contains("var(--space-2)"));
        assert!(local.contains("@media (width < 40rem)"));
        assert_eq!(
            output.local_classes,
            ["fixture-panel", "fixture-panel-title"]
        );
    }

    #[test]
    fn keeps_the_charset_first_when_layered() {
        let css = sockel()
            .layer("sockel")
            .layer_order(["a", "sockel"])
            .layered("@charset \"UTF-8\";\n.x { content: \"é\"; }\n");
        assert_eq!(
            css,
            "@charset \"UTF-8\";\n@layer a, sockel;\n@layer sockel {\n.x { content: \"é\"; }\n}\n"
        );
    }

    #[test]
    fn puts_sockel_in_a_layer_after_the_layer_order() {
        let base = sockel().build().unwrap();
        let output = sockel()
            .local(fixture("fixture/style"))
            .prefix("fixture-")
            .layer("sockel")
            .layer_order(["reset", "sockel", "app"])
            .build()
            .unwrap();
        let (charset, body) = match base.css.strip_prefix(CHARSET) {
            Some(body) => (CHARSET, body),
            None => ("", base.css.as_str()),
        };
        let wrapped = format!("{charset}@layer reset, sockel, app;\n@layer sockel {{\n{body}");
        let rest = output.css.strip_prefix(&wrapped).unwrap();
        assert!(rest.trim_start().starts_with('}'));
        assert!(rest.contains(".fixture-panel {"));
        assert_eq!(output.sockel_classes, base.sockel_classes);
        assert_eq!(
            output.local_classes,
            ["fixture-panel", "fixture-panel-title"]
        );
        assert_eq!(
            sockel().layer_order(["a"]).build().unwrap().css,
            format!("{charset}@layer a;\n{body}")
        );
    }

    #[test]
    fn names_local_constants_without_the_prefix() {
        let dir = std::env::temp_dir().join(format!("sockel-build-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        sockel()
            .local(fixture("fixture/style"))
            .prefix("fixture-")
            .out_dir(&dir)
            .write()
            .unwrap();
        let classes = std::fs::read_to_string(dir.join("classes.rs")).unwrap();
        let css = std::fs::read_to_string(dir.join("style.css")).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            classes,
            "/// `.fixture-panel`\npub const PANEL: &str = \"fixture-panel\";\n\
             /// `.fixture-panel-title`\npub const PANEL_TITLE: &str = \"fixture-panel-title\";\n"
        );
        assert!(css.contains(".fixture-panel"));
    }

    #[test]
    fn refuses_an_unprefixed_local_class() {
        let error = sockel()
            .local(fixture("tests/unprefixed"))
            .prefix("fixture-")
            .build()
            .unwrap_err();
        assert!(
            matches!(&error, Error::Unprefixed { class, .. } if class == "panel"),
            "{error}"
        );
        let error = sockel().local(fixture("fixture/style")).build();
        assert!(matches!(error, Err(Error::NoPrefix)));
    }

    #[test]
    fn allows_an_omitted_sockel_class() {
        let output = sockel()
            .local(fixture("tests/omitted"))
            .prefix("fixture-")
            .with("omit", r#"("heads.label",)"#)
            .build()
            .unwrap();
        assert!(!output.sockel_classes.contains(&"sockel-label".to_owned()));
        assert_eq!(output.local_classes, ["fixture-ok"]);
        let error = sockel()
            .local(fixture("tests/misspelt"))
            .prefix("fixture-")
            .with("omit", r#"("heads.label",)"#)
            .build()
            .unwrap_err();
        assert!(
            matches!(&error, Error::Unprefixed { class, .. } if class == "sockel-lable"),
            "{error}"
        );
    }

    #[test]
    fn allows_a_foreign_local_class() {
        let output = sockel()
            .local(fixture("tests/unprefixed"))
            .prefix("fixture-")
            .foreign(["pan*"])
            .build()
            .unwrap();
        assert_eq!(output.local_classes, ["fixture-ok"]);
        assert!(output.css.contains(".panel"));
    }

    #[test]
    fn lists_every_local_file() {
        let dir = fixture("fixture/style");
        let files = local_files(&dir).unwrap();
        assert_eq!(
            files,
            [
                dir.clone(),
                dir.join("_index.scss"),
                dir.join("_panel.scss")
            ]
        );
    }
}
