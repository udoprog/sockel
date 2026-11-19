//! Popover (`style/_popover.scss`).

use crate::testid::with_testid;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, HtmlElement, Node};
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonSize, ButtonVariant, Glyph, Icon, class, use_overlays};

/// Between the anchor and the popover, in pixels.
const GAP: f64 = 4.0;
/// Between the popover and the viewport's edges, in pixels.
const EDGE: f64 = 8.0;

/// A box in viewport coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn right(&self) -> f64 {
        self.left + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.top + self.height
    }

    /// The part of this box inside `other` (empty when they do not meet).
    pub fn clip(&self, other: Rect) -> Rect {
        let left = self.left.max(other.left);
        let top = self.top.max(other.top);
        Rect {
            left,
            top,
            width: (self.right().min(other.right()) - left).max(0.0),
            height: (self.bottom().min(other.bottom()) - top).max(0.0),
        }
    }

    fn of(element: &Element) -> Self {
        let r = element.get_bounding_client_rect();
        Self {
            left: r.left(),
            top: r.top(),
            width: r.width(),
            height: r.height(),
        }
    }
}

/// Which side of its anchor a popover opens on; it flips to the opposite
/// side when that has more room.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Below,
    Above,
    /// Before the anchor (left, in a left-to-right page).
    Start,
    /// After the anchor.
    End,
}

impl Side {
    fn opposite(self) -> Self {
        match self {
            Side::Below => Side::Above,
            Side::Above => Side::Below,
            Side::Start => Side::End,
            Side::End => Side::Start,
        }
    }
}

/// How a popover lines up with its anchor along the side it opens on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverAlign {
    /// Their start (left or top) edges together.
    #[default]
    Start,
    Center,
    End,
}

/// Where a popover goes: its corner, the side it ended up on, and the most
/// it may be along the axis it opened in before it scrolls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub left: f64,
    pub top: f64,
    pub side: Side,
    pub max_height: f64,
}

impl Placed {
    fn near(&self, other: &Placed) -> bool {
        self.side == other.side
            && (self.left - other.left).abs() < 0.5
            && (self.top - other.top).abs() < 0.5
            && (self.max_height - other.max_height).abs() < 0.5
    }
}

/// What a popover too tall for both sides of its anchor does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fit {
    /// It takes the side with more room and scrolls in it.
    #[default]
    Scroll,
    /// It stays on its side and slides along it, over the anchor, until it
    /// fits the viewport; it scrolls only when taller than the viewport.
    Slide,
}

/// Places a popover of `size` (width, height) beside `anchor`, inside
/// `viewport`: on `side` unless it does not fit there and the opposite
/// side has more room, aligned by `align`, and moved back inside the
/// viewport where it would leave it. Its height is capped by the room on
/// the side it took, never by moving it over the anchor.
pub fn place(
    anchor: Rect,
    size: (f64, f64),
    viewport: Rect,
    side: Side,
    align: PopoverAlign,
) -> Placed {
    place_fit(anchor, size, viewport, side, align, Fit::Scroll)
}

/// [`place`], with `fit` deciding what a popover too tall for both sides
/// above and below its anchor does.
pub fn place_fit(
    anchor: Rect,
    size: (f64, f64),
    viewport: Rect,
    side: Side,
    align: PopoverAlign,
    fit: Fit,
) -> Placed {
    let (width, height) = size;
    let room = |side: Side| match side {
        Side::Below => viewport.bottom() - anchor.bottom() - GAP - EDGE,
        Side::Above => anchor.top - viewport.top - GAP - EDGE,
        Side::Start => anchor.left - viewport.left - GAP - EDGE,
        Side::End => viewport.right() - anchor.right() - GAP - EDGE,
    };
    let need = match side {
        Side::Below | Side::Above => height,
        Side::Start | Side::End => width,
    };
    let slides = fit == Fit::Slide
        && matches!(side, Side::Below | Side::Above)
        && need > room(side)
        && need > room(side.opposite());
    let side = if !slides && need > room(side) && room(side.opposite()) > room(side) {
        side.opposite()
    } else {
        side
    };

    // Lined up along the cross axis, then kept inside the viewport.
    let along = |start: f64, length: f64, own: f64, low: f64, high: f64| {
        let at = match align {
            PopoverAlign::Start => start,
            PopoverAlign::Center => start + (length - own) / 2.0,
            PopoverAlign::End => start + length - own,
        };
        at.min(high - EDGE - own).max(low + EDGE)
    };

    match side {
        Side::Below | Side::Above => {
            let max_height = if slides {
                (viewport.height - 2.0 * EDGE).max(0.0)
            } else {
                room(side).max(0.0)
            };
            let shown = height.min(max_height);
            let top = match (side, slides) {
                (Side::Below, false) => anchor.bottom() + GAP,
                (_, false) => anchor.top - GAP - shown,
                (Side::Below, true) => viewport.bottom() - EDGE - shown,
                (_, true) => viewport.top + EDGE,
            };
            let left = along(
                anchor.left,
                anchor.width,
                width,
                viewport.left,
                viewport.right(),
            );
            Placed {
                left,
                top,
                side,
                max_height,
            }
        }
        Side::Start | Side::End => {
            let max_height = (viewport.height - 2.0 * EDGE).max(0.0);
            let shown = height.min(max_height);
            let left = if side == Side::End {
                anchor.right() + GAP
            } else {
                anchor.left - GAP - width
            };
            let left = left
                .min(viewport.right() - EDGE - width)
                .max(viewport.left + EDGE);
            let top = along(
                anchor.top,
                anchor.height,
                shown,
                viewport.top,
                viewport.bottom(),
            );
            Placed {
                left,
                top,
                side,
                max_height,
            }
        }
    }
}

/// The room an open popover keeps its side with: content that grows past
/// its side's room scrolls there rather than flipping the popover across
/// its anchor, while the side has at least this much (or all it needs).
const KEEP_ROOM: f64 = 200.0;

