//! Picker, PickerMenu, ChoiceList and PickerDialog: a choice among options
//! in a list with a search field over it, opened from a field, a button or
//! a dialog (`style/_picker.scss`).

use phrasebook02::yew023::use_phrases_with;
use web_sys::{Element, HtmlElement, HtmlInputElement};
use yew::prelude::*;

use crate::listbox::{ListKey, Listbox, listbox_option, option_id};
use crate::phrasebookcatalog01::{L10N, label};
use crate::roving::{menu_step, printable};
use crate::select::scroll_to;
use crate::{
    Button, ButtonSize, ButtonVariant, Dialog, DialogSize, Dismiss, Glyph, Icon, InputSize, Pager,
    PopoverAlign, Side, class, input_box, page_count, use_overlays, use_popover,
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
    /// Checked beside the current one, such as those already chosen.
    pub marked: bool,
    /// Found by the app for the query (see [`ChoiceListProps::on_search`]):
    /// listed whatever it reads like.
    pub found: bool,
    /// The value as text, the row's `data-value`.
    pub data_value: Option<AttrValue>,
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
            marked: false,
            found: false,
            data_value: None,
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

    pub fn marked(self, marked: bool) -> Self {
        Self { marked, ..self }
    }

    pub fn found(self, found: bool) -> Self {
        Self { found, ..self }
    }

    pub fn data_value(self, value: impl Into<AttrValue>) -> Self {
        Self {
            data_value: Some(value.into()),
            ..self
        }
    }
}

