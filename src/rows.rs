//! RowList, RowListItem and PickRow (`style/_rows.scss`).

use crate::testid::with_testid;
use crate::{button::is_empty, class};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RowListProps {
    /// Lines between the rows, drawn in the rows' own reserved borders.
    #[prop_or_default]
    pub divided: bool,
    /// A bordered, rounded surface around the rows.
    #[prop_or_default]
    pub boxed: bool,
    #[prop_or("ul")]
    pub tag: &'static str,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The rows ([`RowListItem`]s, or an empty state).
    #[prop_or_default]
    pub children: Html,
}

impl Default for RowListProps {
    fn default() -> Self {
        Self {
            divided: Default::default(),
            boxed: Default::default(),
            tag: ::yew::html::IntoPropValue::into_prop_value("ul"),
            class: Default::default(),
            testid: Default::default(),
            children: Default::default(),
        }
    }
}

/// A row list's classes: `divided` draws lines between rows, `boxed` a surface.
pub fn row_list(divided: bool, boxed: bool) -> Classes {
    classes!(
        class::ROW_LIST,
        divided.then_some(class::DIVIDED),
        boxed.then_some(class::BOXED)
    )
}

/// A column of rows. Divided, the lines are borders every row reserves, so
/// dividing the list or not moves nothing.
#[function_component]
pub fn RowList(props: &RowListProps) -> Html {
    with_testid(
        html! {
            <@{props.tag} class={classes!(row_list(props.divided, props.boxed), props.class.clone())}>
                { props.children.clone() }
            </@>
        },
        props.testid.clone(),
    )
}

#[derive(Properties, PartialEq)]
pub struct RowListItemProps {
    /// The row's name, one line, cut: a link or plain text.
    pub name: Html,
    /// A line under the name, cut.
    #[prop_or_default]
    pub detail: Option<Html>,
    /// What begins the row, before the name: a drag handle, a mark.
    #[prop_or_default]
    pub start: Option<Html>,
    /// Parts to the top, and the name wraps: a tall row.
    #[prop_or_default]
    pub top: bool,
    #[prop_or("li")]
    pub tag: &'static str,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub onpointerdown: Callback<PointerEvent>,
    /// What ends the row: an [`ActionRow`](crate::ActionRow), a checkbox.
    #[prop_or_default]
    pub children: Html,
}

impl Default for RowListItemProps {
    fn default() -> Self {
        Self {
            name: Default::default(),
            detail: Default::default(),
            start: Default::default(),
            top: Default::default(),
            tag: ::yew::html::IntoPropValue::into_prop_value("li"),
            class: Default::default(),
            testid: Default::default(),
            onpointerdown: Default::default(),
            children: Default::default(),
        }
    }
}

/// A row list item's classes: `top` for a tall row.
pub fn row_list_item(top: bool) -> Classes {
    classes!(class::ROW_LIST_ITEM, top.then_some(class::TOP))
}

/// A row of a [`RowList`]: a name and detail that take the room, then what
/// ends the row. At least `--control-height` high, as a compact empty state.
#[function_component]
pub fn RowListItem(props: &RowListItemProps) -> Html {
    with_testid(
        html! {
            <@{props.tag} class={classes!(row_list_item(props.top), props.class.clone())} onpointerdown={props.onpointerdown.clone()}>
                if let Some(start) = &props.start {
                    { start.clone() }
                }
                <div class={class::ROW_LIST_MAIN}>
                    <span class={class::ROW_LIST_NAME}>{ props.name.clone() }</span>
                    if let Some(detail) = &props.detail {
                        <span class={class::ROW_LIST_DETAIL}>{ detail.clone() }</span>
                    }
                </div>
                if !is_empty(&props.children) {
                    { props.children.clone() }
                }
            </@>
        },
        props.testid.clone(),
    )
}

#[derive(Properties, Default, PartialEq)]
pub struct PickRowProps {
    pub name: Html,
    #[prop_or_default]
    pub detail: Option<Html>,
    /// The chosen row (`aria-current`), marked by its background alone.
    #[prop_or_default]
    pub current: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// Before the name: an icon or an avatar.
    #[prop_or_default]
    pub mark: Option<Html>,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// What ends the row, after the name and detail.
    #[prop_or_default]
    pub children: Html,
}

/// A pick row's classes.
pub fn pick_row() -> Classes {
    classes!(class::PICK_ROW)
}

/// A row that is one press, choosing the thing it names. Chosen, hovered,
/// focused or disabled it keeps its box.
#[function_component]
pub fn PickRow(props: &PickRowProps) -> Html {
    with_testid(
        html! {
            <button type="button" class={classes!(pick_row(), props.class.clone())}
                aria-current={props.current.then_some("true")}
                disabled={props.disabled} onclick={props.onclick.clone()}>
                if let Some(mark) = &props.mark {
                    <span class={class::PICK_ROW_MARK} aria-hidden="true">{ mark.clone() }</span>
                }
                <span class={class::PICK_ROW_NAME}>{ props.name.clone() }</span>
                if let Some(detail) = &props.detail {
                    <span class={class::PICK_ROW_DETAIL}>{ detail.clone() }</span>
                }
                { props.children.clone() }
            </button>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn divided_and_boxed_are_modifiers() {
        assert_eq!(names(row_list(false, false)), [class::ROW_LIST]);
        assert_eq!(
            names(row_list(true, true)),
            [class::ROW_LIST, class::DIVIDED, class::BOXED]
        );
    }

    #[test]
    fn top_is_a_modifier() {
        assert_eq!(names(row_list_item(false)), [class::ROW_LIST_ITEM]);
        assert_eq!(
            names(row_list_item(true)),
            [class::ROW_LIST_ITEM, class::TOP]
        );
    }

    #[test]
    fn a_pick_row_is_its_class() {
        assert_eq!(names(pick_row()), [class::PICK_ROW]);
    }
}
