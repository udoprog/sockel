//! A timeout as a hook.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

/// A hook that tells `expired` the `generation` after `ms` while `armed`,
/// restarted whenever the generation changes and cancelled when it goes.
#[hook]
pub(crate) fn use_timeout(armed: bool, generation: u32, ms: u32, expired: Callback<u32>) {
    use_effect_with((armed, generation, ms), move |&(armed, generation, ms)| {
        let window = web_sys::window();
        let mut pending = None;
        if let (true, Some(window)) = (armed, window.as_ref()) {
            let closure = Closure::once_into_js(move || expired.emit(generation));
            let id = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.unchecked_ref(),
                    ms as i32,
                )
                .ok();
            pending = id.map(|id| (window.clone(), id));
        }
        move || {
            if let Some((window, id)) = pending {
                window.clear_timeout_with_handle(id);
            }
        }
    });
}
