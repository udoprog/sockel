use yew::prelude::*;

use crate::class;

use plinth::{Avatar, AvatarSize, Cluster, Image, Ratio};

use crate::controls::{Demo, Select, Text, Toggle, bind};

/// Loads, in a different shape than every box it is shown in.
const PICTURE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 300 200'%3E%3Crect width='300' height='200' fill='%236d3fc0'/%3E%3Ccircle cx='150' cy='100' r='60' fill='%23ece6f7'/%3E%3C/svg%3E";
/// Fails to decode, without the network.
const BROKEN: &str = "data:image/png;base64,AAAA";
/// Never answers: an address nothing is at.
const SLOW: &str = "http://192.0.2.1/never.png";

const SOURCES: &[(&str, Option<&str>)] = &[
    ("Picture (loads)", Some(PICTURE)),
    ("Broken (error)", Some(BROKEN)),
    ("Never answers (loading)", Some(SLOW)),
    ("None (empty)", None),
];

const RATIOS: &[(&str, Ratio)] = &[
    ("Poster", Ratio::Poster),
    ("Screenshot", Ratio::Screenshot),
    ("Banner", Ratio::Banner),
    ("Square", Ratio::Square),
    ("Explicit 4/3", Ratio::Explicit(4, 3)),
];

const WIDTHS: &[(&str, &str)] = &[
    ("Container", ""),
    ("6rem", class::WIDTH_SMALL),
    ("12rem", class::WIDTH_MEDIUM),
];

#[function_component]
pub fn ImagePage() -> Html {
    let source = use_state(|| SOURCES[0].1);
    let ratio = use_state(|| Ratio::Poster);
    let width = use_state(|| class::WIDTH_MEDIUM);
    let alt = use_state(|| Text::Short);
    let lazy = use_state(|| true);

    let controls = html! {
        <>
            <Select<Option<&'static str>> label="Source" options={SOURCES} value={*source} on_change={bind(&source)} />
            <Select<Ratio> label="Ratio" options={RATIOS} value={*ratio} on_change={bind(&ratio)} />
            <Select<&'static str> label="Width" options={WIDTHS} value={*width} on_change={bind(&width)} />
            <Select<Text> label="alt" options={Text::OPTIONS} value={*alt} on_change={bind(&alt)} />
            <Toggle label="lazy" value={*lazy} on_change={bind(&lazy)} />
        </>
    };

    html! {
        <Demo {controls}>
            <Image
                ratio={*ratio}
                src={source.map(AttrValue::from)}
                alt={alt.of("A picture")}
                lazy={*lazy}
                class={classes!((!width.is_empty()).then_some(*width))}
            />
        </Demo>
    }
}

const NAMES: &[(&str, &str)] = &[
    ("Short", "Ada Lovelace"),
    ("One word", "Ada"),
    ("Long", "Augusta Ada King Countess of Lovelace"),
    ("Empty", ""),
];

const SIZES: &[(&str, AvatarSize)] = &[
    ("Medium", AvatarSize::Medium),
    ("Small", AvatarSize::Small),
    ("Large", AvatarSize::Large),
    ("Huge", AvatarSize::Huge),
];

#[function_component]
pub fn AvatarPage() -> Html {
    let source = use_state(|| SOURCES[0].1);
    let name = use_state(|| NAMES[0].1);
    let size = use_state(|| AvatarSize::Medium);

    let controls = html! {
        <>
            <Select<Option<&'static str>> label="Source" options={SOURCES} value={*source} on_change={bind(&source)} />
            <Select<&'static str> label="Name" options={NAMES} value={*name} on_change={bind(&name)} />
            <Select<AvatarSize> label="Size" options={SIZES} value={*size} on_change={bind(&size)} />
        </>
    };

    html! {
        <Demo {controls}>
            <Cluster>
                <Avatar name={*name} src={source.map(AttrValue::from)} size={*size} />
                <Avatar name={*name} src={source.map(AttrValue::from)} size={*size} />
            </Cluster>
        </Demo>
    }
}
