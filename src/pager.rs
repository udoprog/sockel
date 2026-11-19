//! Pager (`style/_pager.scss`).

use crate::phrasebookcatalog01::{L10N, label};
use crate::testid::with_testid;
use crate::{Glyph, Icon, class};
use phrasebook02::args;
use phrasebook02::yew023::use_phrases_with;
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

/// What a summary pager says in place of its numbers: which items the page
/// shows ("1–50 of 312") over where it stands ("Page 1 of 7"), in the
/// app's words.
#[derive(Clone, Debug, PartialEq)]
pub struct PagerSummary {
    pub shown: AttrValue,
    pub position: AttrValue,
}

impl PagerSummary {
    pub fn new(shown: impl Into<AttrValue>, position: impl Into<AttrValue>) -> Self {
        Self {
            shown: shown.into(),
            position: position.into(),
        }
    }
}

/// The numbers of a summary pager whose words come from the catalog:
/// `total` items, `size` to a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PagerCounts {
    pub total: usize,
    pub size: usize,
}

impl PagerCounts {
    pub fn new(total: usize, size: usize) -> Self {
        Self { total, size }
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
    /// Gives every button a `title`, its accessible name, as a tooltip.
    #[prop_or_default]
    pub titles: bool,
    /// What the navigation is for (`aria-label` of the `nav`).
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub first_label: Option<AttrValue>,
    #[prop_or_default]
    pub previous_label: Option<AttrValue>,
    #[prop_or_default]
    pub next_label: Option<AttrValue>,
    #[prop_or_default]
    pub last_label: Option<AttrValue>,
    /// The accessible name of a numbered button, with the page's number
    /// appended ("Page 3"); else the catalog's.
    #[prop_or_default]
    pub page_label: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    /// Loading the page: the buttons keep their room and ignore presses.
    #[prop_or_default]
    pub busy: bool,
    /// Instead of the numbers and the first and last page: what the page
    /// shows and where it stands (a polite live region), then the previous
    /// and next page only (`SUMMARY`). A page past the last stays as given,
    /// so previous goes back to the last page.
    #[prop_or_default]
    pub summary: Option<PagerSummary>,
    /// A summary pager in the catalog's words ("51–100 of 312", "Page 2 of
    /// 7"), from the counts and `page`; `summary` wins when both are given.
    #[prop_or_default]
    pub counts: Option<PagerCounts>,
    /// The `nav` element, for an app that scrolls to what it pages.
    #[prop_or_default]
    pub nav_ref: NodeRef,
    /// Which items the page shows ("1–25 of 80"), at the row's start, the
    /// buttons at its end (`RANGED`); the row wraps in a narrow container.
    #[prop_or_default]
    pub range: Option<AttrValue>,
    /// Instead of the numbers and the first and last page: "3 / 35" between
    /// previous and next, for a menu or drop-down too narrow for a row of
    /// numbers. The value is the accessible name of that text ("Page 3 of
    /// 35"), in the app's words.
    #[prop_or_default]
    pub compact: Option<AttrValue>,
    /// Makes the number of each page but the current a drop target: called
    /// with the page, counted from 0, it gives the value of the button's
    /// `data-sort-page` attribute, for the list being reordered by dragging
    /// to read where a row was let go.
    #[prop_or_default]
    pub drop_page: Option<Callback<usize, AttrValue>>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

impl Default for PagerProps {
    fn default() -> Self {
        Self {
            page: Default::default(),
            pages: Default::default(),
            on_page: Default::default(),
            slots: ::yew::html::IntoPropValue::into_prop_value(7),
            edges: ::yew::html::IntoPropValue::into_prop_value(true),
            phone_steps: ::yew::html::IntoPropValue::into_prop_value(true),
            titles: Default::default(),
            label: Default::default(),
            first_label: Default::default(),
            previous_label: Default::default(),
            next_label: Default::default(),
            last_label: Default::default(),
            page_label: Default::default(),
            disabled: Default::default(),
            busy: Default::default(),
            summary: Default::default(),
            counts: Default::default(),
            nav_ref: Default::default(),
            range: Default::default(),
            compact: Default::default(),
            drop_page: Default::default(),
            class: Default::default(),
            testid: Default::default(),
        }
    }
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
    let phrases = use_phrases_with(&L10N);
    let nav_label = label(&phrases, &props.label, "sockel-pager-pages");
    let first_label = label(&phrases, &props.first_label, "sockel-pager-first");
    let previous_label = label(&phrases, &props.previous_label, "sockel-pager-previous");
    let next_label = label(&phrases, &props.next_label, "sockel-pager-next");
    let last_label = label(&phrases, &props.last_label, "sockel-pager-last");
    let page_label = |n: usize| match &props.page_label {
        Some(given) => format!("{given} {n}"),
        None => phrases.msg("sockel-pager-page", &args([("page", n)])),
    };
    if let Some(summary) = &props.summary {
        return summary_pager(props, summary, &nav_label, &previous_label, &next_label);
    }
    if let Some(counts) = props.counts {
        let size = counts.size.max(1);
        let total = counts.total;
        let start = (props.page * size + 1).min(total);
        let end = ((props.page + 1) * size).min(total);
        let summary = PagerSummary::new(
            phrases.msg(
                "sockel-pager-counts-shown",
                &args([("start", start), ("end", end), ("total", total)]),
            ),
            phrases.msg(
                "sockel-pager-counts-position",
                &args([("page", props.page + 1), ("pages", props.pages.max(1))]),
            ),
        );
        return summary_pager(props, &summary, &nav_label, &previous_label, &next_label);
    }

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
                title={props.titles.then(|| label.clone())}
                disabled={blocked || off} onclick={go(to)}>
                <span class={classes!(class::PAGER_MARK, (!forward).then_some(class::PAGER_FLIP))} aria-hidden="true">
                    <Icon glyph={Glyph::ChevronRight} />
                    { double }
                </span>
            </button>
        }
    };

