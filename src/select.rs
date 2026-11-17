//! Select: a field's box showing what is chosen, opening a [`Listbox`] of
//! its options (`style/_select.scss`).

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::listbox::{ListKey, Listbox, face_label, listbox, listbox_option, option_id};
use crate::roving::printable;
use crate::{
    Button, ButtonSize, ButtonVariant, Dismiss, Glyph, Icon, InputSize, PopoverAlign, Side, class,
    input_box, use_overlays,
};

/// One option of a [`Select`].
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption<T> {
    pub value: T,
    pub label: AttrValue,
    /// Before its label in the list (not on the face, which keeps one
    /// width).
    pub icon: Option<Html>,
    /// Shown, but cannot be picked.
    pub disabled: bool,
    /// A line under its label in the list, such as why it cannot be picked
    /// (the app's, so it may carry its own class or test id).
    pub description: Option<Html>,
    /// The heading it is listed under. A group's options are given
    /// together; options of no group are listed by themselves.
    pub group: Option<AttrValue>,
}

impl<T> SelectOption<T> {
    pub fn new(value: T, label: impl Into<AttrValue>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            disabled: false,
            description: None,
            group: None,
        }
    }

    pub fn icon(self, icon: Html) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }

    pub fn description(self, description: Html) -> Self {
        Self {
            description: Some(description),
            ..self
        }
    }

    pub fn group(self, group: impl Into<AttrValue>) -> Self {
        Self {
            group: Some(group.into()),
            ..self
        }
    }
}

/// `options` in runs of one group (`None`: those of no group), in order.
fn group_runs<T>(options: &[SelectOption<T>]) -> Vec<(Option<&AttrValue>, Vec<usize>)> {
    let mut runs: Vec<(Option<&AttrValue>, Vec<usize>)> = Vec::new();
    for (i, option) in options.iter().enumerate() {
        match runs.last_mut() {
            Some((group, run)) if *group == option.group.as_ref() => run.push(i),
            _ => runs.push((option.group.as_ref(), vec![i])),
        }
    }
    runs
}

/// How wide a [`Select`]'s face is, whatever is chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectWidth {
    /// Its widest option or placeholder, cut past its container.
    #[default]
    Widest,
    /// Its container's (`.plinth-fill`).
    Fill,
}

/// A face's classes: the input box's (`input-box`, size, `invalid`,
/// `disabled`, `busy`), `select`, and `fill` for [`SelectWidth::Fill`].
pub fn select(
    size: InputSize,
    width: SelectWidth,
    invalid: bool,
    disabled: bool,
    busy: bool,
) -> Classes {
    classes!(
        input_box(size, invalid, disabled, busy),
        class::SELECT,
        (width == SelectWidth::Fill).then_some(class::FILL)
    )
}

/// Whether `option` is listed with `filter` typed: its label or its
/// group's holds it, ignoring case.
pub fn filter_matches<T>(option: &SelectOption<T>, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    let filter = filter.to_lowercase();
    option.label.to_lowercase().contains(&filter)
        || option
            .group
            .as_ref()
            .is_some_and(|g| g.to_lowercase().contains(&filter))
}

/// The index of the option holding `value`.
pub fn chosen<T: PartialEq>(options: &[SelectOption<T>], value: Option<&T>) -> Option<usize> {
    let value = value?;
    options.iter().position(|o| &o.value == value)
}

