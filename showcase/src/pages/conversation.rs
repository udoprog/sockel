//! The pages of the parts a conversation is made of: Lightbox, LogView and
//! Composer.

use yew::prelude::*;

use sockel::{
    Button, ComposeKey, Composer, ComposerMessage, Glyph, Icon, Image, LIGHTBOX_ZOOMS,
    LOG_END_SLOP, Lightbox, LightboxItem, LightboxKey, LogGroup, LogView, Ratio, Stack, at_end,
    compose_key, composed, composer, composer_draft, lightbox, lightbox_key, log_group, log_view,
    set_composer_draft, swipe_step,
};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Toggle, bind};

/// A picture of `w` by `h` in a colour, drawn here: no file or network.
fn picture(w: u32, h: u32, hue: u32, label: &str) -> String {
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' viewBox='0 0 {w} {h}'>\
         <rect width='100%' height='100%' fill='hsl({hue} 55% 45%)'/>\
         <circle cx='{}' cy='{}' r='{}' fill='hsl({hue} 70% 70%)'/>\
         <text x='50%' y='92%' font-family='sans-serif' font-size='{}' fill='white' text-anchor='middle'>{label} {w}×{h}</text></svg>",
        w / 2,
        h / 2,
        w.min(h) / 4,
        (w.min(h) / 12).max(12),
    );
    format!(
        "data:image/svg+xml,{}",
        svg.replace('#', "%23")
            .replace(' ', "%20")
            .replace('<', "%3C")
            .replace('>', "%3E")
            .replace('\'', "%27")
    )
}

fn pictures(captions: bool, originals: bool) -> Vec<LightboxItem> {
    [
        (1600, 900, 210, "Board"),
        (900, 1400, 30, "Phone"),
        (1200, 1200, 140, "Square"),
        (2400, 600, 280, "Wide"),
    ]
    .into_iter()
    .map(|(w, h, hue, name)| {
        let src = picture(w, h, hue, name);
        let mut item = LightboxItem::new(src.clone(), format!("{name} screenshot"));
        if captions {
            item = item.caption(format!(
                "{name}: {w} by {h}, a caption under the image that may run to two lines"
            ));
        }
        if originals {
            item = item.href(src);
        }
        item
    })
    .collect()
}

#[function_component]
pub fn LightboxPage() -> Html {
    let captions = use_state(|| true);
    let originals = use_state(|| false);
    let index = use_state(|| None::<usize>);
    let items = pictures(*captions, *originals);

    let controls = html! {
        <>
            <Toggle label="caption" value={*captions} on_change={bind(&captions)} />
            <Toggle label="href" value={*originals} on_change={bind(&originals)} />
        </>
    };
    let code = snippet(
        &["let items = vec![LightboxItem::new(src, \"Board screenshot\").caption(\"Board\"), ..];".into()],
        El::new("Lightbox")
            .expr("items", "items")
            .expr("index", "shown")
            .expr("on_index", "show")
            .expr("on_close", "close"),
    );
    let on_index = {
        let index = index.clone();
        Callback::from(move |i: usize| index.set(Some(i)))
    };
    let on_close = {
        let index = index.clone();
        Callback::from(move |()| index.set(None))
    };
    let keys = format!(
        "lightbox(zoomed) → \"{}\" · zooms {:?} · ArrowRight → {:?} · a swipe of -80 px → {:?}",
        lightbox(true).to_string(),
        LIGHTBOX_ZOOMS,
        lightbox_key("ArrowRight", false).map(|k| k == LightboxKey::Next),
        swipe_step(-80.0, 0.0),
    );

    html! {
        <>
            <Demo {controls} {code}>
                <div class={class::THUMBS}>
                    { for items.iter().enumerate().map(|(i, item)| {
                        let index = index.clone();
                        html! {
                            <button type="button" class={class::THUMB} aria-label={format!("Open {}", item.alt)}
                                onclick={Callback::from(move |_| index.set(Some(i)))}>
                                <Image src={item.src.clone()} alt={item.alt.clone()} ratio={Ratio::Square} />
                            </button>
                        }
                    }) }
                </div>
                <code class={class::CLASS_CALL}>{ keys }</code>
                <span class={class::CONTROL_LABEL}>
                    { "Open one: arrows or a swipe step, + and - zoom (or double-click), 0 fits, a zoomed image pans by scrolling, Escape closes." }
                </span>
            </Demo>
            <Lightbox {items} index={*index} {on_index} {on_close} />
        </>
    }
}

const TOOLS: [&str; 4] = [
    "read src/lib.rs",
    "grep use_shortcut",
    "cargo test",
    "edit README.md",
];

