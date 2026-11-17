//! Table, TableRow and TableCell (`style/_table.scss`).

use crate::class;
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

/// A table's classes: `fixed` for columns sized by the first row.
pub fn table(fixed: bool) -> Classes {
    classes!(class::TABLE, fixed.then_some(class::FIXED))
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

    html! {
        <div class={classes!(class::TABLE_FRAME, props.class.clone())} data-testid={props.testid.clone()}>
            if let Some(bulk) = &props.bulk {
                <div class={table_bulk(!props.bulk_active)} role="status">{ bulk.clone() }</div>
            }
            <div class={table_wrap(props.sticky)}>
                <table class={table(props.fixed)} aria-label={props.label.clone()}>
                    <thead><tr>{ props.head.clone() }</tr></thead>
                    <tbody>{ rows }</tbody>
                </table>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TableRowProps {
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
    html! {
        <tr class={classes!(class::TABLE_ROW, props.class.clone())} data-testid={props.testid.clone()}>
            { props.children.clone() }
        </tr>
    }
}

#[derive(Properties, PartialEq)]
pub struct TableCellProps {
    /// A heading cell (`th`).
    #[prop_or_default]
    pub head: bool,
    /// Figures of one width, to the end.
    #[prop_or_default]
    pub numeric: bool,
    /// One line, cut with an ellipsis; only holds a width in a `fixed` table.
    #[prop_or_default]
    pub cut: bool,
    /// The column's width (any CSS length), for a `fixed` table's first row.
    #[prop_or_default]
    pub width: Option<AttrValue>,
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

/// A cell of a row, or a heading (`head`) of the table's head.
#[function_component]
pub fn TableCell(props: &TableCellProps) -> Html {
    let class = classes!(
        table_cell(props.head, props.numeric, props.cut),
        props.class.clone()
    );
    let style = props.width.as_ref().map(|w| format!("width: {w}"));

    if props.head {
        html! {
            <th {class} {style} scope="col" data-testid={props.testid.clone()}>{ props.children.clone() }</th>
        }
    } else {
        html! {
            <td {class} {style} data-testid={props.testid.clone()}>{ props.children.clone() }</td>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
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
    fn a_cell_is_numeric_or_cut_by_modifier() {
        assert_eq!(names(table_cell(false, false, false)), [class::TABLE_CELL]);
        assert_eq!(
            names(table_cell(true, true, true)),
            [class::TABLE_HEAD_CELL, class::NUMERIC, class::TRUNCATE]
        );
    }
}
