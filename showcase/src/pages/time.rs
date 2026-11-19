use yew::prelude::*;

use sockel::{
    JUST_NOW, Stack, Time, TimeStyle, TimeText, now_seconds, time, time_next, time_text,
    use_wake_at,
};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Toggle, bind};

const STYLES: &[(&str, TimeStyle)] = &[
    ("Ago", TimeStyle::Ago),
    ("Since", TimeStyle::Since),
    ("Until", TimeStyle::Until),
    ("Left", TimeStyle::Left),
    ("At", TimeStyle::At),
];

/// The moment, in seconds from when the page opened.
const MOMENTS: &[(&str, i64)] = &[
    ("3 s ago", -3),
    ("50 s ago", -50),
    ("3 min ago", -180),
    ("2 h ago", -7200),
    ("3 d ago", -3 * 86400),
    ("in 20 s", 20),
    ("in 43 min", 43 * 60),
    ("in 2 h", 7200),
];

/// The page's own count of the clock's wakes, to show how rarely it ticks.
#[function_component]
fn Ticks(props: &TicksProps) -> Html {
    use_wake_at(time_next(props.style, props.at, now_seconds()));
    let said = match time_text(props.style, props.at, now_seconds()) {
        Some(TimeText::Said(n, id)) => format!("time_text: {id}, n = {n:?}"),
        Some(TimeText::Within(n, id, outer)) => format!("time_text: {outer} of {id}, n = {n}"),
        None => "time_text: none (a date)".to_owned(),
    };
    html! { <span class={class::CONTROL_LABEL}>{ said }</span> }
}

#[derive(Properties, PartialEq)]
struct TicksProps {
    style: TimeStyle,
    at: i64,
}

#[function_component]
pub fn TimePage() -> Html {
    let opened = use_state(now_seconds);
    let style = use_state(TimeStyle::default);
    let moment = use_state(|| -50i64);
    let end = use_state(|| false);
    let frozen = use_state(|| false);

    let controls = html! {
        <>
            <Select<TimeStyle> label="style" options={STYLES} value={*style} on_change={bind(&style)} />
            <Select<i64> label="Moment" options={MOMENTS} value={*moment} on_change={bind(&moment)} />
            <Toggle label="end" value={*end} on_change={bind(&end)} />
            <Toggle label="Frozen now" value={*frozen} on_change={bind(&frozen)} />
        </>
    };

    let at = *opened + *moment;
    let now = frozen.then_some(*opened);
    let code = snippet(
        &[],
        El::new("Time")
            .expr("at", "at")
            .variant("style", *style)
            .expr_if(*frozen, "now", "Some(now)")
            .flag("end", *end),
    );

    html! {
        <Demo {controls} {code}>
            <Stack>
                <span class={class::CONTROL_LABEL}>
                    { "Updated " }
                    <Time {at} style={*style} {now} end={*end} />
                    { " · the words after it do not move" }
                </span>
                <ul class={class::SAMPLE}>
                    { for MOMENTS.iter().map(|(name, offset)| html! {
                        <li>
                            <Time at={*opened + offset} style={*style} {now} end={*end} />
                            { format!(" {name}") }
                        </li>
                    }) }
                </ul>
                <Ticks style={*style} {at} />
                <code class={class::CLASS_CALL}>
                    { format!("time(style, end) → \"{}\" · just now below {JUST_NOW} s", time(*style, *end).to_string()) }
                </code>
            </Stack>
        </Demo>
    }
}
