use phrasebook02::{Args, Phrases, yew023::use_phrases_with};
use wasm_bindgen::JsCast;
use web_sys::{Element, KeyboardEvent, KeyboardEventInit};
use yew::prelude::*;

use plinth::{
    Button, ButtonSize, ButtonVariant, DragHandle, EmptyState, Reorder, ReorderHandle, ReorderItem,
    ReorderRow, ReorderText, drag_handle, drag_target, key_target, landing, move_item, reorder_row,
    slot_at,
};

use crate::class;
use crate::code::{El, each, snippet};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const NAMES: &[&str] = &[
    "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel",
];

/// What `reorder_text` says, in the provided language: the text callback
/// overrides plinth's own words, so the page looks up the same messages.
fn say(phrases: &Phrases, text: &ReorderText) -> String {
    let (id, name, place) = match text {
        ReorderText::Handle { name } => ("plinth-reorder-handle", Some(name), None),
        ReorderText::Hint => ("plinth-reorder-hint", None, None),
        ReorderText::Lifted {
            name,
            position,
            count,
        } => (
            "plinth-reorder-lifted",
            Some(name),
            Some((*position, *count)),
        ),
        ReorderText::Moving {
            name,
            position,
            count,
        } => (
            "plinth-reorder-moving",
            Some(name),
            Some((*position, *count)),
        ),
        ReorderText::Dropped {
            name,
            position,
            count,
        } => (
            "plinth-reorder-dropped",
            Some(name),
            Some((*position, *count)),
        ),
        ReorderText::Cancelled {
            name,
            position,
            count,
        } => (
            "plinth-reorder-cancelled",
            Some(name),
            Some((*position, *count)),
        ),
    };
    let mut args = Args::new();
    if let Some(name) = name {
        args.set("name", name.as_str());
    }
    if let Some((position, count)) = place {
        args.set("position", position).set("count", count);
    }
    phrases.msg(id, &args)
}

const COUNTS: &[(&str, usize)] = &[("5", 5), ("0 (empty)", 0), ("1", 1), ("8", 8)];

/// Keys pressed on the second row's handle in each frame: the probe cannot
/// drag, so it lifts and aims by keyboard.
const KEYS: &[&[&str]] = &[
    &[],
    &[" "],
    &[" ", "ArrowDown"],
    &[" ", "ArrowUp"],
    &[" ", "End"],
    &[" ", "Escape"],
];

const KEY_NAMES: &[(&str, usize)] = &[
    ("None", 0),
    ("Space", 1),
    ("Space, Down", 2),
    ("Space, Up", 3),
    ("Space, End", 4),
    ("Space, Escape", 5),
];

