//! Toast and ToastHost (`style/_toast.scss`).

use std::cell::Cell;

use crate::timer::use_timeout;
use crate::{Button, ButtonSize, ButtonVariant, Glyph, Icon, Tone, class, use_overlays};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

/// How long a toast shows, in milliseconds, unless told otherwise.
pub const TOAST_MS: u32 = 8000;

#[derive(Properties, PartialEq)]
pub struct ToastHostProps {
    #[prop_or_default]
    pub class: Classes,
    /// Lifted by `--toast-tray`, the height of a tray or bar the app keeps
    /// at the bottom of the window.
    #[prop_or_default]
    pub raised: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The toasts, newest last.
    #[prop_or_default]
    pub children: Html,
}

/// A host's classes: `toast-host` and `raised`.
pub fn toast_host(raised: bool) -> Classes {
    classes!(class::TOAST_HOST, raised.then_some(class::RAISED))
}

/// Where toasts stand: fixed over the window's bottom in the overlay
/// layer, in a polite live region that is always there, so a toast
/// arriving is announced, and never put out of reach by a dialog. Fixed, it
/// takes no room in the page: a toast coming or going moves nothing.
#[function_component]
pub fn ToastHost(props: &ToastHostProps) -> Html {
    use_overlays().portal(html! {
        <div class={classes!(toast_host(props.raised), props.class.clone())}
            role="region" aria-live="polite" aria-relevant="additions"
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </div>
    })
}

#[derive(Properties, PartialEq)]
pub struct ToastProps {
    #[prop_or_default]
    pub class: Classes,
    /// The accent unless set.
    #[prop_or(Tone::Accent)]
    pub tone: Tone,
    /// The icon, which the app supplies, before the text.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// The action's label ("Undo"); none for a toast of text alone.
    #[prop_or_default]
    pub action: Option<AttrValue>,
    /// The action was taken. Neither this nor `ondismiss` is called twice.
    #[prop_or_default]
    pub onaction: Callback<()>,
    /// The action's icon, which the app supplies, before its label (none by
    /// default).
    #[prop_or_default]
    pub action_icon: Option<Html>,
    /// The action button's `data-testid` (none by default).
    #[prop_or_default]
    pub action_testid: Option<AttrValue>,
    /// The toast went: closed or timed out. Not called after the action.
    #[prop_or_default]
    pub ondismiss: Callback<()>,
    /// Milliseconds it shows; 0 keeps it until dismissed. The clock stops
    /// while the pointer is over it or focus is in it, and starts afresh
    /// after. Give each toast its own `key`, so a new one has its own clock.
    #[prop_or(TOAST_MS)]
    pub duration: u32,
    /// Ctrl or Cmd+Z takes the action while the toast shows, unless typing
    /// in a field, whose own undo is left alone.
    #[prop_or_default]
    pub shortcut: bool,
    /// The close button's accessible name.
    #[prop_or(AttrValue::Static("Dismiss"))]
    pub close_label: AttrValue,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The text.
    #[prop_or_default]
    pub children: Html,
}

/// A toast's classes: `toast` and its tone.
pub fn toast(tone: Tone) -> Classes {
    classes!(class::TOAST, tone.class())
}

/// What holds a toast's clock: the pointer over it, focus in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Holds {
    pub(crate) hover: bool,
    pub(crate) focus: bool,
}

impl Holds {
    pub(crate) fn held(self) -> bool {
        self.hover || self.focus
    }
}

/// Whether a key press is the undo shortcut outside a field.
pub(crate) fn is_undo(
    key: &str,
    ctrl: bool,
    meta: bool,
    shift: bool,
    alt: bool,
    typing: bool,
) -> bool {
    (ctrl || meta) && !shift && !alt && !typing && key.eq_ignore_ascii_case("z")
}

/// A one-shot: only the first `take` is true.
#[derive(Default)]
pub(crate) struct Once(Cell<bool>);

impl Once {
    pub(crate) fn take(&self) -> bool {
        !self.0.replace(true)
    }
}

fn typing(target: Option<web_sys::EventTarget>) -> bool {
    target
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        .is_some_and(|t| {
            matches!(t.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
                || t.closest("[contenteditable]").ok().flatten().is_some()
        })
}

/// Ctrl/Cmd+Z on the document while held.
struct UndoKey {
    closure: Closure<dyn Fn(KeyboardEvent)>,
}

impl UndoKey {
    fn new(act: Callback<()>) -> Option<Self> {
        let closure = Closure::<dyn Fn(KeyboardEvent)>::new(move |e: KeyboardEvent| {
            if !e.repeat()
                && is_undo(
                    &e.key(),
                    e.ctrl_key(),
                    e.meta_key(),
                    e.shift_key(),
                    e.alt_key(),
                    typing(e.target()),
                )
            {
                e.prevent_default();
                act.emit(());
            }
        });
        let document = web_sys::window()?.document()?;
        document
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .ok()?;
        Some(Self { closure })
    }
}

impl Drop for UndoKey {
    fn drop(&mut self) {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            _ = document.remove_event_listener_with_callback(
                "keydown",
                self.closure.as_ref().unchecked_ref(),
            );
        }
    }
}

