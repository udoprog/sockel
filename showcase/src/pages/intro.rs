use yew::prelude::*;

use sockel::{
    Button, ButtonVariant, Card, CardGrid, Choice, ChoiceOption, Cluster, ColorOption, ColorSelect,
    Count, Field, Gap, Inset, Pill, SectionHead, Stack, Switch, TextInput, Tone,
};

use super::fields::use_control_id;
use crate::class;
use crate::code::CodeCard;
use crate::controls::Toggle;

const DOCS: &str = "https://docs.rs/sockel";
const REPOSITORY: &str = "https://github.com/udoprog/sockel";

/// What sockel is for, in a few cards.
const STRENGTHS: &[(&str, &str)] = &[
    (
        "Nothing jitters",
        "A button going busy, a count gaining digits, an error arriving: each \
         component keeps its box, so the page around it stays where it was.",
    ),
    (
        "Your colours",
        "sockel reads the app's colours as custom properties (background, surface, \
         text, accent and a few more) and brings the spacing, type and size scales.",
    ),
    (
        "Only what you use",
        "Each component is a cargo feature gating its Rust and its CSS together; \
         sockel-build compiles those and the app's own Sass into one stylesheet.",
    ),
];

/// The files of a small app, as the crate docs give them.
const GETTING_STARTED: &str = r#"# Cargo.toml
[dependencies]
sockel = { version = "0.2", default-features = false, features = ["button", "layout"] }
yew = { version = "0.23", features = ["csr"] }

[build-dependencies]
sockel-build = "0.2"

// build.rs: sockel's CSS and the app's style/, as one stylesheet.
fn main() {
    sockel_build::Config::new().local("style/").prefix("app-").compile();
}

// src/main.rs
use yew::prelude::*;

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

#[function_component]
fn App() -> Html {
    let busy = use_state(|| false);
    let onclick = { let busy = busy.clone(); Callback::from(move |_| busy.set(true)) };

    html! {
        <>
            <sockel::Styles css={STYLESHEET} />
            <sockel::Stack>
                <sockel::Button busy={*busy} {onclick}>{ "Save" }</sockel::Button>
            </sockel::Stack>
        </>
    }
}"#;

/// The landing page: what sockel is, two demos to play with, and how to
/// start.
#[function_component]
pub fn IntroPage() -> Html {
    html! {
        <Stack gap={Gap::Loose}>
            <p class={class::PROSE}>
                { "sockel is a set of Yew components that keep their size as their content \
                   and state change. A label swapping for a spinner, a figure growing, an \
                   error showing up: none of it moves what sits beside or below, so a page \
                   never jitters under the reader's pointer." }
            </p>
            <Cluster>
                <Button variant={ButtonVariant::Primary} href={DOCS}>{ "Read the docs" }</Button>
                <Button href={REPOSITORY}>{ "Source on GitHub" }</Button>
            </Cluster>
            <CardGrid>
                { for STRENGTHS.iter().map(|(title, text)| html! {
                    <Card tag="li" inset={Inset::Compact}>
                        <Stack gap={Gap::Snug}>
                            <strong>{ *title }</strong>
                            <span class={class::MUTED}>{ *text }</span>
                        </Stack>
                    </Card>
                }) }
            </CardGrid>
            <Compare />
            <Playground />
            <SectionHead title="Getting started" />
            <p class={class::PROSE}>
                { "Add sockel with the features you render, build the stylesheet in build.rs \
                   and mount it once. Every component has a page here, in the navigation; the \
                   docs on docs.rs cover features, theming and configuration." }
            </p>
            <CodeCard code={GETTING_STARTED} />
        </Stack>
    }
}

/// Toggles one state of the compare demo.
fn flip(state: &UseStateHandle<bool>) -> Callback<bool> {
    let state = state.clone();
    Callback::from(move |on| state.set(on))
}

/// The same three states shown with plain markup and with sockel's
/// components, side by side.
#[function_component]
fn Compare() -> Html {
    let busy = use_state(|| false);
    let many = use_state(|| false);
    let error = use_state(|| false);

    let all = {
        let (busy, many, error) = (busy.clone(), many.clone(), error.clone());
        let next = !(*busy && *many && *error);
        Callback::from(move |_: MouseEvent| {
            busy.set(next);
            many.set(next);
            error.set(next);
        })
    };

    let state = State {
        busy: *busy,
        count: if *many { 1024 } else { 7 },
        error: *error,
    };

    html! {
        <Stack gap={Gap::Loose}>
            <SectionHead title="Without and with sockel" />
            <p class={class::PROSE}>
                { "Flip the states and watch the outlined neighbours. On the left, plain \
                   markup: the button's label grows, the count gains digits and the error \
                   line pushes the next field down. On the right, the same with sockel: \
                   nothing else moves." }
            </p>
            <Card inset={Inset::Compact}>
                <Cluster gap={Gap::Loose}>
                    <Toggle label="Saving" value={*busy} on_change={flip(&busy)} />
                    <Toggle label="1,024 unread" value={*many} on_change={flip(&many)} />
                    <Toggle label="Email error" value={*error} on_change={flip(&error)} />
                    <Button variant={ButtonVariant::Primary} onclick={all}>{ "Flip them all" }</Button>
                </Cluster>
            </Card>
            <div class={class::FRAMES}>
                <section class={classes!(class::FRAME, class::WIDE)} aria-label="Without sockel">
                    <span class={class::FRAME_LABEL}>{ "Without sockel" }</span>
                    <Naive {state} />
                </section>
                <section class={classes!(class::FRAME, class::WIDE)} aria-label="With sockel">
                    <span class={class::FRAME_LABEL}>{ "With sockel" }</span>
                    <Held {state} />
                </section>
            </div>
        </Stack>
    }
}

