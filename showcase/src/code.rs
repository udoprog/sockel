//! The code a page's component is rendered with, as an adopter writes it:
//! built from the page's control state, highlighted, and shown under the
//! frames with a Copy button.

use std::fmt::{Debug, Write};

use wasm_bindgen::{JsCast, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;

use crate::class;

use plinth::{Card, Cluster, Gap, Inset, Justify, Stack};

/// One element of a snippet: a component or tag, its props, its children.
pub struct El {
    name: String,
    props: Vec<(&'static str, String)>,
    children: Vec<Child>,
}

enum Child {
    Line(String),
    El(El),
}

impl El {
    /// `name` is a component or a tag; an empty one is a fragment.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            props: Vec::new(),
            children: Vec::new(),
        }
    }

    /// `name="value"`.
    pub fn text(mut self, name: &'static str, value: &str) -> Self {
        let quoted = format!("{value:?}");
        self.props.push((name, quoted));
        self
    }

    pub fn text_if(self, on: bool, name: &'static str, value: &str) -> Self {
        if on { self.text(name, value) } else { self }
    }

    /// `name={expr}`.
    pub fn expr(mut self, name: &'static str, expr: impl Into<String>) -> Self {
        self.props.push((name, format!("{{{}}}", expr.into())));
        self
    }

    pub fn expr_if(self, on: bool, name: &'static str, expr: impl Into<String>) -> Self {
        if on { self.expr(name, expr) } else { self }
    }

    /// `name=true`, when on.
    pub fn flag(mut self, name: &'static str, on: bool) -> Self {
        if on {
            self.props.push((name, "true".to_owned()));
        }
        self
    }

    /// `name={Type::Variant}`, unless the value is its type's default.
    pub fn variant<T: Debug + Default + PartialEq>(self, name: &'static str, value: T) -> Self {
        let ty = std::any::type_name::<T>().rsplit("::").next().unwrap_or("");
        self.expr_if(value != T::default(), name, format!("{ty}::{value:?}"))
    }

    pub fn line(mut self, line: impl Into<String>) -> Self {
        self.children.push(Child::Line(line.into()));
        self
    }

    pub fn line_if(self, on: bool, line: impl Into<String>) -> Self {
        if on { self.line(line) } else { self }
    }

    pub fn child(mut self, el: El) -> Self {
        self.children.push(Child::El(el));
        self
    }

    /// One prop per line once there are more than two; self-closing
    /// without children.
    fn write(&self, out: &mut String, depth: usize) {
        let pad = "    ".repeat(depth);
        let props = |sep: &str| -> String {
            self.props
                .iter()
                .map(|(name, value)| format!("{sep}{name}={value}"))
                .collect()
        };
        let multiline = self.props.len() > 2;
        let open = if multiline {
            format!("{}\n{pad}", props(&format!("\n{pad}    ")))
        } else {
            props(" ")
        };

        if self.children.is_empty() {
            let space = if multiline { "" } else { " " };
            let _ = writeln!(out, "{pad}<{}{open}{space}/>", self.name);
            return;
        }

        let _ = writeln!(out, "{pad}<{}{open}>", self.name);
        for child in &self.children {
            match child {
                Child::Line(line) => {
                    let _ = writeln!(out, "{pad}    {line}");
                }
                Child::El(el) => el.write(out, depth + 1),
            }
        }
        let _ = writeln!(out, "{pad}</{}>", self.name);
    }
}

/// A snippet: any `let`s, then `html! { root }`.
pub fn snippet(lets: &[String], root: El) -> AttrValue {
    let mut out = String::new();
    for line in lets {
        let _ = writeln!(out, "{line}");
    }
    if !lets.is_empty() {
        out.push('\n');
    }
    out.push_str("html! {\n");
    root.write(&mut out, 1);
    out.push('}');
    out.into()
}

/// `{ for name.iter().map(each) } // N things`, the line for a list of
/// children.
pub fn each(list: &str, each: &str, count: usize, thing: &str) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!("{{ for {list}.iter().map({each}) }} // {count} {thing}{plural}")
}

/// `{ "text" }`.
pub fn text_node(text: &str) -> String {
    format!("{{ {text:?} }}")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Str,
    Comment,
    Keyword,
    Tag,
    Attr,
    Type,
    Number,
    Punct,
}

