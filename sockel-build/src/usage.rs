//! A test helper for adopters: every class the project's styles define must
//! be written by its sources.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::{Config, Error, const_name};

/// Where a project's classes are written, and the classes nothing in
/// those sources writes on purpose.
///
/// A source writes a class by its constant (`class::PANEL`) or by its name in
/// a string literal. Comments, and a file's code from its first
/// `#[cfg(test)]` on, are not counted.
#[derive(Clone, Debug, Default)]
pub struct Usage {
    sources: Vec<PathBuf>,
    dynamic: Vec<String>,
    foreign: Vec<String>,
}

impl Usage {
    pub fn new() -> Self {
        Self::default()
    }

    /// A directory (searched for `.rs` files) or a file the project writes classes in.
    pub fn source(mut self, path: impl Into<PathBuf>) -> Self {
        self.sources.push(path.into());
        self
    }

    /// Classes the project builds at runtime (`"app-tone-*"`, a trailing `*` is a prefix).
    pub fn dynamic<S: Into<String>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.dynamic.extend(classes.into_iter().map(Into::into));
        self
    }

    /// Classes a dependency writes (same patterns as [`Usage::dynamic`]).
    pub fn foreign<S: Into<String>>(mut self, classes: impl IntoIterator<Item = S>) -> Self {
        self.foreign.extend(classes.into_iter().map(Into::into));
        self
    }

    /// Counts how often the sources write each class `config` builds: the
    /// local ones, or sockel's own without local styles. With
    /// `STYLE_INVENTORY=PATH` set, also writes `class<TAB>uses` lines to `PATH`.
    pub fn report(&self, config: &Config) -> Result<Report, Error> {
        let output = config.build()?;
        let (classes, prefix) = match (&config.local, &config.prefix) {
            (Some(_), Some(prefix)) => (output.local_classes, prefix.as_str()),
            _ => (output.sockel_classes, "sockel-"),
        };
        let mut text = String::new();
        for source in &self.sources {
            for file in rust_files(source)? {
                text.push_str(&code(&std::fs::read_to_string(file)?));
                text.push('\n');
            }
        }
        let idents = count_words(&text.replace(|c: char| !c.is_alphanumeric() && c != '_', " "));
        let literals = count_words(&literals(&text));

        let mut uses = BTreeMap::new();
        for class in classes {
            let constant = class.strip_prefix(prefix).map(const_name).transpose()?;
            let count = constant
                .and_then(|name| idents.get(&name).copied())
                .unwrap_or(0)
                + literals.get(&class).copied().unwrap_or(0);
            uses.insert(class, count);
        }
        let allowed = |class: &str| {
            self.dynamic
                .iter()
                .chain(&self.foreign)
                .any(|pattern| matches(pattern, class))
        };
        let stale = self
            .dynamic
            .iter()
            .chain(&self.foreign)
            .filter(|pattern| !uses.keys().any(|class| matches(pattern, class)))
            .cloned()
            .collect();
        let report = Report {
            dead: uses
                .iter()
                .filter(|(class, count)| **count == 0 && !allowed(class))
                .map(|(class, _)| class.clone())
                .collect(),
            single_use: uses
                .iter()
                .filter(|(_, count)| **count == 1)
                .map(|(class, _)| class.clone())
                .collect(),
            stale,
            uses,
        };
        if let Some(path) = std::env::var_os("STYLE_INVENTORY") {
            let lines: String = report
                .uses
                .iter()
                .map(|(class, count)| format!("{class}\t{count}\n"))
                .collect();
            std::fs::write(path, lines)?;
        }
        Ok(report)
    }

    /// Prints the [`Report`] and panics when it is not [clean](Report::is_clean).
    pub fn assert_clean(&self, config: &Config) {
        let report = self
            .report(config)
            .unwrap_or_else(|error| panic!("sockel-build: {error}"));
        println!("{report}");
        assert!(report.is_clean(), "{report}");
    }
}

