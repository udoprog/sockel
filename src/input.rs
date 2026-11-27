//! TextInput, Textarea, SearchField and InputGroup (`style/_inputs.scss`).

use crate::phrasebookcatalog01::{L10N, label};
use crate::testid::with_testid;
use crate::{Glyph, Icon, class, use_field};
use phrasebook02::yew023::use_phrases_with;
use web_sys::HtmlInputElement;
use yew::prelude::*;

/// An input's height: `--control-height`, `--control-height-small` or
/// `--touch-height`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputSize {
    #[default]
    Regular,
    Small,
    Touch,
}

/// The `type` of a [`TextInput`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputKind {
    #[default]
    Text,
    Email,
    Url,
    Tel,
    /// A number (`type="number"`); for digits without a spinner, `Text`
    /// with `inputmode="numeric"`.
    Number,
    /// A local date and time (`type="datetime-local"`).
    DateTime,
}

impl InputKind {
    fn attr(self) -> &'static str {
        match self {
            InputKind::Text => "text",
            InputKind::Email => "email",
            InputKind::Url => "url",
            InputKind::Tel => "tel",
            InputKind::Number => "number",
            InputKind::DateTime => "datetime-local",
        }
    }
}

pub(crate) fn size_class(size: InputSize) -> Option<&'static str> {
    match size {
        InputSize::Regular => None,
        InputSize::Small => Some(class::SMALL),
        InputSize::Touch => Some(class::TOUCH),
    }
}

/// A bare input's classes: `input`, its size, `invalid` and `busy`.
pub fn input(size: InputSize, invalid: bool, busy: bool) -> Classes {
    classes!(
        class::INPUT,
        size_class(size),
        invalid.then_some(class::INVALID),
        busy.then_some(class::BUSY)
    )
}

/// A native `<select>`'s classes: `input` and `native-select`, its size,
/// `invalid`, and `fill` to take its container's width (otherwise as wide as
/// its widest option).
pub fn native_select(size: InputSize, invalid: bool, fill: bool) -> Classes {
    classes!(
        input(size, invalid, false),
        class::NATIVE_SELECT,
        fill.then_some(class::FILL)
    )
}

/// The classes of the box around an input and its marks: `input-box`, its
/// size, `invalid`, `disabled` and `busy`.
pub fn input_box(size: InputSize, invalid: bool, disabled: bool, busy: bool) -> Classes {
    classes!(
        class::INPUT_BOX,
        size_class(size),
        invalid.then_some(class::INVALID),
        disabled.then_some(class::DISABLED),
        busy.then_some(class::BUSY)
    )
}

/// A textarea's classes: `textarea`, its size, `invalid` and `busy`.
pub fn textarea(size: InputSize, invalid: bool, busy: bool) -> Classes {
    classes!(
        class::TEXTAREA,
        size_class(size),
        invalid.then_some(class::INVALID),
        busy.then_some(class::BUSY)
    )
}

/// The classes of each member of an [`InputGroup`], in order, for the
/// member's `class`. `detached[i]` starts a new run of joined members after
/// a gap (the flag of the first member means nothing). In a run the first
/// member squares its end corners, the last its start corners, and those
/// between all of them; a member alone in its run is a whole control.
pub fn group_classes(detached: &[bool]) -> Vec<Classes> {
    let len = detached.len();
    let starts = |i: usize| i == 0 || detached[i];
    (0..len)
        .map(|i| {
            let first = starts(i);
            let last = i + 1 == len || detached[i + 1];
            let position = match (first, last) {
                (true, true) => None,
                (true, false) => Some(class::GROUP_FIRST),
                (false, true) => Some(class::GROUP_LAST),
                (false, false) => Some(class::GROUP_MIDDLE),
            };
            classes!(
                position,
                (i > 0 && detached[i]).then_some(class::GROUP_DETACHED)
            )
        })
        .collect()
}

#[derive(Properties, Default, PartialEq)]
pub struct InputGroupProps {
    #[prop_or_default]
    pub class: Classes,
    /// The members, each with the classes [`group_classes`] gives it.
    #[prop_or_default]
    pub children: Html,
}

