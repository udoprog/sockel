//! Choice (`style/_toggles.scss`).

use crate::class;
use crate::roving::{self, Arrows};
use yew::prelude::*;

/// How a [`Choice`]'s options share its width: all of them the width of
/// the widest unless set, so no selection changes any.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChoiceFit {
    #[default]
    Equal,
    /// The group fills its container, the options equal in it (`STRETCH`).
    Stretch,
    /// Each option `--choice-width`, its label cut with an ellipsis
    /// (`FIXED`).
    Fixed,
}

/// One option of a [`Choice`].
#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceOption {
    pub value: AttrValue,
    /// The text, or the accessible name of an icon-only option.
    pub label: AttrValue,
    /// An icon the app supplies, before the label.
    pub icon: Option<Html>,
    /// Only the icon is shown; the label is kept as its name.
    pub icon_only: bool,
    pub disabled: bool,
    /// The app's `data-testid` on the option.
    pub testid: Option<AttrValue>,
    /// A `data-tip` for the option (an icon-only option's hint).
    pub tip: Option<AttrValue>,
    /// The app's classes on the option.
    pub class: Classes,
    /// More `data-*` attributes (`("data-shape", ..)`) the app's code reads.
    pub data: Vec<(&'static str, AttrValue)>,
}

impl ChoiceOption {
    pub fn new(value: impl Into<AttrValue>, label: impl Into<AttrValue>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            icon_only: false,
            disabled: false,
            testid: None,
            tip: None,
            class: Classes::new(),
            data: Vec::new(),
        }
    }

    pub fn data(mut self, name: &'static str, value: impl Into<AttrValue>) -> Self {
        self.data.push((name, value.into()));
        self
    }

    pub fn testid(mut self, testid: impl Into<AttrValue>) -> Self {
        self.testid = Some(testid.into());
        self
    }

    pub fn tip(mut self, tip: impl Into<AttrValue>) -> Self {
        self.tip = Some(tip.into());
        self
    }

    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.class = class.into();
        self
    }

    pub fn icon(mut self, icon: Html) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_only(mut self, icon: Html) -> Self {
        self.icon = Some(icon);
        self.icon_only = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Properties, PartialEq)]
pub struct ChoiceProps {
    #[prop_or_default]
    pub class: Classes,
    pub options: Vec<ChoiceOption>,
    /// The selected option's value; none selected when it matches no option.
    #[prop_or_default]
    pub value: Option<AttrValue>,
    /// Called with the newly selected value, by click or arrow key.
    pub on_change: Callback<AttrValue>,
    /// What is being chosen (`aria-label` of the group).
    pub label: AttrValue,
    /// The id of the element that names the group (`aria-labelledby`),
    /// instead of `label`, which is then not set.
    #[prop_or_default]
    pub labelled_by: Option<AttrValue>,
    #[prop_or_default]
    pub fit: ChoiceFit,
    /// The small control height.
    #[prop_or_default]
    pub small: bool,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A choice's classes: `choice`, its fit, `small`, `invalid` and `disabled`.
pub fn choice(fit: ChoiceFit, small: bool, invalid: bool, disabled: bool) -> Classes {
    let fit = match fit {
        ChoiceFit::Equal => None,
        ChoiceFit::Stretch => Some(class::STRETCH),
        ChoiceFit::Fixed => Some(class::FIXED),
    };
    classes!(
        class::CHOICE,
        fit,
        small.then_some(class::SMALL),
        invalid.then_some(class::INVALID),
        disabled.then_some(class::DISABLED)
    )
}

fn disabled(options: &[ChoiceOption]) -> Vec<bool> {
    options.iter().map(|o| o.disabled).collect()
}

fn roving(options: &[ChoiceOption], value: Option<&AttrValue>) -> Option<usize> {
    let selected = options.iter().position(|o| Some(&o.value) == value);
    roving::stop(&disabled(options), selected)
}

/// A segmented pick-one group (`role="radiogroup"`): options of one width
/// whatever is selected, the selected one marked by its background alone.
/// One tab stop; arrows, Home and End move the selection and focus.
#[function_component]
pub fn Choice(props: &ChoiceProps) -> Html {
    let stop = roving(&props.options, props.value.as_ref());

    let onkeydown = {
        let options = props.options.clone();
        let on_change = props.on_change.clone();
        let disabled = props.disabled;
        Callback::from(move |e: KeyboardEvent| {
            if disabled {
                return;
            }
            let off = self::disabled(&options);
            if let Some(next) = roving::keydown(&e, "radio", &off, stop, Arrows::Both) {
                on_change.emit(options[next].value.clone());
            }
        })
    };

    html! {
        <div role="radiogroup" class={classes!(choice(props.fit, props.small, props.invalid, props.disabled), props.class.clone())}
            aria-label={props.labelled_by.is_none().then(|| props.label.clone())}
            aria-labelledby={props.labelled_by.clone()}
            aria-invalid={props.invalid.then_some("true")}
            aria-disabled={props.disabled.then_some("true")}
            data-testid={props.testid.clone()}
            {onkeydown}>
            { for props.options.iter().enumerate().map(|(i, option)| {
                let selected = props.value.as_ref() == Some(&option.value);
                let onclick = {
                    let on_change = props.on_change.clone();
                    let value = option.value.clone();
                    Callback::from(move |_: MouseEvent| on_change.emit(value.clone()))
                };
                let mut button = html! {
                    <button type="button" role="radio" class={classes!(class::CHOICE_OPTION, option.class.clone())}
                        data-testid={option.testid.clone()}
                        data-tip={option.tip.clone()}
                        aria-checked={selected.to_string()}
                        aria-label={option.icon_only.then(|| option.label.clone())}
                        title={(option.icon_only && option.tip.is_none()).then(|| option.label.clone())}
                        tabindex={if stop == Some(i) { "0" } else { "-1" }}
                        disabled={props.disabled || option.disabled}
                        {onclick}>
                        { for option.icon.clone().map(|icon| html! {
                            <span class={class::CHOICE_ICON} aria-hidden="true">{ icon }</span>
                        }) }
                        if !option.icon_only {
                            <span class={class::CHOICE_LABEL}>{ option.label.clone() }</span>
                        }
                    </button>
                };
                if let Html::VTag(tag) = &mut button {
                    for (name, value) in &option.data {
                        std::rc::Rc::make_mut(tag).add_attribute(name, value.clone());
                    }
                }
                button
            }) }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(options: &[ChoiceOption], from: usize, key: &str) -> Option<usize> {
        roving::step(&disabled(options), Some(from), key, Arrows::Both)
    }

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn options() -> Vec<ChoiceOption> {
        ["a", "b", "c"]
            .iter()
            .map(|v| ChoiceOption::new(*v, v.to_uppercase()))
            .collect()
    }

    fn render(value: Option<&str>, options: Vec<ChoiceOption>) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Choice>::with_props(yew::props!(ChoiceProps {
                options: options,
                value: value.map(AttrValue::from),
                on_change: Callback::noop(),
                label: "Pick"
            }))
            .hydratable(false)
            .render(),
        )
    }

