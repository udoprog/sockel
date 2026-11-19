//! StatusLabel (`style/_badges.scss`): a status whose width is the widest of
//! its states.

use crate::badge::{Tone, pill};
use crate::testid::with_testid;
use crate::{Glyph, Icon, class};
use yew::prelude::*;

/// A [`StatusLabel`]'s classes: `status-label`, its tone and, as a pill,
/// the pill's without its dot.
pub fn status_label(tone: Tone, as_pill: bool) -> Classes {
    if as_pill {
        classes!(class::STATUS_LABEL, pill(tone, true, false))
    } else {
        classes!(class::STATUS_LABEL, tone.class())
    }
}

#[derive(Properties, PartialEq)]
pub struct StatusLabelProps {
    #[prop_or_default]
    pub class: Classes,
    /// The text of every state the label can be in. All of them are laid
    /// out, so the label is as wide as the widest, and only the current one
    /// is seen.
    pub states: Vec<AttrValue>,
    /// The index in `states` shown; none shows nothing, in the same box.
    #[prop_or_default]
    pub current: Option<usize>,
    #[prop_or_default]
    pub tone: Tone,
    /// Reserves a slot for a spinner before the text.
    #[prop_or_default]
    pub spinner: bool,
    /// Spins in the slot; the slot is there either way.
    #[prop_or_default]
    pub busy: bool,
    /// Drawn as a [`Pill`](crate::Pill) without a dot.
    #[prop_or_default]
    pub pill: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A status (`role="status"`) that does not resize when it changes: every
/// state's text shares one grid cell, the others hidden.
#[function_component]
pub fn StatusLabel(props: &StatusLabelProps) -> Html {
    let text = |(i, state): (usize, &AttrValue)| {
        let shown = props.current == Some(i);
        html! {
            <span class={classes!(class::STATUS_TEXT, (!shown).then_some(class::HIDDEN))}
                aria-hidden={(!shown).then_some("true")}>
                { state.clone() }
            </span>
        }
    };

    with_testid(
        html! {
            <span class={classes!(status_label(props.tone, props.pill), props.class.clone())}
                role="status">
                if props.spinner {
                    <span class={class::STATUS_SPINNER} aria-hidden="true">
                        if props.busy { <Icon glyph={Glyph::Spinner} /> }
                    </span>
                }
                <span class={class::STATUS_LABELS}>{ for props.states.iter().enumerate().map(text) }</span>
            </span>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(c: Classes) -> Vec<String> {
        c.into_iter().map(|c| c.to_string()).collect()
    }

    fn render(props: StatusLabelProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<StatusLabel>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props(current: Option<usize>) -> StatusLabelProps {
        yew::props!(StatusLabelProps {
            states: vec!["Loading...".into(), "Saved".into(), "Failed".into()],
            current,
        })
    }

    #[test]
    fn it_marks_tone_and_pill() {
        assert_eq!(
            names(status_label(Tone::Neutral, false)),
            [class::STATUS_LABEL]
        );
        assert_eq!(
            names(status_label(Tone::Success, false)),
            [class::STATUS_LABEL, class::SUCCESS]
        );
        assert_eq!(
            names(status_label(Tone::Danger, true)),
            [
                class::STATUS_LABEL,
                class::PILL,
                class::DANGER,
                class::PLAIN
            ]
        );
    }

    #[test]
    fn every_state_is_rendered_and_only_the_current_is_visible() {
        let html = render(props(Some(1)));
        for text in ["Loading...", "Saved", "Failed"] {
            assert!(html.contains(&format!(">{text}<")), "{text} in {html}");
        }
        assert_eq!(html.matches(class::HIDDEN).count(), 2, "{html}");
        assert!(
            html.contains(&format!("<span class=\"{}\">Saved<", class::STATUS_TEXT)),
            "{html}"
        );
    }

    #[test]
    fn no_current_state_hides_all_and_keeps_the_text() {
        let html = render(props(None));
        assert_eq!(html.matches(class::HIDDEN).count(), 3, "{html}");
        assert!(html.contains(">Loading...<"), "{html}");
    }

    #[test]
    fn the_spinner_slot_is_there_idle_and_busy() {
        let mut p = props(Some(0));
        p.spinner = true;
        let idle = render(p);
        assert!(idle.contains(class::STATUS_SPINNER), "{idle}");
        assert!(!idle.contains(class::GLYPH), "{idle}");

        let mut p = props(Some(0));
        p.spinner = true;
        p.busy = true;
        assert!(render(p).contains(class::GLYPH));
        assert!(!render(props(Some(0))).contains(class::STATUS_SPINNER));
    }
}
