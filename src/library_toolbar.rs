//! LibraryToolbar (`style/_layout.scss`).

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
    /// The search icon before the field.
    #[prop_or_default]
    pub icon: Html,
    /// The filters after the field: filter buttons of class `filter`, one
    /// width each.
    #[prop_or_default]
    pub children: Html,
}

/// A list's search field, which grows, and its filters, which keep
/// a width of their own: a picked filter never moves the field. On a
/// phone the field has the row to itself and the filters share the next.
#[function_component]
pub fn LibraryToolbar(props: &LibraryToolbarProps) -> Html {
    let oninput = props.on_input.reform(|e: InputEvent| {
        e.target_unchecked_into::<web_sys::HtmlInputElement>()
            .value()
    });

    html! {
        <div class="library-toolbar">
            <label class="library-search">
                { props.icon.clone() }
                <input class="input" type="search" value={props.value.clone()} data-testid={props.testid.clone()}
                    placeholder={props.label.clone()} aria-label={props.label.clone()}
                    data-autofocus={props.autofocus.then_some("")} {oninput} />
            </label>
            { props.children.clone() }
        </div>
    }
}
