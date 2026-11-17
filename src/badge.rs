//! Pill, Badge, Dot and Count (`style/_badges.scss`): short marks of a fixed
//! box.

use crate::class;
use yew::prelude::*;

/// The one tone vocabulary of marks and messages: a modifier class that
/// only sets `--tone` (from `--accent`, `--success`, `--warning` or
/// `--danger`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// The muted text colour; no class.
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
    /// The accent, its dot pulsing while the work runs.
    Busy,
    /// A value that is not set: outlined, no fill.
    Unset,
}

impl Tone {
    pub(crate) fn class(self) -> Option<&'static str> {
        match self {
            Tone::Neutral => None,
            Tone::Accent => Some(class::ACCENT),
            Tone::Success => Some(class::SUCCESS),
            Tone::Warning => Some(class::WARNING),
            Tone::Danger => Some(class::DANGER),
            Tone::Busy => Some(class::BUSY),
            Tone::Unset => Some(class::UNSET),
        }
    }
}

/// A [`Pill`]'s classes: `pill`, its tone, `plain` without the dot and
/// `pill-button` when it is a button.
pub fn pill(tone: Tone, plain: bool, button: bool) -> Classes {
    classes!(
        class::PILL,
        tone.class(),
        plain.then_some(class::PLAIN),
        button.then_some(class::PILL_BUTTON)
    )
}

#[derive(Properties, PartialEq)]
pub struct PillProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub tone: Tone,
    /// No leading dot.
    #[prop_or_default]
    pub plain: bool,
    /// Makes the pill a `<button>`, called when clicked.
    #[prop_or_default]
    pub on_click: Option<Callback<MouseEvent>>,
    /// Only for a pill with `on_click`.
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The label, cut with an ellipsis when the pill is out of room.
    #[prop_or_default]
    pub children: Html,
}

/// A tinted label of one height, led by a dot in its tone. Its box does not
/// change with its tone or hover.
#[function_component]
pub fn Pill(props: &PillProps) -> Html {
    let class = classes!(
        pill(props.tone, props.plain, props.on_click.is_some()),
        props.class.clone()
    );
    let label = html! { <span class={class::PILL_LABEL}>{ props.children.clone() }</span> };

    match &props.on_click {
        Some(on_click) => html! {
            <button type="button" {class} disabled={props.disabled}
                data-testid={props.testid.clone()} onclick={on_click.clone()}>
                { label }
            </button>
        },
        None => html! {
            <span {class} data-testid={props.testid.clone()}>{ label }</span>
        },
    }
}

/// A [`Badge`]'s classes: `badge`, its tone, `solid` and `corner`.
pub fn badge(tone: Tone, solid: bool, corner: bool) -> Classes {
    classes!(
        class::BADGE,
        solid.then_some(class::SOLID),
        corner.then_some(class::CORNER),
        tone.class()
    )
}

#[derive(Properties, PartialEq)]
pub struct BadgeProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub tone: Tone,
    /// Filled with the tone (the accent unless set).
    #[prop_or_default]
    pub solid: bool,
    /// Over the top end corner of its positioned parent.
    #[prop_or_default]
    pub corner: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A short text or figure in a bordered pill, as wide as tall at least.
#[function_component]
pub fn Badge(props: &BadgeProps) -> Html {
    html! {
        <span class={classes!(badge(props.tone, props.solid, props.corner), props.class.clone())}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </span>
    }
}

/// A [`Dot`]'s classes: `dot`, its tone and `on` when filled.
pub fn dot(tone: Tone, on: bool) -> Classes {
    classes!(class::DOT, tone.class(), on.then_some(class::ON))
}

#[derive(Properties, PartialEq)]
pub struct DotProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub tone: Tone,
    /// Filled; otherwise an empty ring of the same box.
    #[prop_or(true)]
    pub on: bool,
    /// What it says to a screen reader; without one the dot is decoration.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A small status circle of `--dot-size`, filled or a ring in the same box.
#[function_component]
pub fn Dot(props: &DotProps) -> Html {
    html! {
        <span class={classes!(dot(props.tone, props.on), props.class.clone())}
            role={props.label.is_some().then_some("img")}
            aria-label={props.label.clone()}
            aria-hidden={props.label.is_none().then_some("true")}
            data-testid={props.testid.clone()} />
    }
}

/// The text of `value` in a [`Count`]: `max+` above `max`.
pub fn count_text(value: u32, max: u32) -> String {
    if value > max {
        format!("{max}+")
    } else {
        value.to_string()
    }
}

