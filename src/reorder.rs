//! Reorder and DragHandle (`style/_reorder.scss`).

use std::cell::RefCell;
use std::rc::Rc;

use phrasebook02::yew023::use_phrases_with;
use phrasebook02::{Args, Phrases};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{DomRect, Element, HtmlElement};
use yew::prelude::*;

use crate::phrasebookcatalog01::L10N;
use crate::timer::Ticker;
use crate::tooltip::next_id;
use crate::{Glyph, Icon, RowListItem, class, row_list};

/// How far a pointer travels before a press on a handle becomes a drag.
const SLOP: f64 = 4.0;
/// How near the edge of what scrolls a dragged row scrolls it, and its most
/// per tick.
const EDGE: f64 = 40.0;
const STEP: f64 = 14.0;

/// The attribute of a place outside the list a row may be let go on (a
/// [`Pager`](crate::Pager)'s page numbers, with its `drop_page`), and the
/// one a row held over it sets on it.
const DROP_PAGE: &str = "data-sort-page";
const DROP_OVER: &str = "data-drop-over";

/// What a press on is not a drag of the row: its own controls.
const CONTROLS: &str = "button, input, a, textarea, select, label, [contenteditable], \
    [role=button], [role=checkbox], [role=radio], [role=switch], [role=slider], [role=menuitem]";
/// A control the row drags by all the same (`body_drag`), such as the
/// button that picks it.
const SURFACE: &str = "data-reorder-surface";

/// One row of a [`Reorder`].
#[derive(Clone, PartialEq)]
pub struct ReorderRow {
    /// Stable across moves, so the row keeps its element (and focus).
    pub key: AttrValue,
    /// The row's name, one line, cut; also what the announcements call it.
    pub name: AttrValue,
    pub detail: Option<Html>,
    /// What ends the row.
    pub end: Html,
}

impl ReorderRow {
    pub fn new(key: impl Into<AttrValue>, name: impl Into<AttrValue>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            detail: None,
            end: Html::default(),
        }
    }

    pub fn detail(mut self, detail: Html) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn end(mut self, end: Html) -> Self {
        self.end = end;
        self
    }
}

/// What a [`Reorder`] says: a handle's name, its hint and the
/// announcements. Positions count from 1.
#[derive(Clone, Debug, PartialEq)]
pub enum ReorderText {
    Handle {
        name: AttrValue,
    },
    Hint,
    Lifted {
        name: AttrValue,
        position: usize,
        count: usize,
    },
    Moving {
        name: AttrValue,
        position: usize,
        count: usize,
    },
    Dropped {
        name: AttrValue,
        position: usize,
        count: usize,
    },
    Cancelled {
        name: AttrValue,
        position: usize,
        count: usize,
    },
}

/// The catalog's English text of a [`ReorderText`].
pub fn reorder_text(text: &ReorderText) -> String {
    phrase(&Phrases::from_catalog(&L10N), text)
}

pub(crate) fn phrase(phrases: &Phrases, text: &ReorderText) -> String {
    let (id, name, place) = match text {
        ReorderText::Handle { name } => ("sockel-reorder-handle", Some(name), None),
        ReorderText::Hint => ("sockel-reorder-hint", None, None),
        ReorderText::Lifted {
            name,
            position,
            count,
        } => ("sockel-reorder-lifted", Some(name), Some((position, count))),
        ReorderText::Moving {
            name,
            position,
            count,
        } => ("sockel-reorder-moving", Some(name), Some((position, count))),
        ReorderText::Dropped {
            name,
            position,
            count,
        } => (
            "sockel-reorder-dropped",
            Some(name),
            Some((position, count)),
        ),
        ReorderText::Cancelled {
            name,
            position,
            count,
        } => (
            "sockel-reorder-cancelled",
            Some(name),
            Some((position, count)),
        ),
    };
    let mut args = Args::new();
    if let Some(name) = name {
        args.set("name", name.as_str());
    }
    if let Some((position, count)) = place {
        args.set("position", *position).set("count", *count);
    }
    phrases.msg(id, &args)
}

/// The index a row lifted from `from` lands at when dropped in `slot` (the
/// gap before row `slot`, `0..=len`): an index in the list without it.
pub fn landing(from: usize, slot: usize) -> usize {
    if slot > from { slot - 1 } else { slot }
}

/// The slot a lifted row's middle at `y` falls in, among the rows' middles
/// measured at the press.
pub fn slot_at(middles: &[f64], y: f64) -> usize {
    middles.iter().filter(|m| **m < y).count()
}

/// Where a row lifted from `from` lands while its edges are at `top` and
/// `bottom`, among the rows' middles measured at the press: above every row
/// whose middle its top has passed, or below every one its bottom has, so a
/// drag kept within the list reaches either end whatever the rows'
/// heights. An index in the list without the row, as [`landing`] gives.
pub fn drag_target(middles: &[f64], from: usize, top: f64, bottom: f64) -> usize {
    if from >= middles.len() {
        return from;
    }
    let up = middles[..from].iter().filter(|m| **m > top).count();
    let down = middles[from + 1..].iter().filter(|m| **m < bottom).count();
    from - up + down
}

/// The index a row lifted from `from` lands at in a wrapping layout while
/// its middle is at (`x`, `y`), among the rows' `centers` measured at the
/// press: that of the nearest cell (one without a box, `NaN`, is none), as
/// an index in the list without the row.
pub fn grid_target(centers: &[(f64, f64)], from: usize, x: f64, y: f64) -> usize {
    centers
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.0.is_nan() && !c.1.is_nan())
        .min_by(|(_, a), (_, b)| {
            (a.0 - x)
                .hypot(a.1 - y)
                .total_cmp(&(b.0 - x).hypot(b.1 - y))
        })
        .map_or(from, |(at, _)| at)
}

/// How far to scroll for a pointer at `y` when what scrolls shows `top` to
/// `bottom`: up (negative) or down (positive) by up to a step, more the
/// nearer the edge, none away from them.
pub fn edge_step(y: f64, top: f64, bottom: f64) -> f64 {
    if y < top + EDGE {
        -STEP * ((top + EDGE - y) / EDGE).min(1.0)
    } else if y > bottom - EDGE {
        STEP * ((y - (bottom - EDGE)) / EDGE).min(1.0)
    } else {
        0.0
    }
}

/// Where a row lifted from `from` lands at `to` among the rows that move,
/// from 1, and how many move: fixed rows (`stays`) are not counted.
fn place(stays: &[bool], from: usize, to: usize) -> (usize, usize) {
    let before = (0..stays.len())
        .filter(|i| *i != from)
        .take(to)
        .filter(|i| !stays[*i])
        .count();
    (before + 1, stays.iter().filter(|s| !**s).count())
}

/// Moves `list[from]` to `to`, as [`Reorder`]'s `on_move` reports it.
pub fn move_item<T>(list: &mut Vec<T>, from: usize, to: usize) {
    if from < list.len() {
        let item = list.remove(from);
        list.insert(to.min(list.len()), item);
    }
}

/// Moves the rows at `rows` (indices in `list`, in order) together to `to`, an
/// index in the list without them, as [`Reorder`]'s `on_move_picked` reports
/// it.
pub fn move_items<T>(list: &mut Vec<T>, rows: &[usize], to: usize) {
    let mut moved = Vec::new();
    let mut rest = Vec::new();
    for (at, item) in std::mem::take(list).into_iter().enumerate() {
        if rows.contains(&at) {
            moved.push(item);
        } else {
            rest.push(item);
        }
    }
    let to = to.min(rest.len());
    list.extend(rest.drain(..to));
    list.append(&mut moved);
    list.append(&mut rest);
}

