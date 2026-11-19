//! PropertyList, PropertyRow and InlineEdit: an entity's fields as label and
//! value rows, each value edited where it stands (`style/_property-list.scss`).

use std::cell::Cell;
use std::rc::Rc;

use phrasebook02::Args;
use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonSize, ButtonVariant, InputSize, class, input, textarea};

/// A property list's classes: `property-list`, `boxed` when its fields show
/// their boxes at rest.
pub fn property_list(boxed: bool) -> Classes {
    classes!(class::PROPERTY_LIST, boxed.then_some(class::BOXED))
}

#[derive(Properties, PartialEq)]
pub struct PropertyListProps {
    /// What the list describes, for assistive technology.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// Fields in the values show their boxes at rest; by default a box
    /// shows only under the pointer or with focus, so the rows read as
    /// text.
    #[prop_or_default]
    pub boxed: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// Its [`PropertyRow`]s.
    #[prop_or_default]
    pub children: Html,
}

/// Label and value rows (`<dl>`): the labels in a column
/// `--property-label-width` wide (`--property-label-phone-width` on a
/// phone), each value beside its label at least a control's height, so a
/// row keeps its height whether its value is read or edited.
#[function_component]
pub fn PropertyList(props: &PropertyListProps) -> Html {
    html! {
        <dl class={classes!(property_list(props.boxed), props.class.clone())}
            aria-label={props.label.clone()} data-testid={props.testid.clone()}>
            { props.children.clone() }
        </dl>
    }
}

#[derive(Properties, PartialEq)]
pub struct PropertyRowProps {
    /// The property's name.
    pub label: AttrValue,
    /// A line under the value (what it means, why it cannot change).
    #[prop_or_default]
    pub hint: Option<AttrValue>,
    /// The label above the value, across the row (a description).
    #[prop_or_default]
    pub stacked: bool,
    /// The label's id, for a control that names itself by it
    /// (`aria-labelledby`).
    #[prop_or_default]
    pub id: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The value: text, a mark, an [`InlineEdit`], or a
    /// [`Picker`](crate::Picker) or [`Select`](crate::Select) with `fill`,
    /// which commit on a pick.
    #[prop_or_default]
    pub children: Html,
}

/// A property row's classes: `property-row`, `stacked` with the label
/// above the value.
pub fn property_row(stacked: bool) -> Classes {
    classes!(class::PROPERTY_ROW, stacked.then_some(class::STACKED))
}

/// One label and its value in a [`PropertyList`].
#[function_component]
pub fn PropertyRow(props: &PropertyRowProps) -> Html {
    html! {
        <div class={classes!(property_row(props.stacked), props.class.clone())}
            data-testid={props.testid.clone()}>
            <dt class={class::PROPERTY_LABEL} id={props.id.clone()}>{ props.label.clone() }</dt>
            <dd class={class::PROPERTY_VALUE}>
                { props.children.clone() }
                if let Some(hint) = props.hint.clone() {
                    <span class={class::PROPERTY_HINT}>{ hint }</span>
                }
            </dd>
        </div>
    }
}

/// What a key in an [`InlineEdit`]'s field does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    Commit,
    Cancel,
    /// Left to the field (a letter, a newline in long text).
    Type,
}

/// What `key` does in an inline edit's field: Escape cancels; Enter
/// commits a line, and Ctrl or Cmd+Enter long text, where Enter alone is a
/// newline. A press while composing is the input method's.
pub fn edit_key(key: &str, multiline: bool, mod_held: bool, composing: bool) -> EditKey {
    match key {
        _ if composing => EditKey::Type,
        "Escape" => EditKey::Cancel,
        "Enter" if !multiline || mod_held => EditKey::Commit,
        _ => EditKey::Type,
    }
}

/// An inline edit's face's classes: the input's box (so reading and
/// editing are one box), `inline-edit`, `empty` with no value, `multiline`
/// for long text.
pub fn inline_edit(empty: bool, multiline: bool) -> Classes {
    classes!(
        input(InputSize::Regular, false, false),
        class::INLINE_EDIT,
        empty.then_some(class::EMPTY),
        multiline.then_some(class::MULTILINE)
    )
}

