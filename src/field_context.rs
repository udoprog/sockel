//! What a [`Field`](crate::Field) tells the control inside it.

use yew::prelude::*;

/// The ids and state a [`Field`](crate::Field) hands the control inside
/// it, read with [`use_field`]. sockel's controls take them when their own
/// `id`, `describedby` and `invalid` are unset.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FieldContext {
    /// The id the label's `for` names; `None` in a group.
    pub id: Option<AttrValue>,
    /// The ids of the field's hint and error, as [`field_describedby`](crate::field_describedby) gives them.
    pub describedby: Option<AttrValue>,
    /// The field shows an error.
    pub invalid: bool,
}

impl FieldContext {
    /// A control's `id`, `describedby` and `invalid`: its own where set,
    /// else the field's.
    pub(crate) fn control(
        &self,
        id: &Option<AttrValue>,
        describedby: &Option<AttrValue>,
        invalid: bool,
    ) -> (Option<AttrValue>, Option<AttrValue>, bool) {
        (
            id.clone().or_else(|| self.id.clone()),
            describedby.clone().or_else(|| self.describedby.clone()),
            invalid || self.invalid,
        )
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct FieldSlot(pub(crate) Option<FieldContext>);

/// The [`Field`](crate::Field) around this component, for a control of
/// its own:
///
/// ```
/// use sockel::use_field;
/// use yew::prelude::*;
///
/// #[function_component]
/// fn Slider() -> Html {
///     let field = use_field().unwrap_or_default();
///     html! {
///         <input type="range" id={field.id} aria-describedby={field.describedby}
///             aria-invalid={field.invalid.then_some("true")} />
///     }
/// }
/// ```
#[hook]
pub fn use_field() -> Option<FieldContext> {
    use_context::<FieldSlot>().and_then(|slot| slot.0)
}

#[cfg(feature = "quantity")]
#[derive(Properties, PartialEq)]
pub(crate) struct NoFieldProps {
    pub children: Html,
}

#[cfg(feature = "quantity")]
/// Hides the field from a part of a composite control that is not the one
/// its label names.
#[function_component]
pub(crate) fn NoField(props: &NoFieldProps) -> Html {
    html! {
        <ContextProvider<FieldSlot> context={FieldSlot(None)}>
            { props.children.clone() }
        </ContextProvider<FieldSlot>>
    }
}
