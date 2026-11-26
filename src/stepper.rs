//! Stepper (`style/_stepper.scss`).

use crate::input::{InputSize, input_box};
use crate::phrasebookcatalog01::{L10N, label};
use crate::testid::with_testid;
use crate::timer::Timer;
use crate::{Glyph, Icon, class, use_field};
use phrasebook02::yew023::use_phrases_with;
use std::rc::Rc;
use wasm_bindgen::JsCast;
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
    parse_decimal(text, 0)
}

/// The number typed as `text` in units of the last of `decimals` digits:
/// at most that many after a point or comma (none, and no separator, for
/// 0), a sign allowed.
pub fn parse_decimal(text: &str, decimals: u8) -> Option<i64> {
    let text = text.trim();
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (whole, part) = digits.split_once(['.', ',']).unwrap_or((digits, ""));

    if whole.is_empty() && part.is_empty()
        || part.len() > usize::from(decimals)
        || digits.len() != whole.len() + part.len() && decimals == 0
        || !whole
            .chars()
            .chain(part.chars())
            .all(|c| c.is_ascii_digit())
    {
        return None;
    }

    let scale = 10_i64.pow(u32::from(decimals));
    let whole = if whole.is_empty() {
        0
    } else {
        whole.parse::<i64>().ok()?
    };
    let part = format!("{part:0<width$}", width = usize::from(decimals));
    let part = if part.is_empty() {
        0
    } else {
        part.parse::<i64>().ok()?
    };
    let value = whole.checked_mul(scale)?.checked_add(part)?;
    Some(if negative { -value } else { value })
}

/// `value` as a stepper writes it: its last `decimals` digits after the
/// point, trailing zeros left out, and at least `pad` digits before it.
pub fn format_number(value: i64, decimals: u8, pad: u8) -> String {
    let scale = 10_i64.pow(u32::from(decimals));
    let sign = if value < 0 { "-" } else { "" };
    let (whole, part) = (value.abs() / scale, value.abs() % scale);
    let whole = format!("{sign}{whole:0pad$}", pad = usize::from(pad));

    if part == 0 {
        return whole;
    }

    let part = format!("{part:0width$}", width = usize::from(decimals));
    format!("{whole}.{}", part.trim_end_matches('0'))
}

/// The characters the box of a stepper must hold for the bounds `min` and
/// `max`: the sign, the integer digits of the wider bound (at least `pad`),
/// and the separator with every one of `decimals` places, whatever trailing
/// zeros `format_number` leaves out. With no decimals, the longest of the
/// two as `format_number` writes them.
pub fn number_width(min: i64, max: i64, decimals: u8, pad: u8) -> usize {
    let scale = 10_i64.pow(u32::from(decimals));
    let one = |n: i64| {
        let whole = (n / scale) * scale;
        let sign = usize::from(n < 0 && whole == 0);
        sign + format_number(whole, decimals, pad).len()
    };
    let places = if decimals == 0 {
        0
    } else {
        1 + usize::from(decimals)
    };
    one(min).max(one(max)) + places
}

/// `value` moved by `delta` onto the nearest multiple of `snap` in that
/// direction, within `min..=max`: from 7, +5 onto 5s is 10 and -5 is 5.
pub fn step_by(value: i64, delta: i64, snap: i64, min: i64, max: i64) -> i64 {
    let snap = snap.max(1);
    let next = value + delta;
    let next = if delta >= 0 {
        next.div_euclid(snap) * snap
    } else {
        -(-next).div_euclid(snap) * snap
    };
    next.clamp(min, max.max(min))
}

/// As [`step_by`], but a step that would pass a bound is refused (`None`)
/// rather than stopped at it.
pub fn step_within(value: i64, delta: i64, snap: i64, min: i64, max: i64) -> Option<i64> {
    let next = step_by(value, delta, snap, i64::MIN, i64::MAX);
    (min..=max).contains(&next).then_some(next)
}

/// The number a `live` stepper reports as `text` is typed: one within
/// `min..=max` other than `value`; nothing when not `live`.
fn live_value(live: bool, text: &str, value: i64, decimals: u8, min: i64, max: i64) -> Option<i64> {
    parse_decimal(text, decimals).filter(|n| live && *n != value && (min..=max).contains(n))
}

