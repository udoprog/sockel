use yew::prelude::*;

use crate::class;

use sockel::{Button, ButtonSize, ButtonVariant, Cluster, Glyph, Icon, button};

use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const VARIANTS: &[(&str, ButtonVariant)] = &[
    ("Secondary", ButtonVariant::Secondary),
    ("Primary", ButtonVariant::Primary),
    ("Ghost", ButtonVariant::Ghost),
    ("Quiet", ButtonVariant::Quiet),
    ("Danger", ButtonVariant::Danger),
    ("Success", ButtonVariant::Success),
];

const SIZES: &[(&str, ButtonSize)] = &[
    ("Regular", ButtonSize::Regular),
    ("Small", ButtonSize::Small),
    ("Touch", ButtonSize::Touch),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Placement {
    None,
    Leading,
    Trailing,
}

const ICONS: &[(&str, Placement)] = &[
    ("Leading", Placement::Leading),
    ("Trailing", Placement::Trailing),
    ("None", Placement::None),
];

const PRESSED: &[(&str, Option<bool>)] = &[
    ("Not a toggle", None),
    ("Off (click to toggle)", Some(false)),
    ("On (click to toggle)", Some(true)),
];

#[function_component]
pub fn ButtonPage() -> Html {
    let variant = use_state(ButtonVariant::default);
    let size = use_state(ButtonSize::default);
    let text = use_state(|| Text::Short);
    let placement = use_state(|| Placement::Leading);
    let fold = use_state(|| false);
    let fold_wide = use_state(|| false);
    let pressed = use_state(|| None);
    let busy = use_state(|| false);
    let disabled = use_state(|| false);
    let link = use_state(|| false);
    let selected = use_state(|| false);
    let link_look = use_state(|| false);
    let clicks = use_state(|| 0);

    let controls = html! {
        <>
            <Select<ButtonVariant> label="variant" options={VARIANTS} value={*variant} on_change={bind(&variant)} />
            <Select<ButtonSize> label="size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Select<Text> label="Label" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<Placement> label="icon" options={ICONS} value={*placement} on_change={bind(&placement)} />
            <Select<Option<bool>> label="pressed" options={PRESSED} value={*pressed} on_change={bind(&pressed)} />
            <Toggle label="fold (on a phone)" value={*fold} on_change={bind(&fold)} />
            <Toggle label="fold_wide (above a phone)" value={*fold_wide} on_change={bind(&fold_wide)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="selected" value={*selected} on_change={bind(&selected)} />
            <Toggle label="link (a text link look)" value={*link_look} on_change={bind(&link_look)} />
            <Toggle label="href (a link)" value={*link} on_change={bind(&link)} />
        </>
    };

    let onclick = {
        let clicks = clicks.clone();
        let pressed = pressed.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            clicks.set(*clicks + 1);
            if let Some(on) = *pressed {
                pressed.set(Some(!on));
            }
        })
    };

    let label = text.of("Save the draft");
    let icon_only = *placement != Placement::None && label.is_empty();
    let fold_applies = *fold && *placement != Placement::None && !label.is_empty();
    let classes = button(*variant, *size, icon_only, fold_applies);

    let code = snippet(
        &[],
        El::new("Button")
            .variant("variant", *variant)
            .variant("size", *size)
            .expr_if(
                *placement != Placement::None,
                "icon",
                "html! { <Icon glyph={Glyph::Check} /> }",
            )
            .flag("trailing", *placement == Placement::Trailing)
            .flag("fold", *fold)
            .flag("fold_wide", *fold_wide)
            .expr_if(
                pressed.is_some(),
                "pressed",
                format!("Some({})", pressed.unwrap_or_default()),
            )
            .flag("selected", *selected)
            .flag("link", *link_look)
            .flag("busy", *busy)
            .flag("disabled", *disabled)
            .text_if(*link, "href", "#button")
            .text("title", "Save the draft")
            .text_if(label.is_empty(), "label", "Save the draft")
            .expr("onclick", "on_click")
            .line_if(!label.is_empty(), text_node(&label)),
    );

    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("button(variant, size, icon_only, fold) → \"{}\"", classes.to_string()) }
            </code>
            <Cluster>
                <Button variant={*variant} size={*size}
                    icon={(*placement != Placement::None).then(|| html! { <Icon glyph={Glyph::Check} /> })}
                    trailing={*placement == Placement::Trailing}
                    fold={*fold} fold_wide={*fold_wide} pressed={*pressed} selected={*selected} link={*link_look} busy={*busy} disabled={*disabled}
                    href={link.then(|| AttrValue::from("#button"))}
                    title="Save the draft" label={label.is_empty().then(|| AttrValue::from("Save the draft"))}
                    {onclick}>
                    { label }
                </Button>
                <span class={class::CONTROL_LABEL}>{ format!("Clicked {} times", *clicks) }</span>
            </Cluster>
        </Demo>
    }
}
