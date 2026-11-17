//! Menu, its items, RowActions, SplitButton and ContextMenu
//! (`style/_menu.scss`).

use std::rc::Rc;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::roving::{Typeahead, menu_step, printable, typeahead};
use crate::timer::Timer;
use crate::{
    Button, ButtonSize, ButtonVariant, Dismiss, Glyph, Icon, PopoverAlign, Side, class,
    use_overlays, use_popover_within,
};

/// What the arrow keys, Home, End and type-ahead move between.
const ITEMS: &str = ":is([role=menuitem], [role=menuitemcheckbox], [role=menuitemradio]):not(:disabled, [aria-disabled=true])";

/// What closes the menu when picked: an action or a radio, not a check.
const PICKS: &str =
    ":is([role=menuitem], [role=menuitemradio]):not(:disabled, [aria-disabled=true])";

/// What Tab moves between in a menu that holds fields.
const TABBABLE: &str = "input:not(:disabled), textarea:not(:disabled), select:not(:disabled), button:not(:disabled), a[href], [tabindex]:not([tabindex='-1'])";

/// How long a touch is held to open a context menu, in milliseconds.
const LONG_PRESS_MS: u32 = 500;

/// How far a touch may move and still be a long press, in pixels.
const LONG_PRESS_SLOP: i32 = 10;

/// A menu's least width: its token, and the width it opened at, so it
/// never narrows while open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuWidth {
    /// `--menu-width`.
    #[default]
    Regular,
    /// `--menu-width-wide`: a list of names.
    Wide,
    /// `--menu-width-wider`: rows with a second line.
    Wider,
    /// `--menu-width-panel`, fixed: a form-like panel.
    Panel,
}

/// A menu's classes: `menu`, its width's modifier, `tight` for rows
/// without a gap, and `placed` once it has been placed (hidden until then).
pub fn menu(width: MenuWidth, tight: bool, placed: bool) -> Classes {
    let width = match width {
        MenuWidth::Regular => None,
        MenuWidth::Wide => Some(class::WIDE),
        MenuWidth::Wider => Some(class::WIDER),
        MenuWidth::Panel => Some(class::PANEL),
    };
    classes!(
        class::MENU,
        width,
        tight.then_some(class::TIGHT),
        placed.then_some(class::PLACED)
    )
}

fn items_of(list: &NodeRef) -> Vec<HtmlElement> {
    let Some(list) = list.cast::<Element>() else {
        return Vec::new();
    };
    let Ok(found) = list.query_selector_all(ITEMS) else {
        return Vec::new();
    };
    (0..found.length())
        .filter_map(|i| found.item(i)?.dyn_into::<HtmlElement>().ok())
        .collect()
}

fn focused(items: &[HtmlElement]) -> Option<usize> {
    let active = web_sys::window()?.document()?.active_element()?;
    items
        .iter()
        .position(|item| AsRef::<Element>::as_ref(item) == &active)
}

/// The label an item is found by in type-ahead.
fn label_of(item: &HtmlElement) -> String {
    item.query_selector(&format!(".{}", class::MENU_ITEM_LABEL))
        .ok()
        .flatten()
        .and_then(|label| label.text_content())
        .or_else(|| item.text_content())
        .unwrap_or_default()
}

/// Focuses `element` where the page stands.
fn focus_quietly(element: &HtmlElement) {
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    _ = element.focus_with_options(&options);
}

fn is_field(e: &Element) -> bool {
    e.matches("input, textarea, select").unwrap_or(false)
}

/// Whether Tab (Shift+Tab with `back`) has another of the list's own
/// controls to go to: only in a list that holds fields.
fn tabs_within(list: &NodeRef, back: bool) -> bool {
    let Some(list) = list.cast::<Element>() else {
        return false;
    };
    if !list
        .query_selector("input, textarea, select")
        .is_ok_and(|f| f.is_some())
    {
        return false;
    }
    let Ok(found) = list.query_selector_all(TABBABLE) else {
        return false;
    };
    let shown: Vec<Element> = (0..found.length())
        .filter_map(|i| found.item(i)?.dyn_into::<Element>().ok())
        .filter(|e| {
            let r = e.get_bounding_client_rect();
            r.width() > 0.0 || r.height() > 0.0
        })
        .collect();
    let active = web_sys::window().and_then(|w| w.document()?.active_element());
    let Some(at) = active.and_then(|a| shown.iter().position(|e| *e == a)) else {
        return false;
    };
    if back { at > 0 } else { at + 1 < shown.len() }
}

/// Whether focus is in `list`, or nowhere.
fn focus_in(list: &NodeRef) -> bool {
    let active = web_sys::window().and_then(|w| w.document()?.active_element());
    crate::popover::focus_lost()
        || list
            .get()
            .is_some_and(|l| active.is_some_and(|a| l.contains(Some(a.as_ref()))))
}

/// The keyboard of an open menu's list: arrows, Home, End, type-ahead;
/// Tab closes it and moves on from where focus goes `back` to, unless it
/// moves between the list's own fields and buttons. A focused field keeps
/// its keys. Escape is the popover layer's.
#[hook]
fn use_menu_keys(
    list: NodeRef,
    back: Callback<()>,
    close: Callback<()>,
) -> Callback<KeyboardEvent> {
    let typed = use_mut_ref(Typeahead::default);
    Callback::from(move |e: KeyboardEvent| {
        if e.key() == "Tab" {
            if tabs_within(&list, e.shift_key()) {
                return;
            }
            // The browser's Tab then moves on from the opener.
            back.emit(());
            close.emit(());
            return;
        }
        if e.target_dyn_into::<Element>().is_some_and(|t| is_field(&t)) {
            return;
        }
        let items = items_of(&list);
        let at = focused(&items);
        let next = match menu_step(&e.key(), at, items.len()) {
            Some(next) => Some(next),
            None => {
                let Some(c) = printable(&e) else { return };
                let mut typed = typed.borrow_mut();
                // Space activates the item unless a search is under way.
                let Some(query) = typed.push(c, e.time_stamp()) else {
                    return;
                };
                let labels: Vec<_> = items.iter().map(label_of).collect();
                typeahead(&labels, at, query)
            }
        };
        e.prevent_default();
        if let Some(item) = next.and_then(|i| items.get(i)) {
            _ = item.focus();
        }
    })
}

