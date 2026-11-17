use yew::prelude::*;

use plinth::{
    Button, Field, Glyph, Icon, InputGroup, InputSize, Select, SelectOption, SelectWidth, Stack,
    field_describedby, group_classes,
};

use super::fields::use_control_id;
use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select as Control, Text, Toggle, bind};

const WIDTHS: &[(&str, SelectWidth)] =
    &[("Widest", SelectWidth::Widest), ("Fill", SelectWidth::Fill)];

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

const VALUES: &[(&str, Option<usize>)] = &[
    ("none (the placeholder)", None),
    ("Small", Some(0)),
    ("Medium", Some(1)),
    ("the last", Some(3)),
];

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    text: Text,
    width: SelectWidth,
    size: InputSize,
    placeholder: bool,
    invalid: bool,
    disabled: bool,
    busy: bool,
    option_disabled: bool,
}

#[derive(Properties, PartialEq)]
struct SampleProps {
    settings: Settings,
    value: Option<usize>,
    on_change: Callback<usize>,
}

/// A Select in a Field and one joined to a button, each frame with ids of
/// its own.
#[function_component]
fn Sample(props: &SampleProps) -> Html {
    let s = props.settings;
    let id = use_control_id();
    let options = vec![
        SelectOption::new(0, "Small"),
        SelectOption::new(1, "Medium").icon(html! { <Icon glyph={Glyph::Plus} /> }),
        SelectOption::new(2, "Large").disabled(s.option_disabled),
        SelectOption::new(3, s.text.of("Extra large")),
    ];
    let placeholder = s.placeholder.then_some(AttrValue::Static("Pick a size"));
    let error = s.invalid.then(|| html! { "Pick a size that is in stock." });
    let joined = group_classes(&[false, false]);

    html! {
        <Stack>
            <Field id={id.clone()} label={html! { "Size" }} error={error} reserve_error=true
                disabled={s.disabled} busy={s.busy}>
                <Select<usize> id={id.clone()} options={options.clone()} value={props.value}
                    on_change={props.on_change.clone()} placeholder={placeholder.clone()}
                    width={s.width} size={s.size} invalid={s.invalid} disabled={s.disabled}
                    busy={s.busy} describedby={field_describedby(&id, false, s.invalid)} />
            </Field>
            <InputGroup>
                <Select<usize> class={joined[0].clone()} label="Size" {options} value={props.value}
                    on_change={props.on_change.clone()} {placeholder} width={s.width} size={s.size}
                    invalid={s.invalid} disabled={s.disabled} busy={s.busy} />
                <Button class={joined[1].clone()}>{ "Apply" }</Button>
            </InputGroup>
            <p class={class::SAMPLE}>{ "Pick any option or none: the face keeps its width. Arrows, Home, End, PageUp, PageDown and typing move; Enter or Space picks; Escape closes." }</p>
        </Stack>
    }
}

#[function_component]
pub fn SelectPage() -> Html {
    let settings = use_state(|| Settings {
        text: Text::Short,
        width: SelectWidth::Widest,
        size: InputSize::Regular,
        placeholder: true,
        invalid: false,
        disabled: false,
        busy: false,
        option_disabled: true,
    });
    let value = use_state(|| Some(1usize));
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
        Callback::from(move |v: usize| value.set(Some(v)))
    };

    let controls = html! {
        <>
            <Control<Option<usize>> label="value" options={VALUES} value={*value} on_change={bind(&value)} />
            <Control<Text> label="Option text" options={TEXTS} value={s.text} on_change={{
                let settings = settings.clone();
                Callback::from(move |text| settings.set(Settings { text, ..*settings }))
            }} />
            <Control<SelectWidth> label="width" options={WIDTHS} value={s.width} on_change={{
                let settings = settings.clone();
                Callback::from(move |width| settings.set(Settings { width, ..*settings }))
            }} />
            <Control<InputSize> label="size" options={SIZES} value={s.size} on_change={{
                let settings = settings.clone();
                Callback::from(move |size| settings.set(Settings { size, ..*settings }))
            }} />
            <Toggle label="placeholder" value={s.placeholder} on_change={set(|s, on| s.placeholder = on)} />
            <Toggle label="invalid" value={s.invalid} on_change={set(|s, on| s.invalid = on)} />
            <Toggle label="disabled" value={s.disabled} on_change={set(|s, on| s.disabled = on)} />
            <Toggle label="busy" value={s.busy} on_change={set(|s, on| s.busy = on)} />
            <Toggle label="an option disabled" value={s.option_disabled} on_change={set(|s, on| s.option_disabled = on)} />
        </>
    };

    let code = snippet(
        &[
            "let options = vec![".to_owned(),
            "    SelectOption::new(Size::Small, \"Small\"),".to_owned(),
            "    SelectOption::new(Size::Medium, \"Medium\"),".to_owned(),
            format!(
                "    SelectOption::new(Size::Large, \"Large\").disabled({}),",
                s.option_disabled
            ),
            "];".to_owned(),
        ],
        El::new("Select<Size>")
            .expr("id", "id")
            .expr("options", "options")
            .expr("value", "size")
            .expr("on_change", "set_size")
            .text_if(s.placeholder, "placeholder", "Pick a size")
            .variant("width", s.width)
            .variant("size", s.size)
            .flag("invalid", s.invalid)
            .flag("disabled", s.disabled)
            .flag("busy", s.busy),
    );

    html! {
        <Demo {controls} {code}>
            <Sample settings={s} value={*value} on_change={pick} />
        </Demo>
    }
}