/// Where the rows `carried` with the row lifted from `from` land when it
/// aims at `to` (an index in the list without it): an index in the list
/// without any of them.
fn carried_landing(carried: &[bool], from: usize, to: usize) -> usize {
    (0..carried.len())
        .filter(|i| *i != from)
        .take(to)
        .filter(|i| !carried[*i])
        .count()
}

/// Whether the rows `carried` landing at `to` (an index in the list without
/// them) change the order: not when they already lie together there.
fn carried_moves(carried: &[bool], to: usize) -> bool {
    let rows: Vec<usize> = (0..carried.len()).filter(|i| carried[*i]).collect();
    let together = rows.windows(2).all(|w| w[1] == w[0] + 1);
    !(together && rows.first() == Some(&to))
}

/// Where `key` aims a row lifted from `from` that carries the rows `carried`
/// (none: it moves alone), aiming at `to` among `count` rows: past every gap
/// where the carried rows would land as they do from `to`.
fn carried_target(
    carried: &[bool],
    from: usize,
    key: &str,
    to: usize,
    count: usize,
) -> Option<usize> {
    let mut at = key_target(key, to, count)?;
    if !carried.get(from).copied().unwrap_or_default() {
        return Some(at);
    }
    let was = carried_landing(carried, from, to);
    while carried_landing(carried, from, at) == was {
        let next = key_target(key, at, count)?;
        if next == at {
            break;
        }
        at = next;
    }
    Some(at)
}

