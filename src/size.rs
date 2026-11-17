//! An element's width as a hook, and the one-per-frame gate it and
//! [`Seam`](crate::Seam) report through.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, ResizeObserver, ResizeObserverEntry, ResizeObserverSize};
use yew::prelude::*;

/// Readings coalesced to one per animation frame, each reported only when it
/// differs from the last reported.
#[derive(Debug, Default)]
pub(crate) struct Gate {
    pending: Option<f64>,
    last: Option<f64>,
    asked: bool,
}

impl Gate {
    /// Keeps `value` for the next frame; true when a frame must be asked for.
    pub(crate) fn offer(&mut self, value: f64) -> bool {
        self.pending = Some(value);
        !std::mem::replace(&mut self.asked, true)
    }

    /// At the frame: the latest reading, unless it is the one last reported.
    pub(crate) fn frame(&mut self) -> Option<f64> {
        self.asked = false;
        let value = self.pending.take()?;
        (self.last != Some(value)).then(|| {
            self.last = Some(value);
            value
        })
    }

    /// Forgets what was last reported, so the next reading is reported.
    pub(crate) fn reset(&mut self) {
        self.last = None;
    }
}

/// A gate whose readings reach `report` at the next animation frame;
/// dropping it cancels a frame still asked for.
pub(crate) struct Framed {
    gate: Rc<RefCell<Gate>>,
    frame: Rc<RefCell<Option<i32>>>,
    callback: Closure<dyn FnMut()>,
}

impl Framed {
    pub(crate) fn new(report: impl Fn(f64) + 'static) -> Self {
        let gate = Rc::new(RefCell::new(Gate::default()));
        let frame = Rc::new(RefCell::new(None));
        let callback = {
            let (gate, frame) = (gate.clone(), frame.clone());
            Closure::<dyn FnMut()>::new(move || {
                frame.borrow_mut().take();
                let value = gate.borrow_mut().frame();
                if let Some(value) = value {
                    report(value);
                }
            })
        };
        Self {
            gate,
            frame,
            callback,
        }
    }

    pub(crate) fn offer(&self, value: f64) {
        if !self.gate.borrow_mut().offer(value) {
            return;
        }
        let id = web_sys::window().and_then(|w| {
            w.request_animation_frame(self.callback.as_ref().unchecked_ref())
                .ok()
        });
        *self.frame.borrow_mut() = id;
    }

    pub(crate) fn reset(&self) {
        self.gate.borrow_mut().reset();
    }
}

impl Drop for Framed {
    fn drop(&mut self) {
        if let (Some(id), Some(window)) = (self.frame.borrow_mut().take(), web_sys::window()) {
            _ = window.cancel_animation_frame(id);
        }
    }
}

/// An entry's border box width: padding and border included.
fn border_width(entry: &ResizeObserverEntry) -> f64 {
    entry
        .border_box_size()
        .iter()
        .next()
        .map(|size| size.unchecked_into::<ResizeObserverSize>().inline_size())
        .unwrap_or_else(|| entry.content_rect().width())
}

/// How wide `node`'s element is (its border box, in CSS pixels): `0.0` until
/// the first measurement, then at most one new value per animation frame, and
/// only when it changed, so a component laid out against it renders once per
/// frame of a resize.
#[hook]
pub fn use_width(node: &NodeRef) -> f64 {
    let width = use_state_eq(|| 0.0f64);

    {
        let width = width.clone();
        use_effect_with(node.clone(), move |node| {
            let framed = Framed::new(move |w| width.set(w));
            let observed = Rc::new(framed);
            let callback = {
                let observed = observed.clone();
                Closure::<dyn FnMut(web_sys::js_sys::Array)>::new(
                    move |entries: web_sys::js_sys::Array| {
                        if let Some(entry) = entries.iter().next_back() {
                            observed.offer(border_width(&entry.unchecked_into()));
                        }
                    },
                )
            };
            let observer = ResizeObserver::new(callback.as_ref().unchecked_ref()).ok();
            if let (Some(observer), Some(element)) = (&observer, node.cast::<Element>()) {
                observer.observe(&element);
            }
            move || {
                if let Some(observer) = observer {
                    observer.disconnect();
                }
                drop(callback);
                drop(observed);
            }
        });
    }

    *width
}

#[cfg(test)]
mod tests {
    use super::Gate;

    #[test]
    fn a_burst_of_readings_asks_for_one_frame_and_reports_the_last() {
        let mut gate = Gate::default();
        assert!(gate.offer(100.0));
        assert!(!gate.offer(120.0));
        assert!(!gate.offer(140.0));
        assert_eq!(gate.frame(), Some(140.0));
        assert_eq!(gate.frame(), None);
    }

    #[test]
    fn an_unchanged_reading_is_not_reported_again() {
        let mut gate = Gate::default();
        gate.offer(100.0);
        assert_eq!(gate.frame(), Some(100.0));
        assert!(gate.offer(100.0));
        assert_eq!(gate.frame(), None);
        gate.offer(101.0);
        assert_eq!(gate.frame(), Some(101.0));
    }

    #[test]
    fn a_reset_reports_the_same_reading_again() {
        let mut gate = Gate::default();
        gate.offer(0.5);
        assert_eq!(gate.frame(), Some(0.5));
        gate.reset();
        gate.offer(0.5);
        assert_eq!(gate.frame(), Some(0.5));
    }
}
