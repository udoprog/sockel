//! Tooltip, HoverCard and ReasonButton (`style/_tooltip.scss`).

use crate::testid::with_testid;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

use crate::popover::Listener;
use crate::timer::Timer;
use crate::{
    Button, ButtonProps, ButtonSize, ButtonVariant, Dismiss, Overlays, Placed, PopoverAlign, Side,
    class, popover::use_tip_popover, use_overlays,
};

/// How long a pointer rests on the anchor before a tooltip shows, in
/// milliseconds.
pub const TOOLTIP_DELAY: u32 = 250;
/// How long a hover card waits to open, and to close after the pointer left
/// the anchor and the card.
pub const HOVER_CARD_DELAY: u32 = 400;

static IDS: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn next_id(kind: &str) -> AttrValue {
    AttrValue::from(format!(
        "sockel-{kind}-{}",
        IDS.fetch_add(1, Ordering::Relaxed)
    ))
}

/// How long to wait before the shown state flips: to open when the
/// pointer enters a closed anchor, to close when it leaves an open one;
/// none when there is nothing to change.
fn wait(open: bool, entering: bool, open_ms: u32, close_ms: u32) -> Option<u32> {
    match (open, entering) {
        (false, true) => Some(open_ms),
        (true, false) => Some(close_ms),
        _ => None,
    }
}

/// `list` (an `aria-describedby`) with `id` in it.
fn describe(list: Option<&str>, id: &str) -> String {
    match list {
        Some(list) if list.split_whitespace().any(|t| t == id) => list.to_owned(),
        Some(list) if !list.trim().is_empty() => format!("{} {id}", list.trim()),
        _ => id.to_owned(),
    }
}

/// `list` without `id`; none when nothing is left.
fn undescribe(list: Option<&str>, id: &str) -> Option<String> {
    let rest: Vec<_> = list?.split_whitespace().filter(|t| *t != id).collect();
    (!rest.is_empty()).then(|| rest.join(" "))
}

#[hook]
fn use_describes(anchor: NodeRef, id: AttrValue, active: bool) {
    use_effect_with((id, active), move |(id, active)| {
        let id = id.clone();
        let element = anchor.cast::<Element>();
        if let (true, Some(element)) = (*active, &element) {
            let list = element.get_attribute("aria-describedby");
            _ = element.set_attribute("aria-describedby", &describe(list.as_deref(), &id));
        }
        move || {
            if let Some(element) = element {
                let list = element.get_attribute("aria-describedby");
                match undescribe(list.as_deref(), &id) {
                    Some(rest) => _ = element.set_attribute("aria-describedby", &rest),
                    None => _ = element.remove_attribute("aria-describedby"),
                }
            }
        }
    });
}

/// The shown state of a tip or card and its timer.
struct Control {
    open: UseStateHandle<bool>,
    flag: Cell<bool>,
    open_ms: Cell<u32>,
    close_ms: Cell<u32>,
    timer: RefCell<Option<Timer>>,
}

impl Control {
    fn cancel(&self) {
        self.timer.take();
    }

    fn now(&self, open: bool) {
        self.cancel();
        self.flag.set(open);
        self.open.set(open);
    }

    fn pointer(self: &Rc<Self>, entering: bool) {
        self.cancel();
        let ms = if entering {
            self.open_ms.get()
        } else {
            self.close_ms.get()
        };
        match wait(self.flag.get(), entering, ms, self.close_ms.get()) {
            Some(0) => self.now(entering),
            Some(ms) => {
                let this = self.clone();
                *self.timer.borrow_mut() = Timer::start(ms, move || {
                    this.flag.set(entering);
                    this.open.set(entering);
                });
            }
            None => {}
        }
    }
}

/// Whether a tip or card is shown for its anchor, and how to change that.
#[derive(Clone)]
struct Hover {
    open: bool,
    control: Rc<Control>,
}

impl Hover {
    fn hide(&self) -> Callback<Dismiss> {
        let control = self.control.clone();
        Callback::from(move |_| control.now(false))
    }
}

