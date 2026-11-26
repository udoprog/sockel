//! Finds a dependency's Rust sources through `cargo metadata`, read with a
//! small JSON reader so sockel-build takes no dependency for it.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Error;

/// The source directory of each package named `name` in the build graph of
/// the package at `manifest_dir` (or of the current directory): the
/// directory of its library's root file. Every version of the package found
/// is returned.
pub(crate) fn sources(manifest_dir: Option<&Path>, name: &str) -> Result<Vec<PathBuf>, Error> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.args(["metadata", "--format-version", "1"]);
    if let Some(dir) = manifest_dir {
        command.arg("--manifest-path").arg(dir.join("Cargo.toml"));
    }
    // A build script builds for `TARGET`; packages only other platforms
    // use need not be downloaded to answer.
    if let Some(target) = std::env::var_os("TARGET") {
        command.arg("--filter-platform").arg(target);
    }
    let output = command.output().map_err(|error| {
        failed(format!(
            "run `cargo metadata` to find dependency {name}: {error}"
        ))
    })?;
    if !output.status.success() {
        return Err(failed(format!(
            "`cargo metadata` failed while finding dependency {name}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let json = Parser::new(&text)
        .document()
        .ok_or_else(|| failed("`cargo metadata` printed JSON sockel-build cannot read".into()))?;
    package_sources(&json, name)
}

fn package_sources(json: &Json, name: &str) -> Result<Vec<PathBuf>, Error> {
    let mut dirs = Vec::new();
    let mut found = false;
    for package in json.get("packages").map(Json::array).unwrap_or_default() {
        if package.get("name").and_then(Json::str) != Some(name) {
            continue;
        }
        found = true;
        let lib = package
            .get("targets")
            .map(Json::array)
            .unwrap_or_default()
            .iter()
            .find(|target| {
                target
                    .get("kind")
                    .map(Json::array)
                    .unwrap_or_default()
                    .iter()
                    .any(|kind| matches!(kind.str(), Some("lib" | "rlib" | "dylib")))
            })
            .and_then(|target| target.get("src_path")?.str());
        let Some(dir) = lib.and_then(|path| Path::new(path).parent()) else {
            return Err(failed(format!(
                "dependency {name} has no library whose sources Usage::dependency can read"
            )));
        };
        if !dirs.iter().any(|known: &PathBuf| known == dir) {
            dirs.push(dir.to_owned());
        }
    }
    if !found {
        return Err(failed(format!(
            "Usage::dependency: no package named {name} in the build graph"
        )));
    }
    Ok(dirs)
}

fn failed(message: String) -> Error {
    Error::Io(io::Error::new(io::ErrorKind::NotFound, message))
}

#[derive(Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number,
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn array(&self) -> &[Json] {
        match self {
            Json::Array(items) => items,
            _ => &[],
        }
    }

    fn str(&self) -> Option<&str> {
        match self {
            Json::String(s) => Some(s),
            _ => None,
        }
    }
}

struct Parser<'a> {
    rest: &'a str,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self { rest: text }
    }

    fn document(mut self) -> Option<Json> {
        let value = self.value(0)?;
        self.skip_space();
        self.rest.is_empty().then_some(value)
    }

    fn skip_space(&mut self) {
        self.rest = self.rest.trim_start_matches([' ', '\t', '\n', '\r']);
    }

    fn eat(&mut self, token: &str) -> bool {
        self.skip_space();
        match self.rest.strip_prefix(token) {
            Some(rest) => {
                self.rest = rest;
                true
            }
            None => false,
        }
    }

    fn value(&mut self, depth: usize) -> Option<Json> {
        if depth > 128 {
            return None;
        }
        let depth = depth.checked_add(1)?;
        self.skip_space();
        match self.rest.as_bytes().first()? {
            b'{' => {
                self.eat("{");
                let mut fields = Vec::new();
                if self.eat("}") {
                    return Some(Json::Object(fields));
                }
                loop {
                    self.skip_space();
                    let key = self.string()?;
                    if !self.eat(":") {
                        return None;
                    }
                    fields.push((key, self.value(depth)?));
                    if self.eat("}") {
                        return Some(Json::Object(fields));
                    }
                    if !self.eat(",") {
                        return None;
                    }
                }
            }
            b'[' => {
                self.eat("[");
                let mut items = Vec::new();
                if self.eat("]") {
                    return Some(Json::Array(items));
                }
                loop {
                    items.push(self.value(depth)?);
                    if self.eat("]") {
                        return Some(Json::Array(items));
                    }
                    if !self.eat(",") {
                        return None;
                    }
                }
            }
            b'"' => self.string().map(Json::String),
            b't' => self.eat("true").then_some(Json::Bool(true)),
            b'f' => self.eat("false").then_some(Json::Bool(false)),
            b'n' => self.eat("null").then_some(Json::Null),
            b'-' | b'0'..=b'9' => {
                self.rest = self.rest.trim_start_matches(|c: char| {
                    matches!(c, '-' | '+' | '.' | 'e' | 'E' | '0'..='9')
                });
                Some(Json::Number)
            }
            _ => None,
        }
    }

    fn string(&mut self) -> Option<String> {
        self.rest = self.rest.strip_prefix('"')?;
        let mut out = String::new();
        loop {
            let at = self.rest.find(['"', '\\'])?;
            let (text, rest) = self.rest.split_at_checked(at)?;
            out.push_str(text);
            let mut chars = rest.chars();
            let quote = chars.next()? == '"';
            if quote {
                self.rest = chars.as_str();
                return Some(out);
            }
            let c = chars.next()?;
            self.rest = chars.as_str();
            out.push(match c {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{8}',
                'f' => '\u{c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => {
                    let high = self.hex()?;
                    if (0xd800..0xdc00).contains(&high) {
                        self.rest = self.rest.strip_prefix("\\u")?;
                        let low = self.hex()?;
                        if !(0xdc00..0xe000).contains(&low) {
                            return None;
                        }
                        let code = 0x10000u32.checked_add((high & 0x3ff) << 10 | (low & 0x3ff))?;
                        char::from_u32(code)?
                    } else {
                        char::from_u32(high)?
                    }
                }
                _ => return None,
            });
        }
    }

    fn hex(&mut self) -> Option<u32> {
        let digits = self.rest.get(..4)?;
        let value = u32::from_str_radix(digits, 16).ok()?;
        self.rest = self.rest.get(4..)?;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_json() {
        let json = Parser::new(
            r#" {"a": [1, -2.5e3, true, false, null], "b": "x\"\\\u00e9\ud83d\ude00\n", "c": {}} "#,
        )
        .document()
        .unwrap();
        assert_eq!(json.get("a").unwrap().array().len(), 5);
        assert_eq!(json.get("b").unwrap().str(), Some("x\"\\é😀\n"));
        assert_eq!(json.get("c"), Some(&Json::Object(Vec::new())));
        assert!(Parser::new("[1,]").document().is_none());
        assert!(Parser::new("{\"a\" 1}").document().is_none());
        assert!(Parser::new("\"\\ud800\"").document().is_none());
        assert!(Parser::new("[1] 2").document().is_none());
    }

    #[test]
    fn finds_a_package_s_library_directory() {
        let json = Parser::new(
            r#"{"packages": [
                {"name": "other", "targets": [{"kind": ["lib"], "src_path": "/o/src/lib.rs"}]},
                {"name": "dep", "targets": [
                    {"kind": ["bin"], "src_path": "/d/src/main.rs"},
                    {"kind": ["lib"], "src_path": "/d/src/lib.rs"}
                ]},
                {"name": "bin-only", "targets": [{"kind": ["bin"], "src_path": "/b/main.rs"}]}
            ]}"#,
        )
        .document()
        .unwrap();
        assert_eq!(
            package_sources(&json, "dep").unwrap(),
            [PathBuf::from("/d/src")]
        );
        assert!(package_sources(&json, "missing").is_err());
        assert!(package_sources(&json, "bin-only").is_err());
    }
}
