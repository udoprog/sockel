use yew::prelude::*;

use plinth::{
    Button, ColorOption, ColorSelect, ColorSelectWidth, Field, InputGroup, InputSize, Stack,
    field_describedby, group_classes,
};

use super::fields::use_control_id;
use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select as Control, Text, Toggle};

const WIDTHS: &[(&str, ColorSelectWidth)] = &[
    ("Fixed", ColorSelectWidth::Fixed),
    ("Fill", ColorSelectWidth::Fill),
    ("Swatch", ColorSelectWidth::Swatch),
];

const SIZES: &[(&str, InputSize)] = &[
    ("Regular", InputSize::Regular),
    ("Small", InputSize::Small),
    ("Touch", InputSize::Touch),
];

const TEXTS: &[(&str, Text)] = &[
    ("Short", Text::Short),
    ("Long", Text::Long),
    ("Long, unbroken", Text::Unbroken),
];

const COLUMNS: &[(&str, usize)] = &[("6", 6), ("4", 4), ("3", 3)];

const VALUES: &[(&str, Option<&str>)] = &[
    ("none (the placeholder)", None),
    ("Red", Some("#d92d20")),
    ("the last", Some("#175cd3")),
    ("no colour", Some("")),
    ("a custom colour", Some("#8e44ad")),
];

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    text: Text,
    width: ColorSelectWidth,
    size: InputSize,
    columns: usize,
    placeholder: bool,
    custom: bool,
    palette: bool,
    invalid: bool,
    disabled: bool,
    busy: bool,
}

#[derive(Properties, PartialEq)]
struct SampleProps {
    settings: Settings,
    value: Option<AttrValue>,
    on_change: Callback<AttrValue>,
}

/// A palette as an app passes it: data, the last label of the chosen length.
fn palette(text: Text, full: bool) -> Vec<ColorOption> {
    if !full {
        return Vec::new();
    }
    vec![
        ColorOption::new("", "None"),
        ColorOption::new("#d92d20", "Red"),
        ColorOption::new("#dc8a00", "Orange"),
        ColorOption::new("#e7c000", "Yellow"),
        ColorOption::new("#12a150", "Green"),
        ColorOption::new("#0e9384", "Teal"),
        ColorOption::new("#7a5af8", "Violet"),
        ColorOption::new("#d444a0", "Pink"),
        ColorOption::new("#667085", "Grey"),
        ColorOption::new("#175cd3", text.of("Blue")),
    ]
}

/// A ColorSelect in a Field and one joined to a button, each frame with ids
/// of its own.
#[function_component]
fn Sample(props: &SampleProps) -> Html {
    let s = props.settings;
    let id = use_control_id();
    let options = palette(s.text, s.palette);
    let value = props.value.clone();
    let placeholder = s.placeholder.then_some(AttrValue::Static("Pick a colour"));
    let custom = s.custom.then_some(AttrValue::Static("Custom"));
    let error = s
        .invalid
        .then(|| html! { "Pick a colour from the palette." });
    let joined = group_classes(&[false, false]);

    html! {
        <Stack>
            <Field id={id.clone()} label={html! { "Colour" }} error={error} reserve_error=true
                disabled={s.disabled} busy={s.busy}>
                <ColorSelect id={id.clone()} options={options.clone()} value={value.clone()}
                    on_change={props.on_change.clone()} placeholder={placeholder.clone()}
                    custom={custom.clone()} columns={s.columns} width={s.width} size={s.size}
                    invalid={s.invalid} disabled={s.disabled} busy={s.busy}
                    describedby={field_describedby(&id, false, s.invalid)} />
            </Field>
            <InputGroup>
                <ColorSelect class={joined[0].clone()} label="Colour" {options} {value}
                    on_change={props.on_change.clone()} {placeholder} {custom} columns={s.columns}
                    width={s.width} size={s.size} invalid={s.invalid} disabled={s.disabled}
                    busy={s.busy} />
                <Button class={joined[1].clone()}>{ "Apply" }</Button>
            </InputGroup>
            <p class={class::SAMPLE}>{ "Pick any colour: the face keeps its width. The arrows move over the grid; Enter or Space picks; Escape closes. With a custom row, Tab moves to the colour input." }</p>
        </Stack>
    }
}

