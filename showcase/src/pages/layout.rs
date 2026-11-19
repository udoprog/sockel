use yew::prelude::*;

use crate::class;

use sockel::{
    Align, Card, Cluster, Gap, Inset, Justify, Stack, TextInput, Toolbar, ToolbarSpacer, card,
    cluster, stack, stack_with, toolbar,
};

use crate::code::{El, each, snippet, text_node};
use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind, button, samples};

const GAPS: &[(&str, Gap)] = &[
    ("Normal", Gap::Normal),
    ("None", Gap::None),
    ("Hair", Gap::Hair),
    ("Snug", Gap::Snug),
    ("Tight", Gap::Tight),
    ("Medium", Gap::Medium),
    ("Loose", Gap::Loose),
    ("Wide", Gap::Wide),
];

const JUSTIFIES: &[(&str, Justify)] = &[
    ("Start", Justify::Start),
    ("Center", Justify::Center),
    ("Between", Justify::Between),
    ("End", Justify::End),
];

const ALIGNS: &[(&str, Align)] = &[
    ("Center", Align::Center),
    ("Top", Align::Top),
    ("Baseline", Align::Baseline),
    ("Bottom", Align::Bottom),
    ("Stretch", Align::Stretch),
];

const INSETS: &[(&str, Inset)] = &[
    ("Normal", Inset::Normal),
    ("Compact", Inset::Compact),
    ("Flush", Inset::Flush),
];

const TAGS: &[(&str, &str)] = &[
    ("div", "div"),
    ("section", "section"),
    ("ul", "ul"),
    ("ol", "ol"),
];

#[function_component]
pub fn StackPage() -> Html {
    let gap = use_state(|| Gap::Normal);
    let tag = use_state(|| "div");
    let count = use_state(|| 3);
    let text = use_state(|| Text::Short);
    let flush = use_state(|| false);
    let paragraphs = use_state(|| false);

    let controls = html! {
        <>
            <Select<Gap> label="gap" options={GAPS} value={*gap} on_change={bind(&gap)} />
            <Select<&'static str> label="tag" options={TAGS} value={*tag} on_change={bind(&tag)} />
            <Select<usize> label="Items" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="flush" value={*flush} on_change={bind(&flush)} />
            <Toggle label="Items are paragraphs" value={*paragraphs} on_change={bind(&paragraphs)} />
        </>
    };

    let code = snippet(
        &[],
        El::new("Stack")
            .variant("gap", *gap)
            .text_if(*tag != "div", "tag", *tag)
            .flag("flush", *flush)
            .line_if(*count > 0, items(*tag, *count, *paragraphs)),
    );

    html! {
        <Demo {controls} {code}>
            <Stack gap={*gap} tag={*tag} flush={*flush} class={class::OUTLINED}>
                { samples(*tag, *count, *text, false, *paragraphs) }
            </Stack>
        </Demo>
    }
}

#[function_component]
pub fn ClusterPage() -> Html {
    let gap = use_state(|| Gap::Normal);
    let tag = use_state(|| "div");
    let justify = use_state(|| Justify::Start);
    let align = use_state(|| Align::Center);
    let nowrap = use_state(|| false);
    let inline = use_state(|| false);
    let mixed = use_state(|| true);
    let lines = use_state(|| false);
    let flush = use_state(|| false);
    let paragraphs = use_state(|| false);
    let count = use_state(|| 3);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Select<Gap> label="gap" options={GAPS} value={*gap} on_change={bind(&gap)} />
            <Select<&'static str> label="tag" options={TAGS} value={*tag} on_change={bind(&tag)} />
            <Select<Justify> label="justify" options={JUSTIFIES} value={*justify} on_change={bind(&justify)} />
            <Select<Align> label="align" options={ALIGNS} value={*align} on_change={bind(&align)} />
            <Select<usize> label="Items" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="nowrap" value={*nowrap} on_change={bind(&nowrap)} />
            <Toggle label="inline" value={*inline} on_change={bind(&inline)} />
            <Toggle label="lines" value={*lines} on_change={bind(&lines)} />
            <Toggle label="flush" value={*flush} on_change={bind(&flush)} />
            <Toggle label="Mixed heights" value={*mixed} on_change={bind(&mixed)} />
            <Toggle label="Items are paragraphs" value={*paragraphs} on_change={bind(&paragraphs)} />
        </>
    };

    let code = snippet(
        &[],
        El::new("Cluster")
            .variant("gap", *gap)
            .text_if(*tag != "div", "tag", *tag)
            .variant("justify", *justify)
            .variant("align", *align)
            .flag("nowrap", *nowrap)
            .flag("inline", *inline)
            .flag("lines", *lines)
            .flag("flush", *flush)
            .line_if(*count > 0, items(*tag, *count, *paragraphs)),
    );

    html! {
        <Demo {controls} {code}>
            <Cluster gap={*gap} justify={*justify} align={*align} nowrap={*nowrap} inline={*inline}
                lines={*lines} flush={*flush} tag={*tag} class={class::OUTLINED}>
                { samples(*tag, *count, *text, *mixed, *paragraphs) }
            </Cluster>
        </Demo>
    }
}