/// Focuses the list's checked radio once the menu is placed, else its
/// first item, else (a menu holding only fields) its first control.
#[hook]
fn use_focus_first(list: NodeRef, placed: bool) {
    use_effect_with(placed, move |&placed| {
        if !placed {
            return;
        }
        let items = items_of(&list);
        let checked = items.iter().find(|i| {
            i.get_attribute("role").as_deref() == Some("menuitemradio")
                && i.get_attribute("aria-checked").as_deref() == Some("true")
        });
        let control = || {
            list.cast::<Element>()?
                .query_selector(TABBABLE)
                .ok()??
                .dyn_into::<HtmlElement>()
                .ok()
        };
        if let Some(item) = checked
            .cloned()
            .or_else(|| items.first().cloned())
            .or_else(control)
        {
            _ = item.focus();
        }
    });
}

/// Holds the width a menu opened at while it stays open.
#[hook]
pub(crate) fn use_held_width(list: NodeRef, placed: bool) -> Option<i32> {
    let held = use_state_eq(|| None::<i32>);
    {
        let held = held.clone();
        use_effect_with(placed, move |&placed| {
            held.set(
                placed
                    .then(|| list.cast::<HtmlElement>().map(|l| l.offset_width()))
                    .flatten(),
            );
        });
    }
    *held
}

#[derive(Properties, PartialEq)]
struct ListProps {
    anchor: NodeRef,
    open: bool,
    on_dismiss: Callback<Dismiss>,
    /// Gives focus back to the opener (or what had it), at once.
    back: Callback<()>,
    side: Side,
    align: PopoverAlign,
    within: Option<AttrValue>,
    width: MenuWidth,
    tight: bool,
    sheet: bool,
    label: Option<AttrValue>,
    class: Classes,
    testid: Option<AttrValue>,
    /// Rendered, hidden, while closed.
    #[prop_or_default]
    keep_mounted: bool,
    children: Html,
}

/// The open list in the overlay layer: placed beside `anchor`, closed by a
/// pick, Tab, Escape or a press outside. A pick gives focus `back` before
/// the menu goes, so a dialog it opens returns focus there when it closes.
#[function_component]
fn MenuList(props: &ListProps) -> Html {
    let overlays = use_overlays();
    let list = use_node_ref();
    let placed = use_popover_within(
        props.open,
        props.anchor.clone(),
        list.clone(),
        props.side,
        props.align,
        props.within.clone(),
        props.on_dismiss.clone(),
    );
    use_focus_first(list.clone(), placed.is_some());
    let held = use_held_width(list.clone(), placed.is_some());
    let onkeydown = use_menu_keys(
        list.clone(),
        props.back.clone(),
        props.on_dismiss.reform(|()| Dismiss::Close),
    );

    if !props.open && !props.keep_mounted {
        return Html::default();
    }

    let onclick = {
        let (on_dismiss, back, list) = (props.on_dismiss.clone(), props.back.clone(), list.clone());
        Callback::from(move |e: MouseEvent| {
            let picked = e
                .target_dyn_into::<Element>()
                .and_then(|t| t.closest(PICKS).ok().flatten());
            if picked.is_some() {
                if focus_in(&list) {
                    back.emit(());
                }
                on_dismiss.emit(Dismiss::Close);
            }
        })
    };
    let style = placed.map(|p| {
        let held = held.map_or(String::new(), |w| format!(" --menu-open-width: {w}px;"));
        format!(
            "--popover-left: {}px; --popover-top: {}px; --popover-max-height: {}px;{held}",
            p.left, p.top, p.max_height
        )
    });

    overlays.portal(html! {
        <>
            if props.sheet && props.open {
                <div class={class::MENU_SCRIM} aria-hidden="true"></div>
            }
            <div ref={list}
                class={classes!(menu(props.width, props.tight, placed.is_some()),
                    props.sheet.then_some(class::SHEET), props.class.clone())}
                hidden={!props.open}
                role="menu" aria-label={props.label.clone()} {style}
                data-testid={props.testid.clone()} {onclick} {onkeydown}>
                { props.children.clone() }
            </div>
        </>
    })
}

/// Its own open state, or its owner's.
#[hook]
fn use_open(open: Option<bool>, on_toggle: Callback<bool>) -> (bool, Callback<bool>) {
    let own = use_state(|| false);
    let is_open = open.unwrap_or(*own);
    let set = {
        let controlled = open.is_some();
        Callback::from(move |next: bool| {
            if !controlled {
                own.set(next);
            }
            on_toggle.emit(next);
        })
    };
    (is_open, set)
}

