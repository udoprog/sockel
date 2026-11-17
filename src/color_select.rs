//! ColorSelect: a swatch face opening a grid of the app's colours, with an
//! optional custom colour (`style/_color-select.scss`).

use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;

use crate::listbox::{ListKey, Listbox, option_id};
use crate::select::{printable, scroll_to};
use crate::{Dismiss, Glyph, Icon, InputSize, PopoverAlign, Side, class, input_box, overlay};

/// One colour of a palette.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorOption {
    /// Any CSS colour; empty is no colour, drawn struck through.
    pub value: AttrValue,
    pub label: AttrValue,
}

impl ColorOption {
    pub fn new(value: impl Into<AttrValue>, label: impl Into<AttrValue>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// How wide a [`ColorSelect`]'s face is, whatever is chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSelectWidth {
    /// `--color-select-width`, the label cut past it.
    #[default]
    Fixed,
    /// Its container's (`.plinth-fill`).
    Fill,
    /// The swatch alone, a square (`.plinth-swatch-only`).
    Swatch,
}

/// A face's classes: the input box's, `color-select`, and `fill` or
/// `swatch-only`.
pub fn color_select(
    size: InputSize,
    width: ColorSelectWidth,
    invalid: bool,
    disabled: bool,
    busy: bool,
) -> Classes {
    classes!(
        input_box(size, invalid, disabled, busy),
        class::COLOR_SELECT,
        match width {
            ColorSelectWidth::Fixed => None,
            ColorSelectWidth::Fill => Some(class::FILL),
            ColorSelectWidth::Swatch => Some(class::SWATCH_ONLY),
        }
    )
}

/// A swatch's classes: `color-swatch`, and `none` for no colour.
pub fn color_swatch(none: bool) -> Classes {
    classes!(class::COLOR_SWATCH, none.then_some(class::NONE))
}

/// The open popup's classes: `color-list`, and `placed` once placed.
pub fn color_list(placed: bool) -> Classes {
    classes!(class::COLOR_LIST, placed.then_some(class::PLACED))
}

/// A colour's cell: `color-option`, `active` under the keyboard or pointer.
pub fn color_option(active: bool) -> Classes {
    classes!(class::COLOR_OPTION, active.then_some(class::ACTIVE))
}

/// The index of the option holding `value`.
pub fn color_index(options: &[ColorOption], value: Option<&str>) -> Option<usize> {
    let value = value?;
    options.iter().position(|o| o.value == value)
}

/// What `value` is called on the face: its option's label, else the value
/// itself (a custom colour).
pub fn color_name<'a>(options: &'a [ColorOption], value: &'a str) -> &'a str {
    color_index(options, Some(value)).map_or(value, |i| options[i].label.as_str())
}

/// The value a native colour input holds for `value`: itself if it is
/// `#rrggbb`, else black.
pub fn color_input_value(value: &str) -> String {
    let hex = value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|c| c.is_ascii_hexdigit());
    if hex { value } else { "#000000" }.to_owned()
}

/// Read `key` on an open grid of `columns` over `pickable`: the arrows move
/// over it, the rest is [`Listbox::key`]'s.
pub fn color_key(
    list: &mut Listbox,
    key: &str,
    alt: bool,
    columns: usize,
    pickable: &[usize],
) -> ListKey {
    let columns = isize::try_from(columns.max(1)).unwrap_or(1);
    let step = match key {
        "ArrowRight" => 1,
        "ArrowLeft" => -1,
        "ArrowDown" if !alt => columns,
        "ArrowUp" if !alt => -columns,
        _ => return list.key(key, alt, pickable),
    };
    list.step(pickable, step);
    ListKey::Moved
}