/// [`place_fit`] for a popover already open on `kept`: it stays on that
/// side (above or below) while the side has `KEEP_ROOM` or all it needs,
/// capped to the room there, so growing content never makes it jump.
fn place_kept(
    anchor: Rect,
    size: (f64, f64),
    viewport: Rect,
    side: Side,
    align: PopoverAlign,
    fit: Fit,
    kept: Option<Side>,
) -> Placed {
    let (width, height) = size;
    let room = match kept {
        Some(Side::Below) => Some(viewport.bottom() - anchor.bottom() - GAP - EDGE),
        Some(Side::Above) => Some(anchor.top - viewport.top - GAP - EDGE),
        _ => None,
    };
    match (kept, room) {
        (Some(kept), Some(room)) if room >= height.min(KEEP_ROOM) => place_fit(
            anchor,
            (width, height.min(room)),
            viewport,
            kept,
            align,
            fit,
        ),
        _ => place_fit(anchor, size, viewport, side, align, fit),
    }
}

/// Why a popover asks its owner to close it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dismiss {
    /// A press outside it and its anchor: focus stays where the press put it.
    Outside,
    /// Escape: focus goes back to the anchor.
    Escape,
    /// The sheet's close button: focus goes back to the anchor.
    Close,
}

impl Dismiss {
    /// Whether closing for this reason hands focus back to the anchor.
    pub fn returns_focus(self) -> bool {
        self != Dismiss::Outside
    }

    /// The dismissal a key asks for.
    pub fn for_key(key: &str) -> Option<Self> {
        (key == "Escape").then_some(Dismiss::Escape)
    }

    /// The dismissal a press asks for, by whether it fell inside the anchor
    /// or the popover.
    pub fn for_press(in_anchor: bool, in_popover: bool) -> Option<Self> {
        (!in_anchor && !in_popover).then_some(Dismiss::Outside)
    }
}

/// An open popover's Escape, for the `onkeydown` of its anchor and surface:
/// yew runs the app's handlers on their ancestors before the document
/// hears the key, so Escape is taken there and its default prevented,
/// letting those handlers check `defaultPrevented`.
#[derive(Clone, Default)]
pub(crate) struct Escape(Rc<RefCell<Option<Keyed>>>);

type Keyed = Rc<dyn Fn(&web_sys::Event)>;

impl Escape {
    pub(crate) fn on(&self, e: &KeyboardEvent) {
        let keyed = self.0.borrow().clone();
        if let Some(keyed) = keyed {
            keyed(e);
        }
    }

    /// `f`, then Escape unless `f` took the key.
    pub(crate) fn after(&self, f: Callback<KeyboardEvent>) -> Callback<KeyboardEvent> {
        let escape = self.clone();
        Callback::from(move |e: KeyboardEvent| {
            f.emit(e.clone());
            escape.on(&e);
        })
    }
}

/// A listener added to a target, removed when dropped.
pub(crate) struct Listener {
    target: web_sys::EventTarget,
    kind: &'static str,
    capture: bool,
    closure: Closure<dyn Fn(web_sys::Event)>,
}

impl Listener {
    pub(crate) fn new(
        target: web_sys::EventTarget,
        kind: &'static str,
        capture: bool,
        f: impl Fn(web_sys::Event) + 'static,
    ) -> Self {
        Self::with(target, kind, capture, true, f)
    }

    /// Not passive, so `f` can prevent the default (Tab kept in a dialog).
    pub(crate) fn active(
        target: web_sys::EventTarget,
        kind: &'static str,
        capture: bool,
        f: impl Fn(web_sys::Event) + 'static,
    ) -> Self {
        Self::with(target, kind, capture, false, f)
    }

    fn with(
        target: web_sys::EventTarget,
        kind: &'static str,
        capture: bool,
        passive: bool,
        f: impl Fn(web_sys::Event) + 'static,
    ) -> Self {
        let closure = Closure::<dyn Fn(web_sys::Event)>::new(f);
        let options = web_sys::AddEventListenerOptions::new();
        options.set_capture(capture);
        options.set_passive(passive);
        _ = target.add_event_listener_with_callback_and_add_event_listener_options(
            kind,
            closure.as_ref().unchecked_ref(),
            &options,
        );
        Self {
            target,
            kind,
            capture,
            closure,
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        _ = self.target.remove_event_listener_with_callback_and_bool(
            self.kind,
            self.closure.as_ref().unchecked_ref(),
            self.capture,
        );
    }
}

/// What the viewport lets a fixed box use: without a scrollbar.
/// The ancestors of `element` whose overflow clips it.
fn clippers(element: &Element) -> Vec<Element> {
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    let style = |e: &Element, property: &str| {
        window
            .get_computed_style(e)
            .ok()
            .flatten()
            .and_then(|s| s.get_property_value(property).ok())
            .unwrap_or_default()
    };
    let mut chain = Vec::new();
    let mut at = Some(element.clone());
    while let Some(e) = at {
        if e.tag_name() == "BODY" || e.tag_name() == "HTML" {
            break;
        }
        let clips = ["overflow-x", "overflow-y"]
            .iter()
            .any(|p| !matches!(style(&e, p).as_str(), "" | "visible"));
        let fixed = style(&e, "position") == "fixed";
        at = e.parent_element();
        chain.push((e, Clip { clips, fixed }));
    }
    let keep = clipping(&chain.iter().map(|(_, c)| *c).collect::<Vec<_>>());
    chain
        .into_iter()
        .enumerate()
        .filter(|(i, _)| keep.contains(i))
        .map(|(_, (e, _))| e)
        .collect()
}

/// One box from an element up through its ancestors.
#[derive(Clone, Copy)]
struct Clip {
    clips: bool,
    fixed: bool,
}

/// Which of `chain` (the element first, then its ancestors) clip the
/// element: those with overflow below the first fixed box, which leaves
/// the ones above it behind.
fn clipping(chain: &[Clip]) -> Vec<usize> {
    let mut found = Vec::new();
    for (i, c) in chain.iter().enumerate() {
        if i > 0 && c.clips {
            found.push(i);
        }
        if c.fixed {
            break;
        }
    }
    found
}

/// Whether `anchor` is out of sight: not laid out, or wholly outside one
/// of the boxes clipping it.
fn out_of_sight(anchor: &Element, clippers: &[Element]) -> bool {
    if anchor.get_client_rects().length() == 0 {
        return true;
    }
    let at = Rect::of(anchor);
    clippers.iter().any(|c| {
        let b = Rect::of(c);
        at.bottom() <= b.top || at.top >= b.bottom() || at.right() <= b.left || at.left >= b.right()
    })
}

/// The box a popover beside `anchor` is placed and kept inside: the
/// viewport, cut to `anchor`'s closest ancestor matching the selector
/// `within` when it has one.
pub fn placement_bounds(anchor: &Element, within: Option<&str>) -> Option<Rect> {
    let viewport = viewport()?;
    Some(
        match within.and_then(|w| anchor.closest(w).ok().flatten()) {
            Some(boxed) => viewport.clip(Rect::of(&boxed)),
            None => viewport,
        },
    )
}

fn viewport() -> Option<Rect> {
    let root = web_sys::window()?.document()?.document_element()?;
    Some(Rect {
        left: 0.0,
        top: 0.0,
        width: f64::from(root.client_width()),
        height: f64::from(root.client_height()),
    })
}

/// Whether a popover closing with focus nowhere hands it back to its
/// anchor: as its dismissal says (`Escape` and `Close` do, `Outside` does
/// not), else when its owner closed it only if focus or a press was ever
/// inside it, so a card that only the pointer opened and left (a hover
/// card) does not take focus to its anchor as it goes.
pub(crate) fn hands_back(dismissed: Option<bool>, inside: bool) -> bool {
    dismissed.unwrap_or(inside)
}

/// Whether focus is nowhere: on `<body>`, where it falls when the element
/// that had it goes.
pub(crate) fn focus_lost() -> bool {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return false;
    };
    match (document.active_element(), document.body()) {
        (None, _) => true,
        (Some(active), Some(body)) => {
            AsRef::<Node>::as_ref(&active).is_same_node(Some(body.as_ref()))
        }
        _ => false,
    }
}

