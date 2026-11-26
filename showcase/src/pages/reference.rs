use yew::prelude::*;

use sockel::{Gap, Stack, Table, TableCell, TableRow};

use crate::class;
use crate::code::CodeCard;
use crate::controls::section_head;

const ID: &str = "reference";

const SETUP: &str = r#"// build.rs
fn main() {
    sockel_build::Config::new()
        .local("style/")
        .prefix("app-")
        .with("text-md", "1.0625rem")
        .with("$radius-lg", "12px")
        .compile();
}"#;

pub struct Section {
    pub title: &'static str,
    pub vars: &'static [Var],
}

pub struct Var {
    pub name: &'static str,
    pub default: &'static str,
    pub doc: &'static str,
    /// The custom property `:root` emits it as, if any.
    pub property: Option<&'static str>,
}

pub struct MixinSection {
    pub title: &'static str,
    pub mixins: &'static [Mixin],
}

pub struct Mixin {
    pub name: &'static str,
    /// Its arguments as declared, empty when it takes none.
    pub args: &'static str,
    /// Whether it wraps the block given to its `@include`.
    pub content: bool,
    pub doc: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/reference.rs"));

/// Text with its `code` spans set as code.
pub(super) fn prose(text: &str) -> Html {
    text.split('`')
        .enumerate()
        .map(|(n, part)| {
            if n % 2 == 1 {
                html! { <code>{ part.to_owned() }</code> }
            } else {
                html! { { part.to_owned() } }
            }
        })
        .collect()
}

pub(super) fn head(names: &[&'static str]) -> Html {
    names
        .iter()
        .map(|name| html! { <TableCell head=true>{ *name }</TableCell> })
        .collect()
}

fn section(section: &Section) -> Html {
    let rows: Html = section
        .vars
        .iter()
        .map(|var| {
            let property = match var.property {
                Some(property) => html! { <code>{ property }</code> },
                None => html! { "Sass only" },
            };
            html! {
                <TableRow>
                    <TableCell title=true>
                        <Stack gap={Gap::Hair}>
                            <code>{ format!("${}", var.name) }</code>
                            <span class={class::CONTROL_LABEL}>{ property }</span>
                        </Stack>
                    </TableCell>
                    <TableCell wrap=true>
                        <Stack gap={Gap::Tight}>
                            <span>{ prose(var.doc) }</span>
                            <span class={class::CONTROL_LABEL}>{ "Default: " }<code>{ var.default }</code></span>
                        </Stack>
                    </TableCell>
                </TableRow>
            }
        })
        .collect();

    html! {
        <Stack gap={Gap::Snug}>
            { section_head(ID, section.title) }
            <Table head={head(&["Variable", "What it sets"])}
                columns={2} top=true cards=true framed=true label={section.title}>
                { rows }
            </Table>
        </Stack>
    }
}

/// Every variable sockel's Sass can be configured with, from `style/_tokens.scss`.
#[function_component]
pub fn ReferencePage() -> Html {
    let breakpoints: Html = BREAKPOINTS
        .iter()
        .map(|(name, width)| {
            html! {
                <TableRow>
                    <TableCell title=true><code>{ *name }</code></TableCell>
                    <TableCell><code>{ *width }</code></TableCell>
                </TableRow>
            }
        })
        .collect();

    let parts: Html = PARTS
        .iter()
        .map(|(part, features)| {
            let owner = if features.is_empty() {
                html! { <span class={class::MUTED}>{ "with its component" }</span> }
            } else {
                features
                    .iter()
                    .map(|f| html! { <><code>{ *f }</code>{ " " }</> })
                    .collect()
            };
            html! {
                <TableRow>
                    <TableCell title=true><code>{ *part }</code></TableCell>
                    <TableCell wrap=true>{ owner }</TableCell>
                </TableRow>
            }
        })
        .collect();

    html! {
        <Stack gap={Gap::Loose}>
            <p class={class::PROSE}>
                { "Every value sockel's styles read is a !default variable of style/_tokens.scss. \
                   Set one in build.rs with sockel_build::Config::with(\"text-md\", \"1.0625rem\") \
                   (the $ is optional, the value any Sass expression), or with @use \"pkg:sockel/tokens\" \
                   with (...) under trug. Values change, never classes. A variable emitted as a custom \
                   property can also be overridden at runtime; a null default sets nothing, and its line says what applies \
                   instead. This page is generated from the comments beside each variable." }
            </p>
            <CodeCard code={SETUP} />
            { for SECTIONS.iter().map(section) }
            <Stack gap={Gap::Snug}>
                { section_head(ID, "Breakpoint widths") }
                <p class={class::PROSE}>
                    { "$breakpoints is merged over these, so an app moves or adds one without \
                       restating the rest: $breakpoints: (\"phone\": 42rem)." }
                </p>
                <Table head={head(&["Name", "Width"])} columns={2} framed=true label="Breakpoint widths">
                    { breakpoints }
                </Table>
            </Stack>
            <Stack gap={Gap::Snug}>
                { section_head(ID, "Parts $omit can name") }
                <p class={class::PROSE}>
                    { "A part owned by features is left out on its own while none of them is on; \
                       naming it in $omit then is an error." }
                </p>
                <Table head={head(&["Part", "Built with the feature"])} columns={2} cards=true framed=true label="Parts">
                    { parts }
                </Table>
            </Stack>
        </Stack>
    }
}
