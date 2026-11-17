use yew::prelude::*;

use crate::class;

use plinth::{
    Button, ButtonSize, ButtonVariant, Cluster, Dismiss, Popover, PopoverAlign, Side, Stack, button,
};

use crate::code::{El, snippet, text_node};
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
    within: bool,
    flush: bool,
    focus_first: bool,
    match_anchor: bool,
    keep_focus: bool,
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
                // Wide, when the popover matches it, to show it doing so.
                <button type="button" ref={anchor.clone()} class={classes}
                    style={props.match_anchor.then_some("min-width: 20rem")}
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
                within={props.within.then(|| AttrValue::from(format!(".{}", class::FRAME)))}
                flush={props.flush} focus_first={props.focus_first}
                match_anchor={props.match_anchor} keep_focus={props.keep_focus}
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
    let within = use_state(|| false);
    let flush = use_state(|| false);
    let focus_first = use_state(|| false);
    let match_anchor = use_state(|| false);
    let keep_focus = use_state(|| false);

    let controls = html! {
        <>
            <Select<Side> label="side" options={SIDES} value={*side} on_change={bind(&side)} />
            <Select<PopoverAlign> label="align" options={ALIGNS} value={*align} on_change={bind(&align)} />
            <Select<Text> label="Content" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="sheet (on a phone)" value={*sheet} on_change={bind(&sheet)} />
            <Toggle label="title" value={*titled} on_change={bind(&titled)} />
            <Toggle label="within its frame" value={*within} on_change={bind(&within)} />
            <Toggle label="flush" value={*flush} on_change={bind(&flush)} />
            <Toggle label="focus_first" value={*focus_first} on_change={bind(&focus_first)} />
            <Toggle label="match_anchor" value={*match_anchor} on_change={bind(&match_anchor)} />
            <Toggle label="keep_focus" value={*keep_focus} on_change={bind(&keep_focus)} />
        </>
    };

    let words = text.of("Filters for the list");
    let code = snippet(
        &[],
        El::new("Popover")
            .expr("anchor", "anchor.clone()")
            .expr("open", "*open")
            .expr("on_dismiss", "on_dismiss")
            .variant("side", *side)
            .variant("align", *align)
            .expr_if(!*sheet, "sheet", "false")
            .text_if(*titled, "title", "Filters")
            .text_if(*within, "within", &format!(".{}", class::FRAME))
            .expr_if(*flush, "flush", "true")
            .expr_if(*focus_first, "focus_first", "true")
            .expr_if(*match_anchor, "match_anchor", "true")
            .expr_if(*keep_focus, "keep_focus", "true")
            .child(
                El::new("Stack")
                    .line_if(!words.is_empty(), format!("<p>{}</p>", text_node(&words)))
                    .line(
                        "<Button variant={ButtonVariant::Primary} size={ButtonSize::Small} onclick={done}>{ \"Done\" }</Button>",
                    ),
            ),
    );

    html! {
        <Demo {controls} {code}>
            <PopoverDemo side={*side} align={*align} text={*text} sheet={*sheet} titled={*titled} within={*within} flush={*flush}
                focus_first={*focus_first} match_anchor={*match_anchor} keep_focus={*keep_focus} />
        </Demo>
    }
}
