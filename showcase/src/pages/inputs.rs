use yew::prelude::*;

use plinth::{
    Button, ButtonSize, Field, InputGroup, InputKind, InputSize, SearchField, TextInput, Textarea,
    field_describedby, group_classes,
};

use super::fields::use_control_id;
use crate::class;
use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const SIZES: &[(&str, InputSize)] = &[
    ("Regular", InputSize::Regular),
    ("Small", InputSize::Small),
    ("Touch", InputSize::Touch),
];

const KINDS: &[(&str, InputKind)] = &[
    ("Text", InputKind::Text),
    ("Email", InputKind::Email),
    ("Url", InputKind::Url),
    ("Tel", InputKind::Tel),
];

const BUTTONS: &[(&str, usize)] = &[("2", 2), ("0", 0), ("1", 1), ("3", 3)];

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[derive(Clone, Copy, PartialEq)]
struct TextSettings {
    kind: InputKind,
    size: InputSize,
    unit: Text,
    placeholder: bool,
    invalid: bool,
    disabled: bool,
    readonly: bool,
    busy: bool,
}

#[derive(Properties, PartialEq)]
struct TextCaseProps {
    settings: TextSettings,
    value: AttrValue,
    on_input: Callback<String>,
}

#[function_component]
fn TextCase(props: &TextCaseProps) -> Html {
    let s = props.settings;
    let id = use_control_id();
    let unit_id = format!("{id}-unit");
    let area_id = format!("{id}-area");
    let error = s
        .invalid
        .then(|| html! { { "Use at least three letters" } });
    let describedby = field_describedby(&id, false, s.invalid);
    let placeholder = s.placeholder.then(|| AttrValue::from("Type here"));
    let unit = s.unit.of("ms");

    html! {
        <>
            <Field id={id.clone()} label={html! { "Name" }} error={error.clone()} reserve_error=true
                disabled={s.disabled} busy={s.busy}>
                <TextInput id={id.clone()} value={props.value.clone()} on_input={props.on_input.clone()}
                    kind={s.kind} size={s.size} placeholder={placeholder.clone()} invalid={s.invalid}
                    disabled={s.disabled} readonly={s.readonly} busy={s.busy}
                    describedby={describedby.clone()} />
            </Field>
            <Field id={unit_id.clone()} label={html! { "Delay" }} error={error.clone()}
                reserve_error=true disabled={s.disabled} busy={s.busy}>
                <TextInput id={unit_id} value={props.value.clone()} on_input={props.on_input.clone()}
                    size={s.size} unit={html! { { unit } }} inputmode="numeric" maxlength={Some(6)}
                    placeholder={placeholder.clone()}
                    invalid={s.invalid} disabled={s.disabled} readonly={s.readonly} busy={s.busy}
                    describedby={describedby.clone()} />
            </Field>
            <Field id={area_id.clone()} label={html! { "Notes" }} error={error} reserve_error=true
                disabled={s.disabled} busy={s.busy}>
                <Textarea id={area_id} value={props.value.clone()} on_input={props.on_input.clone()}
                    size={s.size} placeholder={placeholder} invalid={s.invalid}
                    disabled={s.disabled} readonly={s.readonly} busy={s.busy}
                    describedby={describedby} />
            </Field>
        </>
    }
}

