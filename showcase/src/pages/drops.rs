use yew::prelude::*;

use plinth::{DropState, DropZone, Refusal, accepts, drop_zone, refusal};

use crate::class;
use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const ACCEPTS: &[(&str, &str)] = &[
    ("Images", "image/*"),
    ("Documents", ".pdf,.txt"),
    ("Anything", ""),
];
const SIZES: &[(&str, Option<u64>)] = &[("No limit", None), ("1 KB", Some(1024))];

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn DropZonePage() -> Html {
    let accept = use_state(|| "image/*");
    let max_size = use_state(|| None::<u64>);
    let text = use_state(|| Text::Short);
    let multiple = use_state(|| false);
    let pick = use_state(|| true);
    let paste = use_state(|| false);
    let tall = use_state(|| false);
    let busy = use_state(|| false);
    let error = use_state(|| false);
    let disabled = use_state(|| false);
    let taken = use_state(String::new);
    let refused = use_state(|| None::<Refusal>);

    let controls = html! {
        <>
            <Select<&'static str> label="accept" options={ACCEPTS} value={*accept} on_change={bind(&accept)} />
            <Select<Option<u64>> label="max_size" options={SIZES} value={*max_size} on_change={bind(&max_size)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="multiple" value={*multiple} on_change={bind(&multiple)} />
            <Toggle label="pick" value={*pick} on_change={bind(&pick)} />
            <Toggle label="paste" value={*paste} on_change={bind(&paste)} />
            <Toggle label="tall" value={*tall} on_change={bind(&tall)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="error" value={*error} on_change={bind(&error)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let words = text.of("Drop images here");
    let code = snippet(
        &[],
        El::new("DropZone")
            .text_if(!accept.is_empty(), "accept", &accept)
            .expr_if(max_size.is_some(), "max_size", "Some(1024)")
            .flag("multiple", *multiple)
            .flag("paste", *paste)
            .flag("tall", *tall)
            .expr_if(*pick, "pick", "\"Choose a file\"")
            .expr_if(*busy, "busy", "Some(\"Uploading\".into())")
            .expr_if(*error, "error", "Some(\"Upload failed\".into())")
            .flag("disabled", *disabled)
            .expr("on_files", "on_files")
            .expr("on_refused", "on_refused")
            .line_if(!words.is_empty(), text_node(&words)),
    );

    let on_files = {
        let taken = taken.clone();
        let refused = refused.clone();
        Callback::from(move |files: Vec<web_sys::File>| {
            refused.set(None);
            taken.set(
                files
                    .iter()
                    .map(|file| file.name())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        })
    };
    let on_refused = bind_some(&refused);

    let state = if *disabled {
        DropState::Idle
    } else if *busy {
        DropState::Busy
    } else if *error {
        DropState::Error
    } else {
        DropState::Idle
    };

    html! {
        <Demo {controls} {code}>
            <DropZone accept={*accept} max_size={*max_size} multiple={*multiple} paste={*paste} tall={*tall}
                pick={pick.then_some(AttrValue::Static("Choose a file"))}
                busy={busy.then_some(AttrValue::Static("Uploading"))}
                error={error.then_some(AttrValue::Static("Upload failed"))}
                disabled={*disabled} {on_files} {on_refused}>
                <p class={class::CONTROL_LABEL}>{ words }</p>
            </DropZone>
            <span class={class::CONTROL_LABEL}>
                { format!("Taken: {}", if taken.is_empty() { "none" } else { taken.as_str() }) }
            </span>
            <span class={class::CONTROL_LABEL}>{ format!("Refused: {:?}", *refused) }</span>
            <code class={class::CLASS_CALL}>
                { format!("accepts({:?}, \"a.png\", \"image/png\") → {}", *accept, accepts(&accept, "a.png", "image/png")) }
            </code>
            <code class={class::CLASS_CALL}>
                { format!("refusal({:?}, {:?}, \"a.png\", \"image/png\", 2048) → {:?}", *accept, *max_size, refusal(&accept, *max_size, "a.png", "image/png", 2048)) }
            </code>
            { shown("drop_zone(state, disabled)", &drop_zone(state, *disabled)) }
        </Demo>
    }
}

fn bind_some<T: Clone + 'static>(state: &UseStateHandle<Option<T>>) -> Callback<T> {
    let state = state.clone();
    Callback::from(move |value| state.set(Some(value)))
}