#[derive(Properties, PartialEq)]
pub struct ColorSelectProps {
    /// The palette, in the order it is laid out.
    pub options: Vec<ColorOption>,
    /// What is chosen; the placeholder while `None`.
    #[prop_or_default]
    pub value: Option<AttrValue>,
    /// Told a colour picked that is not the one chosen.
    #[prop_or_default]
    pub on_change: Callback<AttrValue>,
    /// Shown, muted, while nothing is chosen.
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    /// The custom colour row's label; without it there is no such row.
    #[prop_or_default]
    pub custom: Option<AttrValue>,
    /// How many colours a row of the grid holds.
    #[prop_or(6)]
    pub columns: usize,
    /// The accessible name without a `Field` (`aria-label`).
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// The face's id, a `Field`'s `id`.
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    #[prop_or_default]
    pub width: ColorSelectWidth,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// Its palette is loading: it opens on what it has (`aria-busy`).
    #[prop_or_default]
    pub busy: bool,
    /// The face's classes.
    #[prop_or_default]
    pub class: Classes,
    /// The face's test id; the grid's is it suffixed `-options`.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

fn swatch(value: &str, none: bool) -> Html {
    let style = (!none).then(|| format!("--swatch: {value};"));
    html! { <span class={color_swatch(none)} aria-hidden="true" {style} /> }
}

/// A colour picker on the select-only combobox pattern: a face (a button,
/// `role="combobox"`) showing the chosen colour's swatch and name, opening
/// a grid of swatches in the overlay layer while focus stays on it. The
/// arrows move over the grid (Left and Right by one, Up and Down by a row),
/// Home, End, PageUp, PageDown and typing move too, Enter, Space and
/// Alt+ArrowUp pick, Escape closes. With `custom`, Tab moves into the
/// native colour input below the grid.
#[function_component]
pub fn ColorSelect(props: &ColorSelectProps) -> Html {
    let id = use_state(|| crate::tooltip::next_id("color-select"));
    let face = use_node_ref();
    let list = use_node_ref();
    let custom_input = use_node_ref();
    let open = use_state_eq(|| false);
    let state = use_mut_ref(Listbox::default);
    let redraw = use_force_update();
    // Enter or Space opened or picked: the click that follows would toggle it.
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
    let columns = props.columns.max(1);
    let value = props.value.as_deref();
    let current = color_index(options, value);
    let pickable: Vec<usize> = (0..options.len()).collect();
    let labels: Vec<String> = options.iter().map(|o| o.label.to_string()).collect();

    let show = {
        let (open, state, pickable) = (open.clone(), state.clone(), pickable.clone());
        let disabled = props.disabled;
        move |last: bool| {
            if disabled {
                return;
            }
            let mut state = state.borrow_mut();
            match current {
                Some(i) => state.set(Some(i)),
                None => state.go_to_end(&pickable, last),
            }
            open.set(true);
        }
    };
    let choose = {
        let (open, on_change, current) =
            (open.clone(), props.on_change.clone(), props.value.clone());
        Callback::from(move |picked: AttrValue| {
            open.set(false);
            if current.as_ref() != Some(&picked) {
                on_change.emit(picked);
            }
        })
    };
    let pick = {
        let choose = choose.clone();
        let values: Vec<AttrValue> = options.iter().map(|o| o.value.clone()).collect();
        move |i: usize| {
            if let Some(value) = values.get(i) {
                choose.emit(value.clone());
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
    let has_custom = props.custom.is_some();
    let onkeydown = {
        let (is_open, state, redraw) = (*open, state.clone(), redraw.clone());
        let (open, pick, show) = (open.clone(), pick.clone(), show.clone());
        let (labels, pickable) = (labels.clone(), pickable.clone());
        let custom_input = custom_input.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            *swallow.borrow_mut() = matches!(key.as_str(), "Enter" | " ");
            let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
            if !is_open {
                if matches!(
                    key.as_str(),
                    "ArrowDown"
                        | "ArrowUp"
                        | "ArrowLeft"
                        | "ArrowRight"
                        | "Enter"
                        | " "
                        | "Home"
                        | "End"
                ) {
                    e.prevent_default();
                    show(matches!(key.as_str(), "ArrowUp" | "ArrowLeft" | "End"));
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
                match custom_input.cast::<HtmlElement>() {
                    Some(input) if has_custom && !e.shift_key() => {
                        e.prevent_default();
                        _ = input.focus();
                    }
                    _ => open.set(false),
                }
                return;
            }
            let taken = color_key(
                &mut state.borrow_mut(),
                &key,
                e.alt_key(),
                columns,
                &pickable,
            );
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

    let popup = (*open).then(|| {
        let cell = |(i, o): (usize, &ColorOption)| {
            let selected = current == Some(i);
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
            html! {
                <div id={option_id(&id, i)} role="option" class={color_option(is_active)}
                    aria-selected={selected.to_string()} aria-label={o.label.clone()}
                    title={o.label.clone()} {onclick} {onmousemove}>
                    { swatch(&o.value, o.value.is_empty()) }
                    <span class={classes!(class::COLOR_OPTION_MARK, (!selected).then_some(class::OFF))}
                        aria-hidden="true">
                        <Icon glyph={Glyph::Check} />
                    </span>
                </div>
            }
        };
        // Focus stays on the face while the grid is pressed.
        let hold_focus = Callback::from(|e: MouseEvent| e.prevent_default());
        let on_custom = {
            let (choose, face) = (choose.clone(), face.clone());
            Callback::from(move |e: Event| {
                let input: HtmlInputElement = e.target_unchecked_into();
                choose.emit(input.value().into());
                if let Some(face) = face.cast::<HtmlElement>() {
                    _ = face.focus();
                }
            })
        };
        let onkeydown = {
            let (open, face) = (open.clone(), face.clone());
            Callback::from(move |e: KeyboardEvent| {
                if e.key() == "Tab" {
                    e.prevent_default();
                    open.set(false);
                    if let Some(face) = face.cast::<HtmlElement>() {
                        _ = face.focus();
                    }
                }
            })
        };
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
        let custom = props.custom.clone().map(|label| {
            let shown = color_input_value(value.unwrap_or_default());
            html! {
                <label class={class::COLOR_CUSTOM}>
                    <span class={class::COLOR_CUSTOM_LABEL}>{ label }</span>
                    <input ref={custom_input.clone()} type="color" class={class::COLOR_CUSTOM_INPUT}
                        value={shown} onchange={on_custom}
                        data-testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-custom")))} />
                </label>
            }
        });
        overlay(html! {
            <div ref={list.clone()} class={color_list(placed.is_some())} {style} {onkeydown}>
                <div id={list_id.clone()} role="listbox" class={class::COLOR_GRID}
                    aria-label={props.label.clone()} onmousedown={hold_focus}
                    style={format!("--color-select-columns: {columns};")}
                    data-testid={props.testid.as_ref().map(|t| AttrValue::from(format!("{t}-options")))}>
                    { for options.iter().enumerate().map(cell) }
                </div>
                { for custom }
            </div>
        })
    });

    let (face_swatch, face_label) = match (value, current) {
        (Some(v), Some(i)) => (swatch(v, v.is_empty()), Some(options[i].label.clone())),
        (Some(v), None) if !v.is_empty() => (swatch(v, false), Some(AttrValue::from(v.to_owned()))),
        _ => (swatch("", true), None),
    };
    let text = match (&face_label, &props.placeholder) {
        (Some(label), _) => {
            html! { <span class={class::COLOR_SELECT_LABEL}>{ label.clone() }</span> }
        }
        (None, Some(p)) => {
            html! { <span class={classes!(class::COLOR_SELECT_LABEL, class::COLOR_SELECT_PLACEHOLDER)}>{ p.clone() }</span> }
        }
        (None, None) => html! { <span class={class::COLOR_SELECT_LABEL} /> },
    };

    let swatch_only = props.width == ColorSelectWidth::Swatch;

    html! {
        <>
            <button ref={face} type="button" id={props.id.clone()}
                class={classes!(
                    color_select(props.size, props.width, props.invalid, props.disabled, props.busy),
                    props.class.clone()
                )}
                role="combobox" aria-haspopup="listbox" aria-expanded={open.to_string()}
                aria-controls={open.then(|| list_id.clone())}
                aria-activedescendant={open.then(|| active.map(|i| option_id(&id, i))).flatten()}
                aria-label={props.label.clone().map(|l| match &face_label {
                    Some(name) => AttrValue::from(format!("{l}: {name}")),
                    None => l,
                })}
                aria-describedby={props.describedby.clone()}
                aria-invalid={props.invalid.then_some("true")}
                aria-busy={props.busy.then_some("true")}
                disabled={props.disabled} data-testid={props.testid.clone()}
                {onclick} {onmousedown} {onkeydown}>
                { face_swatch }
                if !swatch_only {
                    { text }
                    <span class={class::COLOR_SELECT_CHEVRON} aria-hidden="true">
                        <Icon glyph={Glyph::ChevronDown} />
                    </span>
                }
            </button>
            { popup.unwrap_or_default() }
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn render(props: ColorSelectProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<ColorSelect>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn palette() -> Vec<ColorOption> {
        vec![
            ColorOption::new("", "None"),
            ColorOption::new("#d92d20", "Red"),
            ColorOption::new("#12a150", "Green"),
            ColorOption::new("#175cd3", "Blue"),
        ]
    }

    #[test]
    fn width_and_the_input_box_states_are_modifiers() {
        let fixed = |size| {
            names(color_select(
                size,
                ColorSelectWidth::Fixed,
                false,
                false,
                false,
            ))
        };
        assert_eq!(
            fixed(InputSize::Regular),
            [class::INPUT_BOX, class::COLOR_SELECT]
        );
        assert_eq!(
            fixed(InputSize::Small),
            [class::INPUT_BOX, class::SMALL, class::COLOR_SELECT]
        );
        let all = |width| names(color_select(InputSize::Regular, width, true, true, true));
        assert_eq!(
            all(ColorSelectWidth::Fill),
            [
                class::INPUT_BOX,
                class::INVALID,
                class::DISABLED,
                class::BUSY,
                class::COLOR_SELECT,
                class::FILL
            ]
        );
        assert_eq!(
            all(ColorSelectWidth::Swatch).last().map(String::as_str),
            Some(class::SWATCH_ONLY)
        );
    }

    #[test]
    fn none_placed_and_active_are_modifiers() {
        assert_eq!(names(color_swatch(false)), [class::COLOR_SWATCH]);
        assert_eq!(
            names(color_swatch(true)),
            [class::COLOR_SWATCH, class::NONE]
        );
        assert_eq!(names(color_list(false)), [class::COLOR_LIST]);
        assert_eq!(names(color_list(true)), [class::COLOR_LIST, class::PLACED]);
        assert_eq!(names(color_option(false)), [class::COLOR_OPTION]);
        assert_eq!(
            names(color_option(true)),
            [class::COLOR_OPTION, class::ACTIVE]
        );
    }

    #[test]
    fn a_value_round_trips_through_its_option_and_the_native_input() {
        let options = palette();
        for (i, option) in options.iter().enumerate() {
            assert_eq!(color_index(&options, Some(&option.value)), Some(i));
            assert_eq!(color_name(&options, &option.value), option.label);
        }
        assert_eq!(color_index(&options, Some("#000001")), None);
        assert_eq!(color_index(&options, None), None);
        // A custom colour is named by itself.
        assert_eq!(color_name(&options, "#abcdef"), "#abcdef");
        assert_eq!(color_input_value("#abcdef"), "#abcdef");
        assert_eq!(color_input_value("red"), "#000000");
        assert_eq!(color_input_value(""), "#000000");
        assert_eq!(color_input_value("#12345g"), "#000000");
    }

    #[test]
    fn the_arrows_pick_by_cell_and_row_and_enter_picks() {
        let pickable: Vec<usize> = (0..10).collect();
        let mut list = Listbox::default();
        list.set(Some(0));
        assert_eq!(
            color_key(&mut list, "ArrowRight", false, 4, &pickable),
            ListKey::Moved
        );
        assert_eq!(list.active(), Some(1));
        assert_eq!(
            color_key(&mut list, "ArrowDown", false, 4, &pickable),
            ListKey::Moved
        );
        assert_eq!(list.active(), Some(5));
        assert_eq!(
            color_key(&mut list, "ArrowDown", false, 4, &pickable),
            ListKey::Moved
        );
        assert_eq!(list.active(), Some(9));
        // Stops at the end rather than wrapping.
        color_key(&mut list, "ArrowDown", false, 4, &pickable);
        assert_eq!(list.active(), Some(9));
        assert_eq!(
            color_key(&mut list, "ArrowUp", false, 4, &pickable),
            ListKey::Moved
        );
        assert_eq!(list.active(), Some(5));
        assert_eq!(
            color_key(&mut list, "ArrowLeft", false, 4, &pickable),
            ListKey::Moved
        );
        assert_eq!(list.active(), Some(4));
        assert_eq!(
            color_key(&mut list, "Enter", false, 4, &pickable),
            ListKey::Pick(4)
        );
        assert_eq!(
            color_key(&mut list, "ArrowUp", true, 4, &pickable),
            ListKey::Pick(4)
        );
        assert_eq!(
            color_key(&mut list, "Home", false, 4, &pickable),
            ListKey::Moved
        );
        assert_eq!(list.active(), Some(0));
        assert_eq!(
            color_key(&mut list, "x", false, 4, &pickable),
            ListKey::Ignored
        );
    }

    #[test]
    fn the_face_shows_the_chosen_swatch_and_name() {
        let html = render(yew::props!(ColorSelectProps {
            options: palette(),
            value: Some(AttrValue::Static("#12a150")),
        }));
        assert!(html.contains("--swatch: #12a150;"), "{html}");
        assert!(html.contains(">Green</span>"), "{html}");
        assert!(html.contains("role=\"combobox\""), "{html}");
        assert!(!html.contains("role=\"listbox\""), "{html}");
    }

    #[test]
    fn nothing_chosen_shows_the_placeholder_struck_through() {
        let html = render(yew::props!(ColorSelectProps {
            options: palette(),
            placeholder: Some(AttrValue::Static("Pick a colour")),
        }));
        assert!(html.contains(class::NONE), "{html}");
        assert!(html.contains(class::COLOR_SELECT_PLACEHOLDER), "{html}");
        assert!(html.contains("Pick a colour"), "{html}");
    }
}