#[function_component]
pub fn TextInputPage() -> Html {
    let settings = use_state(|| TextSettings {
        kind: InputKind::Text,
        size: InputSize::Regular,
        unit: Text::Short,
        placeholder: true,
        invalid: false,
        disabled: false,
        readonly: false,
        busy: false,
    });
    let value = use_state(|| String::from("Ada Lovelace"));
    let text = use_state(|| Text::Short);

    let pick_text = {
        let value = value.clone();
        let text = text.clone();
        Callback::from(move |t: Text| {
            value.set(t.of("Ada Lovelace"));
            text.set(t);
        })
    };
    let set = |f: fn(&mut TextSettings, bool)| {
        let settings = settings.clone();
        Callback::from(move |on| {
            let mut next = *settings;
            f(&mut next, on);
            settings.set(next);
        })
    };
    let s = *settings;
    let controls = html! {
        <>
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={pick_text} />
            <Select<InputSize> label="Size" options={SIZES} value={s.size} on_change={{
                let settings = settings.clone();
                Callback::from(move |size| settings.set(TextSettings { size, ..*settings }))
            }} />
            <Select<InputKind> label="Kind" options={KINDS} value={s.kind} on_change={{
                let settings = settings.clone();
                Callback::from(move |kind| settings.set(TextSettings { kind, ..*settings }))
            }} />
            <Select<Text> label="Unit" options={Text::OPTIONS} value={s.unit} on_change={{
                let settings = settings.clone();
                Callback::from(move |unit| settings.set(TextSettings { unit, ..*settings }))
            }} />
            <Toggle label="placeholder" value={s.placeholder} on_change={set(|s, on| s.placeholder = on)} />
            <Toggle label="invalid" value={s.invalid} on_change={set(|s, on| s.invalid = on)} />
            <Toggle label="disabled" value={s.disabled} on_change={set(|s, on| s.disabled = on)} />
            <Toggle label="readonly" value={s.readonly} on_change={set(|s, on| s.readonly = on)} />
            <Toggle label="busy" value={s.busy} on_change={set(|s, on| s.busy = on)} />
        </>
    };
    let on_input = bind(&value);
    let placeholder = |el: El| el.text_if(s.placeholder, "placeholder", "Type here");
    let common = |el: El| {
        placeholder(el)
            .variant("size", s.size)
            .flag("invalid", s.invalid)
            .flag("disabled", s.disabled)
            .flag("readonly", s.readonly)
            .flag("busy", s.busy)
    };
    let code = snippet(
        &[],
        El::new("")
            .child(common(
                El::new("TextInput")
                    .expr("id", "id.clone()")
                    .expr("value", "value.clone()")
                    .expr("on_input", "on_input.clone()")
                    .variant("kind", s.kind),
            ))
            .child(common(
                El::new("TextInput")
                    .expr("value", "value.clone()")
                    .expr("on_input", "on_input.clone()")
                    .expr("unit", format!("html! {{ {:?} }}", s.unit.of("ms")))
                    .text("inputmode", "numeric")
                    .expr("maxlength", "Some(6)"),
            ))
            .child(common(
                El::new("Textarea")
                    .expr("value", "value.clone()")
                    .expr("on_input", "on_input.clone()"),
            )),
    );

    html! {
        <Demo {controls} {code}>
            <TextCase settings={s} value={AttrValue::from((*value).clone())} {on_input} />
            { shown("input(size, invalid, busy)", &plinth::input(s.size, s.invalid, s.busy)) }
            { shown("input_box(size, invalid, disabled, busy)", &plinth::input_box(s.size, s.invalid, s.disabled, s.busy)) }
            { shown("textarea(size, invalid, busy)", &plinth::textarea(s.size, s.invalid, s.busy)) }
        </Demo>
    }
}

#[derive(Clone, Copy, PartialEq)]
struct SearchSettings {
    size: InputSize,
    label: Text,
    invalid: bool,
    disabled: bool,
    busy: bool,
}

#[derive(Properties, PartialEq)]
struct SearchCaseProps {
    settings: SearchSettings,
    value: AttrValue,
    on_input: Callback<String>,
}

#[function_component]
fn SearchCase(props: &SearchCaseProps) -> Html {
    let s = props.settings;
    html! {
        <SearchField value={props.value.clone()} on_input={props.on_input.clone()}
            label={s.label.of("Search")} size={s.size} invalid={s.invalid}
            disabled={s.disabled} busy={s.busy} />
    }
}

#[function_component]
pub fn SearchFieldPage() -> Html {
    let settings = use_state(|| SearchSettings {
        size: InputSize::Regular,
        label: Text::Short,
        invalid: false,
        disabled: false,
        busy: false,
    });
    let value = use_state(String::new);
    let text = use_state(|| Text::Empty);
    let pick_text = {
        let value = value.clone();
        let text = text.clone();
        Callback::from(move |t: Text| {
            value.set(t.of("query"));
            text.set(t);
        })
    };
    let set = |f: fn(&mut SearchSettings, bool)| {
        let settings = settings.clone();
        Callback::from(move |on| {
            let mut next = *settings;
            f(&mut next, on);
            settings.set(next);
        })
    };
    let s = *settings;
    let controls = html! {
        <>
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={pick_text} />
            <Select<InputSize> label="Size" options={SIZES} value={s.size} on_change={{
                let settings = settings.clone();
                Callback::from(move |size| settings.set(SearchSettings { size, ..*settings }))
            }} />
            <Select<Text> label="Label" options={Text::OPTIONS} value={s.label} on_change={{
                let settings = settings.clone();
                Callback::from(move |label| settings.set(SearchSettings { label, ..*settings }))
            }} />
            <Toggle label="invalid" value={s.invalid} on_change={set(|s, on| s.invalid = on)} />
            <Toggle label="disabled" value={s.disabled} on_change={set(|s, on| s.disabled = on)} />
            <Toggle label="busy" value={s.busy} on_change={set(|s, on| s.busy = on)} />
        </>
    };
    let on_input = bind(&value);
    let code = snippet(
        &[],
        El::new("SearchField")
            .expr("value", "value.clone()")
            .expr("on_input", "on_input.clone()")
            .text("label", &s.label.of("Search"))
            .variant("size", s.size)
            .flag("invalid", s.invalid)
            .flag("disabled", s.disabled)
            .flag("busy", s.busy),
    );

    html! {
        <Demo {controls} {code}>
            <SearchCase settings={s} value={AttrValue::from((*value).clone())} {on_input} />
            { shown("input_box(size, invalid, disabled, busy)", &plinth::input_box(s.size, s.invalid, s.disabled, s.busy)) }
            <p class={class::CONTROL_LABEL}>{ format!("Searching for {:?}", *value) }</p>
        </Demo>
    }
}

