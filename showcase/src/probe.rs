//! The stability probe: cycles a page's states through its own controls,
//! measures the boxes the page declares fixed in both frames, and reports
//! every box that moved or resized, as text.

// It runs only in the browser; elsewhere its tests use it.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use std::collections::BTreeMap;
use std::fmt::Write as _;

use yew::prelude::*;

use crate::class;

/// A box, relative to its frame's corner.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// For a `Size::Flush` part, the room left between its right edge and
    /// its container's content edge.
    pub gap: f64,
}

/// What of a box must hold.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    Width,
    Height,
    Both,
    /// Its size and its place in the frame.
    Box,
    /// Its right edge reaches the content edge of its closest ancestor of
    /// this class, so the container reserves no scrollbar room beside it, wherever on the page it is open (measured outside the
    /// frames, as overlays are).
    Flush(&'static str),
}

/// The states a box holds across, by control label, or one of its values
/// as `label=value` (`focus` and `click` name the probe's own states).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Across {
    All,
    Only(&'static [&'static str]),
    Except(&'static [&'static str]),
}

impl Across {
    fn covers(self, state: &State) -> bool {
        let named = |names: &[&str]| {
            names.iter().any(|n| match n.split_once('=') {
                Some((control, value)) => control == state.control && value == state.value,
                None => *n == state.control,
            })
        };
        match self {
            Across::All => true,
            Across::Only(names) => named(names),
            Across::Except(names) => !named(names),
        }
    }
}

/// A box a page declares fixed: every element of class `part` in a frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hold {
    pub part: &'static str,
    pub size: Size,
    pub across: Across,
}

/// Where a box was measured: its frame, its part and its index among the
/// part's elements in that frame.
pub type Key = (&'static str, &'static str, usize);

pub type Boxes = BTreeMap<Key, Rect>;

/// The boxes measured in one state.
pub struct State {
    pub control: String,
    pub value: String,
    pub boxes: Boxes,
}

/// A box that changed in a state, by how much; `None` when it was gone.
pub struct Change<'a> {
    pub state: &'a State,
    pub key: Key,
    pub delta: Option<Rect>,
}

/// Below a pixel's half, a change is rounding.
const TOLERANCE: f64 = 0.5;

/// The changes to `hold`'s boxes, against `baseline`, in the states it
/// holds across.
pub fn changes<'a>(hold: &Hold, baseline: &Boxes, states: &'a [State]) -> Vec<Change<'a>> {
    let mut found = Vec::new();

    if let Size::Flush(_) = hold.size {
        for state in states.iter().filter(|s| hold.across.covers(s)) {
            for (key, after) in state.boxes.iter().filter(|(k, _)| k.1 == hold.part) {
                if after.gap.abs() > TOLERANCE {
                    found.push(Change {
                        state,
                        key: *key,
                        delta: Some(Rect {
                            gap: after.gap,
                            ..Rect::default()
                        }),
                    });
                }
            }
        }
        return found;
    }

    for state in states.iter().filter(|s| hold.across.covers(s)) {
        for (key, before) in baseline.iter().filter(|(k, _)| k.1 == hold.part) {
            let Some(after) = state.boxes.get(key) else {
                found.push(Change {
                    state,
                    key: *key,
                    delta: None,
                });
                continue;
            };
            let delta = Rect {
                x: after.x - before.x,
                y: after.y - before.y,
                width: after.width - before.width,
                height: after.height - before.height,
                gap: 0.0,
            };
            let moved = |d: f64| d.abs() > TOLERANCE;
            let changed = match hold.size {
                Size::Width => moved(delta.width),
                Size::Height => moved(delta.height),
                Size::Both => moved(delta.width) || moved(delta.height),
                Size::Box => {
                    moved(delta.width) || moved(delta.height) || moved(delta.x) || moved(delta.y)
                }
                Size::Flush(_) => false,
            };
            if changed {
                found.push(Change {
                    state,
                    key: *key,
                    delta: Some(delta),
                });
            }
        }
    }

    found
}

