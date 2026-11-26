//! The CodeEditor page, with a toy highlighter, diagnostics and completions
//! for a small expression language.

use std::iter::Peekable;
use std::rc::Rc;
use std::str::CharIndices;

use yew::prelude::*;

use sockel::{
    CodeEditor, CodeSpan, CodeToken, Completion, CompletionQuery, Completions, Diagnostic,
    Severity, code_editor,
};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Toggle, bind};

const SAMPLE: &str = "// The price with tax, rounded.\nlet price = 120\nlet tax = rate(\"SE\")\nif price > 100 {\n\tround(price * tax)\n} else {\n\tprice + shiping\n}\n";

const KEYWORDS: &[&str] = &["let", "if", "else", "fn", "return", "true", "false"];
const FUNCTIONS: &[(&str, &str)] = &[
    ("rate", "the tax rate of a country"),
    ("round", "to the nearest whole number"),
    ("min", "the least of two"),
    ("max", "the greatest of two"),
];

const ROWS: &[(&str, u32)] = &[("2", 2), ("4", 4), ("8", 8)];

/// Takes the chars `f` accepts, moving `end` past them.
fn eat(chars: &mut Peekable<CharIndices<'_>>, end: &mut usize, f: impl Fn(char) -> bool) {
    while let Some((i, c)) = chars.next_if(|&(_, c)| f(c)) {
        *end = i.saturating_add(c.len_utf8());
    }
}

/// The spans of `text`, and the names `let` binds.
pub(super) fn highlight(text: &str) -> (Vec<CodeSpan>, Vec<String>) {
    let mut spans = Vec::new();
    let mut bound = Vec::new();
    let mut chars = text.char_indices().peekable();
    let mut after_let = false;
    let rest = |at: usize| text.get(at..).unwrap_or_default();

    while let Some((start, c)) = chars.next() {
        let mut end = start.saturating_add(c.len_utf8());

        let token = if c == '/' && rest(end).starts_with('/') {
            eat(&mut chars, &mut end, |c| c != '\n');
            CodeToken::Comment
        } else if c == '"' {
            eat(&mut chars, &mut end, |c| c != '"' && c != '\n');
            if chars.next_if(|&(_, c)| c == '"').is_some() {
                end = end.saturating_add(1);
                CodeToken::String
            } else {
                CodeToken::Error
            }
        } else if c.is_ascii_digit() {
            eat(&mut chars, &mut end, |c| c.is_ascii_digit() || c == '.');
            CodeToken::Number
        } else if c.is_alphabetic() || c == '_' {
            eat(&mut chars, &mut end, |c| c.is_alphanumeric() || c == '_');
            let word = text.get(start..end).unwrap_or_default();
            if after_let {
                bound.push(word.to_owned());
            }
            after_let = word == "let";
            if KEYWORDS.contains(&word) {
                CodeToken::Keyword
            } else if rest(end).starts_with('(') {
                CodeToken::Function
            } else {
                CodeToken::Name
            }
        } else if "+-*/<>=!&|".contains(c) {
            CodeToken::Operator
        } else if "(){}[],.;".contains(c) {
            CodeToken::Punctuation
        } else {
            continue;
        };

        spans.push(CodeSpan::new(start..end, token));
    }

    (spans, bound)
}

/// A warning for each unknown name, an error for each unclosed string and
/// an unbalanced bracket.
fn check(text: &str, spans: &[CodeSpan], bound: &[String]) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut depth = 0u32;

    for span in spans {
        let word = text.get(span.range.clone()).unwrap_or_default();
        match span.token {
            CodeToken::Name if !bound.iter().any(|b| b == word) => out.push(Diagnostic::new(
                span.range.clone(),
                Severity::Warning,
                format!("unknown name `{word}`"),
            )),
            CodeToken::Function if !FUNCTIONS.iter().any(|(f, _)| *f == word) => {
                out.push(Diagnostic::new(
                    span.range.clone(),
                    Severity::Info,
                    format!("`{word}` is not a built-in function"),
                ))
            }
            CodeToken::Error => out.push(Diagnostic::new(
                span.range.clone(),
                Severity::Error,
                "unclosed string",
            )),
            CodeToken::Punctuation if "({[".contains(word) => depth = depth.saturating_add(1),
            CodeToken::Punctuation if ")}]".contains(word) => match depth.checked_sub(1) {
                Some(less) => depth = less,
                None => out.push(Diagnostic::new(
                    span.range.clone(),
                    Severity::Error,
                    "nothing to close",
                )),
            },
            _ => {}
        }
    }

    if depth > 0 {
        out.push(Diagnostic::new(
            text.len()..text.len(),
            Severity::Error,
            "a bracket is left open",
        ));
    }

    out
}

