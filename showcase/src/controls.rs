//! The controls a page sets its component's props with, and the frames it
//! shows the component in.

use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use plinth::{Card, Cluster, Gap, Stack};

#[derive(Properties, PartialEq)]
pub struct SelectProps<T: PartialEq + 'static> {
    pub label: AttrValue,
    pub options: &'static [(&'static str, T)],
    pub value: T,
    pub on_change: Callback<T>,
}

/// One of a fixed set of values, by name.
#[function_component]
pub fn Select<T>(props: &SelectProps<T>) -> Html
where
    T: PartialEq + Copy + 'static,
{
    let options = props.options;
    let onchange = props.on_change.reform(move |e: Event| {
        let index = e
            .target_unchecked_into::<HtmlSelectElement>()
            .selected_index();
        options[usize::try_from(index).unwrap_or_default()].1
    });

    html! {
        <Stack tag="label" gap={Gap::Snug}>
            <span class="control-label">{ props.label.clone() }</span>
            <select class="input small" {onchange}>
                { for options.iter().map(|(name, value)| html! {
                    <option selected={*value == props.value}>{ *name }</option>
                }) }
            </select>
        </Stack>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToggleProps {
    pub label: AttrValue,
    pub value: bool,
    pub on_change: Callback<bool>,
}

#[function_component]
pub fn Toggle(props: &ToggleProps) -> Html {
    let onchange = props
        .on_change
        .reform(|e: Event| e.target_unchecked_into::<HtmlInputElement>().checked());

    html! {
        <Cluster tag="label" gap={Gap::Snug}>
            <input type="checkbox" checked={props.value} {onchange} />
            <span class="control-label">{ props.label.clone() }</span>
        </Cluster>
    }
}

/// A state hook and the callback that sets it, for a control.
pub fn bind<T: Clone + 'static>(state: &UseStateHandle<T>) -> Callback<T> {
    let state = state.clone();
    Callback::from(move |value| state.set(value))
}

/// The length of a text a page shows.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Text {
    Empty,
    Short,
    Long,
    Unbroken,
}

impl Text {
    pub const OPTIONS: &[(&str, Text)] = &[
        ("Short", Text::Short),
        ("Long", Text::Long),
        ("Long, unbroken", Text::Unbroken),
        ("Empty", Text::Empty),
    ];

    /// `short` itself, or that text grown long enough to wrap.
    pub fn of(self, short: &str) -> String {
        match self {
            Text::Empty => String::new(),
            Text::Short => short.to_owned(),
            Text::Long => format!(
                "{short} with a much longer text that wraps onto further lines in a narrow container"
            ),
            Text::Unbroken => format!("{short}-{}", "without-a-single-space".repeat(3)),
        }
    }
}

/// How many items a page shows: none for the empty state.
pub const COUNTS: &[(&str, usize)] = &[("3", 3), ("0 (empty)", 0), ("1", 1), ("8", 8), ("20", 20)];

#[derive(Properties, PartialEq)]
pub struct DemoProps {
    /// The controls.
    pub controls: Html,
    /// The component, as the controls set it.
    pub children: Html,
}

/// A page's controls over its component, shown twice: in a narrow frame
/// (which can be dragged wider) and in a wide one, side by side.
#[function_component]
pub fn Demo(props: &DemoProps) -> Html {
    html! {
        <Stack gap={Gap::Loose}>
            <Card inset={plinth::Inset::Compact}>
                <Cluster gap={Gap::Loose} align={plinth::Align::Bottom}>
                    { props.controls.clone() }
                </Cluster>
            </Card>
            <div class="frames">
                <section class="frame narrow" aria-label="Narrow">
                    <span class="frame-label">{ "Narrow (drag the corner)" }</span>
                    { props.children.clone() }
                </section>
                <section class="frame wide" aria-label="Wide">
                    <span class="frame-label">{ "Wide" }</span>
                    { props.children.clone() }
                </section>
            </div>
        </Stack>
    }
}

/// `count` placeholder items, every other one taller when `mixed`; as
/// paragraphs, they keep their default margins.
pub fn samples(count: usize, text: Text, mixed: bool, paragraphs: bool) -> Html {
    let tag = if paragraphs { "p" } else { "div" };

    (1..=count)
        .map(|n| {
            let tall = mixed && n % 2 == 0;
            html! {
                <@{tag} class={classes!("sample", tall.then_some("tall"))}>
                    { text.of(&format!("Item {n}")) }
                </@>
            }
        })
        .collect()
}

pub fn button(label: String, primary: bool, small: bool) -> Html {
    let class = classes!(
        "button",
        primary.then_some("primary"),
        small.then_some("small")
    );

    html! {
        <button type="button" {class}>
            <span aria-hidden="true">{ "+" }</span>
            <span class="label-text">{ label }</span>
        </button>
    }
}
