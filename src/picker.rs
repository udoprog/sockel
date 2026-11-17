//! Picker, PickerMenu, ChoiceList and PickerDialog: a choice among options
//! in a list with a search field over it, opened from a field, a button or
//! a dialog (`style/_picker.scss`).

use web_sys::{Element, HtmlElement, HtmlInputElement};
use yew::prelude::*;

use crate::listbox::{ListKey, Listbox, listbox_option, option_id};
use crate::select::{printable, scroll_to};
use crate::{
    Button, ButtonSize, ButtonVariant, Dialog, DialogSize, Dismiss, Glyph, Icon, InputSize,
    PopoverAlign, Side, class, input_box, use_overlays, use_popover,
};

/// How many options a list has before it offers a search field.
pub const SEARCH_FROM: usize = 8;

/// One option of a [`ChoiceList`].
#[derive(Clone, Debug, PartialEq)]
pub struct PickerOption<T> {
    pub value: T,
    pub label: AttrValue,
    /// Muted after the label, and searched too.
    pub detail: Option<AttrValue>,
    /// Before the label: an icon, flag or swatch the app supplies.
    pub icon: Option<Html>,
    /// At the row's end: a tag or badge the app supplies.
    pub badge: Option<Html>,
    /// More words it is found by, not shown, such as a language's codes.
    pub keywords: Option<AttrValue>,
    pub disabled: bool,
}

impl<T> PickerOption<T> {
    pub fn new(value: T, label: impl Into<AttrValue>) -> Self {
        Self {
            value,
            label: label.into(),
            detail: None,
            icon: None,
            badge: None,
            keywords: None,
            disabled: false,
        }
    }

    pub fn detail(self, detail: impl Into<AttrValue>) -> Self {
        Self {
            detail: Some(detail.into()),
            ..self
        }
    }

    pub fn icon(self, icon: Html) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }

    pub fn badge(self, badge: Html) -> Self {
        Self {
            badge: Some(badge),
            ..self
        }
    }

    pub fn keywords(self, keywords: impl Into<AttrValue>) -> Self {
        Self {
            keywords: Some(keywords.into()),
            ..self
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }
}

/// The indices of the options in which every word of `query` is found (in
/// the label, detail or keywords, ignoring case); all of them for a blank
/// one. Labels starting with the query come first, then those with a word
/// starting with its first word, then the rest, each in their own order:
/// `swed` finds Swedish before English (Sweden).
pub fn picker_matches<T>(options: &[PickerOption<T>], query: &str) -> Vec<usize> {
    let needle = query.trim().to_lowercase();
    let words: Vec<&str> = needle.split_whitespace().collect();
    let Some(first) = words.first() else {
        return (0..options.len()).collect();
    };
    let mut found: Vec<(u8, usize)> = options
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let label = o.label.to_lowercase();
            let detail = o.detail.as_deref().unwrap_or_default().to_lowercase();
            let keywords = o.keywords.as_deref().unwrap_or_default().to_lowercase();
            let has = |w: &&str| label.contains(w) || detail.contains(w) || keywords.contains(w);
            if !words.iter().all(has) {
                return None;
            }
            let rank = if label.starts_with(&needle) {
                0
            } else if label
                .split(|c: char| !c.is_alphanumeric())
                .any(|w| w.starts_with(first))
            {
                1
            } else {
                2
            };
            Some((rank, i))
        })
        .collect();
    found.sort();
    found.into_iter().map(|(_, i)| i).collect()
}

/// How a [`Picker`]'s field looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PickerVariant {
    /// A form field, as tall as the text fields beside it and
    /// `--menu-width-wide` wide.
    #[default]
    Field,
    /// A small field (`--control-height-small`, `--menu-width` wide), for
    /// dense places.
    Small,
}

/// A picker's field classes: the input box's, `picker`, `small` for
/// [`PickerVariant::Small`] and `fill`.
pub fn picker(variant: PickerVariant, fill: bool, invalid: bool, disabled: bool) -> Classes {
    let size = match variant {
        PickerVariant::Field => InputSize::Regular,
        PickerVariant::Small => InputSize::Small,
    };
    classes!(
        input_box(size, invalid, disabled, false),
        class::PICKER,
        fill.then_some(class::FILL)
    )
}

