//! ReorderTree and ReorderList: rows moved within and between nested lists
//! (`style/_reorder.scss`).

use std::cell::RefCell;
use std::rc::Rc;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use crate::phrasebookcatalog01::L10N;
use crate::reorder::{
    CONTROLS, ReorderItem, ReorderText, SURFACE, phrase, phrase_in, place, scrolled, scroller_of,
    with_data,
};
use crate::tooltip::next_id;
use crate::{DragHandle, class, slot_at};

const SLOP: f64 = 4.0;
const LIST: &str = "data-reorder-list";
/// A list's `name`, which the announcements placing a row in it say.
const LIST_NAME: &str = "data-reorder-list-name";
/// A row that stays put, which the positions announced do not count.
const FIXED: &str = "data-reorder-fixed";

/// A row's place: the list it is in (a [`ReorderList`]'s `id`) and its
/// index there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReorderSpot {
    pub list: AttrValue,
    pub index: usize,
}

/// A row moved between or within the lists of a [`ReorderTree`]: `to.index`
/// is an index in its list without the row, as [`move_item`](crate::move_item)
/// takes within one list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReorderTreeMove {
    pub from: ReorderSpot,
    pub to: ReorderSpot,
}

/// Where a row lifted from `from` aims: before row `slot` of `list` (the
/// gap, `0..=len`, counted with the row still in place).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Aim {
    list: AttrValue,
    slot: usize,
}

impl Aim {
    fn stays(&self, from: &ReorderSpot) -> bool {
        self.list == from.list && (self.slot == from.index || self.slot == from.index + 1)
    }

    fn landing(&self, from: &ReorderSpot) -> ReorderSpot {
        let index = if self.list == from.list && self.slot > from.index {
            self.slot - 1
        } else {
            self.slot
        };
        ReorderSpot {
            list: self.list.clone(),
            index,
        }
    }
}

/// One list of a tree in document order: where it sits (the list and row
/// it is inside, by index into the same slice) and how many rows it has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeList {
    pub parent: Option<(usize, usize)>,
    pub len: usize,
}

/// Every drop gap of a tree's lists in reading order, as (list, slot): a
/// list's gap before row `k`, then the lists inside row `k`, then the gap
/// after it, so the arrows step into a nested list and back out of it.
pub fn tree_slots(lists: &[TreeList]) -> Vec<(usize, usize)> {
    fn walk(lists: &[TreeList], at: usize, out: &mut Vec<(usize, usize)>) {
        for row in 0..lists[at].len {
            out.push((at, row));
            for (inner, list) in lists.iter().enumerate() {
                if list.parent == Some((at, row)) {
                    walk(lists, inner, out);
                }
            }
        }
        out.push((at, lists[at].len));
    }

    let mut out = Vec::new();
    for (at, list) in lists.iter().enumerate() {
        if list.parent.is_none() {
            walk(lists, at, &mut out);
        }
    }
    out
}

struct Lift {
    from: ReorderSpot,
    key: AttrValue,
    name: AttrValue,
    aim: Aim,
    pointer: bool,
}

struct Drag {
    id: i32,
    start: (f64, f64),
    row: Element,
    active: bool,
    scroller: Option<Element>,
    /// Held by the row itself (`body_drag`, a row without a handle), so the
    /// row hears the pointer, not a handle.
    by_row: bool,
}

#[derive(Default)]
struct Live {
    lift: Option<Lift>,
    drag: Option<Drag>,
    status: String,
    /// The key of a row just moved by keyboard, to focus once rendered.
    focus: Option<AttrValue>,
    /// A drag just ended, so the click it ends in is not the row's.
    dragged: bool,
}

/// The catalog's text of a row placed in a named list.
type SayIn = Rc<dyn Fn(ReorderText, &str) -> String>;

/// What a [`ReorderTree`]'s lists and handles share.
#[derive(Clone)]
struct Tree {
    live: Rc<RefCell<Live>>,
    version: u32,
    bump: Callback<()>,
    root: NodeRef,
    on_move: Callback<ReorderTreeMove>,
    accepts: Option<Callback<ReorderTreeMove, bool>>,
    say: Rc<dyn Fn(ReorderText) -> String>,
    /// The catalog's text for a row placed in a named list; `None` when
    /// the app gives `text`.
    say_in: Option<SayIn>,
    hint: AttrValue,
    disabled: bool,
    titles: bool,
    tips: bool,
    side_by_side: bool,
    body_drag: bool,
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.live, &other.live)
            && Rc::ptr_eq(&self.say, &other.say)
            && self.version == other.version
            && self.on_move == other.on_move
            && self.accepts == other.accepts
            && (self.disabled, self.titles, self.tips) == (other.disabled, other.titles, other.tips)
            && (self.side_by_side, self.body_drag) == (other.side_by_side, other.body_drag)
    }
}

