//! FilterPicker: a list's filter button of one width, opening a listbox of
//! choices to check (`style/_filter-picker.scss`).

use web_sys::HtmlElement;
use yew::prelude::*;

use crate::listbox::{ListKey, Listbox, listbox, listbox_option, option_id};
use crate::picker::{ChoiceList, ChoicePopup, PickerOption, SEARCH_FROM};
use crate::roving::printable;
use crate::select::scroll_to;
use crate::{Dismiss, Glyph, Icon, PopoverAlign, Side, class, use_overlays, use_popover};

/// One choice of a [`FilterPicker`].
#[derive(Clone, Debug, PartialEq)]
pub struct FilterChoice<T> {
    pub value: T,
    pub label: AttrValue,
}

impl<T> FilterChoice<T> {
    pub fn new(value: T, label: impl Into<AttrValue>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

/// What the button says for `chosen` (labels, in the order chosen): `all`
/// for none, else the first, and how many more there are.
pub fn filter_summary(all: &str, chosen: &[&str]) -> (String, usize) {
    match chosen.split_first() {
        None => (all.to_owned(), 0),
        Some((first, rest)) => ((*first).to_owned(), rest.len()),
    }
}

/// `chosen` with `value` taken out if it is there, put at the end if not.
pub fn toggled<T: PartialEq + Clone>(chosen: &[T], value: &T) -> Vec<T> {
    if chosen.contains(value) {
        chosen.iter().filter(|v| *v != value).cloned().collect()
    } else {
        chosen.iter().cloned().chain([value.clone()]).collect()
    }
}

/// The button's classes: a [`crate::Button`]'s, `filter-picker`, and `on`
/// while anything is chosen.
pub fn filter_picker(on: bool) -> Classes {
    classes!(class::BUTTON, class::FILTER_PICKER, on.then_some(class::ON))
}

#[derive(Properties, PartialEq)]
pub struct FilterPickerProps<T: PartialEq + 'static> {
    pub choices: Vec<FilterChoice<T>>,
    /// What is chosen, in the order it was; none is all.
    #[prop_or_default]
    pub chosen: Vec<T>,
    /// The choices after a pick (see [`toggled`]); empty for all.
    #[prop_or_default]
    pub on_change: Callback<Vec<T>>,
    /// The button's accessible name, before what it says: "Genre".
    pub label: AttrValue,
    /// The button's words when nothing is chosen, and the all option's
    /// (the app's phrasebook).
    #[prop_or("All".into())]
    pub all: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    /// The button's test id; the list's is it suffixed `-options`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A button `--filter-width` wide saying what is chosen (the first, and a
/// count of the rest in a reserved slot), so choosing never resizes it.
/// It opens a multi-select listbox: an option for all, then a checkable
/// option per choice; a pick leaves it open. Keys as a [`crate::Select`]'s.
/// Past [`SEARCH_FROM`] choices the pop-up is a searchable [`ChoiceList`]
/// instead, a fixed height whatever matches.
#[function_component]
pub fn FilterPicker<T>(props: &FilterPickerProps<T>) -> Html
where
    T: PartialEq + Clone + 'static,
{
    let id = use_state(|| crate::tooltip::next_id("filter-picker"));
    let overlays = use_overlays();
    let face = use_node_ref();
    let list = use_node_ref();
    let open = use_state_eq(|| false);
    let state = use_mut_ref(Listbox::default);
    let redraw = use_force_update();
    let swallow = use_mut_ref(|| false);

    let on_dismiss = {
        let open = open.clone();
        Callback::from(move |_: Dismiss| open.set(false))
    };
    let searchable = props.choices.len() > SEARCH_FROM;
    let placed = use_popover(
        *open && !searchable,
        face.clone(),
        list.clone(),
        Side::Below,
        PopoverAlign::Start,
        on_dismiss.clone(),
    );
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

    let choices = &props.choices;
    // Option 0 is all; option i + 1 is choice i.
    let pickable: Vec<usize> = (0..=choices.len()).collect();
    let labels: Vec<String> = std::iter::once(props.all.to_string())
        .chain(choices.iter().map(|c| c.label.to_string()))
        .collect();
    let chosen_labels: Vec<&str> = props
        .chosen
        .iter()
        .filter_map(|v| choices.iter().find(|c| &c.value == v))
        .map(|c| c.label.as_str())
        .collect();
    let (text, more) = filter_summary(&props.all, &chosen_labels);
    let is_chosen = |i: usize| match i {
        0 => props.chosen.is_empty(),
        _ => props.chosen.contains(&choices[i - 1].value),
    };

    let pick = {
        let (on_change, chosen) = (props.on_change.clone(), props.chosen.clone());
        let values: Vec<T> = choices.iter().map(|c| c.value.clone()).collect();
        move |i: usize| match i {
            0 if chosen.is_empty() => {}
            0 => on_change.emit(Vec::new()),
            _ => on_change.emit(toggled(&chosen, &values[i - 1])),
        }
    };
    let show = {
        let (open, state, disabled) = (open.clone(), state.clone(), props.disabled);
        let pickable = pickable.clone();
        move |last: bool| {
            if !disabled {
                state.borrow_mut().go_to_end(&pickable, last);
                open.set(true);
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
        let (open, pick) = (open.clone(), pick.clone());
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            *swallow.borrow_mut() = matches!(key.as_str(), "Enter" | " ");
            let names: Vec<&str> = labels.iter().map(String::as_str).collect();
            if !is_open {
                if matches!(
                    key.as_str(),
                    "ArrowDown" | "ArrowUp" | "Enter" | " " | "Home" | "End"
                ) {
                    e.prevent_default();
                    show(matches!(key.as_str(), "ArrowUp" | "End"));
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
                        .typeahead(&pickable, &names, c, e.time_stamp()),
                    None => return,
                },
            }
            e.prevent_default();
            redraw.force_update();
        })
    };

    let active = (*open).then(|| state.borrow().active()).flatten();
    let list_id = format!("{}-list", *id);
    let name = format!("{}: {}", props.label, chosen_labels.join(", "));
    let name = if chosen_labels.is_empty() {
        format!("{}: {}", props.label, props.all)
    } else {
        name
    };

    let popup = (*open && !searchable).then(|| {
        let option = |i: usize| {
            let selected = is_chosen(i);
            let is_active = active == Some(i);
            let onclick = {
                let pick = pick.clone();
                Callback::from(move |_: MouseEvent| pick(i))
            };
            let onmousemove = (!is_active).then(|| {
                let (state, redraw) = (state.clone(), redraw.clone());
                Callback::from(move |_: MouseEvent| {
                    if state.borrow_mut().hover(i) {
                        redraw.force_update();
                    }
                })
            });
            let label = match i {
                0 => props.all.clone(),
                _ => choices[i - 1].label.clone(),
            };
            html! {
                <div id={option_id(&id, i)} role="option" class={listbox_option(is_active)}
                    aria-selected={selected.to_string()} {onclick} {onmousemove}>
                    <span class={classes!(class::LISTBOX_OPTION_MARK, (!selected).then_some(class::OFF))}
                        aria-hidden="true">
                        <Icon glyph={Glyph::Check} />
                    </span>
                    <span class={class::LISTBOX_OPTION_LABEL}>{ label }</span>
                </div>
            }
        };
        let onmousedown = Callback::from(|e: MouseEvent| e.prevent_default());
        let anchor_width = face.cast::<HtmlElement>().map(|f| f.offset_width());
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
            <div ref={list.clone()} id={list_id.clone()} role="listbox" aria-multiselectable="true"
                class={classes!(listbox(placed.is_some()), class::FILTER_PICKER_LIST)}
                aria-label={props.label.clone()} {style} {onmousedown}
                data-testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-options")))}>
                { for (0..=choices.len()).map(option) }
            </div>
        })
    });