    #[test]
    fn each_modifier_is_its_class() {
        let of = |fit, small, invalid, disabled| names(choice(fit, small, invalid, disabled));
        assert_eq!(of(ChoiceFit::Equal, false, false, false), [class::CHOICE]);
        assert_eq!(
            of(ChoiceFit::Stretch, false, false, false),
            [class::CHOICE, class::STRETCH]
        );
        assert_eq!(
            of(ChoiceFit::Fixed, false, false, false),
            [class::CHOICE, class::FIXED]
        );
        assert_eq!(
            of(ChoiceFit::Equal, true, false, false),
            [class::CHOICE, class::SMALL]
        );
        assert_eq!(
            of(ChoiceFit::Equal, false, true, false),
            [class::CHOICE, class::INVALID]
        );
        assert_eq!(
            of(ChoiceFit::Equal, false, false, true),
            [class::CHOICE, class::DISABLED]
        );
    }

    #[test]
    fn arrows_move_the_selection_and_wrap() {
        let o = options();
        assert_eq!(step(&o, 0, "ArrowRight"), Some(1));
        assert_eq!(step(&o, 2, "ArrowDown"), Some(0));
        assert_eq!(step(&o, 0, "ArrowLeft"), Some(2));
        assert_eq!(step(&o, 1, "ArrowUp"), Some(0));
        assert_eq!(step(&o, 1, "Home"), Some(0));
        assert_eq!(step(&o, 0, "End"), Some(2));
        assert_eq!(step(&o, 0, "Tab"), None);
    }

    #[test]
    fn arrows_skip_disabled_options() {
        let mut o = options();
        o[1] = o[1].clone().disabled(true);
        assert_eq!(step(&o, 0, "ArrowRight"), Some(2));
        assert_eq!(step(&o, 2, "ArrowLeft"), Some(0));
        o[0] = o[0].clone().disabled(true);
        assert_eq!(step(&o, 2, "Home"), Some(2));
    }

    #[test]
    fn only_the_selected_option_is_a_tab_stop() {
        let html = render(Some("b"), options());
        assert_eq!(html.matches("tabindex=\"0\"").count(), 1, "{html}");
        assert_eq!(html.matches("tabindex=\"-1\"").count(), 2, "{html}");
        assert!(html.find("tabindex=\"0\"") > html.find("aria-checked=\"false\""));
        assert_eq!(html.matches("aria-checked=\"true\"").count(), 1, "{html}");
    }

    #[test]
    fn with_none_selected_the_first_enabled_is_the_tab_stop() {
        let mut o = options();
        o[0] = o[0].clone().disabled(true);
        assert_eq!(roving(&o, None), Some(1));
        let html = render(None, o);
        assert_eq!(html.matches("tabindex=\"0\"").count(), 1, "{html}");
    }

    #[test]
    fn an_icon_only_option_keeps_its_name() {
        let o = vec![ChoiceOption::new("a", "Grid").icon_only(html! { "#" })];
        let html = render(Some("a"), o);
        assert!(html.contains("aria-label=\"Grid\""), "{html}");
        assert!(!html.contains(class::CHOICE_LABEL), "{html}");
    }
}