#[derive(Properties, PartialEq)]
pub struct ReorderTreeProps {
    /// A row moved: from where to where.
    pub on_move: Callback<ReorderTreeMove>,
    /// Whether a row may go to a place; every place when `None`. A row
    /// never goes into a list inside itself.
    #[prop_or_default]
    pub accepts: Option<Callback<ReorderTreeMove, bool>>,
    /// The texts, overriding the catalog's.
    #[prop_or_default]
    pub text: Option<Callback<ReorderText, String>>,
    #[prop_or_default]
    pub disabled: bool,
    /// Each handle's name is its tooltip too.
    #[prop_or_default]
    pub titles: bool,
    /// Each handle's name is its `data-tip` tooltip too, the attribute an
    /// app's own tooltips read.
    #[prop_or_default]
    pub tips: bool,
    /// The outermost lists sit side by side (lanes, columns): Left and
    /// Right move a lifted row to the list beside, at the same position or
    /// its end, and Up, Down, Home and End stay within its list.
    #[prop_or_default]
    pub side_by_side: bool,
    /// A mouse or pen also drags a row from anywhere on it off its
    /// controls (as [`Reorder`](crate::Reorder)'s `body_drag`), a finger
    /// only by the handle; a row without a handle is dragged by itself.
    #[prop_or_default]
    pub body_drag: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The app's markup, holding the [`ReorderList`]s at any depth.
    #[prop_or_default]
    pub children: Html,
}

/// Nested lists whose rows move within a list and between lists, by a drag
/// on their [`ReorderTreeHandle`] or by keyboard: Space lifts, the arrows
/// step through every place the row may go in reading order (into a list
/// inside a row and back out of it), Home and End aim at the ends of the
/// current list, Space drops and Escape cancels; Alt and an arrow moves at
/// once. A dragged row follows the pointer and aims at the innermost list
/// under it that takes it. As in [`Reorder`](crate::Reorder) nothing
/// reflows until the drop.
#[function_component]
pub fn ReorderTree(props: &ReorderTreeProps) -> Html {
    let live = use_mut_ref(Live::default);
    let version = use_state(|| 0u32);
    let root = use_node_ref();
    let hint = use_state(|| next_id("reorder-hint"));
    let phrases = use_phrases_with(&L10N);
    let say_in: Option<SayIn> = match &props.text {
        Some(_) => None,
        None => {
            let phrases = phrases.clone();
            Some(Rc::new(move |t, list| phrase_in(&phrases, &t, list)))
        }
    };
    let say: Rc<dyn Fn(ReorderText) -> String> = match &props.text {
        Some(text) => {
            let text = text.clone();
            Rc::new(move |t| text.emit(t))
        }
        None => Rc::new(move |t| phrase(&phrases, &t)),
    };
    let bump = {
        let version = version.clone();
        Callback::from(move |()| version.set(version.wrapping_add(1)))
    };

    {
        let live = live.clone();
        let root = root.clone();
        use_effect(move || {
            let pending = live.borrow().focus.clone();
            if let Some(key) = pending
                && let Some(handle) = handle_of(&root, &key)
            {
                live.borrow_mut().focus = None;
                let _ = handle.focus();
            }
        });
    }

    let tree = Tree {
        live: live.clone(),
        version: *version,
        bump,
        root: root.clone(),
        on_move: props.on_move.clone(),
        accepts: props.accepts.clone(),
        say: say.clone(),
        say_in,
        hint: (*hint).clone(),
        disabled: props.disabled,
        titles: props.titles,
        tips: props.tips,
        side_by_side: props.side_by_side,
        body_drag: props.body_drag,
    };

    html! {
        <div ref={root} class={classes!(class::REORDER, props.class.clone())} data-testid={props.testid.clone()}>
            <ContextProvider<Tree> context={tree}>
                { props.children.clone() }
            </ContextProvider<Tree>>
            <span id={(*hint).clone()} class={class::VISUALLY_HIDDEN}>{ say(ReorderText::Hint) }</span>
            <span class={class::VISUALLY_HIDDEN} role="status" aria-live="assertive">
                { live.borrow().status.clone() }
            </span>
        </div>
    }
}

fn handle_of(root: &NodeRef, key: &str) -> Option<HtmlElement> {
    let root = root.cast::<Element>()?;
    let row = root
        .query_selector(&format!(
            "[data-reorder-key=\"{}\"]",
            key.replace('"', "\\\"")
        ))
        .ok()??;
    row.query_selector(&format!(".{}", class::DRAG_HANDLE))
        .ok()??
        .dyn_into()
        .ok()
}