/// A short-lived message after an action ("Removed 3 items" and Undo): it
/// dismisses itself after `duration`, its clock held while the pointer or
/// focus is in it. It is one box that the host lays over the window, so its
/// coming and going changes nothing beneath; its text wraps within a
/// maximum width.
#[function_component]
pub fn Toast(props: &ToastProps) -> Html {
    let holds = use_state(Holds::default);
    let once = use_memo((), |()| Once::default());

    let act = {
        let (once, onaction) = (once.clone(), props.onaction.clone());
        Callback::from(move |()| {
            if once.take() {
                onaction.emit(());
            }
        })
    };
    let dismiss = {
        let (once, ondismiss) = (once.clone(), props.ondismiss.clone());
        Callback::from(move |()| {
            if once.take() {
                ondismiss.emit(());
            }
        })
    };

    use_timeout(
        props.duration > 0 && !holds.held(),
        0,
        props.duration,
        dismiss.reform(|_: u32| ()),
    );

    let latest = use_mut_ref(Callback::<()>::noop);
    *latest.borrow_mut() = act.clone();
    use_effect_with(props.shortcut && props.action.is_some(), move |&on| {
        let key = on
            .then(|| UndoKey::new(Callback::from(move |()| latest.borrow().emit(()))))
            .flatten();
        move || drop(key)
    });

    let hold = |set: fn(&mut Holds, bool), on: bool| {
        let holds = holds.clone();
        Callback::from(move |_: Event| {
            let mut next = *holds;
            set(&mut next, on);
            holds.set(next);
        })
    };
    let (enter, leave) = (
        hold(|h, on| h.hover = on, true),
        hold(|h, on| h.hover = on, false),
    );
    let (focusin, focusout) = (
        hold(|h, on| h.focus = on, true),
        hold(|h, on| h.focus = on, false),
    );

    html! {
        <div class={classes!(toast(props.tone), props.class.clone())}
            data-testid={props.testid.clone()}
            onmouseenter={enter.reform(|e: MouseEvent| e.into())}
            onmouseleave={leave.reform(|e: MouseEvent| e.into())}
            onfocusin={focusin.reform(|e: FocusEvent| e.into())}
            onfocusout={focusout.reform(|e: FocusEvent| e.into())}>
            if let Some(icon) = &props.icon {
                <span class={class::TOAST_ICON} aria-hidden="true">{ icon.clone() }</span>
            }
            <div class={class::TOAST_BODY}>{ props.children.clone() }</div>
            if let Some(label) = &props.action {
                <span class={class::TOAST_ACTION}>
                    <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                        icon={props.action_icon.clone()} testid={props.action_testid.clone()}
                        onclick={act.reform(|_: MouseEvent| ())}>
                        { label.clone() }
                    </Button>
                </span>
            }
            <span class={class::TOAST_CLOSE}>
                <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                    icon={html! { <Icon glyph={Glyph::Close} /> }}
                    label={props.close_label.clone()}
                    onclick={dismiss.reform(|_: MouseEvent| ())} />
            </span>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_tone_is_a_modifier() {
        assert_eq!(names(toast(Tone::Neutral)), [class::TOAST]);
        assert_eq!(names(toast(Tone::Accent)), [class::TOAST, class::ACCENT]);
        assert_eq!(names(toast(Tone::Success)), [class::TOAST, class::SUCCESS]);
        assert_eq!(names(toast(Tone::Warning)), [class::TOAST, class::WARNING]);
        assert_eq!(names(toast(Tone::Danger)), [class::TOAST, class::DANGER]);
    }

    #[test]
    fn a_host_is_raised_by_a_modifier() {
        assert_eq!(names(toast_host(false)), [class::TOAST_HOST]);
        assert_eq!(names(toast_host(true)), [class::TOAST_HOST, class::RAISED]);
    }

    #[test]
    fn the_clock_stops_while_the_pointer_or_focus_is_in() {
        let mut holds = Holds::default();
        assert!(!holds.held());
        holds.hover = true;
        assert!(holds.held());
        holds.focus = true;
        holds.hover = false;
        assert!(holds.held(), "focus alone still holds it");
        holds.focus = false;
        assert!(!holds.held());
    }

    #[test]
    fn the_action_and_the_dismissal_happen_once() {
        let once = Once::default();
        assert!(once.take());
        assert!(!once.take());
    }

    #[test]
    fn undo_is_ctrl_or_cmd_z_outside_a_field() {
        assert!(is_undo("z", true, false, false, false, false));
        assert!(is_undo("Z", false, true, false, false, false));
        assert!(!is_undo("z", false, false, false, false, false));
        assert!(!is_undo("z", true, false, true, false, false), "redo");
        assert!(!is_undo("z", true, false, false, true, false));
        assert!(!is_undo("z", true, false, false, false, true), "typing");
        assert!(!is_undo("y", true, false, false, false, false));
    }

    fn render(action_icon: Option<Html>, action_testid: Option<AttrValue>) -> String {
        let props = ToastProps {
            class: Classes::new(),
            tone: Tone::Accent,
            icon: None,
            action: Some("Undo".into()),
            onaction: Callback::noop(),
            action_icon,
            action_testid,
            ondismiss: Callback::noop(),
            duration: TOAST_MS,
            shortcut: false,
            close_label: "Dismiss".into(),
            testid: None,
            children: html! { "Removed" },
        };
        futures::executor::block_on(
            yew::LocalServerRenderer::<Toast>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn the_action_takes_an_icon_and_a_testid_only_when_given() {
        let plain = render(None, None);
        assert!(plain.contains("Undo"), "{plain}");
        assert!(!plain.contains("data-testid"), "{plain}");
        let given = render(
            Some(html! { <svg data-mark="undo" /> }),
            Some("undo".into()),
        );
        assert!(given.contains("data-mark=\"undo\""), "{given}");
        assert!(given.contains("data-testid=\"undo\""), "{given}");
    }
}