#[derive(Clone, Copy, PartialEq)]
struct State {
    busy: bool,
    count: u32,
    error: bool,
}

#[derive(Properties, PartialEq)]
struct StateProps {
    state: State,
}

/// A neighbour whose place shows whether anything moved.
fn neighbour(text: &str) -> Html {
    html! { <span class={classes!(class::SAMPLE, class::OUTLINED)}>{ text.to_owned() }</span> }
}

/// The states with plain markup: each changes its box.
#[function_component]
fn Naive(props: &StateProps) -> Html {
    let s = props.state;
    let id = use_control_id();
    let email = use_state(|| AttrValue::Static("ada@"));
    let on_input = {
        let email = email.clone();
        Callback::from(move |v: String| email.set(v.into()))
    };

    html! {
        <Stack>
            <Cluster>
                <button type="button" class={classes!(class::BUTTON, class::PRIMARY)}>
                    { if s.busy { "Saving the draft…" } else { "Save" } }
                </button>
                { neighbour("Cancel") }
            </Cluster>
            <Cluster>
                <span>{ "Inbox" }</span>
                <span class={class::CHIP}>{ format_count(s.count) }</span>
                { neighbour("Archive") }
            </Cluster>
            <Stack gap={Gap::Snug}>
                <label for={id.clone()} class={class::CONTROL_LABEL}>{ "Email" }</label>
                <TextInput id={id} value={(*email).clone()} {on_input} />
                if s.error {
                    <span class={class::CONTROL_LABEL}>{ "Enter a whole address." }</span>
                }
            </Stack>
            { neighbour("The next field") }
        </Stack>
    }
}

/// The states with sockel's components: each keeps its box.
#[function_component]
fn Held(props: &StateProps) -> Html {
    let s = props.state;
    let id = use_control_id();
    let email = use_state(|| AttrValue::Static("ada@"));
    let on_input = {
        let email = email.clone();
        Callback::from(move |v: String| email.set(v.into()))
    };
    let error = s.error.then(|| html! { "Enter a whole address." });

    html! {
        <Stack>
            <Cluster>
                <Button variant={ButtonVariant::Primary} busy={s.busy}>{ "Save" }</Button>
                { neighbour("Cancel") }
            </Cluster>
            <Cluster>
                <span>{ "Inbox" }</span>
                <Count value={s.count} />
                { neighbour("Archive") }
            </Cluster>
            <Field id={id.clone()} label={html! { "Email" }} {error} reserve_error=true>
                <TextInput id={id} value={(*email).clone()} {on_input} />
            </Field>
            { neighbour("The next field") }
        </Stack>
    }
}

