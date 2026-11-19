//! Composer: a message's box at the foot of a conversation, which grows
//! with its text, sends on Enter, takes attachments and keeps a draft per
//! key (`style/_composer.scss`).

use std::cell::RefCell;
use std::collections::HashMap;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use web_sys::{File, HtmlTextAreaElement};
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{
    Button, ButtonSize, ButtonVariant, DropZone, Glyph, Icon, InputSize, Refusal, class, textarea,
};

thread_local! {
    static DRAFTS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// The draft kept for `key`, if any: what a [`Composer`] with that
/// `draft_key` held when it was last left.
pub fn composer_draft(key: &str) -> Option<String> {
    DRAFTS.with_borrow(|d| d.get(key).cloned())
}

/// Sets (or with an empty text, drops) the draft kept for `key`: an app
/// restoring drafts it saved itself, or clearing one.
pub fn set_composer_draft(key: &str, text: &str) {
    DRAFTS.with_borrow_mut(|d| {
        if text.is_empty() {
            d.remove(key);
        } else {
            d.insert(key.to_owned(), text.to_owned());
        }
    });
}

/// What Enter does in a composer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposeKey {
    Send,
    /// A newline (Shift or Alt+Enter, or Enter when it does not send).
    Newline,
    /// Any other key, left to the field.
    Type,
}

/// What a press of `key` does: with `enter_sends`, Enter sends and Shift
/// or Alt+Enter is a newline (terminals and keyboards differ, so both);
/// without it Enter is a newline and Ctrl or Cmd+Enter sends. A press
/// while composing is the input method's.
pub fn compose_key(
    key: &str,
    enter_sends: bool,
    shift: bool,
    alt: bool,
    mod_held: bool,
    composing: bool,
) -> ComposeKey {
    match key {
        "Enter" if composing => ComposeKey::Type,
        "Enter" if mod_held => ComposeKey::Send,
        "Enter" if enter_sends && !shift && !alt => ComposeKey::Send,
        "Enter" => ComposeKey::Newline,
        _ => ComposeKey::Type,
    }
}

/// What a composer sends: its text, trimmed of its ends' blank lines, and
/// the files attached.
#[derive(Clone, Debug, PartialEq)]
pub struct ComposerMessage {
    pub text: String,
    pub files: Vec<File>,
}

/// A composer's classes: `composer`, `busy` while sending.
pub fn composer(busy: bool) -> Classes {
    classes!(class::COMPOSER, busy.then_some(class::BUSY))
}