#[derive(Properties, PartialEq)]
pub struct ReorderListProps {
    /// The list's name in the tree's moves, unique within it.
    pub id: AttrValue,
    /// What the list is called (its `aria-label`): the announcements that
    /// place a row in it say it ("Moved Alpha to Done, 2 of 5"), unless the
    /// tree's `text` is given.
    #[prop_or_default]
    pub name: Option<AttrValue>,
    /// The rows sit side by side: the drop is marked at a row's side and
    /// a drag aims by the pointer's position across.
    #[prop_or_default]
    pub across: bool,
    /// Each row takes focus on a tap (`tabindex="-1"`), so what shows on
    /// focus within it shows on a touch screen too.
    #[prop_or_default]
    pub focusable: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The rows, each with a [`ReorderTreeHandle`] in it, as in
    /// [`Reorder`](crate::Reorder): a `fixed` row has no handle and stays,
    /// a `lifted_only` one shows only while a row is lifted, a `disabled`
    /// one cannot be lifted; `picked` is `Reorder`'s alone.
    #[prop_or_default]
    pub children: ChildrenWithProps<ReorderItem>,
}

/// One list of a [`ReorderTree`], at any depth inside it, laid out by
/// `class`. An empty list aimed at wears `DROP_INSIDE`.
#[function_component]
pub fn ReorderList(props: &ReorderListProps) -> Html {
    let Some(tree) = use_context::<Tree>() else {
        return html! { <div class={props.class.clone()}>{ for props.children.iter() }</div> };
    };
    let len = props.children.len();
    let (lifted, aim, lifting) = {
        let live = tree.live.borrow();
        let lift = live.lift.as_ref();
        let lifting = lift.is_some();
        let lifted = lift
            .filter(|l| l.from.list == props.id)
            .map(|l| l.from.index);
        let aim = lift
            .filter(|l| l.aim.list == props.id && !l.aim.stays(&l.from))
            .map(|l| l.aim.slot);
        (lifted, aim, lifting)
    };
    let rows: Html = props
        .children
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item = &item.props;
            let mut classes = classes!(class::REORDER_ROW, item.class.clone());
            if props.across {
                classes.push(class::REORDER_TILED);
            }
            let still = item.fixed || item.disabled || tree.disabled;
            if tree.body_drag && !still {
                classes.push(class::REORDER_GRAB);
            }
            if lifted == Some(index) {
                classes.push(class::LIFTED);
            }
            if aim == Some(index) {
                classes.push(class::DROP_BEFORE);
            } else if aim == Some(len) && index + 1 == len {
                classes.push(class::DROP_AFTER);
            }
            let grip = TreeGrip {
                tree: tree.clone(),
                spot: ReorderSpot {
                    list: props.id.clone(),
                    index,
                },
                key: item.id.clone(),
                name: item.name.clone(),
                lifted: lifted == Some(index),
                fixed: item.fixed,
                disabled: item.disabled,
            };
            let body = tree.body_drag.then(|| {
                let (down, moved, up, cancel) = (grip.clone(), grip.clone(), grip.clone(), grip.clone());
                (
                    Callback::from(move |e: PointerEvent| down.body_down(e)),
                    Callback::from(move |e: PointerEvent| if moved.by_row() { moved.pointer_move(&e) }),
                    Callback::from(move |e: PointerEvent| if up.by_row() { up.pointer_up(&e) }),
                    Callback::from(move |_: PointerEvent| if cancel.by_row() { cancel.cancel() }),
                )
            });
            let (onpointerdown, onpointermove, onpointerup, onpointercancel) = match body {
                Some((down, moved, up, cancel)) => (Some(down), Some(moved), Some(up), Some(cancel)),
                None => (None, None, None, None),
            };
            let onclick = {
                let live = tree.live.clone();
                let onclick = item.onclick.clone();
                Callback::from(move |e: MouseEvent| {
                    if !std::mem::take(&mut live.borrow_mut().dragged) {
                        onclick.emit(e);
                    }
                })
            };
            let row = html! {
                <div class={classes} data-testid={item.testid.clone()} data-reorder-key={item.id.clone()}
                    data-reorder-fixed={item.fixed.then_some("")} hidden={item.lifted_only && !lifting}
                    tabindex={props.focusable.then_some("-1")} aria-current={item.current.then_some("true")}
                    {onclick} ondblclick={item.ondblclick.clone()} oncontextmenu={item.oncontextmenu.clone()}
                    {onpointerdown} {onpointermove} {onpointerup} {onpointercancel}
                    onmouseenter={item.onmouseenter.clone()} onmouseleave={item.onmouseleave.clone()}>
                    { item.children.clone() }
                </div>
            };
            html! {
                <ContextProvider<TreeGrip> key={item.id.to_string()} context={grip}>
                    { with_data(row, &item.data) }
                </ContextProvider<TreeGrip>>
            }
        })
        .collect();
    let inside = (len == 0 && aim == Some(0)).then_some(class::DROP_INSIDE);
    html! {
        <div class={classes!(props.class.clone(), inside)} data-testid={props.testid.clone()}
            data-reorder-list={props.id.clone()} data-reorder-across={props.across.then_some("")}
            data-reorder-list-name={props.name.clone()} aria-label={props.name.clone()}>
            { rows }
        </div>
    }
}

