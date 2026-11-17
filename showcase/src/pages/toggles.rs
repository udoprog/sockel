use yew::prelude::*;

use crate::class;

use plinth::{
    Checkbox, ChipList, ChipToggle, Choice, ChoiceFit, ChoiceOption, Cluster, Glyph, Icon, Stack,
    Switch, checkbox, chip_count, choice, switch,
};

use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Off,
    On,
    Mixed,
}

const MARKS: &[(&str, Mark)] = &[
    ("Off", Mark::Off),
    ("On", Mark::On),
    ("Indeterminate", Mark::Mixed),
];

#[function_component]
pub fn CheckboxPage() -> Html {
    let mark = use_state(|| Mark::Off);
    let text = use_state(|| Text::Short);
    let disabled = use_state(|| false);
    let invalid = use_state(|| false);

    let controls = html! {
        <>
            <Select<Mark> label="State" options={MARKS} value={*mark} on_change={bind(&mark)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="invalid" value={*invalid} on_change={bind(&invalid)} />
        </>
    };
    let on_change = {
        let mark = mark.clone();
        Callback::from(move |on: bool| mark.set(if on { Mark::On } else { Mark::Off }))
    };

    let words = text.of("Subscribe to the newsletter");
    let code = snippet(
        &[],
        El::new("Checkbox")
            .flag("checked", *mark == Mark::On)
            .expr_if(*mark != Mark::On, "checked", "false")
            .flag("indeterminate", *mark == Mark::Mixed)
            .flag("disabled", *disabled)
            .flag("invalid", *invalid)
            .expr("on_change", "on_change")
            .text_if(words.is_empty(), "label", "Subscribe")
            .line_if(!words.is_empty(), text_node(&words)),
    );

    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("checkbox(disabled) → \"{}\"", checkbox(*disabled).to_string()) }
            </code>
            <Checkbox checked={*mark == Mark::On} indeterminate={*mark == Mark::Mixed}
                disabled={*disabled} invalid={*invalid} {on_change}
                label={text.of("").is_empty().then(|| AttrValue::from("Subscribe"))}>
                { text.of("Subscribe to the newsletter") }
            </Checkbox>
        </Demo>
    }
}