    let slots = if props.compact.is_none() {
        pager_slots(pages, page, props.slots)
    } else {
        Vec::new()
    };
    let numbers = slots
        .into_iter()
        .map(|slot| match slot {
            Slot::Gap => html! { <span class={class::PAGER_GAP} aria-hidden="true">{ "…" }</span> },
            Slot::Page(n) => {
                let current = n == page;
                let label = page_label(n + 1);
                html! {
                    <button type="button" class={class::PAGER_BUTTON}
                        title={props.titles.then(|| label.clone())}
                        aria-label={label}
                        aria-current={current.then_some("page")}
                        data-sort-page={props.drop_page.as_ref().filter(|_| !current).map(|f| f.emit(n))}
                        disabled={blocked}
                        onclick={if current { Callback::noop() } else { go(n) }}>
                        { n + 1 }
                    </button>
                }
            }
        });

    let at = props.compact.as_ref().map(|name| {
        html! {
            <span class={class::PAGER_AT} aria-label={name.clone()}>{ format!("{} / {pages}", page + 1) }</span>
        }
    });
    let edges = props.edges && at.is_none();
    let buttons = html! {
        <>
            if edges {
                { step(&first_label, false, 0, page == 0, true) }
            }
            { step(&previous_label, false, page.saturating_sub(1), page == 0, false) }
            { for at }
            { for numbers }
            { step(&next_label, true, (page + 1).min(pages - 1), page + 1 >= pages, false) }
            if edges {
                { step(&last_label, true, pages - 1, page + 1 >= pages, true) }
            }
        </>
    };

    with_testid(
        html! {
            <nav class={classes!(pager(props.disabled, props.busy), props.range.is_some().then_some(class::RANGED), props.class.clone())}
                aria-label={nav_label} aria-busy={props.busy.then_some("true")}
                style={format!("--pager-chars: {}", pages.to_string().len())}
                ref={props.nav_ref.clone()}>
                if let Some(range) = &props.range {
                    <span class={class::PAGER_RANGE}>{ range.clone() }</span>
                    <span class={class::PAGER_GROUP}>{ buttons }</span>
                } else {
                    { buttons }
                }
            </nav>
        },
        props.testid.clone(),
    )
}

