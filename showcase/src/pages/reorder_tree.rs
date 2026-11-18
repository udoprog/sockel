use yew::prelude::*;

use sockel::{
    ReorderItem, ReorderList, ReorderSpot, ReorderTree, ReorderTreeHandle, ReorderTreeMove,
    TreeList, tree_slots,
};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Toggle, bind};

const GROUPS: &[(&str, &[&str])] = &[
    ("Fruit", &["Apple", "Banana", "Cherry"]),
    ("Roots", &["Beet", "Carrot"]),
    ("Empty", &[]),
];

type Groups = Vec<(AttrValue, Vec<AttrValue>)>;

/// Applies a move: a group among the groups, an entry within or between
/// groups.
fn apply(groups: &mut Groups, m: &ReorderTreeMove) {
    let (from, to) = (&m.from, &m.to);
    if from.list == "groups" {
        let group = groups.remove(from.index);
        groups.insert(to.index.min(groups.len()), group);
        return;
    }
    let find = |groups: &Groups, list: &AttrValue| groups.iter().position(|(n, _)| n == list);
    let (Some(a), Some(b)) = (find(groups, &from.list), find(groups, &to.list)) else {
        return;
    };
    let entry = groups[a].1.remove(from.index);
    let into = &mut groups[b].1;
    into.insert(to.index.min(into.len()), entry);
}

#[function_component]
pub fn ReorderTreePage() -> Html {
    let groups = use_state(|| {
        GROUPS
            .iter()
            .map(|(name, entries)| {
                (
                    AttrValue::from(*name),
                    entries.iter().map(|e| AttrValue::from(*e)).collect(),
                )
            })
            .collect::<Groups>()
    });
    let disabled = use_state(|| false);
    let titles = use_state(|| false);
    let moved = use_state(String::new);

    let on_move = {
        let groups = groups.clone();
        let moved = moved.clone();
        Callback::from(move |m: ReorderTreeMove| {
            let mut next = (*groups).clone();
            apply(&mut next, &m);
            groups.set(next);
            moved.set(format!(
                "{} {} → {} {}",
                m.from.list, m.from.index, m.to.list, m.to.index
            ));
        })
    };
    // Groups stay among the groups, entries in a group.
    let accepts =
        Callback::from(|m: ReorderTreeMove| (m.from.list == "groups") == (m.to.list == "groups"));

    let controls = html! {
        <>
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="titles" value={*titles} on_change={bind(&titles)} />
        </>
    };
    let code = snippet(
        &[],
        El::new("ReorderTree")
            .expr("on_move", "on_move.clone()")
            .expr("accepts", "Some(accepts)")
            .flag("disabled", *disabled)
            .flag("titles", *titles),
    );
    let shape = [
        TreeList {
            parent: None,
            len: 2,
        },
        TreeList {
            parent: Some((0, 0)),
            len: 1,
        },
    ];
    let spot = ReorderSpot {
        list: "groups".into(),
        index: 0,
    };

    html! {
        <Demo {controls} {code}>
            <ReorderTree {on_move} accepts={Some(accepts)} disabled={*disabled} titles={*titles}>
                <ReorderList id="groups" class={class::REORDER_ITEMS}>
                    { for groups.iter().map(|(name, entries)| html_nested! {
                        <ReorderItem id={name.clone()} name={name.clone()} class={class::REORDER_GROUP}>
                            <div class={class::REORDER_ITEM}>
                                <ReorderTreeHandle />
                                <strong>{ name.clone() }</strong>
                            </div>
                            <ReorderList id={name.clone()} class={class::REORDER_ENTRIES}>
                                { for entries.iter().map(|entry| html_nested! {
                                    <ReorderItem id={entry.clone()} name={entry.clone()} class={class::REORDER_ITEM}>
                                        <ReorderTreeHandle />
                                        <span>{ entry.clone() }</span>
                                    </ReorderItem>
                                }) }
                            </ReorderList>
                        </ReorderItem>
                    }) }
                </ReorderList>
            </ReorderTree>
            <p class={class::CONTROL_LABEL}>{ format!("Moved: {}", *moved) }</p>
            <p class={class::CONTROL_LABEL}>
                { format!("tree_slots([2 rows, 1 inside row 0]) = {:?}; {spot:?}", tree_slots(&shape)) }
            </p>
        </Demo>
    }
}
