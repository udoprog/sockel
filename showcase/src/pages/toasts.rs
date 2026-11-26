use yew::prelude::*;

use crate::class;

use sockel::{
    Button, Field, Glyph, Icon, SaveBar, TextInput, Toast, ToastHost, Tone, field_describedby,
    save_bar, toast, toast_host,
};

use super::fields::use_control_id;

use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const TONES: &[(&str, Tone)] = &[
    ("Neutral", Tone::Neutral),
    ("Accent", Tone::Accent),
    ("Success", Tone::Success),
    ("Warning", Tone::Warning),
    ("Danger", Tone::Danger),
];

const DURATIONS: &[(&str, u32)] = &[("8 s", 8000), ("3 s", 3000), ("Until dismissed", 0)];

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn ToastPage() -> Html {
    let tone = use_state(|| Tone::Accent);
    let text = use_state(|| Text::Short);
    let icon = use_state(|| true);
    let action = use_state(|| true);
    let shortcut = use_state(|| false);
    let live = use_state(|| false);
    let raised = use_state(|| false);
    let duration = use_state(|| 8000);
    let shows = use_state(|| 0);
    let undone = use_state(|| 0);

    let controls = html! {
        <>
            <Select<Tone> label="tone" options={TONES} value={*tone} on_change={bind(&tone)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="action" value={*action} on_change={bind(&action)} />
            <Toggle label="shortcut" value={*shortcut} on_change={bind(&shortcut)} />
            <Select<u32> label="duration" options={DURATIONS} value={*duration} on_change={bind(&duration)} />
            <Toggle label="raised" value={*raised} on_change={bind(&raised)} />
            <Toggle label="Live toast" value={*live} on_change={bind(&live)} />
        </>
    };

    let onaction = {
        let (live, undone) = (live.clone(), undone.clone());
        Callback::from(move |()| {
            undone.set(*undone + 1);
            live.set(false);
        })
    };
    let ondismiss = {
        let live = live.clone();
        Callback::from(move |()| live.set(false))
    };

    let words = text.of("Removed 3 items");
    let glyph = || icon.then(|| html! { <Icon glyph={Glyph::Check} /> });
    let code = snippet(
        &[],
        El::new("ToastHost").flag("raised", *raised).child(
            El::new("Toast")
                .expr_if(*tone != Tone::Accent, "tone", format!("Tone::{:?}", *tone))
                .expr_if(*icon, "icon", "html! { <Icon glyph={Glyph::Check} /> }")
                .text_if(*action, "action", "Undo")
                .expr_if(*action, "onaction", "on_undo")
                .expr("ondismiss", "on_dismiss")
                .expr_if(*duration != 8000, "duration", duration.to_string())
                .flag("shortcut", *shortcut)
                .line_if(!words.is_empty(), text_node(&words)),
        ),
    );

    html! {
        <Demo {controls} {code}>
            { shown("toast(tone)", &toast(*tone)) }
            { shown("toast_host(raised)", &toast_host(*raised)) }
            <Toast tone={*tone} icon={glyph()} duration=0
                action={action.then_some(AttrValue::Static("Undo"))}>
                { words.clone() }
            </Toast>
            <div class={class::SAMPLE}>{ "Page content under the toast host" }</div>
            if *live {
                <ToastHost raised={*raised}>
                    <Toast key={*shows} tone={*tone} icon={glyph()} duration={*duration}
                        shortcut={*shortcut}
                        action={action.then_some(AttrValue::Static("Undo"))}
                        onaction={onaction.clone()} ondismiss={ondismiss.clone()}>
                        { words.clone() }
                    </Toast>
                </ToastHost>
            }
            <span class={class::CONTROL_LABEL}>
                { format!("Undone {} times. A live toast is fixed over the window's bottom: tick Live toast, hover it to hold its clock, press Ctrl+Z with the shortcut.", *undone) }
            </span>
        </Demo>
    }
}

#[function_component]
pub fn SaveBarPage() -> Html {
    let dirty = use_state(|| true);
    let busy = use_state(|| false);
    let disabled = use_state(|| false);
    let discard = use_state(|| true);
    let sticky = use_state(|| false);
    let text = use_state(|| Text::Short);
    let status = use_state(|| Text::Empty);
    let saves = use_state(|| 0);
    let name = use_state(|| AttrValue::from("Ada"));
    let name_id = use_control_id();

    let controls = html! {
        <>
            <Toggle label="dirty" value={*dirty} on_change={bind(&dirty)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="ondiscard" value={*discard} on_change={bind(&discard)} />
            <Toggle label="sticky" value={*sticky} on_change={bind(&sticky)} />
            <Select<Text> label="label" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<Text> label="status" options={Text::OPTIONS} value={*status} on_change={bind(&status)} />
        </>
    };

    let onsave = {
        let saves = saves.clone();
        Callback::from(move |_: MouseEvent| saves.set(*saves + 1))
    };
    let ondiscard = Callback::from(|_: MouseEvent| ());
    let on_name = {
        let (name, dirty) = (name.clone(), dirty.clone());
        Callback::from(move |value: String| {
            name.set(value.into());
            dirty.set(true);
        })
    };

    let label = text.of("Unsaved changes");
    let status_text = status.of("Could not save: offline");
    let code = snippet(
        &[],
        El::new("SaveBar")
            .flag("dirty", *dirty)
            .flag("busy", *busy)
            .flag("disabled", *disabled)
            .text_if(*text != Text::Short, "label", &label)
            .expr_if(
                *status != Text::Empty,
                "status",
                "html! { <span>{ error }</span> }",
            )
            .expr_if(*discard, "ondiscard", "on_discard")
            .expr("onsave", "on_save")
            .flag("sticky", *sticky),
    );

    html! {
        <Demo {controls} {code}>
            { shown("save_bar(idle, sticky)", &save_bar(!*dirty, *sticky)) }
            <Field id={name_id.clone()} label={html! { "Name" }}
                hint={html! { "Sticky, in a short window: focused, it scrolls clear of the bar." }}>
                <TextInput id={name_id.clone()} value={(*name).clone()} on_input={on_name}
                    describedby={field_describedby(&name_id, true, false)} />
            </Field>
            <SaveBar dirty={*dirty} busy={*busy} disabled={*disabled} sticky={*sticky}
                label={(*text != Text::Short).then_some(label)}
                status={html! { { status_text } }}
                onsave={onsave.clone()}
                ondiscard={discard.then_some(ondiscard)}>
                <Button>{ "Preview" }</Button>
            </SaveBar>
            <div class={class::SAMPLE}>{ "Content after the form" }</div>
            <span class={class::CONTROL_LABEL}>
                { format!("Saved {} times. The bar keeps its room when clean; the browser's own prompt guards the tab only while dirty.", *saves) }
            </span>
        </Demo>
    }
}