/// Places `popover` beside `anchor` while `open`, measured before it is
/// shown, and again on scroll, resize and a change of either's size. A
/// press outside both, or Escape, asks `on_dismiss` to close it, unless an
/// overlay opened after it (a dialog from it) is still open; when it
/// closes, focus left nowhere goes back to the anchor after Escape or the
/// sheet's close, and after its owner closed it if focus or a press was
/// inside it (not after a press outside, nor for a card the pointer only
/// passed over). `None` until it is placed.
#[hook]
pub fn use_popover(
    open: bool,
    anchor: NodeRef,
    popover: NodeRef,
    side: Side,
    align: PopoverAlign,
    on_dismiss: Callback<Dismiss>,
) -> Option<Placed> {
    use_placement(
        open,
        Anchor::new(anchor, None),
        popover,
        side,
        align,
        Fit::Scroll,
        None,
        on_dismiss,
        false,
    )
    .0
}

/// `use_popover` kept inside `anchor`'s closest ancestor matching the
/// selector `within` (see [`placement_bounds`]), not the viewport alone.
#[hook]
#[allow(clippy::too_many_arguments)]
pub fn use_popover_within(
    open: bool,
    anchor: NodeRef,
    popover: NodeRef,
    side: Side,
    align: PopoverAlign,
    within: Option<AttrValue>,
    on_dismiss: Callback<Dismiss>,
) -> Option<Placed> {
    use_placement(
        open,
        Anchor::new(anchor, None),
        popover,
        side,
        align,
        Fit::Scroll,
        within,
        on_dismiss,
        false,
    )
    .0
}

/// `use_popover` for a tip: it never keeps the overlay beneath it from
/// being dismissed.
#[hook]
pub(crate) fn use_tip_popover(
    open: bool,
    anchor: NodeRef,
    popover: NodeRef,
    side: Side,
    align: PopoverAlign,
    on_dismiss: Callback<Dismiss>,
) -> Option<Placed> {
    use_placement(
        open,
        Anchor::new(anchor, None),
        popover,
        side,
        align,
        Fit::Scroll,
        None,
        on_dismiss,
        true,
    )
    .0
}

/// What a popover stands beside: its owner's node, or an element the owner
/// did not render (the button just pressed).
#[derive(Clone, PartialEq)]
struct Anchor {
    node: NodeRef,
    element: Option<Element>,
}

impl Anchor {
    fn new(node: NodeRef, element: Option<Element>) -> Self {
        Self { node, element }
    }

    fn element(&self) -> Option<Element> {
        self.element.clone().or_else(|| self.node.cast::<Element>())
    }

    fn node(&self) -> Option<Node> {
        self.element().map(Node::from)
    }
}