/// What the sources write of the styled classes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// Each styled class with the number of times the sources write it.
    pub uses: BTreeMap<String, usize>,
    /// Classes written once.
    pub single_use: Vec<String>,
    /// Classes never written and not allowed.
    pub dead: Vec<String>,
    /// Allowed patterns matching no styled class.
    pub stale: Vec<String>,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.dead.is_empty() && self.stale.is_empty()
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} classes, {} single-use, {} dead",
            self.uses.len(),
            self.single_use.len(),
            self.dead.len()
        )?;
        for class in &self.dead {
            writeln!(f, "dead: .{class} is styled and written by no source")?;
        }
        for pattern in &self.stale {
            writeln!(f, "stale: allowed `{pattern}` matches no styled class")?;
        }
        Ok(())
    }
}

pub(crate) fn matches(pattern: &str, class: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => class.starts_with(prefix),
        None => pattern == class,
    }
}

fn rust_files(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for found in crate::local_files(path)? {
        if found.extension().is_some_and(|ext| ext == "rs") && found.is_file() {
            files.push(found);
        }
    }
    Ok(files)
}

fn count_words(text: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for word in text.split(|c: char| c.is_whitespace()) {
        if !word.is_empty() {
            *counts.entry(word.to_owned()).or_insert(0) += 1;
        }
    }
    counts
}

/// The source without comments and without its test code, string literals
/// kept as `\u{1}contents\u{2}`.
fn code(source: &str) -> String {
    let source = source
        .find("#[cfg(test)]")
        .map_or(source, |end| &source[..end]);
    let mut out = String::new();
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => {
                for next in chars.by_ref() {
                    if next == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut last = '\0';
                for next in chars.by_ref() {
                    if last == '*' && next == '/' {
                        break;
                    }
                    last = next;
                }
            }
            '"' => {
                out.push('\u{1}');
                let mut escaped = false;
                for next in chars.by_ref() {
                    match next {
                        _ if escaped => {
                            escaped = false;
                            out.push(next);
                        }
                        '\\' => escaped = true,
                        '"' => break,
                        _ => out.push(next),
                    }
                }
                out.push('\u{2}');
            }
            c => out.push(c),
        }
    }
    out
}

/// The words inside string literals of [`code`]'s output, `{x}` and
/// other punctuation splitting them.
fn literals(code: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in code.chars() {
        match c {
            '\u{1}' => inside = true,
            '\u{2}' => {
                inside = false;
                out.push(' ');
            }
            c if inside => out.push(if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                ' '
            }),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Config, Usage) {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/usage");
        (
            crate::sockel().local(dir.join("style")).prefix("usage-"),
            Usage::new().source(dir.join("src")),
        )
    }

    #[test]
    fn finds_the_dead_class() {
        let (config, usage) = fixture();
        let report = usage.report(&config).unwrap();
        assert_eq!(report.dead, ["usage-dead", "usage-tone-warm"]);
        assert_eq!(report.single_use, ["usage-once"]);
        assert_eq!(report.uses.len(), 4);
        assert!(!report.is_clean());
        assert!(
            report
                .to_string()
                .starts_with("4 classes, 1 single-use, 2 dead")
        );
    }

    #[test]
    fn passes_with_the_dead_class_allowed() {
        let (config, usage) = fixture();
        let usage = usage.dynamic(["usage-tone-*"]).foreign(["usage-dead"]);
        assert!(usage.report(&config).unwrap().is_clean());
    }

    #[test]
    fn flags_an_allowance_that_matches_nothing() {
        let (config, usage) = fixture();
        let report = usage
            .dynamic(["usage-tone-*", "usage-gone"])
            .foreign(["usage-dead"])
            .report(&config)
            .unwrap();
        assert_eq!(report.stale, ["usage-gone"]);
        assert!(!report.is_clean());
    }
}
