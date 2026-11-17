//! Popover (`style/_popover.scss`).

use std::cell::Cell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, HtmlElement, Node};
use yew::prelude::*;

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
    let side = if need > room(side) && room(side.opposite()) > room(side) {
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
            let max_height = room(side).max(0.0);
            let shown = height.min(max_height);
            let top = if side == Side::Below {
                anchor.bottom() + GAP
            } else {
                anchor.top - GAP - shown
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
    let mut found = Vec::new();
    let mut at = element.parent_element();
    while let Some(e) = at {
        if e.tag_name() == "BODY" || e.tag_name() == "HTML" {
            break;
        }
        let clips = window
            .get_computed_style(&e)
            .ok()
            .flatten()
            .is_some_and(|style| {
                ["overflow-x", "overflow-y"]
                    .iter()
                    .any(|p| style.get_property_value(p).is_ok_and(|v| v != "visible"))
            });
        if clips {
            found.push(e.clone());
        }
        at = e.parent_element();
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
    use_placement(open, anchor, popover, side, align, on_dismiss, false)
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
    use_placement(open, anchor, popover, side, align, on_dismiss, true)
}

#[hook]
#[allow(clippy::too_many_arguments)]
fn use_placement(
    open: bool,
    anchor: NodeRef,
    popover: NodeRef,
    side: Side,
    align: PopoverAlign,
    on_dismiss: Callback<Dismiss>,
    tip: bool,
) -> Option<Placed> {
    let placed = use_state_eq(|| None::<Placed>);
    let overlays = use_overlays();
    // The latest callback, read by listeners made when it opened.
    let latest = use_mut_ref(Callback::noop);
    *latest.borrow_mut() = on_dismiss;
    let on_dismiss = latest;

    {
        let placed = placed.clone();
        use_effect_with((open, side, align), move |&(open, side, align)| {
            // Why it was dismissed (whether that returns focus), and
            // whether focus or a press was ever inside it.
            let refocus = Rc::new(Cell::new(None::<bool>));
            let inside = Rc::new(Cell::new(false));
            let mut held: Vec<Listener> = Vec::new();
            let mut observer = None;

            if open {
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
                let clipped = anchor
                    .cast::<Element>()
                    .map(|a| clippers(&a))
                    .unwrap_or_default();
                let measure = {
                    let (anchor, popover, placed) =
                        (anchor.clone(), popover.clone(), placed.clone());
                    let (on_dismiss, refocus) = (on_dismiss.clone(), refocus.clone());
                    let clipped = clipped.clone();
                    let gone = Rc::new(Cell::new(false));
                    Rc::new(move || {
                        let (Some(at), Some(pop), Some(viewport)) = (
                            anchor.cast::<Element>(),
                            popover.cast::<HtmlElement>(),
                            viewport(),
                        ) else {
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
                        // Its natural height, whatever max-height cuts it to.
                        let size = (
                            f64::from(pop.offset_width()),
                            f64::from(pop.scroll_height()),
                        );
                        let next = place(Rect::of(&at), size, viewport, side, align);
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
                        let within = |node: &NodeRef, event: &web_sys::Event| {
                            let target = event.target().and_then(|t| t.dyn_into::<Node>().ok());
                            node.get().is_some_and(|n| n.contains(target.as_ref()))
                        };
                        let pressed = {
                            let (anchor, popover) = (anchor.clone(), popover.clone());
                            let (on_dismiss, refocus) = (on_dismiss.clone(), refocus.clone());
                            let layer = layer.clone();
                            move |e: web_sys::Event| {
                                if !layer.is_top() {
                                    return;
                                }
                                if let Some(why) =
                                    Dismiss::for_press(within(&anchor, &e), within(&popover, &e))
                                {
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
                        let keyed = {
                            let (on_dismiss, refocus) = (on_dismiss.clone(), refocus.clone());
                            let layer = layer.clone();
                            move |e: web_sys::Event| {
                                if !layer.is_top() {
                                    return;
                                }
                                // Only keydown reaches here; unchecked, as an
                                // event from another frame fails `instanceof`.
                                let key = e.unchecked_ref::<web_sys::KeyboardEvent>().key();
                                if let Some(why) = Dismiss::for_key(&key) {
                                    refocus.set(Some(why.returns_focus()));
                                    on_dismiss.borrow().emit(why);
                                }
                            }
                        };
                        held.push(Listener::new(document.into(), "keydown", false, keyed));
                    }
                }

                let resized = {
                    let measure = measure.clone();
                    Closure::<dyn Fn()>::new(move || measure())
                };
                if let Ok(watch) = web_sys::ResizeObserver::new(resized.as_ref().unchecked_ref()) {
                    for node in [&anchor, &popover] {
                        if let Some(element) = node.cast::<Element>() {
                            watch.observe(&element);
                        }
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
                drop(held);
                if let Some((watch, _)) = observer {
                    watch.disconnect();
                }
                if open
                    && hands_back(refocus.get(), inside.get())
                    && focus_lost()
                    && let Some(anchor) = anchor.cast::<HtmlElement>()
                {
                    // Where the page stands: the anchor may be scrolled
                    // away under a sheet.
                    let options = web_sys::FocusOptions::new();
                    options.set_prevent_scroll(true);
                    _ = anchor.focus_with_options(&options);
                }
            }
        });
    }

    *placed
}

#[derive(Properties, PartialEq)]
pub struct PopoverProps {
    /// What it opens beside, and where focus returns when it closes.
    pub anchor: NodeRef,
    pub open: bool,
    /// Asked to close it (the owner sets `open` to false).
    pub on_dismiss: Callback<Dismiss>,
    #[prop_or_default]
    pub side: Side,
    #[prop_or_default]
    pub align: PopoverAlign,
    /// A sheet from the bottom below the phone breakpoint, over a scrim
    /// (`.plinth-sheet`); on by default.
    #[prop_or(true)]
    pub sheet: bool,
    /// Its accessible name, and the sheet's heading.
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The sheet's close button's accessible name.
    #[prop_or(AttrValue::Static("Close"))]
    pub close_label: AttrValue,
    /// Takes focus once it is placed (it is hidden until then), such as the
    /// safe button of a confirm.
    #[prop_or_default]
    pub focus: Option<NodeRef>,
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
    let node = use_node_ref();
    let overlays = use_overlays();
    let placed = use_popover(
        props.open,
        props.anchor.clone(),
        node.clone(),
        props.side,
        props.align,
        props.on_dismiss.clone(),
    );
    {
        let focus = props.focus.clone();
        use_effect_with(placed.is_some(), move |&placed| {
            if placed && let Some(target) = focus.and_then(|f| f.cast::<HtmlElement>()) {
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                _ = target.focus_with_options(&options);
            }
        });
    }

    if !props.open {
        return Html::default();
    }

    let style = placed.map(|p| {
        format!(
            "--popover-left: {}px; --popover-top: {}px; --popover-max-height: {}px;",
            p.left, p.top, p.max_height
        )
    });
    let head = (props.sheet && props.title.is_some()).then(|| {
        let close = props.on_dismiss.reform(|_: MouseEvent| Dismiss::Close);
        html! {
            <div class={class::POPOVER_HEAD}>
                <span class={class::POPOVER_TITLE}>{ props.title.clone() }</span>
                <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                    icon={html! { <Icon glyph={Glyph::Close} /> }}
                    label={props.close_label.clone()} onclick={close} />
            </div>
        }
    });

    overlays.portal(html! {
        <>
            if props.sheet {
                <div class={class::POPOVER_SCRIM} aria-hidden="true"></div>
            }
            <div ref={node}
                class={classes!(popover(props.sheet, placed.is_some()), props.class.clone())}
                role="dialog" aria-label={props.title.clone()} {style}
                onpointerenter={props.on_pointer_enter.clone()}
                onpointerleave={props.on_pointer_leave.clone()}
                data-testid={props.testid.clone()}>
                { for head }
                <div class={class::POPOVER_BODY}>{ props.children.clone() }</div>
            </div>
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

    fn render(open: bool, sheet: bool, title: Option<&'static str>) -> String {
        let props = PopoverProps {
            anchor: NodeRef::default(),
            open,
            on_dismiss: Callback::noop(),
            side: Side::Below,
            align: PopoverAlign::Start,
            sheet,
            title: title.map(AttrValue::Static),
            close_label: "Close".into(),
            focus: None,
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
    }

    #[test]
    fn without_a_sheet_there_is_no_scrim_or_head() {
        let html = render(true, false, Some("Filters"));
        assert!(!html.contains(class::POPOVER_SCRIM), "{html}");
        assert!(!html.contains(class::POPOVER_HEAD), "{html}");
        assert!(html.contains(">Body</div>"), "{html}");
    }
}