fn press(keys: &[&str]) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let lists = doc.get_elements_by_class_name(plinth::class::REORDER);
    for l in 0..lists.length() {
        let Some(handle) = lists.item(l).and_then(|list| {
            list.get_elements_by_class_name(plinth::class::DRAG_HANDLE)
                .item(1)
        }) else {
            continue;
        };
        for key in std::iter::once(&"Escape").chain(keys) {
            let init = KeyboardEventInit::new();
            init.set_key(key);
            init.set_bubbles(true);
            init.set_cancelable(true);
            if let Ok(event) = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init) {
                let _ = handle.unchecked_ref::<Element>().dispatch_event(&event);
            }
        }
    }
}

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn ReorderPage() -> Html {
    let count = use_state(|| 5usize);
    let text = use_state(|| Text::Short);
    let detail = use_state(|| true);
    let end = use_state(|| false);
    let divided = use_state(|| true);
    let boxed = use_state(|| true);
    let disabled = use_state(|| false);
    let children = use_state(|| false);
    let quick_keys = use_state(|| false);
    let titles = use_state(|| false);
    let keys = use_state(|| 0usize);
    let order = use_state(|| (0..NAMES.len()).collect::<Vec<_>>());
    let said = use_state(String::new);

    {
        let pressed = KEYS[*keys];
        use_effect_with(*keys, move |_| press(pressed));
    }

    let on_move = {
        let order = order.clone();
        Callback::from(move |(from, to): (usize, usize)| {
            let mut next = (*order).clone();
            move_item(&mut next, from, to);
            order.set(next);
        })
    };
    let phrases = use_phrases_with(&plinth::phrasebookcatalog01::L10N);
    let text_cb = {
        let said = said.clone();
        Callback::from(move |t: ReorderText| {
            let words = say(&phrases, &t);
            if !matches!(t, ReorderText::Handle { .. } | ReorderText::Hint) {
                said.set(words.clone());
            }
            words
        })
    };
    let rows: Vec<ReorderRow> = order
        .iter()
        .filter(|i| **i < *count)
        .map(|&i| {
            let row = ReorderRow::new(NAMES[i], text.of(NAMES[i]));
            let row = if *detail {
                row.detail(html! { { format!("Step {}", i + 1) } })
            } else {
                row
            };
            if *end {
                row.end(html! {
                    <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}>{ "Edit" }</Button>
                })
            } else {
                row
            }
        })
        .collect();

    let controls = html! {
        <>
            <Select<usize> label="Rows" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Name" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<usize> label="Keys"
                options={KEY_NAMES}
                value={*keys} on_change={bind(&keys)} />
            <Toggle label="detail" value={*detail} on_change={bind(&detail)} />
            <Toggle label="end" value={*end} on_change={bind(&end)} />
            <Toggle label="divided" value={*divided} on_change={bind(&divided)} />
            <Toggle label="boxed" value={*boxed} on_change={bind(&boxed)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="children" value={*children} on_change={bind(&children)} />
            <Toggle label="quick_keys" value={*quick_keys} on_change={bind(&quick_keys)} />
            <Toggle label="titles" value={*titles} on_change={bind(&titles)} />
        </>
    };
    let reorder = El::new("Reorder")
        .expr_if(!*children, "rows", "rows.clone()")
        .expr("on_move", "on_move.clone()")
        .flag("divided", *divided && !*children)
        .flag("boxed", *boxed && !*children)
        .flag("disabled", *disabled)
        .flag("quick_keys", *quick_keys)
        .flag("titles", *titles);
    let reorder = if *children {
        reorder.text("class", class::REORDER_ITEMS).line(each(
            "steps",
            "step_item",
            rows.len(),
            "ReorderItem",
        ))
    } else {
        reorder
    };
    let code = snippet(&[], reorder);
    let lifted = (1, 3);

    html! {
        <Demo {controls} {code}>
            if rows.is_empty() {
                <EmptyState compact={true} title="Nothing to order" />
            } else if *children {
                // The app's own rows, its handle placed in them.
                <Reorder {on_move} text={text_cb} disabled={*disabled} quick_keys={*quick_keys}
                    titles={*titles} class={class::REORDER_ITEMS}>
                    { for rows.iter().map(|row| html_nested! {
                        <ReorderItem id={row.key.clone()} name={row.name.clone()} class={class::REORDER_ITEM}>
                            <ReorderHandle />
                            <strong>{ row.name.clone() }</strong>
                            if let Some(detail) = &row.detail {
                                <span>{ detail.clone() }</span>
                            }
                            { row.end.clone() }
                        </ReorderItem>
                    }) }
                </Reorder>
            } else {
                <Reorder {rows} {on_move} text={text_cb} divided={*divided} boxed={*boxed}
                    disabled={*disabled} quick_keys={*quick_keys} titles={*titles} />
            }
            <p class={class::CONTROL_LABEL}>{ format!("Said: {}", *said) }</p>
            { shown("reorder_row(3, Some((1, 3)))", &reorder_row(3, Some(lifted))) }
            { shown("drag_handle(true)", &drag_handle(true)) }
            <p class={class::CONTROL_LABEL}>
                { format!(
                    "landing(1, slot_at(middles, y)) = {}; key_target(\"End\", 1, 5) = {:?}",
                    landing(1, slot_at(&[10.0, 30.0, 50.0], 45.0)),
                    key_target("End", 1, 5),
                ) }
            </p>
            <p class={class::CONTROL_LABEL}>
                { format!(
                    "drag_target(middles, 0, top, bottom) = {}",
                    drag_target(&[10.0, 30.0, 50.0], 0, 34.0, 52.0),
                ) }
            </p>
            <DragHandle label="Move (a handle alone)" />
        </Demo>
    }
}