/// A summary pager: the counts, then previous and next.
fn summary_pager(
    props: &PagerProps,
    summary: &PagerSummary,
    nav_label: &AttrValue,
    previous_label: &AttrValue,
    next_label: &AttrValue,
) -> Html {
    let pages = props.pages.max(1);
    let page = props.page;
    let blocked = props.disabled || props.busy;

    let step = |label: &AttrValue, forward: bool, to: usize, off: bool| {
        let on_page = props.on_page.clone();
        let hidden = (!props.phone_steps).then_some(class::DESKTOP_ONLY);
        html! {
            <button type="button" class={classes!(class::PAGER_BUTTON, hidden)} aria-label={label.clone()}
                title={props.titles.then(|| label.clone())}
                disabled={blocked || off} onclick={Callback::from(move |_: MouseEvent| on_page.emit(to))}>
                <span class={classes!(class::PAGER_MARK, (!forward).then_some(class::PAGER_FLIP))} aria-hidden="true">
                    <Icon glyph={Glyph::ChevronRight} />
                </span>
            </button>
        }
    };

    with_testid(
        html! {
            <nav class={classes!(pager(props.disabled, props.busy), class::SUMMARY, props.class.clone())}
                aria-label={nav_label.clone()} aria-busy={props.busy.then_some("true")}
                style="--pager-chars: 1"
                ref={props.nav_ref.clone()}>
                <span class={class::PAGER_STATUS} aria-live="polite">
                    <span class={class::PAGER_SHOWN}>{ summary.shown.clone() }</span>
                    <span class={class::PAGER_POSITION}>{ summary.position.clone() }</span>
                </span>
                <span class={class::PAGER_STEPS}>
                    { step(previous_label, false, page.saturating_sub(1).min(pages - 1), page == 0) }
                    { step(next_label, true, page + 1, page + 1 >= pages) }
                </span>
            </nav>
        },
        props.testid.clone(),
    )
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

    fn render(props: PagerProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Pager>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn titles_repeat_each_buttons_name() {
        let plain = render(yew::props!(PagerProps {
            page: 0,
            pages: 3,
            on_page: Callback::noop()
        }));
        assert!(!plain.contains("title="), "{plain}");
        let titled = render(yew::props!(PagerProps {
            page: 0,
            pages: 3,
            on_page: Callback::noop(),
            titles: true,
        }));
        for title in [
            "First page",
            "Previous page",
            "Page 2",
            "Next page",
            "Last page",
        ] {
            assert!(
                titled.contains(&format!("title=\"{title}\"")),
                "{title} in {titled}"
            );
        }
    }

    #[test]
    fn a_summary_has_its_counts_and_two_steps() {
        let html = render(yew::props!(PagerProps {
            page: 1,
            pages: 7,
            on_page: Callback::noop(),
            summary: Some(PagerSummary::new("51–100 of 312", "Page 2 of 7")),
        }));
        assert!(html.contains("51–100 of 312"), "{html}");
        assert!(html.contains("Page 2 of 7"), "{html}");
        assert!(html.contains(class::SUMMARY), "{html}");
        assert_eq!(html.matches("<button").count(), 2, "{html}");
        assert!(!html.contains(class::PAGER_GAP), "{html}");
        // Its two buttons are as wide as a page button.
        assert!(html.contains("--pager-chars: 1"), "{html}");
    }

    #[test]
    fn counts_word_the_summary_from_the_catalog() {
        let html = render(yew::props!(PagerProps {
            page: 1,
            pages: 7,
            on_page: Callback::noop(),
            counts: Some(PagerCounts::new(312, 50)),
        }));
        assert!(html.contains("51–100 of 312"), "{html}");
        assert!(html.contains("Page 2 of 7"), "{html}");
        assert!(html.contains(class::SUMMARY), "{html}");

        let html = render(yew::props!(PagerProps {
            page: 0,
            pages: 1,
            on_page: Callback::noop(),
            counts: Some(PagerCounts::new(0, 50)),
        }));
        assert!(html.contains("0–0 of 0"), "{html}");

        let html = render(yew::props!(PagerProps {
            page: 1,
            pages: 7,
            on_page: Callback::noop(),
            summary: Some(PagerSummary::new("Given", "Here")),
            counts: Some(PagerCounts::new(312, 50)),
        }));
        assert!(html.contains("Given") && !html.contains("of 312"), "{html}");
    }

    #[test]
    fn a_range_compact_and_drop_targets() {
        let html = render(yew::props!(PagerProps {
            page: 1,
            pages: 35,
            on_page: Callback::noop(),
            range: Some(AttrValue::from("26–50 of 840")),
            compact: Some(AttrValue::from("Page 2 of 35")),
        }));
        assert!(html.contains("26–50 of 840"), "{html}");
        assert!(html.contains(">2 / 35<"), "{html}");
        assert!(html.contains("aria-label=\"Page 2 of 35\""), "{html}");
        assert!(!html.contains("First page"), "{html}");
        assert_eq!(html.matches("<button").count(), 2, "{html}");

        let html = render(yew::props!(PagerProps {
            page: 0,
            pages: 3,
            on_page: Callback::noop(),
            drop_page: Some(Callback::from(|p: usize| AttrValue::from(
                (p * 25).to_string()
            ))),
        }));
        assert!(html.contains("data-sort-page=\"25\""), "{html}");
        assert!(html.contains("data-sort-page=\"50\""), "{html}");
        assert!(!html.contains("data-sort-page=\"0\""), "{html}");
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