#[function_component]
pub fn ColorSelectPage() -> Html {
    let settings = use_state(|| Settings {
        text: Text::Short,
        width: ColorSelectWidth::Fixed,
        size: InputSize::Regular,
        columns: 6,
        placeholder: true,
        custom: true,
        palette: true,
        invalid: false,
        disabled: false,
        busy: false,
    });
    let value = use_state(|| Some(AttrValue::Static("#d92d20")));
    let s = *settings;
    let set = |f: fn(&mut Settings, bool)| {
        let settings = settings.clone();
        Callback::from(move |on| {
            let mut next = *settings;
            f(&mut next, on);
            settings.set(next);
        })
    };
    let pick = {
        let value = value.clone();
        Callback::from(move |v: AttrValue| value.set(Some(v)))
    };
    // The value control names a colour picked from the popup by its preset,
    // else as a custom one.
    let preset = (*value).as_ref().map(|v| {
        VALUES
            .iter()
            .find_map(|(_, known)| known.filter(|k| *k == v.as_str()))
            .unwrap_or("#8e44ad")
    });
    let set_value = {
        let value = value.clone();
        Callback::from(move |v: Option<&'static str>| value.set(v.map(AttrValue::Static)))
    };

    let controls = html! {
        <>
            <Control<Option<&'static str>> label="value" options={VALUES} value={preset} on_change={set_value} />
            <Control<Text> label="Label text" options={TEXTS} value={s.text} on_change={{
                let settings = settings.clone();
                Callback::from(move |text| settings.set(Settings { text, ..*settings }))
            }} />
            <Control<ColorSelectWidth> label="width" options={WIDTHS} value={s.width} on_change={{
                let settings = settings.clone();
                Callback::from(move |width| settings.set(Settings { width, ..*settings }))
            }} />
            <Control<InputSize> label="size" options={SIZES} value={s.size} on_change={{
                let settings = settings.clone();
                Callback::from(move |size| settings.set(Settings { size, ..*settings }))
            }} />
            <Control<usize> label="columns" options={COLUMNS} value={s.columns} on_change={{
                let settings = settings.clone();
                Callback::from(move |columns| settings.set(Settings { columns, ..*settings }))
            }} />
            <Toggle label="placeholder" value={s.placeholder} on_change={set(|s, on| s.placeholder = on)} />
            <Toggle label="custom" value={s.custom} on_change={set(|s, on| s.custom = on)} />
            <Toggle label="palette" value={s.palette} on_change={set(|s, on| s.palette = on)} />
            <Toggle label="invalid" value={s.invalid} on_change={set(|s, on| s.invalid = on)} />
            <Toggle label="disabled" value={s.disabled} on_change={set(|s, on| s.disabled = on)} />
            <Toggle label="busy" value={s.busy} on_change={set(|s, on| s.busy = on)} />
        </>
    };

    let code = snippet(
        &[
            "let palette = vec![".to_owned(),
            "    ColorOption::new(\"#d92d20\", \"Red\"),".to_owned(),
            "    ColorOption::new(\"#12a150\", \"Green\"),".to_owned(),
            "    ColorOption::new(\"#175cd3\", \"Blue\"),".to_owned(),
            "];".to_owned(),
        ],
        El::new("ColorSelect")
            .expr("id", "id")
            .expr("options", "palette")
            .expr("value", "colour")
            .expr("on_change", "set_colour")
            .text_if(s.placeholder, "placeholder", "Pick a colour")
            .text_if(s.custom, "custom", "Custom")
            .expr_if(s.columns != 6, "columns", s.columns.to_string())
            .variant("width", s.width)
            .variant("size", s.size)
            .flag("invalid", s.invalid)
            .flag("disabled", s.disabled)
            .flag("busy", s.busy),
    );

    html! {
        <Demo {controls} {code}>
            <Sample settings={s} value={(*value).clone()} on_change={pick} />
        </Demo>
    }
}