#[derive(Properties, PartialEq)]
pub struct MenuProps {
    /// The opener's visible text; without it the opener is its icon alone.
    #[prop_or_default]
    pub text: Html,
    /// The opener's icon, which the app supplies; without one, a text
    /// opener ends in a chevron.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// The accessible name of the opener and the menu (`aria-label`).
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub variant: ButtonVariant,
    /// The opener's size: the full control height unless set.
    #[prop_or_default]
    pub size: ButtonSize,
    /// The opener's classes, beside the button's.
    #[prop_or_default]
    pub opener_class: Classes,
    /// The opener's tooltip (`data-tip`, for [`DataTips`](crate::DataTips)).
    #[prop_or_default]
    pub tip: Option<AttrValue>,
    /// The opener's native tooltip (`title`).
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// A click on the opener does this instead of opening the menu, for an
    /// opener that is a tool of its own; a right-click (a long press on a
    /// touch screen) opens the menu, as it does for a [`MenuProps::pressed`]
    /// toggle.
    #[prop_or_default]
    pub on_click: Option<Callback<MouseEvent>>,
    /// The opener is a toggle and whether it is on (`aria-pressed`).
    #[prop_or_default]
    pub pressed: Option<bool>,
    #[prop_or_default]
    pub width: MenuWidth,
    /// No gap between rows.
    #[prop_or_default]
    pub tight: bool,
    /// Below the phone breakpoint, a sheet from the bottom over a scrim
    /// (`SHEET`), as a popover is; off by default.
    #[prop_or_default]
    pub sheet: bool,
    #[prop_or_default]
    pub side: Side,
    /// `End` by default, so it opens toward the page from an opener at a
    /// row's end.
    #[prop_or(PopoverAlign::End)]
    pub align: PopoverAlign,
    /// A selector: keep it inside the anchor's closest ancestor matching
    /// it (a window of the app's), not the viewport alone.
    #[prop_or_default]
    pub within: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    /// Open, for a menu its owner also opens another way (a right-click on
    /// its row); its own state when `None`.
    #[prop_or_default]
    pub open: Option<bool>,
    /// Told whenever it would open or close.
    #[prop_or_default]
    pub on_toggle: Callback<bool>,
    /// The menu's classes.
    #[prop_or_default]
    pub class: Classes,
    /// The menu's test id; its opener's is it suffixed `-toggle`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The opener's test id, in place of [`MenuProps::testid`] suffixed
    /// `-toggle`.
    #[prop_or_default]
    pub opener_testid: Option<AttrValue>,
    /// Keep the items rendered, hidden, while the menu is closed: for an
    /// item that owns what it opens (a dialog), which closing the menu must
    /// not drop. Off by default: a closed menu renders no list.
    #[prop_or_default]
    pub keep_mounted: bool,
    /// The items: [`MenuItem`], [`MenuCheck`], [`MenuRadio`],
    /// [`MenuSeparator`], [`MenuHeading`], [`MenuNote`].
    #[prop_or_default]
    pub children: Html,
}

/// A button opening a list of actions beside it in the overlay layer. A
/// pick, Tab, Escape or a press outside closes it, and all but the press
/// give focus back to the opener (Tab then moves on from it); the arrows,
/// Home, End and type-ahead move between its items.
#[function_component]
pub fn Menu(props: &MenuProps) -> Html {
    let opener = use_node_ref();
    let (is_open, set_open) = use_open(props.open, props.on_toggle.clone());
    let onclick = {
        let (set_open, on_click) = (set_open.clone(), props.on_click.clone());
        Callback::from(move |e: MouseEvent| match &on_click {
            Some(on_click) => {
                if is_open {
                    set_open.emit(false);
                }
                on_click.emit(e);
            }
            None => set_open.emit(!is_open),
        })
    };
    let oncontextmenu = {
        let (set_open, disabled) = (set_open.clone(), props.disabled);
        Callback::from(move |e: MouseEvent| {
            // A second right-click reaches the browser's own menu.
            if !disabled && !is_open {
                e.prevent_default();
                set_open.emit(true);
            }
        })
    };
    let on_dismiss = set_open.reform(|_: Dismiss| false);
    let back = {
        let opener = opener.clone();
        Callback::from(move |()| {
            if let Some(opener) = opener.cast::<HtmlElement>() {
                focus_quietly(&opener);
            }
        })
    };
    // Focus left on the opener (a panel of text alone): its keys close it.
    let onkeydown = {
        let set_open = set_open.clone();
        Callback::from(move |e: KeyboardEvent| {
            if !is_open {
                return;
            }
            match e.key().as_str() {
                "Escape" => {
                    e.stop_propagation();
                    set_open.emit(false);
                }
                "Tab" => set_open.emit(false),
                _ => {}
            }
        })
    };
    let has_text = !crate::button::is_empty(&props.text);
    let (icon, trailing) = match &props.icon {
        Some(icon) => (Some(icon.clone()), false),
        None if has_text => (Some(html! { <Icon glyph={Glyph::ChevronDown} /> }), true),
        None => (Some(html! { <Icon glyph={Glyph::Ellipsis} /> }), false),
    };

    let button = html! {
        <Button node_ref={opener.clone()} variant={props.variant} size={props.size}
            class={props.opener_class.clone()} {icon} {trailing}
            label={props.label.clone()} tip={props.tip.clone()} title={props.title.clone()}
            disabled={props.disabled}
            pressed={props.pressed}
            expanded={Some(is_open)} haspopup={AttrValue::Static("menu")}
            testid={props.opener_testid.clone().or_else(|| {
                props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-toggle")))
            })}
            {onclick} {onkeydown}>
            { props.text.clone() }
        </Button>
    };

    html! {
        <>
            // Kept while `on_click` comes and goes, so the opener keeps focus.
            if props.on_click.is_some() || props.pressed.is_some() {
                <span class={class::CONTEXT_TARGET} {oncontextmenu}>{ button }</span>
            } else {
                { button }
            }
            <MenuList anchor={opener} open={is_open} {on_dismiss} {back}
                side={props.side} align={props.align} within={props.within.clone()}
                width={props.width} tight={props.tight} sheet={props.sheet}
                label={props.label.clone()} class={props.class.clone()}
                testid={props.testid.clone()} keep_mounted={props.keep_mounted}>
                { props.children.clone() }
            </MenuList>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct MenuItemProps {
    /// Its icon, which the app supplies, in a fixed box.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// A second line under the label (an option's detail).
    #[prop_or_default]
    pub detail: Option<AttrValue>,
    /// Markup in the second line, after [`MenuItemProps::detail`]: a mark
    /// or chips the app renders.
    #[prop_or_default]
    pub detail_html: Option<Html>,
    /// A note at the row's end, such as its shortcut.
    #[prop_or_default]
    pub hint: Option<AttrValue>,
    /// A link row (`<a>`): picking it follows the link and closes the menu.
    #[prop_or_default]
    pub href: Option<AttrValue>,
    /// A link row opening in a new tab (`target="_blank"`,
    /// `rel="noopener noreferrer"`): another site's page.
    #[prop_or_default]
    pub new_tab: bool,
    /// A link row that downloads what it links to (`download`).
    #[prop_or_default]
    pub download: bool,
    /// The row's native tooltip (`title`).
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// A button row that opens a dialog: whether it is open now
    /// (`aria-haspopup="dialog"`, `aria-expanded`).
    #[prop_or_default]
    pub dialog: Option<bool>,
    /// The page shown (`aria-current="page"`), marked, in a menu of links.
    #[prop_or_default]
    pub current: bool,
    /// A styled tip (`data-tip`, for [`DataTips`](crate::DataTips)) naming
    /// what it acts on.
    #[prop_or_default]
    pub tip: Option<AttrValue>,
    /// In the danger colour (Delete).
    #[prop_or_default]
    pub danger: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The label.
    #[prop_or_default]
    pub children: Html,
}

/// A menu item's classes: `menu-item`, `danger` in the danger colour.
pub fn menu_item(danger: bool) -> Classes {
    classes!(class::MENU_ITEM, danger.then_some(class::DANGER))
}

/// The row every item is: an optional mark (a check's or radio's, whose
/// box is always there), an icon, the label and its detail, a hint.
struct Row<'a> {
    role: &'static str,
    checked: Option<bool>,
    props: &'a MenuItemProps,
    onclick: Callback<MouseEvent>,
}

impl Row<'_> {
    fn render(self) -> Html {
        let p = self.props;
        let mark = self.checked.map(|on| {
            html! {
                <span class={classes!(class::MENU_ITEM_MARK, (!on).then_some(class::OFF))} aria-hidden="true">
                    <Icon glyph={Glyph::Check} />
                </span>
            }
        });
        let icon = p.icon.clone().map(|icon| {
            html! { <span class={class::MENU_ITEM_ICON} aria-hidden="true">{ icon }</span> }
        });
        let detail = (p.detail.is_some() || p.detail_html.is_some()).then(|| {
            html! {
                <span class={class::MENU_ITEM_DETAIL}>
                    { for p.detail.clone() }
                    { for p.detail_html.clone() }
                </span>
            }
        });
        let hint = p.hint.clone().map(|hint| {
            html! { <span class={class::MENU_ITEM_HINT}>{ hint }</span> }
        });
        let onclick = {
            let (onclick, disabled) = (self.onclick, p.disabled);
            Callback::from(move |e: MouseEvent| {
                if !disabled {
                    onclick.emit(e);
                }
            })
        };
        let content = html! {
            <>
                { for mark }
                { for icon }
                <span class={class::MENU_ITEM_TEXT}>
                    <span class={class::MENU_ITEM_LABEL}>{ p.children.clone() }</span>
                    { for detail }
                </span>
                { for hint }
            </>
        };
        let class = classes!(menu_item(p.danger), p.class.clone());
        let disabled = p.disabled.then_some("true");
        let current = p.current.then_some("page");
        match &p.href {
            Some(href) => html! {
                <a role={self.role} tabindex="-1" {class}
                    href={(!p.disabled).then(|| href.clone())}
                    target={p.new_tab.then_some("_blank")}
                    rel={p.new_tab.then_some("noopener noreferrer")}
                    download={p.download.then_some("")} title={p.title.clone()}
                    aria-disabled={disabled} aria-current={current} data-tip={p.tip.clone()}
                    data-testid={p.testid.clone()} {onclick}>
                    { content }
                </a>
            },
            None => html! {
                <button type="button" role={self.role} tabindex="-1" {class}
                    aria-checked={self.checked.map(|on| on.to_string())}
                    aria-haspopup={p.dialog.map(|_| "dialog")}
                    aria-expanded={p.dialog.map(|open| open.to_string())}
                    title={p.title.clone()}
                    aria-disabled={disabled} aria-current={current} data-tip={p.tip.clone()}
                    data-testid={p.testid.clone()} {onclick}>
                    { content }
                </button>
            },
        }
    }
}

