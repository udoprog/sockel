//! SaveBar and the unload guard (`style/_save-bar.scss`).

use crate::testid::with_testid;
use std::cell::Cell;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, HtmlElement};
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

/// What scrolls `bar`: the nearest element around it that scrolls up and
/// down, else the page.
fn scroller_of(bar: &Element) -> Option<Element> {
    let window = web_sys::window()?;
    let mut at = bar.parent_element();
    while let Some(el) = at {
        if let Ok(Some(style)) = window.get_computed_style(&el)
            && let Ok(overflow) = style.get_property_value("overflow-y")
            && matches!(overflow.as_str(), "auto" | "scroll")
        {
            return Some(el);
        }
        at = el.parent_element();
    }
    window.document()?.scrolling_element()
}

/// What must stay clear of the bar while `focused` has focus: its field,
/// hint and error included, where it is in one.
fn kept_clear(focused: Element) -> Element {
    #[cfg(feature = "field")]
    if let Ok(Some(field)) = focused.closest(&format!(".{}", class::FIELD)) {
        return field;
    }
    focused
}

/// How far to scroll so that `kept` (a box, from its top to its bottom)
/// ends `gap` above `bar` (its top and bottom), without scrolling
/// `focused_top` above `view_top`. None when it is clear already, or beyond
/// the bar, where the browser's own scrolling brings it in.
fn overlap(
    kept: (f64, f64),
    focused_top: f64,
    bar: (f64, f64),
    gap: f64,
    view_top: f64,
) -> Option<f64> {
    let ((top, bottom), (bar_top, bar_bottom)) = (kept, bar);
    if bottom <= bar_top - gap || top >= bar_bottom {
        return None;
    }
    let by = (bottom + gap - bar_top).min(focused_top - view_top);
    (by >= 1.0).then_some(by)
}

/// Scrolls the focused element (its field, where it is in one) out from
/// under a sticky `bar`.
fn clear_focus(bar: &Element) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(focused) = window.document().and_then(|d| d.active_element()) else {
        return;
    };
    if bar.contains(Some(&focused)) || focused.tag_name() == "BODY" {
        return;
    }
    let Some(scroller) = scroller_of(bar) else {
        return;
    };
    let page = window
        .document()
        .and_then(|d| d.scrolling_element())
        .is_some_and(|p| p == scroller);
    let view_top = if page {
        0.0
    } else {
        scroller.get_bounding_client_rect().top()
    };
    let gap = window
        .get_computed_style(bar)
        .ok()
        .flatten()
        .and_then(|s| s.get_property_value("bottom").ok())
        .and_then(|b| b.strip_suffix("px").and_then(|n| n.parse::<f64>().ok()))
        .unwrap_or(0.0);
    let bar_rect = bar.get_bounding_client_rect();
    let focused_rect = focused.get_bounding_client_rect();
    let kept = kept_clear(focused).get_bounding_client_rect();
    if let Some(by) = overlap(
        (kept.top(), kept.bottom()),
        focused_rect.top(),
        (bar_rect.top(), bar_rect.bottom()),
        gap,
        view_top,
    ) {
        scroller.scroll_by_with_x_and_y(0.0, by);
    }
}

/// While held, keeps whatever takes focus clear of a sticky bar.
struct FocusClearance {
    closure: Option<Closure<dyn Fn(web_sys::Event)>>,
}

impl FocusClearance {
    fn new(bar: NodeRef) -> Self {
        let closure = Closure::<dyn Fn(web_sys::Event)>::new(move |_: web_sys::Event| {
            if let Some(bar) = bar.cast::<HtmlElement>() {
                clear_focus(&bar);
            }
        });
        let added = web_sys::window()
            .and_then(|w| w.document())
            .is_some_and(|d| {
                d.add_event_listener_with_callback_and_bool(
                    "focusin",
                    closure.as_ref().unchecked_ref(),
                    true,
                )
                .is_ok()
            });
        Self {
            closure: added.then_some(closure),
        }
    }
}

impl Drop for FocusClearance {
    fn drop(&mut self) {
        if let (Some(document), Some(closure)) =
            (web_sys::window().and_then(|w| w.document()), &self.closure)
        {
            _ = document.remove_event_listener_with_callback_and_bool(
                "focusin",
                closure.as_ref().unchecked_ref(),
                true,
            );
        }
    }
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
    /// Stays at the bottom of its scroll container, raised over the form,
    /// and while dirty scrolls the focused field out from under itself.
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

impl Default for SaveBarProps {
    fn default() -> Self {
        Self {
            dirty: Default::default(),
            busy: Default::default(),
            disabled: Default::default(),
            label: Default::default(),
            save_label: Default::default(),
            discard_label: Default::default(),
            status: Default::default(),
            onsave: Default::default(),
            ondiscard: Default::default(),
            submit: Default::default(),
            guard: ::yew::html::IntoPropValue::into_prop_value(true),
            sticky: Default::default(),
            class: Default::default(),
            testid: Default::default(),
            children: Default::default(),
        }
    }
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
    let bar = use_node_ref();
    use_effect_with(
        (props.sticky && props.dirty, bar.clone()),
        |(watch, bar)| {
            let clearance = watch.then(|| {
                if let Some(element) = bar.cast::<HtmlElement>() {
                    clear_focus(&element);
                }
                FocusClearance::new(bar.clone())
            });
            move || drop(clearance)
        },
    );

    with_testid(
        html! {
            <div ref={bar} class={classes!(save_bar(!props.dirty, props.sticky), props.class.clone())}>
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
    fn overlap_clears_the_focused_field() {
        let bar = (100.0, 150.0);
        // Clear of the bar, or beyond it.
        assert_eq!(overlap((0.0, 80.0), 40.0, bar, 8.0, 0.0), None);
        assert_eq!(overlap((150.0, 190.0), 160.0, bar, 8.0, 0.0), None);
        // Under the bar, or too near it: scrolled to end `gap` above it.
        assert_eq!(overlap((60.0, 120.0), 80.0, bar, 8.0, 0.0), Some(28.0));
        assert_eq!(overlap((110.0, 140.0), 110.0, bar, 8.0, 0.0), Some(48.0));
        assert_eq!(overlap((50.0, 95.0), 50.0, bar, 8.0, 0.0), Some(3.0));
        // Taller than the room: the focused element's top stays in view.
        assert_eq!(overlap((10.0, 300.0), 20.0, bar, 8.0, 0.0), Some(20.0));
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
