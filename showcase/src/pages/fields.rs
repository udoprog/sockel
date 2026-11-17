use std::sync::atomic::{AtomicUsize, Ordering};

use yew::prelude::*;

use plinth::{
    Field, FieldCheck, Fieldset, FormRow, FormRows, field, field_describedby, field_error,
    form_row, form_rows,
};

use crate::class;
use crate::controls::{Demo, Select, Text, Toggle, bind};

/// A control id of its own for each frame's copy of the field.
#[hook]
fn use_control_id() -> AttrValue {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let id = use_state(|| format!("showcase-field-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
    AttrValue::from((*id).clone())
}

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    label: Text,
    hint: Text,
    error: Text,
    required: bool,
    reserve_error: bool,
    disabled: bool,
    busy: bool,
    help: bool,
}

#[derive(Properties, PartialEq)]
struct CaseProps {
    settings: Settings,
    row: bool,
}

#[function_component]
fn Case(props: &CaseProps) -> Html {
    let id = use_control_id();
    let s = props.settings;
    let label = s.label.of("Display name");
    let hint = s.hint.of("Shown on your profile");
    let error = s.error.of("Use at least three letters");
    let (hint, error) = (
        (!hint.is_empty()).then_some(hint),
        (!error.is_empty()).then_some(error),
    );
    let describedby = field_describedby(&id, hint.is_some(), error.is_some());
    let control = html! {
        <input id={id.clone()} class={plinth::class::INPUT} value="Ada"
            required={s.required} disabled={s.disabled}
            aria-invalid={error.is_some().then_some("true")}
            aria-describedby={describedby} />
    };
    let help = s.help.then(|| html! { <a href="#field">{ "Why?" }</a> });
    let label = html! { { label } };
    let hint = hint.map(|h| html! { { h } });
    let error = error.map(|e| html! { { e } });

    if props.row {
        html! {
            <FormRow id={id.clone()} {label} {hint} {error} {help} required={s.required}
                reserve_error={s.reserve_error} disabled={s.disabled} busy={s.busy}>
                { control }
            </FormRow>
        }
    } else {
        html! {
            <Field id={id.clone()} {label} {hint} {error} {help} required={s.required}
                reserve_error={s.reserve_error} disabled={s.disabled} busy={s.busy}>
                { control }
            </Field>
        }
    }
}

const HINT_OPTIONS: &[(&str, Text)] = &[
    ("none", Text::Empty),
    ("short", Text::Short),
    ("long", Text::Long),
    ("long, unbroken", Text::Unbroken),
];

fn settings_controls(s: &UseStateHandle<Settings>, extra: Html) -> Html {
    let set = |f: fn(&mut Settings, bool)| {
        let s = s.clone();
        Callback::from(move |on| {
            let mut next = *s;
            f(&mut next, on);
            s.set(next);
        })
    };
    let pick = |f: fn(&mut Settings, Text)| {
        let s = s.clone();
        Callback::from(move |text| {
            let mut next = *s;
            f(&mut next, text);
            s.set(next);
        })
    };

    html! {
        <>
            <Select<Text> label="label" options={Text::OPTIONS} value={s.label}
                on_change={pick(|s, t| s.label = t)} />
            <Select<Text> label="hint" options={HINT_OPTIONS} value={s.hint}
                on_change={pick(|s, t| s.hint = t)} />
            <Select<Text> label="error" options={HINT_OPTIONS} value={s.error}
                on_change={pick(|s, t| s.error = t)} />
            <Toggle label="required" value={s.required} on_change={set(|s, on| s.required = on)} />
            <Toggle label="reserve_error" value={s.reserve_error}
                on_change={set(|s, on| s.reserve_error = on)} />
            <Toggle label="disabled" value={s.disabled} on_change={set(|s, on| s.disabled = on)} />
            <Toggle label="busy" value={s.busy} on_change={set(|s, on| s.busy = on)} />
            <Toggle label="help" value={s.help} on_change={set(|s, on| s.help = on)} />
            { extra }
        </>
    }
}

fn initial() -> Settings {
    Settings {
        label: Text::Short,
        hint: Text::Short,
        error: Text::Empty,
        required: false,
        reserve_error: true,
        disabled: false,
        busy: false,
        help: false,
    }
}

#[function_component]
pub fn FieldPage() -> Html {
    let settings = use_state(initial);
    let controls = settings_controls(&settings, html! {});
    let classes = field(settings.disabled, settings.busy);
    let error = field_error(settings.reserve_error);

    html! {
        <Demo {controls}>
            <code class={class::CLASS_CALL}>
                { format!("field(disabled, busy) → \"{}\"; field_error(reserve) → \"{}\"", classes.to_string(), error.to_string()) }
            </code>
            <Case settings={*settings} row={false} />
            <Case settings={*settings} row={false} />
        </Demo>
    }
}

#[function_component]
pub fn FormRowsPage() -> Html {
    let settings = use_state(initial);
    let wide = use_state(|| true);
    let controls = settings_controls(
        &settings,
        html! { <Toggle label="wide_labels" value={*wide} on_change={bind(&wide)} /> },
    );
    let classes = form_rows(*wide);
    let row = form_row(settings.disabled, settings.busy);

    html! {
        <Demo {controls}>
            <code class={class::CLASS_CALL}>
                { format!("form_rows(wide_labels) → \"{}\"; form_row(disabled, busy) → \"{}\"", classes.to_string(), row.to_string()) }
            </code>
            <FormRows wide_labels={*wide}>
                <Case settings={*settings} row={true} />
                <Case settings={*settings} row={true} />
            </FormRows>
            <Fieldset legend={html! { "Notifications" }}
                hint={html! { "Sent to the address on your account" }}>
                <FieldCheck label={html! { "Email me when a card lands" }} disabled={settings.disabled}>
                    <input type="checkbox" disabled={settings.disabled} />
                </FieldCheck>
            </Fieldset>
        </Demo>
    }
}