#[derive(Properties, PartialEq)]
pub struct SelectProps<T: PartialEq + 'static> {
    pub options: Vec<SelectOption<T>>,
    /// What is chosen; the placeholder while `None` or not an option.
    #[prop_or_default]
    pub value: Option<T>,
    /// Told an option picked that is not the one chosen.
    #[prop_or_default]
    pub on_change: Callback<T>,
    /// Shown, muted, while nothing is chosen.
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    /// The accessible name without a `Field` (`aria-label`).
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// The face's id, a `Field`'s `id`, so its label names and opens it.
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    #[prop_or_default]
    pub width: SelectWidth,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// Its options are loading: it opens on what it has (`aria-busy`).
    #[prop_or_default]
    pub busy: bool,
    /// The face's classes.
    #[prop_or_default]
    pub class: Classes,
    /// The face's test id; the list's is it suffixed `-options`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// From how many options the list has a filter field above it, which
    /// takes focus on opening; `None` (the default), never.
    #[prop_or_default]
    pub filter_from: Option<usize>,
    /// The filter field's placeholder and accessible name.
    #[prop_or(AttrValue::Static("Filter"))]
    pub filter_label: AttrValue,
    /// Shown when the filter matches no option.
    #[prop_or(AttrValue::Static("No matches"))]
    pub no_match: AttrValue,
    /// Picks several values ([`SelectProps::values`]) in place of one: a
    /// pick checks or unchecks its option and the list stays open.
    #[prop_or_default]
    pub multiple: bool,
    /// What is picked, with `multiple`.
    #[prop_or_default]
    pub values: Vec<T>,
    /// Told the values picked after one was checked or unchecked, in the
    /// options' order, with `multiple`.
    #[prop_or_default]
    pub on_change_values: Callback<Vec<T>>,
    /// The face's text while several are picked, such as how many rows
    /// they match together; by default the first picked and how many more
    /// (`Small +2`).
    #[prop_or_default]
    pub summary: Option<AttrValue>,
    /// Below the phone breakpoint, the list is a sheet from the bottom over
    /// a scrim, headed by `label` and a close button, its filter field (if
    /// any) not focused on opening so the keyboard does not cover it. Off
    /// by default.
    #[prop_or_default]
    pub sheet: bool,
    /// The sheet's close button's accessible name.
    #[prop_or(AttrValue::Static("Close"))]
    pub close_label: AttrValue,
}

/// The indexes of the options picked: `values` with `multiple`, else the
/// one holding `value`.
pub fn picked<T: PartialEq>(
    options: &[SelectOption<T>],
    multiple: bool,
    value: Option<&T>,
    values: &[T],
) -> Vec<usize> {
    if !multiple {
        return chosen(options, value).into_iter().collect();
    }
    (0..options.len())
        .filter(|&i| values.contains(&options[i].value))
        .collect()
}

/// A multiple face's text while several are picked: `summary`, else the
/// first and how many more.
fn summarize(first: &str, picked: usize, summary: Option<&AttrValue>) -> Option<AttrValue> {
    (picked > 1).then(|| match summary {
        Some(summary) => summary.clone(),
        None => format!("{first} +{}", picked - 1).into(),
    })
}