    let search = searchable.then(|| {
        let options: Vec<PickerOption<usize>> = std::iter::once(props.all.clone())
            .chain(choices.iter().map(|c| c.label.clone()))
            .enumerate()
            .map(|(i, l)| PickerOption::new(i, l))
            .collect();
        let chosen: Vec<usize> = if props.chosen.is_empty() {
            vec![0]
        } else {
            (1..=choices.len()).filter(|&i| is_chosen(i)).collect()
        };
        let on_pick = Callback::from(move |i: usize| pick(i));
        html! {
            <ChoicePopup anchor={face.clone()} open={*open} on_dismiss={on_dismiss.clone()}
                testid={props.testid.clone()}>
                <ChoiceList<usize> {options} multi=true {chosen} {on_pick}
                    label={props.label.clone()} autofocus=true
                    testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-options")))} />
            </ChoicePopup>
        }
    });

    html! {
        <>
            <button ref={face} type="button"
                class={classes!(filter_picker(!props.chosen.is_empty()), props.class.clone())}
                aria-haspopup="listbox" aria-expanded={open.to_string()}
                aria-controls={open.then(|| list_id.clone())}
                aria-activedescendant={active.map(|i| option_id(&id, i))}
                aria-label={name.clone()} title={name}
                disabled={props.disabled} data-testid={props.testid.clone()}
                {onclick} {onmousedown} {onkeydown}>
                <span class={class::FILTER_PICKER_TEXT}>{ text }</span>
                <span class={classes!(class::FILTER_PICKER_COUNT, (more == 0).then_some(class::OFF))}
                    aria-hidden="true">
                    { format!("+{more}") }
                </span>
                <span class={class::FILTER_PICKER_CHEVRON} aria-hidden="true">
                    <Icon glyph={Glyph::ChevronDown} />
                </span>
            </button>
            { popup.unwrap_or_default() }
            { search.unwrap_or_default() }
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn on_is_a_modifier() {
        assert_eq!(
            names(filter_picker(false)),
            [class::BUTTON, class::FILTER_PICKER]
        );
        assert_eq!(
            names(filter_picker(true)),
            [class::BUTTON, class::FILTER_PICKER, class::ON]
        );
    }

    #[test]
    fn counts_the_choices_after_the_first() {
        assert_eq!(filter_summary("All", &[]), ("All".to_owned(), 0));
        assert_eq!(filter_summary("All", &["Rock"]), ("Rock".to_owned(), 0));
        assert_eq!(
            filter_summary("All", &["Rock", "Jazz", "Folk"]),
            ("Rock".to_owned(), 2)
        );
    }

    #[test]
    fn a_pick_toggles_a_choice_keeping_the_order_chosen() {
        let chosen = toggled(&[3, 1], &2);
        assert_eq!(chosen, [3, 1, 2]);
        assert_eq!(toggled(&chosen, &1), [3, 2]);
        assert_eq!(toggled(&[2], &2), Vec::<i32>::new());
    }

    #[test]
    fn the_count_keeps_its_slot_when_there_is_none() {
        let html = futures::executor::block_on(
            yew::LocalServerRenderer::<FilterPicker<u32>>::with_props(yew::props!(
                FilterPickerProps<u32> {
                    choices: vec![FilterChoice::new(1, "Rock"), FilterChoice::new(2, "Jazz")],
                    label: AttrValue::Static("Genre"),
                }
            ))
            .hydratable(false)
            .render(),
        );
        assert!(html.contains(">All</span>"), "{html}");
        assert!(html.contains(">+0</span>"), "{html}");
        assert!(html.contains("aria-label=\"Genre: All\""), "{html}");
    }
}
