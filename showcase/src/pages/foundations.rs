use yew::prelude::*;

use crate::class;

use sockel::{Card, Cluster, Gap, Inset, Stack};

use crate::code::{El, each, snippet, text_node};
use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind, samples};

/// How many lines a text keeps: one (`truncate`), a clamp's, or all.
const LINES: &[(&str, Option<&str>)] = &[
    ("1 (truncate)", Some(sockel::class::TRUNCATE)),
    ("2", Some(sockel::class::CLAMP_2)),
    ("3", Some(sockel::class::CLAMP_3)),
    ("4", Some(sockel::class::CLAMP_4)),
    ("All", None),
];

/// The name of a class's constant in `sockel::class`.
fn constant(class: &str) -> String {
    format!(
        "sockel::class::{}",
        class
            .trim_start_matches("sockel-")
            .replace('-', "_")
            .to_uppercase()
    )
}

const FIGURES: &[(&str, &str)] = &[
    ("1111", "1111"),
    ("8888", "8888"),
    ("1010", "1010"),
    ("0", "0"),
];

const MARKS: &[(&str, &str)] = &[
    ("background", class::BY_BACKGROUND),
    ("color", class::BY_COLOR),
    ("underline", class::BY_UNDERLINE),
];

const TABS: &[&str] = &["Overview", "Members", "Settings"];

const COLOURS: &[&str] = &[
    "--danger",
    "--success",
    "--warning",
    "--on-accent",
    "--surface-raised",
    "--scrim",
];

