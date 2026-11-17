use yew::prelude::*;

use crate::class;

use plinth::{
    Button, ButtonSize, ButtonVariant, Cluster, Dismiss, Popover, PopoverAlign, Side, Stack, button,
};

use crate::controls::{Demo, Select, Text, Toggle, bind};

const SIDES: &[(&str, Side)] = &[
    ("Below", Side::Below),
    ("Above", Side::Above),
    ("Start", Side::Start),
    ("End", Side::End),
];

const ALIGNS: &[(&str, PopoverAlign)] = &[
    ("Start", PopoverAlign::Start),
    ("Center", PopoverAlign::Center),
    ("End", PopoverAlign::End),
];

#[derive(Properties, PartialEq)]
struct PopoverDemoProps {
    side: Side,
    align: PopoverAlign,
    text: Text,
    sheet: bool,
    titled: bool,
}

/// One anchor and its popover, with its own open state, so each frame has
/// its own.
#[function_component]
fn PopoverDemo(props: &PopoverDemoProps) -> Html {
    let anchor = use_node_ref();
    let open = use_state(|| false);
    let closed = use_state(|| None::<Dismiss>);

    let toggle = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(!*open))
    };
    let on_dismiss = {
        let (open, closed) = (open.clone(), closed.clone());
        Callback::from(move |why| {
            open.set(false);
            closed.set(Some(why));
        })
    };
    let done = {
        let (open, closed) = (open.clone(), closed.clone());
        Callback::from(move |_: MouseEvent| {
            open.set(false);
            closed.set(None);
        })
    };
    let text = props.text.of("Filters for the list");
    let classes = button(ButtonVariant::Secondary, ButtonSize::Regular, false, false);

    html! {
        <Stack>
            <Cluster>
                <button type="button" ref={anchor.clone()} class={classes}
                    aria-expanded={open.to_string()} onclick={toggle}>
                    <span class={plinth::class::BUTTON_LABEL}>{ "Open the popover" }</span>
                </button>
            </Cluster>
            <p class={class::SAMPLE}>{ "The page under the popover: it does not move as it opens, flips or closes." }</p>
            <span class={class::CONTROL_LABEL}>
                { match *closed {
                    Some(why) => format!("Dismissed: {why:?}"),
                    None => "Dismissed: -".to_owned(),
                } }
            </span>
            <Popover anchor={anchor} open={*open} {on_dismiss}
                side={props.side} align={props.align} sheet={props.sheet}
                title={props.titled.then_some(AttrValue::Static("Filters"))}>
                <Stack>
                    if !text.is_empty() {
                        <p class={class::PROSE}>{ text }</p>
                    }
                    <Cluster>
                        <Button variant={ButtonVariant::Primary} size={ButtonSize::Small} onclick={done}>
                            { "Done" }
                        </Button>
                    </Cluster>
                </Stack>
            </Popover>
        </Stack>
    }
}

#[function_component]
pub fn PopoverPage() -> Html {
    let side = use_state(Side::default);
    let align = use_state(PopoverAlign::default);
    let text = use_state(|| Text::Short);
    let sheet = use_state(|| true);
    let titled = use_state(|| true);

    let controls = html! {
        <>
            <Select<Side> label="side" options={SIDES} value={*side} on_change={bind(&side)} />
            <Select<PopoverAlign> label="align" options={ALIGNS} value={*align} on_change={bind(&align)} />
            <Select<Text> label="Content" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="sheet (on a phone)" value={*sheet} on_change={bind(&sheet)} />
            <Toggle label="title" value={*titled} on_change={bind(&titled)} />
        </>
    };

    html! {
        <Demo {controls}>
            <PopoverDemo side={*side} align={*align} text={*text} sheet={*sheet} titled={*titled} />
        </Demo>
    }
}