/// An action: picking it closes the menu.
#[function_component]
pub fn MenuItem(props: &MenuItemProps) -> Html {
    Row {
        role: "menuitem",
        checked: None,
        props,
        onclick: props.onclick.clone(),
    }
    .render()
}

#[derive(Properties, PartialEq)]
pub struct MenuCheckProps {
    pub checked: bool,
    /// Told the new state; the menu stays open.
    pub on_change: Callback<bool>,
    /// Its icon, in the box after its mark.
    #[prop_or_default]
    pub icon: Option<Html>,
    #[prop_or_default]
    pub detail: Option<AttrValue>,
    #[prop_or_default]
    pub hint: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

fn item_props(
    icon: &Option<Html>,
    detail: &Option<AttrValue>,
    hint: &Option<AttrValue>,
    disabled: bool,
    class: &Classes,
    testid: &Option<AttrValue>,
    children: &Html,
) -> MenuItemProps {
    MenuItemProps {
        icon: icon.clone(),
        detail: detail.clone(),
        detail_html: None,
        hint: hint.clone(),
        href: None,
        new_tab: false,
        download: false,
        title: None,
        dialog: None,
        current: false,
        tip: None,
        danger: false,
        disabled,
        onclick: Callback::noop(),
        class: class.clone(),
        testid: testid.clone(),
        children: children.clone(),
    }
}

/// An on or off setting (`menuitemcheckbox`): picking it toggles it and
/// leaves the menu open. Its check's box is there either way.
#[function_component]
pub fn MenuCheck(props: &MenuCheckProps) -> Html {
    let item = item_props(
        &props.icon,
        &props.detail,
        &props.hint,
        props.disabled,
        &props.class,
        &props.testid,
        &props.children,
    );
    let checked = props.checked;
    Row {
        role: "menuitemcheckbox",
        checked: Some(checked),
        props: &item,
        onclick: props.on_change.reform(move |_| !checked),
    }
    .render()
}

#[derive(Properties, PartialEq)]
pub struct MenuRadioProps {
    pub checked: bool,
    /// Picked; the menu closes.
    pub on_pick: Callback<()>,
    /// Its icon, in the box after its mark.
    #[prop_or_default]
    pub icon: Option<Html>,
    #[prop_or_default]
    pub detail: Option<AttrValue>,
    #[prop_or_default]
    pub hint: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// One of a set of choices (`menuitemradio`): picking it closes the menu.
/// Its check's box is there either way.
#[function_component]
pub fn MenuRadio(props: &MenuRadioProps) -> Html {
    let item = item_props(
        &props.icon,
        &props.detail,
        &props.hint,
        props.disabled,
        &props.class,
        &props.testid,
        &props.children,
    );
    Row {
        role: "menuitemradio",
        checked: Some(props.checked),
        props: &item,
        onclick: props.on_pick.reform(|_| ()),
    }
    .render()
}

#[derive(Properties, PartialEq)]
pub struct MenuPartProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A rule between groups of items.
#[function_component]
pub fn MenuSeparator(props: &MenuPartProps) -> Html {
    html! { <div role="separator" class={classes!(class::MENU_SEPARATOR, props.class.clone())}
    data-testid={props.testid.clone()}></div> }
}

/// A group's heading, on one line.
#[function_component]
pub fn MenuHeading(props: &MenuPartProps) -> Html {
    html! {
        <div role="presentation" class={classes!(class::MENU_HEADING, props.class.clone())}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </div>
    }
}

/// A line of text among the items: that there are none, or what they are.
#[function_component]
pub fn MenuNote(props: &MenuPartProps) -> Html {
    html! {
        <p role="presentation" class={classes!(class::MENU_NOTE, props.class.clone())}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </p>
    }
}

/// One of a row's actions, shown as its icon and named by its label.
#[derive(Clone, PartialEq)]
pub struct RowAction {
    pub icon: Html,
    /// Its accessible name and tooltip.
    pub label: AttrValue,
    pub onclick: Callback<MouseEvent>,
    /// Last in the row, muted until pointed at or focused.
    pub danger: bool,
    /// A toggle and whether it is on.
    pub pressed: Option<bool>,
    pub disabled: bool,
    pub testid: Option<AttrValue>,
}

impl RowAction {
    pub fn new(icon: Html, label: impl Into<AttrValue>, onclick: Callback<MouseEvent>) -> Self {
        Self {
            icon,
            label: label.into(),
            onclick,
            danger: false,
            pressed: None,
            disabled: false,
            testid: None,
        }
    }

