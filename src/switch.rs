//! Switch (`style/_switch.scss`).

use crate::testid::with_testid;
use crate::{Glyph, Icon, class};
use yew::prelude::*;

#[derive(Properties, Default, PartialEq)]
pub struct SwitchProps {
    #[prop_or_default]
    pub class: Classes,
    pub on: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// Applying the change: a spinner in the thumb, which keeps its place;
    /// a busy switch ignores clicks (`aria-busy`).
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub invalid: bool,
    pub on_change: Callback<bool>,
    /// The accessible name, when there is no text or a field supplies it.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// A tip for [`DataTips`](crate::DataTips) (`data-tip`).
    #[prop_or_default]
    pub tip: Option<AttrValue>,
    /// A native hover hint (`title`).
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The text beside the switch.
    #[prop_or_default]
    pub children: Html,
}

/// A switch's classes: `switch`, `on` and `invalid`.
pub fn switch(on: bool, invalid: bool) -> Classes {
    classes!(
        class::SWITCH,
        on.then_some(class::ON),
        invalid.then_some(class::INVALID)
    )
}

/// A button with `role="switch"`: a track and a thumb of fixed size, the
/// thumb moved by transform, so on, off, busy, hovered and focused are the
/// same box.
#[function_component]
pub fn Switch(props: &SwitchProps) -> Html {
    let onclick = {
        let on_change = props.on_change.clone();
        let (on, blocked) = (props.on, props.busy || props.disabled);
        Callback::from(move |_: MouseEvent| {
            if !blocked {
                on_change.emit(!on);
            }
        })
    };
    let has_text = props.children != html! {};

    with_testid(
        html! {
            <button type="button" role="switch" class={classes!(switch(props.on, props.invalid), props.class.clone())}
                aria-checked={props.on.to_string()}
                aria-busy={props.busy.then_some("true")}
                aria-invalid={props.invalid.then_some("true")}
                aria-label={props.label.clone()}
                disabled={props.disabled}
                data-tip={props.tip.clone()}
                title={props.title.clone()}
                {onclick}>
                <span class={class::SWITCH_TRACK}>
                    <span class={class::SWITCH_THUMB}>
                        if props.busy { <Icon glyph={Glyph::Spinner} /> }
                    </span>
                </span>
                if has_text {
                    <span class={class::SWITCH_LABEL}>{ props.children.clone() }</span>
                }
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

    fn render<C: BaseComponent>(props: C::Properties) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<C>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn a_switch_marks_on_and_invalid() {
        assert_eq!(names(switch(false, false)), [class::SWITCH]);
        assert_eq!(names(switch(true, false)), [class::SWITCH, class::ON]);
        assert_eq!(names(switch(false, true)), [class::SWITCH, class::INVALID]);
    }

    #[test]
    fn a_switch_is_a_switch_with_its_state() {
        let html = render::<Switch>(yew::props!(SwitchProps {
            on: true,
            busy: true,
            on_change: Callback::noop()
        }));
        assert!(html.contains("role=\"switch\""), "{html}");
        assert!(html.contains("aria-checked=\"true\""), "{html}");
        assert!(html.contains("aria-busy=\"true\""), "{html}");
        assert!(html.contains(class::SPIN), "{html}");
    }

    #[test]
    fn a_tip_is_data_tip_only_when_set() {
        let switch = render::<Switch>(yew::props!(SwitchProps {
            on: false,
            tip: AttrValue::from("Live"),
            on_change: Callback::noop()
        }));
        assert!(switch.contains("data-tip=\"Live\""), "{switch}");
    }

    #[test]
    fn a_title_is_set_only_when_given() {
        let switch = |title: Option<&'static str>| {
            render::<Switch>(yew::props!(SwitchProps {
                on: false,
                title: title.map(AttrValue::from),
                on_change: Callback::noop()
            }))
        };
        assert!(!switch(None).contains("title="));
        assert!(switch(Some("Turn on")).contains("title=\"Turn on\""));
    }
}
