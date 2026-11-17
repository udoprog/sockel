//! The open overlays, innermost last, so Escape and a press outside close
//! only the innermost; and the page's scroll lock while a dialog is open.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, Default)]
pub(crate) struct Layers {
    next: u32,
    open: Vec<(u32, bool)>,
}

impl Layers {
    /// A transient layer (a tip) is topmost only while nothing else opened
    /// after it, and never keeps the layers below it from being topmost.
    pub(crate) fn push(&mut self, transient: bool) -> u32 {
        self.next += 1;
        self.open.push((self.next, transient));
        self.next
    }

    pub(crate) fn remove(&mut self, id: u32) {
        self.open.retain(|&(o, _)| o != id);
    }

    /// Whether anything but a tip is open.
    pub(crate) fn any_open(&self) -> bool {
        self.open.iter().any(|&(_, transient)| !transient)
    }

    pub(crate) fn is_top(&self, id: u32) -> bool {
        match self.open.iter().find(|&&(o, _)| o == id) {
            Some(&(_, true)) => self.open.last().is_some_and(|&(o, _)| o == id),
            Some(_) => self
                .open
                .iter()
                .rev()
                .find(|&&(_, transient)| !transient)
                .is_some_and(|&(o, _)| o == id),
            None => false,
        }
    }
}

/// One app's stack of open overlays.
pub(crate) type Stack = Rc<RefCell<Layers>>;

thread_local! {
    // Per document, not per stack: what it locks is the one root.
    static LOCKS: Cell<usize> = const { Cell::new(0) };
}

/// An overlay opening: its place on its stack, taken off when dropped.
pub(crate) struct Layer {
    stack: Stack,
    id: u32,
}

impl Layer {
    pub(crate) fn open(stack: &Stack, transient: bool) -> Self {
        Self {
            id: stack.borrow_mut().push(transient),
            stack: stack.clone(),
        }
    }

    /// Whether nothing opened after it on its stack is still open.
    pub(crate) fn is_top(&self) -> bool {
        self.stack.borrow().is_top(self.id)
    }
}

impl Drop for Layer {
    fn drop(&mut self) {
        self.stack.borrow_mut().remove(self.id);
    }
}

/// The page's scroll locked while held: the first lock sets
/// `.plinth-scroll-locked` on the root, with `.plinth-gutter-kept` when the
/// page had a scrollbar, so its room stays and the page does not move; the
/// width of the room kept (by it or the page's own `scrollbar-gutter`) is
/// `--scroll-gutter` on the root, for layers reaching over it.
pub(crate) struct ScrollLock;

impl ScrollLock {
    pub(crate) fn take() -> Self {
        if LOCKS.replace(LOCKS.get() + 1) == 0
            && let Some(root) = root()
        {
            let classes = root.class_list();
            if root.scroll_height() > root.client_height() {
                _ = classes.add_1(crate::class::GUTTER_KEPT);
            }
            _ = classes.add_1(crate::class::SCROLL_LOCKED);
            set_gutter(&root, Some(gutter(&root)));
        }
        Self
    }
}

impl Drop for ScrollLock {
    fn drop(&mut self) {
        let left = LOCKS.get().saturating_sub(1);
        LOCKS.set(left);
        if left == 0
            && let Some(root) = root()
        {
            _ = root
                .class_list()
                .remove_2(crate::class::SCROLL_LOCKED, crate::class::GUTTER_KEPT);
            set_gutter(&root, None);
        }
    }
}

/// The room beside the page kept for its scrollbar: the window's width
/// beside the root's box (its `clientWidth` misreports it when zoomed).
fn gutter(root: &web_sys::Element) -> f64 {
    let window = web_sys::window().and_then(|w| w.inner_width().ok());
    let width = window.and_then(|w| w.as_f64()).unwrap_or_default();
    (width - root.get_bounding_client_rect().width()).max(0.0)
}

fn set_gutter(root: &web_sys::Element, width: Option<f64>) {
    use wasm_bindgen::JsCast;

    let Some(style) = root.dyn_ref::<web_sys::HtmlElement>().map(|e| e.style()) else {
        return;
    };
    _ = match width {
        Some(width) => style.set_property("--scroll-gutter", &format!("{width}px")),
        None => style.remove_property("--scroll-gutter").map(drop),
    };
}

fn root() -> Option<web_sys::Element> {
    web_sys::window()?.document()?.document_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_innermost_is_the_last_opened_still_open() {
        let mut layers = Layers::default();
        let (dialog, popover) = (layers.push(false), layers.push(false));
        assert!(layers.is_top(popover) && !layers.is_top(dialog));
        layers.remove(popover);
        assert!(layers.is_top(dialog));
        let confirm = layers.push(false);
        layers.remove(dialog);
        assert!(layers.is_top(confirm));
        layers.remove(confirm);
        assert!(!layers.is_top(confirm));
    }

    #[test]
    fn any_open_counts_all_but_tips() {
        let mut layers = Layers::default();
        assert!(!layers.any_open());
        let tip = layers.push(true);
        assert!(!layers.any_open());
        let panel = layers.push(false);
        assert!(layers.any_open());
        layers.remove(panel);
        layers.remove(tip);
        assert!(!layers.any_open());
    }

    #[test]
    fn a_tip_does_not_take_the_top_from_what_it_covers() {
        let mut layers = Layers::default();
        let dialog = layers.push(false);
        let tip = layers.push(true);
        assert!(layers.is_top(dialog) && layers.is_top(tip));
        let menu = layers.push(false);
        assert!(layers.is_top(menu) && !layers.is_top(dialog) && !layers.is_top(tip));
        layers.remove(menu);
        layers.remove(tip);
        assert!(layers.is_top(dialog));
    }

    #[test]
    fn a_layer_leaves_its_own_stack_when_dropped() {
        let (a, b) = (Stack::default(), Stack::default());
        let outer = Layer::open(&a, false);
        let other = Layer::open(&b, false);
        assert!(outer.is_top() && other.is_top(), "stacks are apart");
        let inner = Layer::open(&a, false);
        assert!(!outer.is_top() && other.is_top());
        drop(inner);
        assert!(outer.is_top());
        assert!(a.borrow().open.len() == 1 && b.borrow().open.len() == 1);
    }
}
