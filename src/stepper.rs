//! Stepper (`style/_stepper.scss`).

use crate::input::{InputSize, input_box};
use crate::{Glyph, Icon, class};
use web_sys::HtmlInputElement;
use yew::prelude::*;

/// The classes of a [`Stepper`]'s box: `input-box`, its size, `invalid`,
/// `disabled` and `busy`, and `stepper`.
pub fn stepper(size: InputSize, invalid: bool, disabled: bool, busy: bool) -> Classes {
    classes!(input_box(size, invalid, disabled, busy), class::STEPPER)
}

/// The multiple of `step` one step from `value` in `dir` (`1` or `-1`):
/// from 7 by 5 it is 10 up and 5 down, from 10 it is 15 and 5. Within
/// `min..=max`. `count` steps at once for the paging keys.
pub fn step_from(value: i64, dir: i64, count: i64, step: i64, min: i64, max: i64) -> i64 {
    let step = step.max(1);
    let mut next = value;
    for _ in 0..count.max(1) {
        next = if dir > 0 {
            (next.div_euclid(step) + 1) * step
        } else if next.rem_euclid(step) == 0 {
            next - step
        } else {
            next.div_euclid(step) * step
        };
    }
    next.clamp(min, max.max(min))
}

/// The number in typed `text`, if it is one.
pub fn parse_number(text: &str) -> Option<i64> {
    text.trim().parse().ok()
}

/// The number a `live` stepper reports as `text` is typed: one within
/// `min..=max` other than `value`; nothing when not `live`.
fn live_value(live: bool, text: &str, value: i64, min: i64, max: i64) -> Option<i64> {
    parse_number(text).filter(|n| live && *n != value && (min..=max).contains(n))
}

/// The value a key moves a stepper to, if it is one of its keys: the arrows
/// step, Page Up and Down ten steps, Home and End go to the bounds.
pub fn key_value(key: &str, value: i64, step: i64, min: i64, max: i64) -> Option<i64> {
    Some(match key {
        "ArrowUp" => step_from(value, 1, 1, step, min, max),
        "ArrowDown" => step_from(value, -1, 1, step, min, max),
        "PageUp" => step_from(value, 1, 10, step, min, max),
        "PageDown" => step_from(value, -1, 10, step, min, max),
        "Home" => min,
        "End" => max.max(min),
        _ => return None,
    })
}

#[derive(Properties, PartialEq)]
pub struct StepperProps {
    pub value: i64,
    /// A number typed or stepped to, always within `min..=max`.
    pub on_change: Callback<i64>,
    pub min: i64,
    pub max: i64,
    /// How far the buttons and arrow keys step, onto its multiples.
    #[prop_or(1)]
    pub step: i64,
    /// The accessible name of the number.
    pub label: AttrValue,
    #[prop_or("Decrease".into())]
    pub less_label: AttrValue,
    #[prop_or("Increase".into())]
    pub more_label: AttrValue,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub busy: bool,
    /// Report each number typed within the bounds at once, not only when
    /// it is committed; the typed text stays until then.
    #[prop_or_default]
    pub live: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A number between a minus and a plus button, in one box. What is typed is
/// kept while it is no number within the bounds (marked invalid) and
/// clamped when committed; the buttons are out of the tab order since the
/// arrow keys step it. The number's room is that of the longest of `min`
/// and `max` in figures of one width, so the box does not change from the
/// least value to the most.
#[function_component]
pub fn Stepper(props: &StepperProps) -> Html {
    let draft = use_state(|| None::<String>);
    let (min, max) = (props.min, props.max.max(props.min));
    let chars = min.to_string().len().max(max.to_string().len());
    let shown = draft
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| props.value.to_string());
    let typed_ok = draft
        .as_deref()
        .is_none_or(|d| parse_number(d).is_some_and(|n| (min..=max).contains(&n)));

    let set = {
        let draft = draft.clone();
        let on_change = props.on_change.clone();
        let value = props.value;
        Callback::from(move |n: i64| {
            draft.set(None);
            if n != value {
                on_change.emit(n);
            }
        })
    };
    let commit = {
        let draft = draft.clone();
        let set = set.clone();
        Callback::from(move |()| match draft.as_deref().and_then(parse_number) {
            Some(n) => set.emit(n.clamp(min, max)),
            None => draft.set(None),
        })
    };

