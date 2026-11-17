//! Checkbox and Switch (`style/_toggles.scss`).

use crate::{Glyph, Icon, class};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CheckboxProps {
    #[prop_or_default]
    pub class: Classes,
    pub checked: bool,
    /// Neither on nor off, as a parent of a partly chosen list; drawn with
    /// a minus and set as the input's DOM property. A click still reports
    /// the new `checked`.
    #[prop_or_default]
    pub indeterminate: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// Wrong (`aria-invalid`): the box's border in `--danger`.
    #[prop_or_default]
    pub invalid: bool,
    pub on_change: Callback<bool>,
    /// The accessible name, when there is no text or a field supplies it.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The text beside the box.
    #[prop_or_default]
    pub children: Html,
}

/// A checkbox's classes: `checkbox`, and `disabled`.
pub fn checkbox(disabled: bool) -> Classes {
    classes!(class::CHECKBOX, disabled.then_some(class::DISABLED))
}

/// A native checkbox drawn in a fixed `--check-size` box, with the check or
/// minus glyph over it: the box is the same checked, unchecked,
/// indeterminate, hovered or focused. Controlled: it shows `checked`.
#[function_component]
pub fn Checkbox(props: &CheckboxProps) -> Html {
    let input = use_node_ref();
    {
        let input = input.clone();
        let indeterminate = props.indeterminate;
        use_effect(move || {
            if let Some(input) = input.cast::<HtmlInputElement>() {
                input.set_indeterminate(indeterminate);
            }
        });
    }
    let onchange = props
        .on_change
        .reform(|e: Event| e.target_unchecked_into::<HtmlInputElement>().checked());
    let glyph = if props.indeterminate {
        Some(Glyph::Minus)
    } else {
        props.checked.then_some(Glyph::Check)
    };
    let has_text = props.children != html! {};

    html! {
        <label class={classes!(checkbox(props.disabled), props.class.clone())}>
            <span class={class::CHECKBOX_MARK}>
                <input type="checkbox" class={class::CHECKBOX_INPUT} ref={input}
                    checked={props.checked} disabled={props.disabled}
                    aria-invalid={props.invalid.then_some("true")}
                    aria-label={props.label.clone()}
                    data-testid={props.testid.clone()}
                    {onchange} />
                { for glyph.map(|glyph| html! { <Icon {glyph} class={class::CHECKBOX_GLYPH} /> }) }
            </span>
            if has_text {
                <span class={class::CHECKBOX_LABEL}>{ props.children.clone() }</span>
            }
        </label>
    }
}

#[derive(Properties, PartialEq)]
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

    html! {
        <button type="button" role="switch" class={classes!(switch(props.on, props.invalid), props.class.clone())}
            aria-checked={props.on.to_string()}
            aria-busy={props.busy.then_some("true")}
            aria-invalid={props.invalid.then_some("true")}
            aria-label={props.label.clone()}
            disabled={props.disabled}
            data-testid={props.testid.clone()}
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
    }
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
    fn a_checkbox_marks_disabled() {
        assert_eq!(names(checkbox(false)), [class::CHECKBOX]);
        assert_eq!(names(checkbox(true)), [class::CHECKBOX, class::DISABLED]);
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
    fn a_checkbox_draws_a_glyph_only_when_on_or_mixed() {
        let draw = |checked, indeterminate| {
            render::<Checkbox>(yew::props!(CheckboxProps {
                checked: checked,
                indeterminate: indeterminate,
                on_change: Callback::noop()
            }))
            .contains(class::CHECKBOX_GLYPH)
        };
        assert!(!draw(false, false));
        assert!(draw(true, false));
        assert!(draw(false, true));
    }
}
