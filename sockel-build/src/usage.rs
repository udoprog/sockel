//! A test helper for adopters: every class the project's styles define must
//! be written by its sources.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::{Config, Error, const_name};

/// Where a project's classes are written, and the classes nothing in
/// those sources writes on purpose.
///
/// A source writes a class by its constant (`class::PANEL`) or by its name in
/// a string literal. Comments, and each item marked `#[cfg(test)]` (a test
/// module, function or `use`), are not counted.
#[derive(Clone, Debug, Default)]
pub struct Usage {
    sources: Vec<PathBuf>,
    dynamic: Vec<String>,
    foreign: Vec<String>,
    unprefixed: Vec<String>,
    one_file_per_component: bool,
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

    /// Words a class-like literal writes that are not classes, though the
    /// prefix and the word is a styled class (`"button"` beside `.app-button`).
    pub fn unprefixed<S: Into<String>>(mut self, words: impl IntoIterator<Item = S>) -> Self {
        self.unprefixed.extend(words.into_iter().map(Into::into));
        self
    }

    /// Also fails when a class has a top-level rule in more than one local
    /// Sass file: a component's rules live in its own file.
    pub fn one_file_per_component(mut self) -> Self {
        self.one_file_per_component = true;
        self
    }

    /// Counts how often the sources write each class `config` builds: the
    /// local ones, or sockel's own without local styles. With
    /// `STYLE_INVENTORY=PATH` set, also writes `class<TAB>uses` lines to `PATH`.
    pub fn report(&self, config: &Config) -> Result<Report, Error> {
        let (output, omitted, pruned) = config.build_with(true)?;
        let css_bytes = output.css.len();
        let (classes, prefix) = match (&config.local, &config.prefix) {
            (Some(_), Some(prefix)) => (output.local_classes, prefix.as_str()),
            _ => (output.sockel_classes, "sockel-"),
        };
        let stems: BTreeSet<&str> = classes
            .iter()
            .filter_map(|class| class.strip_prefix(prefix))
            .filter(|stem| {
                !self.unprefixed.iter().any(|pattern| matches(pattern, stem))
                    && !self
                        .dynamic
                        .iter()
                        .any(|pattern| matches(pattern, &format!("{prefix}{stem}")))
                    && classes
                        .binary_search_by(|class| class.as_str().cmp(stem))
                        .is_err()
            })
            .collect();
        let mut files = Vec::new();
        let mut known = Helpers::new();
        for source in &self.sources {
            for file in rust_files(source)? {
                let code = code(&std::fs::read_to_string(&file)?);
                helpers(&code, &mut known);
                files.push((file, code));
            }
        }
        let mut unprefixed = BTreeMap::<String, BTreeSet<PathBuf>>::new();
        let mut text = String::new();
        for (file, code) in files {
            for word in bare_words(&code, &known) {
                if stems.contains(word) {
                    unprefixed
                        .entry(word.to_owned())
                        .or_default()
                        .insert(file.clone());
                }
            }
            text.push_str(&code);
            text.push('\n');
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
        let omitted: BTreeMap<String, usize> = omitted
            .into_iter()
            .filter_map(|class| {
                let constant = class.strip_prefix("sockel-").map(const_name).transpose();
                let named = constant.ok().flatten().map_or(0, |name| {
                    text.matches(&format!("sockel::class::{name}")).count()
                });
                let count = named + literals.get(&class).copied().unwrap_or(0);
                (count > 0).then_some((class, count))
            })
            .collect();
        let report = Report {
            css_bytes,
            pruned: pruned.classes,
            pruned_rules: pruned.rules,
            pruned_bytes: pruned.bytes,
            omitted,
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
            unprefixed: unprefixed
                .into_iter()
                .map(|(word, files)| (word, files.into_iter().collect()))
                .collect(),
            split: match (&config.local, self.one_file_per_component) {
                (Some(dir), true) => split_components(dir)?,
                _ => BTreeMap::new(),
            },
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

    pub(crate) fn sources(&self) -> &[PathBuf] {
        &self.sources
    }

    /// The sockel classes among `classes` that sockel's Rust in `sockel_src`
    /// writes only in components the sources never reach, and the sources do
    /// not write themselves. A component is reached when a source, or a
    /// reached component, names one of its module's top-level `pub` items.
    pub(crate) fn unreached(
        &self,
        sockel_src: &Path,
        classes: &[String],
    ) -> Result<BTreeSet<String>, Error> {
        let mut app = String::new();
        for source in &self.sources {
            for file in rust_files(source)? {
                app.push_str(&code(&std::fs::read_to_string(&file)?));
                app.push('\n');
            }
        }
        if !sockel_src.is_dir() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "{}: no directory of sockel's Rust sources, which Config::prune reads",
                    sockel_src.display()
                ),
            )));
        }
        let files = rust_files(sockel_src)?;
        let mut modules = Vec::new();
        for file in files {
            if file.file_name().is_some_and(|name| name == "lib.rs") {
                continue;
            }
            modules.push(Module::new(&code(&std::fs::read_to_string(&file)?)));
        }
        let app_idents = idents(&app);
        let mut reached = app_idents.clone();
        let mut used = vec![false; modules.len()];
        for _ in 0..=modules.len() {
            let mut changed = false;
            for (module, used) in modules.iter().zip(&mut used) {
                if !*used && module.exports.iter().any(|name| reached.contains(name)) {
                    *used = true;
                    changed = true;
                    reached.extend(module.idents.iter().cloned());
                }
            }
            if !changed {
                break;
            }
        }
        let app_literals = words(&literals(&app));
        let built = |words: &BTreeSet<String>, class: &str| {
            words.contains(class)
                || words.iter().any(|word| {
                    word.len() > "sockel-".len() && word.ends_with('-') && class.starts_with(word)
                })
        };
        let mut unreached = BTreeSet::new();
        for class in classes {
            let Some(name) = class
                .strip_prefix("sockel-")
                .and_then(|stem| const_name(stem).ok())
            else {
                continue;
            };
            let writes =
                |module: &Module| module.classes.contains(&name) || module.literals.contains(class);
            let allowed = self
                .dynamic
                .iter()
                .chain(&self.foreign)
                .any(|pattern| matches(pattern, class));
            let kept = allowed
                || app_idents.contains(&name)
                || built(&app_literals, class)
                || !modules.iter().any(writes)
                || modules.iter().zip(&used).any(|(module, used)| {
                    *used && (writes(module) || built(&module.literals, class))
                });
            if !kept {
                unreached.insert(class.clone());
            }
        }
        Ok(unreached)
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
#[non_exhaustive]
pub struct Report {
    /// The stylesheet's size in bytes, uncompressed.
    pub css_bytes: usize,
    /// With [`Config::prune`], the sockel classes whose rules it drops.
    pub pruned: Vec<String>,
    /// With [`Config::prune`], the rules it drops whole.
    pub pruned_rules: usize,
    /// With [`Config::prune`], the bytes it drops, uncompressed.
    pub pruned_bytes: usize,
    /// Each styled class with the number of times the sources write it.
    pub uses: BTreeMap<String, usize>,
    /// Classes written once.
    pub single_use: Vec<String>,
    /// Sockel classes the build leaves out (`$omit`, a feature) that the
    /// sources write (a literal, or `sockel::class::NAME` in full), with the
    /// number of times. Components of sockel's own that emit one are not seen.
    pub omitted: BTreeMap<String, usize>,
    /// Classes never written and not allowed.
    pub dead: Vec<String>,
    /// Allowed patterns matching no styled class.
    pub stale: Vec<String>,
    /// Words written in a class-like literal without the prefix their
    /// class has, with the files writing them.
    pub unprefixed: BTreeMap<String, Vec<PathBuf>>,
    /// With [`Usage::one_file_per_component`], classes with a top-level
    /// rule in more than one local Sass file, with those files.
    pub split: BTreeMap<String, Vec<PathBuf>>,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.dead.is_empty()
            && self.omitted.is_empty()
            && self.stale.is_empty()
            && self.unprefixed.is_empty()
            && self.split.is_empty()
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} classes, {} single-use, {} dead, {} bytes of CSS",
            self.uses.len(),
            self.single_use.len(),
            self.dead.len(),
            self.css_bytes
        )?;
        if !self.pruned.is_empty() {
            writeln!(
                f,
                "pruned: {} classes, {} rules, {} bytes",
                self.pruned.len(),
                self.pruned_rules,
                self.pruned_bytes
            )?;
        }
        for class in &self.dead {
            writeln!(f, "dead: .{class} is styled and written by no source")?;
        }
        for class in self.omitted.keys() {
            writeln!(
                f,
                "omitted: .{class} is written by a source, and the build leaves it out"
            )?;
        }
        for pattern in &self.stale {
            writeln!(f, "stale: allowed `{pattern}` matches no styled class")?;
        }
        for (word, files) in &self.unprefixed {
            writeln!(
                f,
                "unprefixed: \"{word}\" is written as a class without its prefix ({}); \
                 name the class, or allow the word with Usage::unprefixed",
                paths(files)
            )?;
        }
        for (class, files) in &self.split {
            writeln!(
                f,
                "split: .{class} has a rule of its own in {}; keep a component in its \
                 file and add a modifier there",
                paths(files)
            )?;
        }
        Ok(())
    }
}

