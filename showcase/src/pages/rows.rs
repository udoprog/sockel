use yew::prelude::*;

use crate::class;

use sockel::{
    ActionRow, Button, ButtonSize, ButtonVariant, Card, CardGrid, Checkbox, EmptyState, ErrorBox,
    Glyph, GridSize, Icon, PickRow, RowList, RowListItem, SelectionBar, action_row, card_grid,
    pick_row, row_list, row_list_item, selection_bar,
};

use crate::code::{El, each, snippet, text_node};
use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind};

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

/// What a list shows: its rows, or the empty or the failed state in their place.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shown {
    Rows,
    Empty,
    Error,
}

const SHOWN: &[(&str, Shown)] = &[
    ("Rows", Shown::Rows),
    ("Empty", Shown::Empty),
    ("Error", Shown::Error),
];

fn small(label: &str, variant: ButtonVariant) -> Html {
    html! { <Button size={ButtonSize::Small} {variant}>{ label.to_owned() }</Button> }
}

#[function_component]
pub fn RowListPage() -> Html {
    let count = use_state(|| 3);
    let shown_state = use_state(|| Shown::Rows);
    let name = use_state(|| Text::Short);
    let detail = use_state(|| Text::Short);
    let divided = use_state(|| true);
    let boxed = use_state(|| true);
    let top = use_state(|| false);
    let actions = use_state(|| true);
    let overlay = use_state(|| false);

    let controls = html! {
        <>
            <Select<usize> label="Rows" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Shown> label="State" options={SHOWN} value={*shown_state} on_change={bind(&shown_state)} />
            <Select<Text> label="Name" options={Text::OPTIONS} value={*name} on_change={bind(&name)} />
            <Select<Text> label="Detail" options={Text::OPTIONS} value={*detail} on_change={bind(&detail)} />
            <Toggle label="divided" value={*divided} on_change={bind(&divided)} />
            <Toggle label="boxed" value={*boxed} on_change={bind(&boxed)} />
            <Toggle label="top" value={*top} on_change={bind(&top)} />
            <Toggle label="Actions" value={*actions} on_change={bind(&actions)} />
            <Toggle label="overlay" value={*overlay} on_change={bind(&overlay)} />
        </>
    };

    let rows: Html = (1..=*count)
        .map(|n| {
            html! {
                <RowListItem top={*top} name={html! { { name.of(&format!("Row {n}")) } }}
                    detail={(*detail != Text::Empty).then(|| html! { { detail.of("Updated today") } })}>
                    if *actions {
                        <ActionRow overlay={*overlay} nowrap=true>
                            { small("Edit", ButtonVariant::Secondary) }
                            { small("Delete", ButtonVariant::Danger) }
                        </ActionRow>
                    }
                </RowListItem>
            }
        })
        .collect();

    let body = match *shown_state {
        Shown::Rows if *count > 0 => rows,
        Shown::Error => html! {
            <ErrorBox title="Could not load the rows" cause="TypeError: failed to fetch">
                { "The list is not available right now" }
            </ErrorBox>
        },
        _ => html! {
            <li>
                <EmptyState compact=true title="No rows yet" icon={html! { <Icon glyph={Glyph::Search} /> }} />
            </li>
        },
    };

    let code = snippet(
        &[],
        El::new("RowList")
            .flag("divided", *divided)
            .flag("boxed", *boxed)
            .child(
                El::new("RowListItem")
                    .flag("top", *top)
                    .expr("name", format!("html! {{ {:?} }}", name.of("Row 1")))
                    .expr_if(
                        *detail != Text::Empty,
                        "detail",
                        format!("Some(html! {{ {:?} }})", detail.of("Updated today")),
                    )
                    .line_if(
                        *actions,
                        format!(
                            "<ActionRow{}>{{ edit_and_delete }}</ActionRow>",
                            if *overlay {
                                " overlay=true nowrap=true"
                            } else {
                                " nowrap=true"
                            }
                        ),
                    ),
            )
            .line(each("rows", "row", *count, "row")),
    );

    html! {
        <Demo {controls} {code}>
            { shown("row_list(divided, boxed)", &row_list(*divided, *boxed)) }
            { shown("row_list_item(top)", &row_list_item(*top)) }
            <RowList divided={*divided} boxed={*boxed}>{ body }</RowList>
            <span class={class::CONTROL_LABEL}>{ "Hover or tab to a row to reveal its overlay actions." }</span>
        </Demo>
    }
}

