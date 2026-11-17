use yew::prelude::*;

use crate::class;

use sockel::{Cluster, Glyph, Icon, IconButton, IconSet, Stack, icon};

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
    IconSet::default()
        .with(
            Glyph::Close,
            shape("M5 3l7 7 7-7 2 2-7 7 7 7-2 2-7-7-7 7-2-2 7-7-7-7z"),
        )
        .with(Glyph::ChevronDown, shape("M3 7h18l-9 11z"))
        .with(Glyph::ChevronUp, shape("M3 17h18L12 6z"))
        .with(Glyph::ChevronRight, shape("M7 3v18l11-9z"))
        .with(Glyph::Check, shape("M2 13l3-3 4 4 10-10 3 3-13 13z"))
        .with(Glyph::Plus, shape("M10 3h4v7h7v4h-7v7h-4v-7H3v-4h7z"))
        .with(Glyph::Minus, shape("M3 10h18v4H3z"))
        .with(
            Glyph::Spinner,
            shape("M12 2a10 10 0 1 0 10 10h-4a6 6 0 1 1-6-6z"),
        )
        .with(
            Glyph::Grip,
            shape("M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z"),
        )
        .with(
            Glyph::Ellipsis,
            shape("M2 9h6v6H2zM9 9h6v6H9zM16 9h6v6h-6z"),
        )
        .with(
            Glyph::Search,
            shape(
                "M10 2a8 8 0 1 0 5 14l6 6 2-2-6-6A8 8 0 0 0 10 2zm0 4a4 4 0 1 1 0 8 4 4 0 0 1 0-8z",
            ),
        )
        .with(
            Glyph::Eye,
            shape("M2 12l5-7h10l5 7-5 7H7zm10-4a4 4 0 1 0 0 8 4 4 0 0 0 0-8z"),
        )
        .with(
            Glyph::EyeOff,
            shape("M2 12l5-7h10l5 7-5 7H7zM4 20L20 4l1 1L5 21z"),
        )
        .with(Glyph::Copy, shape("M8 3h13v13H8zM3 8h3v13h13v-3H3z"))
        .with_named("trash", shape("M6 3h12v2h4v2H2V5h4zM5 9h14l-1 12H6z"))
        .with_named(
            "star",
            shape("M12 2l3 7 7 .7-5.3 4.8L18 22l-6-3.6L6 22l1.3-7.5L2 9.7 9 9z"),
        )
}

const SETS: &[(&str, bool)] = &[("Built-in (no provider)", false), ("Sample provider", true)];

const SIZES: &[(&str, &str)] = &[
    ("1em (text size)", ""),
    ("2em box", "font-size: 2em"),
    ("Small text", "font-size: 0.75rem"),
];

#[derive(Clone, Copy, PartialEq)]
enum Pick {
    Glyph(Glyph),
    Named(&'static str),
}

const PICKS: &[(&str, Pick)] = &[
    ("Close", Pick::Glyph(Glyph::Close)),
    ("ChevronDown", Pick::Glyph(Glyph::ChevronDown)),
    ("ChevronUp", Pick::Glyph(Glyph::ChevronUp)),
    ("ChevronRight", Pick::Glyph(Glyph::ChevronRight)),
    ("Check", Pick::Glyph(Glyph::Check)),
    ("Plus", Pick::Glyph(Glyph::Plus)),
    ("Minus", Pick::Glyph(Glyph::Minus)),
    ("Spinner", Pick::Glyph(Glyph::Spinner)),
    ("Grip", Pick::Glyph(Glyph::Grip)),
    ("Ellipsis", Pick::Glyph(Glyph::Ellipsis)),
    ("Search", Pick::Glyph(Glyph::Search)),
    ("Eye", Pick::Glyph(Glyph::Eye)),
    ("EyeOff", Pick::Glyph(Glyph::EyeOff)),
    ("Copy", Pick::Glyph(Glyph::Copy)),
    ("\"trash\" (app icon)", Pick::Named("trash")),
    ("\"star\" (app icon)", Pick::Named("star")),
    ("\"ghost\" (not in any set)", Pick::Named("ghost")),
];

impl Pick {
    fn label(self) -> String {
        match self {
            Pick::Glyph(g) => name(g),
            Pick::Named(n) => n.to_owned(),
        }
    }

    fn icon(self) -> Html {
        match self {
            Pick::Glyph(glyph) => html! { <Icon {glyph} /> },
            Pick::Named(n) => html! { <Icon name={n} /> },
        }
    }

    fn el(self) -> El {
        match self {
            Pick::Glyph(g) => El::new("Icon").expr("glyph", format!("Glyph::{}", name(g))),
            Pick::Named(n) => El::new("Icon").text("name", n),
        }
    }
}

fn name(glyph: Glyph) -> String {
    format!("{glyph:?}")
}

#[derive(Properties, PartialEq)]
struct GlyphsProps {
    pick: Pick,
    size: &'static str,
    disabled: bool,
}

#[function_component]
fn Glyphs(props: &GlyphsProps) -> Html {
    let style = (!props.size.is_empty()).then_some(props.size);
    html! {
        <Stack>
            <code class={class::CLASS_CALL}>
                { format!("icon(class, {}) → \"{}\"", name(Glyph::Close), icon(Classes::new(), Glyph::Close).to_string()) }
            </code>
            <Cluster>
                { for Glyph::ALL.iter().map(|g| html! {
                    <span class={class::OUTLINED} {style} title={name(*g)}><Icon glyph={*g} /></span>
                }) }
            </Cluster>
            <div {style}><Cluster>
                <IconButton class={classes!(class::BUTTON, class::PRIMARY)} disabled={props.disabled}>
                    { props.pick.icon() }
                    <span class={class::LABEL_TEXT}>{ props.pick.label() }</span>
                </IconButton>
                <IconButton class={classes!(class::BUTTON, class::GHOST, sockel::class::ICON)}
                    disabled={props.disabled} title={props.pick.label()}>
                    { props.pick.icon() }
                </IconButton>
            </Cluster></div>
        </Stack>
    }
}

#[function_component]
pub fn IconPage() -> Html {
    let sample = use_state(|| false);
    let pick = use_state(|| Pick::Glyph(Glyph::Close));
    let size = use_state(|| "");
    let disabled = use_state(|| false);

    let controls = html! {
        <>
            <Select<bool> label="Icon set" options={SETS} value={*sample} on_change={bind(&sample)} />
            <Select<Pick> label="icon" options={PICKS} value={*pick} on_change={bind(&pick)} />
            <Select<&'static str> label="Size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let body = html! { <Glyphs pick={*pick} size={*size} disabled={*disabled} /> };
    let set = sample.then(sample_set);

    let button = El::new("IconButton")
        .text("class", "button primary")
        .flag("disabled", *disabled)
        .child(pick.el())
        .line(format!("<span>{}</span>", text_node(&pick.label())));
    let sized = if size.is_empty() {
        button
    } else {
        El::new("div").text("style", *size).child(button)
    };
    let lets = [
        "let icon_set = IconSet::default()".to_owned(),
        "    .with(Glyph::Close, html! { <MyIcon name=\"x\" /> })".to_owned(),
        "    .with_named(\"trash\", html! { <MyIcon name=\"bin\" /> });".to_owned(),
    ];
    let code = snippet(
        if *sample { &lets } else { &[] },
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