#[function_component]
pub fn ToolbarPage() -> Html {
    let gap = use_state(|| Gap::Normal);
    let boxed = use_state(|| true);
    let spacer = use_state(|| true);
    let small = use_state(|| false);
    let count = use_state(|| 3);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Select<Gap> label="gap" options={GAPS} value={*gap} on_change={bind(&gap)} />
            <Toggle label="boxed" value={*boxed} on_change={bind(&boxed)} />
            <Toggle label="ToolbarSpacer" value={*spacer} on_change={bind(&spacer)} />
            <Toggle label="Small buttons" value={*small} on_change={bind(&small)} />
            <Select<usize> label="Buttons" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Labels" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
        </>
    };

    let half = count.div_ceil(2);
    let buttons = |range: std::ops::RangeInclusive<usize>| -> Html {
        range
            .map(|n| button(text.of(&format!("Action {n}")), false, *small))
            .collect()
    };

    let rest = *count - half;
    let code = snippet(
        &[],
        El::new("Toolbar")
            .text("label", "Showcase toolbar")
            .flag("boxed", *boxed)
            .variant("gap", *gap)
            .line_if(
                half > 0,
                each(
                    if *spacer { "first" } else { "actions" },
                    "button",
                    if *spacer { half } else { *count },
                    "button",
                ),
            )
            .line_if(*spacer, "<ToolbarSpacer />")
            .line_if(*spacer && rest > 0, each("rest", "button", rest, "button")),
    );

    html! {
        <Demo {controls} {code}>
            <Toolbar label="Showcase toolbar" boxed={*boxed} gap={*gap}>
                { buttons(1..=half) }
                if *spacer { <ToolbarSpacer /> }
                { buttons(half + 1..=*count) }
            </Toolbar>
        </Demo>
    }
}

#[function_component]
pub fn CardPage() -> Html {
    let inset = use_state(|| Inset::Normal);
    let tag = use_state(|| "div");
    let text = use_state(|| Text::Short);
    let list = use_state(|| false);
    let stacked = use_state(|| false);

    let controls = html! {
        <>
            <Select<Inset> label="inset" options={INSETS} value={*inset} on_change={bind(&inset)} />
            <Toggle label="Stacked, with a row" value={*stacked} on_change={bind(&stacked)} />
            <Select<&'static str> label="tag" options={TAGS} value={*tag} on_change={bind(&tag)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="Holds a list" value={*list} on_change={bind(&list)} />
        </>
    };

    let content = if *list {
        html! { <Stack gap={Gap::Hair}>{ samples("div", 4, *text, false, false) }</Stack> }
    } else {
        html! { <p class={class::PROSE}>{ text.of("A card's content") }</p> }
    };

    let content = match *tag {
        "ul" | "ol" => html! { <li>{ content }</li> },
        _ => content,
    };
    let inner = if *list {
        El::new("Stack")
            .variant("gap", Gap::Hair)
            .line(each("items", "item", 4, "item"))
    } else {
        El::new("p").line(text_node(&text.of("A card's content")))
    };
    let code = snippet(
        &[],
        El::new("Card")
            .variant("inset", *inset)
            .text_if(*tag != "div", "tag", *tag)
            .child(match *tag {
                "ul" | "ol" => El::new("li").child(inner),
                _ => inner,
            }),
    );

    html! {
        <Demo {controls} {code}>
            <Card inset={*inset} tag={*tag} gap={(*stacked).then_some(Gap::Normal)}>
                { content }
                if *stacked {
                    <Cluster>
                        <TextInput value="" on_input={Callback::noop()} label="Name"
                            class={classes!(sockel::class::IN_ROW)} />
                        { button("Save".to_owned(), false, false) }
                    </Cluster>
                }
            </Card>
        </Demo>
    }
}