fn paths(files: &[PathBuf]) -> String {
    files
        .iter()
        .map(|file| file.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
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

/// A module of sockel's: the items it exports, the identifiers it names,
/// the classes it writes by constant and the words of its literals.
struct Module {
    exports: Vec<String>,
    idents: BTreeSet<String>,
    classes: BTreeSet<String>,
    literals: BTreeSet<String>,
}

impl Module {
    fn new(code: &str) -> Self {
        let mut exports = Vec::new();
        for line in code.lines() {
            let Some(rest) = line
                .strip_prefix("pub ")
                .or_else(|| line.strip_prefix("pub(crate) "))
            else {
                continue;
            };
            let words: Vec<&str> = rest
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .filter(|word| !word.is_empty())
                .collect();
            let kinds = ["fn", "struct", "enum", "trait", "type", "static", "union"];
            let name = match words.as_slice() {
                ["const", name, ..] if *name != "fn" => Some(*name),
                _ => words
                    .iter()
                    .position(|word| kinds.contains(word))
                    .and_then(|at| words.get(at + 1).copied()),
            };
            exports.extend(name.map(str::to_owned));
        }
        let classes = code
            .split("class::")
            .skip(1)
            .map(|rest| {
                let end = rest
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(rest.len());
                rest[..end].to_owned()
            })
            .collect();
        Self {
            exports,
            idents: idents(code),
            classes,
            literals: words(&literals(code)),
        }
    }
}

fn idents(code: &str) -> BTreeSet<String> {
    words(&code.replace(|c: char| !c.is_alphanumeric() && c != '_', " "))
}

fn words(text: &str) -> BTreeSet<String> {
    text.split_whitespace().map(str::to_owned).collect()
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
    without_tests(&strip(source))
}

/// `code` without each `#[cfg(test)]` and the item it marks: up to the
/// item's `;`, or through its first `{ ... }` block.
fn without_tests(code: &str) -> String {
    const MARK: &str = "#[cfg(test)]";
    let mut out = String::new();
    let mut rest = code;
    while let Some(at) = find_outside_literals(rest, MARK) {
        out.push_str(&rest[..at]);
        let item = &rest[at + MARK.len()..];
        let mut depth = 0usize;
        let mut inside = false;
        let mut end = item.len();
        for (i, c) in item.char_indices() {
            match c {
                '\u{1}' => inside = true,
                '\u{2}' => inside = false,
                _ if inside => {}
                ';' if depth == 0 => {
                    end = i + 1;
                    break;
                }
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        out.extend(item[..end].chars().filter(|&c| c == '\n'));
        rest = &item[end..];
    }
    out.push_str(rest);
    out
}

fn find_outside_literals(code: &str, needle: &str) -> Option<usize> {
    let mut inside = false;
    for (i, c) in code.char_indices() {
        match c {
            '\u{1}' => inside = true,
            '\u{2}' => inside = false,
            _ if !inside && code[i..].starts_with(needle) => return Some(i),
            _ => {}
        }
    }
    None
}

/// The source without comments, string literals kept as
/// `\u{1}contents\u{2}`.
fn strip(source: &str) -> String {
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

/// The words of [`code`]'s class-like literals: lowercase words and `{}`
/// placeholders only, not an attribute's value (`type="button"`), a match
/// pattern or a comparison. A word with a placeholder in it is left out.
fn bare_words<'a>(code: &'a str, helpers: &Helpers) -> Vec<&'a str> {
    let mut words = Vec::new();
    let mut rest = code;
    while let Some(start) = rest.find('\u{1}') {
        let before = &code[..code.len() - rest.len() + start];
        let after = &rest[start + 1..];
        let Some(end) = after.find('\u{2}') else {
            break;
        };
        let literal = &after[..end];
        rest = &after[end + 1..];
        if !class_position(before, rest, helpers) {
            continue;
        }
        let parts: Vec<&str> = literal.split(' ').collect();
        let class_like = |part: &&str| {
            part.starts_with(|c: char| c.is_ascii_lowercase())
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        };
        let placeholder = |part: &&str| part.starts_with('{') && part.ends_with('}');
        if parts
            .iter()
            .all(|part| class_like(part) || placeholder(part))
        {
            words.extend(parts.into_iter().filter(class_like));
        }
    }
    words
}

/// For each `fn` in the sources (a method's name led by `.`), which of its
/// parameters past `self` are named for a class.
type Helpers = BTreeMap<String, Vec<bool>>;

fn helpers(code: &str, helpers: &mut Helpers) {
    let mut rest = code;
    while let Some(at) = rest.find("fn ") {
        let before_ok = rest[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
        rest = &rest[at + 3..];
        let name_end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        let name = &rest[..name_end];
        let Some(open) = rest.find('(') else {
            break;
        };
        if !before_ok || name.is_empty() || rest[name_end..open].contains([';', '{']) {
            continue;
        }
        let mut depth = 0usize;
        let mut params = Vec::new();
        let mut start = open + 1;
        for (i, c) in rest[open..].char_indices().map(|(i, c)| (open + i, c)) {
            match c {
                '(' | '[' | '<' => depth += 1,
                '>' if rest[..i].ends_with('-') => {}
                ')' | ']' | '>' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        params.push(&rest[start..i]);
                        break;
                    }
                }
                ',' if depth == 1 => {
                    params.push(&rest[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        let is_self = |param: &str| {
            param
                .split(':')
                .next()
                .unwrap_or("")
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .any(|word| word == "self")
        };
        let method = params.first().is_some_and(|param| is_self(param));
        let flags: Vec<bool> = params
            .iter()
            .map(|param| param.trim())
            .filter(|param| !param.is_empty() && !is_self(param))
            .map(|param| names_class(param.split(':').next().unwrap_or("")))
            .collect();
        let key = if method {
            format!(".{name}")
        } else {
            name.to_owned()
        };
        let known = helpers.entry(key).or_default();
        known.resize(known.len().max(flags.len()), false);
        for (known, flag) in known.iter_mut().zip(flags) {
            *known |= flag;
        }
    }
}

fn ident_before(head: &str) -> &str {
    let head = head.trim_end().trim_end_matches('!');
    let at = head
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'))
        .map_or(0, |at| at + 1);
    &head[at..]
}

fn names_class(name: &str) -> bool {
    name.to_ascii_lowercase().contains("class")
}

/// Whether the literal ending `before` sits in a class. Walking out of the
/// groups around it, through `Some(..)`, tuples, slices and the arms of a
/// `match` or `if`, it reaches an attribute, field, binding or macro named
/// for a class, or a parameter of a source's own `fn` named for one. A
/// tuple's element counts only when bound by a tuple pattern naming it.
fn class_position(before: &str, after: &str, helpers: &Helpers) -> bool {
    let after = after.trim_start();
    if after.starts_with("=>") || after.starts_with("==") || after.starts_with("!=") {
        return false;
    }
    let bytes = before.as_bytes();
    let mut depth = 0usize;
    let mut commas = 0usize;
    let mut element = None;
    let mut inside = false;
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        let c = bytes[i];
        match c {
            2 => inside = true,
            1 => inside = false,
            _ if inside => {}
            b')' | b']' | b'}' => depth += 1,
            b'(' | b'[' | b'{' if depth > 0 => depth -= 1,
            b',' if depth == 0 => commas += 1,
            b'(' | b'[' => {
                let head = before[..i].trim_end();
                let name = ident_before(head);
                if c == b'(' && name.is_empty() && element.is_none() {
                    element = Some(commas);
                }
                let params = (c == b'(' && !name.is_empty() && !head.ends_with('!'))
                    .then(|| {
                        let callee = name.rsplit("::").next().unwrap_or(name);
                        let method = head[..head.len() - name.len()].trim_end().ends_with('.');
                        helpers.get(&if method {
                            format!(".{callee}")
                        } else {
                            callee.to_owned()
                        })
                    })
                    .flatten();
                match params {
                    Some(params) => return params.get(commas).copied().unwrap_or(false),
                    None if names_class(name) => return true,
                    None => {}
                }
                commas = 0;
            }
            b'{' => {
                let head = before[..i].trim_end();
                let statement = head
                    .rfind([';', '{', '}'])
                    .map_or(head, |at| &head[at + 1..]);
                let branch = statement.trim() == "else"
                    || statement
                        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .any(|word| word == "match" || word == "if");
                if !branch && (!head.ends_with('=') || head.ends_with("==")) {
                    return false;
                }
                commas = 0;
            }
            b';' if depth == 0 => return false,
            b'=' | b':' if depth == 0 => {
                let head = &before[..i];
                let next = &before[i + 1..];
                if c == b':' && (head.ends_with(':') || next.starts_with(':')) {
                    continue;
                }
                if c == b'=' && next.starts_with('>') {
                    continue;
                }
                if c == b'=' && (head.ends_with(['=', '!', '<', '>']) || next.starts_with('=')) {
                    return false;
                }
                if c == b':' && commas > 0 {
                    return false;
                }
                if names_class(ident_before(head)) {
                    return true;
                }
                if c == b':' {
                    return false;
                }
                let statement = head
                    .rfind([';', '{', '}'])
                    .map_or(head, |at| &head[at + 1..]);
                let pattern = statement.split(':').next().unwrap_or(statement).trim();
                let Some(pattern) = ["let", "const", "static"].iter().find_map(|word| {
                    let rest = pattern
                        .trim_start_matches("pub(crate)")
                        .trim_start_matches("pub")
                        .trim_start();
                    rest.strip_prefix(word)
                        .filter(|rest| rest.starts_with([' ', '(']))
                }) else {
                    return false;
                };
                let pattern = pattern.trim_start();
                let pattern = pattern.strip_prefix("mut ").unwrap_or(pattern);
                return match (pattern.strip_prefix('('), element) {
                    (Some(tuple), Some(at)) => tuple
                        .trim_end_matches(')')
                        .split(',')
                        .nth(at)
                        .is_some_and(names_class),
                    (None, Some(_)) => false,
                    _ => names_class(pattern),
                };
            }
            _ => {}
        }
    }
    false
}

/// The classes with a top-level rule (`.name {`, or `.name,` in a list) in
/// more than one Sass file under `dir`.
fn split_components(dir: &Path) -> Result<BTreeMap<String, Vec<PathBuf>>, Error> {
    let mut owners = BTreeMap::<String, Vec<PathBuf>>::new();
    for file in crate::local_files(dir)? {
        if !file.extension().is_some_and(|ext| ext == "scss") || !file.is_file() {
            continue;
        }
        let text = sass_without_comments(&std::fs::read_to_string(&file)?);
        let roots: BTreeSet<&str> = text
            .lines()
            .filter_map(|line| {
                let name = line.strip_prefix('.')?;
                let end = name
                    .find(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_'))
                    .unwrap_or(name.len());
                let rest = name[end..].trim_start();
                (end > 0 && (rest.starts_with('{') || rest.starts_with(',')))
                    .then_some(&name[..end])
            })
            .collect();
        for class in roots {
            owners
                .entry(class.to_owned())
                .or_default()
                .push(file.clone());
        }
    }
    owners.retain(|_, files| files.len() > 1);
    Ok(owners)
}

/// Sass without `/* */` comments, and `//` ones after a space or at a
/// line's start (never a URL's).
fn sass_without_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if let Some(after) = rest.strip_prefix("/*") {
            rest = after.find("*/").map_or("", |end| &after[end + 2..]);
        } else if rest.starts_with("//") && (out.is_empty() || out.ends_with(char::is_whitespace)) {
            rest = rest.find('\n').map_or("", |end| &rest[end..]);
        } else {
            out.push(c);
            rest = &rest[c.len_utf8()..];
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
    fn skips_only_the_items_marked_for_tests() {
        let source = "#[cfg(test)]\nuse a::B;\nfn f() { \"kept\" }\n\
                      #[cfg(test)]\nmod tests { fn g() { \"}\" ; \"gone\" } }\n\
                      const C: &str = \"after\";\n";
        let words = literals(&code(source));
        assert_eq!(
            words.split_whitespace().collect::<Vec<_>>(),
            ["kept", "after"]
        );
    }

    #[test]
    fn finds_the_dead_class() {
        let (config, usage) = fixture();
        let report = usage.report(&config).unwrap();
        assert_eq!(report.dead, ["usage-dead", "usage-tone-warm"]);
        assert_eq!(report.single_use, ["usage-once"]);
        assert_eq!(report.uses.len(), 4);
        assert!(!report.is_clean());
        assert!(report.css_bytes > 0);
        assert!(
            report
                .to_string()
                .starts_with("4 classes, 1 single-use, 2 dead, ")
        );
    }

    #[test]
    fn passes_with_the_dead_class_allowed() {
        let (config, usage) = fixture();
        let usage = usage.dynamic(["usage-tone-*"]).foreign(["usage-dead"]);
        assert!(usage.report(&config).unwrap().is_clean());
    }

    #[test]
    fn finds_class_words_written_without_the_prefix() {
        let (config, usage) = fixture();
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/usage/renamed");
        let usage = usage
            .source(&dir)
            .dynamic(["usage-tone-*"])
            .foreign(["usage-dead"]);
        let report = usage.report(&config).unwrap();
        let file = dir.join("renamed.rs");
        assert_eq!(
            report.unprefixed,
            BTreeMap::from(
                ["dead", "live", "once"].map(|word| (word.to_owned(), vec![file.clone()]))
            )
        );
        assert!(!report.is_clean());
        assert!(
            report
                .to_string()
                .contains("unprefixed: \"dead\" is written as a class without its prefix")
        );
        let usage = usage.unprefixed(["dead", "live", "once"]);
        assert!(usage.report(&config).unwrap().is_clean());
    }

    #[test]
    fn reads_class_positions_only() {
        let source = "fn pill(class: &str, label: &str) {}\n\
                      fn f() {\n\
                      pill(\"in-call\", \"label\");\n\
                      let (class, key) = match x { A => (\"in-arm\", \"key\") };\n\
                      const TONE_CLASSES: &[(&str, &str)] = &[(\"label\", class::A)];\n\
                      html! { <b class=\"in-attr\" type=\"button\" /> };\n\
                      }\n";
        let code = code(source);
        let mut known = Helpers::new();
        helpers(&code, &mut known);
        assert_eq!(bare_words(&code, &known), ["in-call", "in-arm", "in-attr"]);
    }

    #[test]
    fn finds_a_written_class_the_build_omits() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/omits");
        let usage = Usage::new().source(&dir);
        let kept = usage.report(&crate::sockel()).unwrap();
        assert!(kept.omitted.is_empty());
        let config = crate::sockel().with("omit", r#"("heads.label",)"#);
        let report = usage.report(&config).unwrap();
        assert_eq!(
            report.omitted,
            BTreeMap::from([("sockel-label".to_owned(), 1)])
        );
        assert!(!report.is_clean());
        assert!(report.to_string().contains("omitted: .sockel-label"));
        let styled = config.local(dir.join("../omitted")).prefix("fixture-");
        assert!(usage.report(&styled).unwrap().omitted.is_empty());
    }

    #[test]
    fn finds_a_component_split_over_files_when_asked() {
        let (config, usage) = fixture();
        let usage = usage.dynamic(["usage-tone-*"]).foreign(["usage-dead"]);
        assert!(usage.report(&config).unwrap().split.is_empty());
        let report = usage.one_file_per_component().report(&config).unwrap();
        let style = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/usage/style");
        assert_eq!(
            report.split,
            BTreeMap::from([(
                "usage-once".to_owned(),
                vec![style.join("_index.scss"), style.join("_split.scss")]
            )])
        );
        assert!(!report.is_clean());
    }

    #[test]
    fn finds_the_classes_only_unused_components_write() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/prune");
        let classes = [
            "alpha",
            "alpha-extra",
            "beta",
            "delta",
            "gamma",
            "loose",
            "own",
            "shared",
            "tone-warm",
        ]
        .map(|stem| format!("sockel-{stem}"));
        let usage = Usage::new().source(dir.join("app"));
        let src = dir.join("sockel/src");
        let unreached = usage.unreached(&src, &classes).unwrap();
        assert_eq!(
            unreached.into_iter().collect::<Vec<_>>(),
            ["sockel-delta", "sockel-gamma", "sockel-shared"]
        );
        let unreached = usage
            .dynamic(["sockel-gam*"])
            .foreign(["sockel-delta"])
            .unreached(&src, &classes)
            .unwrap();
        assert_eq!(unreached.into_iter().collect::<Vec<_>>(), ["sockel-shared"]);
        assert!(
            Usage::new()
                .unreached(&dir.join("missing"), &classes)
                .is_err()
        );
    }

    #[test]
    fn prunes_the_rules_of_unused_components() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/prune/button");
        let usage = Usage::new().source(&dir);
        let plain = crate::sockel().build().unwrap();
        let config = crate::sockel().prune(usage.clone());
        let pruned = config.build().unwrap();
        assert!(pruned.css.contains("\n.sockel-button {"));
        assert!(!pruned.css.contains(".sockel-error-box"));
        assert_eq!(pruned.sockel_classes, plain.sockel_classes);
        let report = usage.report(&config).unwrap();
        assert!(
            report
                .pruned
                .iter()
                .any(|class| class == "sockel-error-box")
        );
        assert!(!report.pruned.iter().any(|class| class == "sockel-button"));
        assert_eq!(report.pruned_bytes, plain.css.len() - pruned.css.len());
        assert_eq!(report.css_bytes, pruned.css.len());
        assert!(report.pruned_rules > 0);
        assert!(report.to_string().contains("\npruned: "));
        let off = usage.report(&crate::sockel()).unwrap();
        assert!(off.pruned.is_empty() && off.pruned_rules == 0 && off.pruned_bytes == 0);

        let (local, usage) = fixture();
        let plain = local.build().unwrap();
        let pruned = local.prune(usage).build().unwrap();
        let tail = |css: &str| css[css.find(".usage-").unwrap()..].to_owned();
        assert_eq!(tail(&pruned.css), tail(&plain.css));
        assert!(pruned.css.len() < plain.css.len());
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
