//! Pager (`style/_pager.scss`).

use crate::{Glyph, Icon, class};
use yew::prelude::*;

/// The fewest slots a pager can lay out: first, a gap, the current page, a
/// gap and last.
pub const MIN_SLOTS: usize = 5;

/// One numbered slot of a pager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// A page, counted from 0.
    Page(usize),
    /// A run of pages left out.
    Gap,
}

/// How many pages `total` items fill, `size` to a page; at least one.
pub fn page_count(total: usize, size: usize) -> usize {
    total.div_ceil(size.max(1)).max(1)
}

/// The numbered slots for `pages` pages with `page` current: every page when
/// they fit in `slots`, otherwise exactly `slots` entries, the first and last
/// page always among them and a gap hiding at least two pages. Because the
/// count never changes with `page`, nor does the pager's width.
pub fn pager_slots(pages: usize, page: usize, slots: usize) -> Vec<Slot> {
    let slots = slots.max(MIN_SLOTS);
    let page = page.min(pages.saturating_sub(1));

    if pages <= slots {
        return (0..pages).map(Slot::Page).collect();
    }

    let last = pages - 1;
    // The numbers in the middle layout, between two gaps and the edges.
    let middle = slots - 4;

    if page + 3 < slots {
        (0..slots - 2)
            .map(Slot::Page)
            .chain([Slot::Gap, Slot::Page(last)])
            .collect()
    } else if page + slots - 3 > last {
        [Slot::Page(0), Slot::Gap]
            .into_iter()
            .chain((pages - (slots - 2)..pages).map(Slot::Page))
            .collect()
    } else {
        let start = page - (middle - 1) / 2;
        [Slot::Page(0), Slot::Gap]
            .into_iter()
            .chain((start..start + middle).map(Slot::Page))
            .chain([Slot::Gap, Slot::Page(last)])
            .collect()
    }
}

