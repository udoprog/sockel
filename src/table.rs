//! Table, TableRow and TableCell (`style/_table.scss`).

use crate::class;
use crate::testid::with_testid;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TableProps {
    /// The heading cells ([`TableCell`] with `head`, or a [`SortHeader`](crate::SortHeader)).
    pub head: Html,
    /// How many columns there are, for the empty row to span.
    #[prop_or(1)]
    pub columns: usize,
    /// Sizes the columns by the first row alone, so cells are cut (`cut`)
    /// instead of widening a column as data arrives or a sort changes.
    #[prop_or_default]
    pub fixed: bool,
    /// Holds the head while the rows scroll, past `--table-max-height`.
    #[prop_or_default]
    pub sticky: bool,
    /// Below the phone breakpoint a row is a card: the head is for a screen
    /// reader only, the first cell on a line of its own (with the `actions`
    /// cell at its end, if any), empty cells gone.
    #[prop_or_default]
    pub cards: bool,
    /// With `cards`, the `actions` cell goes on a line under the row instead.
    #[prop_or_default]
    pub foot: bool,
    /// With `cards`, each cell after the first is a field, its label over its
    /// value, two to a line.
    #[prop_or_default]
    pub stacked: bool,
    /// Tops the cells, for rows whose text runs on.
    #[prop_or_default]
    pub top: bool,
    /// Draws the table as a card: bordered, rounded and raised on the
    /// surface, a faint band behind the headings, the last row's line the
    /// frame's own.
    #[prop_or_default]
    pub framed: bool,
    /// The bulk actions above the table. Given, its room is always kept.
    #[prop_or_default]
    pub bulk: Option<Html>,
    /// Shows the bulk actions; otherwise they are hidden and keep their room.
    #[prop_or_default]
    pub bulk_active: bool,
    /// Stands in the rows' place: an empty state (compact) or an error.
    #[prop_or_default]
    pub empty: Option<Html>,
    /// The table's name for a screen reader.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The rows ([`TableRow`]s).
    #[prop_or_default]
    pub children: Html,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            head: Default::default(),
            columns: ::yew::html::IntoPropValue::into_prop_value(1),
            fixed: Default::default(),
            sticky: Default::default(),
            cards: Default::default(),
            foot: Default::default(),
            stacked: Default::default(),
            top: Default::default(),
            framed: Default::default(),
            bulk: Default::default(),
            bulk_active: Default::default(),
            empty: Default::default(),
            label: Default::default(),
            class: Default::default(),
            testid: Default::default(),
            children: Default::default(),
        }
    }
}

/// A table's classes: `fixed` for columns sized by the first row.
pub fn table(fixed: bool) -> Classes {
    classes!(class::TABLE, fixed.then_some(class::FIXED))
}

/// A table's mode classes: `cards` (phone cards), `foot` (their actions under
/// the row), `top` (cells at the top).
pub fn table_mode(cards: bool, foot: bool, top: bool) -> Classes {
    classes!(
        cards.then_some(class::CARDS),
        foot.then_some(class::FOOT),
        top.then_some(class::TOP)
    )
}

/// A table frame's classes: `framed` draws it as a card.
pub fn table_frame(framed: bool) -> Classes {
    classes!(class::TABLE_FRAME, framed.then_some(class::FRAMED))
}

/// A table wrap's classes: a scroll area, `sticky` capped in height.
pub fn table_wrap(sticky: bool) -> Classes {
    classes!(class::TABLE_WRAP, sticky.then_some(class::STICKY))
}

/// A bulk bar's classes: `idle` while hidden.
pub fn table_bulk(idle: bool) -> Classes {
    classes!(class::TABLE_BULK, idle.then_some(class::IDLE))
}

/// A table in a scroll area that keeps its scrollbar's room, with a bulk
/// actions slot reserved above it. Rows are at least `--control-height`
/// high and every line is a reserved border, so hover, the bulk bar and
/// sorting move no row; `fixed` also keeps the columns.
#[function_component]
pub fn Table(props: &TableProps) -> Html {
    let rows = match &props.empty {
        Some(empty) => html! {
            <tr><td colspan={props.columns.to_string()} class={class::TABLE_EMPTY}>{ empty.clone() }</td></tr>
        },
        None => props.children.clone(),
    };

    with_testid(
        html! {
            <div class={classes!(table_frame(props.framed), props.class.clone())}>
                if let Some(bulk) = &props.bulk {
                    <div class={table_bulk(!props.bulk_active)} role="status">{ bulk.clone() }</div>
                }
                <div class={table_wrap(props.sticky)}>
                    <table class={classes!(table(props.fixed), table_mode(props.cards, props.foot, props.top), props.stacked.then_some(class::STACKED))} aria-label={props.label.clone()}>
                        <thead><tr>{ props.head.clone() }</tr></thead>
                        <tbody>{ rows }</tbody>
                    </table>
                </div>
            </div>
        },
        props.testid.clone(),
    )
}

#[derive(Properties, Default, PartialEq)]
pub struct TableRowProps {
    /// Marks the row's start edge in the danger colour.
    #[prop_or_default]
    pub flagged: bool,
    /// What the row stands for (`data-value`), for a test or script to find it by.
    #[prop_or_default]
    pub value: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The row's [`TableCell`]s.
    #[prop_or_default]
    pub children: Html,
}

/// A table row, tinted on hover and focus.
#[function_component]
pub fn TableRow(props: &TableRowProps) -> Html {
    with_testid(
        html! {
            <tr class={classes!(
                class::TABLE_ROW,
                props.flagged.then_some(class::FLAGGED),
                props.class.clone()
            )} data-value={props.value.clone()}>
                { props.children.clone() }
            </tr>
        },
        props.testid.clone(),
    )
}