/// The utilities and mixins against jitter, and the colour contract.
#[function_component]
pub fn FoundationsPage() -> Html {
    let text = use_state(|| Text::Long);
    let lines = use_state(|| Some(sockel::class::TRUNCATE));
    let figure = use_state(|| "1111");
    let numeric = use_state(|| true);
    let reserve = use_state(|| true);
    let mono = use_state(|| true);
    let count = use_state(|| 3);
    let scroll_area = use_state(|| true);
    let mark = use_state(|| class::BY_BACKGROUND);
    let tab = use_state(|| 0);

    let controls = html! {
        <>
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<Option<&'static str>> label="Lines" options={LINES} value={*lines} on_change={bind(&lines)} />
            <Select<&'static str> label="Figure" options={FIGURES} value={*figure} on_change={bind(&figure)} />
            <Toggle label="numeric" value={*numeric} on_change={bind(&numeric)} />
            <Toggle label="reserve-ch" value={*reserve} on_change={bind(&reserve)} />
            <Toggle label="mono" value={*mono} on_change={bind(&mono)} />
            <Select<usize> label="Scrolled items" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Toggle label="scroll-area" value={*scroll_area} on_change={bind(&scroll_area)} />
            <Select<&'static str> label="current by" options={MARKS} value={*mark} on_change={bind(&mark)} />
        </>
    };

    let tabs = TABS.iter().enumerate().map(|(index, name)| {
        let selected = index == *tab;
        let tab = tab.clone();
        let onclick = Callback::from(move |_: MouseEvent| tab.set(index));
        html! {
            <button type="button" role="tab" class={classes!(class::TAB, *mark)}
                aria-selected={if selected { "true" } else { "false" }} {onclick}>
                { *name }
            </button>
        }
    });

    let class_of = |on: bool, class: &str| -> String {
        if on {
            format!(" class={{{}}}", constant(class))
        } else {
            String::new()
        }
    };
    let mark_name = MARKS
        .iter()
        .find(|(_, m)| m == &*mark)
        .map_or("", |(name, _)| *name);
    let code = snippet(
        &[],
        El::new("")
            .line(format!(
                "<p{}>{}</p>",
                class_of(lines.is_some(), lines.unwrap_or_default()),
                text_node(&text.of("A title"))
            ))
            .line(format!(
                "<span{}>{}</span>",
                class_of(*numeric, sockel::class::NUMERIC),
                text_node(*figure)
            ))
            .line(if *reserve {
                format!(
                    "<span class={{{}}} style=\"--reserve-ch: 6\">{}</span>",
                    constant(sockel::class::RESERVE_CH),
                    text_node(*figure)
                )
            } else {
                format!("<span>{}</span>", text_node(*figure))
            })
            .line(format!(
                "<span{}>{}</span>",
                class_of(*mono, sockel::class::MONO),
                text_node("List<Int>")
            ))
            .child(
                El::new("div")
                    .expr_if(*scroll_area, "class", constant(sockel::class::SCROLL_AREA))
                    .line_if(*count > 0, each("items", "item", *count, "item")),
            )
            .line(format!(
                "<div role=\"tablist\">{{ for tabs }}</div> // each tab: @include p.current({});",
                if mark_name == "background" {
                    ""
                } else {
                    mark_name
                }
            ))
            .line(format!(
                "<span class={{{}}}>{}</span>",
                constant(sockel::class::PHONE_ONLY),
                text_node("phone-only")
            ))
            .line(format!(
                "<span class={{{}}}>{}</span>",
                constant(sockel::class::DESKTOP_ONLY),
                text_node("desktop-only")
            ))
            .line(format!(
                "<a href=\"#foundations\" class={{{}}}>{}</a>",
                constant(sockel::class::FOCUS_RING),
                text_node("A link with the focus ring: tab to it")
            )),
    );

    html! {
        <Stack gap={Gap::Loose}>
            <Demo {controls} {code}>
                <Stack gap={Gap::Medium}>
                    <h2 class={sockel::class::VISUALLY_HIDDEN}>{ "Foundations" }</h2>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Text fitting" }</span>
                        <p class={classes!(class::SAMPLE, class::PROSE, *lines)}>
                            { text.of("A title") }
                        </p>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Figures" }</span>
                        <span class={classes!(class::READOUT, numeric.then_some(sockel::class::NUMERIC))}>
                            { *figure }
                        </span>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Reserved width (6ch)" }</span>
                        <span>
                            <span class={classes!(class::READOUT, reserve.then_some(sockel::class::RESERVE_CH))}
                                style="--reserve-ch: 6; display: inline-block; outline: 1px dashed currentColor;">
                                { *figure }
                            </span>
                            { " follows" }
                        </span>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Code face" }</span>
                        <span class={classes!(mono.then_some(sockel::class::MONO))}>{ "List<Int> = [4, 2, 6]" }</span>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Scroll area" }</span>
                        <div class={classes!(class::SCROLL_BOX, scroll_area.then_some(sockel::class::SCROLL_AREA))}>
                            <Stack gap={Gap::Snug}>{ samples("div", *count, Text::Short, false, false) }</Stack>
                        </div>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Current item (click one)" }</span>
                        <div role="tablist" class={class::TABS}>{ for tabs }</div>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "By width" }</span>
                        <span class={classes!(class::SAMPLE, sockel::class::PHONE_ONLY)}>{ "phone-only" }</span>
                        <span class={classes!(class::SAMPLE, sockel::class::DESKTOP_ONLY)}>{ "desktop-only" }</span>
                    </Stack>
                    <a href="#foundations" class={classes!(sockel::class::FOCUS_RING, class::PROSE)}>
                        { "A link with the focus ring: tab to it" }
                    </a>
                </Stack>
            </Demo>
            <Card inset={Inset::Compact}>
                <Stack gap={Gap::Snug}>
                    <span class={class::CONTROL_LABEL}>
                        { "The colours sockel derives from the app's seven; the app sets any of them by name." }
                    </span>
                    <Cluster gap={Gap::Tight}>
                        { for COLOURS.iter().map(|name| html! {
                            <Cluster gap={Gap::Snug} inline=true>
                                <span class={class::SWATCH} style={format!("--swatch: var({name})")} />
                                <code>{ *name }</code>
                            </Cluster>
                        }) }
                    </Cluster>
                </Stack>
            </Card>
        </Stack>
    }
}