#[derive(Properties, PartialEq)]
pub struct PagerProps {
    /// The current page, counted from 0.
    pub page: usize,
    /// How many pages there are.
    pub pages: usize,
    /// Called with the page asked for, counted from 0.
    pub on_page: Callback<usize>,
    /// How many numbered slots there are at most ([`MIN_SLOTS`] or more).
    #[prop_or(7)]
    pub slots: usize,
    /// First and last page buttons, hidden on a phone.
    #[prop_or(true)]
    pub edges: bool,
    /// The previous and next page buttons below the phone breakpoint;
    /// `false` hides them there (`DESKTOP_ONLY`, from `_utilities.scss`).
    #[prop_or(true)]
    pub phone_steps: bool,
    /// What the navigation is for (`aria-label` of the `nav`).
    #[prop_or("Pages".into())]
    pub label: AttrValue,
    #[prop_or("First page".into())]
    pub first_label: AttrValue,
    #[prop_or("Previous page".into())]
    pub previous_label: AttrValue,
    #[prop_or("Next page".into())]
    pub next_label: AttrValue,
    #[prop_or("Last page".into())]
    pub last_label: AttrValue,
    /// The accessible name of a numbered button; the page's number is
    /// appended ("Page 3").
    #[prop_or("Page".into())]
    pub page_label: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    /// Loading the page: the buttons keep their room and ignore presses.
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A pager's classes: `pager`, `disabled` and `busy`.
pub fn pager(disabled: bool, busy: bool) -> Classes {
    classes!(
        class::PAGER,
        disabled.then_some(class::DISABLED),
        busy.then_some(class::BUSY)
    )
}

/// A pager button's classes: `pager-button`, and `pager-edge` for the first
/// and last page, which a phone hides.
pub fn pager_button(edge: bool) -> Classes {
    classes!(class::PAGER_BUTTON, edge.then_some(class::PAGER_EDGE))
}

/// Page buttons with a fixed number of slots: first, previous, numbers with
/// gaps, next and last. The numbered slots are always the same count and
/// each as wide as the widest page number, so the pager keeps its width from
/// the first page to the last. The current page is marked by `aria-current`,
/// its colour and background, never its weight.
#[function_component]
pub fn Pager(props: &PagerProps) -> Html {
    let pages = props.pages.max(1);
    let page = props.page.min(pages - 1);
    let blocked = props.disabled || props.busy;

    let go = |to: usize| {
        let on_page = props.on_page.clone();
        Callback::from(move |_: MouseEvent| on_page.emit(to))
    };
    let step = |label: &AttrValue, forward: bool, to: usize, off: bool, edge: bool| {
        let double =
            edge.then(|| html! { <Icon glyph={Glyph::ChevronRight} class={class::PAGER_ECHO} /> });
        let hidden = (!edge && !props.phone_steps).then_some(class::DESKTOP_ONLY);
        html! {
            <button type="button" class={classes!(pager_button(edge), hidden)} aria-label={label.clone()}
                disabled={blocked || off} onclick={go(to)}>
                <span class={classes!(class::PAGER_MARK, (!forward).then_some(class::PAGER_FLIP))} aria-hidden="true">
                    <Icon glyph={Glyph::ChevronRight} />
                    { double }
                </span>
            </button>
        }
    };

    let numbers = pager_slots(pages, page, props.slots)
        .into_iter()
        .map(|slot| match slot {
            Slot::Gap => html! { <span class={class::PAGER_GAP} aria-hidden="true">{ "…" }</span> },
            Slot::Page(n) => {
                let current = n == page;
                html! {
                    <button type="button" class={class::PAGER_BUTTON}
                        aria-label={format!("{} {}", props.page_label, n + 1)}
                        aria-current={current.then_some("page")}
                        disabled={blocked}
                        onclick={if current { Callback::noop() } else { go(n) }}>
                        { n + 1 }
                    </button>
                }
            }
        });

    html! {
        <nav class={classes!(pager(props.disabled, props.busy), props.class.clone())}
            aria-label={props.label.clone()} aria-busy={props.busy.then_some("true")}
            style={format!("--pager-chars: {}", pages.to_string().len())}
            data-testid={props.testid.clone()}>
            if props.edges {
                { step(&props.first_label, false, 0, page == 0, true) }
            }
            { step(&props.previous_label, false, page.saturating_sub(1), page == 0, false) }
            { for numbers }
            { step(&props.next_label, true, (page + 1).min(pages - 1), page + 1 >= pages, false) }
            if props.edges {
                { step(&props.last_label, true, pages - 1, page + 1 >= pages, true) }
            }
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Slot::{Gap, Page};

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn few_pages_are_all_shown() {
        assert_eq!(pager_slots(1, 0, 7), [Page(0)]);
        assert_eq!(pager_slots(3, 2, 7), [Page(0), Page(1), Page(2)]);
        assert_eq!(pager_slots(7, 3, 7), (0..7).map(Page).collect::<Vec<_>>());
        assert_eq!(pager_slots(0, 0, 7), []);
    }

    #[test]
    fn many_pages_gap_where_they_are_far() {
        let at = |page| pager_slots(20, page, 7);
        assert_eq!(
            at(0),
            [Page(0), Page(1), Page(2), Page(3), Page(4), Gap, Page(19)]
        );
        assert_eq!(at(3), at(0));
        assert_eq!(
            at(4),
            [Page(0), Gap, Page(3), Page(4), Page(5), Gap, Page(19)]
        );
        assert_eq!(
            at(10),
            [Page(0), Gap, Page(9), Page(10), Page(11), Gap, Page(19)]
        );
        assert_eq!(
            at(15),
            [Page(0), Gap, Page(14), Page(15), Page(16), Gap, Page(19)]
        );
        assert_eq!(
            at(16),
            [
                Page(0),
                Gap,
                Page(15),
                Page(16),
                Page(17),
                Page(18),
                Page(19)
            ]
        );
        assert_eq!(at(19), at(16));
    }

    #[test]
    fn the_slot_count_never_changes_with_the_page() {
        for slots in MIN_SLOTS..=9 {
            for pages in 1..=40 {
                let want = pages.min(slots);
                for page in 0..pages {
                    let got = pager_slots(pages, page, slots);
                    assert_eq!(got.len(), want, "{pages} pages, page {page}, {slots} slots");
                    if pages > slots {
                        assert_eq!(got[0], Page(0));
                        assert_eq!(got[got.len() - 1], Page(pages - 1));
                    }
                }
            }
        }
    }

    #[test]
    fn the_current_page_is_shown_and_pages_ascend() {
        for pages in [8, 9, 20, 100] {
            for page in 0..pages {
                let got = pager_slots(pages, page, 7);
                assert!(
                    got.contains(&Page(page)),
                    "{pages} pages, page {page}: {got:?}"
                );
                let mut hidden_ok = true;
                let mut prev: Option<usize> = None;
                let mut gap = false;
                for slot in &got {
                    match slot {
                        Gap => gap = true,
                        Page(n) => {
                            if let Some(p) = prev {
                                hidden_ok &= *n > p && (n - p == 1 || (gap && n - p >= 3));
                            }
                            prev = Some(*n);
                            gap = false;
                        }
                    }
                }
                assert!(hidden_ok, "{pages} pages, page {page}: {got:?}");
            }
        }
    }

    #[test]
    fn the_smallest_pager_still_reaches_both_ends() {
        assert_eq!(
            pager_slots(30, 0, 5),
            [Page(0), Page(1), Page(2), Gap, Page(29)]
        );
        assert_eq!(
            pager_slots(30, 15, 5),
            [Page(0), Gap, Page(15), Gap, Page(29)]
        );
        assert_eq!(
            pager_slots(30, 29, 5),
            [Page(0), Gap, Page(27), Page(28), Page(29)]
        );
        assert_eq!(pager_slots(30, 15, 2), pager_slots(30, 15, 5));
    }

    #[test]
    fn a_page_past_the_end_counts_as_the_last() {
        assert_eq!(pager_slots(20, 99, 7), pager_slots(20, 19, 7));
    }

    #[test]
    fn pages_round_up() {
        assert_eq!(page_count(0, 50), 1);
        assert_eq!(page_count(50, 50), 1);
        assert_eq!(page_count(51, 50), 2);
        assert_eq!(page_count(10, 0), 10);
    }

    #[test]
    fn disabled_busy_and_edge_are_modifiers() {
        assert_eq!(names(pager(false, false)), [class::PAGER]);
        assert_eq!(
            names(pager(true, true)),
            [class::PAGER, class::DISABLED, class::BUSY]
        );
        assert_eq!(names(pager_button(false)), [class::PAGER_BUTTON]);
        assert_eq!(
            names(pager_button(true)),
            [class::PAGER_BUTTON, class::PAGER_EDGE]
        );
    }
}