#[derive(Properties, PartialEq)]
struct GroupCaseProps {
    size: InputSize,
    buttons: usize,
    detached: bool,
    invalid: bool,
    disabled: bool,
    label: Text,
    value: AttrValue,
    on_input: Callback<String>,
}

#[function_component]
fn GroupCase(props: &GroupCaseProps) -> Html {
    let button_size = match props.size {
        InputSize::Regular => ButtonSize::Regular,
        InputSize::Small => ButtonSize::Small,
        InputSize::Touch => ButtonSize::Touch,
    };
    let members = 1 + props.buttons;
    let detached: Vec<bool> = (0..members)
        .map(|i| props.detached && i + 1 == members && i > 0)
        .collect();
    let mut classes = group_classes(&detached).into_iter();

    html! {
        <InputGroup>
            <TextInput value={props.value.clone()} on_input={props.on_input.clone()}
                size={props.size} invalid={props.invalid} disabled={props.disabled}
                label="Address" placeholder="Address" class={classes.next().unwrap_or_default()} />
            { for (1..=props.buttons).map(|n| html! {
                <Button size={button_size} disabled={props.disabled}
                    class={classes.next().unwrap_or_default()}>
                    { props.label.of(&format!("Go {n}")) }
                </Button>
            }) }
        </InputGroup>
    }
}

#[function_component]
pub fn InputGroupPage() -> Html {
    let size = use_state(|| InputSize::Regular);
    let buttons = use_state(|| 2usize);
    let detached = use_state(|| false);
    let invalid = use_state(|| false);
    let disabled = use_state(|| false);
    let label = use_state(|| Text::Short);
    let value = use_state(|| String::from("ada@example.com"));

    let controls = html! {
        <>
            <Select<InputSize> label="Size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Select<usize> label="Buttons" options={BUTTONS} value={*buttons} on_change={bind(&buttons)} />
            <Select<Text> label="Labels" options={Text::OPTIONS} value={*label} on_change={bind(&label)} />
            <Toggle label="detached" value={*detached} on_change={bind(&detached)} />
            <Toggle label="invalid" value={*invalid} on_change={bind(&invalid)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };
    let members = 1 + *buttons;
    let flags: Vec<bool> = (0..members)
        .map(|i| *detached && i + 1 == members && i > 0)
        .collect();
    let calls: Vec<String> = group_classes(&flags)
        .iter()
        .map(|c| format!("\"{}\"", c.to_string()))
        .collect();

    let mut member_classes = group_classes(&flags).into_iter().map(|c| c.to_string());
    let mut member = |el: El| el.text_if(true, "class", &member_classes.next().unwrap_or_default());
    let button_size = match *size {
        InputSize::Regular => ButtonSize::Regular,
        InputSize::Small => ButtonSize::Small,
        InputSize::Touch => ButtonSize::Touch,
    };
    let mut group = El::new("InputGroup").child(member(
        El::new("TextInput")
            .expr("value", "value.clone()")
            .expr("on_input", "on_input.clone()")
            .variant("size", *size)
            .flag("invalid", *invalid)
            .flag("disabled", *disabled),
    ));
    for n in 1..=*buttons {
        group = group.child(member(
            El::new("Button")
                .variant("size", button_size)
                .flag("disabled", *disabled)
                .line(text_node(&label.of(&format!("Go {n}")))),
        ));
    }
    let code = snippet(&[], group);

    html! {
        <Demo {controls} {code}>
            <GroupCase size={*size} buttons={*buttons} detached={*detached} invalid={*invalid}
                disabled={*disabled} label={*label} value={AttrValue::from((*value).clone())}
                on_input={bind(&value)} />
            <code class={class::CLASS_CALL}>{ format!("group_classes(detached) → [{}]", calls.join(", ")) }</code>
        </Demo>
    }
}