/// The WAI-ARIA select-only combobox: a face in the tab order (a button,
/// `role="combobox"`) showing what is chosen, opening a listbox in the
/// overlay layer while focus stays on it. The arrows, Enter, Space, Home,
/// End and typing open it; open, the arrows, PageUp, PageDown, Home, End
/// and type-ahead move, Enter, Space and Alt+ArrowUp pick, Escape and Tab
/// close. Its face is one width whatever is chosen ([`SelectWidth`]).
#[function_component]
pub fn Select<T>(props: &SelectProps<T>) -> Html
where
    T: PartialEq + Clone + 'static,
{
    let overlays = use_overlays();
    let id = use_state(|| crate::tooltip::next_id("select"));
    let face = use_node_ref();
    let list = use_node_ref();
    let open = use_state_eq(|| false);
    let state = use_mut_ref(Listbox::default);
    let redraw = use_force_update();
    // Enter or Space opened or picked: the browser's click that follows
    // would toggle it again.
    let swallow = use_mut_ref(|| false);
    // What the filter field holds, and the field.
    let filter = use_state(String::new);
    let input = use_node_ref();
    // The sheet's head, shown only while the list is a sheet.
    let head = use_node_ref();

    let on_dismiss = {
        let open = open.clone();
        Callback::from(move |_: Dismiss| open.set(false))
    };
    let placed = crate::use_popover(
        *open,
        face.clone(),
        list.clone(),
        Side::Below,
        PopoverAlign::Start,
        on_dismiss,
    );
    let held = crate::menu::use_held_width(list.clone(), placed.is_some());
    let filtering = props
        .filter_from
        .is_some_and(|from| props.options.len() >= from);
    {
        let (input, head) = (input.clone(), head.clone());
        use_effect_with(placed.is_some() && filtering, move |&focus| {
            // Not on a sheet: the keyboard would cover it.
            let sheet = head
                .cast::<HtmlElement>()
                .is_some_and(|h| h.offset_height() > 0);
            if focus
                && !sheet
                && let Some(input) = input.cast::<web_sys::HtmlInputElement>()
            {
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                _ = input.focus_with_options(&options);
                let end = input.value().encode_utf16().count() as u32;
                _ = input.set_selection_range(end, end);
            }
        });
    }
    {
        let open = open.clone();
        use_effect_with(props.disabled, move |&disabled| {
            if disabled {
                open.set(false);
            }
        });
    }
    {
        let (state, base, is_open) = (state.clone(), (*id).clone(), *open);
        use_effect(move || {
            let mut state = state.borrow_mut();
            if is_open
                && state.take_scroll()
                && let Some(active) = state.active()
            {
                scroll_to(&option_id(&base, active));
            }
        });
    }

    let options = &props.options;
    let multiple = props.multiple;
    let checked = picked(options, multiple, props.value.as_ref(), &props.values);
    // The one on the face, and where the list opens.
    let current = checked.first().copied();
    let many = summarize(
        current.map_or("", |i| options[i].label.as_str()),
        checked.len(),
        props.summary.as_ref(),
    );
    let shown: Vec<usize> = (0..options.len())
        .filter(|&i| !filtering || filter_matches(&options[i], &filter))
        .collect();
    let pickable: Vec<usize> = shown
        .iter()
        .copied()
        .filter(|&i| !options[i].disabled)
        .collect();
    if *open && filtering {
        // What the filter leaves: the active option one of them.
        state.borrow_mut().settle(&pickable);
    }
    let labels: Vec<&str> = options.iter().map(|o| o.label.as_str()).collect();

    let show = {
        let (open, state, filter) = (open.clone(), state.clone(), filter.clone());
        // Opening clears the filter: every enabled option.
        let pickable: Vec<usize> = (0..options.len())
            .filter(|&i| !options[i].disabled)
            .collect();
        let disabled = props.disabled;
        move |last: bool| {
            if disabled {
                return;
            }
            filter.set(String::new());
            let mut state = state.borrow_mut();
            match current.filter(|i| pickable.contains(i)) {
                Some(i) => state.set(Some(i)),
                None => state.go_to_end(&pickable, last),
            }
            open.set(true);
        }
    };
    let pick = {
        let (open, on_change) = (open.clone(), props.on_change.clone());
        let on_change_values = props.on_change_values.clone();
        let values: Vec<(T, bool)> = options
            .iter()
            .map(|o| (o.value.clone(), o.disabled))
            .collect();
        let checked = checked.clone();
        move |i: usize| {
            let Some((value, false)) = values.get(i) else {
                return;
            };
            if multiple {
                // Checked or unchecked, the list kept open.
                let on = !checked.contains(&i);
                let next = (0..values.len())
                    .filter(|&j| if j == i { on } else { checked.contains(&j) })
                    .map(|j| values[j].0.clone())
                    .collect();
                on_change_values.emit(next);
                return;
            }
            open.set(false);
            if current != Some(i) {
                on_change.emit(value.clone());
            }
        }
    };

    let onclick = {
        let (open, show, swallow) = (open.clone(), show.clone(), swallow.clone());
        Callback::from(move |_: MouseEvent| {
            if std::mem::take(&mut *swallow.borrow_mut()) {
                return;
            }
            if *open {
                open.set(false);
            } else {
                show(false);
            }
        })
    };
    let onmousedown = {
        let swallow = swallow.clone();
        Callback::from(move |_: MouseEvent| *swallow.borrow_mut() = false)
    };
    let onkeydown = {
        let (is_open, state, redraw) = (*open, state.clone(), redraw.clone());
        let (open, pick, show) = (open.clone(), pick.clone(), show.clone());
        let (filter, pickable) = (filter.clone(), pickable.clone());
        let labels: Vec<String> = labels.iter().map(|&l| l.to_owned()).collect();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            *swallow.borrow_mut() = matches!(key.as_str(), "Enter" | " ");
            let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
            if !is_open {
                if matches!(
                    key.as_str(),
                    "ArrowDown" | "ArrowUp" | "Enter" | " " | "Home" | "End"
                ) {
                    e.prevent_default();
                    show(matches!(key.as_str(), "ArrowUp" | "End"));
                } else if let Some(c) = printable(&e) {
                    e.prevent_default();
                    show(false);
                    if filtering {
                        // Into the filter, which takes focus once placed.
                        filter.set(c.to_string());
                    } else {
                        state
                            .borrow_mut()
                            .typeahead(&pickable, &labels, c, e.time_stamp());
                    }
                }
                return;
            }
            if key == "Tab" {
                open.set(false);
                return;
            }
            let taken = state
                .borrow_mut()
                .key(&key, e.alt_key(), &pickable, e.time_stamp());
            match taken {
                ListKey::Moved => {}
                ListKey::Pick(i) => pick(i),
                ListKey::Ignored => match printable(&e) {
                    Some(c) => state
                        .borrow_mut()
                        .typeahead(&pickable, &labels, c, e.time_stamp()),
                    None => return,
                },
            }
            e.prevent_default();
            redraw.force_update();
        })
    };

    let active = state.borrow().active();
    let list_id = format!("{}-list", *id);

    let text = |(i, option): (usize, &SelectOption<T>)| {
        let shown = current == Some(i) && many.is_none();
        html! {
            <span class={classes!(class::SELECT_TEXT, (!shown).then_some(class::HIDDEN))}
                aria-hidden={(!shown).then_some("true")}>
                { option.label.clone() }
            </span>
        }
    };
    let placeholder = props.placeholder.clone().map(|p| {
        let shown = current.is_none();
        html! {
            <span class={classes!(class::SELECT_PLACEHOLDER, (!shown).then_some(class::HIDDEN))}
                aria-hidden={(!shown).then_some("true")}>
                { p }
            </span>
        }
    });

    let summary = many.clone().map(|text| {
        html! { <span class={class::SELECT_TEXT}>{ text }</span> }
    });

    let popup = (*open).then(|| {
        let option = |(i, o): (usize, &SelectOption<T>)| {
            let selected = checked.contains(&i);
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
                    aria-disabled={o.disabled.then_some("true")} {onclick} {onmousemove}>
                    <span class={classes!(class::LISTBOX_OPTION_MARK, (!selected).then_some(class::OFF))}
                        aria-hidden="true">
                        <Icon glyph={Glyph::Check} />
                    </span>
                    if let Some(icon) = &o.icon {
                        <span class={class::LISTBOX_OPTION_ICON} aria-hidden="true">{ icon.clone() }</span>
                    }
                    if let Some(description) = &o.description {
                        <span class={class::LISTBOX_OPTION_TEXT}>
                            <span class={class::LISTBOX_OPTION_LABEL}>{ o.label.clone() }</span>
                            <span class={class::LISTBOX_OPTION_DESCRIPTION}>{ description.clone() }</span>
                        </span>
                    } else {
                        <span class={class::LISTBOX_OPTION_LABEL}>{ o.label.clone() }</span>
                    }
                </div>
            }
        };
        let rows = group_runs(options).into_iter().enumerate().map(|(n, (group, run))| {
            let run: Vec<Html> = run
                .into_iter()
                .filter(|i| shown.contains(i))
                .map(|i| option((i, &options[i])))
                .collect();
            match group {
                _ if run.is_empty() => Html::default(),
                None => html! { <>{ for run }</> },
                Some(group) => {
                    let heading = format!("{}-group-{n}", *id);
                    html! {
                        <div class={class::LISTBOX_GROUP} role="group" aria-labelledby={heading.clone()}>
                            <div id={heading} class={class::LISTBOX_GROUP_NAME} role="presentation">
                                { group.clone() }
                            </div>
                            { for run }
                        </div>
                    }
                }
            }
        });
        // Focus stays on the face, or in the filter field.
        let onmousedown = Callback::from(|e: MouseEvent| {
            let in_field = e
                .target()
                .is_some_and(|t| t.dyn_into::<web_sys::HtmlInputElement>().is_ok());
            if !in_field {
                e.prevent_default();
            }
        });
        let anchor_width = face.cast::<HtmlElement>().map(|f| f.offset_width());
        let style = placed.map(|p| {
            let mut style = format!(
                "--popover-left: {}px; --popover-top: {}px; --popover-max-height: {}px;",
                p.left, p.top, p.max_height
            );
            if let Some(w) = anchor_width {
                style.push_str(&format!(" --listbox-anchor-width: {w}px;"));
            }
            if let Some(w) = held {
                style.push_str(&format!(" --listbox-open-width: {w}px;"));
            }
            style
        });
        let testid = props
            .testid
            .as_ref()
            .map(|t| AttrValue::from(format!("{t}-options")));
        if !filtering && !props.sheet {
            return overlays.portal(html! {
                <div ref={list.clone()} id={list_id.clone()} role="listbox"
                    class={listbox(placed.is_some())} aria-label={props.label.clone()}
                    aria-multiselectable={multiple.then_some("true")} {style} {onmousedown} data-testid={testid}>
                    { for rows }
                </div>
            });
        }
        let oninput = {
            let (filter, redraw) = (filter.clone(), redraw.clone());
            Callback::from(move |e: InputEvent| {
                filter.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
                redraw.force_update();
            })
        };
        let onkeydown = {
            let (state, redraw, pick) = (state.clone(), redraw.clone(), pick.clone());
            let (open, face, pickable) = (open.clone(), face.clone(), pickable.clone());
            Callback::from(move |e: KeyboardEvent| {
                let key = e.key();
                if key == "Tab" {
                    // From the face, so Tab goes on as from a closed field.
                    if let Some(face) = face.cast::<HtmlElement>() {
                        _ = face.focus();
                    }
                    open.set(false);
                    return;
                }
                // Home, End and Space are the text's.
                if matches!(key.as_str(), "Home" | "End" | " ") {
                    return;
                }
                match state
                    .borrow_mut()
                    .key(&key, e.alt_key(), &pickable, e.time_stamp())
                {
                    ListKey::Ignored => return,
                    ListKey::Moved => {}
                    ListKey::Pick(i) => pick(i),
                }
                e.prevent_default();
                redraw.force_update();
            })
        };
        let empty = filtering && pickable.is_empty() && shown.is_empty();
        let options = html! {
            <div id={list_id.clone()} role="listbox" class={class::SELECT_OPTIONS}
                aria-label={props.label.clone()}
                aria-multiselectable={multiple.then_some("true")}>
                { for rows }
            </div>
        };
        let filter_field = filtering.then(|| html! {
                <div class={class::SELECT_FILTER}>
                    <span class={input_box(InputSize::Regular, false, false, false)}>
                        <span class={class::INPUT_MARK}><Icon glyph={Glyph::Search} /></span>
                        <input ref={input.clone()} class={class::INPUT_CONTROL} type="text"
                            value={(*filter).clone()} placeholder={props.filter_label.clone()}
                            aria-label={props.filter_label.clone()} role="combobox"
                            aria-autocomplete="list" aria-expanded="true"
                            aria-controls={list_id.clone()}
                            aria-activedescendant={active.map(|i| option_id(&id, i))}
                            autocomplete="off" spellcheck="false"
                            data-testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-filter")))}
                            {oninput} {onkeydown} />
                    </span>
                </div>
        });
        let empty = empty.then(|| html! {
            <p class={class::SELECT_EMPTY}>{ props.no_match.clone() }</p>
        });
        if !props.sheet {
            return overlays.portal(html! {
                <div ref={list.clone()} class={classes!(listbox(placed.is_some()), class::SELECT_POPUP)}
                    {style} {onmousedown} data-testid={testid}>
                    { for filter_field }
                    { options }
                    { for empty }
                </div>
            });
        }
        let close = {
            let (open, face) = (open.clone(), face.clone());
            Callback::from(move |_: MouseEvent| {
                if let Some(face) = face.cast::<HtmlElement>() {
                    _ = face.focus();
                }
                open.set(false);
            })
        };
        overlays.portal(html! {
            <>
                <div class={class::SELECT_SCRIM} aria-hidden="true"></div>
                <div ref={list.clone()}
                    class={classes!(
                        listbox(placed.is_some()),
                        filtering.then_some(class::SELECT_POPUP),
                        class::SELECT_SHEET
                    )}
                    {style} {onmousedown} data-testid={testid}>
                    <div ref={head.clone()} class={class::SELECT_HEAD}>
                        <span class={class::SELECT_TITLE}>{ props.label.clone() }</span>
                        <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                            icon={html! { <Icon glyph={Glyph::Close} /> }}
                            label={props.close_label.clone()} onclick={close} />
                    </div>
                    <div class={class::SELECT_BODY}>
                        { for filter_field }
                        { options }
                        { for empty }
                    </div>
                </div>
            </>
        })
    });

    html! {
        <>
            <button ref={face} type="button" id={props.id.clone()}
                class={classes!(
                    select(props.size, props.width, props.invalid, props.disabled, props.busy),
                    props.class.clone()
                )}
                role="combobox" aria-haspopup="listbox" aria-expanded={open.to_string()}
                aria-controls={open.then(|| list_id.clone())}
                aria-activedescendant={(*open && !filtering).then(|| active.map(|i| option_id(&id, i))).flatten()}
                aria-label={face_label(
                    props.label.as_ref(),
                    many.as_deref().or(current.map(|i| options[i].label.as_str())),
                )}
                aria-describedby={props.describedby.clone()}
                aria-invalid={props.invalid.then_some("true")}
                aria-busy={props.busy.then_some("true")}
                disabled={props.disabled} data-testid={props.testid.clone()}
                {onclick} {onmousedown} {onkeydown}>
                <span class={class::SELECT_VALUE}>
                    { for options.iter().enumerate().map(text) }
                    { for summary }
                    { for placeholder }
                </span>
                <span class={class::SELECT_CHEVRON} aria-hidden="true">
                    <Icon glyph={Glyph::ChevronDown} />
                </span>
            </button>
            { popup.unwrap_or_default() }
        </>
    }
}