    let oninput = {
        let draft = draft.clone();
        let on_change = props.on_change.clone();
        let (value, live) = (props.value, props.live);
        Callback::from(move |e: InputEvent| {
            let text = e.target_unchecked_into::<HtmlInputElement>().value();
            if let Some(n) = live_value(live, &text, value, min, max) {
                on_change.emit(n);
            }
            draft.set(Some(text));
        })
    };
    let onchange = commit.reform(|_: Event| ());
    let onblur = commit.reform(|_: FocusEvent| ());
    let onfocus = Callback::from(|e: FocusEvent| {
        e.target_unchecked_into::<HtmlInputElement>().select();
    });
    let onkeydown = {
        let set = set.clone();
        let commit = commit.clone();
        let (value, step) = (props.value, props.step);
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                commit.emit(());
            } else if let Some(n) = key_value(&e.key(), value, step, min, max) {
                e.prevent_default();
                set.emit(n);
            }
        })
    };
    let button = |dir: i64, glyph: Glyph, label: &AttrValue, at_bound: bool| {
        let set = set.clone();
        let (value, step) = (props.value, props.step);
        let onclick = Callback::from(move |_: MouseEvent| {
            set.emit(step_from(value, dir, 1, step, min, max));
        });
        html! {
            <button type="button" class={class::STEPPER_BUTTON} tabindex="-1"
                aria-label={label.clone()} disabled={props.disabled || at_bound} {onclick}>
                <Icon {glyph} />
            </button>
        }
    };

    html! {
        <span class={classes!(
            stepper(props.size, props.invalid || !typed_ok, props.disabled, props.busy),
            props.class.clone()
        )} role="group" aria-busy={props.busy.then_some("true")}
            style={format!("--stepper-chars: {chars}")}>
            { button(-1, Glyph::Minus, &props.less_label, props.value <= min) }
            <input class={classes!(class::INPUT_CONTROL, class::STEPPER_CONTROL, (props.value == 0).then_some(class::ZERO))}
                id={props.id.clone()} type="text" inputmode="numeric" autocomplete="off"
                value={shown} aria-label={props.label.clone()}
                aria-invalid={(props.invalid || !typed_ok).then_some("true")}
                disabled={props.disabled} data-testid={props.testid.clone()}
                {oninput} {onchange} {onblur} {onfocus} {onkeydown} />
            { button(1, Glyph::Plus, &props.more_label, props.value >= max) }
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn render(props: StepperProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Stepper>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props(value: i64) -> StepperProps {
        StepperProps {
            value,
            on_change: Callback::noop(),
            min: 0,
            max: 100,
            step: 1,
            label: "Count".into(),
            less_label: "Decrease".into(),
            more_label: "Increase".into(),
            id: None,
            size: InputSize::Regular,
            invalid: false,
            disabled: false,
            busy: false,
            live: false,
            class: Classes::new(),
            testid: None,
        }
    }

    #[test]
    fn the_box_is_an_input_box_with_modifiers() {
        assert_eq!(
            names(stepper(InputSize::Regular, false, false, false)),
            [class::INPUT_BOX, class::STEPPER]
        );
        assert_eq!(
            names(stepper(InputSize::Touch, true, true, true)),
            [
                class::INPUT_BOX,
                class::TOUCH,
                class::INVALID,
                class::DISABLED,
                class::BUSY,
                class::STEPPER
            ]
        );
        assert_eq!(
            names(stepper(InputSize::Small, false, false, false)),
            [class::INPUT_BOX, class::SMALL, class::STEPPER]
        );
    }

    #[test]
    fn steps_clamp_at_the_bounds() {
        assert_eq!(step_from(99, 1, 1, 1, 0, 100), 100);
        assert_eq!(step_from(100, 1, 1, 1, 0, 100), 100);
        assert_eq!(step_from(0, -1, 1, 1, 0, 100), 0);
        assert_eq!(step_from(5, 1, 10, 1, 0, 8), 8);
        assert_eq!(key_value("Home", 40, 1, 0, 100), Some(0));
        assert_eq!(key_value("End", 40, 1, 0, 100), Some(100));
        assert_eq!(key_value("a", 40, 1, 0, 100), None);
    }

    #[test]
    fn steps_round_onto_multiples_of_the_step() {
        assert_eq!(step_from(7, 1, 1, 5, 0, 100), 10);
        assert_eq!(step_from(7, -1, 1, 5, 0, 100), 5);
        assert_eq!(step_from(10, 1, 1, 5, 0, 100), 15);
        assert_eq!(step_from(10, -1, 1, 5, 0, 100), 5);
        assert_eq!(step_from(-7, -1, 1, 5, -100, 100), -10);
        assert_eq!(step_from(7, 1, 10, 5, 0, 100), 55);
        assert_eq!(key_value("ArrowUp", 7, 5, 0, 100), Some(10));
        assert_eq!(key_value("PageDown", 60, 5, 0, 100), Some(10));
    }

    #[test]
    fn a_live_stepper_reports_numbers_within_its_bounds_as_typed() {
        assert_eq!(live_value(true, "12", 7, 0, 100), Some(12));
        assert_eq!(live_value(true, "7", 7, 0, 100), None);
        assert_eq!(live_value(true, "150", 7, 0, 100), None);
        assert_eq!(live_value(true, "", 7, 0, 100), None);
        assert_eq!(live_value(false, "12", 7, 0, 100), None);
    }

    #[test]
    fn the_number_is_marked_at_zero() {
        assert!(render(props(0)).contains(class::ZERO));
        assert!(!render(props(5)).contains(class::ZERO));
    }

    #[test]
    fn typed_text_is_a_number_or_nothing() {
        assert_eq!(parse_number(" 12 "), Some(12));
        assert_eq!(parse_number("-3"), Some(-3));
        assert_eq!(parse_number("1."), None);
        assert_eq!(parse_number(""), None);
    }

    #[test]
    fn the_number_has_the_room_of_its_longest_bound() {
        let at_min = render(props(0));
        let at_max = render(props(100));
        assert!(at_min.contains("--stepper-chars: 3"), "{at_min}");
        assert!(at_max.contains("--stepper-chars: 3"), "{at_max}");
        let negative = render(StepperProps {
            min: -50,
            max: 9,
            ..props(0)
        });
        assert!(negative.contains("--stepper-chars: 3"), "{negative}");
    }

    #[test]
    fn a_button_at_its_bound_is_disabled_in_place() {
        let at_min = render(props(0));
        let mid = render(props(5));
        assert_eq!(at_min.matches(class::STEPPER_BUTTON).count(), 2);
        assert_eq!(
            at_min.matches("disabled=\"disabled\"").count(),
            1,
            "{at_min}"
        );
        assert_eq!(mid.matches("disabled=\"disabled\"").count(), 0, "{mid}");
        let off = render(StepperProps {
            disabled: true,
            ..props(5)
        });
        assert_eq!(off.matches("disabled=\"disabled\"").count(), 3, "{off}");
    }
}
