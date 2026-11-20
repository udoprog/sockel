//! Drops the selectors of a stylesheet that only name unreached classes.

use std::collections::BTreeSet;

/// At-rules whose block holds rules.
const GROUPS: &[&str] = &[
    "@media",
    "@supports",
    "@container",
    "@layer",
    "@scope",
    "@document",
    "@starting-style",
];

/// `css` without its selectors naming only classes in `unused` (at least
/// one outside parentheses, so `:not(.x)` alone is kept), without the rules
/// and group at-rules left empty, and the number of rules dropped. Nothing
/// else changes.
pub(crate) fn prune(css: &str, unused: &BTreeSet<String>) -> (String, usize) {
    let mut out = String::with_capacity(css.len());
    let mut rules = 0;
    block(css, unused, &mut out, &mut rules);
    if !css.starts_with(char::is_whitespace) && out.len() < css.len() {
        out.drain(..out.len() - out.trim_start().len());
    }
    (out, rules)
}

fn block(s: &str, unused: &BTreeSet<String>, out: &mut String, rules: &mut usize) {
    let mut at = 0;
    while at < s.len() {
        let start = at;
        let item = s.len() - s[start..].trim_start().len();
        if item == s.len() {
            out.push_str(&s[start..]);
            return;
        }
        if s[item..].starts_with("/*") {
            let end = s[item + 2..]
                .find("*/")
                .map_or(s.len(), |end| item + end + 4);
            out.push_str(&s[start..end]);
            at = end;
            continue;
        }
        let Some((stop, b'{')) = scan(s, item) else {
            let end = scan(s, item).map_or(s.len(), |(stop, _)| stop + 1);
            out.push_str(&s[start..end]);
            at = end;
            continue;
        };
        let close = closing(s, stop);
        let end = (close + 1).min(s.len());
        let prelude = &s[item..stop];
        at = end;
        if prelude.starts_with('@') {
            let name = prelude
                .split(|c: char| c.is_whitespace() || c == '(')
                .next();
            let body = &s[stop + 1..close.min(s.len())];
            if !name.is_some_and(|name| GROUPS.contains(&name)) {
                out.push_str(&s[start..end]);
                continue;
            }
            let mut inner = String::new();
            block(body, unused, &mut inner, rules);
            if inner.trim().is_empty() && !body.trim().is_empty() {
                continue;
            }
            out.push_str(&s[start..=stop]);
            out.push_str(&inner);
            out.push_str(&s[close.min(s.len())..end]);
            continue;
        }
        let selectors = split(prelude.trim_end());
        let kept: Vec<&str> = selectors
            .iter()
            .map(|selector| selector.trim())
            .filter(|selector| !dead(selector, unused))
            .collect();
        if kept.len() == selectors.len() {
            out.push_str(&s[start..end]);
        } else if kept.is_empty() {
            *rules += 1;
        } else {
            let lead = &s[start..item];
            let indent = &lead[lead.rfind('\n').map_or(0, |at| at + 1)..];
            out.push_str(lead);
            out.push_str(&kept.join(&format!(",\n{indent}")));
            out.push_str(&prelude[prelude.trim_end().len()..]);
            out.push_str(&s[stop..end]);
        }
    }
}

/// Whether `selector` names only `unused` classes, one outside parentheses.
fn dead(selector: &str, unused: &BTreeSet<String>) -> bool {
    let mut all = BTreeSet::new();
    crate::selector_classes(selector, &mut all);
    let mut outside = String::new();
    let mut depth = 0usize;
    for c in selector.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => outside.push(c),
            _ => {}
        }
    }
    let mut required = BTreeSet::new();
    crate::selector_classes(&outside, &mut required);
    !required.is_empty() && all.iter().all(|class| unused.contains(class))
}

/// `prelude` split at its top-level commas.
fn split(prelude: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut quote = None;
    let mut start = 0;
    for (i, c) in prelude.char_indices() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '(' | '[') => depth += 1,
            (None, ')' | ']') => depth = depth.saturating_sub(1),
            (None, ',') if depth == 0 => {
                parts.push(&prelude[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&prelude[start..]);
    parts
}

/// The first `{`, `;` or `}` from `from` outside strings, comments and
/// parentheses.
fn scan(s: &str, from: usize) -> Option<(usize, u8)> {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    let mut i = from;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => i = string_end(bytes, i),
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = s[i + 2..].find("*/").map_or(bytes.len(), |end| i + end + 3);
            }
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            c @ (b'{' | b';' | b'}') if depth == 0 => return Some((i, c)),
            _ => {}
        }
        i += 1;
    }
    None
}

/// The `}` closing the `{` at `open`, or `s.len()`.
fn closing(s: &str, open: usize) -> usize {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => i = string_end(bytes, i),
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = s[i + 2..].find("*/").map_or(bytes.len(), |end| i + end + 3);
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    s.len()
}

/// The index of the quote closing the string opened at `at`.
fn string_end(bytes: &[u8], at: usize) -> usize {
    let quote = bytes[at];
    let mut i = at + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            c if c == quote => return i,
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unused(classes: &[&str]) -> BTreeSet<String> {
        classes.iter().map(|class| class.to_string()).collect()
    }

    #[test]
    fn drops_rules_trims_lists_and_empties_groups() {
        let css = ".a {\n  x: 1;\n}\n.b {\n  x: 2;\n}\n\
                   .a,\n.b > .c,\n.b:not(.a) {\n  x: 3;\n}\n\
                   @media (width < 40rem) {\n  .b {\n    x: 4;\n  }\n}\n\
                   @media (hover: none) {\n  .a,\n  .b {\n    x: 5;\n  }\n}\n\
                   @keyframes b {\n  from {\n    opacity: 0;\n  }\n}\n\
                   :root {\n  --b: 1;\n}\n\
                   button:not(.b) {\n  x: 6;\n}\n";
        let (out, rules) = prune(css, &unused(&["b", "c"]));
        assert_eq!(
            out,
            ".a {\n  x: 1;\n}\n\
             .a,\n.b:not(.a) {\n  x: 3;\n}\n\
             @media (hover: none) {\n  .a {\n    x: 5;\n  }\n}\n\
             @keyframes b {\n  from {\n    opacity: 0;\n  }\n}\n\
             :root {\n  --b: 1;\n}\n\
             button:not(.b) {\n  x: 6;\n}\n"
        );
        assert_eq!(rules, 2);
    }

    #[test]
    fn a_dropped_first_rule_leaves_no_blank_start() {
        let (out, _) = prune(".b {\n  x: 1;\n}\n.a {\n  x: 2;\n}\n", &unused(&["b"]));
        assert_eq!(out, ".a {\n  x: 2;\n}\n");
    }

    #[test]
    fn nothing_unused_changes_nothing() {
        let css = crate::sockel().build().unwrap().css;
        assert_eq!(prune(&css, &BTreeSet::new()), (css.clone(), 0));
        let all = crate::class_names(&css).into_iter().collect();
        let (gone, rules) = prune(&css, &all);
        assert!(rules > 100 && gone.len() < css.len() / 2, "{rules}");
    }
}