#[function_component]
pub fn PickRowPage() -> Html {
    let name = use_state(|| Text::Short);
    let detail = use_state(|| Text::Short);
    let mark = use_state(|| true);
    let disabled = use_state(|| false);
    let trailing = use_state(|| false);
    let current = use_state(|| Some(1usize));

    let controls = html! {
        <>
            <Select<Text> label="Name" options={Text::OPTIONS} value={*name} on_change={bind(&name)} />
            <Select<Text> label="Detail" options={Text::OPTIONS} value={*detail} on_change={bind(&detail)} />
            <Toggle label="mark" value={*mark} on_change={bind(&mark)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="Trailing" value={*trailing} on_change={bind(&trailing)} />
        </>
    };

    let rows: Html = (0..3)
        .map(|n| {
            let onclick = {
                let current = current.clone();
                Callback::from(move |_: MouseEvent| current.set(Some(n)))
            };
            html! {
                <PickRow name={html! { { name.of(&format!("Choice {}", n + 1)) } }}
                    detail={(*detail != Text::Empty).then(|| html! { { detail.of("Details") } })}
                    mark={mark.then(|| html! { <Icon glyph={Glyph::Plus} /> })}
                    current={*current == Some(n)} disabled={*disabled} {onclick}>
                    if *trailing { <Icon glyph={Glyph::ChevronRight} /> }
                </PickRow>
            }
        })
        .collect();

    let code = snippet(
        &[],
        El::new("PickRow")
            .expr("name", format!("html! {{ {:?} }}", name.of("Choice 1")))
            .expr_if(
                *detail != Text::Empty,
                "detail",
                format!("Some(html! {{ {:?} }})", detail.of("Details")),
            )
            .expr_if(
                *mark,
                "mark",
                "Some(html! { <Icon glyph={Glyph::Plus} /> })",
            )
            .flag("current", *current == Some(0))
            .flag("disabled", *disabled)
            .expr("onclick", "choose")
            .line_if(*trailing, "<Icon glyph={Glyph::ChevronRight} />"),
    );

    html! {
        <Demo {controls} {code}>
            { shown("pick_row()", &pick_row()) }
            <Card inset={sockel::Inset::Flush}>{ rows }</Card>
            <span class={class::CONTROL_LABEL}>{ "Press a row to choose it; the chosen row is marked by its background." }</span>
        </Demo>
    }
}

const PICKED: &[(&str, usize)] = &[("0", 0), ("1", 1), ("3", 3), ("All", 5)];

#[function_component]
pub fn SelectionBarPage() -> Html {
    let picked = use_state(|| 0usize);
    let label = use_state(|| Text::Short);
    let inset = use_state(|| true);
    let sticky = use_state(|| false);
    let busy = use_state(|| false);
    let disabled = use_state(|| false);

    let controls = html! {
        <>
            <Select<usize> label="picked" options={PICKED} value={*picked} on_change={bind(&picked)} />
            <Select<Text> label="Label" options={Text::OPTIONS} value={*label} on_change={bind(&label)} />
            <Toggle label="inset" value={*inset} on_change={bind(&inset)} />
            <Toggle label="sticky" value={*sticky} on_change={bind(&sticky)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let rows: Html = (0..5)
        .map(|n| {
            let on_change = {
                let picked = picked.clone();
                Callback::from(move |on: bool| {
                    picked.set(if on { *picked + 1 } else { picked.saturating_sub(1) });
                })
            };
            html! {
                <RowListItem name={html! { { format!("Row {}", n + 1) } }}>
                    <Checkbox checked={n < *picked} {on_change} label={format!("Pick row {}", n + 1)} />
                </RowListItem>
            }
        })
        .collect();

    let text = format!("{} selected", *picked);

    let code = snippet(
        &[],
        El::new("")
            .line("<RowList divided=true boxed=true>{ rows_with_checkboxes }</RowList>")
            .child(
                El::new("SelectionBar")
                    .expr("count", picked.to_string())
                    .text("label", &label.of(&text))
                    .flag("inset", *inset)
                    .flag("sticky", *sticky)
                    .line(format!(
                        "<Button size={{ButtonSize::Small}}{}>{}</Button>",
                        if *busy { " busy=true" } else { "" },
                        text_node("Move")
                    )),
            ),
    );

    html! {
        <Demo {controls} {code}>
            <RowList divided=true boxed=true>{ rows }</RowList>
            <SelectionBar count={*picked} label={label.of(&text)} inset={*inset} sticky={*sticky}>
                <Button size={ButtonSize::Small} busy={*busy} disabled={*disabled}>{ "Move" }</Button>
                <Button size={ButtonSize::Small} variant={ButtonVariant::Danger} disabled={*disabled}>{ "Delete" }</Button>
            </SelectionBar>
            { shown("selection_bar(idle, inset, sticky)", &selection_bar(*picked == 0, *inset, *sticky)) }
        </Demo>
    }
}

#[function_component]
pub fn ActionRowPage() -> Html {
    let overlay = use_state(|| true);
    let nowrap = use_state(|| false);
    let count = use_state(|| 2);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Toggle label="overlay" value={*overlay} on_change={bind(&overlay)} />
            <Toggle label="nowrap" value={*nowrap} on_change={bind(&nowrap)} />
            <Select<usize> label="Buttons" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Labels" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
        </>
    };

    let buttons: Html = (1..=*count)
        .map(|n| small(&text.of(&format!("Action {n}")), ButtonVariant::Secondary))
        .collect();

    let code = snippet(
        &[],
        El::new("RowListItem")
            .expr("name", "html! { \"A row with actions\" }")
            .child(
                El::new("ActionRow")
                    .flag("overlay", *overlay)
                    .flag("nowrap", *nowrap)
                    .line(each("buttons", "button", *count, "button")),
            ),
    );

    html! {
        <Demo {controls} {code}>
            { shown("action_row(overlay, nowrap)", &action_row(*overlay, *nowrap)) }
            <RowList divided=true boxed=true>
                <RowListItem name={html! { { "A row with actions" } }}>
                    <ActionRow overlay={*overlay} nowrap={*nowrap}>{ buttons }</ActionRow>
                </RowListItem>
                <RowListItem name={html! { { "Another row" } }} />
            </RowList>
            <span class={class::CONTROL_LABEL}>{ "An overlay is invisible and inert until its row is hovered or has focus; tab to it." }</span>
        </Demo>
    }
}

const SIZES: &[(&str, GridSize)] = &[
    ("Normal", GridSize::Normal),
    ("Wide", GridSize::Wide),
    ("Narrow", GridSize::Narrow),
];

#[function_component]
pub fn CardGridPage() -> Html {
    let count = use_state(|| 3);
    let size = use_state(|| GridSize::Normal);
    let text = use_state(|| Text::Short);
    let mixed = use_state(|| true);

    let controls = html! {
        <>
            <Select<usize> label="Cards" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<GridSize> label="size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="Mixed heights" value={*mixed} on_change={bind(&mixed)} />
        </>
    };

    let cards: Html = (1..=*count)
        .map(|n| {
            let body = if *mixed && n % 2 == 0 {
                format!(
                    "{} and a second line of text",
                    text.of(&format!("Card {n}"))
                )
            } else {
                text.of(&format!("Card {n}"))
            };
            html! { <Card tag="li" inset={sockel::Inset::Compact}>{ body }</Card> }
        })
        .collect();

    let code = snippet(
        &[],
        El::new("CardGrid")
            .variant("size", *size)
            .line(each("cards", "card", *count, "card")),
    );

    html! {
        <Demo {controls} {code}>
            { shown("card_grid(size)", &card_grid(*size)) }
            <CardGrid size={*size}>{ cards }</CardGrid>
            if *count == 0 {
                <EmptyState title="No cards yet" />
            }
        </Demo>
    }
}