/// Adds the `data` attributes to `html`'s element.
pub(crate) fn with_data(mut html: Html, data: &[(&'static str, AttrValue)]) -> Html {
    if let Html::VTag(tag) = &mut html {
        for (name, value) in data {
            Rc::make_mut(tag).add_attribute(name, value.clone());
        }
    }
    html
}

/// Where a lifted row aims after `key`, from `to` among `count` rows.
pub fn key_target(key: &str, to: usize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match key {
        "ArrowUp" => Some(to.saturating_sub(1)),
        "ArrowDown" => Some((to + 1).min(last)),
        "Home" => Some(0),
        "End" => Some(last),
        _ => None,
    }
}

/// A row's classes, with `lift` the lifted row and where it aims: the lifted
/// row is `LIFTED`, the row it lands beside `DROP_BEFORE` or `DROP_AFTER`.
pub fn reorder_row(index: usize, lift: Option<(usize, usize)>) -> Classes {
    let (lifted, before, after) = match lift {
        Some((from, to)) => (
            index == from,
            index == to && to < from,
            index == to && to > from,
        ),
        None => (false, false, false),
    };
    classes!(
        class::REORDER_ROW,
        lifted.then_some(class::LIFTED),
        before.then_some(class::DROP_BEFORE),
        after.then_some(class::DROP_AFTER)
    )
}

/// A drag handle's classes.
pub fn drag_handle(lifted: bool) -> Classes {
    classes!(class::DRAG_HANDLE, lifted.then_some(class::LIFTED))
}

#[derive(Properties, PartialEq)]
pub struct DragHandleProps {
    /// The handle's accessible name ("Move Alpha").
    pub label: AttrValue,
    #[prop_or_default]
    pub lifted: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    /// A tooltip, for apps whose buttons all carry one.
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The app's own tooltip (`data-tip`), for apps that style it in place
    /// of the native `title`.
    #[prop_or_default]
    pub tip: Option<AttrValue>,
    #[prop_or_default]
    pub onpointerdown: Callback<PointerEvent>,
    #[prop_or_default]
    pub onpointermove: Callback<PointerEvent>,
    #[prop_or_default]
    pub onpointerup: Callback<PointerEvent>,
    #[prop_or_default]
    pub onpointercancel: Callback<PointerEvent>,
    #[prop_or_default]
    pub onkeydown: Callback<KeyboardEvent>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// The grip a row is dragged by: a small square button, its glyph from the
/// [`IconSet`](crate::IconSet). It takes no touch scrolling, so a drag
/// starts at once on a phone.
#[function_component]
pub fn DragHandle(props: &DragHandleProps) -> Html {
    html! {
        <button type="button" class={classes!(drag_handle(props.lifted), props.class.clone())}
            aria-label={props.label.clone()} aria-describedby={props.describedby.clone()}
            title={props.title.clone()} data-tip={props.tip.clone()} disabled={props.disabled} data-testid={props.testid.clone()}
            onpointerdown={props.onpointerdown.clone()} onpointermove={props.onpointermove.clone()}
            onpointerup={props.onpointerup.clone()} onpointercancel={props.onpointercancel.clone()}
            onkeydown={props.onkeydown.clone()} onblur={props.onblur.clone()}>
            <Icon glyph={Glyph::Grip} />
        </button>
    }
}

/// One row of a [`Reorder`] in its children mode: the row's own element,
/// holding whatever the app renders, a [`ReorderHandle`] somewhere in it.
#[derive(Clone, Properties, PartialEq)]
pub struct ReorderItemProps {
    /// Stable across moves, so the row keeps its element (and focus).
    pub id: AttrValue,
    /// What the announcements and the handle's name call the row.
    pub name: AttrValue,
    /// The row stays where it is and has no handle, but the others land
    /// beside it: a heading or a marker among them.
    #[prop_or_default]
    pub fixed: bool,
    /// Shown only while a row is lifted, as a place to drop it (a marker
    /// at the list's end); best last, so no row moves when it shows.
    #[prop_or_default]
    pub lifted_only: bool,
    /// The list's current row (`aria-current`).
    #[prop_or_default]
    pub current: bool,
    /// Picked with others: a picked row lifted carries every other picked
    /// row that is not `fixed` (`CARRIED`, where it lies until the drop), and
    /// `on_move_picked` reports their move. A [`Reorder`]'s alone.
    #[prop_or_default]
    pub picked: bool,
    /// The row's `role` (`row` in a table).
    #[prop_or_default]
    pub role: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// More `data-*` attributes (`("data-shape", ..)`) the app's code reads.
    #[prop_or_default]
    pub data: Vec<(&'static str, AttrValue)>,
    /// Never the click a drag of it ends in.
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub ondblclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub oncontextmenu: Callback<MouseEvent>,
    #[prop_or_default]
    pub onmouseenter: Callback<MouseEvent>,
    #[prop_or_default]
    pub onmouseleave: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

/// A row of a [`Reorder`]'s children. Outside a `Reorder` it is a plain
/// `<div>` of its children.
#[function_component]
pub fn ReorderItem(props: &ReorderItemProps) -> Html {
    let row = html! {
        <div class={props.class.clone()} data-testid={props.testid.clone()} role={props.role.clone()}
            aria-current={props.current.then_some("true")}
            onclick={props.onclick.clone()} ondblclick={props.ondblclick.clone()}
            oncontextmenu={props.oncontextmenu.clone()}
            onmouseenter={props.onmouseenter.clone()} onmouseleave={props.onmouseleave.clone()}>
            { props.children.clone() }
        </div>
    };
    with_data(row, &props.data)
}

#[derive(Properties, PartialEq)]
pub struct ReorderHandleProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// The [`DragHandle`] of the [`ReorderItem`] it is in, wired to its
/// [`Reorder`]; nothing outside one.
#[function_component]
pub fn ReorderHandle(props: &ReorderHandleProps) -> Html {
    match use_context::<Grip>() {
        Some(grip) => grip.handle(props.class.clone(), props.testid.clone()),
        None => Html::default(),
    }
}

#[derive(Properties, PartialEq)]
pub struct ReorderProps {
    /// The rows, each a [`RowListItem`] with its handle at the start.
    #[prop_or_default]
    pub rows: Vec<ReorderRow>,
    /// The children mode, in place of `rows`: the rows are the app's own,
    /// each a [`ReorderItem`] with a [`ReorderHandle`] in it, laid out
    /// by `class`.
    #[prop_or_default]
    pub children: ChildrenWithProps<ReorderItem>,
    /// A row moved from `from` to `to`, an index in the list without it:
    /// apply it with [`move_item`].
    pub on_move: Callback<(usize, usize)>,
    /// Picked rows ([`ReorderItemProps::picked`]) moved together, one of
    /// several lifted: their indices in order and `to`, an index in the list
    /// without them; apply it with [`move_items`]. A row lifted alone is
    /// `on_move`'s.
    #[prop_or_default]
    pub on_move_picked: Callback<(Vec<usize>, usize)>,
    /// The texts, overriding the catalog's.
    #[prop_or_default]
    pub text: Option<Callback<ReorderText, String>>,
    #[prop_or_default]
    pub divided: bool,
    #[prop_or_default]
    pub boxed: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// The arrows move a focused handle's row at once, as Alt and an arrow
    /// do; Space and Enter still lift it.
    #[prop_or_default]
    pub quick_keys: bool,
    /// Each handle's name is its tooltip too.
    #[prop_or_default]
    pub titles: bool,
    /// Each handle's name is its `data-tip` tooltip too, the attribute an
    /// app's own tooltips read.
    #[prop_or_default]
    pub tips: bool,
    /// A mouse or pen drags a row from anywhere on it that is off its
    /// controls, not only by the handle; a finger still only by the handle.
    /// A control marked `data-reorder-surface` (the button that picks the
    /// row) drags it too, and is clicked when not dragged.
    #[prop_or_default]
    pub body_drag: bool,
    /// A dragged row near the top or bottom edge of what scrolls the list
    /// (the nearest scrolling ancestor, else the page) scrolls it.
    #[prop_or_default]
    pub auto_scroll: bool,
    /// The rows wrap in a strip or grid (children mode, laid out by
    /// `class`): a dragged row moves in both directions and lands in the
    /// nearest cell, and the drop is marked at the row's side.
    #[prop_or_default]
    pub grid: bool,
    /// Where a lifted row aims, `(from, to)` as `on_move` would report it,
    /// each time that changes; `None` once it is set down.
    #[prop_or_default]
    pub on_aim: Callback<Option<(usize, usize)>>,
    /// A row dragged by the pointer may be let go on a place outside the
    /// list wearing `data-sort-page` (another page's number in a
    /// [`Pager`](crate::Pager) with `drop_page`): `(from, value)`, the row's
    /// index and that attribute's value, in place of `on_move`. While the
    /// pointer holds a row over one, the place wears `data-drop-over` and
    /// no row is marked.
    #[prop_or_default]
    pub on_drop_page: Option<Callback<(usize, AttrValue)>>,
    /// The children mode's list element wears this `role` (`rowgroup` for a
    /// table's body, whose rows wear `role="row"`).
    #[prop_or_default]
    pub role: Option<AttrValue>,
    /// The children mode's list ends with this, after the rows: the tile
    /// that adds one in a strip, which is not a row and never moves.
    #[prop_or_default]
    pub tail: Html,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Lift {
    from: usize,
    to: usize,
    pointer: bool,
}

/// The rows as measured for a drag, within the page and what scrolls the
/// list. A row with no box (one shown only while a row is lifted) has no
/// middle, so no drag passes it.
struct Layout {
    middles: Vec<f64>,
    /// The rows' middles across and down, for a grid.
    centers: Vec<(f64, f64)>,
    /// Half the lifted row's height.
    half: f64,
    least: f64,
    most: f64,
    least_x: f64,
    most_x: f64,
    unshown: bool,
}

impl Layout {
    fn measure(rows: &[DomRect], from: usize, (ox, oy): (f64, f64), grid: bool) -> Option<Self> {
        let own = rows.get(from)?;
        let shown = |r: &DomRect| r.width() > 0.0 || r.height() > 0.0;
        let boxed: Vec<_> = rows.iter().filter(|r| shown(r)).collect();
        let (first, last) = (boxed.first()?, boxed.last()?);
        let (least_x, most_x) = if grid {
            let left = boxed.iter().map(|r| r.left()).fold(f64::MAX, f64::min);
            let right = boxed.iter().map(|r| r.right()).fold(f64::MIN, f64::max);
            (left - own.left(), right - own.right())
        } else {
            (0.0, 0.0)
        };
        let (least, most) = if grid {
            let top = boxed.iter().map(|r| r.top()).fold(f64::MAX, f64::min);
            let bottom = boxed.iter().map(|r| r.bottom()).fold(f64::MIN, f64::max);
            (top - own.top(), bottom - own.bottom())
        } else {
            (first.top() - own.top(), last.bottom() - own.bottom())
        };
        let middle = |r: &DomRect| {
            if shown(r) {
                r.top() + r.height() / 2.0 + oy
            } else {
                f64::NAN
            }
        };
        Some(Self {
            middles: rows.iter().map(middle).collect(),
            centers: if grid {
                rows.iter()
                    .map(|r| (r.left() + r.width() / 2.0 + ox, middle(r)))
                    .collect()
            } else {
                Vec::new()
            },
            half: own.height() / 2.0,
            least,
            most,
            least_x,
            most_x,
            unshown: boxed.len() < rows.len(),
        })
    }
}

fn rects(list: &Element, count: usize) -> Vec<DomRect> {
    let rows = list.children();
    (0..count as u32)
        .filter_map(|i| rows.item(i))
        .map(|r| r.get_bounding_client_rect())
        .collect()
}

struct Drag {
    id: i32,
    /// What takes the pointer once a press on the row's body moves past
    /// `SLOP`, so one that never does keeps its click; `None` once taken
    /// (a press on the handle takes it at once).
    deferred: Option<HtmlElement>,
    from: usize,
    /// Held by the row itself, which has no handle (`body_drag`): the row
    /// hears the pointer.
    by_row: bool,
    /// The place outside the list it is held over (`on_drop_page`).
    over: Option<Element>,
    /// Where it was pressed, within the page and what scrolls the list.
    start: (f64, f64),
    layout: Layout,
    active: bool,
    row: Element,
    /// The pointer's client position at its last move.
    at: (f64, f64),
    scroller: Option<Element>,
    _ticker: Option<Ticker>,
}

/// What scrolls `list`: itself or the nearest element around it that
/// scrolls up and down, else the page.
pub(crate) fn scroller_of(list: &Element) -> Option<Element> {
    let window = web_sys::window()?;
    let mut at = Some(list.clone());
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

fn is_page(el: &Element) -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.scrolling_element())
        .is_some_and(|page| page == *el)
}

/// How far the page and `scroller` (when not the page) are scrolled: what a
/// client position adds to be one measured at the press.
pub(crate) fn scrolled(scroller: Option<&Element>) -> (f64, f64) {
    let Some(window) = web_sys::window() else {
        return (0.0, 0.0);
    };
    let (x, y) = (
        window.scroll_x().unwrap_or_default(),
        window.scroll_y().unwrap_or_default(),
    );
    let inner = scroller
        .filter(|s| !is_page(s))
        .map_or(0.0, |s| f64::from(s.scroll_top()));
    (x, y + inner)
}

/// The top and bottom of what `scroller` shows, in client heights.
fn shown(scroller: &Element) -> (f64, f64) {
    if is_page(scroller) {
        let height = web_sys::window()
            .and_then(|w| w.inner_height().ok())
            .and_then(|h| h.as_f64())
            .unwrap_or_default();
        (0.0, height)
    } else {
        let rect = scroller.get_bounding_client_rect();
        (rect.top(), rect.bottom())
    }
}

#[derive(Default)]
struct Live {
    lift: Option<Lift>,
    drag: Option<Drag>,
    status: String,
    /// The key of a row just moved and the keys before the move: its
    /// handle is focused wherever the app's new order puts it.
    focus: Option<(AttrValue, Vec<AttrValue>, usize)>,
    /// The keys as last rendered.
    keys: Vec<AttrValue>,
    /// The aim last told `on_aim`.
    aimed: Option<(usize, usize)>,
}

/// Keeps the click a drag ends in from reaching anything: once, at the
/// window before anyone else hears it, or not at all once this task ends.
fn swallow_click() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let swallow = Closure::<dyn Fn(web_sys::Event)>::new(|e: web_sys::Event| {
        e.stop_propagation();
        e.prevent_default();
    });
    let f: web_sys::js_sys::Function = swallow.as_ref().clone().unchecked_into();
    let options = web_sys::AddEventListenerOptions::new();
    options.set_capture(true);
    options.set_once(true);
    if window
        .add_event_listener_with_callback_and_add_event_listener_options("click", &f, &options)
        .is_err()
    {
        return;
    }
    let end = Closure::once_into_js(move || {
        if let Some(window) = web_sys::window() {
            let _ = window.remove_event_listener_with_callback_and_bool("click", &f, true);
        }
        drop(swallow);
    });
    let _ = window.set_timeout_with_callback(end.unchecked_ref());
}

/// Measures a drag's rows again once the lift has shown those it had no box
/// for at the press, keeping the pointer's place on the lifted row.
fn remeasure(live: &mut Live, list: &Element, count: usize, grid: bool) {
    if live.lift.is_none() {
        return;
    }
    let Some(drag) = live.drag.as_mut().filter(|d| d.active && d.layout.unshown) else {
        return;
    };
    let style = drag.row.get_attribute("style");
    let _ = drag.row.remove_attribute("style");
    let offset = scrolled(drag.scroller.as_ref());
    let layout = Layout::measure(&rects(list, count), drag.from, offset, grid);
    if let Some(style) = style {
        let _ = drag.row.set_attribute("style", &style);
    }
    let Some(mut layout) = layout else {
        return;
    };
    let from = drag.from;
    drag.start.1 += layout.middles[from] - drag.layout.middles[from];
    if grid {
        drag.start.0 += layout.centers[from].0 - drag.layout.centers[from].0;
    }
    // Once: a row the app keeps without a box stays without a middle.
    layout.unshown = false;
    drag.layout = layout;
}

/// The place outside the list under the client position `(x, y)` that a
/// row may be let go on.
fn drop_page_at((x, y): (f64, f64)) -> Option<Element> {
    web_sys::window()?
        .document()?
        .element_from_point(x as f32, y as f32)?
        .closest(&format!("[{DROP_PAGE}]"))
        .ok()?
}

/// Marks `over` as the place a held row is over, unmarking the one before.
fn mark_over(drag: &mut Drag, over: Option<Element>) {
    if drag.over == over {
        return;
    }
    if let Some(old) = &drag.over {
        let _ = old.remove_attribute(DROP_OVER);
    }
    if let Some(new) = &over {
        let _ = new.set_attribute(DROP_OVER, "");
    }
    drag.over = over;
}

fn handle_at(list: &NodeRef, index: usize) -> Option<HtmlElement> {
    list.cast::<Element>()?
        .children()
        .item(index as u32)?
        .query_selector(&format!(".{}", class::DRAG_HANDLE))
        .ok()??
        .dyn_into()
        .ok()
}

/// A list whose rows move by a drag on their [`DragHandle`] or by keyboard
/// (Space lifts, the arrows, Home and End aim, Space drops, Escape
/// cancels; Alt and an arrow moves at once). While a row is dragged
/// nothing reflows: the rows are measured once at the press, the lifted
/// row follows the pointer by a transform and the drop is marked by a box
/// shadow; the order changes once, on the drop. A screen reader hears each
/// step.
#[function_component]
pub fn Reorder(props: &ReorderProps) -> Html {
    let live = use_mut_ref(Live::default);
    let redraw = use_force_update();
    let list = use_node_ref();
    let hint_id = use_state(|| next_id("reorder-hint"));
    let items = !props.children.is_empty();
    let count = if items {
        props.children.len()
    } else {
        props.rows.len()
    };
    let phrases = use_phrases_with(&L10N);
    let say: Rc<dyn Fn(ReorderText) -> String> = match &props.text {
        Some(text) => {
            let text = text.clone();
            Rc::new(move |t| text.emit(t))
        }
        None => Rc::new(move |t| phrase(&phrases, &t)),
    };
    let keys: Vec<AttrValue> = if items {
        props.children.iter().map(|c| c.props.id.clone()).collect()
    } else {
        props.rows.iter().map(|r| r.key.clone()).collect()
    };
    let stays: Rc<[bool]> = if items {
        props.children.iter().map(|c| c.props.fixed).collect()
    } else {
        vec![false; count].into()
    };
    // The rows a picked row lifted carries: none unless several are picked.
    let mut carried: Vec<bool> = if items {
        props
            .children
            .iter()
            .map(|c| c.props.picked && !c.props.fixed)
            .collect()
    } else {
        vec![false; count]
    };
    if carried.iter().filter(|c| **c).count() < 2 {
        carried.fill(false);
    }
    let carried: Rc<[bool]> = carried.into();
    live.borrow_mut().keys = keys.clone();

    {
        let live = live.clone();
        let list = list.clone();
        let keys = keys.clone();
        let on_aim = props.on_aim.clone();
        let grid = props.grid;
        use_effect(move || {
            let pending = live.borrow().focus.clone();
            if let Some((key, before, to)) = pending {
                if keys != before {
                    live.borrow_mut().focus = None;
                    if let Some(at) = keys.iter().position(|k| *k == key)
                        && let Some(handle) = handle_at(&list, at)
                    {
                        let _ = handle.focus();
                    }
                } else if let Some(window) = web_sys::window() {
                    // Keys that never change (an app keying rows by index)
                    // move no element: once the app has had its say, focus
                    // the handle where the row went.
                    let live = live.clone();
                    let list = list.clone();
                    let end = Closure::once_into_js(move || {
                        if live.borrow_mut().focus.take().is_some()
                            && let Some(handle) = handle_at(&list, to)
                        {
                            let _ = handle.focus();
                        }
                    });
                    let _ = window.set_timeout_with_callback(end.unchecked_ref());
                }
            }
            if let Some(list) = list.cast::<Element>() {
                remeasure(&mut live.borrow_mut(), &list, keys.len(), grid);
            }
            let aim = live.borrow().lift.map(|l| (l.from, l.to));
            if live.borrow().aimed != aim {
                live.borrow_mut().aimed = aim;
                on_aim.emit(aim);
            }
        });
    }

    if live
        .borrow()
        .lift
        .is_some_and(|l| l.from >= count || l.to >= count)
    {
        live.borrow_mut().lift = None;
    }
    let lift = live.borrow().lift;
    let grip = |index: usize, name: AttrValue, fixed: bool| Grip {
        ctx: Ctx {
            live: live.clone(),
            redraw: redraw.clone(),
            list: list.clone(),
            on_move: props.on_move.clone(),
            on_drop_page: props.on_drop_page.clone(),
            on_move_picked: props.on_move_picked.clone(),
            say: say.clone(),
            index,
            count,
            stays: stays.clone(),
            carried: carried.clone(),
            name,
            key: keys[index].clone(),
            quick: props.quick_keys,
            body: props.body_drag,
            auto: props.auto_scroll,
            grid: props.grid,
            disabled: props.disabled,
            fixed,
        },
        lifted: lift.is_some_and(|l| l.from == index),
        disabled: props.disabled,
        hint: (*hint_id).clone(),
        titles: props.titles,
        tips: props.tips,
    };
    let marks = |index: usize, fixed: bool| {
        let carries = lift.is_some_and(|l| l.from != index && carried[l.from]);
        classes!(
            reorder_row(index, lift.map(|l| (l.from, l.to))),
            (carries && carried[index]).then_some(class::CARRIED),
            (props.body_drag && !fixed).then_some(class::REORDER_GRAB),
            props.grid.then_some(class::REORDER_TILED)
        )
    };
    let status = html! {
        <>
            <span id={(*hint_id).clone()} class={class::VISUALLY_HIDDEN}>{ say(ReorderText::Hint) }</span>
            <span class={class::VISUALLY_HIDDEN} role="status" aria-live="assertive">
                { live.borrow().status.clone() }
            </span>
        </>
    };

    if items {
        let rows: Html = props
            .children
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let item = &item.props;
                let grip = grip(index, item.name.clone(), item.fixed);
                let onpointerdown = grip.ctx.clone().body_down();
                // A row without a handle hears its own drag.
                let onpointermove = grip.ctx.clone().row_only(grip.ctx.clone().pointer_move());
                let onpointerup = grip.ctx.clone().row_only(grip.ctx.clone().pointer_up());
                let onpointercancel = grip.ctx.clone().row_only(grip.ctx.clone().pointer_cancel());
                let onclick = item.onclick.clone();
                let row = html! {
                    <div class={classes!(marks(index, item.fixed), item.class.clone())} data-testid={item.testid.clone()}
                        role={item.role.clone()}
                        hidden={item.lifted_only && lift.is_none()} aria-current={item.current.then_some("true")}
                        {onpointerdown} {onpointermove} {onpointerup} {onpointercancel}
                        {onclick} ondblclick={item.ondblclick.clone()}
                        oncontextmenu={item.oncontextmenu.clone()}
                        onmouseenter={item.onmouseenter.clone()} onmouseleave={item.onmouseleave.clone()}>
                        { item.children.clone() }
                    </div>
                };
                html! {
                    <ContextProvider<Grip> key={item.id.to_string()} context={grip}>
                        { with_data(row, &item.data) }
                    </ContextProvider<Grip>>
                }
            })
            .collect();

        // The rows are the list's first children, the hint and the status
        // after them.
        return html! {
            <div ref={list.clone()} class={classes!(class::REORDER, props.class.clone())} data-testid={props.testid.clone()}
                role={props.role.clone()}>
                { rows }
                { props.tail.clone() }
                { status }
            </div>
        };
    }

    let rows: Html = props
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let grip = grip(index, row.name.clone(), false);
            let onpointerdown = grip.ctx.clone().body_down();
            let handle = grip.handle(Classes::new(), None);
            html! {
                <RowListItem key={row.key.to_string()} name={html! { { row.name.clone() } }}
                    detail={row.detail.clone()} start={handle}
                    class={marks(index, false)} {onpointerdown}>
                    { row.end.clone() }
                </RowListItem>
            }
        })
        .collect();

    html! {
        <div class={classes!(class::REORDER, props.class.clone())} data-testid={props.testid.clone()}>
            <ul ref={list.clone()} class={row_list(props.divided, props.boxed)}>{ rows }</ul>
            { status }
        </div>
    }
}