impl Kind {
    fn class(self) -> &'static str {
        match self {
            Kind::Str => class::CODE_STRING,
            Kind::Comment => class::CODE_COMMENT,
            Kind::Keyword => class::CODE_KEYWORD,
            Kind::Tag => class::CODE_TAG,
            Kind::Attr => class::CODE_ATTR,
            Kind::Type => class::CODE_TYPE,
            Kind::Number => class::CODE_NUMBER,
            Kind::Punct => class::CODE_PUNCT,
        }
    }
}

const KEYWORDS: &[&str] = &[
    "let", "move", "for", "in", "if", "else", "fn", "use", "true", "false", "Some", "None",
];

const PUNCT: &[char] = &[
    '<', '>', '{', '}', '/', '=', '(', ')', '|', ',', '.', ';', '!', '&', '[', ']', ':',
];

/// Rust and `html!` source cut into tokens: each piece of `src` in order,
/// with its kind (`None` for whitespace and plain names).
pub fn tokens(src: &str) -> Vec<(Option<Kind>, &str)> {
    let mut out = Vec::new();
    let mut plain = 0;
    let mut i = 0;
    let word = |c: char| c.is_alphanumeric() || c == '_';

    while i < src.len() {
        let rest = &src[i..];
        let c = rest.chars().next().unwrap_or_default();

        let found = if c == '"' {
            let mut end = 1;
            let mut escaped = false;
            for (at, ch) in rest.char_indices().skip(1) {
                end = at + ch.len_utf8();
                match ch {
                    '\\' if !escaped => escaped = true,
                    '"' if !escaped => break,
                    _ => escaped = false,
                }
            }
            Some((Kind::Str, end))
        } else if rest.starts_with("//") {
            Some((Kind::Comment, rest.find('\n').unwrap_or(rest.len())))
        } else if c.is_ascii_digit() && !src[..i].ends_with(word) {
            Some((
                Kind::Number,
                rest.find(|c: char| !word(c)).unwrap_or(rest.len()),
            ))
        } else if word(c) && !src[..i].ends_with(word) {
            let len = rest.find(|c: char| !word(c)).unwrap_or(rest.len());
            let name = &rest[..len];
            let after = &rest[len..];
            if name == "html" && after.starts_with('!') {
                Some((Kind::Keyword, len + 1))
            } else if KEYWORDS.contains(&name) {
                Some((Kind::Keyword, len))
            } else if src[..i].ends_with('<') || src[..i].ends_with("</") {
                Some((Kind::Tag, len))
            } else if after.starts_with('=') && !after.starts_with("==") {
                Some((Kind::Attr, len))
            } else if c.is_uppercase() {
                Some((Kind::Type, len))
            } else {
                i += len;
                continue;
            }
        } else if PUNCT.contains(&c) {
            Some((
                Kind::Punct,
                rest.find(|c| !PUNCT.contains(&c)).unwrap_or(rest.len()),
            ))
        } else {
            None
        };

        match found {
            Some((kind, len)) => {
                if plain < i {
                    out.push((None, &src[plain..i]));
                }
                // A tag name follows `<` or `</`: end the punctuation there.
                let len = if kind == Kind::Punct {
                    rest[..len]
                        .find('<')
                        .map(|at| at + 1 + usize::from(rest[at + 1..len].starts_with('/')))
                        .unwrap_or(len)
                } else {
                    len
                };
                out.push((Some(kind), &rest[..len]));
                i += len;
                plain = i;
            }
            None => i += c.len_utf8(),
        }
    }

    if plain < src.len() {
        out.push((None, &src[plain..]));
    }
    out
}

pub fn highlight(src: &str) -> Html {
    tokens(src)
        .into_iter()
        .map(|(kind, text)| match kind {
            Some(kind) => html! { <span class={kind.class()}>{ text.to_owned() }</span> },
            None => html! { { text.to_owned() } },
        })
        .collect()
}

fn copy(text: &str, copied: UseStateHandle<bool>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let written = window.navigator().clipboard().write_text(text);

    wasm_bindgen_futures::spawn_local(async move {
        if JsFuture::from(written).await.is_err() {
            return;
        }
        copied.set(true);
        let reset = Closure::once_into_js(move || copied.set(false));
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(reset.unchecked_ref(), 1500);
    });
}

#[derive(Properties, PartialEq)]
pub struct CodeCardProps {
    pub code: AttrValue,
}

