use yew::prelude::*;

use crate::class;

use plinth::{Card, Cluster, Gap, Inset, Stack};

use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind, samples};

/// How many lines a text keeps: one (`truncate`), a clamp's, or all.
const LINES: &[(&str, Option<&str>)] = &[
    ("1 (truncate)", Some(plinth::class::TRUNCATE)),
    ("2", Some(plinth::class::CLAMP_2)),
    ("3", Some(plinth::class::CLAMP_3)),
    ("4", Some(plinth::class::CLAMP_4)),
    ("All", None),
];

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
    let lines = use_state(|| Some(plinth::class::TRUNCATE));
    let figure = use_state(|| "1111");
    let numeric = use_state(|| true);
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

    html! {
        <Stack gap={Gap::Loose}>
            <Demo {controls}>
                <Stack gap={Gap::Medium}>
                    <h2 class={plinth::class::VISUALLY_HIDDEN}>{ "Foundations" }</h2>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Text fitting" }</span>
                        <p class={classes!(class::SAMPLE, class::PROSE, *lines)}>
                            { text.of("A title") }
                        </p>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Figures" }</span>
                        <span class={classes!(class::READOUT, numeric.then_some(plinth::class::NUMERIC))}>
                            { *figure }
                        </span>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Scroll area" }</span>
                        <div class={classes!(class::SCROLL_BOX, scroll_area.then_some(plinth::class::SCROLL_AREA))}>
                            <Stack gap={Gap::Snug}>{ samples("div", *count, Text::Short, false, false) }</Stack>
                        </div>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "Current item (click one)" }</span>
                        <div role="tablist" class={class::TABS}>{ for tabs }</div>
                    </Stack>
                    <Stack gap={Gap::Snug}>
                        <span class={class::FRAME_LABEL}>{ "By width" }</span>
                        <span class={classes!(class::SAMPLE, plinth::class::PHONE_ONLY)}>{ "phone-only" }</span>
                        <span class={classes!(class::SAMPLE, plinth::class::DESKTOP_ONLY)}>{ "desktop-only" }</span>
                    </Stack>
                    <a href="#foundations" class={classes!(plinth::class::FOCUS_RING, class::PROSE)}>
                        { "A link with the focus ring: tab to it" }
                    </a>
                </Stack>
            </Demo>
            <Card inset={Inset::Compact}>
                <Stack gap={Gap::Snug}>
                    <span class={class::CONTROL_LABEL}>
                        { "The colours plinth derives from the app's seven; the app sets any of them by name." }
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
