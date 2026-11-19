//! SearchSelect: a text field filtering a [`Listbox`] of options under it
//! (`style/_search-select.scss`).

use phrasebook02::yew023::use_phrases_with;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::listbox::{ListKey, Listbox, listbox, listbox_option, option_id};
use crate::phrasebookcatalog01::{L10N, label};
use crate::select::scroll_to;
use crate::{Dismiss, Glyph, Icon, InputSize, PopoverAlign, Side, class, input_box, use_overlays};

/// One option of a [`SearchSelect`].
#[derive(Clone, Debug, PartialEq)]
pub struct SearchOption<T> {
    pub value: T,
    pub label: AttrValue,
    /// Muted after the label, and searched too.
    pub detail: Option<AttrValue>,
    pub icon: Option<Html>,
    /// The heading it is listed under; options of a group are adjacent.
    pub group: Option<AttrValue>,
    pub disabled: bool,
}

impl<T> SearchOption<T> {
    pub fn new(value: T, label: impl Into<AttrValue>) -> Self {
        Self {
            value,
            label: label.into(),
            detail: None,
            icon: None,
            group: None,
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

    pub fn group(self, group: impl Into<AttrValue>) -> Self {
        Self {
            group: Some(group.into()),
            ..self
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }
}

/// The indices of the options whose label or detail contains `query`,
/// ignoring case and surrounding space; all of them for a blank one.
pub fn search_matches<T>(options: &[SearchOption<T>], query: &str) -> Vec<usize> {
    let needle = query.trim().to_lowercase();
    let has = |text: &str| text.to_lowercase().contains(&needle);
    (0..options.len())
        .filter(|&i| {
            let o = &options[i];
            needle.is_empty() || has(&o.label) || o.detail.as_deref().is_some_and(has)
        })
        .collect()
}

/// What an open [`SearchSelect`] lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Listed {
    /// Options for an earlier query, or none yet: a loading row.
    Loading,
    /// These options, by index.
    Options(Vec<usize>),
}

/// What to list for `query`: the options answer the query `answers` (or
/// any, when `None`), so results arriving for an older query than the one
/// typed are ignored. With `filter`, they are filtered here.
pub fn listed<T>(
    options: &[SearchOption<T>],
    query: &str,
    answers: Option<&str>,
    loading: bool,
    filter: bool,
) -> Listed {
    let stale = answers.is_some_and(|a| a.trim() != query.trim());
    if stale || (loading && options.is_empty()) {
        return Listed::Loading;
    }
    Listed::Options(if filter {
        search_matches(options, query)
    } else {
        (0..options.len()).collect()
    })
}

/// The field's classes: the input box's, `search-select` and `fill`.
pub fn search_select(
    size: InputSize,
    fill: bool,
    invalid: bool,
    disabled: bool,
    busy: bool,
) -> Classes {
    classes!(
        input_box(size, invalid, disabled, busy),
        class::SEARCH_SELECT,
        fill.then_some(class::FILL)
    )
}

#[derive(Properties, PartialEq)]
pub struct SearchSelectProps<T: PartialEq + 'static> {
    pub options: Vec<SearchOption<T>>,
    /// What is chosen, shown in the field while it is not being edited.
    #[prop_or_default]
    pub value: Option<T>,
    #[prop_or_default]
    pub on_change: Callback<T>,
    /// Each edit of the query, for options loaded elsewhere.
    #[prop_or_default]
    pub on_search: Callback<String>,
    /// Filter `options` by the query here; off when `on_search` loads
    /// them already filtered.
    #[prop_or(true)]
    pub filter: bool,
    /// The query the options answer: until it is the one typed, they are
    /// stale and a loading row stands in their place.
    #[prop_or_default]
    pub answers: Option<AttrValue>,
    /// Options are loading (`aria-busy`): a spinner in the field, and a
    /// loading row while there are none.
    #[prop_or_default]
    pub busy: bool,
    /// The placeholder and the accessible name.
    pub label: AttrValue,
    /// The loading row's words (the app's phrasebook).
    #[prop_or_default]
    pub loading_label: Option<AttrValue>,
    /// Said when nothing matches.
    #[prop_or_default]
    pub empty_label: Option<AttrValue>,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    /// As wide as its container (`.sockel-fill`).
    #[prop_or_default]
    pub fill: bool,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub autofocus: bool,
    #[prop_or_default]
    pub class: Classes,
    /// The input's test id; the list's is it suffixed `-options`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// The WAI-ARIA editable combobox with list autocomplete: typing filters
/// the options in a listbox under the field, which keeps the focus. The
/// arrows, PageUp and PageDown move, Enter picks, Escape and Tab close.
/// The list is one width (`--menu-width-wide`, or the field's if wider)
/// and `--picker-rows` rows high whatever it lists, loading included.
#[function_component]
pub fn SearchSelect<T>(props: &SearchSelectProps<T>) -> Html
where
    T: PartialEq + Clone + 'static,
{
    let phrases = use_phrases_with(&L10N);
    let id = use_state(|| crate::tooltip::next_id("search-select"));
    let overlays = use_overlays();
    let field = use_node_ref();
    let list = use_node_ref();
    let open = use_state_eq(|| false);
    // The text typed since the last pick; `None` shows what is chosen.
    let query = use_state_eq(|| None::<String>);
    let state = use_mut_ref(Listbox::default);
    let redraw = use_force_update();

    let on_dismiss = {
        let (open, query) = (open.clone(), query.clone());
        Callback::from(move |_: Dismiss| {
            open.set(false);
            query.set(None);
        })
    };
    let (placed, escape) = crate::popover::use_popover_keyed(
        *open,
        field.clone(),
        list.clone(),
        Side::Below,
        PopoverAlign::Start,
        None,
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

    let options = &props.options;
    let current = props
        .value
        .as_ref()
        .and_then(|v| options.iter().position(|o| &o.value == v));
    let typed = (*query).clone().unwrap_or_default();
    let shown = listed(
        options,
        &typed,
        props.answers.as_deref(),
        props.busy,
        props.filter,
    );
    let rows = match &shown {
        Listed::Options(rows) => rows.clone(),
        Listed::Loading => Vec::new(),
    };
    let pickable: Vec<usize> = rows
        .iter()
        .copied()
        .filter(|&i| !options[i].disabled)
        .collect();
    if *open {
        state.borrow_mut().settle(&pickable);
    }
    let active = (*open).then(|| state.borrow().active()).flatten();

    let pick = {
        let (open, query, on_change) = (open.clone(), query.clone(), props.on_change.clone());
        let values: Vec<(T, bool)> = options
            .iter()
            .map(|o| (o.value.clone(), o.disabled))
            .collect();
        move |i: usize| {
            let Some((value, false)) = values.get(i) else {
                return;
            };
            open.set(false);
            query.set(None);
            if current != Some(i) {
                on_change.emit(value.clone());
            }
        }
    };

    let oninput = {
        let (open, query, state) = (open.clone(), query.clone(), state.clone());
        let on_search = props.on_search.clone();
        Callback::from(move |e: InputEvent| {
            let text = e.target_unchecked_into::<HtmlInputElement>().value();
            state.borrow_mut().set(None);
            on_search.emit(text.clone());
            query.set(Some(text));
            open.set(true);
        })
    };
    let onkeydown = {
        let (is_open, state, redraw) = (*open, state.clone(), redraw.clone());
        let (open, pick, pickable) = (open.clone(), pick.clone(), pickable.clone());
        let on_dismiss = on_dismiss.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            if !is_open {
                if matches!(key.as_str(), "ArrowDown" | "ArrowUp") {
                    e.prevent_default();
                    state.borrow_mut().go_to_end(&pickable, key == "ArrowUp");
                    open.set(true);
                }
                return;
            }
            match key.as_str() {
                "Tab" => on_dismiss.emit(Dismiss::Outside),
                // Home, End and Space edit the text.
                "ArrowDown" | "ArrowUp" | "PageDown" | "PageUp" | "Enter" => {
                    e.prevent_default();
                    if let ListKey::Pick(i) =
                        state
                            .borrow_mut()
                            .key(&key, e.alt_key(), &pickable, e.time_stamp())
                    {
                        pick(i);
                    }
                    redraw.force_update();
                }
                _ => {}
            }
        })
    };
    let onclick = {
        let (open, disabled) = (open.clone(), props.disabled);
        Callback::from(move |_: MouseEvent| {
            if !disabled {
                open.set(true);
            }
        })
    };

    let list_id = format!("{}-list", *id);
    let text = match (&*query, current) {
        (Some(typed), _) => AttrValue::from(typed.clone()),
        (None, Some(i)) => options[i].label.clone(),
        (None, None) => AttrValue::default(),
    };
    let glyph = if props.busy {
        Glyph::Spinner
    } else {
        Glyph::Search
    };

    let popup = (*open).then(|| {
        let option = |i: usize| {
            let o = &options[i];
            let selected = current == Some(i);
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
                    <span class={class::LISTBOX_OPTION_LABEL}>{ o.label.clone() }</span>
                    if let Some(detail) = &o.detail {
                        <span class={class::SEARCH_SELECT_DETAIL}>{ detail.clone() }</span>
                    }
                </div>
            }
        };
        let body = match &shown {
            Listed::Loading => html! {
                <div class={class::SEARCH_SELECT_NOTE} role="status">
                    <Icon glyph={Glyph::Spinner} />
                    { label(&phrases, &props.loading_label, "sockel-search-select-loading") }
                </div>
            },
            Listed::Options(rows) if rows.is_empty() => html! {
                <div class={class::SEARCH_SELECT_NOTE} role="status">{ label(&phrases, &props.empty_label, "sockel-search-select-empty") }</div>
            },
            Listed::Options(rows) => html! {
                { for groups(options, rows).into_iter().enumerate().map(|(g, (label, members))| {
                    let members = html! { { for members.into_iter().map(option) } };
                    match label {
                        None => members,
                        Some(label) => {
                            let heading = format!("{}-group-{g}", *id);
                            html! {
                                <div role="group" class={class::SEARCH_SELECT_GROUP}
                                    aria-labelledby={heading.clone()}>
                                    <div id={heading} role="presentation"
                                        class={class::SEARCH_SELECT_HEADING}>{ label }</div>
                                    { members }
                                </div>
                            }
                        }
                    }
                }) }
            },
        };
        let onmousedown = Callback::from(|e: MouseEvent| e.prevent_default());
        let anchor_width = field
            .cast::<web_sys::HtmlElement>()
            .map(|f| f.offset_width());
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
            <div ref={list.clone()} id={list_id.clone()} role="listbox"
                class={classes!(listbox(placed.is_some()), class::SEARCH_SELECT_LIST)}
                aria-label={props.label.clone()} aria-busy={(shown == Listed::Loading).then_some("true")}
                {style} {onmousedown}
                data-testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-options")))}>
                { body }
            </div>
        })
    });

    html! {
        <>
            <span ref={field} class={classes!(
                search_select(props.size, props.fill, props.invalid, props.disabled, props.busy),
                props.class.clone()
            )} aria-busy={props.busy.then_some("true")}>
                <span class={class::INPUT_MARK}><Icon {glyph} /></span>
                <input class={class::INPUT_CONTROL} type="text" id={props.id.clone()}
                    value={text} placeholder={props.label.clone()}
                    role="combobox" aria-autocomplete="list" aria-expanded={open.to_string()}
                    aria-controls={open.then(|| list_id.clone())}
                    aria-activedescendant={active.map(|i| option_id(&id, i))}
                    aria-label={props.label.clone()} aria-describedby={props.describedby.clone()}
                    aria-invalid={props.invalid.then_some("true")}
                    autocomplete="off" spellcheck="false" disabled={props.disabled}
                    data-autofocus={props.autofocus.then_some("")}
                    data-testid={props.testid.clone()}
                    {oninput} onkeydown={escape.after(onkeydown)} {onclick} />
            </span>
            { popup.unwrap_or_default() }
        </>
    }
}