    pub fn danger(self) -> Self {
        Self {
            danger: true,
            ..self
        }
    }

    pub fn pressed(self, on: bool) -> Self {
        Self {
            pressed: Some(on),
            ..self
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }

    pub fn testid(self, testid: impl Into<AttrValue>) -> Self {
        Self {
            testid: Some(testid.into()),
            ..self
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct RowActionsProps {
    #[prop_or_default]
    pub actions: Vec<RowAction>,
    /// The overflow menu's items, behind its ellipsis; no menu without.
    #[prop_or_default]
    pub more: Option<Html>,
    /// The overflow menu's opener's accessible name.
    #[prop_or_default]
    pub more_label: Option<AttrValue>,
    /// The menu holds nothing for this row: its opener stays, disabled, so
    /// every row ends the same way.
    #[prop_or_default]
    pub more_disabled: bool,
    /// The overflow menu's test id; its opener's is it suffixed `-toggle`.
    #[prop_or_default]
    pub more_testid: Option<AttrValue>,
    /// Name each action and the overflow opener with a styled tip
    /// (`data-tip`, for [`DataTips`](crate::DataTips)) instead of `title`.
    #[prop_or_default]
    pub tips: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// The overflow menu open, for a row that also opens it on a
    /// right-click; its own state when `None`.
    #[prop_or_default]
    pub open: Option<bool>,
    #[prop_or_default]
    pub on_toggle: Callback<bool>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// Shown after the actions, before the danger ones.
    #[prop_or_default]
    pub children: Html,
}

/// A row's actions: small icon buttons of one size, always shown, the
/// danger ones last, then its overflow menu.
#[function_component]
pub fn RowActions(props: &RowActionsProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let more_label = label(&phrases, &props.more_label, "plinth-row-actions-more");
    let tips = props.tips;
    let button = |action: &RowAction| {
        let (title, tip) = if tips {
            (None, Some(action.label.clone()))
        } else {
            (Some(action.label.clone()), None)
        };
        html! {
            <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                class={classes!(action.danger.then_some(class::ROW_ACTION_DANGER))}
                icon={action.icon.clone()} label={action.label.clone()} {title} {tip}
                pressed={action.pressed} disabled={props.disabled || action.disabled}
                testid={action.testid.clone()} onclick={action.onclick.clone()} />
        }
    };
    let plain = props.actions.iter().filter(|a| !a.danger).map(button);
    let danger = props.actions.iter().filter(|a| a.danger).map(button);

    html! {
        <span class={classes!(class::ROW_ACTIONS, props.class.clone())} data-testid={props.testid.clone()}>
            { for plain }
            { props.children.clone() }
            { for danger }
            if let Some(more) = &props.more {
                <Menu variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                    label={more_label.clone()} disabled={props.disabled || props.more_disabled}
                    tip={tips.then_some(more_label)} testid={props.more_testid.clone()}
                    open={props.open} on_toggle={props.on_toggle.clone()}>
                    { more.clone() }
                </Menu>
            }
        </span>
    }
}

#[derive(Properties, PartialEq)]
pub struct SplitButtonProps {
    /// The main action's label.
    pub text: Html,
    pub onclick: Callback<MouseEvent>,
    #[prop_or(ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    /// The main action's icon.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// The chevron's accessible name.
    #[prop_or_default]
    pub menu_label: Option<AttrValue>,
    #[prop_or_default]
    pub width: MenuWidth,
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The menu's items: the action's variants.
    #[prop_or_default]
    pub children: Html,
}

/// A main action joined to a menu of its variants: one control, the two
/// parts sharing a height and a border.
#[function_component]
pub fn SplitButton(props: &SplitButtonProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    html! {
        <span class={classes!(class::SPLIT_BUTTON, props.class.clone())}>
            <Button variant={props.variant} size={props.size} class={class::SPLIT_BUTTON_MAIN}
                icon={props.icon.clone()} busy={props.busy} disabled={props.disabled}
                testid={props.testid.clone()} onclick={props.onclick.clone()}>
                { props.text.clone() }
            </Button>
            <Menu variant={props.variant} size={props.size}
                opener_class={class::SPLIT_BUTTON_TOGGLE}
                icon={html! { <Icon glyph={Glyph::ChevronDown} /> }}
                label={label(&phrases, &props.menu_label, "plinth-split-button-menu")} width={props.width}
                disabled={props.disabled || props.busy}
                testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-menu")))}>
                { props.children.clone() }
            </Menu>
        </span>
    }
}

#[derive(Properties, PartialEq)]
pub struct ContextMenuProps {
    /// The menu's items.
    pub items: Html,
    #[prop_or_default]
    pub width: MenuWidth,
    #[prop_or_default]
    pub tight: bool,
    /// A selector: keep it inside the anchor's closest ancestor matching
    /// it (a window of the app's), not the viewport alone.
    #[prop_or_default]
    pub within: Option<AttrValue>,
    /// The menu's accessible name.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// The browser's own menu instead.
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub on_toggle: Callback<bool>,
    /// The menu's classes.
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// What a right-click or a long press opens it on; laid out as if the
    /// ContextMenu were not there (`display: contents`).
    #[prop_or_default]
    pub children: Html,
}

/// Where a context menu opens: a point in viewport coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Point(i32, i32);

/// A menu opened at the pointer by a right-click on its children, or a
/// long press on a touch screen; otherwise a [`Menu`]'s list. Closed by
/// anything but a press outside, it gives focus back to what had it.
#[function_component]
pub fn ContextMenu(props: &ContextMenuProps) -> Html {
    let overlays = use_overlays();
    let at = use_state_eq(|| None::<Point>);
    let point = use_node_ref();
    let press = use_mut_ref(|| None::<(Timer, Point)>);
    // What had focus when it opened, and whether its close gives it back.
    let before = use_mut_ref(|| None::<HtmlElement>);
    let restore = use_mut_ref(|| false);

    let show = {
        let (at, on_toggle, before) = (at.clone(), props.on_toggle.clone(), before.clone());
        Callback::from(move |p: Option<Point>| {
            if p.is_some() && at.is_none() {
                *before.borrow_mut() = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                    .and_then(|e| e.dyn_into::<HtmlElement>().ok());
            }
            on_toggle.emit(p.is_some());
            at.set(p);
        })
    };
    let oncontextmenu = {
        let (show, disabled) = (show.clone(), props.disabled);
        Callback::from(move |e: MouseEvent| {
            if !disabled {
                e.prevent_default();
                show.emit(Some(Point(e.client_x(), e.client_y())));
            }
        })
    };
    {
        let press = press.clone();
        use_effect_with((), move |()| move || drop(press.borrow_mut().take()));
    }
    let cancel = {
        let press = press.clone();
        Rc::new(move || {
            press.borrow_mut().take();
        })
    };
    let onpointerdown = {
        let (show, press, cancel, disabled) =
            (show.clone(), press.clone(), cancel.clone(), props.disabled);
        Callback::from(move |e: PointerEvent| {
            cancel();
            if disabled || e.pointer_type() != "touch" {
                return;
            }
            let p = Point(e.client_x(), e.client_y());
            let show = show.clone();
            let timer = Timer::start(LONG_PRESS_MS, move || show.emit(Some(p)));
            *press.borrow_mut() = timer.map(|t| (t, p));
        })
    };
    let onpointermove = {
        let (press, cancel) = (press.clone(), cancel.clone());
        Callback::from(move |e: PointerEvent| {
            let moved = press.borrow().as_ref().is_some_and(|(_, Point(x, y))| {
                (e.client_x() - *x).abs() > LONG_PRESS_SLOP
                    || (e.client_y() - *y).abs() > LONG_PRESS_SLOP
            });
            if moved {
                cancel();
            }
        })
    };
    let onpointerup = {
        let cancel = cancel.clone();
        Callback::from(move |_: PointerEvent| cancel())
    };
    let on_dismiss = {
        let (show, restore) = (show.clone(), restore.clone());
        Callback::from(move |why: Dismiss| {
            *restore.borrow_mut() = why.returns_focus();
            show.emit(None);
        })
    };
    let back = {
        let before = before.clone();
        Callback::from(move |()| {
            if let Some(before) = before.borrow().as_ref() {
                focus_quietly(before);
            }
        })
    };
    {
        let open = at.is_some();
        use_effect_with(open, move |&open| {
            let back = before.borrow_mut().take();
            if !open
                && std::mem::take(&mut *restore.borrow_mut())
                && crate::popover::focus_lost()
                && let Some(back) = back
            {
                _ = back.focus();
            } else if open {
                *before.borrow_mut() = back;
            }
        });
    }
    let point_style = at.map(|Point(x, y)| format!("--menu-x: {x}px; --menu-y: {y}px;"));
    let open = at.is_some();

    html! {
        <div class={class::CONTEXT_TARGET} {oncontextmenu} {onpointerdown}
            {onpointermove} onpointerup={onpointerup.clone()} onpointercancel={onpointerup}>
            { props.children.clone() }
            if open {
                { overlays.portal(html! {
                    <span ref={point.clone()} class={class::MENU_POINT} style={point_style} aria-hidden="true"></span>
                }) }
            }
            <MenuList anchor={point} {open} {on_dismiss} {back}
                side={Side::Below} align={PopoverAlign::Start} within={props.within.clone()}
                width={props.width} tight={props.tight} sheet=false
                label={props.label.clone()} class={props.class.clone()}
                testid={props.testid.clone()}>
                { props.items.clone() }
            </MenuList>
        </div>
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

    #[test]
    fn a_disabled_row_neither_moves_nor_closes() {
        assert!(ITEMS.contains("[aria-disabled=true]"));
        assert!(PICKS.contains("[aria-disabled=true]"));
    }

    #[test]
    fn each_width_and_tight_are_modifiers() {
        let of = |w| names(menu(w, false, false));
        assert_eq!(of(MenuWidth::Regular), [class::MENU]);
        assert_eq!(of(MenuWidth::Wide), [class::MENU, class::WIDE]);
        assert_eq!(of(MenuWidth::Wider), [class::MENU, class::WIDER]);
        assert_eq!(of(MenuWidth::Panel), [class::MENU, class::PANEL]);
        assert_eq!(
            names(menu(MenuWidth::Regular, true, true)),
            [class::MENU, class::TIGHT, class::PLACED]
        );
        assert_eq!(names(menu_item(false)), [class::MENU_ITEM]);
        assert_eq!(names(menu_item(true)), [class::MENU_ITEM, class::DANGER]);
    }

    #[test]
    fn a_closed_menu_is_its_opener_alone() {
        let html = render::<Menu>(yew::props!(MenuProps {
            text: html! { "Sort" },
            testid: Some(AttrValue::Static("sort")),
        }));
        assert!(html.contains("aria-haspopup=\"menu\""), "{html}");
        assert!(html.contains("aria-expanded=\"false\""), "{html}");
        assert!(html.contains("data-testid=\"sort-toggle\""), "{html}");
        assert!(!html.contains("role=\"menu\""), "{html}");
        // A text opener ends in its chevron, at the full control height.
        assert!(!html.contains(class::SMALL), "{html}");
        assert!(html.contains(class::BUTTON_LABEL), "{html}");
    }

    #[test]
    fn an_open_menu_renders_its_items_hidden_until_placed() {
        let html = render::<Menu>(yew::props!(MenuProps {
            open: Some(true),
            label: Some(AttrValue::Static("Row")),
            children: html! {
                <>
                    <MenuHeading>{ "Edit" }</MenuHeading>
                    <MenuItem hint="Ctrl+C" detail="To the clipboard">{ "Copy" }</MenuItem>
                    <MenuSeparator />
                    <MenuItem danger=true>{ "Delete" }</MenuItem>
                </>
            },
        }));
        assert!(html.contains("aria-expanded=\"true\""), "{html}");
        assert!(html.contains("role=\"menu\""), "{html}");
        assert!(!html.contains(class::PLACED), "{html}");
        assert!(html.contains("role=\"menuitem\""), "{html}");
        assert!(html.contains("tabindex=\"-1\""), "{html}");
        assert!(html.contains(class::MENU_ITEM_DETAIL), "{html}");
        assert!(html.contains(class::MENU_ITEM_HINT), "{html}");
        assert!(html.contains(class::MENU_SEPARATOR), "{html}");
        assert!(
            html.contains(&format!("{} {}", class::MENU_ITEM, class::DANGER)),
            "{html}"
        );
    }

    #[test]
    fn a_kept_menu_renders_its_items_hidden_while_closed() {
        let html = render::<Menu>(yew::props!(MenuProps {
            text: html! { "More" },
            keep_mounted: true,
            sheet: true,
            opener_testid: Some(AttrValue::Static("more")),
            children: html! { <MenuItem dialog={Some(false)}>{ "Rename…" }</MenuItem> },
        }));
        assert!(html.contains("aria-expanded=\"false\""), "{html}");
        assert!(html.contains("data-testid=\"more\""), "{html}");
        assert!(html.contains("role=\"menu\""), "{html}");
        assert!(html.contains(" hidden"), "{html}");
        assert!(!html.contains(class::MENU_SCRIM), "{html}");
        assert!(html.contains("aria-haspopup=\"dialog\""), "{html}");
        let html = render::<Menu>(yew::props!(MenuProps {
            open: Some(true),
            keep_mounted: true,
            children: html! { <MenuItem>{ "Rename…" }</MenuItem> },
        }));
        assert!(html.contains("role=\"menu\""), "{html}");
        assert!(!html.contains(" hidden"), "{html}");
    }

    #[test]
    fn a_download_row_downloads_and_takes_a_title() {
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            href: Some(AttrValue::Static("/export")),
            download: true,
            title: Some(AttrValue::Static("As a file")),
        }));
        assert!(html.contains("download=\"\""), "{html}");
        assert!(html.contains("title=\"As a file\""), "{html}");
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            href: Some(AttrValue::Static("/export")),
        }));
        assert!(!html.contains("download"), "{html}");
        assert!(!html.contains("title"), "{html}");
    }