#[function_component]
pub fn LogViewPage() -> Html {
    let rows = use_state(|| 12usize);
    let grouped = use_state(|| true);
    let following = use_state(|| true);

    let add = |n: usize| {
        let rows = rows.clone();
        Callback::from(move |_: MouseEvent| rows.set(*rows + n))
    };
    let controls = html! {
        <>
            <Toggle label="Grouped" value={*grouped} on_change={bind(&grouped)} />
            <Button onclick={add(1)}>{ "Add a row" }</Button>
            <Button onclick={add(10)}>{ "Add 10" }</Button>
        </>
    };
    let code = snippet(
        &[],
        El::new("LogView")
            .expr("count", "rows.len()")
            .text("label", "Transcript")
            .expr("on_follow", "set_following")
            .line("{ for rows.iter().map(row) }")
            .child(
                El::new("LogGroup")
                    .expr("summary", "html! { \"4 tool calls\" }")
                    .expr("count", "Some(4)")
                    .line("{ for calls.iter().map(row) }"),
            ),
    );
    let on_follow = {
        let following = following.clone();
        Callback::from(move |on: bool| following.set(on))
    };
    let row = |n: usize| html! { <p class={class::LOG_ROW}>{ format!("{n}. A message in the log, as the app renders it.") }</p> };

    html! {
        <Demo {controls} {code}>
            <div class={class::LOG_BOX}>
                <LogView count={*rows} label="Transcript" {on_follow}>
                    { for (1..=*rows).map(|n| {
                        if *grouped && n % 5 == 0 {
                            html! {
                                <LogGroup key={n} summary={html! { format!("{} tool calls", TOOLS.len()) }} count={Some(TOOLS.len())}>
                                    { for TOOLS.iter().map(|t| html! { <p class={class::LOG_ROW}>{ *t }</p> }) }
                                </LogGroup>
                            }
                        } else {
                            row(n)
                        }
                    }) }
                </LogView>
            </div>
            <span class={class::CONTROL_LABEL}>
                { format!(
                    "{} rows, {}. Scroll up and add rows: it holds still and counts them; Jump to latest returns. log_view(true) → \"{}\", log_group(true) → \"{}\", within {LOG_END_SLOP} px of the end is at it ({}).",
                    *rows,
                    if *following { "following" } else { "held" },
                    log_view(true).to_string(),
                    log_group(true).to_string(),
                    at_end(980.0, 1000.0, 0.0),
                ) }
            </span>
        </Demo>
    }
}

const KEYS: &[(&str, &str)] = &[("Session A", "session-a"), ("Session B", "session-b")];

#[function_component]
pub fn ComposerPage() -> Html {
    let key = use_state(|| "session-a");
    let enter_sends = use_state(|| true);
    let attachments = use_state(|| true);
    let busy = use_state(|| false);
    let icon = use_state(|| true);
    let sent = use_state(Vec::<String>::new);
    let controlled = use_state(|| false);
    let value = use_state(String::new);

    let controls = html! {
        <>
            <Select<&'static str> label="draft_key" options={KEYS} value={*key} on_change={bind(&key)} />
            <Toggle label="enter_sends" value={*enter_sends} on_change={bind(&enter_sends)} />
            <Toggle label="attachments" value={*attachments} on_change={bind(&attachments)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="send_icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="value (controlled)" value={*controlled} on_change={bind(&controlled)} />
        </>
    };
    let code = snippet(
        &[],
        El::new("Composer")
            .expr("draft_key", "Some(session.id.clone())")
            .text("placeholder", "Message the lead")
            .flag("attachments", *attachments)
            .expr_if(!*enter_sends, "enter_sends", "false")
            .expr_if(
                *icon,
                "send_icon",
                "Some(html! { <Icon glyph={Glyph::ChevronUp} /> })",
            )
            .flag("busy", *busy)
            .expr_if(*controlled, "value", "Some(text.clone().into())")
            .expr_if(*controlled, "on_input", "set_text")
            .text("send_tip", "Send (Enter; Shift+Enter starts a new line)")
            .expr("on_send", "send"),
    );
    let on_send = {
        let (sent, value) = (sent.clone(), value.clone());
        Callback::from(move |m: ComposerMessage| {
            let mut all = (*sent).clone();
            all.push(format!("{:?} with {} files", m.text, m.files.len()));
            sent.set(all);
            // Controlled: the app clears the text once the send is confirmed.
            value.set(String::new());
        })
    };
    let on_input = {
        let value = value.clone();
        Callback::from(move |text: String| value.set(text))
    };
    let other = if *key == "session-a" {
        "session-b"
    } else {
        "session-a"
    };
    let kept = composer_draft(other).unwrap_or_default();
    // Seeds the other session's draft once, so switching shows one kept.
    if composer_draft("session-b").is_none() && sent.is_empty() {
        set_composer_draft("session-b", "A draft kept for session B");
    }
    let rule = format!(
        "Enter → {:?}, Shift+Enter → {:?}; composed(\"\\nhi\\n\") = {:?}; composer(busy) → \"{}\"",
        compose_key("Enter", *enter_sends, false, false, false, false) == ComposeKey::Send,
        compose_key("Enter", *enter_sends, true, false, false, false),
        composed("\nhi\n"),
        composer(true).to_string(),
    );

    html! {
        <Demo {controls} {code}>
            <Stack>
                <ul class={class::SAMPLE}>
                    { for sent.iter().map(|s| html! { <li>{ s.clone() }</li> }) }
                </ul>
                <Composer draft_key={AttrValue::from(*key)} placeholder="Message the lead"
                    attachments={*attachments} enter_sends={*enter_sends} busy={*busy}
                    send_icon={icon.then(|| html! { <Icon glyph={Glyph::ChevronUp} /> })}
                    value={controlled.then(|| AttrValue::from((*value).clone()))} {on_input}
                    send_tip="Send (Enter; Shift+Enter starts a new line)"
                    field_testid="composer-field" send_testid="composer-send"
                    {on_send} />
                <span class={class::CONTROL_LABEL}>
                    { format!("The other session's draft: {kept:?}. Drop or paste a file, or Attach. {rule}") }
                </span>
            </Stack>
        </Demo>
    }
}
