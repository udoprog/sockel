//! Lanes and Lane: lists side by side whose rows move within and between
//! them, by drag or keyboard, on [`ReorderTree`] (`style/_lanes.scss`).

use crate::testid::with_testid;
use yew::prelude::*;

use crate::{Count, ReorderItem, ReorderList, ReorderText, ReorderTree, ReorderTreeMove, class};

#[derive(Properties, PartialEq)]
pub struct LanesProps {
    /// A row moved: from which lane and place to which, as
    /// [`ReorderTree`]'s `on_move` (`to.index` counted without the row).
    pub on_move: Callback<ReorderTreeMove>,
    /// Whether a row may go to a place; every place when `None`.
    #[prop_or_default]
    pub accepts: Option<Callback<ReorderTreeMove, bool>>,
    /// The texts, overriding the catalog's (which name the lane).
    #[prop_or_default]
    pub text: Option<Callback<ReorderText, String>>,
    #[prop_or_default]
    pub disabled: bool,
    /// A mouse or pen drags a row from anywhere on it off its controls, a
    /// finger only by its handle (on by default).
    #[prop_or(true)]
    pub body_drag: bool,
    /// Each handle's name is its `data-tip` tooltip too.
    #[prop_or_default]
    pub tips: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    pub children: ChildrenWithProps<Lane>,
}

impl Default for LanesProps {
    fn default() -> Self {
        Self {
            on_move: Default::default(),
            accepts: Default::default(),
            text: Default::default(),
            disabled: Default::default(),
            body_drag: ::yew::html::IntoPropValue::into_prop_value(true),
            tips: Default::default(),
            class: Default::default(),
            testid: Default::default(),
            children: Default::default(),
        }
    }
}

/// The lanes' classes.
pub fn lanes() -> Classes {
    classes!(class::LANES)
}

/// Lists side by side, like a board's columns, whose rows can be dragged
/// within and between them. Each is a [`Lane`] of [`ReorderItem`]s, each
/// with a [`ReorderTreeHandle`](crate::ReorderTreeHandle).
///
/// From the keyboard, Space on a handle lifts the row, the arrows move it
/// (Left and Right to the next lane), Space drops it and Escape cancels.
/// On a narrow screen each lane takes nearly the full width and snaps into
/// view. Nothing reflows until the drop.
#[function_component]
pub fn Lanes(props: &LanesProps) -> Html {
    html! {
        <ReorderTree class={classes!(lanes(), props.class.clone())} testid={props.testid.clone()}
            on_move={props.on_move.clone()} accepts={props.accepts.clone()} text={props.text.clone()}
            disabled={props.disabled} tips={props.tips} body_drag={props.body_drag} side_by_side=true>
            <div class={classes!(class::LANES_TRACK, class::SCROLL_AREA)}>
                { for props.children.iter() }
            </div>
        </ReorderTree>
    }
}

#[derive(Properties, Default, PartialEq)]
pub struct LaneProps {
    /// The lane in the moves, unique among the lanes.
    pub id: AttrValue,
    /// What the lane is called: its head, its name for assistive
    /// technology and in the announcements.
    pub name: AttrValue,
    /// The count in the head; the rows' when `None`.
    #[prop_or_default]
    pub count: Option<u32>,
    /// What ends the head: a menu, an add button.
    #[prop_or_default]
    pub end: Html,
    /// What shows while the lane has no rows (over its drop area, so a
    /// row can still be dropped there).
    #[prop_or_default]
    pub empty: Html,
    /// What follows the rows: an add button, a "more" link.
    #[prop_or_default]
    pub foot: Html,
    /// The lane's own content is current (`aria-current`), as a board's
    /// column a filter picked.
    #[prop_or_default]
    pub current: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: ChildrenWithProps<ReorderItem>,
}

/// One lane of [`Lanes`]: a head of its name, count and `end`, held at the
/// top while the lanes scroll, over its rows (the drop area, as tall as the
/// tallest lane) and its `foot`.
#[function_component]
pub fn Lane(props: &LaneProps) -> Html {
    let len = props.children.len();
    let count = props
        .count
        .unwrap_or_else(|| u32::try_from(len).unwrap_or(u32::MAX));
    with_testid(
        html! {
            <section class={classes!(class::LANE, props.class.clone())} aria-label={props.name.clone()}
                aria-current={props.current.then_some("true")}>
                <header class={class::LANE_HEAD}>
                    <span class={class::LANE_TITLE}>{ props.name.clone() }</span>
                    <Count class={class::LANE_COUNT} value={count} />
                    <span class={class::LANE_END}>{ props.end.clone() }</span>
                </header>
                <div class={class::LANE_BODY}>
                    <ReorderList id={props.id.clone()} name={props.name.clone()} class={class::LANE_ROWS}>
                        { for props.children.iter() }
                    </ReorderList>
                    if len == 0 {
                        <div class={class::LANE_EMPTY}>{ props.empty.clone() }</div>
                    }
                </div>
                <div class={class::LANE_FOOT}>{ props.foot.clone() }</div>
            </section>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Properties, PartialEq)]
    struct BoardProps {
        rows: usize,
    }

    #[function_component]
    fn Board(props: &BoardProps) -> Html {
        html! {
            <Lanes on_move={Callback::noop()}>
                <Lane id="a" name="Todo">
                    { for (0..props.rows).map(|n| html_nested! {
                        <ReorderItem id={format!("r{n}")} name={format!("Row {n}")}>{ "row" }</ReorderItem>
                    }) }
                </Lane>
                <Lane id="b" name="Done" count={Some(7)} empty={html! { "none" }} />
            </Lanes>
        }
    }

    fn render(rows: usize) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Board>::with_props(BoardProps { rows }).render(),
        )
    }

    #[test]
    fn each_lane_is_a_named_list_with_its_count() {
        let html = render(2);
        assert!(html.contains(r#"data-reorder-list="a""#), "{html}");
        assert!(html.contains(r#"data-reorder-list-name="Done""#), "{html}");
        assert!(html.contains(">2<") && html.contains(">7<"), "{html}");
    }

    #[test]
    fn only_an_empty_lane_shows_its_empty_state() {
        let html = render(1);
        assert_eq!(html.matches(class::LANE_EMPTY).count(), 1, "{html}");
        assert!(html.contains("none"), "{html}");
    }

    #[test]
    fn a_body_drag_marks_every_row_as_grabbable() {
        let html = render(2);
        assert_eq!(html.matches(class::REORDER_GRAB).count(), 2, "{html}");
    }
}
