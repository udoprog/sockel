//! Listbox: the keyboard side of a list of options that never take focus
//! (`style/_listbox.scss`).
//!
//! What has focus (a [`crate::Select`]'s face, a search field) names the
//! active option with `aria-activedescendant`. Options are known by their
//! index in the caller's list; the caller says which can be moved to, in
//! order (`pickable`: those shown and not disabled), renders each with
//! [`option_id`] and [`listbox_option`], and scrolls the active one into
//! view when [`Listbox::take_scroll`] says it moved by keyboard.

use yew::prelude::*;

use crate::{class, typeahead};

/// How far PageUp and PageDown move.
pub const LISTBOX_PAGE: usize = 10;

/// How long a pause ends a type-ahead search, in milliseconds.
const TYPEAHEAD_MS: f64 = 700.0;

/// The id of option `index` of the list whose ids start with `base`.
pub fn option_id(base: &str, index: usize) -> String {
    format!("{base}-option-{index}")
}

/// The open list's classes: `listbox`, and `placed` once placed.
pub fn listbox(placed: bool) -> Classes {
    classes!(class::LISTBOX, placed.then_some(class::PLACED))
}

/// An option's classes: `listbox-option`, `active` under the keyboard or
/// pointer. Selected and disabled are its `aria-selected` and
/// `aria-disabled`.
pub fn listbox_option(active: bool) -> Classes {
    classes!(class::LISTBOX_OPTION, active.then_some(class::ACTIVE))
}

/// What a key did to an open list, as [`Listbox::key`] reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKey {
    /// The active option moved (or the key was taken and nothing moved).
    Moved,
    /// Pick this option.
    Pick(usize),
    /// Not a key for the list.
    Ignored,
}

/// The active option of a list and the letters typed to find one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Listbox {
    active: Option<usize>,
    typed: String,
    typed_at: f64,
    scroll: bool,
}

impl Listbox {
    pub fn active(&self) -> Option<usize> {
        self.active
    }

    /// Make `index` active, as a pointer over it does (no scrolling).
    /// Whether it changed.
    pub fn hover(&mut self, index: usize) -> bool {
        let changed = self.active != Some(index);
        self.active = Some(index);
        changed
    }

    /// Make `index` active and scroll to it.
    pub fn set(&mut self, index: Option<usize>) {
        self.active = index;
        self.scroll = true;
    }

    /// Move `step` places among `pickable`, stopping at either end; from
    /// none, to the first (or the last, stepping back).
    pub fn step(&mut self, pickable: &[usize], step: isize) {
        let Some(last) = pickable.len().checked_sub(1) else {
            self.active = None;
            return;
        };
        let next = match self.position(pickable) {
            None if step < 0 => last,
            None => 0,
            Some(at) => at.saturating_add_signed(step).min(last),
        };
        self.set(Some(pickable[next]));
    }

    /// Make the first (or `last`) of `pickable` active.
    pub fn go_to_end(&mut self, pickable: &[usize], last: bool) {
        self.active = None;
        self.step(pickable, if last { -1 } else { 1 });
    }

    /// Keep the active option one of `pickable`, else the first of them.
    pub fn settle(&mut self, pickable: &[usize]) {
        if !self.active.is_some_and(|a| pickable.contains(&a)) {
            self.go_to_end(pickable, false);
        }
    }

    /// Type `c` at `now` (ms) and move to the next of `pickable` whose
    /// label (`labels[i]`, by index in the whole list) starts with what was
    /// typed, as a [`crate::Menu`]'s type-ahead does.
    pub fn typeahead(&mut self, pickable: &[usize], labels: &[&str], c: char, now: f64) {
        if now - self.typed_at > TYPEAHEAD_MS {
            self.typed.clear();
        }
        self.typed_at = now;
        self.typed.extend(c.to_lowercase());
        let names: Vec<String> = pickable
            .iter()
            .map(|&i| labels.get(i).copied().unwrap_or_default().to_owned())
            .collect();
        if let Some(found) = typeahead(&names, self.position(pickable), &self.typed) {
            self.set(Some(pickable[found]));
        }
    }

    /// Read `key` (`alt` held) on an open list. Typing letters is the
    /// caller's, by [`Listbox::typeahead`] or a filter.
    pub fn key(&mut self, key: &str, alt: bool, pickable: &[usize]) -> ListKey {
        let page = LISTBOX_PAGE as isize;
        match key {
            "ArrowDown" => self.step(pickable, 1),
            "ArrowUp" if alt => return self.active.map_or(ListKey::Moved, ListKey::Pick),
            "ArrowUp" => self.step(pickable, -1),
            "PageDown" => self.step(pickable, page),
            "PageUp" => self.step(pickable, -page),
            "Home" | "End" => self.go_to_end(pickable, key == "End"),
            "Enter" | " " => return self.active.map_or(ListKey::Moved, ListKey::Pick),
            _ => return ListKey::Ignored,
        }
        ListKey::Moved
    }

