use yew::prelude::*;

use crate::class;

use plinth::{
    Button, ButtonSize, ButtonVariant, Cluster, DataTips, HoverCard, PopoverAlign, ReasonButton,
    Side, Stack, Tooltip, button,
};

use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const SIDES: &[(&str, Side)] = &[
    ("Above", Side::Above),
    ("Below", Side::Below),
    ("Start", Side::Start),
    ("End", Side::End),
];

const ALIGNS: &[(&str, PopoverAlign)] = &[
    ("Start", PopoverAlign::Start),
    ("Center", PopoverAlign::Center),
    ("End", PopoverAlign::End),
];

const DELAYS: &[(&str, u32)] = &[("250 ms", 250), ("0 (at once)", 0), ("1000 ms", 1000)];

const CARD_DELAYS: &[(&str, u32)] = &[("400 ms", 400), ("0 (at once)", 0), ("1000 ms", 1000)];

#[derive(Properties, PartialEq)]
struct TooltipDemoProps {
    side: Side,
    delay: u32,
    text: Text,
    attribute: bool,
}

#[function_component]
fn TooltipDemo(props: &TooltipDemoProps) -> Html {
    let anchor = use_node_ref();
    let text = props.text.of("Remove from the list");
    let classes = button(ButtonVariant::Secondary, ButtonSize::Regular, false, false);

    html! {
        <Stack>
            <Cluster>
                if props.attribute {
                    <button type="button" class={classes} data-tip={text.clone()}>
                        <span class={plinth::class::BUTTON_LABEL}>{ "Remove" }</span>
                    </button>
                } else {
                    <Button node_ref={anchor.clone()}>{ "Remove" }</Button>
                    <Tooltip {anchor} text={text} side={props.side} delay={props.delay} />
                }
            </Cluster>
            <p class={class::SAMPLE}>{ "The page under the tip: it does not move as the tip shows." }</p>
        </Stack>
    }
}