/// Controls joined in a row: one border between neighbours, the corners
/// that join squared, and the same height while they share a size.
#[function_component]
pub fn InputGroup(props: &InputGroupProps) -> Html {
    html! { <div class={classes!(class::INPUT_GROUP, props.class.clone())} role="group">{ props.children.clone() }</div> }
}

#[derive(Properties, Default, PartialEq)]
pub struct TextInputProps {
    pub value: AttrValue,
    /// Each edit, with the new text.
    pub on_input: Callback<String>,
    /// The text when the edit is committed (blur or enter).
    #[prop_or_default]
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub on_blur: Callback<FocusEvent>,
    #[prop_or_default]
    pub on_keydown: Callback<KeyboardEvent>,
    /// The `id` a [`Field`](crate::Field)'s label names.
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub name: Option<AttrValue>,
    /// The most characters it takes.
    #[prop_or_default]
    pub maxlength: Option<u32>,
    /// The id of a `<datalist>` of suggestions.
    #[prop_or_default]
    pub list: Option<AttrValue>,
    /// The keyboard a touch screen shows (`decimal`, `numeric`, ...).
    #[prop_or_default]
    pub inputmode: Option<AttrValue>,
    /// The label of a touch keyboard's enter key (`go`, `search`, ...).
    #[prop_or_default]
    pub enterkeyhint: Option<AttrValue>,
    /// The browser's autofill (`off`, `username`, ...), only when set.
    #[prop_or_default]
    pub autocomplete: Option<AttrValue>,
    /// Spell checking on or off, only when set.
    #[prop_or_default]
    pub spellcheck: Option<bool>,
    #[prop_or_default]
    pub kind: InputKind,
    #[prop_or_default]
    pub size: InputSize,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    /// The accessible name, when there is no label for `id`.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// `field_describedby`, the ids of the field's hint and error.
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    /// A unit after the text, inside the field (`aria-hidden`; name it in
    /// the label).
    #[prop_or_default]
    pub unit: Option<Html>,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub required: bool,
    /// Waiting on something (`aria-busy`, a progress cursor).
    #[prop_or_default]
    pub busy: bool,
    /// Takes the focus on mount; inside a dialog or popover
    /// (`[role=dialog]`) it is instead the one they focus as they open
    /// (`data-autofocus`).
    #[prop_or_default]
    pub autofocus: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub node_ref: NodeRef,
}

/// Focuses `node` unless a dialog or popover holds it, which focuses its
/// `data-autofocus` element itself once it is open and placed.
fn focus_unless_in_dialog(node: &NodeRef) {
    let Some(input) = node.cast::<web_sys::HtmlElement>() else {
        return;
    };
    if input.closest("[role=dialog]").ok().flatten().is_none() {
        _ = input.focus();
    }
}

fn value_of(e: &InputEvent) -> String {
    e.target_unchecked_into::<HtmlInputElement>().value()
}

/// A single line of text at the height of a control. With a `unit` it is a
/// box around the text and the unit, the box taking the border and the
/// focus ring; the unit never moves the text's start.
#[function_component]
pub fn TextInput(props: &TextInputProps) -> Html {
    let oninput = props.on_input.reform(|e: InputEvent| value_of(&e));
    let onchange = props
        .on_change
        .reform(|e: Event| e.target_unchecked_into::<HtmlInputElement>().value());
    let (id, describedby, invalid) =
        use_field()
            .unwrap_or_default()
            .control(&props.id, &props.describedby, props.invalid);
    let aria_invalid = invalid.then_some("true");
    let aria_busy = props.busy.then_some("true");
    {
        let node = props.node_ref.clone();
        let autofocus = props.autofocus;
        use_effect_with((), move |_| {
            if autofocus {
                focus_unless_in_dialog(&node);
            }
        });
    }

    macro_rules! field {
        ($class:expr) => {
            with_testid(html! {
                <input class={$class} id={id.clone()} name={props.name.clone()}
                    type={props.kind.attr()} value={props.value.clone()}
                    maxlength={props.maxlength.map(|n| n.to_string())} list={props.list.clone()}
                    inputmode={props.inputmode.clone()} enterkeyhint={props.enterkeyhint.clone()}
                    autocomplete={props.autocomplete.clone()}
                    spellcheck={props.spellcheck.map(|on| if on { "true" } else { "false" })}
                    placeholder={props.placeholder.clone()} aria-label={props.label.clone()}
                    aria-describedby={describedby.clone()}
                    aria-invalid={aria_invalid} aria-busy={aria_busy}
                    disabled={props.disabled} readonly={props.readonly} required={props.required}
                    data-autofocus={props.autofocus.then_some("")} ref={props.node_ref.clone()}
                    {oninput} {onchange} onblur={props.on_blur.clone()}
                    onkeydown={props.on_keydown.clone()} />
            }, props.testid.clone())
        };
    }

    match &props.unit {
        None => field!(classes!(
            input(props.size, invalid, props.busy),
            props.class.clone()
        )),
        Some(unit) => html! {
            <span class={classes!(
                input_box(props.size, invalid, props.disabled, props.busy),
                props.class.clone()
            )}>
                { field!(classes!(class::INPUT_CONTROL)) }
                <span class={classes!(class::INPUT_MARK, class::INPUT_UNIT)} aria-hidden="true">
                    { unit.clone() }
                </span>
            </span>
        },
    }
}

