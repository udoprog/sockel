use yew::prelude::*;

use crate::class;

use sockel::{
    Button, ButtonSize, ButtonVariant, EmptyState, ErrorBox, Glyph, Icon, SortDirection,
    SortHeader, Table, TableCell, TableRow, next_sort, sort_header, sort_mark, table, table_bulk,
    table_cell, table_cell_part, table_mode, table_wrap,
};

use crate::code::{El, each, snippet, text_node};
use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind};

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

/// A column of the demo's data.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Column {
    Name,
    Size,
}

type Sort = Option<(Column, SortDirection)>;

const SORTS: &[(&str, Sort)] = &[
    ("None", None),
    (
        "Name, ascending",
        Some((Column::Name, SortDirection::Ascending)),
    ),
    (
        "Name, descending",
        Some((Column::Name, SortDirection::Descending)),
    ),
    (
        "Size, ascending",
        Some((Column::Size, SortDirection::Ascending)),
    ),
    (
        "Size, descending",
        Some((Column::Size, SortDirection::Descending)),
    ),
];

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

const KINDS: [&str; 3] = ["Track", "Mix", "Project"];

/// Rows of (name, kind, size), unsorted.
fn data(count: usize, name: Text) -> Vec<(String, &'static str, u32)> {
    (1..=count)
        .map(|n| {
            let size = ((n * 7919) % 997) as u32 * 13;
            (
                name.of(&format!("Item {}", (n * 5) % 9 + n)),
                KINDS[n % 3],
                size,
            )
        })
        .collect()
}

