use yew::prelude::*;

use sockel::{Lane, Lanes, ReorderItem, ReorderTreeHandle, ReorderTreeMove};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const LANES: &[(&str, &[&str])] = &[
    ("Todo", &["Write the brief", "Book the room", "Order paper"]),
    ("Doing", &["Draft the plan"]),
    ("Review", &[]),
    ("Done", &["Pick a date", "Ask the team"]),
];

type Board = Vec<(AttrValue, Vec<AttrValue>)>;

fn board() -> Board {
    LANES
        .iter()
        .map(|(name, rows)| {
            (
                AttrValue::from(*name),
                rows.iter().map(|r| AttrValue::from(*r)).collect(),
            )
        })
        .collect()
}

/// Applies a move within or between lanes.
fn apply(board: &mut Board, m: &ReorderTreeMove) {
    let find = |board: &Board, id: &AttrValue| board.iter().position(|(n, _)| n == id);
    let (Some(a), Some(b)) = (find(board, &m.from.list), find(board, &m.to.list)) else {
        return;
    };
    let row = board[a].1.remove(m.from.index);
    let into = &mut board[b].1;
    into.insert(m.to.index.min(into.len()), row);
}

const ACCEPTS: &[(&str, bool)] = &[("every lane", false), ("not into Done", true)];

#[function_component]
pub fn LanesPage() -> Html {
    let board = use_state(board);
    let body_drag = use_state(|| true);
    let disabled = use_state(|| false);
    let guarded = use_state(|| false);
    let text = use_state(|| Text::Short);
    let moved = use_state(String::new);

    let controls = html! {
        <>
            <Toggle label="body_drag" value={*body_drag} on_change={bind(&body_drag)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Select<bool> label="accepts" options={ACCEPTS} value={*guarded} on_change={bind(&guarded)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <span class={class::CONTROL_LABEL}>{ (*moved).clone() }</span>
        </>
    };

    let on_move = {
        let board = board.clone();
        let moved = moved.clone();
        Callback::from(move |m: ReorderTreeMove| {
            let mut next = (*board).clone();
            apply(&mut next, &m);
            moved.set(format!(
                "moved {}:{} to {}:{}",
                m.from.list, m.from.index, m.to.list, m.to.index
            ));
            board.set(next);
        })
    };
    let accepts = guarded.then(|| Callback::from(|m: ReorderTreeMove| m.to.list != "Done"));

    let code = snippet(
        &["let board = use_state(board);".to_owned()],
        El::new("Lanes")
            .expr("on_move", "apply_to(&board)")
            .flag("body_drag", !*body_drag)
            .flag("disabled", *disabled)
            .expr_if(
                *guarded,
                "accepts",
                "Some(Callback::from(|m: ReorderTreeMove| m.to.list != \"Done\"))",
            )
            .line("{ for board.iter().map(|(id, rows)| html_nested! {")
            .line("    <Lane id={id.clone()} name={id.clone()} empty={html! { \"Nothing here\" }}>")
            .line("        { for rows.iter().map(|row| html_nested! {")
            .line("            <ReorderItem id={row.clone()} name={row.clone()}>")
            .line("                <ReorderTreeHandle /> { row.clone() }")
            .line("            </ReorderItem>")
            .line("        }) }")
            .line("    </Lane>")
            .line("}) }"),
    );

    let text = *text;
    html! {
        <Demo {controls} {code}>
            <div class={class::LANES_BOX}>
                <Lanes {on_move} {accepts} body_drag={*body_drag} disabled={*disabled}>
                    { for board.iter().map(|(id, rows)| html_nested! {
                        <Lane id={id.clone()} name={id.clone()}
                            empty={html! { { text.of("Nothing here") } }}
                            end={html! { <button type="button" class={class::BUTTON}>{ "+" }</button> }}>
                            { for rows.iter().map(|row| html_nested! {
                                <ReorderItem id={row.clone()} name={row.clone()} class={class::LANE_CARD}>
                                    <ReorderTreeHandle />
                                    <span>{ text.of(row) }</span>
                                </ReorderItem>
                            }) }
                        </Lane>
                    }) }
                </Lanes>
            </div>
        </Demo>
    }
}