#[derive(Properties, PartialEq)]
pub struct TextareaProps {
    pub value: AttrValue,
    pub on_input: Callback<String>,
    #[prop_or_default]
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub name: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    /// The lines it is high, at least `--input-height`.
    #[prop_or(3)]
    pub rows: u32,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub describedby: Option<AttrValue>,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub readonly: bool,
    #[prop_or_default]
    pub required: bool,
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The textarea, for a caller that focuses, measures or sizes it.
    #[prop_or_default]
    pub node_ref: NodeRef,
    #[prop_or_default]
    pub on_keydown: Callback<KeyboardEvent>,
    #[prop_or_default]
    pub on_paste: Callback<Event>,
    #[prop_or_default]
    pub on_drop: Callback<DragEvent>,
}

impl Default for TextareaProps {
    fn default() -> Self {
        Self {
            value: Default::default(),
            on_input: Default::default(),
            on_change: Default::default(),
            id: Default::default(),
            name: Default::default(),
            size: Default::default(),
            rows: ::yew::html::IntoPropValue::into_prop_value(3),
            placeholder: Default::default(),
            label: Default::default(),
            describedby: Default::default(),
            invalid: Default::default(),
            disabled: Default::default(),
            readonly: Default::default(),
            required: Default::default(),
            busy: Default::default(),
            class: Default::default(),
            testid: Default::default(),
            node_ref: Default::default(),
            on_keydown: Default::default(),
            on_paste: Default::default(),
            on_drop: Default::default(),
        }
    }
}

/// Lines of text: as high as its `rows`, resized only vertically by the
/// user, and the same border and focus ring as a [`TextInput`].
#[function_component]
pub fn Textarea(props: &TextareaProps) -> Html {
    let oninput = props.on_input.reform(|e: InputEvent| {
        e.target_unchecked_into::<web_sys::HtmlTextAreaElement>()
            .value()
    });
    let onchange = props.on_change.reform(|e: Event| {
        e.target_unchecked_into::<web_sys::HtmlTextAreaElement>()
            .value()
    });
    let (id, describedby, invalid) =
        use_field()
            .unwrap_or_default()
            .control(&props.id, &props.describedby, props.invalid);

    with_testid(
        html! {
            <textarea class={classes!(textarea(props.size, invalid, props.busy), props.class.clone())}
                {id} name={props.name.clone()} rows={props.rows.to_string()}
                value={props.value.clone()} placeholder={props.placeholder.clone()}
                aria-label={props.label.clone()} aria-describedby={describedby}
                aria-invalid={invalid.then_some("true")}
                aria-busy={props.busy.then_some("true")}
                disabled={props.disabled} readonly={props.readonly} required={props.required} ref={props.node_ref.clone()}
                {oninput} {onchange} onkeydown={props.on_keydown.clone()}
                onpaste={props.on_paste.clone()} ondrop={props.on_drop.clone()} />
        },
        props.testid.clone(),
    )
}

