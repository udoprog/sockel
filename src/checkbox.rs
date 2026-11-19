//! Checkbox (`style/_checkbox.scss`).

use crate::testid::with_testid;
use crate::{Glyph, Icon, class};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, Default, PartialEq)]
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
    /// A tip for [`DataTips`](crate::DataTips) (`data-tip` on the label).
    #[prop_or_default]
    pub tip: Option<AttrValue>,
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
        <label class={classes!(checkbox(props.disabled), props.class.clone())} data-tip={props.tip.clone()}>
            <span class={class::CHECKBOX_MARK}>
                { with_testid(html! { <input type="checkbox" class={class::CHECKBOX_INPUT} ref={input}
                    checked={props.checked} disabled={props.disabled}
                    aria-invalid={props.invalid.then_some("true")}
                    aria-label={props.label.clone()}
                    {onchange} /> }, props.testid.clone()) }
                { for glyph.map(|glyph| html! { <Icon {glyph} class={class::CHECKBOX_GLYPH} /> }) }
            </span>
            if has_text {
                <span class={class::CHECKBOX_LABEL}>{ props.children.clone() }</span>
            }
        </label>
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
    fn a_tip_is_data_tip_only_when_set() {
        let checkbox = |tip: Option<&'static str>| {
            render::<Checkbox>(yew::props!(CheckboxProps {
                checked: false,
                tip: tip.map(AttrValue::from),
                on_change: Callback::noop()
            }))
        };
        assert!(!checkbox(None).contains("data-tip"));
        assert!(checkbox(Some("Pin")).contains("data-tip=\"Pin\""));
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
