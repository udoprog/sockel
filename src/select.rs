//! Select: a field's box showing what is chosen, opening a [`Listbox`] of
//! its options (`style/_select.scss`).

use web_sys::HtmlElement;
use yew::prelude::*;

use crate::listbox::{ListKey, Listbox, listbox, listbox_option, option_id};
use crate::{Dismiss, Glyph, Icon, InputSize, PopoverAlign, Side, class, input_box, use_overlays};

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
}

impl<T> SelectOption<T> {
    pub fn new(value: T, label: impl Into<AttrValue>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            disabled: false,
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
}

/// A key that types a character, rather than a command.
pub(crate) fn printable(e: &KeyboardEvent) -> Option<char> {
    if e.ctrl_key() || e.meta_key() || e.alt_key() {
        return None;
    }
    let key = e.key();
    let mut chars = key.chars();
    let c = chars.next()?;
    (chars.next().is_none() && c != ' ').then_some(c)
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
    let current = chosen(options, props.value.as_ref());
    let pickable: Vec<usize> = (0..options.len())
        .filter(|&i| !options[i].disabled)
        .collect();
    let labels: Vec<&str> = options.iter().map(|o| o.label.as_str()).collect();

    let show = {
        let (open, state, pickable) = (open.clone(), state.clone(), pickable.clone());
        let disabled = props.disabled;
        move |last: bool| {
            if disabled {
                return;
            }
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
        let values: Vec<(T, bool)> = options
            .iter()
            .map(|o| (o.value.clone(), o.disabled))
            .collect();
        move |i: usize| {
            let Some((value, false)) = values.get(i) else {
                return;
            };
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
                    state
                        .borrow_mut()
                        .typeahead(&pickable, &labels, c, e.time_stamp());
                }
                return;
            }
            if key == "Tab" {
                open.set(false);
                return;
            }
            let taken = state.borrow_mut().key(&key, e.alt_key(), &pickable);
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
        let shown = current == Some(i);
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

    let popup = (*open).then(|| {
        let option = |(i, o): (usize, &SelectOption<T>)| {
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
                </div>
            }
        };
        // Focus stays on the face.
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
            if let Some(w) = held {
                style.push_str(&format!(" --listbox-open-width: {w}px;"));
            }
            style
        });
        overlays.portal(html! {
            <div ref={list.clone()} id={list_id.clone()} role="listbox"
                class={listbox(placed.is_some())} aria-label={props.label.clone()}
                {style} {onmousedown}
                data-testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-options")))}>
                { for options.iter().enumerate().map(option) }
            </div>
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
                aria-activedescendant={open.then(|| active.map(|i| option_id(&id, i))).flatten()}
                aria-label={props.label.clone()} aria-describedby={props.describedby.clone()}
                aria-invalid={props.invalid.then_some("true")}
                aria-busy={props.busy.then_some("true")}
                disabled={props.disabled} data-testid={props.testid.clone()}
                {onclick} {onmousedown} {onkeydown}>
                <span class={class::SELECT_VALUE}>
                    { for options.iter().enumerate().map(text) }
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