#[derive(Properties, PartialEq)]
pub struct ChoiceListProps<T: PartialEq + 'static> {
    pub options: Vec<PickerOption<T>>,
    /// The current option, checked.
    #[prop_or_default]
    pub value: Option<T>,
    /// Several at once (`aria-multiselectable`): `chosen` are checked, and
    /// a pick toggles one.
    #[prop_or_default]
    pub multi: bool,
    #[prop_or_default]
    pub chosen: Vec<T>,
    /// Each pick; with `multi`, the owner toggles it (see
    /// [`crate::toggled`]).
    #[prop_or_default]
    pub on_pick: Callback<T>,
    /// The list's accessible name.
    pub label: AttrValue,
    /// Offer the search field however few options there are (it comes
    /// past [`SEARCH_FROM`] anyway).
    #[prop_or_default]
    pub always_search: bool,
    /// The search field's placeholder and name (the app's phrasebook).
    #[prop_or("Search".into())]
    pub search_label: AttrValue,
    /// Said when nothing matches.
    #[prop_or("No matches".into())]
    pub empty_label: AttrValue,
    /// Takes the focus when its dialog or popup opens (`data-autofocus`).
    #[prop_or_default]
    pub autofocus: bool,
    #[prop_or_default]
    pub class: Classes,
    /// Its test id; the search field's is it suffixed `-search`, each
    /// option's `-option`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// Options in a listbox `--picker-rows` rows high, under a search field
