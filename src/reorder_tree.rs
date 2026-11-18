//! ReorderTree and ReorderList: rows moved within and between nested lists
//! (`style/_reorder.scss`).

use std::cell::RefCell;
use std::rc::Rc;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use crate::phrasebookcatalog01::L10N;
use crate::reorder::{ReorderItem, ReorderText, phrase, scrolled, scroller_of};
use crate::tooltip::next_id;
use crate::{DragHandle, class, slot_at};

const SLOP: f64 = 4.0;
const LIST: &str = "data-reorder-list";

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
    hint: AttrValue,
    disabled: bool,
    titles: bool,
}

impl PartialEq for Tree {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.live, &other.live)
            && Rc::ptr_eq(&self.say, &other.say)
            && self.version == other.version
            && self.on_move == other.on_move
            && self.accepts == other.accepts
            && (self.disabled, self.titles) == (other.disabled, other.titles)
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
        hint: (*hint).clone(),
        disabled: props.disabled,
        titles: props.titles,
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
    /// The rows, each with a [`ReorderTreeHandle`] in it. An item's
    /// `current` and handlers are the row's; `fixed` and `lifted_only` are
    /// [`Reorder`](crate::Reorder)'s alone.
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
    let (lifted, aim) = {
        let live = tree.live.borrow();
        let lift = live.lift.as_ref();
        let lifted = lift
            .filter(|l| l.from.list == props.id)
            .map(|l| l.from.index);
        let aim = lift
            .filter(|l| l.aim.list == props.id && !l.aim.stays(&l.from))
            .map(|l| l.aim.slot);
        (lifted, aim)
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
            html! {
                <ContextProvider<TreeGrip> key={item.id.to_string()} context={grip}>
                    <div class={classes} data-testid={item.testid.clone()} data-reorder-key={item.id.clone()}
                        tabindex={props.focusable.then_some("-1")} aria-current={item.current.then_some("true")}
                        {onclick} oncontextmenu={item.oncontextmenu.clone()}
                        onmouseenter={item.onmouseenter.clone()} onmouseleave={item.onmouseleave.clone()}>
                        { item.children.clone() }
                    </div>
                </ContextProvider<TreeGrip>>
            }
        })
        .collect();
    let inside = (len == 0 && aim == Some(0)).then_some(class::DROP_INSIDE);
    html! {
        <div class={classes!(props.class.clone(), inside)} data-testid={props.testid.clone()}
            data-reorder-list={props.id.clone()} data-reorder-across={props.across.then_some("")}>
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
    let Some(grip) = use_context::<TreeGrip>() else {
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
        <DragHandle label={label.clone()} title={tree.titles.then_some(label)}
            lifted={grip.lifted} disabled={tree.disabled}
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
    fn announce(&self, text: ReorderText) {
        self.tree.live.borrow_mut().status = (self.tree.say)(text);
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

    fn place_text(&self, aim: &Aim, lists: &[(Element, AttrValue, TreeList)]) -> (usize, usize) {
        let len = lists
            .iter()
            .find(|(_, id, _)| *id == aim.list)
            .map_or(0, |(_, _, l)| l.len);
        let same = aim.list == self.spot.list;
        let count = if same { len } else { len + 1 };
        (aim.landing(&self.spot).index + 1, count)
    }

    fn set_aim(&self, aim: Aim, pointer: bool) {
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
        self.announce(if first {
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
        });
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
        self.announce(ReorderText::Dropped {
            name: lift.name,
            position,
            count,
        });
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
            let count = lists
                .iter()
                .find(|(_, id, _)| *id == self.spot.list)
                .map_or(0, |(_, _, l)| l.len);
            self.announce(ReorderText::Cancelled {
                name: self.name.clone(),
                position: self.spot.index + 1,
                count,
            });
        }
        self.tree.bump.emit(());
    }

    fn pointer_down(&self, e: PointerEvent) {
        if e.button() != 0 || self.tree.disabled || self.tree.live.borrow().lift.is_some() {
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
                    if across {
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
            None if self.tree.disabled => {}
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

#[cfg(test)]
mod tests {
    use super::*;

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