/// The read-out: one line overall, then one per hold and its first changes.
pub fn report(holds: &[Hold], baseline: &Boxes, states: &[State]) -> (bool, String) {
    let mut out = String::new();
    let mut unstable = 0;

    for hold in holds {
        let measured = match hold.size {
            Size::Flush(_) => states
                .iter()
                .flat_map(|s| s.boxes.keys())
                .filter(|k| k.1 == hold.part)
                .count(),
            _ => baseline.keys().filter(|k| k.1 == hold.part).count(),
        };
        let found = changes(hold, baseline, states);
        let verdict = match (measured, found.is_empty()) {
            (0, _) => "no box found",
            (_, true) => "stable",
            (_, false) => "changed",
        };
        if verdict != "stable" {
            unstable += 1;
        }
        let _ = writeln!(
            out,
            "{} {:?} across {:?}: {verdict} ({measured} boxes, {} changes)",
            hold.part,
            hold.size,
            hold.across,
            found.len()
        );
        for change in found.iter().take(8) {
            let (frame, _, index) = change.key;
            let state = format!("{}={}", change.state.control, change.state.value);
            let _ = match change.delta {
                None => writeln!(out, "  {state} {frame}#{index}: gone"),
                Some(d) if matches!(hold.size, Size::Flush(_)) => {
                    writeln!(
                        out,
                        "  {state} {frame}#{index}: short of the edge by {:.1}",
                        d.gap
                    )
                }
                Some(d) => writeln!(
                    out,
                    "  {state} {frame}#{index}: dx {:+.1} dy {:+.1} dw {:+.1} dh {:+.1}",
                    d.x, d.y, d.width, d.height
                ),
            };
        }
    }

    let stable = unstable == 0 && !holds.is_empty();
    let head = format!(
        "probe: {} ({} states, {unstable} of {} holds changed)\n",
        if stable { "stable" } else { "unstable" },
        states.len(),
        holds.len()
    );
    (stable, head + &out)
}

