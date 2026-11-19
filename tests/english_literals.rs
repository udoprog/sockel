//! sockel's components spell no user-visible words: they come from the
//! catalog (`locales/`). A capitalised string literal in component code is
//! taken for one, unless it is a key or a data-transfer type.

use std::path::Path;

const NOT_TEXT: &[&str] = &[
    "Alt",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    "Backspace",
    "Control",
    "Delete",
    "End",
    "Enter",
    "Escape",
    "Files",
    "Home",
    "Meta",
    "PageDown",
    "PageUp",
    "Shift",
    "Space",
    "Tab",
];

/// The capitalised literals of `source` before its tests, outside comments.
fn words(source: &str) -> Vec<(usize, String)> {
    let code = source.split("#[cfg(test)]").next().unwrap_or_default();
    let mut found = Vec::new();
    for (n, line) in code.lines().enumerate() {
        let line = line.split("//").next().unwrap_or_default();
        let mut parts = line.split('"');
        parts.next();
        while let Some(literal) = parts.next() {
            if literal.starts_with(|c: char| c.is_ascii_uppercase())
                && literal
                    .chars()
                    .nth(1)
                    .is_some_and(|c| c.is_ascii_lowercase())
                && !NOT_TEXT.contains(&literal)
            {
                found.push((n + 1, literal.to_owned()));
            }
            parts.next();
        }
    }
    found
}

#[test]
fn components_spell_no_words() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut problems = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for (line, word) in words(&source) {
            problems.push(format!("{}:{line}: {word:?}", path.display()));
        }
    }
    assert!(
        problems.is_empty(),
        "put these in locales/*.ftl and look them up:\n{}",
        problems.join("\n")
    );
}

#[test]
fn a_word_is_found_and_a_key_is_not() {
    let source = "let a = \"Save\";\nif key == \"Enter\" {}\n// \"Note\"\n#[cfg(test)]\n\"Test\"";
    assert_eq!(words(source), vec![(1, "Save".to_owned())]);
}