#[derive(Clone, PartialEq)]
struct TreeGrip {
    tree: Tree,
    spot: ReorderSpot,
    key: AttrValue,
    name: AttrValue,
    lifted: bool,
    /// The row stays put and has no handle ([`ReorderItemProps::fixed`](crate::ReorderItemProps::fixed)).
    fixed: bool,
    disabled: bool,
}

#[derive(Properties, PartialEq)]
pub struct ReorderTreeHandleProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// The [`DragHandle`] of the [`ReorderItem`] of a [`ReorderList`] it is in;
/// nothing outside one.
#[function_component]
pub fn ReorderTreeHandle(props: &ReorderTreeHandleProps) -> Html {
    let Some(grip) = use_context::<TreeGrip>().filter(|g| !g.fixed) else {
        return Html::default();
    };
    let tree = &grip.tree;
    let label: AttrValue = (tree.say)(ReorderText::Handle {
        name: grip.name.clone(),
    })
    .into();
    let down = grip.clone();
    let moved = grip.clone();
    let up = grip.clone();
    let cancel = grip.clone();
    let key = grip.clone();
    let blur = grip.clone();
    html! {
        <DragHandle label={label.clone()} title={tree.titles.then(|| label.clone())} tip={tree.tips.then_some(label)}
            lifted={grip.lifted} disabled={tree.disabled || grip.disabled}
            describedby={tree.hint.clone()} class={props.class.clone()} testid={props.testid.clone()}
            onpointerdown={Callback::from(move |e| down.pointer_down(e))}
            onpointermove={Callback::from(move |e: PointerEvent| moved.pointer_move(&e))}
            onpointerup={Callback::from(move |e: PointerEvent| up.pointer_up(&e))}
            onpointercancel={Callback::from(move |_| cancel.cancel())}
            onkeydown={Callback::from(move |e| key.key_down(e))}
            onblur={Callback::from(move |_| blur.blur())} />
    }
}

/// The lists inside `root` in document order, as [`tree_slots`] takes
/// them, with their elements and ids.
fn lists_of(root: &Element) -> Vec<(Element, AttrValue, TreeList)> {
    let Ok(found) = root.query_selector_all(&format!("[{LIST}]")) else {
        return Vec::new();
    };
    let elements: Vec<Element> = (0..found.length())
        .filter_map(|i| found.item(i))
        .filter_map(|n| n.dyn_into::<Element>().ok())
        .collect();
    let mut out = Vec::new();
    for el in &elements {
        let id: AttrValue = el.get_attribute(LIST).unwrap_or_default().into();
        let parent = el
            .parent_element()
            .and_then(|p| p.closest(&format!("[{LIST}]")).ok().flatten())
            .and_then(|outer| {
                let at = elements.iter().position(|e| *e == outer)?;
                let row = (0..outer.children().length()).position(|i| {
                    outer
                        .children()
                        .item(i)
                        .is_some_and(|r| r.contains(Some(el)))
                })?;
                Some((at, row))
            });
        let len = el.children().length() as usize;
        out.push((el.clone(), id, TreeList { parent, len }));
    }
    out
}

impl TreeGrip {
    /// Whether the row is held by itself, not by its handle.
    fn by_row(&self) -> bool {
        self.tree
            .live
            .borrow()
            .drag
            .as_ref()
            .is_some_and(|d| d.by_row)
    }

    /// Says `text` of the row placed in `list`, naming the list when it
    /// has a name and the catalog speaks.
    fn announce(&self, text: ReorderText, list: &AttrValue) {
        let named = self.tree.say_in.as_ref().zip(self.list_name(list));
        self.tree.live.borrow_mut().status = match named {
            Some((say_in, name)) => say_in(text, &name),
            None => (self.tree.say)(text),
        };
    }

    fn list_name(&self, list: &AttrValue) -> Option<String> {
        let root = self.tree.root.cast::<Element>()?;
        root.query_selector(&format!("[{LIST}=\"{}\"]", list.replace('"', "\\\"")))
            .ok()??
            .get_attribute(LIST_NAME)
    }

    fn row(&self) -> Option<Element> {
        let root = self.tree.root.cast::<Element>()?;
        root.query_selector(&format!(
            "[data-reorder-key=\"{}\"]",
            self.key.replace('"', "\\\"")
        ))
        .ok()?
    }