/// The state of a tip or card: shown `open_ms` after a pointer enters and
/// hidden `close_ms` after it leaves.
#[hook]
fn use_hover(open_ms: u32, close_ms: u32) -> Hover {
    let open = use_state_eq(|| false);
    let control = {
        let open = open.clone();
        use_memo((), move |()| Control {
            open,
            flag: Cell::new(false),
            open_ms: Cell::new(open_ms),
            close_ms: Cell::new(close_ms),
            timer: RefCell::new(None),
        })
    };
    control.open_ms.set(open_ms);
    control.close_ms.set(close_ms);
    use_effect_with((), {
        let control = control.clone();
        move |()| move || control.cancel()
    });
    Hover {
        open: *open,
        control: control.clone(),
    }
}

/// Shows `hover` for `anchor`: after the pointer rests on it (not a touch),
/// at once on keyboard focus, hidden when either leaves and on a press. An
/// anchor already under the pointer or focused when this mounts counts as
/// entered.
#[hook]
fn use_anchor_events(anchor: NodeRef, hover: &Hover) {
    let control = hover.control.clone();
    use_effect_with(anchor, move |anchor| {
        let mut held: Vec<Listener> = Vec::new();
        if let Some(element) = anchor.cast::<Element>() {
            let mut on = |kind: &'static str, f: Box<dyn Fn(&web_sys::Event)>| {
                held.push(Listener::new(
                    element.clone().into(),
                    kind,
                    false,
                    move |e| f(&e),
                ));
            };
            let c = control.clone();
            on(
                "pointerenter",
                Box::new(move |e| {
                    if !is_touch(e) {
                        c.pointer(true);
                    }
                }),
            );
            let c = control.clone();
            on("pointerleave", Box::new(move |_| c.pointer(false)));
            let (c, target) = (control.clone(), element.clone());
            on(
                "focusin",
                Box::new(move |_| {
                    if target.matches(":focus-visible").unwrap_or(false) {
                        c.now(true);
                    }
                }),
            );
            let c = control.clone();
            on("focusout", Box::new(move |_| c.now(false)));
            let c = control.clone();
            on("pointerdown", Box::new(move |_| c.now(false)));

            if element.matches(":focus-visible").unwrap_or(false) {
                control.now(true);
            } else if element.matches(":hover").unwrap_or(false) {
                control.pointer(true);
            }
        }
        move || drop(held)
    });
}

fn is_touch(e: &web_sys::Event) -> bool {
    e.dyn_ref::<web_sys::PointerEvent>()
        .is_some_and(|p| p.pointer_type() == "touch")
}