#[hook]
#[allow(clippy::too_many_arguments)]
fn use_placement(
    open: bool,
    anchor: Anchor,
    popover: NodeRef,
    side: Side,
    align: PopoverAlign,
    fit: Fit,
    within: Option<AttrValue>,
    on_dismiss: Callback<Dismiss>,
    tip: bool,
) -> (Option<Placed>, Escape) {
    let placed = use_state_eq(|| None::<Placed>);
    let escape = use_memo((), |()| Escape::default());
    let overlays = use_overlays();
    // The latest callback, read by listeners made when it opened.
    let latest = use_mut_ref(Callback::noop);
    *latest.borrow_mut() = on_dismiss;
    let on_dismiss = latest;

    {
        let placed = placed.clone();
        let escape = (*escape).clone();
        let deps = (open, side, align, fit, within, anchor.element.clone());
        use_effect_with(deps, move |(open, side, align, fit, within, _)| {
            let (open, side, align, fit) = (*open, *side, *align, *fit);
            let within = within.clone();
            // Why it was dismissed (whether that returns focus), and
            // whether focus or a press was ever inside it.
            let refocus = Rc::new(Cell::new(None::<bool>));
            let inside = Rc::new(Cell::new(false));
            let mut held: Vec<Listener> = Vec::new();
            let mut observer = None;
            // What had focus inside the anchor (the button of a row it
            // lines up with), to hand focus back to rather than the anchor.
            let mut opener = None;

            if open {
                opener = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                    .filter(|a| anchor.node().is_some_and(|n| n.contains(Some(a))))
                    .and_then(|a| a.dyn_into::<HtmlElement>().ok());
                if let Some(element) = popover.cast::<Element>() {
                    for kind in ["focusin", "pointerdown"] {
                        let inside = inside.clone();
                        held.push(Listener::new(
                            element.clone().into(),
                            kind,
                            true,
                            move |_| inside.set(true),
                        ));
                    }
                }
                let layer = Rc::new(if tip {
                    overlays.tip_layer()
                } else {
                    overlays.layer()
                });
                let last = Rc::new(Cell::new(None::<Placed>));
                let clipped = anchor.element().map(|a| clippers(&a)).unwrap_or_default();
                let measure = {
                    let (anchor, popover, placed) =
                        (anchor.clone(), popover.clone(), placed.clone());
                    let (on_dismiss, refocus) = (on_dismiss.clone(), refocus.clone());
                    let clipped = clipped.clone();
                    let gone = Rc::new(Cell::new(false));
                    Rc::new(move || {
                        let (Some(at), Some(pop)) =
                            (anchor.element(), popover.cast::<HtmlElement>())
                        else {
                            return;
                        };
                        let Some(bounds) = placement_bounds(&at, within.as_deref()) else {
                            return;
                        };
                        // Its anchor scrolled away or hidden: nothing to
                        // stand beside, so it goes.
                        if out_of_sight(&at, &clipped) {
                            if !gone.replace(true) {
                                refocus.set(Some(false));
                                on_dismiss.borrow().emit(Dismiss::Outside);
                            }
                            return;
                        }
                        // Its natural height, whatever max-height cuts it to,
                        // with its border (scroll_height leaves it out).
                        let border = f64::from(pop.offset_height() - pop.client_height());
                        let size = (
                            f64::from(pop.offset_width()),
                            f64::from(pop.scroll_height()) + border,
                        );
                        let kept = last.get().map(|l| l.side);
                        let next = place_kept(Rect::of(&at), size, bounds, side, align, fit, kept);
                        if last.get().is_none_or(|l| !l.near(&next)) {
                            last.set(Some(next));
                            placed.set(Some(next));
                        }
                    })
                };
                measure();

                if let Some(window) = web_sys::window() {
                    for kind in ["scroll", "resize"] {
                        let measure = measure.clone();
                        held.push(Listener::new(
                            window.clone().into(),
                            kind,
                            true,
                            move |_| measure(),
                        ));
                    }

                    if let Some(document) = window.document() {
                        let within = |node: Option<Node>, event: &web_sys::Event| {
                            let target = event.target().and_then(|t| t.dyn_into::<Node>().ok());
                            node.is_some_and(|n| n.contains(target.as_ref()))
                        };
                        let pressed = {
                            let (anchor, popover) = (anchor.clone(), popover.clone());
                            let (on_dismiss, refocus) = (on_dismiss.clone(), refocus.clone());
                            let layer = layer.clone();
                            move |e: web_sys::Event| {
                                if !layer.is_top() {
                                    return;
                                }
                                if let Some(why) = Dismiss::for_press(
                                    within(anchor.node(), &e),
                                    within(popover.get(), &e),
                                ) {
                                    refocus.set(Some(why.returns_focus()));
                                    on_dismiss.borrow().emit(why);
                                }
                            }
                        };
                        held.push(Listener::new(
                            document.clone().into(),
                            "pointerdown",
                            true,
                            pressed,
                        ));
                        let keyed: Keyed = {
                            let (on_dismiss, refocus) = (on_dismiss.clone(), refocus.clone());
                            let layer = layer.clone();
                            Rc::new(move |e: &web_sys::Event| {
                                // Not when what has the focus took the key
                                // (a search field clearing its text).
                                if !layer.is_top() || e.default_prevented() {
                                    return;
                                }
                                // Only keydown reaches here; unchecked, as an
                                // event from another frame fails `instanceof`.
                                let key = e.unchecked_ref::<web_sys::KeyboardEvent>().key();
                                if let Some(why) = Dismiss::for_key(&key) {
                                    e.prevent_default();
                                    refocus.set(Some(why.returns_focus()));
                                    on_dismiss.borrow().emit(why);
                                }
                            })
                        };
                        *escape.0.borrow_mut() = Some(keyed.clone());
                        held.push(Listener::active(
                            document.into(),
                            "keydown",
                            false,
                            move |e| keyed(&e),
                        ));
                    }
                }

                let resized = {
                    let measure = measure.clone();
                    Closure::<dyn Fn()>::new(move || measure())
                };
                if let Ok(watch) = web_sys::ResizeObserver::new(resized.as_ref().unchecked_ref()) {
                    for element in [anchor.element(), popover.cast::<Element>()]
                        .into_iter()
                        .flatten()
                    {
                        watch.observe(&element);
                    }
                    for element in &clipped {
                        watch.observe(element);
                    }
                    observer = Some((watch, resized));
                }
            } else {
                placed.set(None);
            }

            move || {
                escape.0.borrow_mut().take();
                drop(held);
                if let Some((watch, _)) = observer {
                    watch.disconnect();
                }
                // Unmounted by its owner, it still holds the focus it is
                // about to drop.
                let holds = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                    .is_some_and(|a| popover.get().is_some_and(|p| p.contains(Some(&a))));
                if open
                    && hands_back(refocus.get(), inside.get())
                    && (holds || focus_lost())
                    && let Some(target) = opener.filter(|o| o.is_connected()).or_else(|| {
                        anchor
                            .element()
                            .and_then(|a| a.dyn_into::<HtmlElement>().ok())
                    })
                {
                    // Where the page stands: the anchor may be scrolled
                    // away under a sheet.
                    let options = web_sys::FocusOptions::new();
                    options.set_prevent_scroll(true);
                    _ = target.focus_with_options(&options);
                }
            }
        });
    }

    (*placed, (*escape).clone())
}

