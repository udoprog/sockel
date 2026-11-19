//! Seam: two panes and the divider dragged between them (`style/_seam.scss`).

use crate::testid::with_testid;
use phrasebook02::yew023::use_phrases_with;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use crate::class;
use crate::phrasebookcatalog01::{L10N, label};
use crate::size::Framed;

/// Which way a seam divides its box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeamAxis {
    /// Side by side, the divider upright.
    #[default]
    Columns,
    /// One over the other, the divider across.
    Rows,
}

/// A seam's classes; `active` while its divider is held.
pub fn seam(axis: SeamAxis, active: bool) -> Classes {
    classes!(
        class::SEAM,
        (axis == SeamAxis::Rows).then_some(class::ROWS),
        active.then_some(class::ACTIVE)
    )
}

/// A seam's box along its axis, in CSS pixels: where it starts, how long it
/// is, the divider's length and each pane's least length.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Span {
    pub origin: f64,
    pub length: f64,
    pub grip: f64,
    pub min_start: f64,
    pub min_end: f64,
}

impl Span {
    /// `at` (the first pane's share of the room beside the divider) kept
    /// within `min` and `max` and leaving each pane its least length; when
    /// the box is too short for both, the first pane keeps its own.
    pub(crate) fn clamp(&self, at: f64, min: f64, max: f64) -> f64 {
        let room = self.length - self.grip;
        if room <= 0.0 {
            return min.clamp(0.0, 1.0);
        }
        let low = min.max(self.min_start / room);
        let high = max.min(1.0 - self.min_end / room);
        at.min(high).max(low).clamp(0.0, 1.0)
    }

    /// The share a pointer at `pointer` asks for, its divider centred on it.
    pub(crate) fn at(&self, pointer: f64, min: f64, max: f64) -> f64 {
        let room = (self.length - self.grip).max(f64::MIN_POSITIVE);
        self.clamp((pointer - self.origin - self.grip / 2.0) / room, min, max)
    }
}

/// The share one arrow key moves the divider by.
const STEP: f64 = 0.05;

