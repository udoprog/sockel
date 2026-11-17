//! The keyboard every composite widget shares: moving among options and
//! type-ahead. What happens at the ends follows the WAI-ARIA Authoring
//! Practices for each role:
//!
//! - A roving group (`Choice`'s radios, `Tabs`) and a `Menu` wrap: past the
//!   last comes the first. A radio group takes all four arrows; a
//!   horizontal tab list takes Left and Right only, so Up and Down still
//!   scroll the page; a menu takes Up and Down.
//! - A listbox (`Select`, `ColorSelect`, `SearchSelect`, `Picker`,
//!   `FilterPicker`) stops at either end, and a grid's step to a row that
//!   is not there stays put.
//! - Type-ahead ([`Typeahead`]) finds the next option starting with what
//!   was typed. Space types while a search is under way, and picks
//!   otherwise.

use web_sys::wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, KeyboardEvent, Node};

/// How long a pause ends a type-ahead search, in milliseconds.
const TYPEAHEAD_MS: f64 = 700.0;

/// Which arrows move a group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arrows {
    Horizontal,
    Vertical,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Move {
    Next,
    Previous,
    First,
    Last,
}

fn movement(key: &str, arrows: Arrows) -> Option<Move> {
    use Arrows::{Both, Horizontal, Vertical};
    match (key, arrows) {
        ("ArrowRight", Horizontal | Both) | ("ArrowDown", Vertical | Both) => Some(Move::Next),
        ("ArrowLeft", Horizontal | Both) | ("ArrowUp", Vertical | Both) => Some(Move::Previous),
        ("Home", _) => Some(Move::First),
        ("End", _) => Some(Move::Last),
        _ => None,
    }
}

/// The option holding the tab stop: the selected one if enabled, else the
/// first that is enabled.
pub(crate) fn stop(disabled: &[bool], selected: Option<usize>) -> Option<usize> {
    selected
        .filter(|i| !disabled[*i])
        .or_else(|| disabled.iter().position(|d| !d))
}

/// The enabled option `key` moves to from `from`, wrapping; from none, the
/// first (or the last, going back). `None` for a key that does not move.
pub(crate) fn step(
    disabled: &[bool],
    from: Option<usize>,
    key: &str,
    arrows: Arrows,
) -> Option<usize> {
    let len = disabled.len();
    let moved = movement(key, arrows)?;
    if len == 0 {
        return None;
    }
    let enabled = |i: &usize| !disabled[*i];
    match moved {
        Move::First => (0..len).find(enabled),
        Move::Last => (0..len).rev().find(enabled),
        Move::Next => {
            let start = from.map_or(0, |f| f + 1);
            (0..len).map(|n| (start + n) % len).find(enabled)
        }
        Move::Previous => {
            let start = from.map_or(len - 1, |f| f + len - 1);
            (0..len).map(|n| (start + len - n) % len).find(enabled)
        }
    }
}

/// Where `key` moves focus among `count` items from `at`: down and up
/// wrap, Home and End go to the ends; `None` for any other key.
pub fn menu_step(key: &str, at: Option<usize>, count: usize) -> Option<usize> {
    step(&vec![false; count], at, key, Arrows::Vertical)
}

/// A listbox's step of `by` places among `len` from `at`, stopping at
/// either end; from none, the first (or the last, stepping back).
pub(crate) fn clamped(at: Option<usize>, len: usize, by: isize) -> Option<usize> {
    let last = len.checked_sub(1)?;
    Some(match at {
        None if by < 0 => last,
        None => 0,
        Some(at) => at.saturating_add_signed(by).min(last),
    })
}

/// A grid's step of `by` places among `len` from `at`: one that would
/// leave the grid stays put.
pub(crate) fn within(at: Option<usize>, len: usize, by: isize) -> Option<usize> {
    match at {
        Some(at) => Some(
            at.checked_add_signed(by)
                .filter(|&to| to < len)
                .unwrap_or(at),
        ),
        None => clamped(None, len, by),
    }
}