#[cfg(target_arch = "wasm32")]
mod dom {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        Document, Element, Event, EventInit, HtmlElement, HtmlSelectElement, KeyboardEvent,
        KeyboardEventInit,
    };

    use super::{Boxes, Hold, Rect, Size, State, class};

    fn list(doc: &Document, selectors: &str) -> Vec<Element> {
        let Ok(nodes) = doc.query_selector_all(selectors) else {
            return Vec::new();
        };
        (0..nodes.length())
            .filter_map(|i| nodes.item(i)?.dyn_into().ok())
            .collect()
    }

    fn in_frame(el: &Element) -> bool {
        el.closest(&format!(".{}", class::FRAME))
            .ok()
            .flatten()
            .is_some()
    }

    /// Lets the app render the last change.
    async fn settle() {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            if let Some(window) = web_sys::window() {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 30);
            }
        });
        let _ = JsFuture::from(promise).await;
    }

    fn measure(doc: &Document, holds: &[Hold]) -> Boxes {
        let mut boxes = Boxes::new();
        let frames = doc.get_elements_by_class_name(class::FRAME);

        for f in 0..frames.length() {
            let Some(frame) = frames.item(f) else {
                continue;
            };
            let name = if frame.class_list().contains(class::NARROW) {
                "narrow"
            } else {
                "wide"
            };
            let origin = frame.get_bounding_client_rect();
            for hold in holds.iter().filter(|h| !matches!(h.size, Size::Flush(_))) {
                let parts = frame.get_elements_by_class_name(hold.part);
                for i in 0..parts.length() {
                    let Some(part) = parts.item(i) else {
                        continue;
                    };
                    let rect = part.get_bounding_client_rect();
                    let mut found = Rect {
                        x: rect.x() - origin.x(),
                        y: rect.y() - origin.y(),
                        width: rect.width(),
                        height: rect.height(),
                        gap: flush_gap(&part, &rect, hold),
                    };
                    if let Some((width, height)) = untransformed(&part) {
                        // A turned box (a spinner) is measured as laid out,
                        // about the same centre.
                        found.x += (found.width - width) / 2.0;
                        found.y += (found.height - height) / 2.0;
                        found.width = width;
                        found.height = height;
                    }
                    boxes.insert((name, hold.part, i as usize), found);
                }
            }
        }

        // An overlay opens outside the frames, in the page's own layer.
        for hold in holds.iter().filter(|h| matches!(h.size, Size::Flush(_))) {
            let parts = doc.get_elements_by_class_name(hold.part);
            for i in 0..parts.length() {
                let Some(part) = parts.item(i) else {
                    continue;
                };
                let rect = part.get_bounding_client_rect();
                boxes.insert(
                    ("page", hold.part, i as usize),
                    Rect {
                        x: rect.x(),
                        y: rect.y(),
                        width: rect.width(),
                        height: rect.height(),
                        gap: flush_gap(&part, &rect, hold),
                    },
                );
            }
        }

        boxes
    }

    /// How far a `Size::Flush` part's right edge is from the content edge
    /// of its container; 0 for any other.
    fn flush_gap(part: &Element, rect: &web_sys::DomRect, hold: &Hold) -> f64 {
        let Size::Flush(within) = hold.size else {
            return 0.0;
        };
        let Some(outer) = part.closest(&format!(".{within}")).ok().flatten() else {
            return 0.0;
        };
        let Some(style) =
            web_sys::window().and_then(|w| w.get_computed_style(&outer).ok().flatten())
        else {
            return 0.0;
        };
        let px = |name: &str| {
            style
                .get_property_value(name)
                .ok()
                .and_then(|v| v.trim_end_matches("px").parse::<f64>().ok())
                .unwrap_or(0.0)
        };
        // From the border box, not the client box: a reserved scrollbar
        // gutter is the room between them.
        let inner_right = outer.get_bounding_client_rect().right()
            - px("border-right-width")
            - px("padding-right");
        inner_right - rect.right()
    }

    /// A transformed element's laid-out size, which its client rect is not.
    fn untransformed(part: &Element) -> Option<(f64, f64)> {
        let el = part.dyn_ref::<HtmlElement>()?;
        let style = web_sys::window()?.get_computed_style(part).ok()??;
        let transform = style.get_property_value("transform").ok()?;
        (transform != "none" && !transform.is_empty())
            .then(|| (f64::from(el.offset_width()), f64::from(el.offset_height())))
    }

    fn label_of(control: &Element) -> String {
        control
            .closest("label")
            .ok()
            .flatten()
            .and_then(|l| {
                l.query_selector(&format!(".{}", class::CONTROL_LABEL))
                    .ok()
                    .flatten()
            })
            .and_then(|l| l.text_content())
            .unwrap_or_default()
            .trim()
            .to_owned()
    }

    fn change(el: &Element) {
        let init = EventInit::new();
        init.set_bubbles(true);
        if let Ok(event) = Event::new_with_event_init_dict("change", &init) {
            let _ = el.dispatch_event(&event);
        }
    }

    /// Closes what a click opened, as a user's Escape would.
    fn escape(el: &Element) {
        let init = KeyboardEventInit::new();
        init.set_bubbles(true);
        init.set_key("Escape");
        if let Ok(event) = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init) {
            let _ = el.dispatch_event(&event);
        }
    }

    /// Every state, each one control away from the page's own.
    pub async fn run(holds: &[Hold]) -> (Boxes, Vec<State>) {
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            return (Boxes::new(), Vec::new());
        };
        let mut states = Vec::new();
        let baseline = measure(&doc, holds);

        let controls = list(&doc, "main select, main input[type=checkbox]");
        for control in controls.iter().filter(|c| !in_frame(c)) {
            let label = label_of(control);
            if let Some(select) = control.dyn_ref::<HtmlSelectElement>() {
                let own = select.selected_index();
                for i in (0..select.length() as i32).filter(|i| *i != own) {
                    let value = select
                        .item(i as u32)
                        .and_then(|o| o.text_content())
                        .unwrap_or_default();
                    select.set_selected_index(i);
                    change(control);
                    settle().await;
                    states.push(State {
                        control: label.clone(),
                        value,
                        boxes: measure(&doc, holds),
                    });
                }
                select.set_selected_index(own);
                change(control);
                settle().await;
            } else if let Some(input) = control.dyn_ref::<HtmlElement>() {
                input.click();
                settle().await;
                states.push(State {
                    control: label,
                    value: "toggled".into(),
                    boxes: measure(&doc, holds),
                });
                input.click();
                settle().await;
            }
        }

        let focusable = list(&doc, "main button, main a[href], main input, main select");
        for (i, el) in focusable.iter().filter(|e| in_frame(e)).enumerate() {
            if let Some(el) = el.dyn_ref::<HtmlElement>() {
                let _ = el.focus();
                settle().await;
                states.push(State {
                    control: "focus".into(),
                    value: i.to_string(),
                    boxes: measure(&doc, holds),
                });
                let _ = el.blur();
            }
        }

        let buttons = list(&doc, "main button");
        for (i, el) in buttons.iter().filter(|e| in_frame(e)).enumerate() {
            if let Some(el) = el.dyn_ref::<HtmlElement>() {
                el.click();
                settle().await;
                states.push(State {
                    control: "click".into(),
                    value: i.to_string(),
                    boxes: measure(&doc, holds),
                });
                escape(el);
                settle().await;
            }
        }
        settle().await;

        (baseline, states)
    }

    /// Whether the address asks for a run on load (`?probe`).
    pub fn asked() -> bool {
        web_sys::window()
            .and_then(|w| w.location().search().ok())
            .is_some_and(|s| s == "?probe")
    }
}

#[derive(Properties, PartialEq)]
pub struct ProbeProps {
    pub holds: &'static [Hold],
}

