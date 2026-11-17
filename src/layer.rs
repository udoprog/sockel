//! The open overlays, innermost last, so Escape and a press outside close
//! only the innermost; and the page's scroll lock while a dialog is open.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, Default)]
pub(crate) struct Layers {
    next: u32,
    open: Vec<u32>,
}

impl Layers {
    pub(crate) fn push(&mut self) -> u32 {
        self.next += 1;
        self.open.push(self.next);
        self.next
    }

    pub(crate) fn remove(&mut self, id: u32) {
        self.open.retain(|&o| o != id);
    }

    pub(crate) fn is_top(&self, id: u32) -> bool {
        self.open.last() == Some(&id)
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
    pub(crate) fn open(stack: &Stack) -> Self {
        Self {
            id: stack.borrow_mut().push(),
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
/// page had a scrollbar, so its room stays and the page does not move.
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
        }
    }
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
        let (dialog, popover) = (layers.push(), layers.push());
        assert!(layers.is_top(popover) && !layers.is_top(dialog));
        layers.remove(popover);
        assert!(layers.is_top(dialog));
        let confirm = layers.push();
        layers.remove(dialog);
        assert!(layers.is_top(confirm));
        layers.remove(confirm);
        assert!(!layers.is_top(confirm));
    }

    #[test]
    fn a_layer_leaves_its_own_stack_when_dropped() {
        let (a, b) = (Stack::default(), Stack::default());
        let outer = Layer::open(&a);
        let other = Layer::open(&b);
        assert!(outer.is_top() && other.is_top(), "stacks are apart");
        let inner = Layer::open(&a);
        assert!(!outer.is_top() && other.is_top());
        drop(inner);
        assert!(outer.is_top());
        assert!(a.borrow().open.len() == 1 && b.borrow().open.len() == 1);
    }
}