/// The item whose label starts with `query` (ignoring case), searching on
/// from the one after `at` and wrapping; a query of one repeated letter
/// cycles through the items starting with it.
pub fn typeahead(labels: &[String], at: Option<usize>, query: &str) -> Option<usize> {
    let query = query.to_lowercase();
    let mut chars = query.chars();
    let first = chars.next()?;
    let query = if chars.all(|c| c == first) {
        first.to_string()
    } else {
        query
    };
    // A longer query may still match the current item.
    let start = match at {
        Some(at) if query.chars().count() > 1 => at,
        Some(at) => at + 1,
        None => 0,
    };
    let count = labels.len();
    (0..count)
        .map(|i| (start + i) % count)
        .find(|&i| labels[i].trim().to_lowercase().starts_with(&query))
}

/// What has been typed to find an option, and when.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Typeahead {
    typed: String,
    at: f64,
}

impl Typeahead {
    /// Whether a search is under way at `now`.
    pub(crate) fn typing(&self, now: f64) -> bool {
        !self.typed.is_empty() && now - self.at <= TYPEAHEAD_MS
    }

    /// Type `c` at `now`: the query so far, or `None` for a Space that
    /// starts no search.
    pub(crate) fn push(&mut self, c: char, now: f64) -> Option<&str> {
        if !self.typing(now) {
            if c == ' ' {
                return None;
            }
            self.typed.clear();
        }
        self.at = now;
        self.typed.extend(c.to_lowercase());
        Some(&self.typed)
    }
}

/// A key that types a character, Space included, rather than a command
/// (none with Ctrl, Meta or Alt held): for a filter or type-ahead.
pub fn printable(e: &KeyboardEvent) -> Option<char> {
    if e.ctrl_key() || e.meta_key() || e.alt_key() {
        return None;
    }
    let key = e.key();
    let mut chars = key.chars();
    let c = chars.next()?;
    chars.next().is_none().then_some(c)
}