#[derive(Properties, PartialEq)]
pub struct SearchFieldProps {
    pub value: AttrValue,
    /// Each edit, and `""` when the field is cleared.
    pub on_input: Callback<String>,
    /// The placeholder and the accessible name.
    pub label: AttrValue,
    /// The clear button's accessible name.
    #[prop_or_default]
    pub clear_label: Option<AttrValue>,
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub size: InputSize,
    #[prop_or_default]
    pub invalid: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// Searching: a spinner takes the search icon's place (`aria-busy`).
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub autofocus: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The text input, for a caller that focuses or reads it.
    #[prop_or_default]
    pub input_ref: Option<NodeRef>,
    /// Keys in the text, before the field's own: escape clears unless this
    /// prevents the default.
    #[prop_or_default]
    pub on_keydown: Option<Callback<KeyboardEvent>>,
    /// Escape clears the text (the default). `false` leaves escape to
    /// `on_keydown`, its default already prevented, for a combobox that
    /// closes its list with it.
    #[prop_or(true)]
    pub escape_clears: bool,
    /// The id of the listbox it filters, making it that list's combobox
    /// (`role="combobox"`, `aria-controls`).
    #[prop_or_default]
    pub controls: Option<AttrValue>,
    /// With `controls`, the active option's id (`aria-activedescendant`).
    #[prop_or_default]
    pub activedescendant: Option<AttrValue>,
    /// With `controls`, whether the list shows (`aria-expanded`); unset is
    /// shown.
    #[prop_or_default]
    pub expanded: Option<bool>,
    /// A placeholder other than the `label`.
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    /// The text's tooltip (`title`).
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The keys that focus it from elsewhere (`aria-keyshortcuts`).
    #[prop_or_default]
    pub keyshortcuts: Option<AttrValue>,
    #[prop_or_default]
    pub on_focus: Option<Callback<FocusEvent>>,
    #[prop_or_default]
    pub on_blur: Option<Callback<FocusEvent>>,
}

impl Default for SearchFieldProps {
    fn default() -> Self {
        Self {
            value: Default::default(),
            on_input: Default::default(),
            label: Default::default(),
            clear_label: Default::default(),
            id: Default::default(),
            size: Default::default(),
            invalid: Default::default(),
            disabled: Default::default(),
            busy: Default::default(),
            autofocus: Default::default(),
            class: Default::default(),
            testid: Default::default(),
            input_ref: Default::default(),
            on_keydown: Default::default(),
            escape_clears: ::yew::html::IntoPropValue::into_prop_value(true),
            controls: Default::default(),
            activedescendant: Default::default(),
            expanded: Default::default(),
            placeholder: Default::default(),
            title: Default::default(),
            keyshortcuts: Default::default(),
            on_focus: Default::default(),
            on_blur: Default::default(),
        }
    }
}