#[function_component]
pub fn ClassesPage() -> Html {
    let gap = use_state(|| Gap::Normal);
    let justify = use_state(|| Justify::Start);
    let align = use_state(|| Align::Center);
    let wrap = use_state(|| true);
    let inline = use_state(|| false);
    let flush = use_state(|| false);
    let boxed = use_state(|| false);
    let inset = use_state(|| Inset::Normal);
    let link = use_state(|| true);

    let controls = html! {
        <>
            <Select<Gap> label="gap" options={GAPS} value={*gap} on_change={bind(&gap)} />
            <Select<Justify> label="justify" options={JUSTIFIES} value={*justify} on_change={bind(&justify)} />
            <Select<Align> label="align" options={ALIGNS} value={*align} on_change={bind(&align)} />
            <Toggle label="wrap" value={*wrap} on_change={bind(&wrap)} />
            <Toggle label="inline" value={*inline} on_change={bind(&inline)} />
            <Toggle label="flush" value={*flush} on_change={bind(&flush)} />
            <Toggle label="boxed" value={*boxed} on_change={bind(&boxed)} />
            <Select<Inset> label="inset" options={INSETS} value={*inset} on_change={bind(&inset)} />
            <Toggle label="link" value={*link} on_change={bind(&link)} />
        </>
    };

    let (stack, stack_call) = if *flush {
        (
            stack_with(*gap, true),
            format!("stack_with(Gap::{:?}, true)", *gap),
        )
    } else {
        (stack(*gap), format!("stack(Gap::{:?})", *gap))
    };
    let cluster = cluster(*gap, *justify, *align, *wrap, *inline);
    let toolbar = toolbar(*boxed);
    let card = card(*inset, *link);
    let shown = |call: &str, classes: &Classes| {
        html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
    };

    let code = snippet(
        &[
            format!("let stack = {stack_call};"),
            format!(
                "let cluster = cluster(Gap::{:?}, Justify::{:?}, Align::{:?}, {}, {});",
                *gap, *justify, *align, *wrap, *inline
            ),
            format!("let toolbar = toolbar({});", *boxed),
            format!("let card = card(Inset::{:?}, {});", *inset, *link),
        ],
        El::new("")
            .line("<ul class={stack}>{ for items.iter().map(item) }</ul>")
            .line("<ul class={cluster}>{ for items.iter().map(item) }</ul>")
            .line("<nav class={toolbar}>{ for actions.iter().map(button) }</nav>")
            .line(format!(
                "<a href=\"#classes\" class={{card}}>{}</a>",
                text_node("A card that is a link: hover or focus it")
            )),
    );

    html! {
        <Demo {controls} {code}>
            <Stack gap={Gap::Loose}>
                { shown(if *flush { "stack_with(gap, flush)" } else { "stack(gap)" }, &stack) }
                <ul class={classes!(stack, sockel::class::PLAIN_LIST, class::OUTLINED)}>{ samples("ul", 3, Text::Short, false, false) }</ul>
                { shown("cluster(gap, justify, align, wrap, inline)", &cluster) }
                <ul class={classes!(cluster, sockel::class::PLAIN_LIST, class::OUTLINED)}>{ samples("ul", 3, Text::Short, true, false) }</ul>
                { shown("toolbar(boxed)", &toolbar) }
                <nav class={toolbar} aria-label="Toolbar classes">
                    { button("One".into(), false, false) }
                    { button("Two".into(), false, false) }
                </nav>
                { shown("card(inset, link)", &card) }
                <a href="#classes" class={card} onclick={Callback::from(|e: MouseEvent| e.prevent_default())}>
                    { "A card that is a link: hover or focus it" }
                </a>
            </Stack>
        </Demo>
    }
}

/// The children line for `count` sample items: paragraphs, or list items
/// in a list.
fn items(tag: &str, count: usize, paragraphs: bool) -> String {
    let thing = match tag {
        "ul" | "ol" => "list item",
        _ if paragraphs => "paragraph",
        _ => "item",
    };
    each("items", "item", count, thing)
}