    /// Whether the lifted row may go to `aim`: not into itself, and as the
    /// tree's `accepts` says.
    fn allows(&self, aim: &Aim, list: &Element) -> bool {
        if self.row().is_some_and(|row| row.contains(Some(list))) {
            return false;
        }
        if aim.stays(&self.spot) {
            return true;
        }
        match &self.tree.accepts {
            Some(accepts) => accepts.emit(ReorderTreeMove {
                from: self.spot.clone(),
                to: aim.landing(&self.spot),
            }),
            None => true,
        }
    }

    /// The row's position landing at `aim`, from 1, and how many rows move
    /// in that list: fixed rows are not counted.
    fn place_text(&self, aim: &Aim, lists: &[(Element, AttrValue, TreeList)]) -> (usize, usize) {
        let stays = lists
            .iter()
            .find(|(_, id, _)| *id == aim.list)
            .map_or_else(Vec::new, |(el, _, _)| fixed_rows(el));
        tree_place(&stays, aim.list == self.spot.list, &self.spot, aim)
    }

    /// Whether the row may be lifted at all.
    fn still(&self) -> bool {
        self.tree.disabled || self.fixed || self.disabled
    }

    fn set_aim(&self, aim: Aim, pointer: bool) {
        let list = aim.list.clone();
        let root = self.tree.root.cast::<Element>();
        let lists = root.as_ref().map(lists_of).unwrap_or_default();
        let at = |aim: &Aim| self.place_text(aim, &lists);
        let (position, count) = if self.tree.live.borrow().lift.is_none() {
            // Lifted where it lies, wherever a drag first aims it.
            at(&Aim {
                list: self.spot.list.clone(),
                slot: self.spot.index,
            })
        } else {
            at(&aim)
        };
        let first = {
            let mut live = self.tree.live.borrow_mut();
            let first = live.lift.is_none();
            if live.lift.as_ref().is_some_and(|l| l.aim == aim) {
                return;
            }
            live.lift = Some(Lift {
                from: self.spot.clone(),
                key: self.key.clone(),
                name: self.name.clone(),
                aim,
                pointer,
            });
            first
        };
        let name = self.name.clone();
        let text = if first {
            ReorderText::Lifted {
                name,
                position,
                count,
            }
        } else {
            ReorderText::Moving {
                name,
                position,
                count,
            }
        };
        // Lifted, it is still where it lies, whatever a drag first aims at.
        let list = if first { self.spot.list.clone() } else { list };
        self.announce(text, &list);
        self.tree.bump.emit(());
    }

    fn drop_lift(&self) {
        let lift = self.tree.live.borrow_mut().lift.take();
        let Some(lift) = lift else {
            return;
        };
        let lists = self
            .tree
            .root
            .cast::<Element>()
            .map(|r| lists_of(&r))
            .unwrap_or_default();
        let (position, count) = self.place_text(&lift.aim, &lists);
        if !lift.aim.stays(&lift.from) {
            self.tree.on_move.emit(ReorderTreeMove {
                from: lift.from.clone(),
                to: lift.aim.landing(&lift.from),
            });
            self.tree.live.borrow_mut().focus = Some(lift.key.clone());
        }
        self.announce(
            ReorderText::Dropped {
                name: lift.name,
                position,
                count,
            },
            &lift.aim.list,
        );
        self.tree.bump.emit(());
    }

    fn cancel(&self) {
        let drag = self.tree.live.borrow_mut().drag.take();
        if let Some(drag) = drag {
            let _ = drag.row.remove_attribute("style");
        }
        if self.tree.live.borrow_mut().lift.take().is_some() {
            let lists = self
                .tree
                .root
                .cast::<Element>()
                .map(|r| lists_of(&r))
                .unwrap_or_default();
            let here = Aim {
                list: self.spot.list.clone(),
                slot: self.spot.index,
            };
            let (position, count) = self.place_text(&here, &lists);
            self.announce(
                ReorderText::Cancelled {
                    name: self.name.clone(),
                    position,
                    count,
                },
                &self.spot.list,
            );
        }
        self.tree.bump.emit(());
    }

    fn pointer_down(&self, e: PointerEvent) {
        if e.button() != 0 || self.still() || self.tree.live.borrow().lift.is_some() {
            return;
        }
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
        self.begin(&e, &handle, false);
    }