/// What a step starts from: the number typed and not yet committed
/// (within the bounds), else `value`.
fn step_base(draft: Option<&str>, value: i64, decimals: u8, min: i64, max: i64) -> i64 {
    draft
        .and_then(|d| parse_decimal(d, decimals))
        .map_or(value, |n| n.clamp(min, max))
}

/// How far a key steps and onto what multiple, if it is one of a stepper's
/// step keys: the arrows step by `arrow` (onto no multiple) or else by
/// `step` onto multiples of `snap`; Page Up and Down ten times as far.
pub fn key_step(key: &str, step: i64, snap: i64, arrow: Option<i64>) -> Option<(i64, i64)> {
    let (by, snap) = match arrow {
        Some(arrow) => (arrow, 1),
        None => (step, snap),
    };

    Some(match key {
        "ArrowUp" => (by, snap),
        "ArrowDown" => (-by, snap),
        "PageUp" => (by * 10, snap),
        "PageDown" => (-by * 10, snap),
        _ => return None,
    })
}

/// The value a key moves a stepper to, if it is one of its keys: the arrows
/// step, Page Up and Down ten steps, Home and End go to the bounds.
pub fn key_value(key: &str, value: i64, step: i64, min: i64, max: i64) -> Option<i64> {
    match key {
        "Home" => Some(min),
        "End" => Some(max.max(min)),
        _ => key_step(key, step, step, None).map(|(by, snap)| step_by(value, by, snap, min, max)),
    }
}

/// The value to report as committed: `pending` (the last number stepped or
/// typed since the last commit), when it is not the one `committed` already.
fn commit_due(pending: Option<i64>, committed: Option<i64>) -> Option<i64> {
    pending.filter(|n| Some(*n) != committed)
}

/// How long after the last step, in milliseconds, `on_commit` waits.
const COMMIT_DELAY: u32 = 500;