/// when there are more than a few (or `always_search`). Focus stays in the
/// field (or on the list, without one), naming the active option with
/// `aria-activedescendant`: the arrows, PageUp and PageDown move, Enter
/// picks; without a field, Home, End, Space and type-ahead too.
#[function_component]
pub fn ChoiceList<T>(props: &ChoiceListProps<T>) -> Html
where
    T: PartialEq + Clone + 'static,
{
    let id = use_state(|| crate::tooltip::next_id("choice-list"));
    let query = use_state_eq(String::new);
    let state = use_mut_ref(Listbox::default);
    let started = use_mut_ref(|| false);
    let redraw = use_force_update();

    let options = &props.options;
    let is_chosen = |i: usize| {
        let value = &options[i].value;
        if props.multi {
            props.chosen.contains(value)
        } else {
            props.value.as_ref() == Some(value)
        }
    };
    let searchable = props.always_search || options.len() > SEARCH_FROM;
    let shown = picker_matches(options, &query);
    let pickable: Vec<usize> = shown
        .iter()
        .copied()
        .filter(|&i| !options[i].disabled)
        .collect();
    {
        let mut state = state.borrow_mut();
        if !std::mem::replace(&mut *started.borrow_mut(), true) {
            match pickable.iter().copied().find(|&i| is_chosen(i)) {
                Some(i) => state.set(Some(i)),
                None => state.go_to_end(&pickable, false),
            }
        } else {
            state.settle(&pickable);
        }
    }
    let active = state.borrow().active();
    {
        let (state, base) = (state.clone(), (*id).clone());
        use_effect(move || {
            let mut state = state.borrow_mut();
            if state.take_scroll()
                && let Some(active) = state.active()
            {
                scroll_to(&option_id(&base, active));
            }
        });
    }

    let pick = {
        let on_pick = props.on_pick.clone();
        let values: Vec<(T, bool)> = options
            .iter()
            .map(|o| (o.value.clone(), o.disabled))
            .collect();
        move |i: usize| {
            if let Some((value, false)) = values.get(i) {
                on_pick.emit(value.clone());
            }
        }
    };
    let onkeydown = {
        let (state, redraw, pick, pickable) = (
            state.clone(),
            redraw.clone(),
            pick.clone(),
            pickable.clone(),
        );
        let labels: Vec<String> = options.iter().map(|o| o.label.to_string()).collect();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            // In the search field, Home, End and Space edit the text.
            if searchable && matches!(key.as_str(), "Home" | "End" | " ") {
                return;
            }
            let taken = state.borrow_mut().key(&key, e.alt_key(), &pickable);
            match taken {
                ListKey::Moved => {}
                ListKey::Pick(i) => pick(i),
                ListKey::Ignored if searchable => return,
                ListKey::Ignored => match printable(&e) {
                    Some(c) => {
                        let names: Vec<&str> = labels.iter().map(String::as_str).collect();
                        state
                            .borrow_mut()
                            .typeahead(&pickable, &names, c, e.time_stamp());
                    }
                    None => return,
                },
            }
            e.prevent_default();
            redraw.force_update();
        })
    };
    let oninput = {
        let (query, state) = (query.clone(), state.clone());
        Callback::from(move |e: InputEvent| {
            state.borrow_mut().set(None);
            query.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    let list_id = format!("{}-list", *id);
    let decorated = options.iter().any(|o| o.icon.is_some());
    let option_testid = props
        .testid
        .as_ref()
        .map(|t| AttrValue::from(format!("{t}-option")));
    let option = |i: usize| {
        let o = &options[i];
        let selected = is_chosen(i);
        let is_active = active == Some(i);
        let onclick = {
            let pick = pick.clone();
            Callback::from(move |_: MouseEvent| pick(i))
        };
        let onmousemove = (!is_active && !o.disabled).then(|| {
            let (state, redraw) = (state.clone(), redraw.clone());
            Callback::from(move |_: MouseEvent| {
                if state.borrow_mut().hover(i) {
                    redraw.force_update();
                }
            })
        });
        html! {
            <div id={option_id(&id, i)} role="option" class={listbox_option(is_active)}
                aria-selected={selected.to_string()}
                aria-disabled={o.disabled.then_some("true")}
                data-testid={option_testid.clone()} {onclick} {onmousemove}>
                <span class={classes!(class::LISTBOX_OPTION_MARK, (!selected).then_some(class::OFF))}
                    aria-hidden="true">
                    <Icon glyph={Glyph::Check} />
                </span>
                if decorated {
                    <span class={class::LISTBOX_OPTION_ICON} aria-hidden="true">
                        { o.icon.clone().unwrap_or_default() }
                    </span>
                }
                <span class={class::LISTBOX_OPTION_LABEL}>{ o.label.clone() }</span>
                if let Some(detail) = &o.detail {
                    <span class={class::CHOICE_LIST_DETAIL}>{ detail.clone() }</span>
                }
                if let Some(badge) = &o.badge {
                    <span class={class::CHOICE_LIST_BADGE}>{ badge.clone() }</span>
                }
            </div>
        }
    };
    let active_id = active.map(|i| option_id(&id, i));
    // Focus stays in the search field.
    let onmousedown = searchable.then(|| Callback::from(|e: MouseEvent| e.prevent_default()));

    html! {
        <div class={classes!(class::CHOICE_LIST, props.class.clone())}
            data-testid={props.testid.clone()}>
            if searchable {
                <span class={classes!(input_box(InputSize::Regular, false, false, false), class::CHOICE_LIST_SEARCH)}>
                    <span class={class::INPUT_MARK}><Icon glyph={Glyph::Search} /></span>
                    <input class={class::INPUT_CONTROL} type="text" value={(*query).clone()}
                        placeholder={props.search_label.clone()} aria-label={props.search_label.clone()}
                        role="combobox" aria-autocomplete="list" aria-expanded="true"
                        aria-controls={list_id.clone()} aria-activedescendant={active_id.clone()}
                        autocomplete="off" spellcheck="false"
                        data-autofocus={props.autofocus.then_some("")}
                        data-testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-search")))}
                        {oninput} onkeydown={onkeydown.clone()} />
                </span>
            }
            <div id={list_id.clone()} role="listbox" class={class::CHOICE_LIST_OPTIONS}
                aria-label={props.label.clone()}
                aria-multiselectable={props.multi.then_some("true")}
                tabindex={(!searchable).then_some("0")}
                aria-activedescendant={(!searchable).then_some(active_id).flatten()}
                data-autofocus={(props.autofocus && !searchable).then_some("")}
                onkeydown={(!searchable).then_some(onkeydown)} {onmousedown}>
                if shown.is_empty() {
                    <div class={class::CHOICE_LIST_NOTE} role="status">{ props.empty_label.clone() }</div>
                } else {
                    { for shown.iter().copied().map(option) }
                }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct PopupProps {
    anchor: NodeRef,
    open: bool,
    on_dismiss: Callback<Dismiss>,
    testid: Option<AttrValue>,
    children: Html,
}

/// A [`ChoiceList`]'s surface beside `anchor` in the overlay layer, its
/// `data-autofocus` element focused once placed; Tab closes it.
#[function_component]
fn ChoicePopup(props: &PopupProps) -> Html {
    let overlays = use_overlays();
    let popup = use_node_ref();
    let placed = use_popover(
        props.open,
        props.anchor.clone(),
        popup.clone(),
        Side::Below,
        PopoverAlign::Start,
        props.on_dismiss.clone(),
    );
    {
        let popup = popup.clone();
        use_effect_with(placed.is_some(), move |&placed| {
            let target = placed
                .then(|| popup.cast::<Element>())
                .flatten()
                .and_then(|p| p.query_selector("[data-autofocus]").ok().flatten())
                .and_then(|e| wasm_bindgen::JsCast::dyn_into::<HtmlElement>(e).ok());
            if let Some(target) = target {
                _ = target.focus();
            }
        });
    }
    if !props.open {
        return Html::default();
    }
    let onkeydown = {
        let on_dismiss = props.on_dismiss.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Tab" {
                e.prevent_default();
                on_dismiss.emit(Dismiss::Escape);
            }
        })
    };
    let anchor_width = props.anchor.cast::<HtmlElement>().map(|a| a.offset_width());
    let style = placed.map(|p| {
        let mut style = format!(
            "--popover-left: {}px; --popover-top: {}px; --popover-max-height: {}px;",
            p.left, p.top, p.max_height
        );
        if let Some(w) = anchor_width {
            style.push_str(&format!(" --listbox-anchor-width: {w}px;"));
        }
        style
    });
    overlays.portal(html! {
        <div ref={popup} class={classes!(class::PICKER_POPUP, placed.is_some().then_some(class::PLACED))}
            {style} {onkeydown}
            data-testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-popup")))}>
            { props.children.clone() }
        </div>
    })
}

#[derive(Properties, PartialEq)]
pub struct PickerProps<T: PartialEq + 'static> {
    pub options: Vec<PickerOption<T>>,
    /// The current option, shown in the field.
    #[prop_or_default]
    pub value: Option<T>,
    #[prop_or_default]
    pub on_change: Callback<T>,
    /// The field's and the list's accessible name.
    pub label: AttrValue,
    /// Shown, muted, while nothing is current.
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub variant: PickerVariant,
    /// As wide as its container (`.plinth-fill`).
    #[prop_or_default]
    pub fill: bool,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// See [`ChoiceListProps::always_search`].
    #[prop_or_default]
    pub always_search: bool,
    #[prop_or("Search".into())]
    pub search_label: AttrValue,
    #[prop_or("No matches".into())]
    pub empty_label: AttrValue,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    /// The field's test id; the popup's is it suffixed `-popup`, the
    /// list's `-list`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A field of one width showing the current option (its icon and label),
/// opening a [`ChoiceList`] below it; a pick closes it, and Escape, Tab or
/// a press outside too. ArrowDown, ArrowUp, Enter and Space open it.
#[function_component]
pub fn Picker<T>(props: &PickerProps<T>) -> Html
where
    T: PartialEq + Clone + 'static,
{
    let face = use_node_ref();
    let open = use_state_eq(|| false);
    let on_dismiss = {
        let open = open.clone();
        Callback::from(move |_: Dismiss| open.set(false))
    };
    let on_pick = {
        let (open, on_change, value) = (open.clone(), props.on_change.clone(), props.value.clone());
        Callback::from(move |v: T| {
            open.set(false);
            if value.as_ref() != Some(&v) {
                on_change.emit(v);
            }
        })
    };
    let toggle = {
        let (open, disabled) = (open.clone(), props.disabled);
        Callback::from(move |_: MouseEvent| {
            if !disabled {
                open.set(!*open);
            }
        })
    };
    let onkeydown = {
        let open = open.clone();
        Callback::from(move |e: KeyboardEvent| {
            if !*open && matches!(e.key().as_str(), "ArrowDown" | "ArrowUp") {
                e.prevent_default();
                open.set(true);
            }
        })
    };

    let current = props
        .value
        .as_ref()
        .and_then(|v| props.options.iter().find(|o| &o.value == v));
    let text = match current {
        Some(o) => html! {
            <>
                if let Some(icon) = &o.icon {
                    <span class={class::PICKER_ICON} aria-hidden="true">{ icon.clone() }</span>
                }
                <span class={class::PICKER_TEXT}>{ o.label.clone() }</span>
            </>
        },
        None => html! {
            <span class={class::PICKER_PLACEHOLDER}>{ props.placeholder.clone() }</span>
        },
    };
    let name = match current {
        Some(o) => format!("{}: {}", props.label, o.label),
        None => props.label.to_string(),
    };

    html! {
        <>
            <button ref={face.clone()} type="button" id={props.id.clone()}
                class={classes!(
                    picker(props.variant, props.fill, props.invalid, props.disabled),
                    props.class.clone()
                )}
                aria-haspopup="listbox" aria-expanded={open.to_string()}
                aria-label={name} aria-describedby={props.describedby.clone()}
                aria-invalid={props.invalid.then_some("true")}
                disabled={props.disabled} data-testid={props.testid.clone()}
                onclick={toggle} {onkeydown}>
                { text }
                <span class={class::PICKER_CHEVRON} aria-hidden="true">
                    <Icon glyph={Glyph::ChevronDown} />
                </span>
            </button>
            <ChoicePopup anchor={face} open={*open} {on_dismiss} testid={props.testid.clone()}>
                <ChoiceList<T> options={props.options.clone()} value={props.value.clone()} {on_pick}
                    label={props.label.clone()} always_search={props.always_search}
                    search_label={props.search_label.clone()} empty_label={props.empty_label.clone()}
                    autofocus=true
                    testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-list")))} />
            </ChoicePopup>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct PickerMenuProps<T: PartialEq + 'static> {
    pub options: Vec<PickerOption<T>>,
    #[prop_or_default]
    pub on_pick: Callback<T>,
    /// The button's visible text: the action, such as Add a key.
    #[prop_or_default]
    pub text: Html,
    /// The button's icon, which the app supplies; else it ends in a chevron.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// The button's and the list's accessible name.
    pub label: AttrValue,
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub always_search: bool,
    #[prop_or("Search".into())]
    pub search_label: AttrValue,
    #[prop_or("No matches".into())]
    pub empty_label: AttrValue,
    /// The button's classes, beside the button's own.
    #[prop_or_default]
    pub class: Classes,
    /// The button's test id; the popup's is it suffixed `-popup`, the
    /// list's `-list`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A [`Button`] for an action that takes an option (adding one), opening a
/// [`ChoiceList`] as a [`Picker`] does; a pick closes it.
#[function_component]
pub fn PickerMenu<T>(props: &PickerMenuProps<T>) -> Html
where
    T: PartialEq + Clone + 'static,
{
    let opener = use_node_ref();
    let open = use_state_eq(|| false);
    let on_dismiss = {
        let open = open.clone();
        Callback::from(move |_: Dismiss| open.set(false))
    };
    let on_pick = {
        let (open, on_pick) = (open.clone(), props.on_pick.clone());
        Callback::from(move |v: T| {
            open.set(false);
            on_pick.emit(v);
        })
    };
    let onclick = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(!*open))
    };
    let (icon, trailing) = match &props.icon {
        Some(icon) => (Some(icon.clone()), false),
        None => (Some(html! { <Icon glyph={Glyph::ChevronDown} /> }), true),
    };

    html! {
        <>
            <Button node_ref={opener.clone()} variant={props.variant} size={props.size}
                class={props.class.clone()} {icon} {trailing}
                label={Some(props.label.clone())} disabled={props.disabled}
                expanded={Some(*open)} haspopup={AttrValue::Static("listbox")}
                testid={props.testid.clone()} {onclick}>
                { props.text.clone() }
            </Button>
            <ChoicePopup anchor={opener} open={*open} {on_dismiss} testid={props.testid.clone()}>
                <ChoiceList<T> options={props.options.clone()} {on_pick}
                    label={props.label.clone()} always_search={props.always_search}
                    search_label={props.search_label.clone()} empty_label={props.empty_label.clone()}
                    autofocus=true
                    testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-list")))} />
            </ChoicePopup>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct PickerDialogProps<T: PartialEq + 'static> {
    pub open: bool,
    pub on_dismiss: Callback<Dismiss>,
    pub title: AttrValue,
    pub options: Vec<PickerOption<T>>,
    /// The current option, checked.
    #[prop_or_default]
    pub value: Option<T>,
    /// Several at once: see [`ChoiceListProps::multi`].
    #[prop_or_default]
    pub multi: bool,
    #[prop_or_default]
    pub chosen: Vec<T>,
    /// Each pick; the owner closes it after a single one.
    #[prop_or_default]
    pub on_pick: Callback<T>,
    #[prop_or("Search".into())]
    pub search_label: AttrValue,
    #[prop_or("No matches".into())]
    pub empty_label: AttrValue,
    #[prop_or_default]
    pub size: DialogSize,
    /// The footer's buttons, such as Done for a multi-select.
    #[prop_or_default]
    pub actions: Html,
    #[prop_or_default]
    pub class: Classes,
    /// The dialog's test id; the list's is it suffixed `-list`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// The searchable picker shell for a large set (languages, countries): a
/// [`Dialog`] holding a [`ChoiceList`] that always searches, its field
/// focused, the dialog one height whatever matches.
#[function_component]
pub fn PickerDialog<T>(props: &PickerDialogProps<T>) -> Html
where
    T: PartialEq + Clone + 'static,
{
    html! {
        <Dialog open={props.open} on_dismiss={props.on_dismiss.clone()} title={props.title.clone()}
            size={props.size} actions={props.actions.clone()}
            class={props.class.clone()} testid={props.testid.clone()}>
            <ChoiceList<T> options={props.options.clone()} value={props.value.clone()}
                multi={props.multi} chosen={props.chosen.clone()} on_pick={props.on_pick.clone()}
                label={props.title.clone()} always_search=true autofocus=true
                search_label={props.search_label.clone()} empty_label={props.empty_label.clone()}
                testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-list")))} />
        </Dialog>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn render(props: ChoiceListProps<u32>) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<ChoiceList<u32>>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn languages() -> Vec<PickerOption<u32>> {
        vec![
            PickerOption::new(0, "English (Sweden)").detail("en-SE"),
            PickerOption::new(1, "Lule Sami (Sweden)"),
            PickerOption::new(2, "Dutch").keywords("swe"),
            PickerOption::new(3, "Swedish").detail("sv"),
            PickerOption::new(4, "Swedish (Finland)").detail("sv-FI"),
        ]
    }

    #[test]
    fn variant_fill_and_states_are_modifiers() {
        assert_eq!(
            names(picker(PickerVariant::Field, false, false, false)),
            [class::INPUT_BOX, class::PICKER]
        );
        assert_eq!(
            names(picker(PickerVariant::Small, true, true, true)),
            [
                class::INPUT_BOX,
                class::SMALL,
                class::INVALID,
                class::DISABLED,
                class::PICKER,
                class::FILL
            ]
        );
    }

    #[test]
    fn ranks_label_starts_then_word_starts_then_the_rest() {
        let o = languages();
        assert_eq!(picker_matches(&o, ""), [0, 1, 2, 3, 4]);
        assert_eq!(picker_matches(&o, "swed"), [3, 4, 0, 1]);
        assert_eq!(picker_matches(&o, "swe"), [3, 4, 0, 1, 2]);
        assert_eq!(picker_matches(&o, " SWEDISH fi "), [4]);
        assert_eq!(picker_matches(&o, "sv-fi"), [4]);
        assert!(picker_matches(&o, "dutch sv").is_empty());
    }

    #[test]
    fn a_few_options_list_without_a_search_field() {
        let html = render(yew::props!(ChoiceListProps<u32> {
            options: languages(),
            value: Some(3),
            label: AttrValue::Static("Language"),
        }));
        assert!(!html.contains("role=\"combobox\""), "{html}");
        assert!(html.contains("tabindex=\"0\""), "{html}");
        assert!(html.contains("aria-selected=\"true\""), "{html}");
        assert!(html.contains(">en-SE</span>"), "{html}");
    }

    #[test]
    fn multi_checks_every_chosen_one_under_a_search_field() {
        let html = render(yew::props!(ChoiceListProps<u32> {
            options: languages(),
            multi: true,
            chosen: vec![1, 4],
            always_search: true,
            label: AttrValue::Static("Languages"),
        }));
        assert!(html.contains("role=\"combobox\""), "{html}");
        assert!(html.contains("aria-multiselectable=\"true\""), "{html}");
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 2, "{html}");
    }

    #[test]
    fn many_options_offer_the_search_field() {
        let options: Vec<_> = (0..=SEARCH_FROM as u32)
            .map(|i| PickerOption::new(i, format!("Option {i}")))
            .collect();
        let html = render(yew::props!(ChoiceListProps<u32> {
            options,
            label: AttrValue::Static("Options"),
        }));
        assert!(html.contains("role=\"combobox\""), "{html}");
    }
}