    /// A press on the row off its handle and controls (`body_drag`), by a
    /// mouse or pen: the row's handle takes the pointer, or the row itself
    /// when it has none.
    fn body_down(&self, e: PointerEvent) {
        if !self.tree.body_drag
            || e.button() != 0
            || e.pointer_type() == "touch"
            || self.still()
            || self.tree.live.borrow().lift.is_some()
        {
            return;
        }
        let Some(target) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
            return;
        };
        let Some(row) = self.row() else {
            return;
        };
        let handle_class = format!(".{}", class::DRAG_HANDLE);
        // A press in a portal (a menu the row opened) bubbles through the
        // row in Yew without being in its element; one on the handle is the
        // handle's.
        if !row.contains(Some(&target)) || target.closest(&handle_class).ok().flatten().is_some() {
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
        let handle = row.query_selector(&handle_class).ok().flatten();
        let by_row = handle.is_none();
        let Ok(handle) = handle.unwrap_or(row).dyn_into::<HtmlElement>() else {
            return;
        };
        self.begin(&e, &handle, by_row);
    }

    fn begin(&self, e: &PointerEvent, handle: &HtmlElement, by_row: bool) {
        let Some(row) = self.row() else {
            return;
        };
        e.prevent_default();
        let _ = handle.focus();
        let _ = handle.set_pointer_capture(e.pointer_id());
        self.tree.live.borrow_mut().dragged = false;
        let scroller = scroller_of(&row);
        let (ox, oy) = scrolled(scroller.as_ref());
        self.tree.live.borrow_mut().drag = Some(Drag {
            id: e.pointer_id(),
            start: (f64::from(e.client_x()) + ox, f64::from(e.client_y()) + oy),
            row,
            active: false,
            scroller,
            by_row,
        });
    }

    fn pointer_move(&self, e: &PointerEvent) {
        let at = (f64::from(e.client_x()), f64::from(e.client_y()));
        {
            let mut live = self.tree.live.borrow_mut();
            let Some(drag) = live.drag.as_mut().filter(|d| d.id == e.pointer_id()) else {
                return;
            };
            let (ox, oy) = scrolled(drag.scroller.as_ref());
            let (dx, dy) = (at.0 + ox - drag.start.0, at.1 + oy - drag.start.1);
            if !drag.active && dx.hypot(dy) < SLOP {
                return;
            }
            drag.active = true;
            let _ = drag.row.set_attribute(
                "style",
                &format!("--reorder-shift: {dy}px; --reorder-shift-x: {dx}px"),
            );
        }
        if let Some(aim) = self.aim_at(at) {
            self.set_aim(aim, true);
        } else if self.tree.live.borrow().lift.is_none() {
            self.set_aim(
                Aim {
                    list: self.spot.list.clone(),
                    slot: self.spot.index,
                },
                true,
            );
        }
    }

    /// Where the row dragged to client `at` aims: the innermost list under
    /// the pointer that takes it, else the lists around that one.
    fn aim_at(&self, at: (f64, f64)) -> Option<Aim> {
        let document = web_sys::window()?.document()?;
        let row = self.row()?;
        let root = self.tree.root.cast::<Element>()?;
        let under = document
            .elements_from_point(at.0 as f32, at.1 as f32)
            .iter()
            .filter_map(|n| n.dyn_into::<Element>().ok())
            .find(|el| !row.contains(Some(el)) && root.contains(Some(el)))?;
        let mut list = under.closest(&format!("[{LIST}]")).ok().flatten();
        while let Some(el) = list {
            if !root.contains(Some(&el)) {
                break;
            }
            let across = el.has_attribute("data-reorder-across");
            let rows = el.children();
            let middles: Vec<f64> = (0..rows.length())
                .filter_map(|i| rows.item(i))
                .map(|r| {
                    let rect = r.get_bounding_client_rect();
                    // A row without a box (lifted_only, not yet shown) is
                    // passed by no pointer.
                    if rect.width() == 0.0 && rect.height() == 0.0 {
                        f64::NAN
                    } else if across {
                        rect.left() + rect.width() / 2.0
                    } else {
                        rect.top() + rect.height() / 2.0
                    }
                })
                .collect();
            let aim = Aim {
                list: el.get_attribute(LIST).unwrap_or_default().into(),
                slot: slot_at(&middles, if across { at.0 } else { at.1 }),
            };
            if self.allows(&aim, &el) {
                return Some(aim);
            }
            list = el
                .parent_element()
                .and_then(|p| p.closest(&format!("[{LIST}]")).ok().flatten());
        }
        None
    }

    fn pointer_up(&self, e: &PointerEvent) {
        let drag = self.tree.live.borrow_mut().drag.take();
        let Some(drag) = drag.filter(|d| d.id == e.pointer_id()) else {
            return;
        };
        let _ = drag.row.remove_attribute("style");
        self.tree.live.borrow_mut().dragged = drag.active;
        let pointer = self
            .tree
            .live
            .borrow()
            .lift
            .as_ref()
            .is_some_and(|l| l.pointer);
        if drag.active && pointer {
            self.drop_lift();
        }
    }