pub(crate) fn scroll_to(id: &str) {
    let Some(element) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    else {
        return;
    };
    let options = web_sys::ScrollIntoViewOptions::new();
    options.set_block(web_sys::ScrollLogicalPosition::Nearest);
    element.scroll_into_view_with_scroll_into_view_options(&options);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn render(props: SelectProps<u32>) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Select<u32>>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn sizes() -> Vec<SelectOption<u32>> {
        vec![
            SelectOption::new(1, "S"),
            SelectOption::new(2, "Medium").disabled(true),
            SelectOption::new(3, "Extra large"),
        ]
    }

    #[test]
    fn fill_and_the_input_box_states_are_modifiers() {
        let widest = |size| names(select(size, SelectWidth::Widest, false, false, false));
        assert_eq!(
            widest(InputSize::Regular),
            [class::INPUT_BOX, class::SELECT]
        );
        assert_eq!(
            widest(InputSize::Small),
            [class::INPUT_BOX, class::SMALL, class::SELECT]
        );
        assert_eq!(
            names(select(
                InputSize::Regular,
                SelectWidth::Fill,
                true,
                true,
                true
            )),
            [
                class::INPUT_BOX,
                class::INVALID,
                class::DISABLED,
                class::BUSY,
                class::SELECT,
                class::FILL
            ]
        );
    }

    #[test]
    fn the_face_holds_every_label_and_shows_the_chosen_one() {
        let html = render(yew::props!(SelectProps<u32> {
            options: sizes(),
            value: Some(3),
            placeholder: Some(AttrValue::Static("Pick a size")),
        }));
        for label in ["S", "Medium", "Extra large", "Pick a size"] {
            assert!(
                html.contains(&format!("{label}</span>")),
                "{label} in {html}"
            );
        }
        // Every label but the chosen one is hidden, in the same cell.
        assert_eq!(html.matches(class::HIDDEN).count(), 3, "{html}");
        let shown = format!("<span class=\"{}\">Extra large", class::SELECT_TEXT);
        assert!(html.contains(&shown), "{html}");
        assert!(html.contains("role=\"combobox\""), "{html}");
        assert!(html.contains("aria-expanded=\"false\""), "{html}");
        assert!(!html.contains("role=\"listbox\""), "{html}");
    }

    #[test]
    fn nothing_chosen_shows_the_placeholder() {
        let html = render(yew::props!(SelectProps<u32> {
            options: sizes(),
            value: Some(9),
            placeholder: Some(AttrValue::Static("Pick a size")),
        }));
        let shown = format!("<span class=\"{}\">Pick a size", class::SELECT_PLACEHOLDER);
        assert!(html.contains(&shown), "{html}");
        assert_eq!(html.matches(class::HIDDEN).count(), 3, "{html}");
    }

    #[test]
    fn groups_are_runs_and_ungrouped_options_stand_alone() {
        let options = vec![
            SelectOption::new(1, "A"),
            SelectOption::new(2, "B").group("X"),
            SelectOption::new(3, "C").group("X"),
            SelectOption::new(4, "D"),
            SelectOption::new(5, "E").group("Y"),
        ];
        let runs: Vec<(Option<&str>, Vec<usize>)> = group_runs(&options)
            .into_iter()
            .map(|(g, run)| (g.map(|g| g.as_str()), run))
            .collect();
        assert_eq!(
            runs,
            [
                (None, vec![0]),
                (Some("X"), vec![1, 2]),
                (None, vec![3]),
                (Some("Y"), vec![4])
            ]
        );
        assert!(group_runs::<u32>(&[]).is_empty());
    }

    #[test]
    fn the_filter_matches_a_label_or_its_group_ignoring_case() {
        let option = SelectOption::new(1, "Termagants").group("Battleline");
        assert!(filter_matches(&option, ""));
        assert!(filter_matches(&option, "MAGA"));
        assert!(filter_matches(&option, "battle"));
        assert!(!filter_matches(&option, "carnifex"));
    }

    #[test]
    fn several_picked_are_summarized_on_the_face() {
        let options = sizes();
        assert_eq!(picked(&options, true, None, &[3, 1]), [0, 2]);
        assert_eq!(picked(&options, false, Some(&3), &[1]), [2]);
        assert_eq!(summarize("S", 1, None), None);
        assert_eq!(summarize("S", 3, None).as_deref(), Some("S +2"));
        let summary = AttrValue::Static("12 units");
        assert_eq!(summarize("S", 2, Some(&summary)), Some(summary));

        let html = render(yew::props!(SelectProps<u32> {
            options: sizes(),
            multiple: true,
            values: vec![1, 3],
        }));
        let shown = format!("<span class=\"{}\">S +1", class::SELECT_TEXT);
        assert!(html.contains(&shown), "{html}");
        // Every label hidden, the summary in their cell.
        assert_eq!(html.matches(class::HIDDEN).count(), 3, "{html}");
    }

    #[test]
    fn a_sheet_leaves_the_closed_face_as_it_was() {
        let face = |sheet| {
            render(yew::props!(SelectProps<u32> {
                options: sizes(),
                value: Some(1),
                label: Some(AttrValue::Static("Size")),
                sheet,
            }))
        };
        assert_eq!(face(true), face(false));
        assert!(!face(true).contains(class::SELECT_SHEET));
    }

    #[test]
    fn a_value_round_trips_through_its_option() {
        let options = sizes();
        for option in &options {
            let at = chosen(&options, Some(&option.value)).unwrap();
            assert_eq!(options[at].value, option.value);
        }
        assert_eq!(chosen(&options, Some(&9)), None);
        assert_eq!(chosen(&options, None), None);
    }
}
