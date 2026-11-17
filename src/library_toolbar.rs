//! LibraryToolbar (`style/_layout.scss`).

use crate::{SearchField, class};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct LibraryToolbarProps {
    /// The search field's text.
    pub value: AttrValue,
    /// The field's placeholder and accessible name.
    pub label: AttrValue,
    /// The field's test id.
    pub testid: AttrValue,
    pub on_input: Callback<String>,
    /// The field takes the focus when its dialog or form opens
    /// (`data-autofocus`).
    #[prop_or_default]
    pub autofocus: bool,
    /// The search field's text input.
    #[prop_or_default]
    pub input_ref: Option<NodeRef>,
    /// Keys in the search field (see [`SearchField`]).
    #[prop_or_default]
    pub on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop_or_default]
    pub class: Classes,
    /// The filters after the field: filter buttons of class `filter`, one
    /// width each.
    #[prop_or_default]
    pub children: Html,
}

/// A list's [`SearchField`], which grows, and its filters, which keep
/// a width of their own: a picked filter never moves the field. On a
/// phone the field has the row to itself and the filters share the next.
#[function_component]
pub fn LibraryToolbar(props: &LibraryToolbarProps) -> Html {
    html! {
        <div class={classes!(class::LIBRARY_TOOLBAR, props.class.clone())}>
            <SearchField class={class::LIBRARY_SEARCH} value={props.value.clone()}
                label={props.label.clone()} testid={props.testid.clone()}
                on_input={props.on_input.clone()} autofocus={props.autofocus}
                input_ref={props.input_ref.clone()} on_keydown={props.on_keydown.clone()} />
            { props.children.clone() }
        </div>
    }
}