    /// The places the row may go, in reading order, and the index of
    /// `aim` among them.
    fn steps(&self, aim: &Aim) -> (Vec<Aim>, Option<usize>) {
        let Some(root) = self.tree.root.cast::<Element>() else {
            return (Vec::new(), None);
        };
        let lists = lists_of(&root);
        let shape: Vec<TreeList> = lists.iter().map(|(_, _, l)| *l).collect();
        let mut out: Vec<Aim> = Vec::new();
        for (list, slot) in tree_slots(&shape) {
            let (el, id, _) = &lists[list];
            let step = Aim {
                list: id.clone(),
                slot,
            };
            // The gaps either side of the row are the one place it stays.
            if step.list == self.spot.list && step.slot == self.spot.index + 1 {
                continue;
            }
            if self.allows(&step, el) {
                out.push(step);
            }
        }
        let norm = if aim.stays(&self.spot) {
            Aim {
                list: self.spot.list.clone(),
                slot: self.spot.index,
            }
        } else {
            aim.clone()
        };
        let at = out.iter().position(|a| *a == norm);
        (out, at)
    }

    fn step(&self, aim: &Aim, key: &str) -> Option<Aim> {
        let (steps, at) = self.steps(aim);
        let at = at?;
        if self.tree.side_by_side {
            return lane_step(&steps, at, key);
        }
        match key {
            "ArrowUp" | "ArrowLeft" => at.checked_sub(1).map(|i| steps[i].clone()),
            "ArrowDown" | "ArrowRight" => steps.get(at + 1).cloned(),
            "Home" => steps.iter().find(|a| a.list == aim.list).cloned(),
            "End" => steps.iter().rev().find(|a| a.list == aim.list).cloned(),
            _ => None,
        }
    }

    fn key_down(&self, e: KeyboardEvent) {
        let key = e.key();
        let lift = self
            .tree
            .live
            .borrow()
            .lift
            .as_ref()
            .map(|l| (l.aim.clone(), l.pointer));
        let here = Aim {
            list: self.spot.list.clone(),
            slot: self.spot.index,
        };
        match lift {
            Some((_, true)) => {
                if key == "Escape" {
                    e.prevent_default();
                    self.cancel();
                }
            }
            Some((aim, false)) => {
                if key == " " || key == "Enter" {
                    e.prevent_default();
                    self.drop_lift();
                } else if key == "Escape" {
                    e.prevent_default();
                    self.cancel();
                } else if let Some(next) = self.step(&aim, &key) {
                    e.prevent_default();
                    self.set_aim(next, false);
                }
            }
            None if self.still() => {}
            None if key == " " || key == "Enter" => {
                e.prevent_default();
                self.set_aim(here, false);
            }
            None if e.alt_key()
                && matches!(
                    key.as_str(),
                    "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight"
                ) =>
            {
                e.prevent_default();
                if let Some(next) = self.step(&here, &key) {
                    self.set_aim(next, false);
                    self.drop_lift();
                }
            }
            None => {}
        }
    }

    fn blur(&self) {
        let lift = self
            .tree
            .live
            .borrow()
            .lift
            .as_ref()
            .is_some_and(|l| !l.pointer && l.from == self.spot);
        if lift {
            self.cancel();
        }
    }
}

/// The fixed rows of a list's element, by index.
fn fixed_rows(list: &Element) -> Vec<bool> {
    let rows = list.children();
    (0..rows.length())
        .filter_map(|i| rows.item(i))
        .map(|r| r.has_attribute(FIXED))
        .collect()
}

/// Where a row from `from` lands at `aim` among the rows of that list that
/// move (`stays` its fixed rows), from 1, and how many move there with it:
/// `same` when the list is the row's own.
fn tree_place(stays: &[bool], same: bool, from: &ReorderSpot, aim: &Aim) -> (usize, usize) {
    let to = aim.landing(from).index;
    if same {
        place(stays, from.index, to)
    } else {
        let (position, count) = place(stays, usize::MAX, to);
        (position, count + 1)
    }
}