/// A roving group's keydown: from the focused option of `role` (else the
/// tab stop) to the one the key names, focused. Its index, for the owner
/// to select.
pub(crate) fn keydown(
    e: &KeyboardEvent,
    role: &str,
    disabled: &[bool],
    stop: Option<usize>,
    arrows: Arrows,
) -> Option<usize> {
    let options = e
        .current_target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .and_then(|group| group.query_selector_all(&format!("[role={role}]")).ok());
    let target = e.target().and_then(|t| t.dyn_into::<Node>().ok());
    let focused = options.as_ref().and_then(|list| {
        (0..list.length())
            .find(|&i| list.item(i).is_some_and(|n| Some(&n) == target.as_ref()))
            .and_then(|i| usize::try_from(i).ok())
    });
    let next = step(disabled, focused.or(stop), &e.key(), arrows)?;
    e.prevent_default();
    let option = options
        .and_then(|list| list.item(u32::try_from(next).ok()?))
        .and_then(|node| node.dyn_into::<HtmlElement>().ok());
    if let Some(option) = option {
        _ = option.focus();
    }
    Some(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_wrap_over_disabled_options() {
        let off = [false, true, false, false];
        let at = |from, key| step(&off, from, key, Arrows::Both);
        assert_eq!(at(Some(0), "ArrowRight"), Some(2));
        assert_eq!(at(Some(0), "ArrowDown"), Some(2));
        assert_eq!(at(Some(3), "ArrowRight"), Some(0));
        assert_eq!(at(Some(2), "ArrowLeft"), Some(0));
        assert_eq!(at(Some(0), "ArrowUp"), Some(3));
        assert_eq!(at(Some(2), "Home"), Some(0));
        assert_eq!(at(Some(0), "End"), Some(3));
        assert_eq!(at(None, "ArrowRight"), Some(0));
        assert_eq!(at(None, "ArrowLeft"), Some(3));
        assert_eq!(at(Some(0), "Tab"), None);
        assert_eq!(
            step(&[true, true], Some(0), "ArrowRight", Arrows::Both),
            None
        );
        assert_eq!(step(&[], None, "Home", Arrows::Both), None);
    }

    #[test]
    fn a_horizontal_group_leaves_up_and_down_alone() {
        let off = [false; 3];
        let at = |key| step(&off, Some(1), key, Arrows::Horizontal);
        assert_eq!(at("ArrowRight"), Some(2));
        assert_eq!(at("ArrowLeft"), Some(0));
        assert_eq!(at("ArrowUp"), None);
        assert_eq!(at("ArrowDown"), None);
        assert_eq!(step(&off, Some(1), "ArrowRight", Arrows::Vertical), None);
    }

    #[test]
    fn arrows_wrap_and_home_and_end_go_to_the_ends() {
        assert_eq!(menu_step("ArrowDown", None, 3), Some(0));
        assert_eq!(menu_step("ArrowDown", Some(2), 3), Some(0));
        assert_eq!(menu_step("ArrowUp", None, 3), Some(2));
        assert_eq!(menu_step("ArrowUp", Some(0), 3), Some(2));
        assert_eq!(menu_step("Home", Some(2), 3), Some(0));
        assert_eq!(menu_step("End", Some(0), 3), Some(2));
        assert_eq!(menu_step("ArrowRight", Some(0), 3), None);
        assert_eq!(menu_step("Enter", Some(0), 3), None);
        assert_eq!(menu_step("ArrowDown", None, 0), None);
    }

    #[test]
    fn the_stop_is_the_selected_option_if_enabled() {
        assert_eq!(stop(&[false, false], Some(1)), Some(1));
        assert_eq!(stop(&[true, false, true], Some(2)), Some(1));
        assert_eq!(stop(&[true, true], None), None);
    }

    #[test]
    fn lists_stop_at_the_ends_and_grids_stay_inside() {
        assert_eq!(clamped(None, 5, 1), Some(0));
        assert_eq!(clamped(None, 5, -1), Some(4));
        assert_eq!(clamped(Some(0), 5, -1), Some(0));
        assert_eq!(clamped(Some(3), 5, 10), Some(4));
        assert_eq!(clamped(Some(3), 0, 1), None);
        // A grid of 8 in rows of 3: up from the top row and down from the
        // last stay put; down into the short row's gap does too.
        assert_eq!(within(Some(1), 8, -3), Some(1));
        assert_eq!(within(Some(4), 8, -3), Some(1));
        assert_eq!(within(Some(4), 8, 3), Some(7));
        assert_eq!(within(Some(5), 8, 3), Some(5));
        assert_eq!(within(Some(7), 8, 1), Some(7));
        assert_eq!(within(None, 8, -3), Some(7));
    }

    #[test]
    fn typeahead_finds_the_next_item_by_its_start() {
        let labels: Vec<String> = ["Copy", "Cut", "Delete", "Paste", "copy link"]
            .map(String::from)
            .to_vec();
        assert_eq!(typeahead(&labels, None, "d"), Some(2));
        assert_eq!(typeahead(&labels, Some(0), "c"), Some(1));
        assert_eq!(typeahead(&labels, Some(1), "cc"), Some(4));
        assert_eq!(typeahead(&labels, Some(4), "c"), Some(0));
        assert_eq!(typeahead(&labels, Some(0), "cu"), Some(1));
        assert_eq!(typeahead(&labels, Some(4), "copy"), Some(4));
        assert_eq!(typeahead(&labels, Some(0), "copy "), Some(4));
        assert_eq!(typeahead(&labels, None, "x"), None);
        assert_eq!(typeahead(&labels, None, ""), None);
    }

    #[test]
    fn space_types_only_while_a_search_is_under_way() {
        let mut typed = Typeahead::default();
        assert!(!typed.typing(0.0));
        assert_eq!(typed.push(' ', 0.0), None);
        assert_eq!(typed.push('N', 10.0), Some("n"));
        assert!(typed.typing(100.0));
        assert_eq!(typed.push(' ', 100.0), Some("n "));
        assert_eq!(typed.push('y', 200.0), Some("n y"));
        assert!(!typed.typing(1000.0));
        assert_eq!(typed.push(' ', 1000.0), None);
        assert_eq!(typed.push('b', 1000.0), Some("b"));
    }
}
