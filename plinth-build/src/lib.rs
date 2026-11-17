//! Builds an app's one stylesheet and its class constants from a `build.rs`:
//! plinth's, and a project's own Sass beside it ([`Config::local`]).
//!
//! plinth's Sass sources are read from the plinth package a build depends on
//! ([`STYLE_VAR`]); local Sass loads them as `@use "plinth" as p;`.
//! [`Config::compile`] writes `style.css` and `classes.rs` (a `pub const` per
//! class) to `OUT_DIR`.

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod features;

/// plinth's sources in this repository.
#[cfg(test)]
const STYLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../style");

#[cfg(test)]
fn plinth() -> Config {
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

/// plinth's Cargo features and the partials (`style/_<name>.scss`) each
/// adds to the stylesheet. Partials no feature names (tokens, hooks, the
/// utilities and what components share) are always in.
pub const FEATURES: &[(&str, &[&str])] = &[
    ("action-group", &["actions"]),
    ("badge", &["badges"]),
    ("button", &["buttons"]),
    ("card-grid", &["card-grid"]),
    ("color-select", &["color-select"]),
    ("confirm", &["confirm"]),
    ("disclosure", &["disclosure"]),
    ("drop-zone", &["drop"]),
    ("field", &["fields"]),
    ("head", &["heads"]),
    ("icons", &["icons"]),
    ("input", &["inputs"]),
    ("layout", &["layout"]),
    ("library-toolbar", &[]),
    ("loadable", &[]),
    ("media", &["media"]),
    ("menu", &["menu"]),
    ("notice", &["notices"]),
    ("overlay", &["popover", "dialog"]),
    ("pager", &["pager"]),
    ("panel", &["panel"]),
    ("picker", &["picker", "filter-picker"]),
    ("reorder", &["reorder"]),
    ("rows", &["rows"]),
    ("search-select", &["search-select"]),
    ("seam", &["seam"]),
    ("select", &["listbox", "select"]),
    ("skeleton", &["skeleton"]),
    ("stepper", &["stepper"]),
    ("table", &["table"]),
    ("tabs", &["tabs"]),
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

/// The features plinth was built with, comma-separated, as a `build.rs` of a
/// package depending on plinth directly sees them (plinth's `links` metadata).
pub const FEATURES_VAR: &str = "DEP_PLINTH_FEATURES";

/// The directory of plinth's Sass sources (`style/` in its package), as a
/// `build.rs` of a package depending on plinth directly sees it.
pub const STYLE_VAR: &str = "DEP_PLINTH_STYLE";

/// The variable [`Config::compile`] sets to plinth's style directory for
/// the package's own crates, so its tests can pass it to [`Config::style`].
pub const TEST_STYLE_VAR: &str = "PLINTH_STYLE";

/// Styles each hook class of `_hooks.scss`, so its class is found like any other.
const HOOKS_PROBE: &str =
    "@use \"plinth/hooks\";\n@each $class in hooks.$classes { .#{$class} { --hook: 0; } }\n";

/// The directory plinth's sources are served under.
const ROOT: &str = "plinth";

/// plinth's sources at relative paths (`plinth/...`), the disk at absolute
/// ones: a project's files are always loaded by absolute path, so nothing
/// else on disk can stand in for plinth's sources.
#[derive(Debug)]
struct Sources {
    style: PathBuf,
    /// Stands in for `_index.scss`: the index of a subset of features.
    index: Option<String>,
}

impl Sources {
    /// A relative path without its `.` components.
    fn relative(path: &Path) -> PathBuf {
        path.components()
            .filter(|part| *part != std::path::Component::CurDir)
            .collect()
    }

    /// A relative path's file in plinth's style directory.
    fn plinth(&self, path: &Path) -> Option<PathBuf> {
        let path = Self::relative(path);
        let name = path.strip_prefix(ROOT).ok()?;
        Some(self.style.join(name))
    }

    fn is_index(&self, path: &Path) -> bool {
        self.index.is_some() && Self::relative(path) == Path::new(ROOT).join("_index.scss")
    }
}

impl grass::Fs for Sources {
    fn is_dir(&self, path: &Path) -> bool {
        if path.is_absolute() {
            path.is_dir()
        } else {
            Self::relative(path) == Path::new(ROOT)
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        if path.is_absolute() {
            path.is_file()
        } else {
            self.is_index(path) || self.plinth(path).is_some_and(|file| file.is_file())
        }
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        if path.is_absolute() {
            return std::fs::read(path);
        }
        if let (true, Some(index)) = (self.is_index(path), &self.index) {
            return Ok(index.as_bytes().to_vec());
        }
        match self.plinth(path) {
            Some(file) => std::fs::read(file),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                path.display().to_string(),
            )),
        }
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        if path.is_absolute() {
            std::fs::canonicalize(path)
        } else {
            Ok(Self::relative(path))
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Sass(Box<grass::Error>),
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
    /// plinth's CSS did not come out first and unchanged.
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
                "local class .{class} is neither plinth's nor prefixed `{prefix}`"
            ),
            Error::NoIndex(dir) => {
                write!(f, "{} has no _index.scss or index.scss", dir.display())
            }
            Error::NotVerbatim => write!(f, "plinth's CSS did not come first and unchanged"),
            Error::Feature(name) => write!(f, "plinth has no feature `{name}`"),
            Error::NoStyle => write!(
                f,
                "plinth's style directory is unknown: {STYLE_VAR} is set only in the \
                 build.rs of a package depending on plinth directly; elsewhere use \
                 Config::style"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<Box<grass::Error>> for Error {
    fn from(error: Box<grass::Error>) -> Self {
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
    features: Option<Vec<String>>,
    foreign: Vec<String>,
}

/// A build's results.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The app's one stylesheet: plinth's CSS, then the local styles' CSS.
    pub css: String,
    /// plinth's classes: every class its selectors name and every hook class, sorted.
    pub plinth_classes: Vec<String>,
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

    /// plinth's Sass sources, instead of [`STYLE_VAR`]'s directory: in a test
    /// `env!("PLINTH_STYLE")` ([`TEST_STYLE_VAR`]), set by the package's
    /// `build.rs`.
    pub fn style(mut self, dir: impl Into<PathBuf>) -> Self {
        self.style = Some(dir.into());
        self
    }

    /// The project's own Sass: `dir/_index.scss` (or `index.scss`), which can
    /// `@use "plinth" as p;` for plinth's tokens and mixins. A relative
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

    /// Sets a plinth token (`"text-md"` or `"$text-md"`) to a Sass value
    /// (`"1.0625rem"`), as `@use "plinth" with (...)` would. Only values
    /// change; the classes stay the same.
    pub fn with(mut self, variable: impl AsRef<str>, value: impl Into<String>) -> Self {
        let name = variable.as_ref().trim_start_matches('$').to_owned();
        self.with.push((name, value.into()));
        self
    }

    /// Builds plinth's CSS from only these [`FEATURES`] (each with the
    /// features it depends on already in), instead of all of them, and
    /// leaves out the [`FEATURE_PARTS`] none of them reads. Without it, the
    /// features are [`FEATURES_VAR`]'s when it is set (in the `build.rs` of
    /// a package depending on plinth directly), so the app's stylesheet has
    /// plinth's subset, and all of them otherwise.
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

    fn plinth_use(&self, enabled: Option<&[String]>) -> String {
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
            return format!("{head}@use \"plinth\";\n");
        }
        format!("{head}@use \"plinth\" with ({});\n", pairs.join(", "))
    }

    pub fn build(&self) -> Result<Output, Error> {
        let enabled = self.enabled();
        let style = self.style_dir()?;
        let sources = Sources {
            index: Self::index(&style, enabled.as_deref())?,
            style,
        };
        let options = grass::Options::default().fs(&sources).load_path("");
        let plinth_use = self.plinth_use(enabled.as_deref());
        let plinth_css = grass::from_string(&plinth_use, &options)?;
        let hooks = grass::from_string(HOOKS_PROBE, &options)?;
        let mut plinth_classes = class_names(&plinth_css);
        plinth_classes.extend(class_names(&hooks));
        plinth_classes.sort();
        plinth_classes.dedup();

        let Some(dir) = &self.local else {
            return Ok(Output {
                css: plinth_css,
                plinth_classes,
                local_classes: Vec::new(),
            });
        };
        let prefix = self.prefix.as_deref().ok_or(Error::NoPrefix)?;
        let index = local_index(dir)?;
        // One compilation, plinth first, so a local `@use "plinth"` reuses
        // the module loaded here and emits nothing again.
        let entry = format!("{plinth_use}@use \"{}\";\n", sass_string(&index));
        let css = grass::from_string(entry, &options)?;
        let local_css = css.strip_prefix(&plinth_css).ok_or(Error::NotVerbatim)?;
        let local_classes = class_names(local_css)
            .into_iter()
            .filter(|class| plinth_classes.binary_search(class).is_err())
            .filter(|class| !self.foreign.iter().any(|pattern| matches(pattern, class)))
            .map(|class| match class.strip_prefix(prefix) {
                Some(rest) if !rest.is_empty() => Ok(class),
                _ => Err(Error::Unprefixed {
                    class,
                    prefix: prefix.to_owned(),
                }),
            })
            .collect::<Result<_, _>>()?;
        Ok(Output {
            css,
            plinth_classes,
            local_classes,
        })
    }

    /// Builds and writes `style.css` (the whole stylesheet) and `classes.rs`:
    /// plinth's constants, or with local styles only the local ones, each
    /// named without its prefix (`plinth-stack` is `STACK`, `army-sidebar`
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
            _ => prefixed_classes_rs(&output.plinth_classes, "plinth-")?,
        };
        std::fs::write(dir.join("style.css"), &output.css)?;
        std::fs::write(dir.join("classes.rs"), classes)?;
        Ok(output)
    }

    /// [`Config::write`] for a `build.rs`: reruns when `build.rs`, a local
    /// file or one of plinth's changes, sets [`TEST_STYLE_VAR`] for the
    /// package's crates, and panics with the error.
    pub fn compile(&self) -> Output {
        let fail = |error: &dyn fmt::Display| -> ! { panic!("plinth-build: {error}") };
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
    fn compiles_plinth_s_sources() {
        let output = plinth().build().unwrap();
        assert!(output.css.contains(".plinth-stack {"));
        for class in ["plinth-page-head-title", "plinth-crumb", "plinth-trailing"] {
            assert!(
                output.plinth_classes.iter().any(|found| found == class),
                "{class}"
            );
        }
        assert!(!output.css.contains("crumb {"));
        let style = Path::new(env!("CARGO_MANIFEST_DIR")).join("../style/_index.scss");
        let on_disk = grass::from_path(style, &grass::Options::default()).unwrap();
        assert_eq!(output.css, on_disk);
    }

    #[test]
    fn configures_token_values_not_classes() {
        let plain = plinth().build().unwrap();
        let config = plinth()
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
            assert_eq!(output.plinth_classes, plain.plinth_classes);
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

    /// The declarations of `selector`'s rule in `css`, one per line, trimmed.
    fn declarations(css: &str, selector: &str) -> Vec<String> {
        let start = css
            .find(&format!("{selector} {{"))
            .unwrap_or_else(|| panic!("no rule {selector} in {css}"));
        let body = &css[start + selector.len() + 2..];
        body[..body.find('}').unwrap()]
            .split(';')
            .map(|line| line.trim().to_owned())
            .filter(|line| !line.is_empty())
            .collect()
    }

    fn compile_probe(probe: &str) -> Result<String, Box<grass::Error>> {
        let sources = test_sources();
        let options = grass::Options::default().fs(&sources).load_path("");
        grass::from_string(format!("@use \"plinth\" as p;\n{probe}"), &options)
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
        let (plinth, probe) = css.split_at(css.find(".x-stack").unwrap());
        for (probe_rule, class) in [
            (".x-stack", ".plinth-stack"),
            (".x-cluster", ".plinth-cluster"),
            (".x-card", ".plinth-card"),
            (".x-field", ".plinth-field"),
            (".x-label", ".plinth-field-label"),
            (".x-button", ".plinth-button"),
            (".x-input", ".plinth-input"),
            (".x-button:hover", ".plinth-button:hover"),
            (
                ".x-button:disabled, .x-button[aria-disabled=true]",
                ".plinth-button:disabled, .plinth-button[aria-disabled=true]",
            ),
        ] {
            assert_eq!(
                declarations(probe, probe_rule),
                declarations(plinth, class),
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
        let plain = plinth().build().unwrap();
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
        let tuned = plinth().with("danger", "#c00").build().unwrap();
        assert!(declarations(&tuned.css, ":root").contains(&"--danger: #c00".to_owned()));
    }

    #[test]
    fn a_button_takes_its_padding_and_primary_weight_from_settings() {
        let plain = plinth().build().unwrap();
        assert!(
            declarations(&plain.css, ".plinth-button")
                .contains(&"padding-inline: var(--button-pad-x, var(--space-3))".to_owned())
        );
        assert!(
            !declarations(&plain.css, ".plinth-button.plinth-primary")
                .iter()
                .any(|line| line.starts_with("font-weight"))
        );
        let tuned = plinth()
            .with("button-padding", "var(--space-4)")
            .with("button-padding-small", "var(--space-3)")
            .with("button-primary-weight", "600")
            .build()
            .unwrap();
        assert!(
            declarations(&tuned.css, ".plinth-button")
                .contains(&"padding-inline: var(--button-pad-x, var(--space-4))".to_owned())
        );
        assert!(
            declarations(&tuned.css, ".plinth-button.plinth-small")
                .contains(&"padding-inline: var(--button-pad-x, var(--space-3))".to_owned())
        );
        assert!(
            declarations(&tuned.css, ".plinth-button.plinth-primary")
                .contains(&"font-weight: 600".to_owned())
        );
    }

    #[test]
    fn a_dialog_takes_its_padding_body_gap_and_shadow_from_settings() {
        let plain = plinth().build().unwrap();
        assert!(
            declarations(&plain.css, ".plinth-dialog")
                .contains(&"--dialog-pad: var(--space-5)".to_owned())
        );
        assert!(
            declarations(&plain.css, ".plinth-dialog")
                .contains(&"box-shadow: var(--shadow-1)".to_owned())
        );
        let body_gap = |css: &str| {
            css.match_indices(".plinth-dialog-body {")
                .flat_map(|(at, _)| declarations(&css[at..], ".plinth-dialog-body"))
                .find(|line| line.starts_with("gap"))
        };
        assert_eq!(body_gap(&plain.css), None);
        let tuned = plinth()
            .with("dialog-padding", "var(--space-6)")
            .with("dialog-padding-phone", "var(--space-4)")
            .with("dialog-body-gap", "var(--space-4)")
            .with("dialog-shadow", "var(--shadow-2)")
            .build()
            .unwrap();
        assert!(
            declarations(&tuned.css, ".plinth-dialog")
                .contains(&"--dialog-pad: var(--space-6)".to_owned())
        );
        assert!(
            declarations(&tuned.css, ".plinth-dialog.plinth-sheet")
                .contains(&"--dialog-pad: var(--space-4)".to_owned())
        );
        assert_eq!(body_gap(&tuned.css).as_deref(), Some("gap: var(--space-4)"));
        assert!(
            declarations(&tuned.css, ".plinth-dialog")
                .contains(&"box-shadow: var(--shadow-2)".to_owned())
        );
    }

    #[test]
    fn a_dialog_head_and_body_take_their_padding_rule_and_title_from_settings() {
        let plain = plinth().build().unwrap();
        let head = declarations(&plain.css, ".plinth-dialog-head");
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
                .contains(".plinth-dialog-head > .plinth-head-heading")
        );
        let tuned = plinth()
            .with("dialog-head-padding", "var(--space-3) var(--space-4)")
            .with("dialog-head-rule", "1px solid var(--border)")
            .with("dialog-body-padding", "0")
            .with("dialog-title-size", "var(--text-sm)")
            .with("dialog-title-weight", "500")
            .build()
            .unwrap();
        let head = declarations(&tuned.css, ".plinth-dialog-head");
        assert!(head.contains(&"padding: var(--space-3) var(--space-4)".to_owned()));
        assert!(head.contains(&"border-bottom: 1px solid var(--border)".to_owned()));
        assert!(head.contains(&"--section-title-size: var(--text-sm)".to_owned()));
        assert!(declarations(&tuned.css, ".plinth-dialog-body").contains(&"padding: 0".to_owned()));
        assert!(
            declarations(
                &tuned.css,
                ".plinth-dialog-head > .plinth-head-heading > .plinth-section-head-title"
            )
            .contains(&"font-weight: 500".to_owned())
        );
    }

    #[test]
    fn a_menu_and_its_rows_take_their_look_from_settings() {
        let plain = plinth().build().unwrap();
        let menu = declarations(&plain.css, ".plinth-menu");
        assert!(menu.contains(&"background: var(--surface-raised)".to_owned()));
        assert!(!menu.iter().any(|l| l.starts_with("font-size")));
        let tuned = plinth()
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
        let menu = declarations(&tuned.css, ".plinth-menu");
        for line in [
            "border-color: var(--edge)",
            "padding: 6px",
            "background: var(--ground)",
            "box-shadow: var(--lift)",
            "font-size: 13px",
        ] {
            assert!(menu.contains(&line.to_owned()), "{line} in {menu:?}");
        }
        let item = declarations(&tuned.css, ".plinth-menu-item");
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
        let tuned = plinth()
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
        assert!(declarations(&tuned.css, ".plinth-pager").contains(&"gap: 2px".to_owned()));
        let button = declarations(&tuned.css, ".plinth-pager-button");
        for line in [
            "border-radius: 999px",
            "color: var(--muted)",
            "font-size: 14px",
        ] {
            assert!(button.contains(&line.to_owned()), "{line} in {button:?}");
        }
        let hover = declarations(&tuned.css, ".plinth-pager-button:hover:not(:disabled)");
        assert!(
            hover.contains(&"color: var(--text)".to_owned()),
            "{hover:?}"
        );
        assert!(tuned.css.contains("background: var(--chosen)"));
        assert!(tuned.css.contains("--control-height: 44px"));
        let mark = declarations(&tuned.css, ".plinth-pager-mark");
        assert!(mark.contains(&"color: var(--text)".to_owned()), "{mark:?}");
    }

    #[test]
    fn a_skeleton_takes_its_look_from_settings() {
        let plain = plinth().build().unwrap();
        assert!(
            !declarations(&plain.css, ".plinth-skeleton")
                .iter()
                .any(|l| l.starts_with("box-shadow"))
        );
        let tuned = plinth()
            .with("skeleton-fill", "var(--glass)")
            .with("skeleton-edge", "var(--soft)")
            .with("skeleton-radius", "0")
            .with("skeleton-line-height", "24px")
            .with("skeleton-line-inset", "0")
            .with("skeleton-phone-height", "44px")
            .build()
            .unwrap();
        let block = declarations(&tuned.css, ".plinth-skeleton");
        for line in [
            "border-radius: 0",
            "background-color: var(--glass)",
            "box-shadow: inset 0 0 0 1px var(--soft)",
        ] {
            assert!(block.contains(&line.to_owned()), "{line} in {block:?}");
        }
        let line = declarations(&tuned.css, ".plinth-skeleton.plinth-line");
        for decl in ["--skeleton-height: 24px", "--skeleton-inset: 0"] {
            assert!(line.contains(&decl.to_owned()), "{decl} in {line:?}");
        }
        assert!(tuned.css.contains("--skeleton-height: 44px"));
    }

    #[test]
    fn a_pick_row_takes_its_look_from_settings() {
        let tuned = plinth()
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
        let row = declarations(&tuned.css, ".plinth-pick-row");
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
        let detail = declarations(&tuned.css, ".plinth-pick-row-detail");
        for line in ["color: var(--hint)", "font-size: 11px"] {
            assert!(detail.contains(&line.to_owned()), "{line} in {detail:?}");
        }
    }

    #[test]
    fn a_selection_bar_takes_its_look_from_settings() {
        let plain = plinth().build().unwrap();
        let bar = declarations(&plain.css, ".plinth-selection-bar");
        assert!(bar.contains(&"border-top: var(--space-px) solid var(--border)".to_owned()));
        assert!(!bar.iter().any(|l| l.starts_with("background")));
        let tuned = plinth()
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
        let bar = declarations(&tuned.css, ".plinth-selection-bar");
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
        let count = declarations(&tuned.css, ".plinth-selection-bar-count");
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
            let at = css.find("\n.plinth-menu-item:focus-visible {").unwrap();
            declarations(&css[at..], ".plinth-menu-item:focus-visible")
        };
        let plain = plinth().build().unwrap();
        let focus = focus_of(&plain.css);
        assert!(focus.contains(&"outline: none".to_owned()), "{focus:?}");
        let row = declarations(&plain.css, ".plinth-menu-item");
        assert!(!row.contains(&"border-width: 0".to_owned()));
        let tuned = plinth().with("menu-item-edge", "outline").build().unwrap();
        let row = declarations(&tuned.css, ".plinth-menu-item");
        assert!(row.contains(&"border-width: 0".to_owned()), "{row:?}");
        let focus = focus_of(&tuned.css);
        assert!(
            focus.contains(&"outline-offset: calc(-1 * var(--space-px))".to_owned()),
            "{focus:?}"
        );
    }

    #[test]
    fn a_buttons_line_height_is_a_setting() {
        let plain = plinth().build().unwrap();
        let base = declarations(&plain.css, ".plinth-button");
        assert!(base.contains(&"line-height: 1".to_owned()));
        let tuned = plinth()
            .with("button-line-height", "inherit")
            .build()
            .unwrap();
        let base = declarations(&tuned.css, ".plinth-button");
        assert!(
            base.contains(&"line-height: inherit".to_owned()),
            "{base:?}"
        );
    }

    #[test]
    fn a_switch_takes_its_fill_thumb_and_inset_from_settings() {
        let plain = plinth().build().unwrap();
        let switch = declarations(&plain.css, ".plinth-switch");
        assert!(switch.contains(&"--switch-fill: var(--surface)".to_owned()));
        let track = declarations(&plain.css, ".plinth-switch-track");
        assert!(track.contains(&"padding: var(--space-px)".to_owned()));
        let tuned = plinth()
            .with("switch-fill", "var(--sunken)")
            .with("switch-on-thumb", "#fff")
            .with("switch-thumb-inset", "3px")
            .build()
            .unwrap();
        let switch = declarations(&tuned.css, ".plinth-switch");
        assert!(
            switch.contains(&"--switch-fill: var(--sunken)".to_owned()),
            "{switch:?}"
        );
        assert!(tuned.css.contains("--switch-thumb: #fff"));
        let track = declarations(&tuned.css, ".plinth-switch-track");
        assert!(
            track.contains(&"padding: calc(3px - var(--space-px))".to_owned()),
            "{track:?}"
        );
        let thumb = declarations(&tuned.css, ".plinth-switch-thumb");
        assert!(
            thumb.contains(&"width: calc(var(--switch-height) - 2 * 3px)".to_owned()),
            "{thumb:?}"
        );
    }

    #[test]
    fn a_selection_bar_count_wraps_only_when_set() {
        let plain = plinth().build().unwrap();
        let count = declarations(&plain.css, ".plinth-selection-bar-count");
        assert!(count.contains(&"white-space: nowrap".to_owned()));
        let tuned = plinth()
            .with("selection-bar-count-wrap", "true")
            .build()
            .unwrap();
        let count = declarations(&tuned.css, ".plinth-selection-bar-count");
        for line in ["flex: 1 1 0", "white-space: normal", "overflow: visible"] {
            assert!(count.contains(&line.to_owned()), "{line} in {count:?}");
        }
    }

    #[test]
    fn a_menu_keeps_its_scrollbar_room_unless_set() {
        let plain = plinth().build().unwrap();
        let menu = declarations(&plain.css, ".plinth-menu");
        assert!(menu.contains(&"scrollbar-gutter: stable".to_owned()));
        assert!(!menu.contains(&"scrollbar-gutter: auto".to_owned()));
        let tuned = plinth().with("menu-scroll-gutter", "auto").build().unwrap();
        let menu = declarations(&tuned.css, ".plinth-menu");
        assert!(
            menu.contains(&"scrollbar-gutter: auto".to_owned()),
            "{menu:?}"
        );
    }

    #[test]
    fn a_button_takes_its_look_and_colours_per_variant_from_settings() {
        let plain = plinth().build().unwrap();
        let base = declarations(&plain.css, ".plinth-button");
        assert!(base.contains(&"border-radius: var(--radius-inner, var(--radius-sm))".to_owned()));
        assert!(
            !base
                .iter()
                .any(|line| line.starts_with("gap") || line.starts_with("transition"))
        );
        assert!(!plain.css.contains(":where(:active"));
        assert!(
            !declarations(&plain.css, ".plinth-button.plinth-danger")
                .iter()
                .any(|line| line.starts_with("--button-fill"))
        );

        let tuned = plinth()
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
        let base = declarations(&tuned.css, ".plinth-button");
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
            declarations(&tuned.css, ".plinth-button.plinth-danger")
                .contains(&"--button-edge: var(--border)".to_owned())
        );
        assert!(
            tuned.css.contains(
                "background-image: linear-gradient(rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.2))"
            )
        );

        assert!(
            plinth()
                .with("button-colours", "(secondary: (glow: red))")
                .build()
                .is_err()
        );
    }

    #[test]
    fn a_button_colour_setting_replaces_plinth_s_in_place() {
        let tuned = plinth()
            .with(
                "button-colours",
                "(danger: (fill: var(--danger-bg), edge: transparent))",
            )
            .build()
            .unwrap();
        let danger = declarations(&tuned.css, ".plinth-button.plinth-danger");
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
        let plain = plinth().build().unwrap();
        let lean = plinth()
            .with(
                "omit",
                r#"("layout.toolbar", "menu.context", "tokens.seam")"#,
            )
            .build()
            .unwrap();
        for part in [".plinth-toolbar", ".plinth-context-target", "--seam-grip:"] {
            assert!(plain.css.contains(part), "{part}");
            assert!(!lean.css.contains(part), "{part}");
        }
        assert!(lean.css.len() < plain.css.len());
        assert!(plinth().with("omit", r#"("seam")"#).build().is_err());
    }

    #[test]
    fn a_popover_takes_its_padding_and_shadow_from_settings() {
        let plain = plinth().build().unwrap();
        assert!(
            declarations(&plain.css, ".plinth-popover-body")
                .contains(&"padding: var(--space-3)".to_owned())
        );
        assert!(
            declarations(&plain.css, ".plinth-popover")
                .contains(&"box-shadow: var(--shadow-1)".to_owned())
        );
        let tuned = plinth()
            .with("popover-padding", "var(--space-3) var(--space-4)")
            .with("popover-shadow", "var(--shadow-2)")
            .build()
            .unwrap();
        assert!(
            declarations(&tuned.css, ".plinth-popover-body")
                .contains(&"padding: var(--space-3) var(--space-4)".to_owned())
        );
        assert!(
            declarations(&tuned.css, ".plinth-popover")
                .contains(&"box-shadow: var(--shadow-2)".to_owned())
        );
    }

    #[test]
    fn a_dialog_shares_the_overlay_level_and_its_old_variable_still_compiles() {
        let plain = plinth().build().unwrap();
        assert!(
            declarations(&plain.css, ".plinth-dialog-layer")
                .contains(&"z-index: var(--layer-overlay)".to_owned())
        );
        assert!(!plain.css.contains("--layer-dialog"));
        assert!(plinth().with("layer-dialog", "250").build().is_ok());
    }

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(name)
    }

    #[test]
    fn puts_plinth_first_and_verbatim_then_the_local_styles() {
        let base = plinth().build().unwrap();
        let output = plinth()
            .local(fixture("fixture/style"))
            .prefix("fixture-")
            .build()
            .unwrap();
        let local = output.css.strip_prefix(&base.css).unwrap();
        assert_eq!(output.plinth_classes, base.plinth_classes);
        assert_eq!(output.css.matches(".plinth-stack {").count(), 1);
        assert!(local.contains(".fixture-panel {"));
        assert!(local.contains("var(--space-2)"));
        assert!(local.contains("@media (width < 40rem)"));
        assert_eq!(
            output.local_classes,
            ["fixture-panel", "fixture-panel-title"]
        );
    }

    #[test]
    fn names_local_constants_without_the_prefix() {
        let dir = std::env::temp_dir().join(format!("plinth-build-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        plinth()
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
        let error = plinth()
            .local(fixture("tests/unprefixed"))
            .prefix("fixture-")
            .build()
            .unwrap_err();
        assert!(
            matches!(&error, Error::Unprefixed { class, .. } if class == "panel"),
            "{error}"
        );
        let error = plinth().local(fixture("fixture/style")).build();
        assert!(matches!(error, Err(Error::NoPrefix)));
    }

    #[test]
    fn allows_a_foreign_local_class() {
        let output = plinth()
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