/// A row's handle, as its [`Reorder`] renders it: also the context a
/// [`ReorderHandle`] in a [`ReorderItem`] reads.
#[derive(Clone)]
struct Grip {
    ctx: Ctx,
    lifted: bool,
    disabled: bool,
    hint: AttrValue,
    titles: bool,
    tips: bool,
}

impl PartialEq for Grip {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (&self.ctx, &other.ctx);
        Rc::ptr_eq(&a.live, &b.live)
            && Rc::ptr_eq(&a.say, &b.say)
            && a.on_move == b.on_move
            && a.on_drop_page == b.on_drop_page
            && a.on_move_picked == b.on_move_picked
            && (a.index, a.count, a.quick) == (b.index, b.count, b.quick)
            && a.stays == b.stays
            && a.carried == b.carried
            && (a.body, a.auto, a.grid, a.disabled, a.fixed)
                == (b.body, b.auto, b.grid, b.disabled, b.fixed)
            && (&a.name, &a.key) == (&b.name, &b.key)
            && (self.lifted, self.disabled, self.titles, self.tips)
                == (other.lifted, other.disabled, other.titles, other.tips)
            && self.hint == other.hint
    }
}

impl Grip {
    fn handle(&self, class: Classes, testid: Option<AttrValue>) -> Html {
        let ctx = &self.ctx;
        if ctx.fixed {
            return Html::default();
        }
        let label: AttrValue = (ctx.say)(ReorderText::Handle {
            name: ctx.name.clone(),
        })
        .into();
        html! {
            <DragHandle label={label.clone()} title={self.titles.then(|| label.clone())} tip={self.tips.then_some(label)}
                lifted={self.lifted} disabled={self.disabled}
                describedby={self.hint.clone()} {class} {testid}
                onpointerdown={ctx.clone().pointer_down()} onpointermove={ctx.clone().pointer_move()}
                onpointerup={ctx.clone().pointer_up()} onpointercancel={ctx.clone().pointer_cancel()}
                onkeydown={ctx.clone().key_down()} onblur={ctx.clone().blur()} />
        }
    }
}

