//! What CodeEditor and CodeSnippet share: highlighted spans, diagnostics
//! and the cutting of a text into pieces at them. Offsets are UTF-8 byte
//! offsets; one past the end clamps and one inside a character floors.

use std::ops::Range;

use yew::prelude::*;

use crate::class;

/// What a highlighted span of code is, which picks its colour.
///
/// The `--code-*` custom properties (`--code-keyword`, `--code-name`,
/// `--code-member`, `--code-type`, `--code-function`, `--code-number`,
/// `--code-string`, `--code-comment`, `--code-operator` and
/// `--code-punctuation`) are the supported theming surface. Their defaults
/// are mixed from the app's `--text`, so they follow a light or dark theme
/// and an app with no syntax palette sets nothing. An app with its own
/// palette points them at it, on the same element as its colours:
///
/// ```css
/// :root { --code-keyword: var(--syntax-keyword); --code-string: var(--syntax-string); }
/// ```
///
/// An app maps its tokenizer's classes onto these with a `match`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeToken {
    Keyword,
    Name,
    Member,
    Type,
    Function,
    Number,
    String,
    Comment,
    Operator,
    Punctuation,
    Error,
    /// A class of the app's own; the app supplies that class's CSS.
    Class(&'static str),
}

impl CodeToken {
    /// The class of a span of this kind, for an app rendering code of its
    /// own.
    pub fn class(self) -> &'static str {
        match self {
            CodeToken::Keyword => class::CODE_TOKEN_KEYWORD,
            CodeToken::Name => class::CODE_TOKEN_NAME,
            CodeToken::Member => class::CODE_TOKEN_MEMBER,
            CodeToken::Type => class::CODE_TOKEN_TYPE,
            CodeToken::Function => class::CODE_TOKEN_FUNCTION,
            CodeToken::Number => class::CODE_TOKEN_NUMBER,
            CodeToken::String => class::CODE_TOKEN_STRING,
            CodeToken::Comment => class::CODE_TOKEN_COMMENT,
            CodeToken::Operator => class::CODE_TOKEN_OPERATOR,
            CodeToken::Punctuation => class::CODE_TOKEN_PUNCTUATION,
            CodeToken::Error => class::CODE_TOKEN_ERROR,
            CodeToken::Class(class) => class,
        }
    }
}

/// A highlighted span of the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeSpan {
    /// UTF-8 byte offsets into the text; past the end clamps, off a char
    /// boundary floors.
    pub range: Range<usize>,
    pub token: CodeToken,
}

impl CodeSpan {
    pub fn new(range: Range<usize>, token: CodeToken) -> Self {
        Self { range, token }
    }
}

/// How bad a diagnostic is, least first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    #[default]
    Error,
}

impl Severity {
    pub(crate) fn class(self) -> &'static str {
        match self {
            Severity::Info => class::INFO,
            Severity::Warning => class::WARNING,
            Severity::Error => class::ERROR,
        }
    }
}

/// A message about a span of the text, underlined there and marked in the
/// gutter. An empty span marks the place after it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// UTF-8 byte offsets into the text; past the end clamps, off a char
    /// boundary floors.
    pub range: Range<usize>,
    pub severity: Severity,
    pub message: AttrValue,
}

impl Diagnostic {
    pub fn new(range: Range<usize>, severity: Severity, message: impl Into<AttrValue>) -> Self {
        Self {
            range,
            severity,
            message: message.into(),
        }
    }
}

/// The byte offset of character `chars` in `text`, for an analyser that
/// counts characters; `text.len()` when `chars` is past the end.
pub fn char_to_byte(text: &str, chars: usize) -> usize {
    text.char_indices()
        .nth(chars)
        .map_or(text.len(), |(index, _)| index)
}

/// One stretch of the highlighted code, or a point in it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Piece {
    Text {
        range: Range<usize>,
        /// The index of the span it is in.
        span: Option<usize>,
        /// The worst severity of the marks over it.
        mark: Option<Severity>,
    },
    /// The index of a point asked for.
    Point(usize),
    /// An empty mark, with the worst severity of those at its place.
    EmptyMark(Severity),
}