#[function_component]
pub fn SwitchPage() -> Html {
    let on = use_state(|| false);
    let text = use_state(|| Text::Short);
    let busy = use_state(|| false);
    let disabled = use_state(|| false);
    let invalid = use_state(|| false);

    let controls = html! {
        <>
            <Toggle label="on" value={*on} on_change={bind(&on)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="invalid" value={*invalid} on_change={bind(&invalid)} />
        </>
    };

    let words = text.of("Notifications");
    let code = snippet(
        &[],
        El::new("Switch")
            .flag("on", *on)
            .expr_if(!*on, "on", "false")
            .flag("busy", *busy)
            .flag("disabled", *disabled)
            .flag("invalid", *invalid)
            .expr("on_change", "on_change")
            .text_if(words.is_empty(), "label", "Notifications")
            .line_if(!words.is_empty(), text_node(&words)),
    );

    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("switch(on, invalid) → \"{}\"", switch(*on, *invalid).to_string()) }
            </code>
            <Switch on={*on} busy={*busy} disabled={*disabled} invalid={*invalid}
                on_change={bind(&on)}
                label={text.of("").is_empty().then(|| AttrValue::from("Notifications"))}>
                { text.of("Notifications") }
            </Switch>
        </Demo>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Content {
    Text,
    Long,
    Icons,
    IconOnly,
    Disabled,
}

const CONTENTS: &[(&str, Content)] = &[
    ("Text", Content::Text),
    ("Long text", Content::Long),
    ("Icons and text", Content::Icons),
    ("Icons only", Content::IconOnly),
    ("A disabled option", Content::Disabled),
];

const FITS: &[(&str, ChoiceFit)] = &[
    ("Equal", ChoiceFit::Equal),
    ("Stretch", ChoiceFit::Stretch),
    ("Fixed", ChoiceFit::Fixed),
];

const SELECTED: &[(&str, usize)] = &[("1st", 0), ("2nd", 1), ("3rd", 2)];

fn options(content: Content) -> Vec<ChoiceOption> {
    let glyphs = [Glyph::Plus, Glyph::Search, Glyph::Close];
    let labels = match content {
        Content::Long => [
            "Day",
            "A much longer week than the others",
            "Month-without-a-single-space-or-break",
        ],
        _ => ["Day", "Week", "Month"],
    };
    labels
        .iter()
        .zip(glyphs)
        .enumerate()
        .map(|(i, (label, glyph))| {
            let option = ChoiceOption::new(format!("option-{i}"), *label);
            let icon = html! { <Icon {glyph} /> };
            match content {
                Content::Icons => option.icon(icon),
                Content::IconOnly => option.icon_only(icon),
                Content::Disabled => option.disabled(i == 1),
                _ => option,
            }
        })
        .collect()
}

#[function_component]
pub fn ChoicePage() -> Html {
    let selected = use_state(|| 0usize);
    let content = use_state(|| Content::Text);
    let fit = use_state(|| ChoiceFit::Equal);
    let small = use_state(|| false);
    let invalid = use_state(|| false);
    let disabled = use_state(|| false);

    let controls = html! {
        <>
            <Select<usize> label="Selected" options={SELECTED} value={*selected} on_change={bind(&selected)} />
            <Select<Content> label="Content" options={CONTENTS} value={*content} on_change={bind(&content)} />
            <Select<ChoiceFit> label="fit" options={FITS} value={*fit} on_change={bind(&fit)} />
            <Toggle label="small" value={*small} on_change={bind(&small)} />
            <Toggle label="invalid" value={*invalid} on_change={bind(&invalid)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let options = options(*content);
    let on_change = {
        let selected = selected.clone();
        let options = options.clone();
        Callback::from(move |value: AttrValue| {
            if let Some(i) = options.iter().position(|o| o.value == value) {
                selected.set(i);
            }
        })
    };

    let mut lets = vec!["let options = vec![".to_owned()];
    lets.extend(
        options
            .iter()
            .zip(["Plus", "Search", "Close"])
            .enumerate()
            .map(|(i, (option, glyph))| {
                let icon = format!("html! {{ <Icon glyph={{Glyph::{glyph}}} /> }}");
                let then = match *content {
                    Content::Icons => format!(".icon({icon})"),
                    Content::IconOnly => format!(".icon_only({icon})"),
                    Content::Disabled if i == 1 => ".disabled(true)".to_owned(),
                    _ => String::new(),
                };
                format!(
                    "    ChoiceOption::new({:?}, {:?}){then},",
                    option.value.as_str(),
                    option.label.as_str()
                )
            }),
    );
    lets.push("];".to_owned());
    let code = snippet(
        &lets,
        El::new("Choice")
            .expr(
                "value",
                format!("Some({:?})", options[*selected].value.as_str()),
            )
            .expr("options", "options")
            .text("label", "Range")
            .variant("fit", *fit)
            .flag("small", *small)
            .flag("invalid", *invalid)
            .flag("disabled", *disabled)
            .expr("on_change", "on_change"),
    );

    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("choice(fit, small, invalid, disabled) → \"{}\"",
                    choice(*fit, *small, *invalid, *disabled).to_string()) }
            </code>
            <Choice value={options.get(*selected).map(|o| o.value.clone())} {options}
                label="Range" fit={*fit} small={*small} invalid={*invalid} disabled={*disabled}
                {on_change} />
            <span class={class::CONTROL_LABEL}>{ "Tab to it, then the arrow keys, Home and End." }</span>
        </Demo>
    }
}

const COUNTS_OF_CHIPS: &[(&str, usize)] =
    &[("1", 1), ("0 (empty)", 0), ("2", 2), ("8", 8), ("20", 20)];

const CHIP_COUNTS: &[(&str, Option<u32>)] = &[
    ("7", Some(7)),
    ("0", Some(0)),
    ("42", Some(42)),
    ("None", None),
];

#[function_component]
pub fn ChipPage() -> Html {
    let chips = use_state(|| 1);
    let text = use_state(|| Text::Short);
    let count = use_state(|| Some(7));
    let disabled = use_state(|| false);
    let on = use_state(|| vec![false; 20]);

    let controls = html! {
        <>
            <Select<usize> label="Chips" options={COUNTS_OF_CHIPS} value={*chips} on_change={bind(&chips)} />
            <Select<Text> label="Label" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<Option<u32>> label="Count" options={CHIP_COUNTS} value={*count} on_change={bind(&count)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let chip = |n: usize| {
        let on_toggle = {
            let on = on.clone();
            Callback::from(move |value: bool| {
                let mut next = (*on).clone();
                next[n] = value;
                on.set(next);
            })
        };
        html! {
            <ChipToggle on={on[n]} count={*count} disabled={*disabled} {on_toggle}>
                { text.of(&format!("Filter {}", n + 1)) }
            </ChipToggle>
        }
    };

    let chip_code = El::new("ChipToggle")
        .expr("on", "on")
        .expr_if(
            count.is_some(),
            "count",
            format!("Some({})", count.unwrap_or(0)),
        )
        .flag("disabled", *disabled)
        .expr("on_toggle", "on_toggle")
        .line(text_node(&text.of("Filter 1")));
    let list = El::new("ChipList").text("label", "Filters");
    let list = if *chips > 0 {
        list.child(chip_code).line_if(
            *chips > 1,
            format!("// and {} more ChipToggles", *chips - 1),
        )
    } else {
        list
    };
    let code = snippet(&[], list);

    html! {
        <Demo {controls} {code}>
            <Stack>
                <code class={class::CLASS_CALL}>
                    { format!("chip_count(count) → \"{}\"", chip_count(count.unwrap_or(0)).to_string()) }
                </code>
                <ChipList label="Filters">
                    { for (0..*chips).map(chip) }
                </ChipList>
                <Cluster>
                    <span class={class::CONTROL_LABEL}>
                        { format!("{} on", on.iter().take(*chips).filter(|o| **o).count()) }
                    </span>
                </Cluster>
            </Stack>
        </Demo>
    }
}