/// What one row's handle acts on.
#[derive(Clone)]
struct Ctx {
    live: Rc<RefCell<Live>>,
    redraw: UseForceUpdateHandle,
    list: NodeRef,
    on_move: Callback<(usize, usize)>,
    on_drop_page: Option<Callback<(usize, AttrValue)>>,
    on_move_picked: Callback<(Vec<usize>, usize)>,
    say: Rc<dyn Fn(ReorderText) -> String>,
    index: usize,
    count: usize,
    /// Which rows are fixed, by index.
    stays: Rc<[bool]>,
    /// Which rows a picked row lifted carries, by index; none unless several
    /// are picked.
    carried: Rc<[bool]>,
    name: AttrValue,
    key: AttrValue,
    /// The arrows move at once (`quick_keys`).
    quick: bool,
    body: bool,
    auto: bool,
    grid: bool,
    disabled: bool,
    /// The row stays put ([`ReorderItemProps::fixed`]).
    fixed: bool,
}

impl Ctx {
    fn announce(&self, text: ReorderText) {
        self.live.borrow_mut().status = (self.say)(text);
    }

    fn lifted(&self, at: usize) -> ReorderText {
        let (position, count) = place(&self.stays, self.index, at);
        ReorderText::Lifted {
            name: self.name.clone(),
            position,
            count,
        }
    }