/// The words starting with the one before the cursor.
fn complete(query: CompletionQuery, bound: &[String]) -> Option<Rc<Completions>> {
    let before = query.text.get(..query.cursor)?;
    let start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
        .last()
        .map_or(query.cursor, |(i, _)| i);
    let word = before.get(start..)?;

    if word.is_empty() && !query.forced {
        return None;
    }

    let after = query.text.get(query.cursor..)?;
    let head = before.get(..start)?;
    let names = bound
        .iter()
        .map(|b| (b.as_str(), "a name"))
        .chain(FUNCTIONS.iter().copied())
        .chain(KEYWORDS.iter().map(|k| (*k, "a keyword")));
    let mut seen = Vec::new();

    let items = names
        .filter(|(name, _)| name.starts_with(word) && *name != word)
        .filter(|(name, _)| {
            let new = !seen.contains(name);
            seen.push(*name);
            new
        })
        .map(|(name, detail)| {
            let call = FUNCTIONS.iter().any(|(f, _)| *f == name);
            let insert = if call {
                format!("{name}()")
            } else {
                name.to_owned()
            };
            let at = start
                .saturating_add(name.len())
                .saturating_add(usize::from(call));
            Completion {
                label: name.to_owned().into(),
                detail: Some(detail.into()),
                text: format!("{head}{insert}{after}"),
                select: at..at,
            }
        })
        .collect::<Vec<_>>();

    Some(Rc::new(Completions { start, items }))
}

#[function_component]
pub fn CodeEditorPage() -> Html {
    let text = use_state(|| SAMPLE.to_owned());
    let diagnostics = use_state(|| true);
    let completions = use_state(|| true);
    let readonly = use_state(|| false);
    let rows = use_state(|| 4u32);

    let controls = html! {
        <>
            <Toggle label="diagnostics" value={*diagnostics} on_change={bind(&diagnostics)} />
            <Toggle label="complete" value={*completions} on_change={bind(&completions)} />
            <Toggle label="readonly" value={*readonly} on_change={bind(&readonly)} />
            <Select<u32> label="rows" options={ROWS} value={*rows} on_change={bind(&rows)} />
        </>
    };
    let code = snippet(
        &[
            "let spans: Rc<[CodeSpan]> = highlight(&text).into();".into(),
            "let diagnostics: Rc<[Diagnostic]> = check(&text).into();".into(),
        ],
        El::new("CodeEditor")
            .expr("value", "text.clone()")
            .expr("on_input", "set_text")
            .text("label", "Expression")
            .expr("spans", "spans")
            .expr_if(*diagnostics, "diagnostics", "diagnostics")
            .expr_if(*completions, "complete", "Some(complete)")
            .expr_if(*rows != 4, "rows", rows.to_string())
            .flag("readonly", *readonly),
    );

    let (spans, bound) = highlight(&text);
    let found = if *diagnostics {
        check(&text, &spans, &bound)
    } else {
        Vec::new()
    };
    let count = |s: Severity| found.iter().filter(|d| d.severity == s).count();
    let (errors, warnings, notes) = (
        count(Severity::Error),
        count(Severity::Warning),
        count(Severity::Info),
    );
    let invalid = errors > 0;
    let complete = completions.then(|| {
        let bound = bound.clone();
        Callback::from(move |query: CompletionQuery| complete(query, &bound))
    });
    let on_input = {
        let text = text.clone();
        Callback::from(move |value: String| text.set(value))
    };
    let value = AttrValue::from((*text).clone());

    html! {
        <Demo {controls} {code}>
            <CodeEditor {value} {on_input} label="Expression" spans={Rc::<[CodeSpan]>::from(spans)}
                diagnostics={Rc::<[Diagnostic]>::from(found)} {complete}
                rows={*rows} readonly={*readonly} testid="code-editor" />
            <span class={class::CONTROL_LABEL}>
                { format!(
                    "{} errors, {} warnings, {} notes. Tab and Shift+Tab indent and outdent, Enter keeps the indent, Ctrl+Space asks for completions; code_editor({invalid}) → \"{}\".",
                    errors, warnings, notes, code_editor(invalid).to_string(),
                ) }
            </span>
        </Demo>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sample_highlights_and_checks() {
        let (spans, bound) = highlight(SAMPLE);
        assert_eq!(bound, ["price", "tax"]);
        assert_eq!(spans[0].token, CodeToken::Comment);
        let found = check(SAMPLE, &spans, &bound);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(&SAMPLE[found[0].range.clone()], "shiping");
    }

    #[test]
    fn completes_the_word_at_the_cursor() {
        let query = CompletionQuery {
            text: "let price = 1\npr".into(),
            cursor: 16,
            forced: false,
        };
        let got = complete(query, &["price".into()]).unwrap();
        assert_eq!(got.start, 14);
        assert_eq!(got.items[0].text, "let price = 1\nprice");
    }
}
