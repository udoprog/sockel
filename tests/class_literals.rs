//! plinth's Rust names classes only through `plinth::class`: no class is
//! spelt out in a string literal under `src/`.

use std::path::Path;

/// A source's string literals (their opening quote's offset and contents),
/// and the source with its comments and literals' contents blanked.
fn lex(source: &str) -> (Vec<(usize, String)>, String) {
    let bytes = source.as_bytes();
    let mut masked = bytes.to_vec();
    let mut literals = Vec::new();
    let mut i = 0;
    let blank = |masked: &mut Vec<u8>, from: usize, to: usize| {
        masked[from..to].iter_mut().for_each(|b| {
            if *b != b'\n' {
                *b = b' ';
            }
        });
    };
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let end = source[i..].find('\n').map_or(bytes.len(), |n| i + n);
                blank(&mut masked, i, end);
                i = end;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let end = source[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |n| i + n + 4);
                blank(&mut masked, i, end);
                i = end;
            }
            b'"' => {
                let mut end = i + 1;
                while end < bytes.len() && bytes[end] != b'"' {
                    end += if bytes[end] == b'\\' { 2 } else { 1 };
                }
                literals.push((i, source[i + 1..end].to_owned()));
                blank(&mut masked, i + 1, end);
                i = end + 1;
            }
            b'\'' if bytes.get(i + 1) == Some(&b'\\') => {
                let end = source[i + 2..]
                    .find('\'')
                    .map_or(bytes.len(), |n| i + n + 3);
                blank(&mut masked, i, end);
                i = end;
            }
            b'\'' if bytes.get(i + 2) == Some(&b'\'') => {
                blank(&mut masked, i, i + 3);
                i += 3;
            }
            _ => i += 1,
        }
    }
    (literals, String::from_utf8(masked).expect("ascii masking"))
}

/// The byte ranges of every `classes!( ... )`'s arguments.
fn classes_calls(masked: &str) -> Vec<(usize, usize)> {
    let mut calls = Vec::new();
    for (start, _) in masked.match_indices("classes!(") {
        let open = start + "classes!(".len();
        let mut depth = 1;
        for (n, c) in masked[open..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                calls.push((open, open + n));
                break;
            }
        }
    }
    calls
}

/// The attribute a literal is the value of (`role` in `role="toolbar"`,
/// `type` in `type={if a { "submit" } else { "button" }}`), on its line.
fn attribute(line: &str) -> Option<&str> {
    let eq = line.rfind('=')?;
    let before = line[..eq].trim_end();
    if line[eq + 1..].starts_with(['=', '>']) || before.ends_with(['=', '!', '<', '>']) {
        return None;
    }
    let start = before
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .map_or(0, |n| n + 1);
    Some(&before[start..])
}

fn violations(source: &str, classes: &[String]) -> Vec<String> {
    let (literals, masked) = lex(source);
    let calls = classes_calls(&masked);
    let line_of = |offset: usize| source[..offset].lines().count().max(1);
    let mut found = Vec::new();
    for (offset, text) in literals {
        let line_start = masked[..offset].rfind('\n').map_or(0, |n| n + 1);
        let line = &masked[line_start..offset];
        let attr = attribute(line);
        let reason = if line
            .trim_end()
            .trim_end_matches('{')
            .trim_end()
            .ends_with('=')
            && attr.is_some_and(|attr| attr.ends_with("class"))
        {
            Some("a class attribute")
        } else if calls.iter().any(|&(a, b)| a <= offset && offset < b) {
            Some("in classes!")
        } else if !text.trim().is_empty()
            && text
                .split_whitespace()
                .all(|word| classes.iter().any(|class| class == word))
            && !attr.is_some_and(|attr| {
                ["type", "role"].contains(&attr)
                    || attr.starts_with("aria-")
                    || attr.starts_with("data-")
            })
        {
            Some("a class name")
        } else {
            None
        };
        if let Some(reason) = reason {
            found.push(format!("line {}: \"{text}\" is {reason}", line_of(offset)));
        }
    }
    found
}

fn rust_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read src/") {
        let path = entry.expect("src/ entry").path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn known_classes() -> Vec<String> {
    plinth_build::Config::new()
        .style(env!("PLINTH_STYLE"))
        .build()
        .expect("plinth's Sass")
        .plinth_classes
}

#[test]
fn src_spells_no_class() {
    let classes = known_classes();
    let mut files = Vec::new();
    rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(!files.is_empty());
    let mut found = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).expect("read source");
        for violation in violations(&source, &classes) {
            found.push(format!("{}: {violation}", file.display()));
        }
    }
    assert!(
        found.is_empty(),
        "use plinth::class constants:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_scan_finds_class_literals() {
    let classes = known_classes();
    let source = r#"
        html! { <div class="anything" role="toolbar"> }
        html! { <span class={"plinth-stack"} title_class="x" /> }
        classes!(class::CARD, flush.then_some("other"));
        fn gap() -> Option<&'static str> { Some("plinth-tight") }
        let kind = 'x';
        html! { <button type={if submit { "submit" } else { "button" }} aria-label="card" /> }
        // "plinth-stack" in a comment
        /* class="plinth-stack" */
        let tag = "div";
    "#;
    let found = violations(source, &classes);
    assert_eq!(found.len(), 5, "{found:#?}");
    for text in ["anything", "plinth-stack", "x", "other", "plinth-tight"] {
        assert!(
            found.iter().any(|v| v.contains(&format!("\"{text}\""))),
            "{text}"
        );
    }
}