    fn moving(&self, at: usize) -> ReorderText {
        let (position, count) = place(&self.stays, self.index, at);
        ReorderText::Moving {
            name: self.name.clone(),
            position,
            count,
        }
    }

    fn cancelled(&self) -> ReorderText {
        let (position, count) = place(&self.stays, self.index, self.index);
        ReorderText::Cancelled {
            name: self.name.clone(),
            position,
            count,
        }
    }

    /// Ends a lift at `to`: the one place the order changes.
    fn drop_at(&self, to: usize) {
        let before = self.live.borrow().keys.clone();
        if self.carried[self.index] {
            let at = carried_landing(&self.carried, self.index, to);
            if carried_moves(&self.carried, at) {
                let rows = (0..self.count).filter(|i| self.carried[*i]).collect();
                self.on_move_picked.emit((rows, at));
                self.live.borrow_mut().focus = Some((self.key.clone(), before, at));
            }
        } else if to != self.index {
            self.on_move.emit((self.index, to));
            self.live.borrow_mut().focus = Some((self.key.clone(), before, to));
        }
        let (position, count) = place(&self.stays, self.index, to);
        self.announce(ReorderText::Dropped {
            name: self.name.clone(),
            position,
            count,
        });
        self.live.borrow_mut().lift = None;
        self.redraw.force_update();
    }

    fn cancel(&self) {
        let drag = self.live.borrow_mut().drag.take();
        if let Some(mut drag) = drag {
            let _ = drag.row.remove_attribute("style");
            mark_over(&mut drag, None);
        }
        self.live.borrow_mut().lift = None;
        self.announce(self.cancelled());
        self.redraw.force_update();
    }

    fn pointer_down(self) -> Callback<PointerEvent> {
        Callback::from(move |e: PointerEvent| {
            if e.button() != 0 || self.live.borrow().lift.is_some() {
                return;
            }
            // The handle the press is on: yew delegates events, so the
            // current target is the root, and capturing the pointer there
            // would keep every later move from the handle's listeners.
            let Some(handle) = e
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .and_then(|t| {
                    t.closest(&format!(".{}", class::DRAG_HANDLE))
                        .ok()
                        .flatten()
                })
                .and_then(|t| t.dyn_into::<HtmlElement>().ok())
            else {
                return;
            };
            self.begin(&e, &handle, false, false);
        })
    }

    /// A press on the row off its handle and controls (`body_drag`), by a
    /// mouse or pen.
    fn body_down(self) -> Callback<PointerEvent> {
        Callback::from(move |e: PointerEvent| {
            if !self.body
                || self.disabled
                || self.fixed
                || e.button() != 0
                || e.pointer_type() == "touch"
                || self.live.borrow().lift.is_some()
            {
                return;
            }
            let Some(target) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            let handle_class = format!(".{}", class::DRAG_HANDLE);
            if target.closest(&handle_class).ok().flatten().is_some() {
                return;
            }
            let Some(row) = self
                .list
                .cast::<Element>()
                .and_then(|l| l.children().item(self.index as u32))
            else {
                return;
            };
            // Yew bubbles a press in a portal (a menu the row opened) through
            // the row, whose element does not hold it.
            if !row.contains(Some(&target)) {
                return;
            }
            if target
                .closest(CONTROLS)
                .ok()
                .flatten()
                .is_some_and(|c| row.contains(Some(&c)) && !c.has_attribute(SURFACE))
            {
                return;
            }
            // A row without a handle is held by itself.
            let handle = row.query_selector(&handle_class).ok().flatten();
            let by_row = handle.is_none();
            let Ok(handle) = handle.unwrap_or(row).dyn_into::<HtmlElement>() else {
                return;
            };
            self.begin(&e, &handle, by_row, true);
        })
    }

    /// `then` for a pointer event the row itself hears while it is held
    /// by itself, not by its handle.
    fn row_only(self, then: Callback<PointerEvent>) -> Callback<PointerEvent> {
        Callback::from(move |e: PointerEvent| {
            // The row hears a body press until it takes the pointer.
            if self
                .live
                .borrow()
                .drag
                .as_ref()
                .is_some_and(|d| d.by_row || d.deferred.is_some())
            {
                then.emit(e);
            }
        })
    }

    fn begin(&self, e: &PointerEvent, handle: &HtmlElement, by_row: bool, defer: bool) {
        let Some(list) = self.list.cast::<Element>() else {
            return;
        };
        let Some(row) = list.children().item(self.index as u32) else {
            return;
        };
        let scroller = self.auto.then(|| scroller_of(&list)).flatten();
        let (ox, oy) = scrolled(scroller.as_ref());
        let Some(layout) =
            Layout::measure(&rects(&list, self.count), self.index, (ox, oy), self.grid)
        else {
            return;
        };
        e.prevent_default();
        let _ = handle.focus();
        if !defer {
            let _ = handle.set_pointer_capture(e.pointer_id());
        }
        let ticker = scroller.is_some().then(|| {
            let ctx = self.clone();
            Ticker::start(30, move || ctx.tick())
        });
        let drag = Drag {
            id: e.pointer_id(),
            deferred: defer.then(|| handle.clone()),
            from: self.index,
            by_row,
            over: None,
            start: (f64::from(e.client_x()) + ox, f64::from(e.client_y()) + oy),
            layout,
            active: false,
            row,
            at: (f64::from(e.client_x()), f64::from(e.client_y())),
            scroller,
            _ticker: ticker.flatten(),
        };
        self.live.borrow_mut().drag = Some(drag);
    }

    /// Scrolls what holds the list while the dragged row's pointer is near
    /// its edge; `false` once the drag is over.
    fn tick(&self) -> bool {
        let (scroller, at, active) = {
            let live = self.live.borrow();
            let Some(drag) = live.drag.as_ref().filter(|d| d.row.is_connected()) else {
                return false;
            };
            let Some(scroller) = drag.scroller.clone() else {
                return false;
            };
            (scroller, drag.at, drag.active)
        };
        if !active {
            return true;
        }
        let (top, bottom) = shown(&scroller);
        let by = edge_step(at.1, top, bottom);
        if by != 0.0 {
            let before = scroller.scroll_top();
            scroller.set_scroll_top(before + by.round() as i32);
            if scroller.scroll_top() != before {
                self.drag_to(at);
            }
        }
        true
    }

    fn pointer_move(self) -> Callback<PointerEvent> {
        Callback::from(move |e: PointerEvent| {
            let ours = self
                .live
                .borrow()
                .drag
                .as_ref()
                .is_some_and(|d| d.id == e.pointer_id());
            if ours {
                self.drag_to((f64::from(e.client_x()), f64::from(e.client_y())));
            }
        })
    }

