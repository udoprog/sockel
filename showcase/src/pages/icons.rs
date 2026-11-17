use yew::prelude::*;

use crate::class;

use plinth::{Cluster, Glyph, Icon, IconButton, IconSet, Stack, icon};

use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Toggle, bind};

/// A set as an app would supply one: filled shapes, drawn at 48px, to show
/// that the box and not the icon sets the size.
fn sample_set() -> IconSet {
    let shape = |d: &'static str| {
        html! {
            <svg viewBox="0 0 24 24" width="48" height="48" fill="currentColor" aria-hidden="true">
                <path {d} />
            </svg>
        }
    };
    IconSet {
        close: shape("M5 3l7 7 7-7 2 2-7 7 7 7-2 2-7-7-7 7-2-2 7-7-7-7z"),
        chevron_down: shape("M3 7h18l-9 11z"),
        chevron_up: shape("M3 17h18L12 6z"),
        chevron_right: shape("M7 3v18l11-9z"),
        check: shape("M2 13l3-3 4 4 10-10 3 3-13 13z"),
        plus: shape("M10 3h4v7h7v4h-7v7h-4v-7H3v-4h7z"),
        minus: shape("M3 10h18v4H3z"),
        spinner: shape("M12 2a10 10 0 1 0 10 10h-4a6 6 0 1 1-6-6z"),
        grip: shape("M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z"),
        ellipsis: shape("M2 9h6v6H2zM9 9h6v6H9zM16 9h6v6h-6z"),
        search: shape(
            "M10 2a8 8 0 1 0 5 14l6 6 2-2-6-6A8 8 0 0 0 10 2zm0 4a4 4 0 1 1 0 8 4 4 0 0 1 0-8z",
        ),
        eye: shape("M2 12l5-7h10l5 7-5 7H7zm10-4a4 4 0 1 0 0 8 4 4 0 0 0 0-8z"),
        eye_off: shape("M2 12l5-7h10l5 7-5 7H7zM4 20L20 4l1 1L5 21z"),
        copy: shape("M8 3h13v13H8zM3 8h3v13h13v-3H3z"),
    }
}

const SETS: &[(&str, bool)] = &[("Built-in (no provider)", false), ("Sample provider", true)];

const SIZES: &[(&str, &str)] = &[
    ("1em (text size)", ""),
    ("2em box", "font-size: 2em"),
    ("Small text", "font-size: 0.75rem"),
];

const GLYPHS: &[(&str, Glyph)] = &[
    ("Close", Glyph::Close),
    ("ChevronDown", Glyph::ChevronDown),
    ("ChevronUp", Glyph::ChevronUp),
    ("ChevronRight", Glyph::ChevronRight),
    ("Check", Glyph::Check),
    ("Plus", Glyph::Plus),
    ("Minus", Glyph::Minus),
    ("Spinner", Glyph::Spinner),
    ("Grip", Glyph::Grip),
    ("Ellipsis", Glyph::Ellipsis),
    ("Search", Glyph::Search),
    ("Eye", Glyph::Eye),
    ("EyeOff", Glyph::EyeOff),
    ("Copy", Glyph::Copy),
];

fn name(glyph: Glyph) -> String {
    format!("{glyph:?}")
}

#[derive(Properties, PartialEq)]
struct GlyphsProps {
    glyph: Glyph,
    size: &'static str,
    disabled: bool,
}

#[function_component]
fn Glyphs(props: &GlyphsProps) -> Html {
    let style = (!props.size.is_empty()).then_some(props.size);
    html! {
        <Stack>
            <code class={class::CLASS_CALL}>
                { format!("icon(class, {}) → \"{}\"", name(props.glyph), icon(Classes::new(), props.glyph).to_string()) }
            </code>
            <Cluster>
                { for Glyph::ALL.iter().map(|g| html! {
                    <span class={class::OUTLINED} {style} title={name(*g)}><Icon glyph={*g} /></span>
                }) }
            </Cluster>
            <div {style}><Cluster>
                <IconButton class={classes!(class::BUTTON, class::PRIMARY)} disabled={props.disabled}>
                    <Icon glyph={props.glyph} />
                    <span class={class::LABEL_TEXT}>{ name(props.glyph) }</span>
                </IconButton>
                <IconButton class={classes!(class::BUTTON, class::GHOST, plinth::class::ICON)}
                    disabled={props.disabled} title={name(props.glyph)}>
                    <Icon glyph={props.glyph} />
                </IconButton>
            </Cluster></div>
        </Stack>
    }
}

#[function_component]
pub fn IconPage() -> Html {
    let sample = use_state(|| false);
    let glyph = use_state(|| Glyph::Close);
    let size = use_state(|| "");
    let disabled = use_state(|| false);

    let controls = html! {
        <>
            <Select<bool> label="Icon set" options={SETS} value={*sample} on_change={bind(&sample)} />
            <Select<Glyph> label="glyph" options={GLYPHS} value={*glyph} on_change={bind(&glyph)} />
            <Select<&'static str> label="Size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let body = html! { <Glyphs glyph={*glyph} size={*size} disabled={*disabled} /> };
    let set = sample.then(sample_set);

    let button = El::new("IconButton")
        .text("class", "button primary")
        .flag("disabled", *disabled)
        .child(El::new("Icon").expr("glyph", format!("Glyph::{}", name(*glyph))))
        .line(format!("<span>{}</span>", text_node(&name(*glyph))));
    let sized = if size.is_empty() {
        button
    } else {
        El::new("div").text("style", *size).child(button)
    };
    let code = snippet(
        &[],
        if *sample {
            El::new("ContextProvider<IconSet>")
                .expr("context", "icon_set")
                .child(sized)
        } else {
            sized
        },
    );

    html! {
        <Demo {controls} {code}>
            if let Some(set) = set {
                <ContextProvider<IconSet> context={set}>{ body }</ContextProvider<IconSet>>
            } else {
                { body }
            }
        </Demo>
    }
}