/// `use_popover` with its [`Escape`], for the components' own anchors and
/// surfaces.
#[hook]
#[allow(clippy::too_many_arguments)]
pub(crate) fn use_popover_keyed(
    open: bool,
    anchor: NodeRef,
    popover: NodeRef,
    side: Side,
    align: PopoverAlign,
    within: Option<AttrValue>,
    on_dismiss: Callback<Dismiss>,
) -> (Option<Placed>, Escape) {
    use_placement(
        open,
        Anchor::new(anchor, None),
        popover,
        side,
        align,
        Fit::Scroll,
        within,
        on_dismiss,
        false,
    )
}

#[derive(Properties, PartialEq)]
pub struct PopoverProps {
    /// What it opens beside, and where focus returns when it closes.
    #[prop_or_default]
    pub anchor: NodeRef,
    /// An element in place of `anchor`'s, for one its owner did not render
    /// (the button just pressed, a menu's button).
    #[prop_or_default]
    pub anchor_element: Option<Element>,
    pub open: bool,
    /// Asked to close it (the owner sets `open` to false).
    pub on_dismiss: Callback<Dismiss>,
    #[prop_or_default]
    pub side: Side,
    #[prop_or_default]
    pub align: PopoverAlign,
    /// Too tall for both sides above and below its anchor: scroll on the
    /// roomier side (the default) or slide over the anchor to fit.
    #[prop_or_default]
    pub fit: Fit,
    /// A selector: keep it inside the anchor's closest ancestor matching
    /// it (a window of the app's), not the viewport alone.
    #[prop_or_default]
    pub within: Option<AttrValue>,
    /// A sheet from the bottom below the phone breakpoint, over a scrim
    /// (`.sockel-sheet`); on by default.
    #[prop_or(true)]
    pub sheet: bool,
    /// The sheet's head, its title and close button; on by default.
    #[prop_or(true)]
    pub head: bool,
    /// Tab stays inside it, as in a dialog: for one that asks for an
    /// answer, such as a confirm.
    #[prop_or_default]
    pub trap: bool,
    /// A body without padding (`sockel::class::FLUSH`), for content that
    /// lays out its own edges, such as a list running to the surface's.
    #[prop_or_default]
    pub flush: bool,
    /// At least as wide as its anchor (`sockel::class::MATCH_ANCHOR`, the
    /// anchor's width as `--popover-anchor-width`), as a field's list under
    /// it; off by default. A sheet stays the screen's width.
    #[prop_or_default]
    pub match_anchor: bool,
    /// A press inside it, other than on a control (a field, button, link or
    /// anything focusable), keeps focus where it is: a search field that
    /// keeps the keyboard while its results are clicked; off by default.
    #[prop_or_default]
    pub keep_focus: bool,
    /// Its accessible name, and the sheet's heading.
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The sheet's close button's accessible name, else the catalog's.
    #[prop_or_default]
    pub close_label: Option<AttrValue>,
    /// Takes focus once it is placed (it is hidden until then), such as the
    /// safe button of a confirm.
    #[prop_or_default]
    pub focus: Option<NodeRef>,
    /// Without [`PopoverProps::focus`], focus moves once it is placed to an
    /// `[autofocus]` or `[data-autofocus]` element, else its first control,
    /// else the surface, as in a `Dialog`: for a popover of fields or
    /// choices opened from the keyboard.
    #[prop_or_default]
    pub focus_first: bool,
    /// Emitted once it has been placed and is shown, for an owner that
    /// scrolls its content to a row only then (before, it has no height
    /// bound to scroll in): a list opened at the current choice.
    #[prop_or_default]
    pub on_placed: Option<Callback<()>>,
    /// Its ARIA role, `dialog` by default: `tooltip` for a description shown
    /// on hover or focus, which takes no focus and is no dialog.
    #[prop_or(AttrValue::Static("dialog"))]
    pub role: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The pointer entering and leaving the surface, for an owner that keeps
    /// it open while the pointer is over it (a card opened on hover): the
    /// surface is in the overlay layer, so the owner's own `pointerleave`
    /// fires as the pointer moves onto it.
    #[prop_or_default]
    pub on_pointer_enter: Option<Callback<PointerEvent>>,
    #[prop_or_default]
    pub on_pointer_leave: Option<Callback<PointerEvent>>,
    #[prop_or_default]
    pub children: Html,
}

impl Default for PopoverProps {
    fn default() -> Self {
        Self {
            anchor: Default::default(),
            anchor_element: Default::default(),
            open: Default::default(),
            on_dismiss: Default::default(),
            side: Default::default(),
            align: Default::default(),
            fit: Default::default(),
            within: Default::default(),
            sheet: ::yew::html::IntoPropValue::into_prop_value(true),
            head: ::yew::html::IntoPropValue::into_prop_value(true),
            trap: Default::default(),
            flush: Default::default(),
            match_anchor: Default::default(),
            keep_focus: Default::default(),
            title: Default::default(),
            close_label: Default::default(),
            focus: Default::default(),
            focus_first: Default::default(),
            on_placed: Default::default(),
            role: ::yew::html::IntoPropValue::into_prop_value(AttrValue::Static("dialog")),
            class: Default::default(),
            testid: Default::default(),
            on_pointer_enter: Default::default(),
            on_pointer_leave: Default::default(),
            children: Default::default(),
        }
    }
}

/// What `focus_first` focuses in `surface`: an `[autofocus]` or
/// `[data-autofocus]` element, else the body's first control, else the
/// surface itself.
fn first_of(surface: &HtmlElement) -> HtmlElement {
    surface
        .query_selector("[autofocus], [data-autofocus]")
        .ok()
        .flatten()
        .and_then(|e| e.dyn_into::<HtmlElement>().ok())
        .or_else(|| {
            let body = surface
                .query_selector(&format!(".{}", class::POPOVER_BODY))
                .ok()??;
            crate::dialog::focusable(&body).into_iter().next()
        })
        .unwrap_or_else(|| surface.clone())
}

/// What a press in a `keep_focus` popover may still focus.
const KEEPS_PRESS: &str = "input, button, textarea, select, a[href], [tabindex], [contenteditable]";

