use yew::prelude::*;

use crate::class;

use sockel::{
    Badge, Cluster, Count, Dot, Pill, StatusLabel, Tone, badge, count, count_text, dot, pill,
    status_label,
};

use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const TONES: &[(&str, Tone)] = &[
    ("Neutral", Tone::Neutral),
    ("Accent", Tone::Accent),
    ("Success", Tone::Success),
    ("Warning", Tone::Warning),
    ("Danger", Tone::Danger),
    ("Busy", Tone::Busy),
    ("Unset", Tone::Unset),
];

#[function_component]
pub fn PillPage() -> Html {
    let tone = use_state(|| Tone::Success);
    let text = use_state(|| Text::Short);
    let plain = use_state(|| false);
    let button = use_state(|| false);
    let disabled = use_state(|| false);
    let clicks = use_state(|| 0u32);

    let controls = html! {
        <>
            <Select<Tone> label="Tone" options={TONES} value={*tone} on_change={bind(&tone)} />
            <Select<Text> label="Label" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="plain" value={*plain} on_change={bind(&plain)} />
            <Toggle label="on_click (a button)" value={*button} on_change={bind(&button)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };
    let on_click = (*button).then(|| {
        let clicks = clicks.clone();
        Callback::from(move |_: MouseEvent| clicks.set(*clicks + 1))
    });

    let label = text.of("Synced");
    let code = snippet(
        &[],
        El::new("Pill")
            .variant("tone", *tone)
            .flag("plain", *plain)
            .expr_if(*button, "on_click", "on_click")
            .flag("disabled", *button && *disabled)
            .line_if(!label.is_empty(), text_node(&label)),
    );

    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("pill(tone, plain, button) → \"{}\"", pill(*tone, *plain, *button).to_string()) }
            </code>
            <Cluster>
                <Pill tone={*tone} plain={*plain} {on_click} disabled={*disabled}>
                    { text.of("Synced") }
                </Pill>
            </Cluster>
            <span class={class::CONTROL_LABEL}>{ format!("{} clicks", *clicks) }</span>
        </Demo>
    }
}

#[function_component]
pub fn BadgePage() -> Html {
    let tone = use_state(|| Tone::Neutral);
    let text = use_state(|| Text::Short);
    let solid = use_state(|| false);
    let corner = use_state(|| false);
    let on = use_state(|| true);
    let label = use_state(|| false);

    let controls = html! {
        <>
            <Select<Tone> label="Tone" options={TONES} value={*tone} on_change={bind(&tone)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="solid" value={*solid} on_change={bind(&solid)} />
            <Toggle label="corner" value={*corner} on_change={bind(&corner)} />
            <Toggle label="Dot on" value={*on} on_change={bind(&on)} />
            <Toggle label="Dot label" value={*label} on_change={bind(&label)} />
        </>
    };

    let figure = text.of("3");
    let code = snippet(
        &[],
        El::new("")
            .child(
                El::new("Badge")
                    .variant("tone", *tone)
                    .flag("solid", *solid)
                    .flag("corner", *corner)
                    .line_if(!figure.is_empty(), text_node(&figure)),
            )
            .child(
                El::new("Dot")
                    .variant("tone", *tone)
                    .expr_if(!*on, "on", "false")
                    .text_if(*label, "label", "Online"),
            ),
    );

    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("badge(tone, solid, corner) → \"{}\"", badge(*tone, *solid, *corner).to_string()) }
            </code>
            <code class={class::CLASS_CALL}>
                { format!("dot(tone, on) → \"{}\"", dot(*tone, *on).to_string()) }
            </code>
            <Cluster>
                <span class={class::ANCHOR}>
                    { "Inbox" }
                    <Badge tone={*tone} solid={*solid} corner={*corner}>{ text.of("3") }</Badge>
                </span>
                <Dot tone={*tone} on={*on} label={(*label).then(|| AttrValue::from("Online"))} />
            </Cluster>
        </Demo>
    }
}

const VALUES: &[(&str, u32)] = &[
    ("0", 0),
    ("7", 7),
    ("42", 42),
    ("99", 99),
    ("100", 100),
    ("1000", 1000),
];

const MAXES: &[(&str, u32)] = &[("9", 9), ("99", 99), ("999", 999)];

#[function_component]
pub fn CountPage() -> Html {
    let value = use_state(|| 7u32);
    let max = use_state(|| 99u32);
    let hide_zero = use_state(|| false);

    let controls = html! {
        <>
            <Select<u32> label="Value" options={VALUES} value={*value} on_change={bind(&value)} />
            <Select<u32> label="Max" options={MAXES} value={*max} on_change={bind(&max)} />
            <Toggle label="hide_zero" value={*hide_zero} on_change={bind(&hide_zero)} />
        </>
    };

    let code = snippet(
        &[],
        El::new("Count")
            .expr("value", value.to_string())
            .expr_if(*max != 99, "max", max.to_string())
            .flag("hide_zero", *hide_zero),
    );

    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("count(value, hide_zero) → \"{}\"; count_text → \"{}\"",
                    count(*value, *hide_zero).to_string(), count_text(*value, *max)) }
            </code>
            <Cluster>
                <span>{ "Notices" }</span>
                <Count value={*value} max={*max} hide_zero={*hide_zero} />
                <span>{ "beside it" }</span>
            </Cluster>
        </Demo>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Save {
    Idle,
    Saving,
    Saved,
    Failed,
}

const SAVES: &[(&str, Save)] = &[
    ("Idle", Save::Idle),
    ("Saving", Save::Saving),
    ("Saved", Save::Saved),
    ("Failed", Save::Failed),
];

#[function_component]
pub fn StatusLabelPage() -> Html {
    let state = use_state(|| Save::Saving);
    let spinner = use_state(|| true);
    let pill_on = use_state(|| false);

    let controls = html! {
        <>
            <Select<Save> label="State" options={SAVES} value={*state} on_change={bind(&state)} />
            <Toggle label="spinner (a slot)" value={*spinner} on_change={bind(&spinner)} />
            <Toggle label="pill" value={*pill_on} on_change={bind(&pill_on)} />
        </>
    };

    let (current, tone) = match *state {
        Save::Idle => (None, Tone::Neutral),
        Save::Saving => (Some(0), Tone::Busy),
        Save::Saved => (Some(1), Tone::Success),
        Save::Failed => (Some(2), Tone::Danger),
    };
    let states = vec![
        AttrValue::from("Loading..."),
        AttrValue::from("Saved"),
        AttrValue::from("Not saved, change reverted"),
    ];

    let code = snippet(
        &[
            "let states = vec![\"Loading...\".into(), \"Saved\".into(), \"Not saved, change reverted\".into()];"
                .to_owned(),
        ],
        El::new("StatusLabel")
            .expr("states", "states")
            .expr("current", format!("{current:?}"))
            .variant("tone", tone)
            .flag("spinner", *spinner)
            .flag("busy", *state == Save::Saving)
            .flag("pill", *pill_on),
    );

    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("status_label(tone, pill) → \"{}\"", status_label(tone, *pill_on).to_string()) }
            </code>
            <Cluster>
                <StatusLabel {states} {current} {tone} spinner={*spinner}
                    busy={*state == Save::Saving} pill={*pill_on} />
                <span>{ "beside it" }</span>
            </Cluster>
        </Demo>
    }
}