/// The indices of the options in which every word of `query` is found (in
/// the label, detail or keywords, ignoring case); all of them for a blank
/// one. Labels starting with the query come first, then those with a word
/// starting with its first word, then the rest, each in their own order:
/// `swed` finds Swedish before English (Sweden). A `found` option is listed
/// whatever the query, among the rest if it does not match.
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
                return o.found.then_some((2, i));
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
    #[prop_or_default]
    pub search_label: Option<AttrValue>,
    /// Said when nothing matches.
    #[prop_or_default]
    pub empty_label: Option<AttrValue>,
    /// Takes the focus when its dialog or popup opens (`data-autofocus`).
    #[prop_or_default]
    pub autofocus: bool,
    /// Told the query as it changes, and blank when the list mounts, for
    /// options the app finds by it (marked `found`).
    #[prop_or_default]
    pub on_search: Callback<String>,
    /// Lists that many matches at a time, with a [`Pager`] under them;
    /// it opens on the current option's page.
    #[prop_or_default]
    pub page_size: Option<usize>,
    /// Tab and Shift+Tab move the active option, wrapping, rather than
    /// leaving: for a list adding one thing after another.
    #[prop_or_default]
    pub tab_cycles: bool,
    #[prop_or_default]
    pub class: Classes,
    /// Its test id; the search field's is it suffixed `-search`, each
    /// option's `-option`, the pager's `-pager`.
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
    let phrases = use_phrases_with(&L10N);
    let search_label = label(&phrases, &props.search_label, "plinth-picker-search");
    let id = use_state(|| crate::tooltip::next_id("choice-list"));
    let query = use_state_eq(String::new);
    let state = use_mut_ref(Listbox::default);
    let started = use_mut_ref(|| false);
    let redraw = use_force_update();

    let options = &props.options;
    let is_chosen = |i: usize| {
        let value = &options[i].value;
        if options[i].marked {
            true
        } else if props.multi {
            props.chosen.contains(value)
        } else {
            props.value.as_ref() == Some(value)
        }
    };
    let searchable = props.always_search || options.len() > SEARCH_FROM;
    let matches = picker_matches(options, &query);
    let page = use_state_eq(|| {
        let size = props.page_size.unwrap_or(usize::MAX).max(1);
        matches
            .iter()
            .position(|&i| is_chosen(i))
            .map_or(0, |at| at / size)
    });
    let (shown, pages) = match props.page_size {
        Some(size) => {
            let size = size.max(1);
            let pages = page_count(matches.len(), size);
            let start = (*page).min(pages - 1) * size;
            let end = (start + size).min(matches.len());
            (matches[start..end].to_vec(), pages)
        }
        None => (matches.clone(), 1),
    };
    {
        let on_search = props.on_search.clone();
        use_effect_with((), move |()| on_search.emit(String::new()));
    }
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
        let tab_cycles = props.tab_cycles;
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            if tab_cycles && key == "Tab" {
                let at = state.borrow().active();
                let at = pickable.iter().position(|&i| Some(i) == at);
                let arrow = if e.shift_key() {
                    "ArrowUp"
                } else {
                    "ArrowDown"
                };
                if let Some(next) = menu_step(arrow, at, pickable.len()) {
                    e.prevent_default();
                    e.stop_propagation();
                    state.borrow_mut().set(Some(pickable[next]));
                    redraw.force_update();
                }
                return;
            }
            // In the search field, Home, End and Space edit the text.
            if searchable && matches!(key.as_str(), "Home" | "End" | " ") {
                return;
            }
            let taken = state
                .borrow_mut()
                .key(&key, e.alt_key(), &pickable, e.time_stamp());
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
        let (query, state, page) = (query.clone(), state.clone(), page.clone());
        let on_search = props.on_search.clone();
        Callback::from(move |e: InputEvent| {
            let text = e.target_unchecked_into::<HtmlInputElement>().value();
            state.borrow_mut().set(None);
            page.set(0);
            on_search.emit(text.clone());
            query.set(text);
        })
    };
    let on_page = {
        let (page, state) = (page.clone(), state.clone());
        Callback::from(move |p: usize| {
            state.borrow_mut().set(None);
            page.set(p);
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
                data-value={o.data_value.clone()}
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
                        placeholder={search_label.clone()} aria-label={search_label}
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
                style={(!searchable).then(|| format!("--choice-list-rows: {};", shown.len().max(1)))}
                data-autofocus={(props.autofocus && !searchable).then_some("")}
                onkeydown={(!searchable).then_some(onkeydown)} onmousedown={onmousedown.clone()}>
                if shown.is_empty() {
                    <div class={class::CHOICE_LIST_NOTE} role="status">{ label(&phrases, &props.empty_label, "plinth-picker-empty") }</div>
                } else {
                    { for shown.iter().copied().map(option) }
                }
            </div>
            if pages > 1 {
                <div class={class::CHOICE_LIST_PAGER} onmousedown={onmousedown.clone()}>
                    <Pager page={(*page).min(pages - 1)} {pages} {on_page} slots={5} edges={false}
                        testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-pager")))} />
                </div>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct PopupProps {
    pub anchor: NodeRef,
    pub open: bool,
    pub on_dismiss: Callback<Dismiss>,
    pub testid: Option<AttrValue>,
    pub children: Html,
}

/// A [`ChoiceList`]'s surface beside `anchor` in the overlay layer, its
/// `data-autofocus` element focused once placed; Tab closes it and moves on
/// from `anchor`.
#[function_component]
pub(crate) fn ChoicePopup(props: &PopupProps) -> Html {
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
    // Tab goes on from the opener, as if the list had not been open.
    let onkeydown = {
        let (on_dismiss, anchor) = (props.on_dismiss.clone(), props.anchor.clone());
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Tab" {
                if let Some(anchor) = anchor.cast::<HtmlElement>() {
                    _ = anchor.focus();
                }
                on_dismiss.emit(Dismiss::Outside);
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
    #[prop_or_default]
    pub search_label: Option<AttrValue>,
    #[prop_or_default]
    pub empty_label: Option<AttrValue>,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    /// The field's tooltip.
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The current option's icon alone below the phone breakpoint, its
    /// label kept for a screen reader (`.plinth-fold` on the text).
    #[prop_or_default]
    pub fold: bool,
    /// See [`ChoiceListProps::on_search`].
    #[prop_or_default]
    pub on_search: Callback<String>,
    /// See [`ChoiceListProps::page_size`].
    #[prop_or_default]
    pub page_size: Option<usize>,
    #[prop_or_default]
    pub class: Classes,
    /// The field's test id; the popup's is it suffixed `-popup`, the
    /// list's `-list` unless `list_testid` names it.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub list_testid: Option<AttrValue>,
}

/// The list's test id: `list` or the opener's suffixed `-list`.
fn list_testid(testid: &Option<AttrValue>, list: &Option<AttrValue>) -> Option<AttrValue> {
    list.clone().or_else(|| {
        testid
            .as_ref()
            .map(|t| AttrValue::from(format!("{t}-list")))
    })
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
                <span class={classes!(class::PICKER_TEXT, (props.fold && o.icon.is_some()).then_some(class::FOLD))}>
                    { o.label.clone() }
                </span>
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
                title={props.title.clone()}
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
                    autofocus=true on_search={props.on_search.clone()} page_size={props.page_size}
                    testid={list_testid(&props.testid, &props.list_testid)} />
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
    #[prop_or_default]
    pub search_label: Option<AttrValue>,
    #[prop_or_default]
    pub empty_label: Option<AttrValue>,
    /// See [`ChoiceListProps::on_search`].
    #[prop_or_default]
    pub on_search: Callback<String>,
    /// See [`ChoiceListProps::page_size`].
    #[prop_or_default]
    pub page_size: Option<usize>,
    /// See [`ChoiceListProps::tab_cycles`].
    #[prop_or_default]
    pub tab_cycles: bool,
    /// See [`crate::ButtonProps::fold`].
    #[prop_or_default]
    pub fold: bool,
    /// The button's classes, beside the button's own.
    #[prop_or_default]
    pub class: Classes,
    /// The button's test id; the popup's is it suffixed `-popup`, the
    /// list's `-list` unless `list_testid` names it.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub list_testid: Option<AttrValue>,
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
                class={props.class.clone()} {icon} {trailing} fold={props.fold}
                label={Some(props.label.clone())} disabled={props.disabled}
                expanded={Some(*open)} haspopup={AttrValue::Static("listbox")}
                testid={props.testid.clone()} {onclick}>
                { props.text.clone() }
            </Button>
            <ChoicePopup anchor={opener} open={*open} {on_dismiss} testid={props.testid.clone()}>
                <ChoiceList<T> options={props.options.clone()} {on_pick}
                    label={props.label.clone()} always_search={props.always_search}
                    search_label={props.search_label.clone()} empty_label={props.empty_label.clone()}
                    autofocus=true on_search={props.on_search.clone()} page_size={props.page_size}
                    tab_cycles={props.tab_cycles}
                    testid={list_testid(&props.testid, &props.list_testid)} />
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
    #[prop_or_default]
    pub search_label: Option<AttrValue>,
    #[prop_or_default]
    pub empty_label: Option<AttrValue>,
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
    fn a_found_option_is_listed_whatever_the_query() {
        let mut o = languages();
        o.push(PickerOption::new(5, "Klingon").found(true));
        assert_eq!(picker_matches(&o, "swedish fi"), [4, 5]);
        assert_eq!(picker_matches(&o, "kling"), [5]);
    }

    #[test]
    fn marked_options_are_checked_beside_the_current_one() {
        let options: Vec<_> = languages()
            .into_iter()
            .map(|o| {
                let (marked, text) = (o.value == 1, format!("lang-{}", o.value));
                o.marked(marked).data_value(text)
            })
            .collect();
        let html = render(yew::props!(ChoiceListProps<u32> {
            options,
            value: Some(3),
            label: AttrValue::Static("Language"),
        }));
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 2, "{html}");
        assert!(html.contains("data-value=\"lang-4\""), "{html}");
    }

    #[test]
    fn a_page_size_lists_a_page_with_a_pager() {
        let options: Vec<_> = (0..30u32)
            .map(|i| PickerOption::new(i, format!("Option {i}")))
            .collect();
        let html = render(yew::props!(ChoiceListProps<u32> {
            options,
            value: Some(27),
            page_size: Some(25),
            label: AttrValue::Static("Options"),
            testid: AttrValue::Static("list"),
        }));
        assert_eq!(html.matches("role=\"option\"").count(), 5, "{html}");
        assert!(html.contains(">Option 27<"), "{html}");
        assert!(html.contains("data-testid=\"list-pager\""), "{html}");
        assert!(html.contains(class::CHOICE_LIST_PAGER), "{html}");
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
