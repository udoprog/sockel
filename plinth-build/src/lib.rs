//! Builds plinth's stylesheet and its class constants from a `build.rs`.
//!
//! plinth's Sass sources are embedded in this crate and compiled verbatim, so
//! a build sees the sources of the plinth-build version it depends on and no
//! path into plinth's repository. [`Config::compile`] writes `plinth.css` and
//! `classes.rs` (a `pub const` per class) to `OUT_DIR`.

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

macro_rules! sources {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("../../style/", $name)))),*]
    };
}

/// plinth's Sass sources (`style/`), by file name.
pub const SOURCES: &[(&str, &str)] = sources!(
    "_actions.scss",
    "_buttons.scss",
    "_heads.scss",
    "_hooks.scss",
    "_index.scss",
    "_layout.scss",
    "_tokens.scss",
);

/// Styles each hook class of `_hooks.scss`, so its class is found like any other.
const HOOKS_PROBE: &str =
    "@use \"plinth/hooks\";\n@each $class in hooks.$classes { .#{$class} { --hook: 0; } }\n";

/// The directory the embedded sources are served under.
const ROOT: &str = "plinth";

/// The embedded sources as a grass file system.
#[derive(Debug)]
struct Embedded;

impl Embedded {
    fn source(path: &Path) -> Option<&'static str> {
        let name = path.strip_prefix(ROOT).ok()?.to_str()?;
        SOURCES
            .iter()
            .find(|(file, _)| *file == name)
            .map(|(_, source)| *source)
    }
}

impl grass::Fs for Embedded {
    fn is_dir(&self, path: &Path) -> bool {
        path == Path::new(ROOT)
    }

    fn is_file(&self, path: &Path) -> bool {
        Self::source(path).is_some()
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        Self::source(path)
            .map(|source| source.as_bytes().to_vec())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.display().to_string()))
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Sass(error) => write!(f, "compile plinth's Sass: {error}"),
            Error::Io(error) => write!(f, "{error}"),
            Error::Name(class) => write!(f, "class .{class} makes no constant name"),
            Error::Collision(a, b) => write!(f, "classes .{a} and .{b} make the same constant"),
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
}

/// A build's results.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub css: String,
    /// Every class the stylesheet's selectors name and every hook class, sorted.
    pub classes: Vec<String>,
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

    pub fn build(&self) -> Result<Output, Error> {
        let options = grass::Options::default().fs(&Embedded);
        let css = grass::from_path(Path::new(ROOT).join("_index.scss"), &options)?;
        let hooks = grass::from_string(HOOKS_PROBE, &options)?;
        let mut classes = class_names(&css);
        classes.extend(class_names(&hooks));
        classes.sort();
        classes.dedup();
        Ok(Output { css, classes })
    }

    /// Builds and writes `plinth.css` and `classes.rs`.
    pub fn write(&self) -> Result<Output, Error> {
        let output = self.build()?;
        let dir = match &self.out_dir {
            Some(dir) => dir.clone(),
            None => std::env::var_os("OUT_DIR")
                .map(PathBuf::from)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OUT_DIR is not set"))?,
        };
        std::fs::write(dir.join("plinth.css"), &output.css)?;
        std::fs::write(dir.join("classes.rs"), classes_rs(&output.classes)?)?;
        Ok(output)
    }

    /// [`Config::write`] for a `build.rs`: panics with the error.
    pub fn compile(&self) -> Output {
        println!("cargo:rerun-if-changed=build.rs");
        self.write()
            .unwrap_or_else(|error| panic!("plinth-build: {error}"))
    }
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
    Ok(consts
        .into_iter()
        .map(|(name, class)| format!("/// `.{class}`\npub const {name}: &str = \"{class}\";\n"))
        .collect())
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
    fn embeds_every_style_source() {
        let style = Path::new(env!("CARGO_MANIFEST_DIR")).join("../style");
        let mut files: Vec<_> = std::fs::read_dir(&style)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        files.sort();
        let embedded: Vec<_> = SOURCES.iter().map(|(name, _)| *name).collect();
        assert_eq!(files, embedded);
        for (name, source) in SOURCES {
            assert_eq!(std::fs::read_to_string(style.join(name)).unwrap(), *source);
        }
    }

    #[test]
    fn compiles_the_embedded_sources() {
        let output = Config::new().build().unwrap();
        assert!(output.css.contains(".stack {"));
        for class in ["page-head-title", "crumb", "trailing"] {
            assert!(output.classes.iter().any(|found| found == class), "{class}");
        }
        assert!(!output.css.contains("crumb {"));
        let style = Path::new(env!("CARGO_MANIFEST_DIR")).join("../style/_index.scss");
        let on_disk = grass::from_path(style, &grass::Options::default()).unwrap();
        assert_eq!(output.css, on_disk);
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
}