#[function_component]
pub fn TooltipPage() -> Html {
    let side = use_state(|| Side::Above);
    let delay = use_state(|| 250u32);
    let text = use_state(|| Text::Short);
    let attribute = use_state(|| false);

    let controls = html! {
        <>
            <Select<Side> label="side" options={SIDES} value={*side} on_change={bind(&side)} />
            <Select<u32> label="delay" options={DELAYS} value={*delay} on_change={bind(&delay)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="data-tip (DataTips)" value={*attribute} on_change={bind(&attribute)} />
        </>
    };

    let words = text.of("Remove from the list");
    let code = if *attribute {
        snippet(
            &[],
            El::new("DataTips")
                .line("// once, anywhere; then on any element:")
                .line(format!(
                    "<button data-tip={{\"{}\"}}>{{ \"Remove\" }}</button>",
                    words
                )),
        )
    } else {
        snippet(
            &["let anchor = use_node_ref();".to_owned()],
            El::new("Tooltip")
                .expr("anchor", "anchor.clone()")
                .text("text", &words)
                .variant("side", *side)
                .expr_if(*delay != 250, "delay", delay.to_string()),
        )
    };

    html! {
        <Demo {controls} {code}>
            if *attribute {
                <DataTips />
            }
            <TooltipDemo side={*side} delay={*delay} text={*text} attribute={*attribute} />
        </Demo>
    }
}

#[derive(Properties, PartialEq)]
struct HoverCardDemoProps {
    side: Side,
    align: PopoverAlign,
    text: Text,
    delay: u32,
}

#[function_component]
fn HoverCardDemo(props: &HoverCardDemoProps) -> Html {
    let anchor = use_node_ref();
    let text = props.text.of("Added on 3 March by the library");

    html! {
        <Stack>
            <Cluster>
                <Button node_ref={anchor.clone()}>{ "Hover or focus me" }</Button>
                <HoverCard {anchor} side={props.side} align={props.align} delay={props.delay}
                    title="Details">
                    <Stack>
                        <strong>{ "Details" }</strong>
                        if !text.is_empty() {
                            <p class={class::PROSE}>{ text }</p>
                        }
                        <a href="#hover-card">{ "A link inside the card" }</a>
                    </Stack>
                </HoverCard>
            </Cluster>
            <p class={class::SAMPLE}>{ "The page under the card: it does not move as the card opens." }</p>
        </Stack>
    }
}

#[function_component]
pub fn HoverCardPage() -> Html {
    let side = use_state(Side::default);
    let align = use_state(PopoverAlign::default);
    let text = use_state(|| Text::Short);
    let delay = use_state(|| 400u32);

    let controls = html! {
        <>
            <Select<Side> label="side" options={SIDES} value={*side} on_change={bind(&side)} />
            <Select<PopoverAlign> label="align" options={ALIGNS} value={*align} on_change={bind(&align)} />
            <Select<Text> label="Content" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<u32> label="delay" options={CARD_DELAYS} value={*delay} on_change={bind(&delay)} />
        </>
    };

    let words = text.of("Added on 3 March by the library");
    let code = snippet(
        &["let anchor = use_node_ref();".to_owned()],
        El::new("HoverCard")
            .expr("anchor", "anchor.clone()")
            .variant("side", *side)
            .variant("align", *align)
            .expr_if(*delay != 400, "delay", delay.to_string())
            .text("title", "Details")
            .child(
                El::new("Stack")
                    .line("<strong>{ \"Details\" }</strong>")
                    .line_if(!words.is_empty(), format!("<p>{}</p>", text_node(&words))),
            ),
    );

    html! {
        <Demo {controls} {code}>
            <HoverCardDemo side={*side} align={*align} text={*text} delay={*delay} />
        </Demo>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Situation {
    Available,
    Unavailable,
    Working,
}

const SITUATIONS: &[(&str, Situation)] = &[
    ("Available", Situation::Available),
    ("Unavailable, with a reason", Situation::Unavailable),
    ("Working", Situation::Working),
];

const SIZES: &[(&str, ButtonSize)] = &[
    ("Regular", ButtonSize::Regular),
    ("Small", ButtonSize::Small),
    ("Touch", ButtonSize::Touch),
];

#[function_component]
pub fn ReasonButtonPage() -> Html {
    let situation = use_state(|| Situation::Unavailable);
    let reason = use_state(|| Text::Short);
    let label = use_state(|| Text::Short);
    let size = use_state(ButtonSize::default);
    let clicks = use_state(|| 0);

    let controls = html! {
        <>
            <Select<Situation> label="state" options={SITUATIONS} value={*situation} on_change={bind(&situation)} />
            <Select<Text> label="reason" options={Text::OPTIONS} value={*reason} on_change={bind(&reason)} />
            <Select<Text> label="Label" options={Text::OPTIONS} value={*label} on_change={bind(&label)} />
            <Select<ButtonSize> label="size" options={SIZES} value={*size} on_change={bind(&size)} />
        </>
    };

    let words = reason.of("Pick a list first");
    let name = label.of("Save the draft");
    let unavailable = *situation == Situation::Unavailable;
    let onclick = {
        let clicks = clicks.clone();
        Callback::from(move |_: MouseEvent| clicks.set(*clicks + 1))
    };

    let code = snippet(
        &[],
        El::new("ReasonButton")
            .variant("size", *size)
            .expr_if(
                unavailable,
                "reason",
                format!("AttrValue::from(\"{words}\")"),
            )
            .flag("busy", *situation == Situation::Working)
            .text_if(
                *situation == Situation::Working,
                "working",
                "Saving the draft",
            )
            .expr("onclick", "on_click")
            .line_if(!name.is_empty(), text_node(&name)),
    );

    html! {
        <Demo {controls} {code}>
            <Cluster>
                <ReasonButton size={*size} variant={ButtonVariant::Primary}
                    reason={unavailable.then(|| AttrValue::from(words))}
                    busy={*situation == Situation::Working}
                    working="Saving the draft" label={name.is_empty().then(|| AttrValue::from("Save the draft"))}
                    {onclick}>
                    { name }
                </ReasonButton>
                <span class={class::CONTROL_LABEL}>{ format!("Clicked {} times", *clicks) }</span>
            </Cluster>
            <p class={class::SAMPLE}>{ "The page under the button: it does not move across the states." }</p>
        </Demo>
    }
}