#[derive(Properties, PartialEq)]
pub struct TooltipProps {
    /// What it names or explains; nothing is shown while it is empty.
    pub anchor: NodeRef,
    pub text: AttrValue,
    #[prop_or(Side::Above)]
    pub side: Side,
    /// The pointer's rest before it shows, in milliseconds; keyboard focus
    /// shows it at once.
    #[prop_or(TOOLTIP_DELAY)]
    pub delay: u32,
    /// The tip's id, which the anchor's `aria-describedby` names; made
    /// when not given.
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

impl Default for TooltipProps {
    fn default() -> Self {
        Self {
            anchor: Default::default(),
            text: Default::default(),
            side: ::yew::html::IntoPropValue::into_prop_value(Side::Above),
            delay: ::yew::html::IntoPropValue::into_prop_value(TOOLTIP_DELAY),
            id: Default::default(),
            class: Default::default(),
            testid: Default::default(),
        }
    }
}

/// A tooltip's classes: `tooltip`, and `placed` once it has been measured.
pub fn tooltip(placed: bool) -> Classes {
    classes!(class::TOOLTIP, placed.then_some(class::PLACED))
}

fn position(left: f64, top: f64) -> String {
    format!("--tooltip-left: {left}px; --tooltip-top: {top}px;")
}

/// A short tip beside its `anchor` in the overlay layer, shown after the
/// pointer rests on it or at once on keyboard focus, hidden on a press,
/// Escape or when either leaves. The anchor is described by it
/// (`aria-describedby`). It never takes room: the page does not move.
#[function_component]
pub fn Tooltip(props: &TooltipProps) -> Html {
    let overlays = use_overlays();
    let id = use_state(|| props.id.clone().unwrap_or_else(|| next_id("tip")));
    let id = props.id.clone().unwrap_or_else(|| (*id).clone());
    let node = use_node_ref();
    let hover = use_hover(props.delay, 0);
    use_anchor_events(props.anchor.clone(), &hover);
    let has_text = !props.text.trim().is_empty();
    let placed = use_tip_popover(
        hover.open && has_text,
        props.anchor.clone(),
        node.clone(),
        props.side,
        PopoverAlign::Center,
        hover.hide(),
    );
    use_describes(props.anchor.clone(), id.clone(), has_text);

    if !has_text {
        return Html::default();
    }
    tip_view(
        &overlays,
        node,
        id,
        &props.text,
        placed,
        props.class.clone(),
        props.testid.clone(),
    )
}

fn tip_view(
    overlays: &Overlays,
    node: NodeRef,
    id: AttrValue,
    text: &AttrValue,
    placed: Option<Placed>,
    class: Classes,
    testid: Option<AttrValue>,
) -> Html {
    let style = placed.map(|p| position(p.left, p.top));
    overlays.portal(with_testid(
        html! {
            <div ref={node} {id} role="tooltip" {style}
                class={classes!(tooltip(placed.is_some()), class)}>
                { text.clone() }
            </div>
        },
        testid,
    ))
}

#[derive(Properties, PartialEq)]
pub struct HoverCardProps {
    pub anchor: NodeRef,
    #[prop_or_default]
    pub side: Side,
    #[prop_or_default]
    pub align: PopoverAlign,
    /// The wait before it opens, and before it closes once the pointer
    /// is away from the anchor and the card, in milliseconds.
    #[prop_or(HOVER_CARD_DELAY)]
    pub delay: u32,
    /// Its accessible name.
    #[prop_or_default]
    pub title: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

impl Default for HoverCardProps {
    fn default() -> Self {
        Self {
            anchor: Default::default(),
            side: Default::default(),
            align: Default::default(),
            delay: ::yew::html::IntoPropValue::into_prop_value(HOVER_CARD_DELAY),
            title: Default::default(),
            class: Default::default(),
            testid: Default::default(),
            children: Default::default(),
        }
    }
}

/// A hover card's classes: a `popover`, `hover-card`, and `placed` once
/// measured.
pub fn hover_card(placed: bool) -> Classes {
    classes!(
        class::POPOVER,
        class::HOVER_CARD,
        placed.then_some(class::PLACED)
    )
}

/// A richer card beside its `anchor`, opened by hover or keyboard focus and
/// kept open while the pointer is over the card, so its content can be
/// reached. Placed and dismissed as a [`Popover`](crate::Popover) is.
#[function_component]
pub fn HoverCard(props: &HoverCardProps) -> Html {
    let overlays = use_overlays();
    let id = use_state(|| next_id("card"));
    let node = use_node_ref();
    let hover = use_hover(props.delay, props.delay);
    use_anchor_events(props.anchor.clone(), &hover);
    let placed = use_tip_popover(
        hover.open,
        props.anchor.clone(),
        node.clone(),
        props.side,
        props.align,
        hover.hide(),
    );
    use_describes(props.anchor.clone(), (*id).clone(), hover.open);

    if !hover.open {
        return Html::default();
    }
    let style = placed.map(|p| {
        format!(
            "--popover-left: {}px; --popover-top: {}px; --popover-max-height: {}px;",
            p.left, p.top, p.max_height
        )
    });
    let (enter, leave) = (hover.control.clone(), hover.control.clone());
    overlays.portal(with_testid(
        html! {
            <div ref={node} id={(*id).clone()} role="group" aria-label={props.title.clone()} {style}
                class={classes!(hover_card(placed.is_some()), props.class.clone())}
                onpointerenter={Callback::from(move |_| enter.pointer(true))}
                onpointerleave={Callback::from(move |_| leave.pointer(false))}>
                <div class={class::POPOVER_BODY}>{ props.children.clone() }</div>
            </div>
        },
        props.testid.clone(),
    ))
}

#[derive(Properties, Default, PartialEq)]
pub struct ReasonButtonProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    #[prop_or_default]
    pub icon: Option<Html>,
    /// Why it cannot act now: it stays focusable and the tooltip says so.
    #[prop_or_default]
    pub reason: Option<AttrValue>,
    /// Working on what it started (a spinner over the label); the tooltip
    /// says what is happening, when `working` is given.
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub working: Option<AttrValue>,
    #[prop_or_default]
    pub submit: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// The tip a [`ReasonButton`] shows: its reason, else, while busy, what it
/// is doing.
pub fn reason_text(reason: Option<&str>, busy: bool, working: Option<&str>) -> String {
    reason
        .or(busy.then_some(working).flatten())
        .unwrap_or_default()
        .to_owned()
}

/// A [`Button`] that says why it cannot act, or what it is doing, in a
/// [`Tooltip`] that shows on hover and keyboard focus. It is one button
/// whether it is available, unavailable or busy: unavailable is
/// `aria-disabled`, not `disabled`, which would take focus and hover away
/// along with the node, so the box and the focus stay.
#[function_component]
pub fn ReasonButton(props: &ReasonButtonProps) -> Html {
    let anchor = use_node_ref();
    let id = use_state(|| next_id("reason"));
    let text = reason_text(
        props.reason.as_deref(),
        props.busy,
        props.working.as_deref(),
    );
    let button = ButtonProps {
        class: props.class.clone(),
        variant: props.variant,
        size: props.size,
        icon: props.icon.clone(),
        trailing: false,
        chevron: false,
        fold: false,
        fold_wide: false,
        spread: false,
        justify: false,
        labelled: false,
        wide: false,
        form: None,
        tabindex: None,
        hidden: false,
        data: Vec::new(),
        pressed: None,
        selected: false,
        link: false,
        busy: props.busy,
        busy_mark: false,
        disabled: false,
        unavailable: props.reason.is_some(),
        node_ref: anchor.clone(),
        expanded: None,
        haspopup: None,
        href: None,
        submit: props.submit,
        onclick: props.onclick.clone(),
        onkeydown: Callback::noop(),
        onmouseenter: Callback::noop(),
        onmouseleave: Callback::noop(),
        onpointerdown: Callback::noop(),
        onmousedown: Callback::noop(),
        onauxclick: Callback::noop(),
        ondragenter: Callback::noop(),
        ondragover: Callback::noop(),
        ondragleave: Callback::noop(),
        ondrop: Callback::noop(),
        title: None,
        style: None,
        tip: None,
        label: props.label.clone(),
        testid: props.testid.clone(),
        label_testid: None,
        after: None,
        autofocus: false,
        role: None,
        checked: None,
        current: None,
        children: props.children.clone(),
    };
    html! {
        <>
            <Button ..button />
            <Tooltip {anchor} text={text} id={(*id).clone()} />
        </>
    }
}

/// The tip an element asks for with `data-tip`: none when empty, when the
/// element already says it, or while its menu is open, which the tip would
/// cover.
fn data_tip(element: &Element) -> Option<String> {
    let tip = element.get_attribute("data-tip")?;
    let says = element.text_content().unwrap_or_default();
    let expanded = element.get_attribute("aria-expanded").as_deref() == Some("true");
    wanted(&tip, &says, expanded).then_some(tip)
}

fn wanted(tip: &str, says: &str, expanded: bool) -> bool {
    !tip.trim().is_empty() && says.trim() != tip.trim() && !expanded
}

#[derive(Clone, PartialEq)]
struct Tipped {
    key: usize,
    element: Element,
    /// Where it was when the tip was asked for: left, top, width, height.
    rect: (f64, f64, f64, f64),
}

/// Gives every element with a `data-tip` attribute a tooltip with that
/// text, from one set of listeners for the page: mount it once, anywhere.
/// The tip is placed by a stand-in for the element, so it hides when the
/// page scrolls.
#[function_component]
pub fn DataTips() -> Html {
    let overlays = use_overlays();
    let current = use_state_eq(|| None::<Tipped>);
    let id = use_state(|| next_id("tip"));
    let hover = use_hover(TOOLTIP_DELAY, 0);
    let (stand_in, node) = (use_node_ref(), use_node_ref());
    {
        let (current, control) = (current.clone(), hover.control.clone());
        use_effect_with((), move |()| {
            let count = Rc::new(Cell::new(0usize));
            let last = Rc::new(RefCell::new(None::<Tipped>));
            let mut held = Vec::new();
            if let Some(window) = web_sys::window() {
                let find = {
                    let (current, count, last) = (current.clone(), count.clone(), last.clone());
                    move |e: &web_sys::Event| -> Option<bool> {
                        let found = e
                            .target()
                            .and_then(|t| t.dyn_into::<Element>().ok())
                            .and_then(|t| t.closest("[data-tip]").ok().flatten())
                            .filter(|t| data_tip(t).is_some());
                        let mut last = last.borrow_mut();
                        if last.as_ref().map(|t| &t.element) == found.as_ref() {
                            return None;
                        }
                        *last = found.map(|element| {
                            count.set(count.get() + 1);
                            let r = element.get_bounding_client_rect();
                            Tipped {
                                key: count.get(),
                                rect: (r.left(), r.top(), r.width(), r.height()),
                                element,
                            }
                        });
                        current.set(last.clone());
                        Some(last.is_some())
                    }
                };
                let c = control.clone();
                let over = find.clone();
                held.push(Listener::new(
                    window.clone().into(),
                    "pointerover",
                    true,
                    move |e| {
                        if let Some(some) = over(&e) {
                            c.now(false);
                            if some && !is_touch(&e) {
                                c.pointer(true);
                            }
                        }
                    },
                ));
                let c = control.clone();
                held.push(Listener::new(
                    window.clone().into(),
                    "focusin",
                    true,
                    move |e| {
                        let Some(focused) = find(&e) else {
                            return;
                        };
                        c.now(false);
                        let visible = e
                            .target()
                            .and_then(|t| t.dyn_into::<Element>().ok())
                            .is_some_and(|t| t.matches(":focus-visible").unwrap_or(false));
                        if focused && visible {
                            c.now(true);
                        }
                    },
                ));
                let c = control.clone();
                held.push(Listener::new(
                    window.clone().into(),
                    "keydown",
                    true,
                    move |e| {
                        if e.dyn_ref::<KeyboardEvent>()
                            .is_some_and(|k| k.key() == "Escape")
                        {
                            c.now(false);
                        }
                    },
                ));
                for kind in ["pointerdown", "scroll", "focusout"] {
                    let c = control.clone();
                    held.push(Listener::new(
                        window.clone().into(),
                        kind,
                        true,
                        move |_| c.now(false),
                    ));
                }
            }
            move || drop(held)
        });
    }

    let text = current
        .as_ref()
        .and_then(|t| data_tip(&t.element))
        .unwrap_or_default();
    let open = hover.open && !text.is_empty();
    let placed = use_tip_popover(
        open,
        stand_in.clone(),
        node.clone(),
        Side::Above,
        PopoverAlign::Center,
        hover.hide(),
    );
    let element = current.as_ref().map(|t| t.element.clone());
    use_effect_with((element, open), {
        let id = (*id).clone();
        move |(element, open)| {
            let element = element.clone().filter(|_| *open);
            if let Some(element) = &element {
                let list = element.get_attribute("aria-describedby");
                _ = element.set_attribute("aria-describedby", &describe(list.as_deref(), &id));
            }
            move || {
                if let Some(element) = element {
                    let list = element.get_attribute("aria-describedby");
                    match undescribe(list.as_deref(), &id) {
                        Some(rest) => _ = element.set_attribute("aria-describedby", &rest),
                        None => _ = element.remove_attribute("aria-describedby"),
                    }
                }
            }
        }
    });

    let Some(Tipped {
        rect: (left, top, width, height),
        ..
    }) = &*current
    else {
        return Html::default();
    };
    if !open {
        return Html::default();
    }
    let style = format!(
        "--tip-left: {left}px; --tip-top: {top}px; --tip-width: {width}px; --tip-height: {height}px;"
    );
    html! {
        <>
            { overlays.portal(html! { <span ref={stand_in} class={class::TIP_ANCHOR} {style}></span> }) }
            { tip_view(&overlays, node, (*id).clone(), &AttrValue::from(text), placed, Classes::new(), None) }
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn render<C: BaseComponent>(props: C::Properties) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<C>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn reason(reason: Option<&'static str>, busy: bool) -> ReasonButtonProps {
        ReasonButtonProps {
            class: Classes::new(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            icon: None,
            reason: reason.map(AttrValue::Static),
            busy,
            working: Some(AttrValue::Static("Saving")),
            submit: false,
            onclick: Callback::noop(),
            label: None,
            testid: None,
            children: html! { "Save" },
        }
    }

    #[test]
    fn a_data_tip_is_wanted_only_where_it_adds_something() {
        assert!(wanted("Delete", "", false));
        assert!(!wanted(" ", "", false));
        assert!(!wanted("Delete", " Delete ", false));
        assert!(!wanted("More", "", true));
    }

    #[test]
    fn the_pointer_waits_to_open_and_to_close() {
        assert_eq!(wait(false, true, 250, 0), Some(250));
        assert_eq!(wait(true, false, 250, 400), Some(400));
        assert_eq!(wait(true, true, 250, 400), None);
        assert_eq!(wait(false, false, 250, 400), None);
    }

    #[test]
    fn the_anchor_names_the_tip_once() {
        assert_eq!(describe(None, "t"), "t");
        assert_eq!(describe(Some("a"), "t"), "a t");
        assert_eq!(describe(Some("a t"), "t"), "a t");
        assert_eq!(undescribe(Some("a t"), "t").as_deref(), Some("a"));
        assert_eq!(undescribe(Some("t"), "t"), None);
    }

    #[test]
    fn tooltip_and_hover_card_are_modifiers_of_placed() {
        assert_eq!(names(tooltip(false)), [class::TOOLTIP]);
        assert_eq!(names(tooltip(true)), [class::TOOLTIP, class::PLACED]);
        assert_eq!(
            names(hover_card(true)),
            [class::POPOVER, class::HOVER_CARD, class::PLACED]
        );
    }

    #[test]
    fn a_tooltip_is_a_hidden_tooltip_role_with_its_id() {
        let html = render::<Tooltip>(TooltipProps {
            anchor: NodeRef::default(),
            text: "Remove".into(),
            side: Side::Above,
            delay: TOOLTIP_DELAY,
            id: Some("tip-1".into()),
            class: Classes::new(),
            testid: None,
        });
        assert!(html.contains("role=\"tooltip\""), "{html}");
        assert!(html.contains("id=\"tip-1\""), "{html}");
        assert!(!html.contains(class::PLACED), "{html}");
        assert!(html.contains("Remove"), "{html}");
    }

    #[test]
    fn an_empty_tooltip_renders_nothing() {
        let html = render::<Tooltip>(TooltipProps {
            anchor: NodeRef::default(),
            text: "  ".into(),
            side: Side::Above,
            delay: TOOLTIP_DELAY,
            id: None,
            class: Classes::new(),
            testid: None,
        });
        assert!(!html.contains("tooltip"), "{html}");
    }

    #[test]
    fn the_reason_shows_before_what_is_happening() {
        assert_eq!(
            reason_text(Some("Pick one"), true, Some("Saving")),
            "Pick one"
        );
        assert_eq!(reason_text(None, true, Some("Saving")), "Saving");
        assert_eq!(reason_text(None, false, Some("Saving")), "");
    }

    #[test]
    fn a_reason_button_is_one_button_in_every_state() {
        let ready = render::<ReasonButton>(reason(None, false));
        let blocked = render::<ReasonButton>(reason(Some("Pick one"), false));
        let busy = render::<ReasonButton>(reason(None, true));
        for html in [&ready, &blocked, &busy] {
            assert!(html.starts_with("<button"), "{html}");
            assert!(!html.contains(" disabled"), "{html}");
        }
        assert!(!ready.contains("aria-disabled"), "{ready}");
        assert!(!ready.contains("role=\"tooltip\""), "{ready}");
        assert!(blocked.contains("aria-disabled=\"true\""), "{blocked}");
        assert!(blocked.contains("role=\"tooltip\"") && blocked.contains("Pick one"));
        assert!(busy.contains("aria-busy=\"true\"") && busy.contains("Saving"));
    }
}
