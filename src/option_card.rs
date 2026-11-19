//! OptionCard (`style/_checkbox.scss`).

use crate::testid::with_testid;
use crate::{Glyph, Icon, class};
use web_sys::HtmlInputElement;
use yew::prelude::*;

/// Whether an [`OptionCard`] is one of a pick-one set or picked on its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OptionCardKind {
    /// A native radio; cards sharing a `name` are one group.
    #[default]
    Radio,
    Checkbox,
}

#[derive(Properties, PartialEq)]
pub struct OptionCardProps {
    #[prop_or_default]
    pub kind: OptionCardKind,
    pub checked: bool,
    /// The input's `name`: radios of one group share it.
    #[prop_or_default]
    pub name: Option<AttrValue>,
    #[prop_or_default]
    pub value: Option<AttrValue>,
    /// The new `checked` (a radio only ever reports `true`).
    pub on_change: Callback<bool>,
    /// The card's first line, in the text's weight.
    pub label: AttrValue,
    /// A line below the label, in `--muted`.
    #[prop_or_default]
    pub description: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// An option card's classes: `option-card`, `on`, `disabled` and `invalid`.
pub fn option_card(checked: bool, disabled: bool, invalid: bool) -> Classes {
    classes!(
        class::OPTION_CARD,
        checked.then_some(class::ON),
        disabled.then_some(class::DISABLED),
        invalid.then_some(class::INVALID)
    )
}

/// A radio or checkbox drawn as a card: the [`Checkbox`](crate::Checkbox)'s
/// mark (round for a radio), the label and a description line. Checked,
/// hovered, focused or disabled change colours only, so the card keeps its
/// box. Controlled: it shows `checked`.
#[function_component]
pub fn OptionCard(props: &OptionCardProps) -> Html {
    let onchange = props
        .on_change
        .reform(|e: Event| e.target_unchecked_into::<HtmlInputElement>().checked());
    let radio = props.kind == OptionCardKind::Radio;

    html! {
        <label class={classes!(option_card(props.checked, props.disabled, props.invalid), props.class.clone())}>
            <span class={class::CHECKBOX_MARK}>
                { with_testid(html! { <input type={if radio { "radio" } else { "checkbox" }}
                    class={classes!(class::CHECKBOX_INPUT, radio.then_some(class::RADIO))}
                    name={props.name.clone()} value={props.value.clone()}
                    checked={props.checked} disabled={props.disabled}
                    aria-invalid={props.invalid.then_some("true")} {onchange} /> }, props.testid.clone()) }
                if props.checked && !radio {
                    <Icon glyph={Glyph::Check} class={class::CHECKBOX_GLYPH} />
                }
            </span>
            <span class={class::OPTION_CARD_TEXT}>
                <span class={class::OPTION_CARD_LABEL}>{ props.label.clone() }</span>
                if let Some(description) = &props.description {
                    <span class={class::OPTION_CARD_DESCRIPTION}>{ description.clone() }</span>
                }
            </span>
        </label>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn render(kind: OptionCardKind, checked: bool) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<OptionCard>::with_props(yew::props!(OptionCardProps {
                kind: kind,
                checked: checked,
                on_change: Callback::noop(),
                label: AttrValue::from("Keep"),
                description: AttrValue::from("Leave the card as it is."),
            }))
            .hydratable(false)
            .render(),
        )
    }

    #[test]
    fn checked_disabled_and_invalid_are_modifiers() {
        assert_eq!(
            names(option_card(false, false, false)),
            [class::OPTION_CARD]
        );
        assert_eq!(
            names(option_card(true, true, true)),
            [
                class::OPTION_CARD,
                class::ON,
                class::DISABLED,
                class::INVALID
            ]
        );
    }

    #[test]
    fn a_radio_is_round_and_a_checkbox_draws_its_check() {
        let radio = render(OptionCardKind::Radio, true);
        assert!(radio.contains("type=\"radio\""), "{radio}");
        assert!(radio.contains(class::RADIO), "{radio}");
        assert!(!radio.contains(class::CHECKBOX_GLYPH), "{radio}");
        let checkbox = render(OptionCardKind::Checkbox, true);
        assert!(checkbox.contains("type=\"checkbox\""), "{checkbox}");
        assert!(checkbox.contains(class::CHECKBOX_GLYPH), "{checkbox}");
        assert!(!render(OptionCardKind::Checkbox, false).contains(class::CHECKBOX_GLYPH));
        for html in [&radio, &checkbox] {
            assert!(html.contains(class::OPTION_CARD_LABEL), "{html}");
            assert!(html.contains(">Leave the card as it is.<"), "{html}");
        }
    }
}