/// The probe's panel, fixed to the window's corner so it never moves the
/// page. Its `<pre>` is the read-out, and `data-probe` its verdict.
#[function_component]
pub fn Probe(props: &ProbeProps) -> Html {
    let result = use_state(|| None::<(bool, String)>);
    let running = use_state(|| false);

    let start = {
        let result = result.clone();
        let running = running.clone();
        let holds = props.holds;
        Callback::from(move |()| {
            if *running {
                return;
            }
            running.set(true);
            #[cfg(target_arch = "wasm32")]
            {
                let result = result.clone();
                let running = running.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let (baseline, states) = dom::run(holds).await;
                    result.set(Some(report(holds, &baseline, &states)));
                    running.set(false);
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = (&result, holds);
        })
    };

    {
        let start = start.clone();
        use_effect_with((), move |()| {
            #[cfg(target_arch = "wasm32")]
            if dom::asked() {
                start.emit(());
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = start;
        });
    }

    let verdict = match (&*result, *running) {
        (_, true) => "running",
        (None, false) => "idle",
        (Some((true, _)), false) => "stable",
        (Some((false, _)), false) => "unstable",
    };
    let text = result
        .as_ref()
        .map(|(_, text)| text.clone())
        .unwrap_or_else(|| "Run the probe to cycle this page's states.".into());

    html! {
        <aside class={class::PROBE} aria-label="Stability probe" data-probe={verdict}>
            <button type="button" class={classes!(class::BUTTON, class::SMALL)}
                disabled={*running} onclick={start.reform(|_: MouseEvent| ())}>
                { "Run the probe" }
            </button>
            <pre class={class::PROBE_OUT}>{ text }</pre>
        </aside>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(width: f64, height: f64) -> Rect {
        Rect {
            width,
            height,
            ..Rect::default()
        }
    }

    fn state(control: &str, boxes: &[(Key, Rect)]) -> State {
        State {
            control: control.into(),
            value: "v".into(),
            boxes: boxes.iter().copied().collect(),
        }
    }

    const PART: &str = "part";
    const KEY: Key = ("narrow", PART, 0);

    #[test]
    fn a_change_counts_only_in_the_size_held_and_past_rounding() {
        let baseline: Boxes = [(KEY, rect(100.0, 20.0))].into_iter().collect();
        let states = [
            state("Text", &[(KEY, rect(100.4, 40.0))]),
            state("Text", &[(KEY, rect(120.0, 20.0))]),
        ];
        let width = Hold {
            part: PART,
            size: Size::Width,
            across: Across::All,
        };

        let found = changes(&width, &baseline, &states);
        assert_eq!(found.len(), 1);
        assert!(std::ptr::eq(found[0].state, &states[1]));
        assert_eq!(found[0].delta.map(|d| d.width), Some(20.0));

        let both = Hold {
            size: Size::Both,
            ..width
        };
        assert_eq!(changes(&both, &baseline, &states).len(), 2);
    }

    #[test]
    fn a_box_holds_only_across_its_states_and_a_gone_box_is_a_change() {
        let baseline: Boxes = [(KEY, rect(100.0, 20.0))].into_iter().collect();
        let states = [
            state("Text", &[]),
            state("focus", &[(KEY, rect(100.0, 20.0))]),
        ];
        let only_focus = Hold {
            part: PART,
            size: Size::Box,
            across: Across::Only(&["focus"]),
        };
        assert!(changes(&only_focus, &baseline, &states).is_empty());

        let all_but_focus = Hold {
            across: Across::Except(&["focus"]),
            ..only_focus
        };
        let found = changes(&all_but_focus, &baseline, &states);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].delta, None);

        let one_value = Hold {
            across: Across::Only(&["Text=w"]),
            ..only_focus
        };
        assert!(changes(&one_value, &baseline, &states).is_empty());
        let one_value = Hold {
            across: Across::Only(&["Text=v"]),
            ..only_focus
        };
        assert_eq!(changes(&one_value, &baseline, &states).len(), 1);
    }

    #[test]
    fn a_move_counts_only_for_a_whole_box() {
        let baseline: Boxes = [(KEY, rect(100.0, 20.0))].into_iter().collect();
        let moved = Rect {
            y: 8.0,
            ..rect(100.0, 20.0)
        };
        let states = [state("Text", &[(KEY, moved)])];
        let hold = Hold {
            part: PART,
            size: Size::Both,
            across: Across::All,
        };
        assert!(changes(&hold, &baseline, &states).is_empty());
        let hold = Hold {
            size: Size::Box,
            ..hold
        };
        assert_eq!(changes(&hold, &baseline, &states).len(), 1);
    }

    #[test]
    fn the_report_reads_stable_only_when_no_hold_changed_or_went_unmeasured() {
        let baseline: Boxes = [(KEY, rect(100.0, 20.0))].into_iter().collect();
        let states = [state("Text", &[(KEY, rect(100.0, 20.0))])];
        let hold = Hold {
            part: PART,
            size: Size::Both,
            across: Across::All,
        };
        let (stable, text) = report(&[hold], &baseline, &states);
        assert!(stable, "{text}");
        assert!(text.starts_with("probe: stable"), "{text}");

        let missing = Hold {
            part: "other",
            ..hold
        };
        let (stable, text) = report(&[hold, missing], &baseline, &states);
        assert!(!stable);
        assert!(
            text.contains("other Both across All: no box found"),
            "{text}"
        );
    }
}
