//! SaveBar and the unload guard (`style/_save-bar.scss`).

use crate::testid::with_testid;
use std::cell::Cell;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonVariant, class};

thread_local! {
    static HELD: Cell<usize> = const { Cell::new(0) };
}

/// A guard held: counted until dropped.
pub(crate) struct Held;

impl Held {
    pub(crate) fn take() -> Self {
        HELD.set(HELD.get() + 1);
        Self
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        HELD.set(HELD.get().saturating_sub(1));
    }
}

/// Whether any unload guard is held.
pub fn unsaved() -> bool {
    HELD.get() > 0
}

/// The browser's own prompt when the tab is closed or reloaded, held for as
/// long as this lives. Hold one only while there is something to lose.
pub struct UnloadGuard {
    _held: Held,
    closure: Option<Closure<dyn Fn(web_sys::Event)>>,
}

impl UnloadGuard {
    pub fn new() -> Self {
        let closure = Closure::<dyn Fn(web_sys::Event)>::new(|e: web_sys::Event| {
            e.prevent_default();
            if let Some(e) = e.dyn_ref::<web_sys::BeforeUnloadEvent>() {
                e.set_return_value("unsaved");
            }
        });
        let added = web_sys::window().is_some_and(|w| {
            w.add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref())
                .is_ok()
        });
        Self {
            _held: Held::take(),
            closure: added.then_some(closure),
        }
    }
}

impl Default for UnloadGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for UnloadGuard {
    fn drop(&mut self) {
        if let (Some(window), Some(closure)) = (web_sys::window(), &self.closure) {
            _ = window.remove_event_listener_with_callback(
                "beforeunload",
                closure.as_ref().unchecked_ref(),
            );
        }
    }
}

/// Holds an [`UnloadGuard`] while `active` and no longer.
#[hook]
pub fn use_unload_guard(active: bool) {
    use_effect_with(active, |&active| {
        let guard = active.then(UnloadGuard::new);
        move || drop(guard)
    });
}

#[derive(Properties, PartialEq)]
pub struct SaveBarProps {
    /// There is something to save; clean, the bar is hidden and keeps its room.
    pub dirty: bool,
    /// Saving: the save button is busy and ignores clicks.
    #[prop_or_default]
    pub busy: bool,
    /// Saving cannot be done now, besides there being nothing to save.
    #[prop_or_default]
    pub disabled: bool,
    /// The note, else the catalog's ("Unsaved changes").
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub save_label: Option<AttrValue>,
    #[prop_or_default]
    pub discard_label: Option<AttrValue>,
    /// How the last save went (an error, say), before the actions.
    #[prop_or_default]
    pub status: Html,
    #[prop_or_default]
    pub onsave: Callback<MouseEvent>,
    /// Shows a discard button, which calls this.
    #[prop_or_default]
    pub ondiscard: Option<Callback<MouseEvent>>,
    /// The save button submits the form the bar stands in.
    #[prop_or_default]
    pub submit: bool,
    /// Holds an [`UnloadGuard`] while dirty; set it to false to guard
    /// elsewhere.
    #[prop_or(true)]
    pub guard: bool,
    /// Stays at the bottom of its scroll container.
    #[prop_or_default]
    pub sticky: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// Further actions, before discard and save.
    #[prop_or_default]
    pub children: Html,
}

/// A save bar's classes: `idle` while nothing is unsaved.
pub fn save_bar(idle: bool, sticky: bool) -> Classes {
    classes!(
        class::SAVE_BAR,
        idle.then_some(class::IDLE),
        sticky.then_some(class::STICKY)
    )
}

/// The bar of a form's unsaved changes: a note, how the last save went, and
/// discard and save. It is always in the page's flow at one height and only
/// hidden while clean, so a form never moves when its first edit lights it;
/// put it at the form's end (sticky, at the bottom of its scroll container).
#[function_component]
pub fn SaveBar(props: &SaveBarProps) -> Html {
    use_unload_guard(props.guard && props.dirty);
    let phrases = use_phrases_with(&L10N);

    with_testid(
        html! {
            <div class={classes!(save_bar(!props.dirty, props.sticky), props.class.clone())}>
                <span class={class::SAVE_BAR_NOTE} role="status">{ label(&phrases, &props.label, "sockel-save-bar-unsaved") }</span>
                <span class={class::SAVE_BAR_STATUS} role="status">{ props.status.clone() }</span>
                <span class={class::SAVE_BAR_ACTIONS}>
                    { props.children.clone() }
                    if let Some(ondiscard) = &props.ondiscard {
                        <Button variant={ButtonVariant::Ghost} disabled={props.busy}
                            onclick={ondiscard.clone()}>
                            { label(&phrases, &props.discard_label, "sockel-save-bar-discard") }
                        </Button>
                    }
                    <Button variant={ButtonVariant::Primary} submit={props.submit}
                        busy={props.busy} disabled={props.disabled}
                        onclick={props.onsave.clone()}>
                        { label(&phrases, &props.save_label, "sockel-save-bar-save") }
                    </Button>
                </span>
            </div>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn save_bar_modifiers() {
        assert_eq!(names(save_bar(false, false)), [class::SAVE_BAR]);
        assert_eq!(
            names(save_bar(true, true)),
            [class::SAVE_BAR, class::IDLE, class::STICKY]
        );
    }

    #[test]
    fn a_guard_is_held_only_while_it_lives() {
        assert!(!unsaved());
        let first = Held::take();
        let second = Held::take();
        assert!(unsaved());
        drop(first);
        assert!(unsaved(), "the other still holds");
        drop(second);
        assert!(!unsaved());
    }
}
