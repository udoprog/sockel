//! CodeSnippet: code shown as written, never wrapped, with the app's
//! highlighting and marked spans underlined as CodeEditor underlines
//! diagnostics (`style/_code-snippet.scss`). A block scrolls sideways; an
//! inline one sits in a line of text or a table cell.

use crate::testid::with_testid;
use std::ops::Range;
use std::rc::Rc;

use yew::prelude::*;

use crate::class;
use crate::code::{CodeSpan, Diagnostic, Piece, floor, pieces};

#[derive(Properties, PartialEq)]
pub struct CodeSnippetProps {
    /// The code, one line or more.
    pub code: AttrValue,
    #[prop_or_default]
    pub spans: Rc<[CodeSpan]>,
    /// Underlined in their severity, their messages the mark's tooltip.
    #[prop_or_default]
    pub marks: Rc<[Diagnostic]>,
    /// A `<code>` in the text around it rather than a `<pre>` block.
    #[prop_or_default]
    pub inline: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// The classes of a code snippet: `code-snippet`, and `inline` for one in
/// the text around it.
pub fn code_snippet(inline: bool) -> Classes {
    classes!(class::CODE_SNIPPET, inline.then_some(class::INLINE))
}

/// Code shown as written, `spans` highlighted and `marks` underlined: a
/// `<pre>` scrolling sideways, or an `inline` `<code>`.
#[function_component]
pub fn CodeSnippet(props: &CodeSnippetProps) -> Html {
    let source = props.code.as_str();
    let len = source.len();
    let marks = props
        .marks
        .iter()
        .map(|d| {
            let start = floor(source, d.range.start.min(len));
            let end = floor(source, d.range.end.min(len)).max(start);
            (start..end, d)
        })
        .collect::<Vec<_>>();
    let severities = marks
        .iter()
        .map(|(r, d)| (r.clone(), d.severity))
        .collect::<Vec<_>>();
    let title = |covers: &dyn Fn(&Range<usize>) -> bool| {
        let messages = marks
            .iter()
            .filter(|(r, d)| covers(r) && !d.message.is_empty())
            .map(|(_, d)| d.message.as_str())
            .collect::<Vec<_>>();
        (!messages.is_empty()).then(|| messages.join("\n"))
    };

    let mut at = 0;
    let code = pieces(source, &props.spans, &severities, &[])
        .into_iter()
        .map(|piece| match piece {
            Piece::Text { range, span, mark } => {
                let text = source.get(range.clone()).unwrap_or_default().to_owned();
                at = range.end;
                let token = span
                    .and_then(|i| props.spans.get(i))
                    .map(|s| s.token.class());
                let html = match token {
                    Some(class) => html! { <span class={class}>{ text }</span> },
                    None => html! { { text } },
                };

                match mark {
                    Some(severity) => {
                        let title = title(&|r| {
                            !r.is_empty() && r.start <= range.start && range.end <= r.end
                        });
                        html! {
                            <span class={classes!(class::CODE_SNIPPET_MARK, severity.class())} {title}>{ html }</span>
                        }
                    }
                    None => html,
                }
            }
            Piece::EmptyMark(severity) => {
                let title = title(&|r| r.is_empty() && r.start == at);
                html! {
                    <span class={classes!(class::CODE_SNIPPET_MARK, class::CODE_SNIPPET_EMPTY_MARK, severity.class())} {title}></span>
                }
            }
            Piece::Point(_) => Html::default(),
        })
        .collect::<Html>();

    let classes = classes!(code_snippet(props.inline), props.class.clone());

    with_testid(
        if props.inline {
            html! { <code class={classes}>{ code }</code> }
        } else {
            html! { <pre class={classes}>{ code }</pre> }
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CodeToken, Severity};

    fn render(props: CodeSnippetProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<CodeSnippet>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props(code: &'static str) -> CodeSnippetProps {
        CodeSnippetProps {
            code: code.into(),
            spans: Rc::from([]),
            marks: Rc::from([]),
            inline: false,
            class: Classes::new(),
            testid: None,
        }
    }

    #[test]
    fn its_classes() {
        let names = |c: Classes| c.into_iter().map(|c| c.to_string()).collect::<Vec<_>>();
        assert_eq!(names(code_snippet(false)), [class::CODE_SNIPPET]);
        assert_eq!(
            names(code_snippet(true)),
            [class::CODE_SNIPPET, class::INLINE]
        );
    }

    #[test]
    fn a_block_is_a_pre_and_inline_a_code() {
        let html = render(CodeSnippetProps {
            testid: Some("snippet".into()),
            ..props("x")
        });
        assert_eq!(
            html,
            format!(
                "<pre class=\"{}\" data-testid=\"snippet\">x</pre>",
                class::CODE_SNIPPET
            )
        );
        let html = render(CodeSnippetProps {
            inline: true,
            ..props("x")
        });
        assert_eq!(
            html,
            format!(
                "<code class=\"{} {}\">x</code>",
                class::CODE_SNIPPET,
                class::INLINE
            )
        );
    }

    #[test]
    fn renders_spans_and_marks() {
        let html = render(CodeSnippetProps {
            spans: Rc::from([CodeSpan::new(0..3, CodeToken::Keyword)]),
            marks: Rc::from([
                Diagnostic::new(4..5, Severity::Error, "unknown name"),
                Diagnostic::new(5..5, Severity::Info, "add `;`"),
            ]),
            ..props("let x")
        });
        assert!(
            html.contains("<span class=\"sockel-code-token-keyword\">let</span> "),
            "{html}"
        );
        assert!(
            html.contains("<span title=\"unknown name\" class=\"sockel-code-snippet-mark sockel-error\">x</span>"),
            "{html}"
        );
        assert!(
            html.contains("<span title=\"add `;`\" class=\"sockel-code-snippet-mark sockel-code-snippet-empty-mark sockel-info\"></span>"),
            "{html}"
        );
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn bad_ranges_do_not_panic() {
        let html = render(CodeSnippetProps {
            marks: Rc::from([
                Diagnostic::new(1..99, Severity::Warning, ""),
                Diagnostic::new(9..1, Severity::Error, "past"),
            ]),
            ..props("é\nb")
        });
        assert!(html.contains("é\nb"), "{html}");
        assert!(html.contains(class::WARNING), "{html}");
        assert!(!html.contains("title=\"\""), "{html}");
    }
}
