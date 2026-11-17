//! SortHeader (`style/_table.scss`).

use crate::{Glyph, Icon, class};
use yew::prelude::*;

/// Which way a column runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    /// The `aria-sort` value.
    pub fn aria(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }

    /// The other way.
    pub fn reverse(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// What a press on a heading sets: unsorted goes to `first`, `first` to the
/// other way, and that back to unsorted.
pub fn next_sort(current: Option<SortDirection>, first: SortDirection) -> Option<SortDirection> {
    match current {
        None => Some(first),
        Some(d) if d == first => Some(first.reverse()),
        Some(_) => None,
    }
}

#[derive(Properties, PartialEq)]
pub struct SortHeaderProps {
    pub label: AttrValue,
    /// How the column is sorted, if it is.
    #[prop_or_default]
    pub direction: Option<SortDirection>,
    /// The way the column runs when first chosen (names from A, sizes largest first).
    #[prop_or_default]
    pub first: SortDirection,
    /// Figures, set to the end.
    #[prop_or_default]
    pub numeric: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// Called with the direction a press asks for ([`next_sort`]).
    #[prop_or_default]
    pub onsort: Callback<Option<SortDirection>>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A sort header's classes.
pub fn sort_header(numeric: bool) -> Classes {
    classes!(class::SORT_HEADER, numeric.then_some(class::NUMERIC))
}

/// A sort mark's classes: `idle` while the column is not sorted.
pub fn sort_mark(idle: bool) -> Classes {
    classes!(class::SORT_HEADER_MARK, idle.then_some(class::IDLE))
}

/// A sortable column heading with `aria-sort`. The mark has its box whether
/// the column is sorted or not, so sorting never changes the heading's width.
#[function_component]
pub fn SortHeader(props: &SortHeaderProps) -> Html {
    let glyph = match props.direction {
        Some(SortDirection::Ascending) => Glyph::ChevronUp,
        _ => Glyph::ChevronDown,
    };
    let onclick = {
        let onsort = props.onsort.clone();
        let (direction, first) = (props.direction, props.first);
        Callback::from(move |_: MouseEvent| onsort.emit(next_sort(direction, first)))
    };

    html! {
        <th class={classes!(sort_header(props.numeric), props.class.clone())} scope="col"
            aria-sort={props.direction.map_or("none", SortDirection::aria)}
            data-testid={props.testid.clone()}>
            <button type="button" class={classes!(class::SORT_HEADER_BUTTON, props.numeric.then_some(class::NUMERIC))}
                disabled={props.disabled} {onclick}>
                <span class={class::SORT_HEADER_LABEL}>{ props.label.clone() }</span>
                <span class={sort_mark(props.direction.is_none())} aria-hidden="true">
                    <Icon {glyph} />
                </span>
            </button>
        </th>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SortDirection::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_press_cycles_through_both_ways_and_back() {
        let mut state = None;
        let mut seen = Vec::new();

        for _ in 0..4 {
            state = next_sort(state, Ascending);
            seen.push(state);
        }

        assert_eq!(
            seen,
            [Some(Ascending), Some(Descending), None, Some(Ascending)]
        );
    }

    #[test]
    fn a_column_can_start_descending() {
        assert_eq!(next_sort(None, Descending), Some(Descending));
        assert_eq!(next_sort(Some(Descending), Descending), Some(Ascending));
        assert_eq!(next_sort(Some(Ascending), Descending), None);
    }

    #[test]
    fn aria_sort_names_the_direction() {
        assert_eq!(Ascending.aria(), "ascending");
        assert_eq!(Descending.aria(), "descending");
    }

    #[test]
    fn numeric_and_idle_are_modifiers() {
        assert_eq!(names(sort_header(false)), [class::SORT_HEADER]);
        assert_eq!(
            names(sort_header(true)),
            [class::SORT_HEADER, class::NUMERIC]
        );
        assert_eq!(names(sort_mark(false)), [class::SORT_HEADER_MARK]);
        assert_eq!(
            names(sort_mark(true)),
            [class::SORT_HEADER_MARK, class::IDLE]
        );
    }
}