    /// The dragged row follows the pointer at client `at` (or what scrolled
    /// under it).
    fn drag_to(&self, at: (f64, f64)) {
        let mut live = self.live.borrow_mut();
        let Some(drag) = live.drag.as_mut() else {
            return;
        };
        drag.at = at;
        let (ox, oy) = scrolled(drag.scroller.as_ref());
        let dy = at.1 + oy - drag.start.1;
        let dx = if self.grid {
            at.0 + ox - drag.start.0
        } else {
            0.0
        };
        if !drag.active && dx.hypot(dy) < SLOP {
            return;
        }
        let layout = &drag.layout;
        let dy = dy.clamp(layout.least, layout.most);
        let dx = dx.clamp(layout.least_x, layout.most_x);
        let style = if self.grid {
            format!("--reorder-shift: {dy}px; --reorder-shift-x: {dx}px")
        } else {
            format!("--reorder-shift: {dy}px")
        };
        let _ = drag.row.set_attribute("style", &style);
        let over = self.on_drop_page.as_ref().and_then(|_| drop_page_at(at));
        let to = if over.is_some() {
            // Over a place outside the list: the row goes there, not
            // among the rows.
            drag.from
        } else if self.grid {
            let (cx, cy) = layout.centers[drag.from];
            grid_target(&layout.centers, drag.from, cx + dx, cy + dy)
        } else {
            let middle = layout.middles[drag.from] + dy;
            drag_target(
                &layout.middles,
                drag.from,
                middle - layout.half,
                middle + layout.half,
            )
        };
        mark_over(drag, over);
        let from = drag.from;
        let started = !drag.active;
        if let Some(handle) = drag.deferred.take() {
            let _ = handle.set_pointer_capture(drag.id);
        }
        drag.active = true;
        if started || live.lift.is_none_or(|l| l.to != to) {
            live.lift = Some(Lift {
                from,
                to,
                pointer: true,
            });
            drop(live);
            if started {
                self.announce(self.lifted(from));
            }
            self.redraw.force_update();
        }
    }

    fn pointer_up(self) -> Callback<PointerEvent> {
        Callback::from(move |e: PointerEvent| {
            let drag = self.live.borrow_mut().drag.take();
            let Some(mut drag) = drag.filter(|d| d.id == e.pointer_id()) else {
                return;
            };
            let _ = drag.row.remove_attribute("style");
            mark_over(&mut drag, None);
            if drag.active {
                swallow_click();
            }
            let page = match &self.on_drop_page {
                Some(on) if drag.active => {
                    drop_page_at((f64::from(e.client_x()), f64::from(e.client_y())))
                        .and_then(|page| page.get_attribute(DROP_PAGE))
                        .map(|value| (on, value))
                }
                _ => None,
            };
            if let Some((on, value)) = page {
                self.live.borrow_mut().lift = None;
                on.emit((self.index, value.into()));
                self.redraw.force_update();
                return;
            }
            let lift = self.live.borrow().lift;
            match lift {
                Some(lift) if drag.active && lift.pointer => self.drop_at(lift.to),
                _ => {}
            }
        })
    }

    fn pointer_cancel(self) -> Callback<PointerEvent> {
        Callback::from(move |_: PointerEvent| {
            if self.live.borrow().drag.is_some() {
                self.cancel();
            }
        })
    }