/// `TreeGrip::step` for lists side by side: Up and Down, Home and
/// End within the list aimed at, Left and Right to the list beside it
/// that takes the row, at the same position or that list's end.
fn lane_step(steps: &[Aim], at: usize, key: &str) -> Option<Aim> {
    let aim = &steps[at];
    let within: Vec<&Aim> = steps.iter().filter(|a| a.list == aim.list).collect();
    let here = within.iter().position(|a| *a == aim)?;
    let across = |forward: bool| {
        let mut lists: Vec<&AttrValue> = Vec::new();
        for step in steps {
            if lists.last() != Some(&&step.list) && !lists.contains(&&step.list) {
                lists.push(&step.list);
            }
        }
        let index = lists.iter().position(|l| **l == aim.list)?;
        let next = if forward {
            lists.get(index + 1)?
        } else {
            lists.get(index.checked_sub(1)?)?
        };
        let places: Vec<&Aim> = steps.iter().filter(|a| a.list == **next).collect();
        places
            .get(here.min(places.len().saturating_sub(1)))
            .map(|a| (*a).clone())
    };
    match key {
        "ArrowUp" => here.checked_sub(1).map(|i| within[i].clone()),
        "ArrowDown" => within.get(here + 1).map(|a| (*a).clone()),
        "Home" => within.first().map(|a| (*a).clone()),
        "End" => within.last().map(|a| (*a).clone()),
        "ArrowLeft" => across(false),
        "ArrowRight" => across(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tree_rerenders_its_handles_when_tips_change() {
        let tree = |tips| Tree {
            live: Rc::default(),
            version: 0,
            bump: Callback::noop(),
            root: NodeRef::default(),
            on_move: Callback::noop(),
            accepts: None,
            say: Rc::new(|_| String::new()),
            say_in: None,
            hint: AttrValue::default(),
            disabled: false,
            titles: false,
            tips,
            side_by_side: false,
            body_drag: false,
        };
        let live = tree(false);
        let same = Tree {
            tips: true,
            ..live.clone()
        };
        assert!(live != same);
    }

    #[test]
    fn lanes_step_within_a_lane_and_across_at_the_same_place() {
        let aim = |list: &'static str, slot| Aim {
            list: list.into(),
            slot,
        };
        // A row lifted from a[1] of a: [x, row, y]; b: [p]; c: [].
        let steps = [
            aim("a", 0),
            aim("a", 1),
            aim("a", 3),
            aim("b", 0),
            aim("b", 1),
            aim("c", 0),
        ];
        assert_eq!(lane_step(&steps, 1, "ArrowDown"), Some(aim("a", 3)));
        assert_eq!(lane_step(&steps, 2, "ArrowDown"), None);
        assert_eq!(lane_step(&steps, 1, "ArrowRight"), Some(aim("b", 1)));
        assert_eq!(lane_step(&steps, 2, "ArrowRight"), Some(aim("b", 1)));
        assert_eq!(lane_step(&steps, 3, "ArrowRight"), Some(aim("c", 0)));
        assert_eq!(lane_step(&steps, 5, "ArrowRight"), None);
        assert_eq!(lane_step(&steps, 4, "ArrowLeft"), Some(aim("a", 1)));
        assert_eq!(lane_step(&steps, 0, "ArrowLeft"), None);
        assert_eq!(lane_step(&steps, 0, "End"), Some(aim("a", 3)));
    }

    #[test]
    fn the_slots_step_into_a_nested_list_and_out() {
        // A: [x, C, y], C holding B: [p, q].
        let lists = [
            TreeList {
                parent: None,
                len: 3,
            },
            TreeList {
                parent: Some((0, 1)),
                len: 2,
            },
        ];
        assert_eq!(
            tree_slots(&lists),
            [(0, 0), (0, 1), (1, 0), (1, 1), (1, 2), (0, 2), (0, 3)]
        );
    }

    #[test]
    fn an_empty_nested_list_has_one_slot() {
        let lists = [
            TreeList {
                parent: None,
                len: 1,
            },
            TreeList {
                parent: Some((0, 0)),
                len: 0,
            },
        ];
        assert_eq!(tree_slots(&lists), [(0, 0), (1, 0), (0, 1)]);
    }

    #[test]
    fn positions_count_only_the_rows_that_move() {
        let from = ReorderSpot {
            list: "a".into(),
            index: 2,
        };
        let aim = |list: &'static str, slot| Aim {
            list: list.into(),
            slot,
        };
        // A heading first: the row lifted from 2 is 2 of 3 in its list.
        let stays = [true, false, false, false];
        assert_eq!(tree_place(&stays, true, &from, &aim("a", 2)), (2, 3));
        assert_eq!(tree_place(&stays, true, &from, &aim("a", 0)), (1, 3));
        assert_eq!(tree_place(&stays, true, &from, &aim("a", 4)), (3, 3));
        // Into another list with a heading: one more row moves there.
        let stays = [true, false];
        assert_eq!(tree_place(&stays, false, &from, &aim("b", 2)), (2, 2));
        assert_eq!(tree_place(&stays, false, &from, &aim("b", 1)), (1, 2));
    }

    #[test]
    fn a_landing_counts_without_the_row_in_its_own_list() {
        let from = ReorderSpot {
            list: "a".into(),
            index: 1,
        };
        let aim = |list: &'static str, slot| Aim {
            list: list.into(),
            slot,
        };
        assert!(aim("a", 1).stays(&from));
        assert!(aim("a", 2).stays(&from));
        assert_eq!(aim("a", 3).landing(&from).index, 2);
        assert_eq!(aim("a", 0).landing(&from).index, 0);
        assert_eq!(aim("b", 3).landing(&from).index, 3);
    }
}