#[derive(Properties, PartialEq)]
pub struct StepperProps {
    /// The number, in units of the last of [`StepperProps::decimals`].
    pub value: i64,
    /// A number typed or stepped to, always within `min..=max`.
    pub on_change: Callback<i64>,
    /// The settled number, once: when a burst of steps has been quiet for
    /// `commit_delay`, when focus leaves the control, or when a typed number
    /// is committed (Enter, blur); never the number last committed again.
    /// `on_change` still fires on each step.
    #[prop_or_default]
    pub on_commit: Option<Callback<i64>>,
    /// The quiet after the last step before `on_commit`, in milliseconds.
    #[prop_or(COMMIT_DELAY)]
    pub commit_delay: u32,
    pub min: i64,
    pub max: i64,
    /// How far the buttons and arrow keys step, onto its multiples.
    #[prop_or(1)]
    pub step: i64,
    /// What the buttons and arrow keys step onto multiples of, in their
    /// direction; `step` when unset.
    #[prop_or_default]
    pub snap: Option<i64>,
    /// How far the arrow keys step, onto no multiple, instead of `step`
    /// onto multiples of `snap`.
    #[prop_or_default]
    pub arrow: Option<i64>,
    /// How many of the value's last digits follow the decimal point.
    #[prop_or_default]
    pub decimals: u8,
    /// The fewest digits shown before the point, padded with zeros.
    #[prop_or_default]
    pub pad: u8,
    /// The accessible name of the number.
    pub label: AttrValue,
    #[prop_or_default]
    pub less_label: Option<AttrValue>,
    #[prop_or_default]
    pub more_label: Option<AttrValue>,
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
    /// Nothing is clamped: what is typed outside the bounds, or no number,
    /// stays, marked, until it is a number within them (a step from it
    /// lands on the nearest bound), and a step that would pass a bound is
    /// refused.
    #[prop_or_default]
    pub strict: bool,
    /// Whether what is typed is a number within `min..=max`, told as it
    /// changes and when it is left.
    #[prop_or_default]
    pub on_valid: Callback<bool>,
    /// Takes the steps instead, as how far and onto what multiple, for an
    /// owner whose step carries into other fields; the Home and End keys
    /// are then idle, and `can_less` and `can_more` say which way it may
    /// step.
    #[prop_or_default]
    pub on_step: Option<Callback<(i64, i64)>>,
    #[prop_or(true)]
    pub can_less: bool,
    #[prop_or(true)]
    pub can_more: bool,
    /// The values it stands for differ, or it has none (see `on_empty`):
    /// the field shows none, `placeholder` in its place, until a number is
    /// typed or stepped to (from `value`).
    #[prop_or_default]
    pub mixed: bool,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    /// The number may be left out: an emptied field tells this instead of
    /// being marked, and its owner then shows it `mixed`.
    #[prop_or_default]
    pub on_empty: Option<Callback<()>>,
    /// Takes the focus when its dialog or popup opens (`data-autofocus`).
    #[prop_or_default]
    pub autofocus: bool,
    /// What the number counts (%, cells, ft), shown after the box on its
    /// line as part of the control (`.sockel-stepper-field`).
    #[prop_or_default]
    pub unit: Option<AttrValue>,
    /// The unit follows the box as an item of its own, not wrapped with it
    /// in `.sockel-stepper-field`, for a grid with a column of units.
    #[prop_or_default]
    pub unit_apart: bool,
    /// As wide as its container, the number taking the room
    /// (`.sockel-fill`).
    #[prop_or_default]
    pub fill: bool,
    /// A default stands in for the value: the number is quieter
    /// (`.sockel-faint`).
    #[prop_or_default]
    pub faint: bool,
    /// Shown in place of the number, which is then stepped and never typed,
    /// as a length written "4 h 30 min" (`aria-valuetext`).
    #[prop_or_default]
    pub text: Option<AttrValue>,
    /// The number's room in characters, for a `text` whose longest the
    /// stepper cannot tell (the longest of `min`, `max` and `placeholder`
    /// when unset).
    #[prop_or_default]
    pub chars: Option<u8>,
    #[prop_or_default]
    pub class: Classes,
    /// The number's test id; the buttons have it with `-less` and `-more`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

impl Default for StepperProps {
    fn default() -> Self {
        Self {
            value: Default::default(),
            on_change: Default::default(),
            on_commit: Default::default(),
            commit_delay: COMMIT_DELAY,
            min: Default::default(),
            max: Default::default(),
            step: ::yew::html::IntoPropValue::into_prop_value(1),
            snap: Default::default(),
            arrow: Default::default(),
            decimals: Default::default(),
            pad: Default::default(),
            label: Default::default(),
            less_label: Default::default(),
            more_label: Default::default(),
            id: Default::default(),
            size: Default::default(),
            invalid: Default::default(),
            disabled: Default::default(),
            busy: Default::default(),
            live: Default::default(),
            strict: Default::default(),
            on_valid: Default::default(),
            on_step: Default::default(),
            can_less: ::yew::html::IntoPropValue::into_prop_value(true),
            can_more: ::yew::html::IntoPropValue::into_prop_value(true),
            mixed: Default::default(),
            placeholder: Default::default(),
            on_empty: Default::default(),
            autofocus: Default::default(),
            unit: Default::default(),
            unit_apart: Default::default(),
            fill: Default::default(),
            faint: Default::default(),
            text: Default::default(),
            chars: Default::default(),
            class: Default::default(),
            testid: Default::default(),
        }
    }
}

/// A number between a minus and a plus button, in one box. What is typed is
/// kept while it is no number within the bounds (marked invalid) and
/// clamped when committed (unless `strict`); the buttons are out of the tab
/// order since the arrow keys step it. The number's room is that of the
/// longest of `min` and `max` in figures of one width (with every decimal place counted), so the box does not
/// change from the least value to the most.
#[function_component]
pub fn Stepper(props: &StepperProps) -> Html {
    let (field_id, describedby, invalid) =
        use_field()
            .unwrap_or_default()
            .control(&props.id, &None, props.invalid);
    let phrases = use_phrases_with(&L10N);
    let less_label = label(&phrases, &props.less_label, "sockel-stepper-decrease");
    let more_label = label(&phrases, &props.more_label, "sockel-stepper-increase");
    let draft = use_state(|| None::<String>);
    let (min, max) = (props.min, props.max.max(props.min));
    let (decimals, pad) = (props.decimals, props.pad);
    let (strict, live, mixed) = (props.strict, props.live, props.mixed);
    let chars = props.chars.map_or_else(
        || {
            props
                .placeholder
                .iter()
                .map(|p| p.chars().count())
                .fold(number_width(min, max, decimals, pad), usize::max)
        },
        usize::from,
    );
    let shown = match draft.as_deref() {
        _ if props.text.is_some() => props.text.as_deref().unwrap_or_default().to_owned(),
        Some(draft) => draft.to_owned(),
        None if mixed => String::new(),
        None => format_number(props.value, decimals, pad),
    };
    let typed = draft
        .as_deref()
        .map_or(Some(props.value), |d| parse_decimal(d, decimals));
    let typed_ok = typed.is_some_and(|n| (min..=max).contains(&n));

    // A draft the value has left stays only while it says the value, such
    // as `1.` on the way to `1.5`.
    {
        let draft = draft.clone();
        let on_valid = props.on_valid.clone();
        use_effect_with(props.value, move |value| {
            if let Some(text) = draft.as_deref()
                && parse_decimal(text, decimals) != Some(*value)
            {
                draft.set(None);
                on_valid.emit(true);
            }
        });
    }

    // What `on_commit` has yet to tell, what it last told, and the wait.
    let on_commit = use_mut_ref(|| None::<Callback<i64>>);
    *on_commit.borrow_mut() = props.on_commit.clone();
    let pending = use_mut_ref(|| None::<i64>);
    let committed = use_mut_ref(|| (!mixed).then_some(props.value));
    let timer = use_mut_ref(|| None::<Timer>);
    {
        let (pending, committed) = (pending.clone(), committed.clone());
        use_effect_with((props.value, mixed), move |(value, mixed)| {
            if pending.borrow().is_none() {
                *committed.borrow_mut() = (!mixed).then_some(*value);
            }
        });
    }
    let settle = {
        let (on_commit, pending, committed) =
            (on_commit.clone(), pending.clone(), committed.clone());
        Rc::new(move || {
            let due = commit_due(pending.borrow_mut().take(), *committed.borrow());
            if let (Some(n), Some(on_commit)) = (due, on_commit.borrow().as_ref()) {
                *committed.borrow_mut() = Some(n);
                on_commit.emit(n);
            }
        })
    };
    // Tells `on_commit` now, and stops the wait.
    let flush = {
        let (settle, timer) = (settle.clone(), timer.clone());
        Callback::from(move |()| {
            timer.borrow_mut().take();
            settle();
        })
    };
    let delay = props.commit_delay;
    let set = {
        let draft = draft.clone();
        let on_change = props.on_change.clone();
        let on_valid = props.on_valid.clone();
        let value = props.value;
        let (settle, pending, timer) = (settle.clone(), pending.clone(), timer.clone());
        let wanted = props.on_commit.is_some();
        Callback::from(move |n: i64| {
            draft.set(None);
            on_valid.emit(true);
            if n != value || mixed {
                on_change.emit(n);
            }
            if wanted {
                *pending.borrow_mut() = Some(n);
                let settle = settle.clone();
                *timer.borrow_mut() = Timer::start(delay, move || settle());
            }
        })
    };
    let commit = {
        let draft = draft.clone();
        let set = set.clone();
        let flush = flush.clone();
        let on_valid = props.on_valid.clone();
        Callback::from(move |()| {
            let Some(text) = draft.as_deref() else {
                return;
            };
            let typed = parse_decimal(text, decimals);

            if strict {
                match typed.filter(|n| (min..=max).contains(n)) {
                    // Told as it was typed, when it was a number within.
                    Some(_) if live => {
                        draft.set(None);
                        on_valid.emit(true);
                    }
                    Some(n) => set.emit(n),
                    None => {}
                }
            } else {
                match typed {
                    Some(n) => set.emit(n.clamp(min, max)),
                    None => draft.set(None),
                }
            }
            flush.emit(());
        })
    };

    let oninput = {
        let draft = draft.clone();
        let on_change = props.on_change.clone();
        let on_valid = props.on_valid.clone();
        let on_empty = props.on_empty.clone();
        let value = props.value;
        let pending = pending.clone();
        let wanted = props.on_commit.is_some();
        Callback::from(move |e: InputEvent| {
            let text = e.target_unchecked_into::<HtmlInputElement>().value();

            if let Some(on_empty) = on_empty.as_ref().filter(|_| text.trim().is_empty()) {
                draft.set(None);
                on_valid.emit(true);
                if !mixed {
                    on_empty.emit(());
                }
                return;
            }

            let within = parse_decimal(&text, decimals).filter(|n| (min..=max).contains(n));
            on_valid.emit(within.is_some());
            if let Some(n) = live_value(live, &text, value, decimals, min, max) {
                on_change.emit(n);
                if wanted {
                    *pending.borrow_mut() = Some(n);
                }
            }
            draft.set(Some(text));
        })
    };
    let onchange = commit.reform(|_: Event| ());
    let onblur = commit.reform(|_: FocusEvent| ());
    // Focus leaving the whole control, not moving between its parts.
    let onfocusout = {
        let flush = flush.clone();
        Callback::from(move |e: FocusEvent| {
            let inside = e
                .related_target()
                .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
                .zip(
                    e.current_target()
                        .and_then(|t| t.dyn_into::<web_sys::Node>().ok()),
                )
                .is_some_and(|(target, group)| group.contains(Some(&target)));
            if !inside {
                flush.emit(());
            }
        })
    };
    let onfocus = Callback::from(|e: FocusEvent| {
        e.target_unchecked_into::<HtmlInputElement>().select();
    });

    // What a step starts from: what is typed, which a strict stepper takes
    // as it is (a step from outside lands on the nearest bound).
    let base = if strict {
        typed
    } else {
        Some(step_base(draft.as_deref(), props.value, decimals, min, max))
    };
    let step = {
        let set = set.clone();
        let draft = draft.clone();
        let on_step = props.on_step.clone();
        Callback::from(move |(by, snap): (i64, i64)| {
            let Some(base) = base else {
                return;
            };

            if base < min {
                set.emit(min);
            } else if base > max {
                set.emit(max);
            } else if let Some(on_step) = &on_step {
                draft.set(None);
                on_step.emit((by, snap));
            } else if !strict {
                set.emit(step_by(base, by, snap, min, max));
            } else if let Some(next) = step_within(base, by, snap, min, max) {
                set.emit(next);
            }
        })
    };
    let snap = props.snap.unwrap_or(props.step);
    let onkeydown = {
        let set = set.clone();
        let step = step.clone();
        let commit = commit.clone();
        let (by, arrow, on_step) = (props.step, props.arrow, props.on_step.is_some());
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();

            if key == "Enter" {
                commit.emit(());
            } else if let Some(by_snap) = key_step(&key, by, snap, arrow) {
                e.prevent_default();
                step.emit(by_snap);
            } else if !on_step && (key == "Home" || key == "End") && base.is_some() {
                e.prevent_default();
                set.emit(if key == "End" { max } else { min });
            }
        })
    };
    // Outside its bounds it steps back onto the nearest.
    let button = |dir: i64, glyph: Glyph, label: &AttrValue, suffix: &str, may: bool| {
        let step = step.clone();
        let by = dir * props.step;
        let can = !props.disabled
            && base.is_some_and(|n| if dir < 0 { n > min } else { n < max })
            && (may || !typed_ok);
        let onclick = Callback::from(move |_: MouseEvent| step.emit((by, snap)));
        with_testid(
            html! {
                <button type="button" class={class::STEPPER_BUTTON} tabindex="-1"
                    aria-label={label.clone()} data-tip={label.clone()}
                    disabled={!can} {onclick}>
                    <Icon {glyph} />
                </button>
            },
            props.testid.as_ref().map(|id| format!("{id}-{suffix}")),
        )
    };
    let invalid = invalid || !typed_ok;

    let stepper = html! {
        <span class={classes!(
            stepper(props.size, invalid, props.disabled, props.busy),
            props.fill.then_some(class::FILL),
            props.faint.then_some(class::FAINT),
            props.class.clone()
        )} role="group" aria-busy={props.busy.then_some("true")} {onfocusout}
            style={format!("--stepper-chars: {chars}")}>
            { button(-1, Glyph::Minus, &less_label, "less", props.can_less) }
            { with_testid(html! { <input class={classes!(class::INPUT_CONTROL, class::STEPPER_CONTROL, (props.value == 0 && !mixed).then_some(class::ZERO))}
                id={field_id} type="text" role="spinbutton" autocomplete="off"
                inputmode={if min < 0 { "text" } else if decimals > 0 { "decimal" } else { "numeric" }}
                value={shown} aria-label={props.label.clone()}
                placeholder={props.placeholder.clone().filter(|_| mixed)}
                readonly={props.text.is_some()} aria-valuetext={props.text.clone()}
                aria-valuenow={typed.map(|n| format_number(n, decimals, 0))}
                aria-valuemin={format_number(min, decimals, 0)}
                aria-valuemax={format_number(max, decimals, 0)}
                aria-describedby={describedby} aria-invalid={invalid.then_some("true")}
                data-autofocus={props.autofocus.then_some("")}
                disabled={props.disabled}
                {oninput} {onchange} {onblur} {onfocus} {onkeydown} /> }, props.testid.clone()) }
            { button(1, Glyph::Plus, &more_label, "more", props.can_more) }
        </span>
    };

    match &props.unit {
        Some(unit) if props.unit_apart => html! {
            <>
                { stepper }
                <span class={class::STEPPER_UNIT}>{ unit.clone() }</span>
            </>
        },
        Some(unit) => html! {
            <span class={classes!(class::STEPPER_FIELD, props.fill.then_some(class::FILL))}>
                { stepper }
                <span class={class::STEPPER_UNIT}>{ unit.clone() }</span>
            </span>
        },
        None => stepper,
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
            on_commit: None,
            commit_delay: COMMIT_DELAY,
            min: 0,
            max: 100,
            step: 1,
            snap: None,
            arrow: None,
            decimals: 0,
            pad: 0,
            label: "Count".into(),
            less_label: None,
            more_label: None,
            id: None,
            size: InputSize::Regular,
            invalid: false,
            disabled: false,
            busy: false,
            live: false,
            strict: false,
            on_valid: Callback::noop(),
            on_step: None,
            can_less: true,
            can_more: true,
            mixed: false,
            placeholder: None,
            on_empty: None,
            autofocus: false,
            unit: None,
            unit_apart: false,
            fill: false,
            faint: false,
            text: None,
            chars: None,
            class: Classes::new(),
            testid: None,
        }
    }

    #[test]
    fn a_commit_is_due_for_a_number_not_yet_committed() {
        assert_eq!(commit_due(Some(5), Some(4)), Some(5));
        assert_eq!(commit_due(Some(5), None), Some(5));
        assert_eq!(commit_due(Some(5), Some(5)), None);
        assert_eq!(commit_due(None, Some(5)), None);
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
        assert_eq!(live_value(true, "12", 7, 0, 0, 100), Some(12));
        assert_eq!(live_value(true, "7", 7, 0, 0, 100), None);
        assert_eq!(live_value(true, "150", 7, 0, 0, 100), None);
        assert_eq!(live_value(true, "", 7, 0, 0, 100), None);
        assert_eq!(live_value(false, "12", 7, 0, 0, 100), None);
    }

    #[test]
    fn a_step_starts_from_what_is_typed() {
        assert_eq!(step_base(Some("3"), 0, 0, 0, 100), 3);
        assert_eq!(step_base(Some("150"), 0, 0, 0, 100), 100);
        assert_eq!(step_base(Some("x"), 7, 0, 0, 100), 7);
        assert_eq!(step_base(None, 7, 0, 0, 100), 7);
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

    #[test]
    fn width_counts_every_decimal_place() {
        let wide = |n, decimals| format_number(n, decimals, 0).len();
        assert_eq!(number_width(0, 100, 2, 0), 4);
        assert!(number_width(0, 100, 2, 0) >= wide(7, 2));
        assert!(number_width(0, 100, 2, 0) >= wide(100, 2).max("1.00".len()));
        assert_eq!(number_width(0, 10_000, 2, 0), 6);
        assert_eq!(number_width(-50, 50, 2, 0), 5);
        assert_eq!(number_width(-50, 50, 1, 0), 4);
        assert_eq!(number_width(0, 100, 0, 0), 3);
        assert_eq!(number_width(-5, 120, 0, 0), 3);
        assert_eq!(number_width(1, 12, 0, 3), 3);
    }

    #[test]
    fn numbers_are_written_and_read_with_their_decimals() {
        assert_eq!(format_number(12, 0, 0), "12");
        assert_eq!(format_number(3, 0, 2), "03");
        assert_eq!(format_number(150, 2, 0), "1.5");
        assert_eq!(format_number(5, 2, 0), "0.05");
        assert_eq!(format_number(-200, 2, 0), "-2");

        assert_eq!(parse_decimal("1.5", 2), Some(150));
        assert_eq!(parse_decimal("1,25", 2), Some(125));
        assert_eq!(parse_decimal("1.", 2), Some(100));
        assert_eq!(parse_decimal(".5", 2), Some(50));
        assert_eq!(parse_decimal("1.5", 0), None);
        assert_eq!(parse_decimal("1.", 0), None);
        assert_eq!(parse_decimal("1.234", 2), None);
        assert_eq!(parse_decimal("x", 0), None);
        assert_eq!(parse_decimal("-", 0), None);
    }

    #[test]
    fn steps_snap_in_their_direction() {
        assert_eq!(step_by(3, 5, 5, 0, 100), 5);
        assert_eq!(step_by(7, -5, 5, 0, 100), 5);
        assert_eq!(step_by(-3, -5, 5, -100, 100), -5);
        assert_eq!(step_by(98, 5, 5, 0, 100), 100);
        assert_eq!(key_step("ArrowUp", 5, 5, None), Some((5, 5)));
        assert_eq!(key_step("PageDown", 5, 5, Some(1)), Some((-10, 1)));
        assert_eq!(key_step("Home", 5, 5, None), None);
    }

    #[test]
    fn a_unit_wraps_the_box_and_the_buttons_carry_the_test_id() {
        let html = render(StepperProps {
            unit: Some("ft".into()),
            testid: Some("reach".into()),
            ..props(5)
        });
        assert!(html.contains(class::STEPPER_FIELD), "{html}");
        assert!(
            html.contains(&format!("{}=\"reach-less\"", crate::testid::ATTR)),
            "{html}"
        );
        assert!(
            html.contains(&format!("{}=\"reach-more\"", crate::testid::ATTR)),
            "{html}"
        );
    }

    #[test]
    fn a_mixed_stepper_shows_its_placeholder_in_place_of_a_number() {
        let html = render(StepperProps {
            mixed: true,
            placeholder: Some("mixed".into()),
            ..props(5)
        });
        assert!(html.contains("placeholder=\"mixed\""), "{html}");
        assert!(html.contains("value=\"\""), "{html}");
    }

    #[test]
    fn a_step_within_refuses_to_pass_a_bound() {
        assert_eq!(step_within(3, 1, 1, 1, 10), Some(4));
        assert_eq!(step_within(7, -5, 5, 0, 100), Some(5));
        assert_eq!(step_within(98, 5, 5, 0, 100), Some(100));
        assert_eq!(step_within(97, 5, 1, 0, 100), None);
        assert_eq!(step_within(1, -1, 1, 1, 10), None);
        assert_eq!(step_within(10, 1, 1, 1, 10), None);
    }

    #[test]
    fn a_text_stands_in_for_the_number_read_only() {
        let html = render(StepperProps {
            text: Some("4 h 30 min".into()),
            chars: Some(11),
            ..props(270)
        });
        assert!(html.contains("value=\"4 h 30 min\""), "{html}");
        assert!(html.contains("aria-valuetext=\"4 h 30 min\""), "{html}");
        assert!(html.contains("readonly"), "{html}");
        assert!(html.contains("--stepper-chars: 11"), "{html}");
    }

    #[test]
    fn fill_faint_and_a_unit_apart_are_on_the_box() {
        let html = render(StepperProps {
            unit: Some("ft".into()),
            unit_apart: true,
            fill: true,
            faint: true,
            ..props(5)
        });
        assert!(!html.contains(class::STEPPER_FIELD), "{html}");
        assert!(html.contains(class::STEPPER_UNIT), "{html}");
        assert!(html.contains(class::FILL), "{html}");
        assert!(html.contains(class::FAINT), "{html}");
        let field = render(StepperProps {
            unit: Some("ft".into()),
            fill: true,
            ..props(5)
        });
        assert_eq!(field.matches(class::FILL).count(), 2, "{field}");
    }

    #[test]
    fn can_less_and_can_more_disable_the_buttons() {
        let html = render(StepperProps {
            can_less: false,
            ..props(5)
        });
        assert_eq!(html.matches("disabled=\"disabled\"").count(), 1, "{html}");
    }
}
