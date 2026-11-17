//! Field, FormRows and Fieldset (`style/_fields.scss`).

use crate::class;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FieldProps {
    /// The control's `id`, which the label's `for` names; the control in
    /// `children` takes it.
    pub id: AttrValue,
    pub label: Html,
    /// Marks the label with `*`; the control sets `required` itself.
    #[prop_or_default]
    pub required: bool,
    /// Help for the field, at the end of the label's row.
    #[prop_or_default]
    pub help: Option<Html>,
    /// A hint under the control.
    #[prop_or_default]
    pub hint: Option<Html>,
    /// The error under the control and the hint.
    #[prop_or_default]
    pub error: Option<Html>,
    /// Keeps the error line's room while there is no error, so one
    /// appearing moves nothing below; a longer error is cut to the
    /// reserved lines (`--field-error-lines`).
    #[prop_or_default]
    pub reserve_error: bool,
    /// Mutes the field; the control disables itself.
    #[prop_or_default]
    pub disabled: bool,
    /// Waiting on something (`aria-busy`).
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The control.
    #[prop_or_default]
    pub children: Html,
}

/// A field's classes: `field`, `disabled` and `busy`.
pub fn field(disabled: bool, busy: bool) -> Classes {
    classes!(
        class::FIELD,
        disabled.then_some(class::DISABLED),
        busy.then_some(class::BUSY)
    )
}

/// A form row's classes: `form-row`, `disabled` and `busy`.
pub fn form_row(disabled: bool, busy: bool) -> Classes {
    classes!(
        class::FORM_ROW,
        disabled.then_some(class::DISABLED),
        busy.then_some(class::BUSY)
    )
}

/// The classes of a `FormRows` grid: `wide_labels` fixes the label column.
pub fn form_rows(wide_labels: bool) -> Classes {
    classes!(class::FORM_ROWS, wide_labels.then_some(class::WIDE_LABELS))
}

/// The error line's classes, `reserve` for one that keeps its room.
pub fn field_error(reserve: bool) -> Classes {
    classes!(class::FIELD_ERROR, reserve.then_some(class::RESERVE))
}

/// The `aria-describedby` of a field's control: its hint and error, as the
/// field names them. Pass whether the field has each.
pub fn field_describedby(id: &str, hint: bool, error: bool) -> Option<AttrValue> {
    let ids: Vec<String> = [(hint, "hint"), (error, "error")]
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, part)| format!("{id}-{part}"))
        .collect();
    (!ids.is_empty()).then(|| ids.join(" ").into())
}

fn label_parts(props: &FieldProps, label_class: Classes) -> Html {
    html! {
        <>
            <label class={label_class} for={props.id.clone()}>{ props.label.clone() }</label>
            if props.required {
                <span class={class::FIELD_REQUIRED} aria-hidden="true">{ "*" }</span>
            }
            if let Some(help) = &props.help {
                <span class={class::FIELD_HELP}>{ help.clone() }</span>
            }
        </>
    }
}

fn messages(props: &FieldProps) -> Html {
    let hint = props.hint.as_ref().map(|hint| {
        html! { <p id={format!("{}-hint", props.id)} class={class::FIELD_HINT}>{ hint.clone() }</p> }
    });
    let error = (props.error.is_some() || props.reserve_error).then(|| {
        html! {
            <p id={format!("{}-error", props.id)}
                class={field_error(props.reserve_error)}
                aria-live="polite">
                { props.error.clone() }
            </p>
        }
    });
    html! { <>{ for hint }{ for error }</> }
}