/// A count as plain markup writes it, every digit shown.
fn format_count(count: u32) -> String {
    let digits = count.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The colours a theme sets, as sockel reads them.
#[derive(Clone, Copy, PartialEq)]
struct Palette {
    bg: &'static str,
    surface: &'static str,
    text: &'static str,
    muted: &'static str,
    border: &'static str,
    accent: &'static str,
    on_accent: &'static str,
    scheme: &'static str,
}

const PRESETS: &[(&str, Palette)] = &[
    (
        "Paper",
        Palette {
            bg: "#f6f6f8",
            surface: "#ffffff",
            text: "#1b1b22",
            muted: "#62626e",
            border: "#d9d9e0",
            accent: "#6d3fc0",
            on_accent: "#ffffff",
            scheme: "light",
        },
    ),
    (
        "Forest",
        Palette {
            bg: "#e8f4f1",
            surface: "#fbfffe",
            text: "#10302a",
            muted: "#4d6b64",
            border: "#a8cfc6",
            accent: "#0f7a66",
            on_accent: "#ffffff",
            scheme: "light",
        },
    ),
    (
        "Ember",
        Palette {
            bg: "#2a1a12",
            surface: "#38241a",
            text: "#f6e7dd",
            muted: "#c9a994",
            border: "#5c3d2e",
            accent: "#f0955c",
            on_accent: "#2a1a12",
            scheme: "dark",
        },
    ),
    (
        "Night",
        Palette {
            bg: "#0f1420",
            surface: "#182033",
            text: "#e6ecf7",
            muted: "#94a1bb",
            border: "#2b3650",
            accent: "#5aa9ff",
            on_accent: "#0f1420",
            scheme: "dark",
        },
    ),
];

fn accents() -> Vec<ColorOption> {
    vec![
        ColorOption::new("#6d3fc0", "Violet"),
        ColorOption::new("#d92d20", "Red"),
        ColorOption::new("#dc8a00", "Orange"),
        ColorOption::new("#12a150", "Green"),
        ColorOption::new("#0f7a66", "Teal"),
        ColorOption::new("#175cd3", "Blue"),
        ColorOption::new("#d444a0", "Pink"),
        ColorOption::new("#5aa9ff", "Sky"),
        ColorOption::new("#f0955c", "Peach"),
    ]
}

/// A frame of components under colours the visitor picks.
#[function_component]
fn Playground() -> Html {
    let preset = use_state(|| AttrValue::Static("Paper"));
    let accent = use_state(|| None::<AttrValue>);
    let notify = use_state(|| true);
    let period = use_state(|| AttrValue::Static("week"));
    let name = use_state(|| AttrValue::Static("Release notes"));
    let id = use_control_id();

    let palette = PRESETS
        .iter()
        .find(|(name, _)| *name == preset.as_str())
        .map_or(PRESETS[0].1, |(_, p)| *p);
    let chosen = (*accent)
        .clone()
        .unwrap_or(AttrValue::Static(palette.accent));
    let style = format!(
        "--bg: {}; --surface: {}; --text: {}; --muted: {}; \
         --border: {}; --accent: {chosen}; --on-accent: {}; color-scheme: {}",
        palette.bg,
        palette.surface,
        palette.text,
        palette.muted,
        palette.border,
        palette.on_accent,
        palette.scheme,
    );

    let on_preset = {
        let (preset, accent) = (preset.clone(), accent.clone());
        Callback::from(move |v: AttrValue| {
            preset.set(v);
            accent.set(None);
        })
    };
    let on_accent = {
        let accent = accent.clone();
        Callback::from(move |v: AttrValue| accent.set(Some(v)))
    };
    let presets: Vec<ChoiceOption> = PRESETS
        .iter()
        .map(|(name, _)| ChoiceOption::new(*name, *name))
        .collect();
    let periods = vec![
        ChoiceOption::new("day", "Day"),
        ChoiceOption::new("week", "Week"),
        ChoiceOption::new("month", "Month"),
    ];
    let on_name = {
        let name = name.clone();
        Callback::from(move |v: String| name.set(v.into()))
    };
    let name_error = name.trim().is_empty().then(|| html! { "Give it a name." });

    html! {
        <Stack gap={Gap::Loose}>
            <SectionHead title="Bring your own colours" />
            <p class={class::PROSE}>
                { "sockel supplies no colours: it reads the app's. Pick a theme and an accent \
                   and the components below take them at once, set as custom properties on the \
                   frame. The Theme menu in the navigation does the same for the whole showcase, \
                   light or dark." }
            </p>
            <Card inset={Inset::Compact}>
                <Cluster gap={Gap::Loose}>
                    <Choice label="Theme" options={presets} value={Some((*preset).clone())} on_change={on_preset} />
                    <ColorSelect label="Accent" options={accents()} value={Some(chosen.clone())}
                        on_change={on_accent} custom={Some(AttrValue::Static("Custom"))} columns={3} />
                </Cluster>
            </Card>
            <div class={class::SCOPE} {style}>
                <Stack>
                    <Cluster>
                        <Button variant={ButtonVariant::Primary}>{ "Publish" }</Button>
                        <Button>{ "Preview" }</Button>
                        <Pill tone={Tone::Accent}>{ "Draft" }</Pill>
                        <Pill tone={Tone::Success}>{ "Saved" }</Pill>
                        <Count value={3} />
                    </Cluster>
                    <Cluster gap={Gap::Loose}>
                        <Switch on={*notify} on_change={flip(&notify)}>{ "Notify me" }</Switch>
                        <Choice label="Digest" options={periods} value={Some((*period).clone())}
                            on_change={{
                                let period = period.clone();
                                Callback::from(move |v| period.set(v))
                            }} />
                    </Cluster>
                    <Field id={id.clone()} label={html! { "Name" }} error={name_error} reserve_error=true>
                        <TextInput id={id} value={(*name).clone()} on_input={on_name} />
                    </Field>
                </Stack>
            </div>
        </Stack>
    }
}

#[cfg(test)]
mod tests {
    use super::format_count;

    #[test]
    fn counts_are_grouped_by_thousands() {
        assert_eq!(format_count(7), "7");
        assert_eq!(format_count(1024), "1,024");
        assert_eq!(format_count(1_234_567), "1,234,567");
    }
}
