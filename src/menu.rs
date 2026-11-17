//! Menu, its items, RowActions, SplitButton and ContextMenu
//! (`style/_menu.scss`).

use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

use crate::roving::{Typeahead, menu_step, printable, typeahead};
use crate::timer::Timer;
use crate::{
    Button, ButtonSize, ButtonVariant, Dismiss, Glyph, Icon, PopoverAlign, Side, class,
    use_overlays, use_popover,
};

/// What the arrow keys, Home, End and type-ahead move between.
const ITEMS: &str = ":is([role=menuitem], [role=menuitemcheckbox], [role=menuitemradio]):not(:disabled, [aria-disabled=true])";

/// What closes the menu when picked: an action or a radio, not a check.
const PICKS: &str =
    ":is([role=menuitem], [role=menuitemradio]):not(:disabled, [aria-disabled=true])";

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

/// The keyboard of an open menu's list: arrows, Home, End, type-ahead;
/// Tab closes it. Escape is the popover layer's.
#[hook]
fn use_menu_keys(list: NodeRef, close: Callback<()>) -> Callback<KeyboardEvent> {
    let typed = use_mut_ref(Typeahead::default);
    Callback::from(move |e: KeyboardEvent| {
        if e.key() == "Tab" {
            e.prevent_default();
            close.emit(());
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

/// Focuses the list's first item once the menu is placed.
#[hook]
fn use_focus_first(list: NodeRef, placed: bool) {
    use_effect_with(placed, move |&placed| {
        if placed && let Some(first) = items_of(&list).first() {
            _ = first.focus();
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
    side: Side,
    align: PopoverAlign,
    width: MenuWidth,
    tight: bool,
    label: Option<AttrValue>,
    class: Classes,
    testid: Option<AttrValue>,
    children: Html,
}

/// The open list in the overlay layer: placed beside `anchor`, closed by a
/// pick, Tab, Escape or a press outside.
#[function_component]
fn MenuList(props: &ListProps) -> Html {
    let overlays = use_overlays();
    let list = use_node_ref();
    let placed = use_popover(
        props.open,
        props.anchor.clone(),
        list.clone(),
        props.side,
        props.align,
        props.on_dismiss.clone(),
    );
    use_focus_first(list.clone(), placed.is_some());
    let held = use_held_width(list.clone(), placed.is_some());
    let onkeydown = use_menu_keys(list.clone(), props.on_dismiss.reform(|()| Dismiss::Close));

    if !props.open {
        return Html::default();
    }

    let onclick = {
        let on_dismiss = props.on_dismiss.clone();
        Callback::from(move |e: MouseEvent| {
            let picked = e
                .target_dyn_into::<Element>()
                .and_then(|t| t.closest(PICKS).ok().flatten());
            if picked.is_some() {
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
        <div ref={list}
            class={classes!(menu(props.width, props.tight, placed.is_some()), props.class.clone())}
            role="menu" aria-label={props.label.clone()} {style}
            data-testid={props.testid.clone()} {onclick} {onkeydown}>
            { props.children.clone() }
        </div>
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
    #[prop_or_default]
    pub width: MenuWidth,
    /// No gap between rows.
    #[prop_or_default]
    pub tight: bool,
    #[prop_or_default]
    pub side: Side,
    /// `End` by default, so it opens toward the page from an opener at a
    /// row's end.
    #[prop_or(PopoverAlign::End)]
    pub align: PopoverAlign,
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
    /// The items: [`MenuItem`], [`MenuCheck`], [`MenuRadio`],
    /// [`MenuSeparator`], [`MenuHeading`], [`MenuNote`].
    #[prop_or_default]
    pub children: Html,
}

/// A button opening a list of actions beside it in the overlay layer. A
/// pick, Tab, Escape or a press outside closes it, and Escape and a pick
/// give focus back to the opener; the arrows, Home, End and type-ahead
/// move between its items.
#[function_component]
pub fn Menu(props: &MenuProps) -> Html {
    let opener = use_node_ref();
    let (is_open, set_open) = use_open(props.open, props.on_toggle.clone());
    let onclick = {
        let set_open = set_open.clone();
        Callback::from(move |_: MouseEvent| set_open.emit(!is_open))
    };
    let on_dismiss = set_open.reform(|_: Dismiss| false);
    let has_text = !crate::button::is_empty(&props.text);
    let (icon, trailing) = match &props.icon {
        Some(icon) => (Some(icon.clone()), false),
        None if has_text => (Some(html! { <Icon glyph={Glyph::ChevronDown} /> }), true),
        None => (Some(html! { <Icon glyph={Glyph::Ellipsis} /> }), false),
    };

    html! {
        <>
            <Button node_ref={opener.clone()} variant={props.variant} size={props.size}
                class={props.opener_class.clone()} {icon} {trailing}
                label={props.label.clone()} disabled={props.disabled}
                expanded={Some(is_open)} haspopup={AttrValue::Static("menu")}
                testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-toggle")))}
                {onclick}>
                { props.text.clone() }
            </Button>
            <MenuList anchor={opener} open={is_open} {on_dismiss}
                side={props.side} align={props.align} width={props.width} tight={props.tight}
                label={props.label.clone()} class={props.class.clone()}
                testid={props.testid.clone()}>
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
    /// A note at the row's end, such as its shortcut.
    #[prop_or_default]
    pub hint: Option<AttrValue>,
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
        let detail = p.detail.clone().map(|detail| {
            html! { <span class={class::MENU_ITEM_DETAIL}>{ detail }</span> }
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
        html! {
            <button type="button" role={self.role} tabindex="-1"
                class={classes!(menu_item(p.danger), p.class.clone())}
                aria-checked={self.checked.map(|on| on.to_string())}
                aria-disabled={p.disabled.then_some("true")}
                data-testid={p.testid.clone()} {onclick}>
                { for mark }
                { for icon }
                <span class={class::MENU_ITEM_TEXT}>
                    <span class={class::MENU_ITEM_LABEL}>{ p.children.clone() }</span>
                    { for detail }
                </span>
                { for hint }
            </button>
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
    detail: &Option<AttrValue>,
    hint: &Option<AttrValue>,
    disabled: bool,
    class: &Classes,
    testid: &Option<AttrValue>,
    children: &Html,
) -> MenuItemProps {
    MenuItemProps {
        icon: None,
        detail: detail.clone(),
        hint: hint.clone(),
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
    pub children: Html,
}

/// A rule between groups of items.
#[function_component]
pub fn MenuSeparator(props: &MenuPartProps) -> Html {
    html! { <div role="separator" class={classes!(class::MENU_SEPARATOR, props.class.clone())}></div> }
}

/// A group's heading, on one line.
#[function_component]
pub fn MenuHeading(props: &MenuPartProps) -> Html {
    html! {
        <div role="presentation" class={classes!(class::MENU_HEADING, props.class.clone())}>
            { props.children.clone() }
        </div>
    }
}

/// A line of text among the items: that there are none, or what they are.
#[function_component]
pub fn MenuNote(props: &MenuPartProps) -> Html {
    html! {
        <p role="presentation" class={classes!(class::MENU_NOTE, props.class.clone())}>
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
    #[prop_or(AttrValue::Static("More actions"))]
    pub more_label: AttrValue,
    /// The menu holds nothing for this row: its opener stays, disabled, so
    /// every row ends the same way.
    #[prop_or_default]
    pub more_disabled: bool,
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
    let button = |action: &RowAction| {
        html! {
            <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                class={classes!(action.danger.then_some(class::ROW_ACTION_DANGER))}
                icon={action.icon.clone()} label={action.label.clone()} title={action.label.clone()}
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
                    label={props.more_label.clone()} disabled={props.disabled || props.more_disabled}
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
    #[prop_or(AttrValue::Static("More options"))]
    pub menu_label: AttrValue,
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
                label={props.menu_label.clone()} width={props.width}
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
            <MenuList anchor={point} {open} {on_dismiss}
                side={Side::Below} align={PopoverAlign::Start}
                width={props.width} tight={props.tight}
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