/// A label, the control, a hint and an error, stacked; the label's `for`
/// names `id`, and the hint and error carry the ids
/// [`field_describedby`] gives.
#[function_component]
pub fn Field(props: &FieldProps) -> Html {
    html! {
        <div class={classes!(field(props.disabled, props.busy), props.class.clone())}
            aria-busy={props.busy.then_some("true")}
            data-testid={props.testid.clone()}>
            <div class={class::FIELD_HEAD}>
                { label_parts(props, classes!(class::FIELD_LABEL)) }
            </div>
            { props.children.clone() }
            { messages(props) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct FormRowsProps {
    /// Fixes the label column at `--field-label-width` (on a wide
    /// screen), so every section's controls start on one line.
    #[prop_or_default]
    pub wide_labels: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub children: Html,
}

/// A grid of [`FormRow`]s sharing one label column; below the phone
/// breakpoint each label sits above its control.
#[function_component]
pub fn FormRows(props: &FormRowsProps) -> Html {
    html! {
        <div class={classes!(form_rows(props.wide_labels), props.class.clone())}>
            { props.children.clone() }
        </div>
    }
}

/// A [`Field`] as a row of [`FormRows`]: the label in the shared column,
/// the control with its hint and error beside it.
#[function_component]
pub fn FormRow(props: &FieldProps) -> Html {
    html! {
        <div class={classes!(form_row(props.disabled, props.busy), props.class.clone())}
            aria-busy={props.busy.then_some("true")}
            data-testid={props.testid.clone()}>
            <div class={class::FORM_LABEL}>
                { label_parts(props, classes!(class::FIELD_LABEL)) }
            </div>
            <div class={class::FORM_CONTROL}>
                { props.children.clone() }
                { messages(props) }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct FieldsetProps {
    /// The group's title.
    pub legend: Html,
    #[prop_or_default]
    pub hint: Option<Html>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A `<fieldset>` with its legend and an optional hint under it.
#[function_component]
pub fn Fieldset(props: &FieldsetProps) -> Html {
    html! {
        <fieldset class={classes!(class::FIELDSET, props.class.clone())}
            data-testid={props.testid.clone()}>
            <legend class={class::LEGEND}>{ props.legend.clone() }</legend>
            if let Some(hint) = &props.hint {
                <p class={class::FIELD_HINT}>{ hint.clone() }</p>
            }
            { props.children.clone() }
        </fieldset>
    }
}

#[derive(Properties, PartialEq)]
pub struct FieldCheckProps {
    /// The text after the checkbox.
    pub label: Html,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    /// The checkbox.
    #[prop_or_default]
    pub children: Html,
}

/// A checkbox, in `children`, with its label after it, as high as an input.
#[function_component]
pub fn FieldCheck(props: &FieldCheckProps) -> Html {
    let class = classes!(
        class::FIELD_CHECK,
        props.disabled.then_some(class::DISABLED),
        props.class.clone()
    );
    html! { <label {class}>{ props.children.clone() }<span>{ props.label.clone() }</span></label> }
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

    fn props() -> FieldProps {
        FieldProps {
            id: "name".into(),
            label: html! { "Name" },
            required: false,
            help: None,
            hint: None,
            error: None,
            reserve_error: false,
            disabled: false,
            busy: false,
            class: Classes::new(),
            testid: None,
            children: html! { <input id="name" /> },
        }
    }

    #[test]
    fn field_modifiers() {
        assert_eq!(names(field(false, false)), [class::FIELD]);
        assert_eq!(names(field(true, false)), [class::FIELD, class::DISABLED]);
        assert_eq!(names(field(false, true)), [class::FIELD, class::BUSY]);
        assert_eq!(
            names(form_row(true, true)),
            [class::FORM_ROW, class::DISABLED, class::BUSY]
        );
    }

    #[test]
    fn form_rows_and_error_modifiers() {
        assert_eq!(names(form_rows(false)), [class::FORM_ROWS]);
        assert_eq!(
            names(form_rows(true)),
            [class::FORM_ROWS, class::WIDE_LABELS]
        );
        assert_eq!(names(field_error(false)), [class::FIELD_ERROR]);
        assert_eq!(
            names(field_error(true)),
            [class::FIELD_ERROR, class::RESERVE]
        );
    }

    #[test]
    fn the_label_names_the_control() {
        for html in [render::<Field>(props()), render::<FormRow>(props())] {
            assert!(html.contains("<label for=\"name\""), "{html}");
            assert!(html.contains(">Name</label>"), "{html}");
            assert!(html.contains("<input id=\"name\""), "{html}");
        }
    }

    #[test]
    fn hint_and_error_have_the_ids_the_control_describes_itself_by() {
        let html = render::<Field>(FieldProps {
            hint: Some(html! { "A hint" }),
            error: Some(html! { "Wrong" }),
            ..props()
        });
        assert!(html.contains("id=\"name-hint\""), "{html}");
        assert!(html.contains("id=\"name-error\""), "{html}");
        assert_eq!(
            field_describedby("name", true, true).as_deref(),
            Some("name-hint name-error")
        );
        assert_eq!(
            field_describedby("name", false, true).as_deref(),
            Some("name-error")
        );
        assert_eq!(field_describedby("name", false, false), None);
    }

    #[test]
    fn the_error_line_exists_only_with_an_error_or_a_reserve() {
        assert!(!render::<Field>(props()).contains(class::FIELD_ERROR));
        let reserved = render::<Field>(FieldProps {
            reserve_error: true,
            ..props()
        });
        assert!(reserved.contains(class::RESERVE), "{reserved}");
        let shown = render::<Field>(FieldProps {
            error: Some(html! { "Wrong" }),
            ..props()
        });
        assert!(shown.contains(">Wrong</p>"), "{shown}");
        assert!(!shown.contains(class::RESERVE), "{shown}");
    }

    #[test]
    fn required_is_a_hidden_mark_and_busy_is_aria_busy() {
        let html = render::<Field>(FieldProps {
            required: true,
            busy: true,
            ..props()
        });
        assert!(html.contains(">*</span>"), "{html}");
        assert!(html.contains("aria-busy=\"true\""), "{html}");
    }
}