    /// Whether the active option moved by keyboard since the last call, so
    /// it should be scrolled into view.
    pub fn take_scroll(&mut self) -> bool {
        std::mem::take(&mut self.scroll)
    }

    fn position(&self, pickable: &[usize]) -> Option<usize> {
        self.active
            .and_then(|active| pickable.iter().position(|&i| i == active))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Options 0 to 5 with 2 disabled.
    const PICKABLE: [usize; 5] = [0, 1, 3, 4, 5];

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn active_and_placed_are_modifiers() {
        assert_eq!(names(listbox(false)), [class::LISTBOX]);
        assert_eq!(names(listbox(true)), [class::LISTBOX, class::PLACED]);
        assert_eq!(names(listbox_option(false)), [class::LISTBOX_OPTION]);
        assert_eq!(
            names(listbox_option(true)),
            [class::LISTBOX_OPTION, class::ACTIVE]
        );
        assert_eq!(option_id("plinth-select-3", 2), "plinth-select-3-option-2");
    }

    #[test]
    fn steps_over_what_cannot_be_picked_and_stops_at_the_ends() {
        let mut list = Listbox::default();
        list.step(&PICKABLE, 1);
        assert_eq!(list.active(), Some(0));
        list.step(&PICKABLE, -1);
        assert_eq!(list.active(), Some(0));
        list.step(&PICKABLE, 2);
        assert_eq!(list.active(), Some(3));
        list.step(&PICKABLE, 10);
        assert_eq!(list.active(), Some(5));
        assert!(list.take_scroll());
        assert!(!list.take_scroll());
        list.step(&PICKABLE, -1);
        assert_eq!(list.active(), Some(4));
        let mut none = Listbox::default();
        none.step(&PICKABLE, -1);
        assert_eq!(none.active(), Some(5));
    }

    #[test]
    fn keys_move_and_pick() {
        let mut list = Listbox::default();
        assert_eq!(list.key("End", false, &PICKABLE), ListKey::Moved);
        assert_eq!(list.active(), Some(5));
        assert_eq!(list.key("Home", false, &PICKABLE), ListKey::Moved);
        assert_eq!(list.key("ArrowDown", false, &PICKABLE), ListKey::Moved);
        assert_eq!(list.key("ArrowDown", false, &PICKABLE), ListKey::Moved);
        assert_eq!(list.active(), Some(3));
        assert_eq!(list.key("PageUp", false, &PICKABLE), ListKey::Moved);
        assert_eq!(list.active(), Some(0));
        assert_eq!(list.key("Enter", false, &PICKABLE), ListKey::Pick(0));
        assert_eq!(list.key(" ", false, &PICKABLE), ListKey::Pick(0));
        assert_eq!(list.key("ArrowUp", true, &PICKABLE), ListKey::Pick(0));
        assert_eq!(list.key("x", false, &PICKABLE), ListKey::Ignored);
        assert_eq!(Listbox::default().key("Enter", false, &[]), ListKey::Moved);
    }

    #[test]
    fn typeahead_skips_disabled_options_and_resets_after_a_pause() {
        let labels = ["Apple", "Banana", "Blueberry", "Cherry", "Bramble", "Date"];
        let mut list = Listbox::default();
        list.typeahead(&PICKABLE, &labels, 'b', 0.0);
        assert_eq!(list.active(), Some(1));
        // Blueberry (2) is disabled: the next B is Bramble.
        list.typeahead(&PICKABLE, &labels, 'b', 100.0);
        assert_eq!(list.active(), Some(4));
        list.typeahead(&PICKABLE, &labels, 'd', 2000.0);
        assert_eq!(list.active(), Some(5));
        list.typeahead(&PICKABLE, &labels, 'z', 3000.0);
        assert_eq!(list.active(), Some(5));
    }

    #[test]
    fn settles_on_the_first_when_the_active_is_filtered_out() {
        let mut list = Listbox::default();
        list.set(Some(4));
        list.settle(&[1, 4]);
        assert_eq!(list.active(), Some(4));
        list.settle(&[1, 3]);
        assert_eq!(list.active(), Some(1));
        list.settle(&[]);
        assert_eq!(list.active(), None);
    }
}
