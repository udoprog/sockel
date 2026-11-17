use yew::prelude::*;

use crate::class;

use plinth::{ActionGroup, Cluster, IconButton, action_group, icon_button};

use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind};

/// A small icon, as an app would supply one.
fn icon() -> Html {
    html! {
        <svg class={class::SVG_ICON} viewBox="0 0 16 16" width="16" height="16" aria-hidden="true"
            fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
            <path d="M8 3v10M3 8h10" />
        </svg>
    }
}

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn ActionGroupPage() -> Html {
    let end = use_state(|| false);
    let quiet = use_state(|| false);
    let nowrap = use_state(|| false);
    let in_row = use_state(|| true);
    let count = use_state(|| 3);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Toggle label="end" value={*end} on_change={bind(&end)} />
            <Toggle label="quiet" value={*quiet} on_change={bind(&quiet)} />
            <Toggle label="nowrap" value={*nowrap} on_change={bind(&nowrap)} />
            <Toggle label="In a row after text" value={*in_row} on_change={bind(&in_row)} />
            <Select<usize> label="Buttons" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Labels" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
        </>
    };

    let buttons: Html = (1..=*count)
        .map(|n| {
            let class = if *quiet {
                classes!(class::BUTTON, class::GHOST, plinth::class::ICON)
            } else {
                classes!(class::BUTTON, (n == 1).then_some(class::PRIMARY))
            };

            html! {
                <IconButton {class} title={AttrValue::from(format!("Action {n}"))}>
                    { icon() }
                    if !*quiet { <span class={class::LABEL_TEXT}>{ text.of(&format!("Action {n}")) }</span> }
                </IconButton>
            }
        })
        .collect();

    let group = html! {
        <ActionGroup end={*end} quiet={*quiet} nowrap={*nowrap}>{ buttons }</ActionGroup>
    };

    html! {
        <Demo {controls}>
            { shown("action_group(end, quiet, wrap)", &action_group(*end, *quiet, !*nowrap)) }
            if *in_row {
                <Cluster class={class::OUTLINED}>
                    <span>{ "Saved two minutes ago" }</span>
                    { group }
                </Cluster>
            } else {
                <div class={class::OUTLINED}>{ group }</div>
            }
        </Demo>
    }
}

const VARIANTS: &[(&str, &str)] = &[
    ("Plain", ""),
    ("primary", class::PRIMARY),
    ("ghost", class::GHOST),
    ("danger", class::DANGER),
    ("icon (icon alone)", plinth::class::ICON),
];

const DIALOGS: &[(&str, bool)] = &[("No", false), ("Yes (click to open)", true)];

#[function_component]
pub fn IconButtonPage() -> Html {
    let variant = use_state(|| "");
    let trailing = use_state(|| false);
    let disabled = use_state(|| false);
    let small = use_state(|| false);
    let named = use_state(|| false);
    let dialog = use_state(|| false);
    let open = use_state(|| false);
    let clicks = use_state(|| 0);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Select<&'static str> label="class" options={VARIANTS} value={*variant} on_change={bind(&variant)} />
            <Select<Text> label="Label" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<bool> label="dialog" options={DIALOGS} value={*dialog} on_change={bind(&dialog)} />
            <Toggle label="trailing" value={*trailing} on_change={bind(&trailing)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="Small" value={*small} on_change={bind(&small)} />
            <Toggle label="label (aria-label)" value={*named} on_change={bind(&named)} />
        </>
    };

    let onclick = {
        let open = open.clone();
        let clicks = clicks.clone();
        let dialog = *dialog;
        Callback::from(move |_: MouseEvent| {
            clicks.set(*clicks + 1);
            if dialog {
                open.set(!*open);
            }
        })
    };

    let alone = *variant == plinth::class::ICON;
    let class = classes!(
        class::BUTTON,
        (!variant.is_empty()).then_some(*variant),
        small.then_some(class::SMALL)
    );
    let classes = icon_button(class.clone(), *trailing);
    let label = (!alone)
        .then(|| html! { <span class={class::LABEL_TEXT}>{ text.of("Add an item") }</span> });

    html! {
        <Demo {controls}>
            { shown("icon_button(class, trailing)", &classes) }
            <Cluster>
                <IconButton {class} trailing={*trailing} disabled={*disabled}
                    dialog={dialog.then_some(*open)} title="Add an item"
                    label={named.then(|| AttrValue::from("Add an item to the list"))} {onclick}>
                    if *trailing { { for label.clone() } { icon() } } else { { icon() } { for label } }
                </IconButton>
                <span class={class::CONTROL_LABEL}>
                    { format!("Clicked {} times{}", *clicks, if *dialog { format!(", dialog {}", if *open { "open" } else { "closed" }) } else { String::new() }) }
                </span>
            </Cluster>
        </Demo>
    }
}