#[derive(Properties, PartialEq)]
pub struct ComposerProps {
    /// Told the message on Send; the composer then empties itself and its
    /// draft. Nothing is sent while there is neither text nor a file.
    pub on_send: Callback<ComposerMessage>,
    /// The key its draft is kept under (a conversation's id), across
    /// unmounting and between keys; none kept while `None`.
    #[prop_or_default]
    pub draft_key: Option<AttrValue>,
    /// The field's accessible name, else the catalog's.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    /// Enter sends (and Shift or Alt+Enter is a newline); on by default.
    /// Off, Enter is a newline and Ctrl or Cmd+Enter sends.
    #[prop_or(true)]
    pub enter_sends: bool,
    /// Takes files: an attach button, and files dropped on or pasted into
    /// the composer, as a [`DropZone`] takes them.
    #[prop_or_default]
    pub attachments: bool,
    /// The files taken, as for a file input's `accept`.
    #[prop_or_default]
    pub accept: AttrValue,
    /// The largest file taken, in bytes.
    #[prop_or_default]
    pub max_size: Option<u64>,
    /// A file was refused, and why.
    #[prop_or_default]
    pub on_refused: Callback<Refusal>,
    /// Told the text on each edit (a slash command's menu).
    #[prop_or_default]
    pub on_input: Callback<String>,
    /// Told each key press in the field before the composer acts on it; one
    /// whose default the app prevents is left to it (Up in an empty box).
    #[prop_or_default]
    pub on_keydown: Callback<KeyboardEvent>,
    /// The Send button's icon; with one, Send is that icon alone below the
    /// phone breakpoint.
    #[prop_or_default]
    pub send_icon: Option<Html>,
    #[prop_or_default]
    pub send_label: Option<AttrValue>,
    #[prop_or_default]
    pub attach_label: Option<AttrValue>,
    /// Controls before Send (a Comment or Note choice).
    #[prop_or_default]
    pub actions: Html,
    /// Sending: Send is busy and takes nothing more.
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// The text a composer sends: `text` without blank lines at its ends.
pub fn composed(text: &str) -> String {
    text.trim_matches(|c: char| c == '\n' || c == '\r')
        .trim_end()
        .to_owned()
}

/// Fits a textarea's height to its text, up to the CSS `max-height`
/// (`--composer-max-height`), past which it scrolls.
fn fit(area: &HtmlTextAreaElement) {
    let style = area.style();
    _ = style.set_property("height", "auto");
    _ = style.set_property("height", &format!("{}px", area.scroll_height() + 2));
}

/// A growing textarea (from one line to `--composer-max-height`) with Send
/// at its end: Enter sends, Shift or Alt+Enter is a newline; with
/// `attachments`, files dropped, pasted or picked are listed above it, each
/// removable, and go with the message. Its draft is kept per `draft_key`.
#[function_component]
pub fn Composer(props: &ComposerProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let key = props.draft_key.clone();
    let text = use_state(|| key.as_deref().and_then(composer_draft).unwrap_or_default());
    let files = use_state(Vec::<File>::new);
    let area = use_node_ref();

    // Another key: its own draft.
    {
        let (text, area) = (text.clone(), area.clone());
        use_effect_with(key.clone(), move |key| {
            let draft = key.as_deref().and_then(composer_draft).unwrap_or_default();
            if *text != draft {
                text.set(draft);
            }
            if let Some(area) = area.cast::<HtmlTextAreaElement>() {
                fit(&area);
            }
        });
    }
    {
        let area = area.clone();
        use_effect_with((*text).clone(), move |_| {
            if let Some(area) = area.cast::<HtmlTextAreaElement>() {
                fit(&area);
            }
        });
    }

    let send = {
        let (text, files, key) = (text.clone(), files.clone(), key.clone());
        let (on_send, blocked) = (props.on_send.clone(), props.busy || props.disabled);
        Callback::from(move |()| {
            let message = ComposerMessage {
                text: composed(&text),
                files: (*files).clone(),
            };
            if blocked || (message.text.is_empty() && message.files.is_empty()) {
                return;
            }
            if let Some(key) = &key {
                set_composer_draft(key, "");
            }
            text.set(String::new());
            files.set(Vec::new());
            on_send.emit(message);
        })
    };
    let oninput = {
        let (text, key, on_input) = (text.clone(), key.clone(), props.on_input.clone());
        Callback::from(move |e: InputEvent| {
            let Some(area) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
            else {
                return;
            };
            let value = area.value();
            fit(&area);
            if let Some(key) = &key {
                set_composer_draft(key, &value);
            }
            on_input.emit(value.clone());
            text.set(value);
        })
    };
    let onkeydown = {
        let (send, on_keydown, enter_sends) =
            (send.clone(), props.on_keydown.clone(), props.enter_sends);
        Callback::from(move |e: KeyboardEvent| {
            on_keydown.emit(e.clone());
            if e.default_prevented() {
                return;
            }
            let action = compose_key(
                &e.key(),
                enter_sends,
                e.shift_key(),
                e.alt_key(),
                e.ctrl_key() || e.meta_key(),
                e.is_composing(),
            );
            if action == ComposeKey::Send {
                e.prevent_default();
                send.emit(());
            }
        })
    };
    let on_files = {
        let files = files.clone();
        Callback::from(move |taken: Vec<File>| {
            let mut all = (*files).clone();
            all.extend(taken);
            files.set(all);
        })
    };
    let onclick = Callback::from(move |_: MouseEvent| send.emit(()));

    let field = html! {
        <textarea ref={area} class={classes!(textarea(InputSize::Regular, false, false), class::COMPOSER_FIELD)}
            rows="1" value={(*text).clone()}
            aria-label={label(&phrases, &props.label, "sockel-composer-label")}
            placeholder={props.placeholder.clone()} disabled={props.disabled}
            {oninput} {onkeydown} />
    };
    let list = (!files.is_empty()).then(|| {
        html! {
            <ul class={class::COMPOSER_FILES}>
                { for files.iter().enumerate().map(|(i, file)| {
                    let remove = {
                        let files = files.clone();
                        Callback::from(move |_: MouseEvent| {
                            let mut left = (*files).clone();
                            left.remove(i);
                            files.set(left);
                        })
                    };
                    html! {
                        <li class={class::COMPOSER_FILE}>
                            <span class={class::COMPOSER_FILE_NAME}>{ file.name() }</span>
                            <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                                icon={html! { <Icon glyph={Glyph::Close} /> }}
                                label={label(&phrases, &None, "sockel-composer-remove")}
                                onclick={remove} />
                        </li>
                    }
                }) }
            </ul>
        }
    });
    let empty = composed(&text).is_empty() && files.is_empty();
    let send_label = label(&phrases, &props.send_label, "sockel-composer-send-label");
    let send_button = html! {
        <Button variant={ButtonVariant::Primary} icon={props.send_icon.clone()}
            fold={props.send_icon.is_some()} label={send_label.clone()}
            busy={props.busy} disabled={props.disabled || empty} {onclick}
            class={class::COMPOSER_SEND}>
            { send_label.clone() }
        </Button>
    };
    let bar = html! {
        <div class={class::COMPOSER_BAR}>
            { field }
            <div class={class::COMPOSER_ACTIONS}>
                { props.actions.clone() }
                { send_button }
            </div>
        </div>
    };

    let body = if props.attachments {
        html! {
            <DropZone accept={props.accept.clone()} max_size={props.max_size} multiple=true
                paste=true disabled={props.disabled || props.busy}
                pick={Some(label(&phrases, &props.attach_label, "sockel-composer-attach"))}
                {on_files} on_refused={props.on_refused.clone()}>
                { for list }
                { bar }
            </DropZone>
        }
    } else {
        bar
    };

    html! {
        <div class={classes!(composer(props.busy), props.class.clone())} data-testid={props.testid.clone()}>
            { body }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_sends_unless_shifted_or_composing() {
        assert_eq!(
            compose_key("Enter", true, false, false, false, false),
            ComposeKey::Send
        );
        assert_eq!(
            compose_key("Enter", true, true, false, false, false),
            ComposeKey::Newline
        );
        assert_eq!(
            compose_key("Enter", true, false, true, false, false),
            ComposeKey::Newline
        );
        assert_eq!(
            compose_key("Enter", true, false, false, false, true),
            ComposeKey::Type
        );
        assert_eq!(
            compose_key("Enter", false, false, false, false, false),
            ComposeKey::Newline
        );
        assert_eq!(
            compose_key("Enter", false, false, false, true, false),
            ComposeKey::Send
        );
        assert_eq!(
            compose_key("a", true, false, false, false, false),
            ComposeKey::Type
        );
    }

    #[test]
    fn a_draft_is_kept_per_key_and_an_empty_one_dropped() {
        set_composer_draft("a", "hello");
        set_composer_draft("b", "other");
        assert_eq!(composer_draft("a").as_deref(), Some("hello"));
        set_composer_draft("a", "");
        assert_eq!(composer_draft("a"), None);
        assert_eq!(composer_draft("b").as_deref(), Some("other"));
    }

    #[test]
    fn blank_ends_are_not_sent() {
        assert_eq!(composed("\n\nhi\nthere\n\n"), "hi\nthere");
        assert_eq!(composed("  \n"), "");
    }

    #[test]
    fn it_starts_from_its_draft() {
        #[function_component]
        fn Shown() -> Html {
            html! { <Composer draft_key="kept" on_send={Callback::noop()} class={class::SMALL} /> }
        }
        set_composer_draft("kept", "half a thought");
        let html = futures::executor::block_on(yew::LocalServerRenderer::<Shown>::new().render());
        assert!(html.contains("half a thought"), "{html}");
        assert!(
            html.contains(&format!("{} {}", class::COMPOSER, class::SMALL)),
            "{html}"
        );
    }
}