    fn key_down(self) -> Callback<KeyboardEvent> {
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            let lift = self.live.borrow().lift;
            match lift {
                Some(lift) if lift.pointer => {
                    if key == "Escape" {
                        e.prevent_default();
                        self.cancel();
                    }
                }
                Some(lift) => {
                    if key == " " || key == "Enter" {
                        e.prevent_default();
                        self.drop_at(lift.to);
                    } else if key == "Escape" {
                        e.prevent_default();
                        self.cancel();
                    } else if let Some(to) =
                        carried_target(&self.carried, self.index, &key, lift.to, self.count)
                    {
                        e.prevent_default();
                        self.live.borrow_mut().lift = Some(Lift { to, ..lift });
                        self.announce(self.moving(to));
                        self.redraw.force_update();
                    }
                }
                None if key == " " || key == "Enter" => {
                    e.prevent_default();
                    self.live.borrow_mut().lift = Some(Lift {
                        from: self.index,
                        to: self.index,
                        pointer: false,
                    });
                    self.announce(self.lifted(self.index));
                    self.redraw.force_update();
                }
                None if (e.alt_key() || self.quick) && (key == "ArrowUp" || key == "ArrowDown") => {
                    e.prevent_default();
                    if let Some(to) =
                        carried_target(&self.carried, self.index, &key, self.index, self.count)
                    {
                        self.drop_at(to);
                    }
                }
                None => {}
            }
        })
    }

    fn blur(self) -> Callback<FocusEvent> {
        Callback::from(move |_: FocusEvent| {
            let lift = self.live.borrow().lift;
            if lift.is_some_and(|l| !l.pointer && l.from == self.index) {
                self.cancel();
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn lifted_and_the_marks_are_modifiers() {
        assert_eq!(names(reorder_row(1, None)), [class::REORDER_ROW]);
        assert_eq!(
            names(reorder_row(1, Some((1, 1)))),
            [class::REORDER_ROW, class::LIFTED]
        );
        assert_eq!(
            names(reorder_row(0, Some((2, 0)))),
            [class::REORDER_ROW, class::DROP_BEFORE]
        );
        assert_eq!(
            names(reorder_row(3, Some((1, 3)))),
            [class::REORDER_ROW, class::DROP_AFTER]
        );
        assert_eq!(names(reorder_row(2, Some((1, 3)))), [class::REORDER_ROW]);
        assert_eq!(names(drag_handle(false)), [class::DRAG_HANDLE]);
        assert_eq!(
            names(drag_handle(true)),
            [class::DRAG_HANDLE, class::LIFTED]
        );
    }

    #[test]
    fn a_drop_lands_at_an_index_without_the_row() {
        // Slots either side of the lifted row are no move.
        assert_eq!(landing(2, 2), 2);
        assert_eq!(landing(2, 3), 2);
        assert_eq!(landing(2, 0), 0);
        assert_eq!(landing(0, 4), 3);

        let middles = [10.0, 30.0, 50.0, 70.0];
        assert_eq!(slot_at(&middles, 5.0), 0);
        assert_eq!(slot_at(&middles, 55.0), 3);
        assert_eq!(slot_at(&middles, 90.0), 4);
        assert_eq!(landing(0, slot_at(&middles, 10.0 + 65.0)), 3);

        for (from, to, order) in [
            (0, 3, ["b", "c", "d", "a"]),
            (3, 0, ["d", "a", "b", "c"]),
            (1, 2, ["a", "c", "b", "d"]),
            (2, 2, ["a", "b", "c", "d"]),
        ] {
            let mut list = vec!["a", "b", "c", "d"];
            move_item(&mut list, from, to);
            assert_eq!(list, order, "{from} to {to}");
        }
    }

    #[test]
    fn a_drag_kept_within_the_list_reaches_either_end() {
        // Rows 32 high, 48 apart.
        let middles = [16.0, 64.0, 112.0];
        let at = |from: usize, dy: f64| {
            let middle = middles[from] + dy;
            drag_target(&middles, from, middle - 16.0, middle + 16.0)
        };
        assert_eq!(at(0, 0.0), 0);
        assert_eq!(at(0, 31.0), 0);
        assert_eq!(at(0, 33.0), 1);
        // Clamped to the last row's place.
        assert_eq!(at(0, 96.0), 2);
        assert_eq!(at(2, -96.0), 0);
        assert_eq!(at(2, -31.0), 2);
        assert_eq!(at(1, 0.0), 1);
        assert_eq!(at(1, -48.0), 0);
        // A tall row (100 high) under a short one (20 high), clamped to the
        // first row's top, still passes it.
        let middles = [10.0, 74.0];
        assert_eq!(drag_target(&middles, 1, 0.0, 100.0), 0);
        assert_eq!(drag_target(&middles, 0, 104.0, 124.0), 1);
        assert_eq!(drag_target(&middles, 5, 0.0, 1.0), 5);
        // A row without a box is never passed.
        let middles = [16.0, 64.0, f64::NAN];
        assert_eq!(drag_target(&middles, 0, 200.0, 232.0), 1);
    }

    #[test]
    fn picked_rows_move_together() {
        for (rows, to, order) in [
            (&[1, 3][..], 0, ["b", "d", "a", "c", "e"]),
            (&[1, 3][..], 3, ["a", "c", "e", "b", "d"]),
            (&[0, 4][..], 1, ["b", "a", "e", "c", "d"]),
            (&[2, 3][..], 9, ["a", "b", "e", "c", "d"]),
        ] {
            let mut list = vec!["a", "b", "c", "d", "e"];
            move_items(&mut list, rows, to);
            assert_eq!(list, order, "{rows:?} to {to}");
        }
    }

    #[test]
    fn carried_rows_land_among_the_rows_left() {
        // Rows 1 and 3 picked, 3 lifted.
        let carried = [false, true, false, true, false];
        // Aimed before row 0, between 1 and 2, and at the end.
        assert_eq!(carried_landing(&carried, 3, 0), 0);
        assert_eq!(carried_landing(&carried, 3, 2), 1);
        assert_eq!(carried_landing(&carried, 3, 4), 3);
        // Where row 3 lies: they gather under row 2.
        assert_eq!(carried_landing(&carried, 3, 3), 2);
        assert!(carried_moves(&carried, 2));
        // Lying together at 1, landing there moves nothing.
        let together = [false, true, true, false];
        assert!(!carried_moves(&together, 1));
        assert!(carried_moves(&together, 0));
        assert!(carried_moves(&together, 2));
    }

    #[test]
    fn keys_step_carried_rows_past_the_gaps_that_move_nothing() {
        // Rows 2 and 3 picked, 3 focused: up passes 2, which goes with it,
        // and lands over row 1.
        let carried = [false, false, true, true, false];
        assert_eq!(carried_target(&carried, 3, "ArrowUp", 3, 5), Some(1));
        assert_eq!(carried_landing(&carried, 3, 1), 1);
        assert_eq!(carried_target(&carried, 3, "ArrowDown", 3, 5), Some(4));
        // From the top, up stays.
        assert_eq!(carried_target(&carried, 3, "ArrowUp", 0, 5), Some(0));
        // Alone, a row steps one place.
        assert_eq!(carried_target(&[false; 5], 3, "ArrowUp", 3, 5), Some(2));
    }

    #[test]
    fn a_grid_drag_lands_in_the_nearest_cell() {
        // Two rows of three cells, 50 apart across and 40 down.
        let centers = [
            (25.0, 20.0),
            (75.0, 20.0),
            (125.0, 20.0),
            (25.0, 60.0),
            (75.0, 60.0),
            (125.0, 60.0),
        ];
        assert_eq!(grid_target(&centers, 0, 25.0, 20.0), 0);
        assert_eq!(grid_target(&centers, 0, 70.0, 25.0), 1);
        assert_eq!(grid_target(&centers, 0, 125.0, 60.0), 5);
        assert_eq!(grid_target(&centers, 5, 20.0, 15.0), 0);
        assert_eq!(grid_target(&centers, 2, 30.0, 70.0), 3);
        assert_eq!(grid_target(&[], 3, 0.0, 0.0), 3);
        assert_eq!(
            grid_target(&[(25.0, 20.0), (f64::NAN, f64::NAN)], 0, 0.0, 0.0),
            0
        );
    }

    #[test]
    fn the_edges_scroll_faster_the_nearer_the_pointer() {
        assert_eq!(edge_step(300.0, 100.0, 500.0), 0.0);
        assert_eq!(edge_step(140.0, 100.0, 500.0), 0.0);
        assert_eq!(edge_step(100.0, 100.0, 500.0), -STEP);
        assert_eq!(edge_step(-20.0, 100.0, 500.0), -STEP);
        assert_eq!(edge_step(120.0, 100.0, 500.0), -STEP / 2.0);
        assert_eq!(edge_step(500.0, 100.0, 500.0), STEP);
        assert_eq!(edge_step(480.0, 100.0, 500.0), STEP / 2.0);
    }

    #[test]
    fn positions_count_only_the_rows_that_move() {
        let stays = [false, false, true, false];
        assert_eq!(place(&stays, 0, 0), (1, 3));
        assert_eq!(place(&stays, 1, 1), (2, 3));
        // Either side of the fixed row: last but one, or after it.
        assert_eq!(place(&stays, 0, 1), (2, 3));
        assert_eq!(place(&stays, 0, 2), (2, 3));
        assert_eq!(place(&stays, 0, 3), (3, 3));
        assert_eq!(place(&stays, 3, 0), (1, 3));
        assert_eq!(place(&[false; 4], 2, 3), (4, 4));
    }

    #[test]
    fn keys_aim_within_the_list() {
        assert_eq!(key_target("ArrowUp", 0, 4), Some(0));
        assert_eq!(key_target("ArrowDown", 2, 4), Some(3));
        assert_eq!(key_target("ArrowDown", 3, 4), Some(3));
        assert_eq!(key_target("Home", 2, 4), Some(0));
        assert_eq!(key_target("End", 0, 4), Some(3));
        assert_eq!(key_target("a", 0, 4), None);
        assert_eq!(key_target("End", 0, 0), None);
    }

    #[test]
    fn the_announcements_name_the_row_and_its_place() {
        let name = AttrValue::from("Alpha");
        assert_eq!(
            reorder_text(&ReorderText::Handle { name: name.clone() }),
            "Move Alpha"
        );
        assert_eq!(
            reorder_text(&ReorderText::Lifted {
                name: name.clone(),
                position: 1,
                count: 4
            }),
            "Alpha lifted, position 1 of 4"
        );
        assert_eq!(
            reorder_text(&ReorderText::Dropped {
                name,
                position: 3,
                count: 4
            }),
            "Moved Alpha to 3 of 4"
        );
    }

    #[test]
    fn the_announcements_are_in_the_provided_language() {
        use phrasebook02::{Lang, Phrasebook};

        let sv = Lang::new("sv-SE");
        let book = Phrasebook::builder()
            .language(Lang::EN_US, "English")
            .language(sv, "Svenska")
            .catalog(&L10N)
            .build();
        let text = ReorderText::Cancelled {
            name: "Alfa".into(),
            position: 2,
            count: 5,
        };
        assert_eq!(
            phrase(&book.phrases(sv), &text),
            "Alfa tillbaka på plats 2 av 5"
        );
    }
}