#[derive(Properties, PartialEq)]
pub struct SeamProps {
    /// The first pane (left, or top for [`SeamAxis::Rows`]).
    pub start: Html,
    pub end: Html,
    /// The first pane's share of the room beside the divider, 0 to 1.
    pub at: f64,
    /// Told the new share while the divider is dragged (at most once a frame)
    /// or moved by a key; the owner sets `at` to it.
    pub on_change: Callback<f64>,
    /// Told the share once a drag ends or a key moved it.
    #[prop_or_default]
    pub on_settle: Callback<f64>,
    #[prop_or_default]
    pub axis: SeamAxis,
    /// The least and greatest share; each pane also keeps `--seam-min`.
    #[prop_or(0.0)]
    pub min: f64,
    #[prop_or(1.0)]
    pub max: f64,
    /// The share a double-click on the divider returns to.
    #[prop_or(0.5)]
    pub reset: f64,
    /// The divider's name, for assistive technology.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A length in pixels from a computed style (`"128px"`).
fn pixels(value: &str) -> f64 {
    value.trim_end_matches("px").parse().unwrap_or(0.0)
}

fn least(pane: &NodeRef, axis: SeamAxis) -> f64 {
    let style = pane
        .cast::<Element>()
        .zip(web_sys::window())
        .and_then(|(element, window)| window.get_computed_style(&element).ok().flatten());
    let property = match axis {
        SeamAxis::Columns => "min-width",
        SeamAxis::Rows => "min-height",
    };
    style
        .and_then(|style| style.get_property_value(property).ok())
        .map_or(0.0, |value| pixels(&value))
}

fn measure(root: &NodeRef, grip: &NodeRef, panes: [&NodeRef; 2], axis: SeamAxis) -> Option<Span> {
    let rect = root.cast::<Element>()?.get_bounding_client_rect();
    let grip = grip.cast::<Element>()?.get_bounding_client_rect();
    let (origin, length, grip) = match axis {
        SeamAxis::Columns => (rect.left(), rect.width(), grip.width()),
        SeamAxis::Rows => (rect.top(), rect.height(), grip.height()),
    };
    Some(Span {
        origin,
        length,
        grip,
        min_start: least(panes[0], axis),
        min_end: least(panes[1], axis),
    })
}

/// The pointer's coordinate along `axis`.
fn along(e: &PointerEvent, axis: SeamAxis) -> f64 {
    f64::from(match axis {
        SeamAxis::Columns => e.client_x(),
        SeamAxis::Rows => e.client_y(),
    })
}

/// Two panes and a divider between them: dragged (the pointer captured, one
/// change a frame), moved by the arrow keys, Home and End, and reset by a
/// double-click. The panes are laid out by a grid from `at`, each at least
/// `--seam-min` long, so neither collapses and the divider keeps its box.
#[function_component]
pub fn Seam(props: &SeamProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let root = use_node_ref();
    let grip = use_node_ref();
    let start = use_node_ref();
    let end = use_node_ref();
    let held = use_state_eq(|| false);
    let pointer = use_mut_ref(|| None::<i32>);
    let span = use_mut_ref(|| None::<Span>);
    let latest = use_mut_ref(|| props.at);

    let on_change = use_mut_ref(Callback::<f64>::noop);
    *on_change.borrow_mut() = props.on_change.clone();
    // Made at the first grab: its closure needs a browser.
    let framed = use_mut_ref(|| None::<Framed>);

    let (axis, min, max) = (props.axis, props.min, props.max);

    let onpointerdown = {
        let (root, grip, start, end) = (root.clone(), grip.clone(), start.clone(), end.clone());
        let (pointer, span, held, framed) =
            (pointer.clone(), span.clone(), held.clone(), framed.clone());
        let (on_change, latest) = (on_change.clone(), latest.clone());
        Callback::from(move |e: PointerEvent| {
            if e.button() != 0 {
                return;
            }
            e.prevent_default();
            if let Some(element) = grip.cast::<HtmlElement>() {
                _ = element.set_pointer_capture(e.pointer_id());
                _ = element.focus();
            }
            *span.borrow_mut() = measure(&root, &grip, [&start, &end], axis);
            *pointer.borrow_mut() = Some(e.pointer_id());
            let mut framed = framed.borrow_mut();
            let framed = framed.get_or_insert_with(|| {
                let (on_change, latest) = (on_change.clone(), latest.clone());
                Framed::new(move |at| {
                    *latest.borrow_mut() = at;
                    on_change.borrow().emit(at);
                })
            });
            framed.reset();
            held.set(true);
        })
    };

    let onpointermove = {
        let (pointer, span, framed) = (pointer.clone(), span.clone(), framed.clone());
        Callback::from(move |e: PointerEvent| {
            if *pointer.borrow() != Some(e.pointer_id()) {
                return;
            }
            if let (Some(span), Some(framed)) = (*span.borrow(), framed.borrow().as_ref()) {
                framed.offer(span.at(along(&e, axis), min, max));
            }
        })
    };

    let release = {
        let (pointer, held, latest) = (pointer.clone(), held.clone(), latest.clone());
        let on_settle = props.on_settle.clone();
        Callback::from(move |e: PointerEvent| {
            if pointer.borrow_mut().take() != Some(e.pointer_id()) {
                return;
            }
            held.set(false);
            on_settle.emit(*latest.borrow());
        })
    };

    let onkeydown = {
        let (root, grip, start, end) = (root.clone(), grip.clone(), start.clone(), end.clone());
        let (on_change, on_settle) = (props.on_change.clone(), props.on_settle.clone());
        let at = props.at;
        Callback::from(move |e: KeyboardEvent| {
            let target = match (e.key().as_str(), axis) {
                ("ArrowLeft", SeamAxis::Columns) | ("ArrowUp", SeamAxis::Rows) => at - STEP,
                ("ArrowRight", SeamAxis::Columns) | ("ArrowDown", SeamAxis::Rows) => at + STEP,
                ("Home", _) => 0.0,
                ("End", _) => 1.0,
                _ => return,
            };
            e.prevent_default();
            let span = measure(&root, &grip, [&start, &end], axis).unwrap_or_default();
            let at = span.clamp(target, min, max);
            on_change.emit(at);
            on_settle.emit(at);
        })
    };

    let ondblclick = {
        let (root, grip, start, end) = (root.clone(), grip.clone(), start.clone(), end.clone());
        let (on_change, on_settle) = (props.on_change.clone(), props.on_settle.clone());
        let reset = props.reset;
        Callback::from(move |_: MouseEvent| {
            let span = measure(&root, &grip, [&start, &end], axis).unwrap_or_default();
            let at = span.clamp(reset, min, max);
            on_change.emit(at);
            on_settle.emit(at);
        })
    };

    let orientation = match axis {
        SeamAxis::Columns => "vertical",
        SeamAxis::Rows => "horizontal",
    };
    let at = props.at.clamp(0.0, 1.0);

    with_testid(
        html! {
            <div ref={root} class={classes!(seam(axis, *held), props.class.clone())}
                style={format!("--sockel-seam-at: {at}")}>
                <div ref={start} class={class::SEAM_PANE}>{ props.start.clone() }</div>
                <div ref={grip} class={class::SEAM_GRIP} role="separator" tabindex="0"
                    aria-orientation={orientation} aria-label={label(&phrases, &props.label, "sockel-seam-resize")}
                    aria-valuemin="0" aria-valuemax="100"
                    aria-valuenow={format!("{:.0}", at * 100.0)}
                    {onpointerdown} {onpointermove}
                    onpointerup={release.clone()} onpointercancel={release}
                    {onkeydown} {ondblclick} />
                <div ref={end} class={class::SEAM_PANE}>{ props.end.clone() }</div>
            </div>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    const SPAN: Span = Span {
        origin: 100.0,
        length: 1010.0,
        grip: 10.0,
        min_start: 100.0,
        min_end: 200.0,
    };

    #[test]
    fn a_seam_takes_its_axis_and_active_modifiers() {
        assert_eq!(names(seam(SeamAxis::Columns, false)), [class::SEAM]);
        assert_eq!(
            names(seam(SeamAxis::Rows, true)),
            [class::SEAM, class::ROWS, class::ACTIVE]
        );
    }

    #[test]
    fn a_pointer_divides_the_room_beside_the_divider() {
        assert_eq!(SPAN.at(605.0, 0.0, 1.0), 0.5);
        assert_eq!(SPAN.at(405.0, 0.0, 1.0), 0.3);
    }

    #[test]
    fn each_pane_keeps_its_least_length() {
        assert_eq!(SPAN.at(100.0, 0.0, 1.0), 0.1);
        assert_eq!(SPAN.at(-5000.0, 0.0, 1.0), 0.1);
        assert_eq!(SPAN.at(1110.0, 0.0, 1.0), 0.8);
        assert_eq!(SPAN.at(9000.0, 0.0, 1.0), 0.8);
    }

    #[test]
    fn the_share_keeps_within_min_and_max() {
        assert_eq!(SPAN.clamp(0.15, 0.25, 0.6), 0.25);
        assert_eq!(SPAN.clamp(0.7, 0.25, 0.6), 0.6);
        assert_eq!(SPAN.clamp(0.05, 0.0, 1.0), 0.1);
    }

    #[test]
    fn a_box_too_short_for_both_panes_keeps_the_first() {
        let short = Span {
            length: 210.0,
            ..SPAN
        };
        assert_eq!(short.clamp(0.9, 0.0, 1.0), 0.5);
        let none = Span {
            length: 5.0,
            ..SPAN
        };
        assert_eq!(none.clamp(0.9, 0.2, 1.0), 0.2);
    }

    #[test]
    fn a_computed_length_reads_as_pixels() {
        assert_eq!(pixels("128px"), 128.0);
        assert_eq!(pixels("0px"), 0.0);
        assert_eq!(pixels("auto"), 0.0);
    }
}
