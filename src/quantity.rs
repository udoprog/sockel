//! QuantityField (`style/_quantity.scss`).

use crate::phrasebookcatalog01::{L10N, label};
use crate::stepper::number_width;
use crate::{InputSize, Select, SelectOption, Stepper, class};
use phrasebook02::yew023::use_phrases_with;
use yew::prelude::*;

/// A unit of a [`QuantityField`]: its label and how many base units it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub label: AttrValue,
    pub factor: i64,
}

impl Unit {
    pub fn new(label: impl Into<AttrValue>, factor: i64) -> Self {
        Self {
            label: label.into(),
            factor: factor.max(1),
        }
    }
}

/// Seconds, minutes, hours, days and weeks, in seconds. Labels are English
/// abbreviations; an app localizing them builds its own [`Unit`]s.
pub fn durations() -> Vec<Unit> {
    vec![
        Unit::new("s", 1),
        Unit::new("min", 60),
        Unit::new("h", 3_600),
        Unit::new("d", 86_400),
        Unit::new("w", 604_800),
    ]
}

/// Bytes, KiB, MiB and GiB, in bytes.
pub fn byte_sizes() -> Vec<Unit> {
    vec![
        Unit::new("B", 1),
        Unit::new("KiB", 1 << 10),
        Unit::new("MiB", 1 << 20),
        Unit::new("GiB", 1 << 30),
    ]
}

/// The classes of a [`QuantityField`]: `quantity`.
pub fn quantity() -> Classes {
    classes!(class::QUANTITY)
}

/// The index of the largest of `units` (ascending) that divides `value`
/// exactly, so it reads as the fewest digits; the smallest when none does,
/// and for 0.
pub fn readable_unit(value: i64, units: &[Unit]) -> usize {
    if value == 0 {
        return 0;
    }
    units
        .iter()
        .rposition(|unit| value % unit.factor == 0)
        .unwrap_or(0)
}

/// The least and greatest whole numbers of `factor` within `min..=max`.
pub fn unit_bounds(min: i64, max: i64, factor: i64) -> (i64, i64) {
    (-(-min).div_euclid(factor), max.div_euclid(factor))
}

/// The base value after `value` is shown in `factor`: itself when it
/// divides, else rounded to the nearest whole number of `factor` within
/// `min..=max` (to the bound when no whole number fits).
pub fn convert(value: i64, factor: i64, min: i64, max: i64) -> i64 {
    if value % factor == 0 {
        return value.clamp(min, max.max(min));
    }
    let (lo, hi) = unit_bounds(min, max, factor);
    let n = (value + factor / 2).div_euclid(factor);

    if lo <= hi {
        n.clamp(lo, hi) * factor
    } else {
        n.saturating_mul(factor).clamp(min, max.max(min))
    }
}