/// A search icon, the text and a button that clears it. The icon's and the
/// button's room are always there (the button only hidden while the field
/// is empty), so typing never widens, shifts or resizes the field. Clearing
/// (the button, or escape) empties the value and returns the focus to the
/// text.
#[function_component]
pub fn SearchField(props: &SearchFieldProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let own = use_node_ref();
    let input = props.input_ref.clone().unwrap_or(own);
    let empty = props.value.is_empty();
    let combobox = props.controls.is_some();

    let oninput = props.on_input.reform(|e: InputEvent| value_of(&e));
    let clear = {
        let on_input = props.on_input.clone();
        let input = input.clone();
        Callback::from(move |()| {
            on_input.emit(String::new());
            if let Some(input) = input.cast::<HtmlInputElement>() {
                let _ = input.focus();
            }
        })
    };
    let onclick = clear.reform(|_: MouseEvent| ());
    let onkeydown = {
        let clear = clear.clone();
        let on_keydown = props.on_keydown.clone();
        let escape_clears = props.escape_clears;
        Callback::from(move |e: KeyboardEvent| {
            // Prevented before `on_keydown` sees it: a component's message
            // is handled only after this returns.
            if e.key() == "Escape" && !escape_clears {
                e.prevent_default();
            }
            if let Some(on_keydown) = &on_keydown {
                on_keydown.emit(e.clone());
            }
            if e.key() == "Escape" && escape_clears && !empty && !e.default_prevented() {
                e.prevent_default();
                clear.emit(());
            }
        })
    };
    let glyph = if props.busy {
        Glyph::Spinner
    } else {
        Glyph::Search
    };

    html! {
        <span class={classes!(
            input_box(props.size, props.invalid, props.disabled, props.busy),
            props.class.clone()
        )} aria-busy={props.busy.then_some("true")}>
            <span class={class::INPUT_MARK}><Icon {glyph} /></span>
            { with_testid(html! { <input class={class::INPUT_CONTROL} type="search" id={props.id.clone()}
                value={props.value.clone()}
                placeholder={props.placeholder.clone().unwrap_or_else(|| props.label.clone())}
                title={props.title.clone()} aria-keyshortcuts={props.keyshortcuts.clone()}
                aria-label={props.label.clone()} aria-invalid={props.invalid.then_some("true")}
                disabled={props.disabled} enterkeyhint="search"
                data-autofocus={props.autofocus.then_some("")}
                role={combobox.then_some("combobox")}
                aria-autocomplete={combobox.then_some("list")}
                aria-expanded={combobox.then(|| if props.expanded.unwrap_or(true) { "true" } else { "false" })}
                aria-controls={props.controls.clone()}
                aria-activedescendant={props.activedescendant.clone()}
                autocomplete={combobox.then_some("off")} spellcheck={combobox.then_some("false")} ref={input}
                onfocus={props.on_focus.clone()} onblur={props.on_blur.clone()}
                {oninput} {onkeydown} /> }, props.testid.clone()) }
            <button type="button" class={classes!(class::INPUT_CLEAR, empty.then_some(class::EMPTY))}
                aria-label={label(&phrases, &props.clear_label, "sockel-search-field-clear")} disabled={props.disabled || empty}
                aria-hidden={empty.then_some("true")} tabindex={empty.then_some("-1")} {onclick}>
                <Icon glyph={Glyph::Close} />
            </button>
        </span>
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

    fn search(value: &str) -> SearchFieldProps {
        SearchFieldProps {
            value: value.to_owned().into(),
            on_input: Callback::noop(),
            label: "Search".into(),
            clear_label: None,
            id: None,
            size: InputSize::Regular,
            invalid: false,
            disabled: false,
            busy: false,
            autofocus: false,
            class: Classes::new(),
            testid: None,
            input_ref: None,
            on_keydown: None,
            escape_clears: true,
            controls: None,
            activedescendant: None,
            expanded: None,
            placeholder: None,
            title: None,
            keyshortcuts: None,
            on_focus: None,
            on_blur: None,
        }
    }

    #[test]
    fn a_text_input_passes_its_keyboard_and_limits() {
        let html = render::<TextInput>(yew::props!(TextInputProps {
            value: AttrValue::from("59.3"),
            on_input: Callback::noop(),
            maxlength: Some(12),
            list: Some(AttrValue::from("places")),
            inputmode: Some(AttrValue::from("decimal")),
            enterkeyhint: Some(AttrValue::from("go")),
        }));
        for attr in [
            "maxlength=\"12\"",
            "list=\"places\"",
            "inputmode=\"decimal\"",
            "enterkeyhint=\"go\"",
        ] {
            assert!(html.contains(attr), "{attr} in {html}");
        }
    }

    #[test]
    fn autocomplete_and_spellcheck_only_when_set() {
        let plain = render::<TextInput>(yew::props!(TextInputProps {
            value: AttrValue::from(""),
            on_input: Callback::noop(),
        }));
        assert!(!plain.contains("autocomplete") && !plain.contains("spellcheck"));
        let html = render::<TextInput>(yew::props!(TextInputProps {
            value: AttrValue::from(""),
            on_input: Callback::noop(),
            autocomplete: AttrValue::from("off"),
            spellcheck: false,
        }));
        assert!(html.contains("autocomplete=\"off\""), "{html}");
        assert!(html.contains("spellcheck=\"false\""), "{html}");
    }

    #[test]
    fn a_kind_is_its_type() {
        for (kind, attr) in [
            (InputKind::Number, "type=\"number\""),
            (InputKind::DateTime, "type=\"datetime-local\""),
        ] {
            let html = render::<TextInput>(yew::props!(TextInputProps {
                value: AttrValue::from(""),
                on_input: Callback::noop(),
                kind: kind,
            }));
            assert!(html.contains(attr), "{attr} in {html}");
        }
    }

    #[test]
    fn a_textarea_takes_its_ref_and_events() {
        let props = yew::props!(TextareaProps {
            value: AttrValue::from(""),
            on_input: Callback::noop(),
        });
        let html = render::<Textarea>(TextareaProps {
            on_paste: Callback::from(|_: Event| ()),
            on_drop: Callback::from(|_: DragEvent| ()),
            ..props
        });
        assert!(html.contains(class::TEXTAREA), "{html}");
    }

    #[test]
    fn each_size_is_its_modifier() {
        let of = |s| names(input(s, false, false));
        assert_eq!(of(InputSize::Regular), [class::INPUT]);
        assert_eq!(of(InputSize::Small), [class::INPUT, class::SMALL]);
        assert_eq!(of(InputSize::Touch), [class::INPUT, class::TOUCH]);
        assert_eq!(
            names(textarea(InputSize::Small, false, false)),
            [class::TEXTAREA, class::SMALL]
        );
        assert_eq!(
            names(input_box(InputSize::Touch, false, false, false)),
            [class::INPUT_BOX, class::TOUCH]
        );
    }

    #[test]
    fn a_native_select_is_an_input() {
        assert_eq!(
            names(native_select(InputSize::Regular, false, false)),
            [class::INPUT, class::NATIVE_SELECT]
        );
        assert_eq!(
            names(native_select(InputSize::Small, true, true)),
            [
                class::INPUT,
                class::SMALL,
                class::INVALID,
                class::NATIVE_SELECT,
                class::FILL
            ]
        );
    }

    #[test]
    fn invalid_disabled_and_busy_are_modifiers() {
        assert_eq!(
            names(input(InputSize::Regular, true, true)),
            [class::INPUT, class::INVALID, class::BUSY]
        );
        assert_eq!(
            names(input_box(InputSize::Regular, true, true, true)),
            [
                class::INPUT_BOX,
                class::INVALID,
                class::DISABLED,
                class::BUSY
            ]
        );
        assert_eq!(
            names(textarea(InputSize::Regular, true, false)),
            [class::TEXTAREA, class::INVALID]
        );
    }

    #[test]
    fn group_members_are_first_middle_and_last() {
        let of = |detached: &[bool]| -> Vec<Vec<String>> {
            group_classes(detached).into_iter().map(names).collect()
        };
        assert_eq!(of(&[false]), [Vec::<String>::new()]);
        assert_eq!(
            of(&[false, false]),
            [[class::GROUP_FIRST], [class::GROUP_LAST]]
        );
        assert_eq!(
            of(&[false, false, false]),
            [
                [class::GROUP_FIRST],
                [class::GROUP_MIDDLE],
                [class::GROUP_LAST]
            ]
        );
    }

    #[test]
    fn a_detached_member_starts_its_own_run_after_a_gap() {
        let of = |detached: &[bool]| -> Vec<Vec<String>> {
            group_classes(detached).into_iter().map(names).collect()
        };
        assert_eq!(
            of(&[false, false, true]),
            [
                vec![class::GROUP_FIRST.to_owned()],
                vec![class::GROUP_LAST.to_owned()],
                vec![class::GROUP_DETACHED.to_owned()],
            ]
        );
        assert_eq!(
            of(&[false, true, false]),
            [
                vec![],
                vec![
                    class::GROUP_FIRST.to_owned(),
                    class::GROUP_DETACHED.to_owned()
                ],
                vec![class::GROUP_LAST.to_owned()],
            ]
        );
        assert!(group_classes(&[]).is_empty());
    }

    #[test]
    fn the_clear_button_keeps_its_room_empty_or_filled() {
        let empty = render::<SearchField>(search(""));
        let filled = render::<SearchField>(search("abc"));
        for html in [&empty, &filled] {
            assert!(html.contains(class::INPUT_CLEAR), "{html}");
            assert!(html.contains("type=\"search\""), "{html}");
        }
        assert!(empty.contains(class::EMPTY), "{empty}");
        assert!(empty.contains("disabled"), "{empty}");
        assert!(empty.contains("tabindex=\"-1\""), "{empty}");
        assert!(!filled.contains(class::EMPTY), "{filled}");
        assert!(!filled.contains("tabindex"), "{filled}");
    }

    #[test]
    fn escape_clears_unless_turned_off() {
        let props = yew::props!(SearchFieldProps {
            value: AttrValue::from(""),
            on_input: Callback::noop(),
            label: AttrValue::from("Search"),
        });
        assert!(props.escape_clears);
    }

    #[test]
    fn a_field_controlling_a_list_is_its_combobox() {
        let plain = render::<SearchField>(search(""));
        assert!(!plain.contains("combobox"), "{plain}");
        let html = render::<SearchField>(SearchFieldProps {
            controls: Some("list".into()),
            activedescendant: Some("list-2".into()),
            ..search("ab")
        });
        assert!(html.contains("role=\"combobox\""), "{html}");
        assert!(html.contains("aria-controls=\"list\""), "{html}");
        assert!(html.contains("aria-activedescendant=\"list-2\""), "{html}");
        assert!(html.contains(class::INPUT_CLEAR), "{html}");
        assert!(html.contains("aria-expanded=\"true\""), "{html}");
        let shut = render::<SearchField>(SearchFieldProps {
            controls: Some("list".into()),
            expanded: Some(false),
            ..search("ab")
        });
        assert!(shut.contains("aria-expanded=\"false\""), "{shut}");
    }

    #[test]
    fn a_placeholder_title_and_shortcut_are_its_own() {
        let plain = render::<SearchField>(search(""));
        assert!(plain.contains("placeholder=\"Search\""), "{plain}");
        assert!(
            !plain.contains("title=") && !plain.contains("aria-keyshortcuts"),
            "{plain}"
        );
        let html = render::<SearchField>(SearchFieldProps {
            placeholder: Some("Find a rule".into()),
            title: Some("Search the rules".into()),
            keyshortcuts: Some("/".into()),
            ..search("")
        });
        assert!(html.contains("placeholder=\"Find a rule\""), "{html}");
        assert!(html.contains("aria-label=\"Search\""), "{html}");
        assert!(html.contains("title=\"Search the rules\""), "{html}");
        assert!(html.contains("aria-keyshortcuts=\"/\""), "{html}");
    }

    #[test]
    fn busy_swaps_the_icon_without_adding_one() {
        let idle = render::<SearchField>(search(""));
        let busy = render::<SearchField>(SearchFieldProps {
            busy: true,
            ..search("")
        });
        assert_eq!(idle.matches(class::GLYPH).count(), 2);
        assert_eq!(busy.matches(class::GLYPH).count(), 2);
        assert!(busy.contains(class::SPIN) && !idle.contains(class::SPIN));
        assert!(busy.contains("aria-busy=\"true\""), "{busy}");
    }

    #[test]
    fn a_unit_wraps_the_input_in_a_box() {
        let props = |unit| TextInputProps {
            value: "12".into(),
            on_input: Callback::noop(),
            on_change: Callback::noop(),
            on_blur: Callback::noop(),
            on_keydown: Callback::noop(),
            id: Some("n".into()),
            name: None,
            maxlength: None,
            list: None,
            inputmode: None,
            enterkeyhint: None,
            autocomplete: None,
            spellcheck: None,
            kind: InputKind::Text,
            size: InputSize::Regular,
            placeholder: None,
            label: None,
            describedby: Some("n-hint".into()),
            unit,
            invalid: true,
            disabled: false,
            readonly: false,
            required: false,
            busy: false,
            autofocus: false,
            class: Classes::new(),
            testid: None,
            node_ref: NodeRef::default(),
        };
        let bare = render::<TextInput>(props(None));
        assert!(!bare.contains(class::INPUT_BOX), "{bare}");
        assert!(bare.contains("aria-invalid=\"true\""), "{bare}");
        assert!(bare.contains("aria-describedby=\"n-hint\""), "{bare}");
        let with = render::<TextInput>(props(Some(html! { "ms" })));
        assert!(with.contains(class::INPUT_BOX), "{with}");
        assert!(with.contains(class::INPUT_UNIT), "{with}");
        assert!(with.contains(">ms</span>"), "{with}");
    }
}
