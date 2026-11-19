//! SplitView: a list and the detail opened from it, side by side on a
//! [`Seam`], one at a time when narrow (`style/_split-view.scss`).

use crate::testid::with_testid;
use yew::prelude::*;

use crate::{Seam, class};

#[derive(Properties, PartialEq)]
pub struct SplitViewProps {
    /// The list, the first pane.
    pub list: Html,
    /// What is open, from the app's route; `None` when nothing is.
    #[prop_or_default]
    pub detail: Option<Html>,
    /// What the detail pane shows beside the list while nothing is open
    /// (wide only: narrow, the list shows alone).
    #[prop_or_default]
    pub empty: Html,
    /// The list's share of the room beside the divider, 0 to 1, where it
    /// starts and whenever this changes; a drag moves it from there.
    #[prop_or(0.4)]
    pub at: f64,
    /// Told the share once a drag (or a key) has moved the divider, for an
    /// app to keep.
    #[prop_or_default]
    pub on_settle: Callback<f64>,
    /// The least and greatest share; each pane also keeps `--seam-min`.
    #[prop_or(0.2)]
    pub min: f64,
    #[prop_or(0.8)]
    pub max: f64,
    /// The divider's name, for assistive technology.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

impl Default for SplitViewProps {
    fn default() -> Self {
        Self {
            list: Default::default(),
            detail: Default::default(),
            empty: Default::default(),
            at: ::yew::html::IntoPropValue::into_prop_value(0.4),
            on_settle: Default::default(),
            min: ::yew::html::IntoPropValue::into_prop_value(0.2),
            max: ::yew::html::IntoPropValue::into_prop_value(0.8),
            label: Default::default(),
            class: Default::default(),
            testid: Default::default(),
        }
    }
}

/// A split view's classes; `open` while a detail is.
pub fn split_view(open: bool) -> Classes {
    classes!(class::SPLIT_VIEW, open.then_some(class::OPEN))
}

/// A list and its detail. From `$split-view-breakpoint` (`tablet`) of its
/// own width (a container query) they sit side by side on a [`Seam`], the
/// detail pane holding `empty` while nothing is open; below it one pane
/// shows, the detail while one is open and the list otherwise. The app's
/// route says what is open, so back is the app's (or the browser's): a
/// [`class::SPLIT_VIEW_BACK`] in the detail shows only while the panes are
/// one at a time. Both panes stay mounted, so the list keeps its scroll and
/// focus while a detail covers it. Give the split view a height.
#[function_component]
pub fn SplitView(props: &SplitViewProps) -> Html {
    let at = use_state_eq(|| props.at);
    {
        let at = at.clone();
        use_effect_with(props.at, move |value| at.set(*value));
    }
    let on_change = {
        let at = at.clone();
        Callback::from(move |value: f64| at.set(value))
    };
    let open = props.detail.is_some();
    let detail = props.detail.clone().unwrap_or_else(|| props.empty.clone());

    with_testid(
        html! {
            <div class={classes!(split_view(open), props.class.clone())}>
                <Seam class={class::SPLIT_VIEW_SEAM} at={*at} {on_change} on_settle={props.on_settle.clone()}
                    min={props.min} max={props.max} label={props.label.clone()}
                    start={props.list.clone()} end={detail} />
            </div>
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

    fn render(detail: Option<Html>) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<SplitView>::with_props(yew::props!(SplitViewProps {
                list: html! { "list" },
                detail: detail,
                empty: html! { "nothing" },
            }))
            .render(),
        )
    }

    #[test]
    fn a_split_view_is_open_while_a_detail_is() {
        assert_eq!(names(split_view(false)), [class::SPLIT_VIEW]);
        assert_eq!(names(split_view(true)), [class::SPLIT_VIEW, class::OPEN]);
    }

    #[test]
    fn the_detail_pane_holds_the_empty_state_until_one_is_open() {
        let shut = render(None);
        assert!(
            shut.contains("nothing") && !shut.contains(class::OPEN),
            "{shut}"
        );
        let open = render(Some(html! { "card" }));
        assert!(open.contains("card") && !open.contains("nothing"), "{open}");
        assert!(open.contains(class::OPEN), "{open}");
    }
}