#[derive(Properties, PartialEq)]
pub struct QuantityFieldProps {
    /// The quantity in base units; `None` is off (see `off_label`).
    pub value: Option<i64>,
    /// A quantity typed, stepped or converted to, within `min..=max`; `None`
    /// when off is picked.
    pub on_change: Callback<Option<i64>>,
    /// The units, ascending by factor. The base unit need not be among them.
    pub units: Vec<Unit>,
    /// The least and greatest quantity, in base units. The number's room is
    /// that of the longest bound in any unit, so set a real `max`.
    pub min: i64,
    pub max: i64,
    /// The accessible name of the number.
    pub label: AttrValue,
    /// The accessible name of the unit select.
    #[prop_or_default]
    pub unit_label: Option<AttrValue>,
    /// Offers off as a choice in the unit select, with this label.
    #[prop_or_default]
    pub off_label: Option<AttrValue>,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// The number's test id; the unit select's is it suffixed `-unit`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A [`Stepper`] and a unit [`Select`] holding one quantity in base units.
/// The number shows in the largest unit that divides it exactly; picking
/// another unit converts it, rounding and clamping when it does not divide.
/// The select and the number keep their width across units.
#[function_component]
pub fn QuantityField(props: &QuantityFieldProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let unit_label = label(&phrases, &props.unit_label, "sockel-quantity-unit");
    let units = &props.units;
    let last = units.len().saturating_sub(1);
    let (min, max) = (props.min, props.max.max(props.min));
    let unit = use_state(|| props.value.map_or(0, |v| readable_unit(v, units)));
    // What this field last reported: a value other than it came from outside
    // and is shown in its most readable unit.
    let sent = use_mut_ref(|| props.value);

    if units.is_empty() {
        return html! {};
    }

    let current = (*unit).min(last);
    let shown = match props.value {
        Some(v) if *sent.borrow() == props.value && v % units[current].factor == 0 => current,
        Some(v) => readable_unit(v, units),
        None => current,
    };
    let factor = units[shown].factor;
    let (lo, hi) = unit_bounds(min, max, factor);
    let chars = units
        .iter()
        .map(|unit| {
            let (lo, hi) = unit_bounds(min, max, unit.factor);
            number_width(lo, hi.max(lo), 0, 0)
        })
        .max()
        .unwrap_or(1);
    let off = props.value.is_none();

    let on_number = {
        let (on_change, sent) = (props.on_change.clone(), sent.clone());
        Callback::from(move |n: i64| {
            let next = n.saturating_mul(factor).clamp(min, max);
            *sent.borrow_mut() = Some(next);
            on_change.emit(Some(next));
        })
    };

    let on_unit = {
        let (on_change, sent, unit) = (props.on_change.clone(), sent.clone(), unit.clone());
        let (units, value) = (units.clone(), props.value);
        Callback::from(move |pick: Option<usize>| {
            let next = pick.map(|i| {
                unit.set(i);
                let factor = units[i].factor;
                match value {
                    Some(v) => convert(v, factor, min, max),
                    None => factor.clamp(min, max),
                }
            });
            *sent.borrow_mut() = next;

            if next != value {
                on_change.emit(next);
            }
        })
    };

    let mut options: Vec<_> = units
        .iter()
        .enumerate()
        .map(|(i, unit)| SelectOption::new(Some(i), unit.label.clone()))
        .collect();

    if let Some(off_label) = &props.off_label {
        options.push(SelectOption::new(None, off_label.clone()));
    }

    let picked = if off && props.off_label.is_some() {
        None
    } else {
        Some(shown)
    };
    let unit_testid = props
        .testid
        .as_ref()
        .map(|id| AttrValue::from(format!("{id}-unit")));

    html! {
        <div class={quantity()}>
            <Stepper value={props.value.map_or(lo, |v| v / factor)} on_change={on_number}
                min={lo} max={hi.max(lo)} label={props.label.clone()} id={props.id.clone()}
                size={props.size} invalid={props.invalid} disabled={props.disabled || off}
                mixed={off} placeholder={off.then(|| AttrValue::from("–"))}
                chars={Some(chars.min(usize::from(u8::MAX)) as u8)}
                testid={props.testid.clone()} />
            <Select<Option<usize>> {options} value={Some(picked)} on_change={on_unit}
                label={unit_label} size={props.size} invalid={props.invalid}
                disabled={props.disabled} testid={unit_testid} />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(props: QuantityFieldProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<QuantityField>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props(value: Option<i64>) -> QuantityFieldProps {
        QuantityFieldProps {
            value,
            on_change: Callback::noop(),
            units: durations(),
            min: 0,
            max: 4 * 604_800,
            label: "TTL".into(),
            unit_label: None,
            off_label: None,
            id: None,
            size: InputSize::Regular,
            invalid: false,
            disabled: false,
            testid: None,
        }
    }

    #[test]
    fn the_largest_dividing_unit_is_the_most_readable() {
        let units = durations();
        assert_eq!(readable_unit(90, &units), 0);
        assert_eq!(readable_unit(120, &units), 1);
        assert_eq!(readable_unit(7_200, &units), 2);
        assert_eq!(readable_unit(172_800, &units), 3);
        assert_eq!(readable_unit(1_209_600, &units), 4);
        assert_eq!(readable_unit(0, &units), 0);
        assert_eq!(readable_unit(-3_600, &units), 2);
        let coarse = [Unit::new("min", 60), Unit::new("h", 3_600)];
        assert_eq!(readable_unit(90, &coarse), 0);
    }

    #[test]
    fn bounds_in_a_unit_are_whole_numbers_inside_the_range() {
        assert_eq!(unit_bounds(0, 100, 60), (0, 1));
        assert_eq!(unit_bounds(30, 200, 60), (1, 3));
        assert_eq!(unit_bounds(30, 50, 60), (1, 0));
    }

    #[test]
    fn a_unit_change_keeps_the_value_that_divides() {
        assert_eq!(convert(7_200, 3_600, 0, 1_000_000), 7_200);
        assert_eq!(convert(7_200, 60, 0, 1_000_000), 7_200);
    }

    #[test]
    fn a_unit_change_rounds_and_clamps_what_does_not_divide() {
        assert_eq!(convert(5_400, 3_600, 0, 1_000_000), 7_200);
        assert_eq!(convert(5_399, 3_600, 0, 1_000_000), 3_600);
        assert_eq!(convert(5_400, 3_600, 0, 4_000), 3_600);
        assert_eq!(convert(100, 3_600, 60, 1_000_000), 3_600);
        assert_eq!(convert(90, 3_600, 60, 100), 60);
        assert_eq!(convert(5_400, 3_600, 0, 3_700), 3_600);
    }

    #[test]
    fn the_ready_made_unit_sets_ascend_from_one() {
        for units in [durations(), byte_sizes()] {
            assert_eq!(units[0].factor, 1);
            assert!(units.windows(2).all(|w| w[0].factor < w[1].factor));
        }
    }

    #[test]
    fn the_field_is_the_quantity_class() {
        assert_eq!(
            quantity()
                .into_iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>(),
            [class::QUANTITY]
        );
    }

    #[test]
    fn it_shows_the_number_in_the_readable_unit() {
        let html = render(props(Some(7_200)));
        assert!(html.contains(class::QUANTITY), "{html}");
        assert!(html.contains("value=\"2\""), "{html}");
        assert!(html.contains(">h<"), "{html}");
    }

    #[test]
    fn off_disables_the_number_and_names_the_off_choice() {
        let html = render(QuantityFieldProps {
            off_label: Some("Off".into()),
            ..props(None)
        });
        assert!(html.contains("placeholder=\"–\""), "{html}");
        assert!(html.contains(">Off<"), "{html}");
        assert!(html.contains("disabled=\"disabled\""), "{html}");
    }

    #[test]
    fn the_number_has_the_room_of_its_widest_unit() {
        let html = render(props(Some(7_200)));
        assert!(html.contains("--stepper-chars: 7"), "{html}");
    }
}