#[derive(Properties, PartialEq)]
pub struct InlineEditProps {
    pub value: AttrValue,
    /// Told the new text when an edit commits with a change; not told of an
    /// edit that changed nothing or was cancelled.
    pub on_commit: Callback<String>,
    /// The property's name: the field's accessible name, and the face's
    /// with its value.
    pub label: AttrValue,
    /// Shown, muted, with no value; the catalog's when `None`.
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    /// Long text: a textarea where Enter is a newline, with Save
    /// (Ctrl or Cmd+Enter) and Cancel (Escape); a blur leaves it open.
    #[prop_or_default]
    pub multiline: bool,
    /// Told when editing starts and ends (an app pushing a shortcut layer).
    #[prop_or_default]
    pub on_editing: Callback<bool>,
    #[prop_or_default]
    pub maxlength: Option<u32>,
    #[prop_or_default]
    pub disabled: bool,
    /// While a commit is saved: the face shows the new value as `busy`.
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub save_label: Option<AttrValue>,
    #[prop_or_default]
    pub cancel_label: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

fn focus(node: &NodeRef) {
    if let Some(element) = node.cast::<HtmlElement>() {
        _ = element.focus();
    }
}

/// A value read as text that becomes its field where it stands: activate
/// it (a click, Enter or Space) and its text is selected in a field of the
/// same box; Enter or a blur commits, Escape cancels, and focus returns to
/// it. Long text (`multiline`) opens a textarea with Save and Cancel.
#[function_component]
pub fn InlineEdit(props: &InlineEditProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let editing = use_state(|| false);
    let draft = use_mut_ref(String::new);
    // Whether the open edit is still to end: Enter then the field's blur
    // end it once.
    let open = use_memo((), |()| Rc::new(Cell::new(false)));
    let refocus = use_memo((), |()| Rc::new(Cell::new(false)));
    let field = use_node_ref();
    let face = use_node_ref();

    {
        let (field, face, refocus) = (field.clone(), face.clone(), refocus.clone());
        use_effect_with(*editing, move |&on| {
            if on {
                if let Some(input) = field.cast::<HtmlInputElement>() {
                    _ = input.focus();
                    input.select();
                } else if let Some(area) = field.cast::<HtmlTextAreaElement>() {
                    _ = area.focus();
                    area.select();
                }
            } else if refocus.replace(false) {
                focus(&face);
            }
        });
    }

    let end = {
        let (editing, open, refocus) = (editing.clone(), open.clone(), refocus.clone());
        let (draft, value) = (draft.clone(), props.value.clone());
        let (on_commit, on_editing) = (props.on_commit.clone(), props.on_editing.clone());
        Callback::from(move |(commit, back): (bool, bool)| {
            if !open.replace(false) {
                return;
            }
            let text = draft.borrow().clone();
            refocus.set(back);
            editing.set(false);
            on_editing.emit(false);
            if commit && text != value.as_str() {
                on_commit.emit(text);
            }
        })
    };

    let start = {
        let (editing, open, draft) = (editing.clone(), open.clone(), draft.clone());
        let (value, on_editing, disabled) = (
            props.value.clone(),
            props.on_editing.clone(),
            props.disabled || props.busy,
        );
        Callback::from(move |_: MouseEvent| {
            if disabled || open.get() {
                return;
            }
            *draft.borrow_mut() = value.to_string();
            open.set(true);
            editing.set(true);
            on_editing.emit(true);
        })
    };

    let multiline = props.multiline;
    let onkeydown = {
        let end = end.clone();
        Callback::from(move |e: KeyboardEvent| {
            match edit_key(
                &e.key(),
                multiline,
                e.ctrl_key() || e.meta_key(),
                e.is_composing(),
            ) {
                EditKey::Commit => {
                    e.prevent_default();
                    end.emit((true, true));
                }
                EditKey::Cancel => {
                    // Escape closes the edit, not the dialog around it.
                    e.prevent_default();
                    e.stop_propagation();
                    end.emit((false, true));
                }
                EditKey::Type => {}
            }
        })
    };
    let oninput = {
        let draft = draft.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(target) = e.target() {
                let text = match target.dyn_ref::<HtmlInputElement>() {
                    Some(input) => input.value(),
                    None => target
                        .dyn_ref::<HtmlTextAreaElement>()
                        .map(HtmlTextAreaElement::value)
                        .unwrap_or_default(),
                };
                *draft.borrow_mut() = text;
            }
        })
    };
    let onblur = {
        let end = end.clone();
        Callback::from(move |_: FocusEvent| {
            if !multiline {
                end.emit((true, false));
            }
        })
    };

    let class = classes!(props.class.clone());
    if *editing {
        let text = draft.borrow().clone();
        if multiline {
            let rows = (text.lines().count() + 1).clamp(3, 12).to_string();
            let save = {
                let end = end.clone();
                Callback::from(move |_: MouseEvent| end.emit((true, true)))
            };
            let cancel = Callback::from(move |_: MouseEvent| end.emit((false, true)));
            return html! {
                <div class={classes!(class::INLINE_EDIT_EDITOR, class)} data-testid={props.testid.clone()}>
                    <textarea ref={field} class={textarea(InputSize::Regular, false, false)}
                        aria-label={props.label.clone()} value={text} {rows}
                        maxlength={props.maxlength.map(|n| n.to_string())}
                        {onkeydown} {oninput} />
                    <div class={class::INLINE_EDIT_ACTIONS}>
                        <Button size={ButtonSize::Small} onclick={cancel}>
                            { label(&phrases, &props.cancel_label, "sockel-inline-edit-cancel") }
                        </Button>
                        <Button size={ButtonSize::Small} variant={ButtonVariant::Primary} onclick={save}>
                            { label(&phrases, &props.save_label, "sockel-inline-edit-save") }
                        </Button>
                    </div>
                </div>
            };
        }
        return html! {
            <input ref={field} type="text" class={classes!(input(InputSize::Regular, false, false), class::INLINE_EDIT_FIELD, class)}
                aria-label={props.label.clone()} value={text}
                maxlength={props.maxlength.map(|n| n.to_string())}
                data-testid={props.testid.clone()}
                {onkeydown} {oninput} {onblur} />
        };
    }

    let empty = props.value.is_empty();
    let shown = if empty {
        label(&phrases, &props.placeholder, "sockel-inline-edit-none")
    } else {
        props.value.clone()
    };
    let mut args = Args::new();
    args.set("label", props.label.as_str())
        .set("value", shown.as_str());
    let name = phrases.msg("sockel-inline-edit-name", &args);
    let mut args = Args::new();
    args.set("label", props.label.as_str());
    let title = phrases.msg("sockel-inline-edit-edit", &args);
    html! {
        <button ref={face} type="button"
            class={classes!(inline_edit(empty, multiline), props.busy.then_some(class::BUSY), class)}
            aria-label={name} {title} disabled={props.disabled}
            aria-busy={props.busy.then_some("true")}
            data-testid={props.testid.clone()} onclick={start}>
            <span class={class::INLINE_EDIT_TEXT}>{ shown }</span>
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn keys_commit_a_line_and_long_text_by_their_own_chords() {
        assert_eq!(edit_key("Enter", false, false, false), EditKey::Commit);
        assert_eq!(edit_key("Enter", true, false, false), EditKey::Type);
        assert_eq!(edit_key("Enter", true, true, false), EditKey::Commit);
        assert_eq!(edit_key("Escape", true, false, false), EditKey::Cancel);
        assert_eq!(
            edit_key("Enter", false, false, true),
            EditKey::Type,
            "composing"
        );
        assert_eq!(edit_key("a", false, false, false), EditKey::Type);
    }

    #[test]
    fn the_classes_name_only_what_differs() {
        assert_eq!(names(property_list(false)), [class::PROPERTY_LIST]);
        assert_eq!(
            names(property_list(true)),
            [class::PROPERTY_LIST, class::BOXED]
        );
        assert_eq!(
            names(property_row(true)),
            [class::PROPERTY_ROW, class::STACKED]
        );
        let face = names(inline_edit(true, false));
        assert!(face.contains(&class::INPUT.to_owned()));
        assert!(face.contains(&class::INLINE_EDIT.to_owned()));
        assert!(face.contains(&class::EMPTY.to_owned()));
    }

    #[test]
    fn a_row_and_its_face_render_their_parts() {
        #[function_component]
        fn Shown() -> Html {
            html! {
                <PropertyList label="Card" class={class::SMALL}>
                    <PropertyRow label="Title" hint="Shown on the board">
                        <InlineEdit value="" label="Title" on_commit={Callback::noop()} />
                    </PropertyRow>
                </PropertyList>
            }
        }
        let html = futures::executor::block_on(yew::LocalServerRenderer::<Shown>::new().render());
        assert!(
            html.contains(&format!(
                "class=\"{} {}\"",
                class::PROPERTY_LIST,
                class::SMALL
            )),
            "{html}"
        );
        assert!(html.contains(class::PROPERTY_HINT), "{html}");
        assert!(html.contains("aria-label=\"Title: None\""), "{html}");
        assert!(html.contains(">None<"), "{html}");
    }
}
