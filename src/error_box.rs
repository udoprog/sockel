//! ErrorBox (`style/_notices.scss`).

use phrasebook02::yew023::use_phrases_with;
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonSize, class};

#[derive(Properties, PartialEq)]
pub struct ErrorBoxProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The technical cause (an error's text), under the message, cut to
    /// three lines.
    #[prop_or_default]
    pub cause: Option<AttrValue>,
    /// Shows a retry button, which calls this.
    #[prop_or_default]
    pub onretry: Option<Callback<MouseEvent>>,
    /// The retry button's label, else the catalog's.
    #[prop_or_default]
    pub retry_label: Option<AttrValue>,
    /// The retry is running: its button is busy and ignores clicks.
    #[prop_or_default]
    pub retrying: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The message.
    #[prop_or_default]
    pub children: Html,
}

/// An error with what went wrong, its cause and a way to retry. It is an
/// alert; a retry in progress keeps the button's box.
#[function_component]
pub fn ErrorBox(props: &ErrorBoxProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    html! {
        <div class={classes!(class::ERROR_BOX, props.class.clone())} role="alert"
            data-testid={props.testid.clone()}>
            if let Some(title) = &props.title {
                <p class={class::ERROR_BOX_TITLE}>{ title.clone() }</p>
            }
            <p class={class::ERROR_BOX_MESSAGE}>{ props.children.clone() }</p>
            if let Some(cause) = &props.cause {
                <p class={class::ERROR_BOX_CAUSE} title={cause.clone()}>{ cause.clone() }</p>
            }
            if let Some(onretry) = &props.onretry {
                <div class={class::ERROR_BOX_ACTIONS}>
                    <Button size={ButtonSize::Small} busy={props.retrying}
                        onclick={onretry.clone()}>
                        { label(&phrases, &props.retry_label, "plinth-error-box-retry") }
                    </Button>
                </div>
            }
        </div>
    }
}