/// `rows` split into runs of one group, in order.
fn groups<T>(options: &[SearchOption<T>], rows: &[usize]) -> Vec<(Option<AttrValue>, Vec<usize>)> {
    let mut runs: Vec<(Option<AttrValue>, Vec<usize>)> = Vec::new();
    for &i in rows {
        let group = &options[i].group;
        match runs.last_mut() {
            Some((g, members)) if g == group => members.push(i),
            _ => runs.push((group.clone(), vec![i])),
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn fruit() -> Vec<SearchOption<u32>> {
        vec![
            SearchOption::new(0, "Apple").group("Pome"),
            SearchOption::new(1, "Pear")
                .group("Pome")
                .detail("Conference"),
            SearchOption::new(2, "Cherry").group("Stone"),
            SearchOption::new(3, "Plum").group("Stone").disabled(true),
        ]
    }

    #[test]
    fn fill_and_the_input_box_states_are_modifiers() {
        assert_eq!(
            names(search_select(
                InputSize::Regular,
                false,
                false,
                false,
                false
            )),
            [class::INPUT_BOX, class::SEARCH_SELECT]
        );
        assert_eq!(
            names(search_select(InputSize::Small, true, true, true, true)),
            [
                class::INPUT_BOX,
                class::SMALL,
                class::INVALID,
                class::DISABLED,
                class::BUSY,
                class::SEARCH_SELECT,
                class::FILL
            ]
        );
    }

    #[test]
    fn filters_by_label_and_detail_ignoring_case() {
        let options = fruit();
        assert_eq!(search_matches(&options, ""), [0, 1, 2, 3]);
        assert_eq!(search_matches(&options, "  P "), [0, 1, 3]);
        assert_eq!(search_matches(&options, "CONF"), [1]);
        assert_eq!(search_matches(&options, "kiwi"), Vec::<usize>::new());
    }

    #[test]
    fn stale_results_are_ignored_until_the_typed_query_is_answered() {
        let options = fruit();
        // Results for "ap" arrive after "che" was typed.
        assert_eq!(
            listed(&options, "che", Some("ap"), true, false),
            Listed::Loading
        );
        assert_eq!(
            listed(&options, "che", Some("che"), false, false),
            Listed::Options(vec![0, 1, 2, 3])
        );
        assert_eq!(
            listed(&options, "che", None, false, true),
            Listed::Options(vec![2])
        );
        assert_eq!(listed::<u32>(&[], "", None, true, true), Listed::Loading);
        assert_eq!(
            listed::<u32>(&[], "", None, false, true),
            Listed::Options(vec![])
        );
    }

    #[test]
    fn groups_are_runs_of_adjacent_options() {
        let options = fruit();
        let runs = groups(&options, &[0, 1, 2]);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].0.as_deref(), Some("Pome"));
        assert_eq!(runs[0].1, [0, 1]);
        assert_eq!(runs[1].1, [2]);
    }

    #[test]
    fn renders_a_closed_combobox_showing_the_chosen_label() {
        let html = futures::executor::block_on(
            yew::LocalServerRenderer::<SearchSelect<u32>>::with_props(yew::props!(
                SearchSelectProps<u32> {
                    options: fruit(),
                    value: Some(2),
                    label: AttrValue::Static("Fruit"),
                }
            ))
            .hydratable(false)
            .render(),
        );
        assert!(html.contains("role=\"combobox\""), "{html}");
        assert!(html.contains("aria-expanded=\"false\""), "{html}");
        assert!(html.contains("value=\"Cherry\""), "{html}");
        assert!(!html.contains("role=\"listbox\""), "{html}");
    }
}
