use yew::prelude::*;

use sockel::{InputSize, QuantityField, Unit, byte_sizes, durations, quantity, readable_unit};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Toggle, bind};

const SIZES: &[(&str, InputSize)] = &[
    ("Regular", InputSize::Regular),
    ("Small", InputSize::Small),
    ("Touch", InputSize::Touch),
];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Units {
    Durations,
    Bytes,
}

const UNITS: &[(&str, Units)] = &[
    ("Durations", Units::Durations),
    ("Byte sizes", Units::Bytes),
];

#[function_component]
pub fn QuantityPage() -> Html {
    let size = use_state(|| InputSize::Regular);
    let units = use_state(|| Units::Durations);
    let off = use_state(|| true);
    let invalid = use_state(|| false);
    let disabled = use_state(|| false);
    let value = use_state(|| Some(7_200i64));

    let set: Vec<Unit> = match *units {
        Units::Durations => durations(),
        Units::Bytes => byte_sizes(),
    };
    let (builder, max) = match *units {
        Units::Durations => ("durations()", 4 * 604_800),
        Units::Bytes => ("byte_sizes()", 4 << 30),
    };
    let commits = use_state(|| 0usize);
    let on_commit = {
        let commits = commits.clone();
        Callback::from(move |_: Option<i64>| commits.set(*commits + 1))
    };
    let current = (*value).map(|v| v.clamp(0, max));

    let controls = html! {
        <>
            <Select<InputSize> label="Size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Select<Units> label="Units" options={UNITS} value={*units} on_change={bind(&units)} />
            <Toggle label="off" value={*off} on_change={bind(&off)} />
            <Toggle label="invalid" value={*invalid} on_change={bind(&invalid)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };
    let code = snippet(
        &[],
        El::new("QuantityField")
            .expr("value", "value")
            .expr("on_change", "on_change.clone()")
            .expr("on_commit", "Some(on_commit.clone())")
            .expr("units", builder)
            .expr("min", "0")
            .expr("max", max.to_string())
            .text("label", "Keep for")
            .expr_if(*off, "off_label", "Some(\"Off\".into())")
            .variant("size", *size)
            .flag("invalid", *invalid)
            .flag("disabled", *disabled),
    );

    html! {
        <Demo {controls} {code}>
            <QuantityField value={current} on_change={bind(&value)} on_commit={Some(on_commit)} units={set.clone()} min={0} {max}
                label="Keep for" off_label={off.then(|| AttrValue::from("Off"))} size={*size}
                invalid={*invalid} disabled={*disabled} testid="quantity" />
            <code class={class::CLASS_CALL}>
                { format!("quantity() → \"{}\"; readable_unit({}, units) → {:?}",
                    quantity().to_string(), current.unwrap_or(0),
                    set.get(readable_unit(current.unwrap_or(0), &set)).map(|u| u.label.to_string())) }
            </code>
            <p class={class::CONTROL_LABEL}>{ format!("Value {current:?}, committed {} times", *commits) }</p>
        </Demo>
    }
}