    #[test]
    fn a_sheet_menu_has_its_scrim() {
        let open = |sheet| {
            render::<Menu>(yew::props!(MenuProps {
                open: Some(true),
                sheet,
                children: html! { <MenuItem>{ "Copy" }</MenuItem> },
            }))
        };
        let html = open(true);
        assert!(html.contains(class::MENU_SCRIM), "{html}");
        assert!(
            html.contains(&format!("{} {}", class::MENU, class::SHEET)),
            "{html}"
        );
        let html = open(false);
        assert!(!html.contains(class::MENU_SCRIM), "{html}");
        assert!(!html.contains(class::SHEET), "{html}");
    }

    #[test]
    fn a_link_item_is_an_anchor_without_its_href_when_disabled() {
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            href: Some(AttrValue::Static("/help")),
            children: html! { "Help" },
        }));
        assert!(html.starts_with("<a"), "{html}");
        assert!(html.contains("role=\"menuitem\""), "{html}");
        assert!(html.contains("href=\"/help\""), "{html}");
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            href: Some(AttrValue::Static("/help")),
            disabled: true,
        }));
        assert!(!html.contains("href"), "{html}");
        assert!(html.contains("aria-disabled=\"true\""), "{html}");
    }

    #[test]
    fn a_new_tab_link_row_opens_apart_and_its_detail_takes_markup() {
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            href: Some(AttrValue::Static("https://example.org/")),
            new_tab: true,
            detail: Some(AttrValue::Static("Lookup")),
            detail_html: Some(html! { <em>{ "chips" }</em> }),
        }));
        assert!(html.contains("target=\"_blank\""), "{html}");
        assert!(html.contains("rel=\"noopener noreferrer\""), "{html}");
        assert!(html.contains("Lookup<em>chips</em>"), "{html}");
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            href: Some(AttrValue::Static("/help")),
        }));
        assert!(!html.contains("target"), "{html}");
        assert!(!html.contains(class::MENU_ITEM_DETAIL), "{html}");
    }

    #[test]
    fn a_choice_takes_an_icon_and_a_note_a_testid() {
        let html = render::<MenuRadio>(yew::props!(MenuRadioProps {
            checked: false,
            on_pick: Callback::noop(),
            icon: Some(html! { <svg /> }),
        }));
        assert!(html.contains(class::MENU_ITEM_ICON), "{html}");
        let html = render::<MenuNote>(yew::props!(MenuPartProps {
            testid: Some(AttrValue::Static("none")),
        }));
        assert!(html.contains("data-testid=\"none\""), "{html}");
    }

    #[test]
    fn an_item_takes_a_tip() {
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            tip: Some(AttrValue::Static("Delete Pia")),
        }));
        assert!(html.contains("data-tip=\"Delete Pia\""), "{html}");
        let html = render::<MenuItem>(yew::props!(MenuItemProps {}));
        assert!(!html.contains("data-tip"), "{html}");
    }

    #[test]
    fn a_current_item_is_marked_as_the_page_shown() {
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            href: Some(AttrValue::Static("/users")),
            current: true,
        }));
        assert!(html.contains("aria-current=\"page\""), "{html}");
        let html = render::<MenuItem>(yew::props!(MenuItemProps {
            href: Some(AttrValue::Static("/users")),
        }));
        assert!(!html.contains("aria-current"), "{html}");
    }

    #[test]
    fn a_tool_opener_is_pressed_tipped_and_takes_a_right_click() {
        let html = render::<Menu>(yew::props!(MenuProps {
            label: Some(AttrValue::Static("Pen")),
            tip: Some(AttrValue::Static("Pen (right-click for its sizes)")),
            on_click: Some(Callback::noop()),
            pressed: Some(true),
        }));
        assert!(html.contains("aria-pressed=\"true\""), "{html}");
        assert!(html.contains("data-tip=\"Pen (right-click"), "{html}");
        assert!(html.contains(class::CONTEXT_TARGET), "{html}");
        // A plain menu's opener is the button alone.
        let html = render::<Menu>(yew::props!(MenuProps {
            label: Some(AttrValue::Static("Row")),
        }));
        assert!(!html.contains(class::CONTEXT_TARGET), "{html}");
        assert!(!html.contains("data-tip"), "{html}");
    }

    #[test]
    fn tab_reaches_every_control_of_a_field_menu() {
        for part in [
            "input:not(:disabled)",
            "textarea",
            "select",
            "button:not(:disabled)",
            "a[href]",
        ] {
            assert!(TABBABLE.contains(part), "{part}");
        }
    }

    #[test]
    fn a_check_and_a_radio_keep_their_marks_box() {
        let off = render::<MenuCheck>(yew::props!(MenuCheckProps {
            checked: false,
            on_change: Callback::noop(),
            children: html! { "Wrap" },
        }));
        assert!(off.contains("role=\"menuitemcheckbox\""), "{off}");
        assert!(off.contains("aria-checked=\"false\""), "{off}");
        assert!(
            off.contains(&format!("{} {}", class::MENU_ITEM_MARK, class::OFF)),
            "{off}"
        );
        let on = render::<MenuRadio>(yew::props!(MenuRadioProps {
            checked: true,
            on_pick: Callback::noop(),
        }));
        assert!(on.contains("role=\"menuitemradio\""), "{on}");
        assert!(on.contains("aria-checked=\"true\""), "{on}");
        assert!(on.contains(class::MENU_ITEM_MARK), "{on}");
        assert!(!on.contains(class::OFF), "{on}");
    }

    #[test]
    fn row_actions_put_the_danger_ones_last_then_the_menu() {
        let action =
            |label: &'static str| RowAction::new(html! { <svg /> }, label, Callback::noop());
        let html = render::<RowActions>(yew::props!(RowActionsProps {
            actions: vec![action("Delete").danger(), action("Edit")],
            more: Some(html! { <MenuItem>{ "Rename" }</MenuItem> }),
        }));
        let edit = html.find("aria-label=\"Edit\"").unwrap();
        let delete = html.find("aria-label=\"Delete\"").unwrap();
        let more = html.find("aria-label=\"More actions\"").unwrap();
        assert!(edit < delete && delete < more, "{html}");
        assert!(html.contains(class::ROW_ACTION_DANGER), "{html}");
        assert!(html.contains(class::SMALL), "{html}");
    }

    #[test]
    fn row_actions_can_be_tipped_and_their_menu_found() {
        let html = render::<RowActions>(yew::props!(RowActionsProps {
            actions: vec![RowAction::new(html! { <svg /> }, "Edit", Callback::noop())],
            more: Some(html! { <MenuItem>{ "Rename" }</MenuItem> }),
            more_testid: Some(AttrValue::Static("row-more")),
            tips: true,
        }));
        assert!(html.contains("data-tip=\"Edit\""), "{html}");
        assert!(html.contains("data-tip=\"More actions\""), "{html}");
        assert!(!html.contains("title="), "{html}");
        assert!(html.contains("data-testid=\"row-more-toggle\""), "{html}");
        let html = render::<RowActions>(yew::props!(RowActionsProps {
            actions: vec![RowAction::new(html! { <svg /> }, "Edit", Callback::noop())],
        }));
        assert!(html.contains("title=\"Edit\""), "{html}");
        assert!(!html.contains("data-tip"), "{html}");
    }

    #[test]
    fn a_split_button_joins_its_action_and_menu() {
        let html = render::<SplitButton>(yew::props!(SplitButtonProps {
            text: html! { "Save" },
            onclick: Callback::noop(),
        }));
        assert!(html.contains(class::SPLIT_BUTTON_MAIN), "{html}");
        assert!(html.contains(class::SPLIT_BUTTON_TOGGLE), "{html}");
        assert!(html.contains("aria-label=\"More options\""), "{html}");
        assert_eq!(html.matches(class::PRIMARY).count(), 2, "{html}");
    }

    #[test]
    fn a_context_menu_is_its_children_until_opened() {
        let html = render::<ContextMenu>(yew::props!(ContextMenuProps {
            items: html! { <MenuItem>{ "Copy" }</MenuItem> },
            children: html! { <p>{ "Row" }</p> },
        }));
        assert!(html.contains(class::CONTEXT_TARGET), "{html}");
        assert!(html.contains("<p>Row</p>"), "{html}");
        assert!(!html.contains("role=\"menu\""), "{html}");
    }
}
