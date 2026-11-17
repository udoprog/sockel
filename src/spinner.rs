//! Spinner (`style/_skeleton.scss`).

use crate::{Glyph, Icon, class};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SpinnerProps {
    #[prop_or_default]
    pub class: Classes,
    /// What is happening ("Saving"): the spinner is then a status for a
    /// screen reader, and decoration without it.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A spinning glyph in the icon box (`--glyph-size`, `1em`): inline, in the
/// colour of the text around it, and the same box whatever icon set the app
/// provides.
#[function_component]
pub fn Spinner(props: &SpinnerProps) -> Html {
    let labelled = props.label.is_some();
    html! {
        <span class={classes!(class::SPINNER, props.class.clone())}
            role={labelled.then_some("status")}
            aria-label={props.label.clone()}
            aria-hidden={(!labelled).then_some("true")}
            data-testid={props.testid.clone()}>
            <Icon glyph={Glyph::Spinner} />
        </span>
    }
}