/// `source` cut into pieces at the spans, the marks and the points, which
/// are placed before the text at them.
pub(crate) fn pieces(
    source: &str,
    spans: &[CodeSpan],
    marks: &[(Range<usize>, Severity)],
    points: &[Option<usize>],
) -> Vec<Piece> {
    let len = source.len();
    let floor = |at: usize| floor(source, at.min(len));
    let marks = marks
        .iter()
        .map(|(r, s)| {
            let start = floor(r.start);
            (start..floor(r.end).max(start), *s)
        })
        .collect::<Vec<_>>();

    let mut cuts = vec![0, len];
    cuts.extend(
        spans
            .iter()
            .flat_map(|s| [floor(s.range.start), floor(s.range.end)]),
    );
    cuts.extend(marks.iter().flat_map(|(r, _)| [r.start, r.end]));
    cuts.extend(points.iter().flatten().map(|&p| floor(p)));
    cuts.sort_unstable();
    cuts.dedup();

    let mut out = Vec::new();
    let mut spans = spans.iter().enumerate().peekable();

    for (index, &at) in cuts.iter().enumerate() {
        for (point, p) in points.iter().enumerate() {
            if p.map(floor) == Some(at) {
                out.push(Piece::Point(point));
            }
        }

        let empty = marks
            .iter()
            .filter(|(r, _)| r.is_empty() && r.start == at)
            .map(|(_, s)| *s)
            .max();

        if let Some(severity) = empty {
            out.push(Piece::EmptyMark(severity));
        }

        let Some(&end) = cuts.get(index.saturating_add(1)) else {
            break;
        };

        while spans.next_if(|(_, s)| floor(s.range.end) <= at).is_some() {}

        let span = spans
            .peek()
            .filter(|(_, s)| floor(s.range.start) <= at)
            .map(|(i, _)| *i);
        let mark = marks
            .iter()
            .filter(|(r, _)| r.start <= at && end <= r.end && !r.is_empty())
            .map(|(_, s)| *s)
            .max();

        out.push(Piece::Text {
            range: at..end,
            span,
            mark,
        });
    }

    out
}

pub(crate) fn floor(text: &str, mut at: usize) -> usize {
    while at > 0 && !text.is_char_boundary(at) {
        at = at.saturating_sub(1);
    }

    at
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(range: Range<usize>, token: CodeToken) -> CodeSpan {
        CodeSpan::new(range, token)
    }

    fn text(range: Range<usize>, span: Option<usize>, mark: Option<Severity>) -> Piece {
        Piece::Text { range, span, mark }
    }

    #[test]
    fn char_to_byte_clamps() {
        assert_eq!(char_to_byte("aéb", 0), 0);
        assert_eq!(char_to_byte("aéb", 2), 3);
        assert_eq!(char_to_byte("aéb", 3), 4);
        assert_eq!(char_to_byte("aéb", 99), 4);
        assert_eq!(char_to_byte("", 1), 0);
    }

    #[test]
    fn pieces_floor_off_boundary_and_clamp() {
        let spans = [span(0..99, CodeToken::Name)];
        assert_eq!(
            pieces("é", &spans, &[], &[Some(1)]),
            [Piece::Point(0), text(0..2, Some(0), None)]
        );
    }

    #[test]
    fn pieces_follow_spans() {
        let spans = [span(0..1, CodeToken::Name), span(2..3, CodeToken::Number)];
        assert_eq!(
            pieces("x y", &spans, &[], &[]),
            [
                text(0..1, Some(0), None),
                text(1..2, None, None),
                text(2..3, Some(1), None),
            ]
        );
    }

    #[test]
    fn pieces_cut_at_marks_and_points() {
        let spans = [span(0..4, CodeToken::Name)];
        assert_eq!(
            pieces("abcd", &spans, &[(1..3, Severity::Error)], &[Some(2), None]),
            [
                text(0..1, Some(0), None),
                text(1..2, Some(0), Some(Severity::Error)),
                Piece::Point(0),
                text(2..3, Some(0), Some(Severity::Error)),
                text(3..4, Some(0), None),
            ]
        );
    }

    #[test]
    fn overlapping_marks_take_the_worst() {
        let marks = [(0..3, Severity::Info), (1..2, Severity::Warning)];
        assert_eq!(
            pieces("abc", &[], &marks, &[]),
            [
                text(0..1, None, Some(Severity::Info)),
                text(1..2, None, Some(Severity::Warning)),
                text(2..3, None, Some(Severity::Info)),
            ]
        );
    }

    #[test]
    fn pieces_place_points_and_empty_marks_at_the_end() {
        assert_eq!(
            pieces(
                "ab",
                &[],
                &[(2..2, Severity::Warning), (2..2, Severity::Error)],
                &[Some(2), Some(9)]
            ),
            [
                text(0..2, None, None),
                Piece::Point(0),
                Piece::Point(1),
                Piece::EmptyMark(Severity::Error),
            ]
        );
    }

    #[test]
    fn pieces_of_nothing() {
        assert_eq!(pieces("", &[], &[], &[Some(0)]), [Piece::Point(0)]);
    }

    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn pieces_keep_to_char_boundaries() {
        let source = "é+1";
        let got = pieces(
            source,
            &[],
            &[(1..2, Severity::Error), (9..1, Severity::Info)],
            &[Some(1)],
        );
        assert!(got.iter().all(|p| match p {
            Piece::Text { range, .. } => source.get(range.clone()).is_some(),
            _ => true,
        }));
    }
}
