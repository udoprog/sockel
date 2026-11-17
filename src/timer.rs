//! Timeouts that clean up after themselves.

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

/// A timeout that is cleared and its closure freed when dropped.
///
/// Dropping a timer from inside its own callback is not allowed.
pub(crate) struct Timer {
    id: i32,
    _callback: Closure<dyn FnMut()>,
}

impl Timer {
    /// Runs `f` once after `ms`, or never if the timer is dropped first.
    pub(crate) fn start(ms: u32, f: impl FnOnce() + 'static) -> Option<Self> {
        let window = web_sys::window()?;
        let mut f = Some(f);
        let callback = Closure::<dyn FnMut()>::new(move || {
            if let Some(f) = f.take() {
                f();
            }
        });
        let id = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                i32::try_from(ms).unwrap_or(i32::MAX),
            )
            .ok()?;
        Some(Self {
            id,
            _callback: callback,
        })
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(self.id);
        }
    }
}

/// The newest value handed to [`Latest::set`], shared with the timer's closure.
pub(crate) struct Latest<T>(Rc<RefCell<T>>);

impl<T> Clone for Latest<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone> Latest<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub(crate) fn set(&self, value: T) {
        *self.0.borrow_mut() = value;
    }

    pub(crate) fn get(&self) -> T {
        self.0.borrow().clone()
    }
}

/// A hook that tells `expired` the `generation` after `ms` while `armed`,
/// restarted whenever the generation changes and cancelled when it goes.
/// The newest `expired` is the one told.
#[hook]
pub(crate) fn use_timeout(armed: bool, generation: u32, ms: u32, expired: Callback<u32>) {
    let latest = use_memo((), |()| Latest::new(Callback::<u32>::noop()));
    latest.set(expired);
    use_effect_with((armed, generation, ms), move |&(armed, generation, ms)| {
        let latest = (*latest).clone();
        let timer = armed
            .then(|| Timer::start(ms, move || latest.get().emit(generation)))
            .flatten();
        move || drop(timer)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_value_wins_through_every_clone() {
        let latest = Latest::new(1);
        let seen_by_timer = latest.clone();
        latest.set(2);
        latest.set(3);
        assert_eq!(seen_by_timer.get(), 3);
    }

    #[test]
    fn latest_callback_is_the_one_emitted() {
        let hits = Rc::new(RefCell::new(Vec::new()));
        let make = |tag: &'static str| {
            let hits = hits.clone();
            Callback::from(move |g: u32| hits.borrow_mut().push((tag, g)))
        };
        let latest = Latest::new(make("old"));
        let timer_side = latest.clone();
        latest.set(make("new"));
        timer_side.get().emit(7);
        assert_eq!(*hits.borrow(), [("new", 7)]);
    }
}