#[derive(Properties, Default, PartialEq)]
pub struct TableCellProps {
    /// A heading cell (`th`).
    #[prop_or_default]
    pub head: bool,
    /// Figures of one width, to the end.
    #[prop_or_default]
    pub numeric: bool,
    /// Wraps its text, with room for it.
    #[prop_or_default]
    pub wrap: bool,
    /// A row's heading, in bold: a card's first line (with `cards`).
    #[prop_or_default]
    pub title: bool,
    /// A row's actions, to the end; at the end of a card's first line (`cards`).
    #[prop_or_default]
    pub actions: bool,
    /// Takes its card's whole line (with `cards`), as a chart needs.
    #[prop_or_default]
    pub full: bool,
    /// A heading for a screen reader only.
    #[prop_or_default]
    pub hidden: bool,
    /// The cells it spans.
    #[prop_or_default]
    pub colspan: Option<usize>,
    /// What a heading heads (`col` by default). Given on a cell that is not a
    /// `head`, it is a row's heading: a `th` with a cell's look (`row`,
    /// `colgroup`).
    #[prop_or_default]
    pub scope: Option<AttrValue>,
    /// One line, cut with an ellipsis; only holds a width in a `fixed` table.
    #[prop_or_default]
    pub cut: bool,
    /// The column's width (any CSS length), for a `fixed` table's first row.
    #[prop_or_default]
    pub width: Option<AttrValue>,
    /// Its tooltip, shown by [`DataTips`](crate::DataTips) (`data-tip`).
    #[prop_or_default]
    pub tip: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A table cell's classes.
pub fn table_cell(head: bool, numeric: bool, cut: bool) -> Classes {
    classes!(
        if head {
            class::TABLE_HEAD_CELL
        } else {
            class::TABLE_CELL
        },
        numeric.then_some(class::NUMERIC),
        cut.then_some(class::TRUNCATE)
    )
}

/// A cell's extra classes: `wrap`, `title`, `actions`, `full` (a card's whole
/// line), and `hidden` (a heading for a screen reader only).
pub fn table_cell_part(
    wrap: bool,
    title: bool,
    actions: bool,
    full: bool,
    hidden: bool,
) -> Classes {
    classes!(
        wrap.then_some(class::WRAP),
        title.then_some(class::TITLE),
        actions.then_some(class::ACTIONS),
        full.then_some(class::FULL),
        hidden.then_some(class::VISUALLY_HIDDEN)
    )
}

/// A cell of a row, or a heading (`head`) of the table's head.
#[function_component]
pub fn TableCell(props: &TableCellProps) -> Html {
    let class = classes!(
        table_cell(props.head, props.numeric, props.cut),
        table_cell_part(
            props.wrap,
            props.title,
            props.actions,
            props.full,
            props.hidden
        ),
        props.class.clone()
    );
    let style = props.width.as_ref().map(|w| format!("width: {w}"));

    let colspan = props.colspan.map(|n| n.to_string());

    if props.head {
        let scope = props.scope.clone().unwrap_or(AttrValue::Static("col"));
        with_testid(
            html! {
                <th {class} {style} {scope} {colspan} data-tip={props.tip.clone()}>{ props.children.clone() }</th>
            },
            props.testid.clone(),
        )
    } else if let Some(scope) = props.scope.clone() {
        with_testid(
            html! {
                <th {class} {style} {scope} {colspan} data-tip={props.tip.clone()}>{ props.children.clone() }</th>
            },
            props.testid.clone(),
        )
    } else {
        with_testid(
            html! {
                <td {class} {style} {colspan} data-tip={props.tip.clone()}>{ props.children.clone() }</td>
            },
            props.testid.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn frame_framed() {
        assert_eq!(names(table_frame(false)), [class::TABLE_FRAME]);
        assert_eq!(
            names(table_frame(true)),
            [class::TABLE_FRAME, class::FRAMED]
        );
    }

    #[test]
    fn fixed_is_a_modifier() {
        assert_eq!(names(table(false)), [class::TABLE]);
        assert_eq!(names(table(true)), [class::TABLE, class::FIXED]);
    }

    #[test]
    fn sticky_is_a_modifier() {
        assert_eq!(names(table_wrap(false)), [class::TABLE_WRAP]);
        assert_eq!(names(table_wrap(true)), [class::TABLE_WRAP, class::STICKY]);
    }

    #[test]
    fn an_idle_bulk_bar_keeps_its_room() {
        assert_eq!(names(table_bulk(false)), [class::TABLE_BULK]);
        assert_eq!(names(table_bulk(true)), [class::TABLE_BULK, class::IDLE]);
    }

    #[test]
    fn modes_are_modifiers() {
        assert!(names(table_mode(false, false, false)).is_empty());
        assert_eq!(
            names(table_mode(true, true, true)),
            [class::CARDS, class::FOOT, class::TOP]
        );
    }

    #[test]
    fn a_cell_has_its_parts_by_modifier() {
        assert!(names(table_cell_part(false, false, false, false, false)).is_empty());
        assert_eq!(
            names(table_cell_part(true, true, true, true, true)),
            [
                class::WRAP,
                class::TITLE,
                class::ACTIONS,
                class::FULL,
                class::VISUALLY_HIDDEN
            ]
        );
        assert_eq!(
            names(table_cell_part(false, false, false, true, false)),
            [class::FULL]
        );
    }

    #[test]
    fn a_cell_is_numeric_or_cut_by_modifier() {
        assert_eq!(names(table_cell(false, false, false)), [class::TABLE_CELL]);
        assert_eq!(
            names(table_cell(true, true, true)),
            [class::TABLE_HEAD_CELL, class::NUMERIC, class::TRUNCATE]
        );
    }
}
