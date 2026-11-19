use phrasebook02::{Args, Phrases, yew023::use_phrases_with};
use wasm_bindgen::JsCast;
use web_sys::{Element, KeyboardEvent, KeyboardEventInit};
use yew::prelude::*;

use sockel::{
    Button, ButtonSize, ButtonVariant, DragHandle, EmptyState, Reorder, ReorderHandle, ReorderItem,
    ReorderRow, ReorderText, drag_handle, drag_target, edge_step, grid_target, key_target, landing,
    move_item, move_items, reorder_row, reorder_text, slot_at,
};

use crate::class;
use crate::code::{El, each, snippet};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const NAMES: &[&str] = &[
    "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel",
];

/// What `reorder_text` says, in the provided language: the text callback
/// overrides sockel's own words, so the page looks up the same messages.
fn say(phrases: &Phrases, text: &ReorderText) -> String {
    let (id, name, place) = match text {
        ReorderText::Handle { name } => ("sockel-reorder-handle", Some(name), None),
        ReorderText::Hint => ("sockel-reorder-hint", None, None),
        ReorderText::Lifted {
            name,
            position,
            count,
        } => (
            "sockel-reorder-lifted",
            Some(name),
            Some((*position, *count)),
        ),
        ReorderText::Moving {
            name,
            position,
            count,
        } => (
            "sockel-reorder-moving",
            Some(name),
            Some((*position, *count)),
        ),
        ReorderText::Dropped {
            name,
            position,
            count,
        } => (
            "sockel-reorder-dropped",
            Some(name),
            Some((*position, *count)),
        ),
        ReorderText::Cancelled {
            name,
            position,
            count,
        } => (
            "sockel-reorder-cancelled",
            Some(name),
            Some((*position, *count)),
        ),
        // Kinds this page does not translate: sockel's own words.
        _ => return reorder_text(text),
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

/// The fixed heading's place in the order, among the names' indices.
const HEADING: usize = usize::MAX;

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
    let lists = doc.get_elements_by_class_name(sockel::class::REORDER);
    for l in 0..lists.length() {
        let Some(handle) = lists.item(l).and_then(|list| {
            list.get_elements_by_class_name(sockel::class::DRAG_HANDLE)
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
    let body_drag = use_state(|| false);
    let auto_scroll = use_state(|| false);
    let grid = use_state(|| false);
    let picked = use_state(|| false);
    let fixed = use_state(|| false);
    let keys = use_state(|| 0usize);
    // The heading sits after the second name until moved round.
    let order = use_state(|| {
        let mut order = (0..NAMES.len()).collect::<Vec<_>>();
        order.insert(2, HEADING);
        order
    });
    // The entries shown, by index into the order: names within the count,
    // and the heading when `fixed`; moves are indices among these.
    let shown_at: Vec<usize> = order
        .iter()
        .enumerate()
        .filter(|(_, i)| if **i == HEADING { *fixed } else { **i < *count })
        .map(|(at, _)| at)
        .collect();
    let said = use_state(String::new);

    {
        let pressed = KEYS[*keys];
        use_effect_with(*keys, move |_| press(pressed));
    }

    let on_move = {
        let order = order.clone();
        let shown_at = shown_at.clone();
        Callback::from(move |(from, to): (usize, usize)| {
            let mut shown: Vec<usize> = shown_at.iter().map(|at| order[*at]).collect();
            move_item(&mut shown, from, to);
            order.set(reshown(&order, &shown_at, shown));
        })
    };
    // Bravo and Delta picked: lifting either carries the other.
    let on_move_picked = {
        let order = order.clone();
        let shown_at = shown_at.clone();
        Callback::from(move |(rows, to): (Vec<usize>, usize)| {
            let mut shown: Vec<usize> = shown_at.iter().map(|at| order[*at]).collect();
            move_items(&mut shown, &rows, to);
            order.set(reshown(&order, &shown_at, shown));
        })
    };
    let phrases = use_phrases_with(&sockel::phrasebookcatalog01::L10N);
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
    // A grid is the children mode's: the rows are tiles the page lays out,
    // as are rows picked together.
    // Fixed rows are too.
    let as_items = *children || *grid || *picked || *fixed;
    let picks = |name: &str| *picked && matches!(name, "Bravo" | "Delta");
    let rows: Vec<ReorderRow> = shown_at
        .iter()
        .map(|at| order[*at])
        .map(|i| {
            if i == HEADING {
                return ReorderRow::new("Later", "Later");
            }
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
            <Toggle label="body_drag" value={*body_drag} on_change={bind(&body_drag)} />
            <Toggle label="auto_scroll" value={*auto_scroll} on_change={bind(&auto_scroll)} />
            <Toggle label="grid" value={*grid} on_change={bind(&grid)} />
            <Toggle label="picked" value={*picked} on_change={bind(&picked)} />
            <Toggle label="fixed" value={*fixed} on_change={bind(&fixed)} />
        </>
    };
    let grid_on = *grid;
    let items_class = if *grid {
        class::REORDER_TILES
    } else {
        class::REORDER_ITEMS
    };
    let reorder = El::new("Reorder")
        .expr_if(!as_items, "rows", "rows.clone()")
        .expr("on_move", "on_move.clone()")
        .expr_if(*picked, "on_move_picked", "on_move_picked.clone()")
        .flag("divided", *divided && !as_items)
        .flag("boxed", *boxed && !as_items)
        .flag("disabled", *disabled)
        .flag("quick_keys", *quick_keys)
        .flag("titles", *titles)
        .flag("body_drag", *body_drag)
        .flag("auto_scroll", *auto_scroll)
        .flag("grid", *grid);
    let reorder = if as_items {
        reorder
            .text("class", items_class)
            .line_if(
                *fixed,
                "<ReorderItem id=\"later\" name=\"Later\" fixed=true><strong>{ \"Later\" }</strong></ReorderItem> // among the steps",
            )
            .line(each(
                "steps",
                "step_item",
                rows.len() - usize::from(*fixed && !rows.is_empty()),
                "ReorderItem",
            ))
        .line_if(*fixed, "<ReorderItem id=\"end\" name=\"End\" fixed=true lifted_only=true>{ \"Drop here\" }</ReorderItem>")
    } else {
        reorder
    };
    let code = snippet(&[], reorder);
    let lifted = (1, 3);
    let scroll_box = classes!(auto_scroll.then_some(class::REORDER_SCROLL));

    let mut items: Vec<_> = rows
        .iter()
        .map(|row| {
            if row.key == "Later" {
                return html_nested! {
                    <ReorderItem id="later" name="Later" fixed=true class={class::REORDER_ITEM}>
                        <strong>{ "Later" }</strong>
                    </ReorderItem>
                };
            }
            html_nested! {
                <ReorderItem id={row.key.clone()} name={row.name.clone()} picked={picks(&row.key)}
                    class={classes!(class::REORDER_ITEM, grid_on.then_some(class::REORDER_TILE))}>
                    <ReorderHandle />
                    <strong>{ row.name.clone() }</strong>
                    if let Some(detail) = &row.detail {
                        <span>{ detail.clone() }</span>
                    }
                    { row.end.clone() }
                </ReorderItem>
            }
        })
        .collect();
    if *fixed {
        items.push(html_nested! {
            <ReorderItem id="end" name="End" fixed=true lifted_only=true class={class::REORDER_ITEM}>
                { "Drop here" }
            </ReorderItem>
        });
    }

    html! {
        <Demo {controls} {code}>
            if rows.iter().all(|row| row.key == "Later") {
                <EmptyState compact={true} title="Nothing to order" />
            } else if as_items {
                // The app's own rows, its handle placed in them.
                <div class={scroll_box}>
                <Reorder {on_move} {on_move_picked} text={text_cb} disabled={*disabled} quick_keys={*quick_keys}
                    titles={*titles} body_drag={*body_drag} auto_scroll={*auto_scroll} grid={*grid}
                    class={items_class}>
                    { for items }
                </Reorder>
                </div>
            } else {
                <div class={scroll_box}>
                <Reorder {rows} {on_move} text={text_cb} divided={*divided} boxed={*boxed}
                    disabled={*disabled} quick_keys={*quick_keys} titles={*titles}
                    body_drag={*body_drag} auto_scroll={*auto_scroll} />
                </div>
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
            <p class={class::CONTROL_LABEL}>
                { format!(
                    "grid_target(centers, 0, x, y) = {}; edge_step(y, top, bottom) = {}",
                    grid_target(&[(10.0, 10.0), (30.0, 10.0), (10.0, 30.0)], 0, 12.0, 28.0),
                    edge_step(105.0, 100.0, 500.0),
                ) }
            </p>
            <DragHandle label="Move (a handle alone)" />
        </Demo>
    }
}

/// The order with the entries shown at `shown_at` replaced by `shown`, in turn.
fn reshown(order: &[usize], shown_at: &[usize], shown: Vec<usize>) -> Vec<usize> {
    let mut next = order.to_vec();
    for (at, i) in shown_at.iter().zip(shown) {
        next[*at] = i;
    }
    next
}
