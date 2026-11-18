use yew::prelude::*;

use sockel::{
    InputSize, SecretInput, Stepper, format_number, key_step, parse_decimal, step_by, stepper,
};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const SIZES: &[(&str, InputSize)] = &[
    ("Regular", InputSize::Regular),
    ("Small", InputSize::Small),
    ("Touch", InputSize::Touch),
];

const MAXES: &[(&str, i64)] = &[("100", 100), ("9", 9), ("99999", 99999)];
const STEPS: &[(&str, i64)] = &[("1", 1), ("5", 5), ("25", 25)];

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn StepperPage() -> Html {
    let size = use_state(|| InputSize::Regular);
    let max = use_state(|| 100i64);
    let step = use_state(|| 1i64);
    let invalid = use_state(|| false);
    let disabled = use_state(|| false);
    let busy = use_state(|| false);
    let live = use_state(|| false);
    let value = use_state(|| 7i64);

    let controls = html! {
        <>
            <Select<InputSize> label="Size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Select<i64> label="Max" options={MAXES} value={*max} on_change={bind(&max)} />
            <Select<i64> label="Step" options={STEPS} value={*step} on_change={bind(&step)} />
            <Toggle label="invalid" value={*invalid} on_change={bind(&invalid)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="live" value={*live} on_change={bind(&live)} />
        </>
    };
    let code = snippet(
        &[],
        El::new("Stepper")
            .expr("value", "value")
            .expr("on_change", "on_change.clone()")
            .expr("min", "0")
            .expr("max", max.to_string())
            .expr_if(*step != 1, "step", step.to_string())
            .text("label", "Copies")
            .variant("size", *size)
            .flag("invalid", *invalid)
            .flag("disabled", *disabled)
            .flag("busy", *busy)
            .flag("live", *live),
    );
    let clamped = (*value).clamp(0, *max);

    html! {
        <Demo {controls} {code}>
            <Stepper value={clamped} on_change={bind(&value)} min={0} max={*max} step={*step}
                label="Copies" size={*size} invalid={*invalid} disabled={*disabled} busy={*busy} live={*live} />
            { shown("stepper(size, invalid, disabled, busy)", &stepper(*size, *invalid, *disabled, *busy)) }
            <code class={class::CLASS_CALL}>
                { format!("format_number(1250, 2, 3) → \"{}\"; parse_decimal(\"12,5\", 2) → {:?}",
                    format_number(1250, 2, 3), parse_decimal("12,5", 2)) }
            </code>
            <code class={class::CLASS_CALL}>
                { format!("step_by(value, step, step, 0, max) → {}; key_step(\"PageUp\", step, step, None) → {:?}",
                    step_by(clamped, *step, *step, 0, *max), key_step("PageUp", *step, *step, None)) }
            </code>
            <p class={class::CONTROL_LABEL}>{ format!("Value {clamped}") }</p>
        </Demo>
    }
}

#[function_component]
pub fn SecretInputPage() -> Html {
    let size = use_state(|| InputSize::Regular);
    let text = use_state(|| Text::Short);
    let value = use_state(|| Text::Short.of("sk-live-4f9a"));
    let invalid = use_state(|| false);
    let disabled = use_state(|| false);
    let busy = use_state(|| false);
    let copied = use_state(|| 0usize);
    let commit = use_state(|| false);
    let titles = use_state(|| false);
    let edits = use_state(|| 0usize);

    let pick_text = {
        let value = value.clone();
        let text = text.clone();
        Callback::from(move |t: Text| {
            value.set(t.of("sk-live-4f9a"));
            text.set(t);
        })
    };
    let controls = html! {
        <>
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={pick_text} />
            <Select<InputSize> label="Size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Toggle label="invalid" value={*invalid} on_change={bind(&invalid)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="commit" value={*commit} on_change={bind(&commit)} />
            <Toggle label="titles" value={*titles} on_change={bind(&titles)} />
        </>
    };
    let code = snippet(
        &[],
        El::new("SecretInput")
            .expr("value", "value.clone()")
            .expr("on_input", "on_input.clone()")
            .text("label", "API key")
            .variant("size", *size)
            .flag("invalid", *invalid)
            .flag("disabled", *disabled)
            .flag("busy", *busy)
            .flag("commit", *commit)
            .flag("titles", *titles),
    );
    let on_input = {
        let value = value.clone();
        let edits = edits.clone();
        Callback::from(move |text: String| {
            value.set(text);
            edits.set(*edits + 1);
        })
    };
    let on_copy = {
        let copied = copied.clone();
        Callback::from(move |_: String| copied.set(*copied + 1))
    };

    html! {
        <Demo {controls} {code}>
            <SecretInput value={AttrValue::from((*value).clone())} {on_input}
                {on_copy} label="API key" placeholder="Paste a key" size={*size}
                invalid={*invalid} disabled={*disabled} busy={*busy} commit={*commit} titles={*titles} />
            <p class={class::CONTROL_LABEL}>{ format!("Copied {} times, {} edits", *copied, *edits) }</p>
        </Demo>
    }
}
