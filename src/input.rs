//! TextInput, Textarea, SearchField and InputGroup (`style/_inputs.scss`).

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Glyph, Icon, class};
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
}

impl InputKind {
    fn attr(self) -> &'static str {
        match self {
            InputKind::Text => "text",
            InputKind::Email => "email",
            InputKind::Url => "url",
            InputKind::Tel => "tel",
        }
    }
}

fn size_class(size: InputSize) -> Option<&'static str> {
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

#[derive(Properties, PartialEq)]
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

#[derive(Properties, PartialEq)]
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
    /// Takes the focus when its dialog or form opens (`data-autofocus`).
    #[prop_or_default]
    pub autofocus: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub node_ref: NodeRef,
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
    let aria_invalid = props.invalid.then_some("true");
    let aria_busy = props.busy.then_some("true");

    macro_rules! field {
        ($class:expr) => {
            html! {
                <input class={$class} id={props.id.clone()} name={props.name.clone()}
                    type={props.kind.attr()} value={props.value.clone()}
                    maxlength={props.maxlength.map(|n| n.to_string())} list={props.list.clone()}
                    inputmode={props.inputmode.clone()} enterkeyhint={props.enterkeyhint.clone()}
                    placeholder={props.placeholder.clone()} aria-label={props.label.clone()}
                    aria-describedby={props.describedby.clone()}
                    aria-invalid={aria_invalid} aria-busy={aria_busy}
                    disabled={props.disabled} readonly={props.readonly} required={props.required}
                    data-autofocus={props.autofocus.then_some("")}
                    data-testid={props.testid.clone()} ref={props.node_ref.clone()}
                    {oninput} {onchange} onblur={props.on_blur.clone()}
                    onkeydown={props.on_keydown.clone()} />
            }
        };
    }

    match &props.unit {
        None => field!(classes!(
            input(props.size, props.invalid, props.busy),
            props.class.clone()
        )),
        Some(unit) => html! {
            <span class={classes!(
                input_box(props.size, props.invalid, props.disabled, props.busy),
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

    html! {
        <textarea class={classes!(textarea(props.size, props.invalid, props.busy), props.class.clone())}
            id={props.id.clone()} name={props.name.clone()} rows={props.rows.to_string()}
            value={props.value.clone()} placeholder={props.placeholder.clone()}
            aria-label={props.label.clone()} aria-describedby={props.describedby.clone()}
            aria-invalid={props.invalid.then_some("true")}
            aria-busy={props.busy.then_some("true")}
            disabled={props.disabled} readonly={props.readonly} required={props.required}
            data-testid={props.testid.clone()} {oninput} {onchange} />
    }
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
    /// The id of the listbox it filters, making it that list's combobox
    /// (`role="combobox"`, `aria-controls`).
    #[prop_or_default]
    pub controls: Option<AttrValue>,
    /// With `controls`, the active option's id (`aria-activedescendant`).
    #[prop_or_default]
    pub activedescendant: Option<AttrValue>,
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
        Callback::from(move |e: KeyboardEvent| {
            if let Some(on_keydown) = &on_keydown {
                on_keydown.emit(e.clone());
            }
            if e.key() == "Escape" && !empty && !e.default_prevented() {
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
            <input class={class::INPUT_CONTROL} type="search" id={props.id.clone()}
                value={props.value.clone()} placeholder={props.label.clone()}
                aria-label={props.label.clone()} aria-invalid={props.invalid.then_some("true")}
                disabled={props.disabled} enterkeyhint="search"
                data-autofocus={props.autofocus.then_some("")}
                role={combobox.then_some("combobox")}
                aria-autocomplete={combobox.then_some("list")}
                aria-expanded={combobox.then_some("true")}
                aria-controls={props.controls.clone()}
                aria-activedescendant={props.activedescendant.clone()}
                autocomplete={combobox.then_some("off")} spellcheck={combobox.then_some("false")}
                data-testid={props.testid.clone()} ref={input}
                {oninput} {onkeydown} />
            <button type="button" class={classes!(class::INPUT_CLEAR, empty.then_some(class::EMPTY))}
                aria-label={label(&phrases, &props.clear_label, "plinth-search-field-clear")} disabled={props.disabled || empty}
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
            controls: None,
            activedescendant: None,
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
