//! Disclosure (`style/_disclosure.scss`).

use crate::testid::with_testid;
use crate::{Glyph, HeadLevel, Icon, class};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DisclosureProps {
    pub title: Html,
    /// What the section holds, beside the title while it is shut.
    #[prop_or_default]
    pub summary: Option<Html>,
    /// Controls at the row's end, outside the toggle.
    #[prop_or_default]
    pub actions: Html,
    pub open: bool,
    /// Called with the state a press asks for.
    pub on_toggle: Callback<bool>,
    /// The body's id, which the toggle's `aria-controls` names.
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    /// The toggle in a heading of this level, its title at a list head's
    /// size: a section whose title is a heading.
    #[prop_or_default]
    pub level: Option<HeadLevel>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A disclosure's toggle: `heading` when it is in one.
pub fn disclosure_summary(heading: bool) -> Classes {
    classes!(class::DISCLOSURE_SUMMARY, heading.then_some(class::HEADING))
}

/// A disclosure's classes: `disclosure`, and `open` while its body shows.
pub fn disclosure(open: bool) -> Classes {
    classes!(class::DISCLOSURE, open.then_some(class::OPEN))
}

/// A summary row that toggles its body (`aria-expanded`). The row is one
/// height shut or open: the mark turns inside its box, and the summary
/// is cut to one line, so only the body adds height.
#[function_component]
pub fn Disclosure(props: &DisclosureProps) -> Html {
    let onclick = {
        let on_toggle = props.on_toggle.clone();
        let open = props.open;
        Callback::from(move |_: MouseEvent| on_toggle.emit(!open))
    };
    let detail =
        props.summary.as_ref().filter(|_| !props.open).map(
            |summary| html! { <span class={class::DISCLOSURE_DETAIL}>{ summary.clone() }</span> },
        );

    let summary = html! {
        <button type="button" class={disclosure_summary(props.level.is_some())}
            aria-expanded={props.open.to_string()}
            aria-controls={props.open.then(|| props.id.clone()).flatten()}
            disabled={props.disabled} {onclick}>
            <span class={classes!(class::DISCLOSURE_MARK, props.open.then_some(class::OPEN))} aria-hidden="true">
                <Icon glyph={Glyph::ChevronRight} />
            </span>
            <span class={class::DISCLOSURE_TITLE}>{ props.title.clone() }</span>
            { detail }
        </button>
    };
    let heading = match props.level {
        Some(level) => html! {
            <@{level.tag()} class={class::DISCLOSURE_HEADING}>{ summary }</@>
        },
        None => summary,
    };

    with_testid(
        html! {
            <div class={classes!(disclosure(props.open), props.class.clone())}>
                <div class={class::DISCLOSURE_HEAD}>
                    { heading }
                    <span class={class::DISCLOSURE_ACTIONS}>{ props.actions.clone() }</span>
                </div>
                if props.open {
                    <div class={class::DISCLOSURE_BODY} id={props.id.clone()}>{ props.children.clone() }</div>
                }
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

    #[test]
    fn a_heading_toggle_is_a_modifier() {
        assert_eq!(
            names(disclosure_summary(false)),
            [class::DISCLOSURE_SUMMARY]
        );
        assert_eq!(
            names(disclosure_summary(true)),
            [class::DISCLOSURE_SUMMARY, class::HEADING]
        );
    }

    #[test]
    fn open_is_a_modifier() {
        assert_eq!(names(disclosure(false)), [class::DISCLOSURE]);
        assert_eq!(names(disclosure(true)), [class::DISCLOSURE, class::OPEN]);
    }
}
