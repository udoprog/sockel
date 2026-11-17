//! SecretInput (`style/_inputs.scss`).

use std::rc::Rc;

use crate::input::{InputSize, input_box};
use crate::timer::use_timeout;
use crate::{Glyph, Icon, class};
use web_sys::HtmlInputElement;
use yew::prelude::*;

/// How long a revealed secret stays visible, in milliseconds.
pub const REVEAL_MS: u32 = 3000;

/// Whether the secret is shown. Each show has a generation, so an expiry
/// that belongs to an earlier show does not hide a later one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reveal {
    pub shown: bool,
    pub generation: u32,
}

impl Reveal {
    pub fn show(self) -> Self {
        Self {
            shown: true,
            generation: self.generation + 1,
        }
    }

    pub fn hide(self) -> Self {
        Self {
            shown: false,
            ..self
        }
    }

    /// The hide the timer of `generation` asks for.
    pub fn expire(self, generation: u32) -> Self {
        if self.shown && self.generation == generation {
            self.hide()
        } else {
            self
        }
    }
}

enum RevealAction {
    Toggle,
    Hide,
    Expire(u32),
}

#[derive(Default)]
struct RevealState(Reveal);

impl Reducible for RevealState {
    type Action = RevealAction;

    fn reduce(self: Rc<Self>, action: RevealAction) -> Rc<Self> {
        let reveal = self.0;
        Rc::new(RevealState(match action {
            RevealAction::Toggle if reveal.shown => reveal.hide(),
            RevealAction::Toggle => reveal.show(),
            RevealAction::Hide => reveal.hide(),
            RevealAction::Expire(generation) => reveal.expire(generation),
        }))
    }
}

#[derive(Properties, PartialEq)]
pub struct SecretInputProps {
    pub value: AttrValue,
    /// Each edit, with the new text; `""` when cleared.
    pub on_input: Callback<String>,
    /// The accessible name when there is no label for `id`.
    pub label: AttrValue,
    /// The secret, when the copy button is pressed (the text is on the
    /// clipboard too).
    #[prop_or_default]
    pub on_copy: Callback<String>,
    #[prop_or("Show".into())]
    pub show_label: AttrValue,
    #[prop_or("Hide".into())]
    pub hide_label: AttrValue,
    #[prop_or("Copy".into())]
    pub copy_label: AttrValue,
    #[prop_or("Clear".into())]
    pub clear_label: AttrValue,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    /// Milliseconds a shown secret stays shown.
    #[prop_or(REVEAL_MS)]
    pub reveal_ms: u32,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub autofocus: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A password field with three slots after the text: show or hide (the
/// secret hides itself after `reveal_ms`, and when cleared), copy and
/// clear. The text, the slots and the box keep their size whether the
/// secret is dots or text, and while the field is empty (the slots only
/// hidden), so showing, copying or typing moves nothing.
#[function_component]
pub fn SecretInput(props: &SecretInputProps) -> Html {
    let state = use_reducer(RevealState::default);
    let reveal = state.0;
    let empty = props.value.is_empty();
    let shown = reveal.shown && !empty;

    use_timeout(
        shown,
        reveal.generation,
        props.reveal_ms,
        Callback::from({
            let dispatcher = state.dispatcher();
            move |generation| dispatcher.dispatch(RevealAction::Expire(generation))
        }),
    );

    let oninput = props
        .on_input
        .reform(|e: InputEvent| e.target_unchecked_into::<HtmlInputElement>().value());
    let toggle = {
        let dispatcher = state.dispatcher();
        Callback::from(move |_: MouseEvent| dispatcher.dispatch(RevealAction::Toggle))
    };
    let copy = {
        let value = props.value.clone();
        let on_copy = props.on_copy.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(window) = web_sys::window() {
                let _ = window.navigator().clipboard().write_text(&value);
            }
            on_copy.emit(value.to_string());
        })
    };
    let clear = {
        let on_input = props.on_input.clone();
        let dispatcher = state.dispatcher();
        Callback::from(move |_: MouseEvent| {
            dispatcher.dispatch(RevealAction::Hide);
            on_input.emit(String::new());
        })
    };

    let action = |glyph: Glyph, label: &AttrValue, onclick: Callback<MouseEvent>| {
        html! {
            <button type="button" class={classes!(class::INPUT_ACTION, empty.then_some(class::EMPTY))}
                aria-label={label.clone()} disabled={props.disabled || empty}
                aria-hidden={empty.then_some("true")} tabindex={empty.then_some("-1")} {onclick}>
                <Icon {glyph} />
            </button>
        }
    };
    let (glyph, label) = if shown {
        (Glyph::EyeOff, &props.hide_label)
    } else {
        (Glyph::Eye, &props.show_label)
    };

    html! {
        <span class={classes!(
            input_box(props.size, props.invalid, props.disabled, props.busy),
            props.class.clone()
        )} aria-busy={props.busy.then_some("true")}>
            <input class={class::INPUT_CONTROL} type={if shown { "text" } else { "password" }}
                id={props.id.clone()} value={props.value.clone()}
                placeholder={props.placeholder.clone()} aria-label={props.label.clone()}
                aria-invalid={props.invalid.then_some("true")} disabled={props.disabled}
                autocomplete="off" spellcheck="false" autocapitalize="off"
                data-autofocus={props.autofocus.then_some("")}
                data-testid={props.testid.clone()} {oninput} />
            { action(glyph, label, toggle) }
            { action(Glyph::Copy, &props.copy_label, copy) }
            { action(Glyph::Close, &props.clear_label, clear) }
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(props: SecretInputProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<SecretInput>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props(value: &str) -> SecretInputProps {
        SecretInputProps {
            value: value.to_owned().into(),
            on_input: Callback::noop(),
            label: "Key".into(),
            on_copy: Callback::noop(),
            show_label: "Show".into(),
            hide_label: "Hide".into(),
            copy_label: "Copy".into(),
            clear_label: "Clear".into(),
            placeholder: None,
            reveal_ms: REVEAL_MS,
            id: None,
            size: InputSize::Regular,
            invalid: false,
            disabled: false,
            busy: false,
            autofocus: false,
            class: Classes::new(),
            testid: None,
        }
    }

    #[test]
    fn a_shown_secret_hides_when_its_timer_expires() {
        let shown = Reveal::default().show();
        assert!(shown.shown);
        assert!(!shown.expire(shown.generation).shown);
    }

    #[test]
    fn showing_again_restarts_the_timer() {
        let first = Reveal::default().show();
        let again = first.show();
        assert_ne!(first.generation, again.generation);
        assert!(again.expire(first.generation).shown);
        assert!(!again.expire(again.generation).shown);
    }

    #[test]
    fn a_late_expiry_does_not_show_or_hide_a_hidden_secret() {
        let hidden = Reveal::default().show().hide();
        assert_eq!(hidden.expire(hidden.generation), hidden);
    }

    #[test]
    fn the_slots_keep_their_room_empty_or_filled() {
        let empty = render(props(""));
        let filled = render(props("s3cret"));
        for html in [&empty, &filled] {
            assert_eq!(html.matches(class::INPUT_ACTION).count(), 3, "{html}");
            assert!(html.contains("type=\"password\""), "{html}");
        }
        assert_eq!(empty.matches(class::EMPTY).count(), 3, "{empty}");
        assert!(!filled.contains(class::EMPTY), "{filled}");
        assert!(filled.contains("aria-label=\"Show\""), "{filled}");
    }
}