/// A popover's classes: `popover`, `sheet` when it becomes one on a phone,
/// and `placed` once it has been measured and placed (it is hidden until
/// then).
pub fn popover(sheet: bool, placed: bool) -> Classes {
    classes!(
        class::POPOVER,
        sheet.then_some(class::SHEET),
        placed.then_some(class::PLACED)
    )
}

/// A surface beside its anchor in the overlay layer, while `open`: placed
/// before it is seen, flipped and kept inside the viewport, following its
/// anchor; dismissed by a press outside or Escape; a sheet on a phone.
#[function_component]
pub fn Popover(props: &PopoverProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let node = use_node_ref();
    let overlays = use_overlays();
    let anchor = Anchor::new(props.anchor.clone(), props.anchor_element.clone());
    let (placed, escape) = use_placement(
        props.open,
        anchor.clone(),
        node.clone(),
        props.side,
        props.align,
        props.fit,
        props.within.clone(),
        props.on_dismiss.clone(),
        false,
    );
    {
        let (focus, first, node) = (props.focus.clone(), props.focus_first, node.clone());
        let on_placed = props.on_placed.clone();
        use_effect_with(placed.is_some(), move |&placed| {
            if !placed {
                return;
            }
            if let Some(on_placed) = &on_placed {
                on_placed.emit(());
            }
            let target = match focus {
                Some(focus) => focus.cast::<HtmlElement>(),
                None if first => node.cast::<HtmlElement>().map(|surface| first_of(&surface)),
                None => None,
            };
            if let Some(target) = target {
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                _ = target.focus_with_options(&options);
            }
        });
    }

    if !props.open {
        return Html::default();
    }

    let mut style = placed.map(|p| {
        format!(
            "--popover-left: {}px; --popover-top: {}px; --popover-max-height: {}px;",
            p.left, p.top, p.max_height
        )
    });
    if props.match_anchor
        && let Some(anchor) = anchor
            .element()
            .and_then(|a| a.dyn_into::<HtmlElement>().ok())
    {
        let width = format!("--popover-anchor-width: {}px;", anchor.offset_width());
        style = Some(match style {
            Some(style) => format!("{style} {width}"),
            None => width,
        });
    }
    let onmousedown = props.keep_focus.then(|| {
        let node = node.clone();
        Callback::from(move |e: MouseEvent| {
            // Not the surface itself, focusable for `focus_first`.
            let surface = node.cast::<Element>();
            let on_control = e
                .target()
                .and_then(|t| t.dyn_into::<Element>().ok())
                .and_then(|t| t.closest(KEEPS_PRESS).ok().flatten())
                .is_some_and(|c| Some(&c) != surface.as_ref());
            if !on_control {
                e.prevent_default();
            }
        })
    });
    let onkeydown = {
        let (node, trap) = (node.clone(), props.trap);
        Callback::from(move |e: KeyboardEvent| {
            if trap
                && e.key() == "Tab"
                && let Some(root) = node.cast::<Element>()
            {
                crate::dialog::wrap_tab(&root, &e);
            }
            escape.on(&e);
        })
    };
    let head = (props.sheet && props.head && props.title.is_some()).then(|| {
        let close = props.on_dismiss.reform(|_: MouseEvent| Dismiss::Close);
        html! {
            <div class={class::POPOVER_HEAD}>
                <span class={class::POPOVER_TITLE}>{ props.title.clone() }</span>
                <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                    icon={html! { <Icon glyph={Glyph::Close} /> }}
                    label={label(&phrases, &props.close_label, "sockel-popover-close")} onclick={close} />
            </div>
        }
    });

    overlays.portal(html! {
        <>
            if props.sheet {
                <div class={class::POPOVER_SCRIM} aria-hidden="true"></div>
            }
            { with_testid(html! { <div ref={node}
                class={classes!(
                    popover(props.sheet, placed.is_some()),
                    props.match_anchor.then_some(class::MATCH_ANCHOR),
                    props.class.clone()
                )}
                role={props.role.clone()} aria-label={props.title.clone()} {style} {onkeydown} {onmousedown}
                tabindex={props.focus_first.then_some("-1")}
                onpointerenter={props.on_pointer_enter.clone()}
                onpointerleave={props.on_pointer_leave.clone()}>
                { for head }
                <div class={classes!(class::POPOVER_BODY, props.flush.then_some(class::FLUSH))}>
                    { props.children.clone() }
                </div>
            </div> }, props.testid.clone()) }
        </>
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT: Rect = Rect {
        left: 0.0,
        top: 0.0,
        width: 800.0,
        height: 600.0,
    };

    fn anchor(left: f64, top: f64) -> Rect {
        Rect {
            left,
            top,
            width: 100.0,
            height: 30.0,
        }
    }

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_fixed_box_escapes_the_scrollers_around_it() {
        let c = |clips, fixed| Clip { clips, fixed };
        let plain = [
            c(false, false),
            c(true, false),
            c(false, false),
            c(true, false),
        ];
        assert_eq!(clipping(&plain), [1, 3]);
        let anchor_fixed = [c(false, true), c(true, false), c(true, false)];
        assert!(clipping(&anchor_fixed).is_empty());
        let fixed_between = [
            c(false, false),
            c(true, false),
            c(false, true),
            c(true, false),
        ];
        assert_eq!(clipping(&fixed_between), [1]);
        assert!(clipping(&[c(true, false)]).is_empty());
    }

    #[test]
    fn it_opens_below_aligned_to_the_anchor() {
        let p = place(
            anchor(100.0, 100.0),
            (200.0, 150.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::Start,
        );
        assert_eq!((p.left, p.top, p.side), (100.0, 134.0, Side::Below));
        assert_eq!(p.max_height, 600.0 - 130.0 - GAP - EDGE);

        let end = place(
            anchor(300.0, 100.0),
            (200.0, 150.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::End,
        );
        assert_eq!(end.left, 200.0);
        let center = place(
            anchor(300.0, 100.0),
            (200.0, 150.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::Center,
        );
        assert_eq!(center.left, 250.0);
    }

    #[test]
    fn an_open_popover_keeps_its_side_as_it_grows() {
        // Opened below with room for it; grown past the room below, it
        // stays there, capped, rather than flipping above.
        let at = anchor(100.0, 300.0);
        let room = 600.0 - 330.0 - GAP - EDGE;
        let p = place_kept(
            at,
            (200.0, 280.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::Start,
            Fit::Scroll,
            Some(Side::Below),
        );
        assert_eq!(p.side, Side::Below);
        assert_eq!(p.max_height, room);
        assert_eq!(p.top, 330.0 + GAP);

        // Unopened, the same size flips.
        let p = place(
            at,
            (200.0, 280.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::Start,
        );
        assert_eq!(p.side, Side::Above);

        // Too little room left on its side (scrolled near the edge): flips.
        let p = place_kept(
            anchor(100.0, 500.0),
            (200.0, 280.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::Start,
            Fit::Scroll,
            Some(Side::Below),
        );
        assert_eq!(p.side, Side::Above);
    }

    #[test]
    fn it_flips_only_when_the_other_side_has_more_room() {
        // Near the bottom: flips above, and ends a gap above the anchor.
        let p = place(
            anchor(100.0, 500.0),
            (200.0, 150.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::Start,
        );
        assert_eq!(p.side, Side::Above);
        assert_eq!(p.top, 500.0 - GAP - 150.0);

        // Too tall for either side: stays where there is more room, capped.
        let p = place(
            anchor(100.0, 200.0),
            (200.0, 900.0),
            VIEWPORT,
            Side::Above,
            PopoverAlign::Start,
        );
        assert_eq!(p.side, Side::Below);
        assert_eq!(p.max_height, 600.0 - 230.0 - GAP - EDGE);
        assert_eq!(p.top, 230.0 + GAP);

        // Sideways: flips to the other side of the anchor.
        let p = place(
            anchor(680.0, 100.0),
            (200.0, 100.0),
            VIEWPORT,
            Side::End,
            PopoverAlign::Start,
        );
        assert_eq!(p.side, Side::Start);
        assert_eq!(p.left, 680.0 - GAP - 200.0);
    }

    #[test]
    fn sliding_keeps_its_side_and_fits_over_the_anchor() {
        let at = |top, height, side, fit| {
            place_fit(
                anchor(100.0, top),
                (200.0, height),
                VIEWPORT,
                side,
                PopoverAlign::Start,
                fit,
            )
        };
        // Too tall for either side: slid up from below to the bottom edge.
        let p = at(200.0, 500.0, Side::Below, Fit::Slide);
        assert_eq!((p.side, p.top), (Side::Below, 600.0 - EDGE - 500.0));
        assert_eq!(p.max_height, 600.0 - 2.0 * EDGE);
        // From above, down to the top edge.
        let p = at(400.0, 500.0, Side::Above, Fit::Slide);
        assert_eq!((p.side, p.top), (Side::Above, EDGE));
        // Taller than the viewport: at the edge, capped to it.
        let p = at(200.0, 900.0, Side::Below, Fit::Slide);
        assert_eq!((p.top, p.max_height), (EDGE, 600.0 - 2.0 * EDGE));
        // Fitting a side, it is placed as `place` places it.
        for (top, side) in [
            (100.0, Side::Below),
            (500.0, Side::Below),
            (300.0, Side::Above),
        ] {
            assert_eq!(
                at(top, 150.0, side, Fit::Slide),
                at(top, 150.0, side, Fit::Scroll)
            );
        }
        assert_eq!(
            at(200.0, 500.0, Side::Below, Fit::Scroll),
            place(
                anchor(100.0, 200.0),
                (200.0, 500.0),
                VIEWPORT,
                Side::Below,
                PopoverAlign::Start
            )
        );
        // Sideways, it is never slid over the anchor.
        assert_eq!(
            place_fit(
                anchor(300.0, 100.0),
                (500.0, 100.0),
                VIEWPORT,
                Side::End,
                PopoverAlign::Start,
                Fit::Slide
            ),
            place(
                anchor(300.0, 100.0),
                (500.0, 100.0),
                VIEWPORT,
                Side::End,
                PopoverAlign::Start
            )
        );
    }

    #[test]
    fn a_box_within_bounds_it_to_their_overlap() {
        let window = Rect {
            left: 500.0,
            top: -50.0,
            width: 400.0,
            height: 300.0,
        };
        assert_eq!(
            VIEWPORT.clip(window),
            Rect {
                left: 500.0,
                top: 0.0,
                width: 300.0,
                height: 250.0
            }
        );
        let apart = Rect {
            left: 900.0,
            ..window
        };
        assert_eq!(VIEWPORT.clip(apart).width, 0.0);
        // Placed inside the window, not over the page to its left.
        let p = place(
            anchor(520.0, 100.0),
            (200.0, 100.0),
            VIEWPORT.clip(window),
            Side::Below,
            PopoverAlign::End,
        );
        assert_eq!(p.left, 500.0 + EDGE);
    }

    #[test]
    fn it_is_clipped_to_the_viewport() {
        // Off the right edge: moved back in.
        let p = place(
            anchor(750.0, 100.0),
            (200.0, 100.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::Start,
        );
        assert_eq!(p.left, 800.0 - EDGE - 200.0);
        // Off the left edge.
        let p = place(
            anchor(0.0, 100.0),
            (300.0, 100.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::End,
        );
        assert_eq!(p.left, EDGE);
        // Sideways, its top kept inside.
        let p = place(
            anchor(100.0, 580.0),
            (200.0, 100.0),
            VIEWPORT,
            Side::End,
            PopoverAlign::Start,
        );
        assert_eq!(p.top, 600.0 - EDGE - 100.0);
    }

    #[test]
    fn placement_holds_for_the_same_input() {
        let at = |w| {
            place(
                anchor(100.0, 100.0),
                (w, 150.0),
                VIEWPORT,
                Side::Below,
                PopoverAlign::Start,
            )
        };
        assert!(at(200.0).near(&at(200.0)));
        assert!(!at(200.0).near(&place(
            anchor(100.0, 101.0),
            (200.0, 150.0),
            VIEWPORT,
            Side::Below,
            PopoverAlign::Start
        )));
    }

    #[test]
    fn a_press_outside_dismisses_without_returning_focus() {
        assert_eq!(Dismiss::for_press(false, false), Some(Dismiss::Outside));
        assert_eq!(Dismiss::for_press(true, false), None);
        assert_eq!(Dismiss::for_press(false, true), None);
        assert!(!Dismiss::Outside.returns_focus());
    }

    #[test]
    fn closed_by_its_owner_it_hands_focus_back_only_if_it_had_it() {
        assert!(hands_back(None, true));
        assert!(!hands_back(None, false));
        assert!(hands_back(Some(true), false));
        assert!(!hands_back(Some(false), true));
    }

    #[test]
    fn escape_and_close_dismiss_and_return_focus() {
        assert_eq!(Dismiss::for_key("Escape"), Some(Dismiss::Escape));
        assert_eq!(Dismiss::for_key("Enter"), None);
        assert!(Dismiss::Escape.returns_focus());
        assert!(Dismiss::Close.returns_focus());
    }

    #[test]
    fn sheet_and_placed_are_modifiers() {
        assert_eq!(names(popover(false, false)), [class::POPOVER]);
        assert_eq!(names(popover(true, false)), [class::POPOVER, class::SHEET]);
        assert_eq!(names(popover(false, true)), [class::POPOVER, class::PLACED]);
    }

    fn render_with(open: bool, sheet: bool, head: bool, title: Option<&'static str>) -> String {
        render_props(open, sheet, head, title, false)
    }

    fn render_props(
        open: bool,
        sheet: bool,
        head: bool,
        title: Option<&'static str>,
        flush: bool,
    ) -> String {
        let props = PopoverProps {
            anchor: NodeRef::default(),
            anchor_element: None,
            open,
            on_dismiss: Callback::noop(),
            side: Side::Below,
            align: PopoverAlign::Start,
            fit: Fit::Scroll,
            within: None,
            sheet,
            head,
            trap: false,
            flush,
            match_anchor: false,
            keep_focus: false,
            title: title.map(AttrValue::Static),
            close_label: None,
            focus: None,
            focus_first: false,
            on_placed: None,
            role: AttrValue::Static("dialog"),
            class: Classes::new(),
            testid: None,
            on_pointer_enter: None,
            on_pointer_leave: None,
            children: html! { "Body" },
        };
        futures::executor::block_on(
            yew::LocalServerRenderer::<Popover>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn render(open: bool, sheet: bool, title: Option<&'static str>) -> String {
        render_with(open, sheet, true, title)
    }

    #[test]
    fn it_renders_only_while_open_and_hidden_until_placed() {
        assert_eq!(render(false, true, None), "");
        let html = render(true, true, Some("Filters"));
        assert!(
            html.contains(&format!("\"{} {}\"", class::POPOVER, class::SHEET)),
            "{html}"
        );
        assert!(!html.contains(class::PLACED), "{html}");
        assert!(html.contains(class::POPOVER_SCRIM), "{html}");
        assert!(html.contains(class::POPOVER_HEAD), "{html}");
        assert!(html.contains("aria-label=\"Filters\""), "{html}");
        assert!(html.contains("aria-label=\"Close\""), "{html}");
    }

    #[test]
    fn it_opens_without_an_anchor_ref() {
        let html = futures::executor::block_on(
            yew::LocalServerRenderer::<Popover>::with_props(yew::props!(PopoverProps {
                open: true,
                on_dismiss: Callback::noop(),
            }))
            .hydratable(false)
            .render(),
        );
        assert!(html.contains("role=\"dialog\""), "{html}");
    }

    #[test]
    fn focus_first_makes_the_surface_focusable() {
        let html = futures::executor::block_on(
            yew::LocalServerRenderer::<Popover>::with_props(yew::props!(PopoverProps {
                anchor: NodeRef::default(),
                open: true,
                on_dismiss: Callback::noop(),
                focus_first: true,
            }))
            .hydratable(false)
            .render(),
        );
        assert!(html.contains("tabindex=\"-1\""), "{html}");
        assert!(!render(true, true, None).contains("tabindex"));
    }

    #[test]
    fn without_a_sheet_there_is_no_scrim_or_head() {
        let html = render(true, false, Some("Filters"));
        assert!(!html.contains(class::POPOVER_SCRIM), "{html}");
        assert!(!html.contains(class::POPOVER_HEAD), "{html}");
        assert!(html.contains(">Body</div>"), "{html}");
    }

    #[test]
    fn the_sheets_head_and_its_close_come_before_the_body_in_tab_order() {
        let html = render(true, true, Some("Filters"));
        let (head, body) = (
            html.find(class::POPOVER_HEAD),
            html.find(class::POPOVER_BODY),
        );
        assert!(head.is_some() && head < body, "{html}");
    }

    #[test]
    fn a_sheet_without_its_head_keeps_its_scrim_and_name() {
        let html = render_with(true, true, false, Some("Filters"));
        assert!(!html.contains(class::POPOVER_HEAD), "{html}");
        assert!(html.contains(class::POPOVER_SCRIM), "{html}");
        assert!(html.contains("aria-label=\"Filters\""), "{html}");
    }

    #[test]
    fn match_anchor_and_keep_focus_are_off_by_default() {
        let html = render(true, false, None);
        assert!(!html.contains(class::MATCH_ANCHOR), "{html}");
        assert!(!html.contains("--popover-anchor-width"), "{html}");
        assert!(super::KEEPS_PRESS.contains("input"));
    }

    #[test]
    fn a_flush_body_is_a_modifier() {
        let flush = format!("class=\"{} {}\"", class::POPOVER_BODY, class::FLUSH);
        assert!(render_props(true, false, true, None, true).contains(&flush));
        let plain = render_props(true, false, true, None, false);
        assert!(
            plain.contains(&format!("class=\"{}\"", class::POPOVER_BODY)),
            "{plain}"
        );
    }
}
