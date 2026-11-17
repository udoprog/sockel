//! Reorder and DragHandle (`style/_reorder.scss`).

use std::cell::RefCell;
use std::rc::Rc;

use phrasebook02::yew023::use_phrases_with;
use phrasebook02::{Args, Phrases};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use crate::phrasebookcatalog01::L10N;
use crate::tooltip::next_id;
use crate::{Glyph, Icon, RowListItem, class, row_list};

/// How far a pointer travels before a press on a handle becomes a drag.
const SLOP: f64 = 4.0;

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

fn phrase(phrases: &Phrases, text: &ReorderText) -> String {
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

/// Moves `list[from]` to `to`, as [`Reorder`]'s `on_move` reports it.
pub fn move_item<T>(list: &mut Vec<T>, from: usize, to: usize) {
    if from < list.len() {
        let item = list.remove(from);
        list.insert(to.min(list.len()), item);
    }
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
            title={props.title.clone()} disabled={props.disabled} data-testid={props.testid.clone()}
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
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A row of a [`Reorder`]'s children. Outside a `Reorder` it is a plain
/// `<div>` of its children.
#[function_component]
pub fn ReorderItem(props: &ReorderItemProps) -> Html {
    html! {
        <div class={props.class.clone()} data-testid={props.testid.clone()}>
            { props.children.clone() }
        </div>
    }
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

struct Drag {
    id: i32,
    from: usize,
    start: f64,
    middles: Vec<f64>,
    /// Half the lifted row's height.
    half: f64,
    least: f64,
    most: f64,
    active: bool,
    row: Element,
}

#[derive(Default)]
struct Live {
    lift: Option<Lift>,
    drag: Option<Drag>,
    status: String,
    /// The key of a row just moved by keyboard and its index, to focus once
    /// the app has rendered the new order.
    focus: Option<(AttrValue, usize)>,
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

    {
        let live = live.clone();
        let list = list.clone();
        let keys = keys.clone();
        use_effect(move || {
            let pending = live.borrow().focus.clone();
            if let Some((key, to)) = pending
                && keys.get(to) == Some(&key)
            {
                live.borrow_mut().focus = None;
                if let Some(handle) = handle_at(&list, to) {
                    let _ = handle.focus();
                }
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
    let grip = |index: usize, name: AttrValue| Grip {
        ctx: Ctx {
            live: live.clone(),
            redraw: redraw.clone(),
            list: list.clone(),
            on_move: props.on_move.clone(),
            say: say.clone(),
            index,
            count,
            name,
            key: keys[index].clone(),
            quick: props.quick_keys,
        },
        lifted: lift.is_some_and(|l| l.from == index),
        disabled: props.disabled,
        hint: (*hint_id).clone(),
        titles: props.titles,
    };
    let marks = |index: usize| reorder_row(index, lift.map(|l| (l.from, l.to)));
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
                html! {
                    <ContextProvider<Grip> key={item.id.to_string()} context={grip(index, item.name.clone())}>
                        <div class={classes!(marks(index), item.class.clone())} data-testid={item.testid.clone()}>
                            { item.children.clone() }
                        </div>
                    </ContextProvider<Grip>>
                }
            })
            .collect();

        // The rows are the list's first children, the hint and the status
        // after them.
        return html! {
            <div ref={list.clone()} class={classes!(class::REORDER, props.class.clone())} data-testid={props.testid.clone()}>
                { rows }
                { status }
            </div>
        };
    }

    let rows: Html = props
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let handle = grip(index, row.name.clone()).handle(Classes::new(), None);
            html! {
                <RowListItem key={row.key.to_string()} name={html! { { row.name.clone() } }}
                    detail={row.detail.clone()} start={handle}
                    class={marks(index)}>
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
}

impl PartialEq for Grip {
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (&self.ctx, &other.ctx);
        Rc::ptr_eq(&a.live, &b.live)
            && Rc::ptr_eq(&a.say, &b.say)
            && a.on_move == b.on_move
            && (a.index, a.count, a.quick) == (b.index, b.count, b.quick)
            && (&a.name, &a.key) == (&b.name, &b.key)
            && (self.lifted, self.disabled, self.titles)
                == (other.lifted, other.disabled, other.titles)
            && self.hint == other.hint
    }
}

impl Grip {
    fn handle(&self, class: Classes, testid: Option<AttrValue>) -> Html {
        let ctx = &self.ctx;
        let label: AttrValue = (ctx.say)(ReorderText::Handle {
            name: ctx.name.clone(),
        })
        .into();
        html! {
            <DragHandle label={label.clone()} title={self.titles.then_some(label)}
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
    say: Rc<dyn Fn(ReorderText) -> String>,
    index: usize,
    count: usize,
    name: AttrValue,
    key: AttrValue,
    /// The arrows move at once (`quick_keys`).
    quick: bool,
}

impl Ctx {
    fn announce(&self, text: ReorderText) {
        self.live.borrow_mut().status = (self.say)(text);
    }

    fn lifted(&self, at: usize) -> ReorderText {
        ReorderText::Lifted {
            name: self.name.clone(),
            position: at + 1,
            count: self.count,
        }
    }

    fn moving(&self, at: usize) -> ReorderText {
        ReorderText::Moving {
            name: self.name.clone(),
            position: at + 1,
            count: self.count,
        }
    }

    fn cancelled(&self) -> ReorderText {
        ReorderText::Cancelled {
            name: self.name.clone(),
            position: self.index + 1,
            count: self.count,
        }
    }

    /// Ends a lift at `to`: the one place the order changes.
    fn drop_at(&self, to: usize) {
        if to != self.index {
            self.on_move.emit((self.index, to));
            self.live.borrow_mut().focus = Some((self.key.clone(), to));
        }
        self.announce(ReorderText::Dropped {
            name: self.name.clone(),
            position: to + 1,
            count: self.count,
        });
        self.live.borrow_mut().lift = None;
        self.redraw.force_update();
    }

    fn cancel(&self) {
        let drag = self.live.borrow_mut().drag.take();
        if let Some(drag) = drag {
            let _ = drag.row.remove_attribute("style");
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
            let Some(rows) = self.list.cast::<Element>().map(|l| l.children()) else {
                return;
            };
            let Some(row) = rows.item(self.index as u32) else {
                return;
            };
            e.prevent_default();
            let _ = handle.focus();
            let _ = handle.set_pointer_capture(e.pointer_id());
            let scroll = web_sys::window()
                .and_then(|w| w.scroll_y().ok())
                .unwrap_or_default();
            let rects: Vec<_> = (0..self.count as u32)
                .filter_map(|i| rows.item(i))
                .map(|r| r.get_bounding_client_rect())
                .collect();
            let own = row.get_bounding_client_rect();
            let (Some(first), Some(last)) = (rects.first(), rects.last()) else {
                return;
            };
            let drag = Drag {
                id: e.pointer_id(),
                from: self.index,
                start: f64::from(e.page_y()),
                middles: rects
                    .iter()
                    .map(|r| r.top() + r.height() / 2.0 + scroll)
                    .collect(),
                half: own.height() / 2.0,
                least: first.top() - own.top(),
                most: last.bottom() - own.bottom(),
                active: false,
                row,
            };
            self.live.borrow_mut().drag = Some(drag);
        })
    }

    fn pointer_move(self) -> Callback<PointerEvent> {
        Callback::from(move |e: PointerEvent| {
            let mut live = self.live.borrow_mut();
            let Some(drag) = live.drag.as_mut().filter(|d| d.id == e.pointer_id()) else {
                return;
            };
            let dy = f64::from(e.page_y()) - drag.start;
            if !drag.active && dy.abs() < SLOP {
                return;
            }
            let dy = dy.clamp(drag.least, drag.most);
            let _ = drag
                .row
                .set_attribute("style", &format!("--reorder-shift: {dy}px"));
            let middle = drag.middles[drag.from] + dy;
            let to = drag_target(
                &drag.middles,
                drag.from,
                middle - drag.half,
                middle + drag.half,
            );
            let from = drag.from;
            let started = !drag.active;
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
        })
    }

    fn pointer_up(self) -> Callback<PointerEvent> {
        Callback::from(move |e: PointerEvent| {
            let drag = self.live.borrow_mut().drag.take();
            let Some(drag) = drag.filter(|d| d.id == e.pointer_id()) else {
                return;
            };
            let _ = drag.row.remove_attribute("style");
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
                    } else if let Some(to) = key_target(&key, lift.to, self.count) {
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
                    if let Some(to) = key_target(&key, self.index, self.count) {
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
