//! Builds an app's one stylesheet and its class constants from a `build.rs`:
//! plinth's, and a project's own Sass beside it ([`Config::local`]).
//!
//! plinth's Sass sources are embedded in this crate, so a build sees the
//! sources of the plinth-build version it depends on and no path into
//! plinth's repository; local Sass loads them as `@use "plinth" as p;`.
//! [`Config::compile`] writes `style.css` and `classes.rs` (a `pub const` per
//! class) to `OUT_DIR`.

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

mod usage;
pub use usage::{Report, Usage};

macro_rules! sources {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("../../style/", $name)))),*]
    };
}

/// plinth's Sass sources (`style/`), by file name.
pub const SOURCES: &[(&str, &str)] = sources!(
    "_actions.scss",
    "_badges.scss",
    "_buttons.scss",
    "_card-grid.scss",
    "_color-select.scss",
    "_confirm.scss",
    "_dialog.scss",
    "_disclosure.scss",
    "_drop.scss",
    "_fields.scss",
    "_filter-picker.scss",
    "_heads.scss",
    "_hooks.scss",
    "_icons.scss",
    "_index.scss",
    "_inputs.scss",
    "_layout.scss",
    "_listbox.scss",
    "_media.scss",
    "_menu.scss",
    "_notices.scss",
    "_pager.scss",
    "_picker.scss",
    "_popover.scss",
    "_reorder.scss",
    "_rows.scss",
    "_search-select.scss",
    "_select.scss",
    "_skeleton.scss",
    "_stepper.scss",
    "_table.scss",
    "_tabs.scss",
    "_toast.scss",
    "_toggles.scss",
    "_tokens.scss",
    "_tooltip.scss",
    "_utilities.scss",
);

/// Styles each hook class of `_hooks.scss`, so its class is found like any other.
const HOOKS_PROBE: &str =
    "@use \"plinth/hooks\";\n@each $class in hooks.$classes { .#{$class} { --hook: 0; } }\n";

/// The directory the embedded sources are served under.
const ROOT: &str = "plinth";

/// The embedded sources at relative paths (`plinth/...`), the disk at
/// absolute ones: a project's files are always loaded by absolute path, so
/// nothing on disk can stand in for plinth's sources.
#[derive(Debug)]
struct Sources;

impl Sources {
    /// A relative path without its `.` components.
    fn relative(path: &Path) -> PathBuf {
        path.components()
            .filter(|part| *part != std::path::Component::CurDir)
            .collect()
    }

    fn embedded(path: &Path) -> Option<&'static str> {
        let path = Self::relative(path);
        let name = path.strip_prefix(ROOT).ok()?.to_str()?;
        SOURCES
            .iter()
            .find(|(file, _)| *file == name)
            .map(|(_, source)| *source)
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
            Self::embedded(path).is_some()
        }
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        if path.is_absolute() {
            return std::fs::read(path);
        }
        Self::embedded(path)
            .map(|source| source.as_bytes().to_vec())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.display().to_string()))
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
    local: Option<PathBuf>,
    prefix: Option<String>,
    with: Vec<(String, String)>,
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

    /// Sets a plinth token (`"text-md"` or `"$text-md"`) to a Sass value
    /// (`"1.0625rem"`), as `@use "plinth" with (...)` would. Only values
    /// change; the classes stay the same.
    pub fn with(mut self, variable: impl AsRef<str>, value: impl Into<String>) -> Self {
        let name = variable.as_ref().trim_start_matches('$').to_owned();
        self.with.push((name, value.into()));
        self
    }

    fn plinth_use(&self) -> String {
        if self.with.is_empty() {
            return "@use \"plinth\";\n".to_owned();
        }
        let pairs: Vec<_> = self
            .with
            .iter()
            .map(|(name, value)| format!("${name}: {value}"))
            .collect();
        format!("@use \"plinth\" with ({});\n", pairs.join(", "))
    }

    pub fn build(&self) -> Result<Output, Error> {
        let options = grass::Options::default().fs(&Sources).load_path("");
        let plinth_css = grass::from_string(self.plinth_use(), &options)?;
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
        let entry = format!("{}@use \"{}\";\n", self.plinth_use(), sass_string(&index));
        let css = grass::from_string(entry, &options)?;
        let local_css = css.strip_prefix(&plinth_css).ok_or(Error::NotVerbatim)?;
        let local_classes = class_names(local_css)
            .into_iter()
            .filter(|class| plinth_classes.binary_search(class).is_err())
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

    /// [`Config::write`] for a `build.rs`: reruns when `build.rs` or any local
    /// file changes, and panics with the error.
    pub fn compile(&self) -> Output {
        println!("cargo:rerun-if-changed=build.rs");
        if let Some(dir) = &self.local {
            for path in local_files(dir).unwrap_or_else(|error| panic!("plinth-build: {error}")) {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
        self.write()
            .unwrap_or_else(|error| panic!("plinth-build: {error}"))
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
        let plain = Config::new().build().unwrap();
        let config = Config::new()
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
        let options = grass::Options::default().fs(&Sources).load_path("");
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
    fn the_colour_contract_defaults_and_configures() {
        let plain = Config::new().build().unwrap();
        let root = declarations(&plain.css, ":root");
        for token in [
            "--danger: color-mix(in srgb, #d92d20 70%, var(--text))",
            "--on-accent: var(--bg)",
            "--surface-raised: var(--surface)",
            "--focus-ring-color: var(--accent)",
            "--layer-dialog: 200",
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
        let tuned = Config::new().with("danger", "#c00").build().unwrap();
        assert!(declarations(&tuned.css, ":root").contains(&"--danger: #c00".to_owned()));
    }

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(name)
    }

    #[test]
    fn puts_plinth_first_and_verbatim_then_the_local_styles() {
        let plinth = Config::new().build().unwrap();
        let output = Config::new()
            .local(fixture("fixture/style"))
            .prefix("fixture-")
            .build()
            .unwrap();
        let local = output.css.strip_prefix(&plinth.css).unwrap();
        assert_eq!(output.plinth_classes, plinth.plinth_classes);
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
        Config::new()
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
        let error = Config::new()
            .local(fixture("tests/unprefixed"))
            .prefix("fixture-")
            .build()
            .unwrap_err();
        assert!(
            matches!(&error, Error::Unprefixed { class, .. } if class == "panel"),
            "{error}"
        );
        let error = Config::new().local(fixture("fixture/style")).build();
        assert!(matches!(error, Err(Error::NoPrefix)));
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