fn sorted(
    mut rows: Vec<(String, &'static str, u32)>,
    sort: Sort,
) -> Vec<(String, &'static str, u32)> {
    if let Some((column, direction)) = sort {
        rows.sort_by(|a, b| match column {
            Column::Name => a.0.cmp(&b.0),
            Column::Size => a.2.cmp(&b.2),
        });
        if direction == SortDirection::Descending {
            rows.reverse();
        }
    }
    rows
}

fn direction_of(sort: Sort, column: Column) -> Option<SortDirection> {
    sort.filter(|(c, _)| *c == column).map(|(_, d)| d)
}

/// A press on `column`'s heading sets the sort to the direction it asks for.
fn press(sort: &UseStateHandle<Sort>, column: Column) -> Callback<Option<SortDirection>> {
    let sort = sort.clone();
    Callback::from(move |next: Option<SortDirection>| sort.set(next.map(|d| (column, d))))
}

#[function_component]
pub fn TablePage() -> Html {
    let count = use_state(|| 3);
    let shown_state = use_state(|| Shown::Rows);
    let name = use_state(|| Text::Short);
    let fixed = use_state(|| true);
    let sticky = use_state(|| false);
    let bulk = use_state(|| true);
    let picked = use_state(|| false);
    let disabled = use_state(|| false);
    let cards = use_state(|| true);
    let foot = use_state(|| false);
    let top = use_state(|| false);
    let flagged = use_state(|| true);
    let sort = use_state(|| None::<(Column, SortDirection)>);

    let controls = html! {
        <>
            <Select<usize> label="Rows" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Shown> label="State" options={SHOWN} value={*shown_state} on_change={bind(&shown_state)} />
            <Select<Text> label="Name" options={Text::OPTIONS} value={*name} on_change={bind(&name)} />
            <Select<Sort> label="Sort" options={SORTS} value={*sort} on_change={bind(&sort)} />
            <Toggle label="fixed" value={*fixed} on_change={bind(&fixed)} />
            <Toggle label="sticky" value={*sticky} on_change={bind(&sticky)} />
            <Toggle label="bulk" value={*bulk} on_change={bind(&bulk)} />
            <Toggle label="Picked" value={*picked} on_change={bind(&picked)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="cards" value={*cards} on_change={bind(&cards)} />
            <Toggle label="foot" value={*foot} on_change={bind(&foot)} />
            <Toggle label="top" value={*top} on_change={bind(&top)} />
            <Toggle label="flagged" value={*flagged} on_change={bind(&flagged)} />
        </>
    };

    let head = html! {
        <>
            <SortHeader label="Name" direction={direction_of(*sort, Column::Name)}
                disabled={*disabled} onsort={press(&sort, Column::Name)} />
            <TableCell head=true>{ "Kind" }</TableCell>
            <SortHeader label="Size" numeric=true first={SortDirection::Descending}
                direction={direction_of(*sort, Column::Size)} disabled={*disabled}
                onsort={press(&sort, Column::Size)} />
            <TableCell head=true hidden=true>{ "Actions" }</TableCell>
        </>
    };

    let rows: Html = sorted(data(*count, *name), *sort)
        .into_iter()
        .enumerate()
        .map(|(n, (name, kind, size))| {
            html! {
                <TableRow flagged={*flagged && n == 1}>
                    <TableCell cut=true title=true>{ name }</TableCell>
                    <TableCell cut=true>{ kind }</TableCell>
                    <TableCell numeric=true>{ size }</TableCell>
                    <TableCell actions=true width="6rem">
                        <Button size={ButtonSize::Small}>{ "Open" }</Button>
                    </TableCell>
                </TableRow>
            }
        })
        .collect();

    let empty = match *shown_state {
        Shown::Rows if *count > 0 => None,
        Shown::Error => Some(html! {
            <ErrorBox title="Could not load the rows" cause="TypeError: failed to fetch">
                { "The table is not available right now" }
            </ErrorBox>
        }),
        _ => Some(html! {
            <EmptyState compact=true title="No rows yet" icon={html! { <Icon glyph={Glyph::Search} /> }} />
        }),
    };

    let bulk_bar = bulk.then(|| {
        html! {
            <>
                <span class={class::CONTROL_LABEL}>{ if *picked { "2 selected" } else { "" } }</span>
                <Button size={ButtonSize::Small} variant={ButtonVariant::Danger}>{ "Delete" }</Button>
            </>
        }
    });

    let code = snippet(
        &[],
        El::new("Table")
            .expr("head", "html! { <>{ headings }</> }")
            .expr_if(*count == 0 || *shown_state != Shown::Rows, "columns", "4")
            .expr_if(
                *count == 0 || *shown_state != Shown::Rows,
                "empty",
                "Some(html! { <EmptyState compact=true title=\"No rows yet\" /> })",
            )
            .flag("fixed", *fixed)
            .flag("sticky", *sticky)
            .flag("cards", *cards)
            .flag("foot", *foot)
            .flag("top", *top)
            .expr_if(*bulk, "bulk", "Some(html! { { bulk_actions } })")
            .flag("bulk_active", *bulk && *picked)
            .child(
                El::new("TableRow")
                    .flag("flagged", *flagged)
                    .line(format!(
                        "<TableCell cut=true title=true>{}</TableCell>",
                        text_node(&name.of("Item 1"))
                    ))
                    .line("<TableCell numeric=true>{ size }</TableCell>")
                    .line("<TableCell actions=true>{ actions }</TableCell>"),
            )
            .line(each("rows", "row", *count, "row")),
    );

    html! {
        <Demo {controls} {code}>
            <Table head={head} columns={4} fixed={*fixed} sticky={*sticky}
                cards={*cards} foot={*foot} top={*top}
                bulk={bulk_bar} bulk_active={*bulk && *picked} {empty} label="Items">
                { rows }
            </Table>
            <span class={class::CONTROL_LABEL}>{ "Press a heading to sort: ascending, descending, then back." }</span>
            { shown("table(fixed)", &table(*fixed)) }
            { shown("table_wrap(sticky)", &table_wrap(*sticky)) }
            { shown("table_bulk(idle)", &table_bulk(!*picked)) }
            { shown("table_cell(head, numeric, cut)", &table_cell(false, true, true)) }
            { shown("table_mode(cards, foot, top)", &table_mode(*cards, *foot, *top)) }
            { shown("table_cell_part(wrap, title, actions, hidden)", &table_cell_part(false, true, true, false)) }
        </Demo>
    }
}

#[function_component]
pub fn SortHeaderPage() -> Html {
    let label = use_state(|| Text::Short);
    let sort = use_state(|| None::<(Column, SortDirection)>);
    let disabled = use_state(|| false);
    let numeric = use_state(|| true);
    let descending_first = use_state(|| false);

    let controls = html! {
        <>
            <Select<Text> label="Label" options={Text::OPTIONS} value={*label} on_change={bind(&label)} />
            <Select<Sort> label="Sort" options={SORTS} value={*sort} on_change={bind(&sort)} />
            <Toggle label="numeric" value={*numeric} on_change={bind(&numeric)} />
            <Toggle label="Descending first" value={*descending_first} on_change={bind(&descending_first)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let first = if *descending_first {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };
    let text = label.of("Name");
    let label_or = |fallback: &str| {
        if text.is_empty() {
            fallback.to_owned()
        } else {
            text.clone()
        }
    };

    let code = snippet(
        &[],
        El::new("SortHeader")
            .text("label", &label_or("Name"))
            .variant("direction", direction_of(*sort, Column::Name))
            .variant("first", first)
            .flag("numeric", *numeric)
            .flag("disabled", *disabled)
            .expr("onsort", "onsort"),
    );

    html! {
        <Demo {controls} {code}>
            <Table head={html! {
                <>
                    <SortHeader label={label_or("Name")} numeric={*numeric} {first} disabled={*disabled}
                        direction={direction_of(*sort, Column::Name)}
                        onsort={press(&sort, Column::Name)} />
                    <SortHeader label="Size" numeric=true disabled={*disabled}
                        direction={direction_of(*sort, Column::Size)}
                        onsort={press(&sort, Column::Size)} />
                </>
            }} columns={2} fixed=true label="Sort headings">
                <TableRow>
                    <TableCell cut=true>{ "Item 1" }</TableCell>
                    <TableCell numeric=true>{ "1024" }</TableCell>
                </TableRow>
            </Table>
            <span class={class::CONTROL_LABEL}>{ format!("Next press on Name: {}", match next_sort(direction_of(*sort, Column::Name), first) {
                Some(d) => d.aria(),
                None => "none",
            }) }</span>
            { shown("sort_header(numeric)", &sort_header(*numeric)) }
            { shown("sort_mark(idle)", &sort_mark(direction_of(*sort, Column::Name).is_none())) }
        </Demo>
    }
}