/// A [`Count`]'s classes: `count`, `zero` for none and `hidden` when it is
/// hidden.
pub fn count(value: u32, hide_zero: bool) -> Classes {
    classes!(
        class::COUNT,
        (value == 0).then_some(class::ZERO),
        (value == 0 && hide_zero).then_some(class::HIDDEN)
    )
}

#[derive(Properties, PartialEq)]
pub struct CountProps {
    #[prop_or_default]
    pub class: Classes,
    pub value: u32,
    /// The largest value shown whole, above it `max+`. The count's box
    /// holds that many digits and a `+`, whatever it shows.
    #[prop_or(99)]
    pub max: u32,
    /// Hides a count of none without removing its box, so what is beside it
    /// does not move when the first one arrives.
    #[prop_or_default]
    pub hide_zero: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A figure on a soft fill, its width reserved for `max`'s digits.
#[function_component]
pub fn Count(props: &CountProps) -> Html {
    let hidden = props.hide_zero && props.value == 0;
    let digits = props.max.to_string().len();

    html! {
        <span class={classes!(count(props.value, props.hide_zero), props.class.clone())}
            style={format!("--count-digits: {digits}")}
            aria-hidden={hidden.then_some("true")}
            data-testid={props.testid.clone()}>
            { count_text(props.value, props.max) }
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(c: Classes) -> Vec<String> {
        c.into_iter().map(|c| c.to_string()).collect()
    }

    fn render<C: BaseComponent>(props: C::Properties) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<C>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn every_tone_is_its_own_class() {
        let tones = [
            (Tone::Neutral, None),
            (Tone::Accent, Some(class::ACCENT)),
            (Tone::Success, Some(class::SUCCESS)),
            (Tone::Warning, Some(class::WARNING)),
            (Tone::Danger, Some(class::DANGER)),
            (Tone::Busy, Some(class::BUSY)),
            (Tone::Unset, Some(class::UNSET)),
        ];
        for (tone, expected) in tones {
            assert_eq!(tone.class(), expected);
            assert_eq!(
                names(pill(tone, false, false)),
                [Some(class::PILL), expected]
                    .into_iter()
                    .flatten()
                    .map(String::from)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn a_pill_marks_plain_and_button() {
        assert_eq!(names(pill(Tone::Neutral, false, false)), [class::PILL]);
        assert_eq!(
            names(pill(Tone::Neutral, true, true)),
            [class::PILL, class::PLAIN, class::PILL_BUTTON]
        );
    }

    #[test]
    fn a_badge_marks_solid_and_corner() {
        assert_eq!(names(badge(Tone::Neutral, false, false)), [class::BADGE]);
        assert_eq!(
            names(badge(Tone::Danger, true, true)),
            [class::BADGE, class::SOLID, class::CORNER, class::DANGER]
        );
    }

    #[test]
    fn a_dot_marks_on() {
        assert_eq!(
            names(dot(Tone::Success, false)),
            [class::DOT, class::SUCCESS]
        );
        assert_eq!(
            names(dot(Tone::Success, true)),
            [class::DOT, class::SUCCESS, class::ON]
        );
    }

    #[test]
    fn a_count_marks_zero_and_hidden() {
        assert_eq!(names(count(3, true)), [class::COUNT]);
        assert_eq!(names(count(0, false)), [class::COUNT, class::ZERO]);
        assert_eq!(
            names(count(0, true)),
            [class::COUNT, class::ZERO, class::HIDDEN]
        );
    }

    #[test]
    fn a_count_caps_at_its_max() {
        assert_eq!(count_text(0, 99), "0");
        assert_eq!(count_text(99, 99), "99");
        assert_eq!(count_text(100, 99), "99+");
        assert_eq!(count_text(5, 9), "5");
        assert_eq!(count_text(10, 9), "9+");
    }

    #[test]
    fn a_count_reserves_its_digits_and_a_hidden_one_keeps_its_box() {
        let html = render::<Count>(yew::props!(CountProps {
            value: 0,
            max: 1000,
            hide_zero: true
        }));
        assert!(html.contains("--count-digits: 4"), "{html}");
        assert!(html.contains(class::HIDDEN), "{html}");
        assert!(html.contains(">0<"), "{html}");
    }

    #[test]
    fn a_pill_with_on_click_is_a_button() {
        let html = render::<Pill>(yew::props!(PillProps {
            on_click: Some(Callback::noop()),
            children: html! { "Core" }
        }));
        assert!(html.starts_with("<button"), "{html}");
        let html = render::<Pill>(yew::props!(PillProps {
            children: html! { "Core" }
        }));
        assert!(html.starts_with("<span"), "{html}");
    }
}