/// The snippet, highlighted, in a card with a Copy button that keeps its
/// width as its label changes; long lines scroll inside it.
#[function_component]
pub fn CodeCard(props: &CodeCardProps) -> Html {
    let copied = use_state(|| false);
    let onclick = {
        let code = props.code.clone();
        let copied = copied.clone();
        Callback::from(move |_: MouseEvent| copy(&code, copied.clone()))
    };
    let label = |text: &'static str, on: bool| {
        html! { <span class={classes!(class::SWAP_ITEM, (!on).then_some(class::OFF))}>{ text }</span> }
    };

    html! {
        <Card inset={Inset::Compact}>
            <Stack gap={Gap::Snug}>
                <Cluster justify={Justify::Between}>
                    <span class={class::CONTROL_LABEL}>{ "Code" }</span>
                    <button type="button" class={classes!(class::BUTTON, class::SMALL)} {onclick}>
                        <span class={class::SWAP} aria-live="polite">
                            { label("Copy", !*copied) }
                            { label("Copied", *copied) }
                        </span>
                    </button>
                </Cluster>
                <pre class={classes!(class::CODE, plinth::class::SCROLL_AREA)}>
                    <code>{ highlight(&props.code) }</code>
                </pre>
            </Stack>
        </Card>
    }
}

#[cfg(test)]
mod tests {
    use super::{El, Kind, each, snippet, tokens};

    use plinth::Gap;

    fn kinds(src: &str) -> Vec<(Option<Kind>, &str)> {
        tokens(src)
            .into_iter()
            .filter(|(_, t)| !t.trim().is_empty())
            .collect()
    }

    #[test]
    fn each_token_kind_gets_its_class() {
        let src = "html! { <Stack gap={Gap::Loose} flush=true label=\"a \\\"b\\\"\" n={3}>{ x } // two\n</Stack> }";
        assert_eq!(
            kinds(src),
            vec![
                (Some(Kind::Keyword), "html!"),
                (Some(Kind::Punct), "{"),
                (Some(Kind::Punct), "<"),
                (Some(Kind::Tag), "Stack"),
                (Some(Kind::Attr), "gap"),
                (Some(Kind::Punct), "={"),
                (Some(Kind::Type), "Gap"),
                (Some(Kind::Punct), "::"),
                (Some(Kind::Type), "Loose"),
                (Some(Kind::Punct), "}"),
                (Some(Kind::Attr), "flush"),
                (Some(Kind::Punct), "="),
                (Some(Kind::Keyword), "true"),
                (Some(Kind::Attr), "label"),
                (Some(Kind::Punct), "="),
                (Some(Kind::Str), "\"a \\\"b\\\"\""),
                (Some(Kind::Attr), "n"),
                (Some(Kind::Punct), "={"),
                (Some(Kind::Number), "3"),
                (Some(Kind::Punct), "}>{"),
                (None, " x "),
                (Some(Kind::Punct), "}"),
                (Some(Kind::Comment), "// two"),
                (Some(Kind::Punct), "</"),
                (Some(Kind::Tag), "Stack"),
                (Some(Kind::Punct), ">"),
                (Some(Kind::Punct), "}"),
            ]
            .into_iter()
            .filter(|(_, t)| !t.trim().is_empty())
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn tokens_round_trip_the_text() {
        let src = "let a = b(\"<&>\", 'c'); // é\nhtml! { <a href=\"#\">{ \"x\" }</a> }";
        let joined: String = tokens(src).into_iter().map(|(_, t)| t).collect();
        assert_eq!(joined, src);
    }

    #[test]
    fn defaults_are_left_out_and_no_children_self_closes() {
        let code = snippet(
            &[],
            El::new("Stack")
                .variant("gap", Gap::Normal)
                .flag("flush", false),
        );
        assert_eq!(code, "html! {\n    <Stack />\n}");

        let code = snippet(&[], El::new("Stack").variant("gap", Gap::Loose));
        assert_eq!(code, "html! {\n    <Stack gap={Gap::Loose} />\n}");
    }

    #[test]
    fn more_than_two_props_go_one_per_line() {
        let code = snippet(
            &[],
            El::new("Cluster")
                .variant("gap", Gap::Wide)
                .flag("inline", true)
                .text("tag", "ul")
                .line(each("items", "item", 1, "item")),
        );
        assert_eq!(
            code,
            "html! {\n    <Cluster\n        gap={Gap::Wide}\n        inline=true\n        tag=\"ul\"\n    >\n        { for items.iter().map(item) } // 1 item\n    </Cluster>\n}"
        );

        let code = snippet(
            &[],
            El::new("Toolbar")
                .text("label", "a")
                .flag("boxed", true)
                .flag("small", true),
        );
        assert_eq!(
            code,
            "html! {\n    <Toolbar\n        label=\"a\"\n        boxed=true\n        small=true\n    />\n}"
        );
    }
}
