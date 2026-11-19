//! ErrorBox (`style/_notices.scss`).

use crate::testid::with_testid;
use phrasebook02::yew023::use_phrases_with;
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonSize, ButtonVariant, Glyph, Icon, class};

#[derive(Properties, Default, PartialEq)]
pub struct ErrorBoxProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The technical cause (an error's text), under the message, cut to
    /// three lines.
    #[prop_or_default]
    pub cause: Option<AttrValue>,
    /// Causes, one line each under the message and never cut, most
    /// specific last; for a chain of errors.
    #[prop_or_default]
    pub causes: Vec<AttrValue>,
    /// Shows a retry button, which calls this.
    #[prop_or_default]
    pub onretry: Option<Callback<MouseEvent>>,
    /// The retry button's label, else the catalog's.
    #[prop_or_default]
    pub retry_label: Option<AttrValue>,
    /// The retry is running: its button is busy and ignores clicks.
    #[prop_or_default]
    pub retrying: bool,
    /// Before the text, in the error's colour.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// Shows a dismiss button at the end, which calls this. The box does not
    /// remove itself.
    #[prop_or_default]
    pub ondismiss: Option<Callback<MouseEvent>>,
    /// The dismiss button's accessible name and tooltip, else the catalog's.
    #[prop_or_default]
    pub dismiss_label: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The box's `id`, for `aria-describedby` and the like.
    #[prop_or_default]
    pub id: Option<AttrValue>,
    /// The message.
    #[prop_or_default]
    pub children: Html,
}

/// The classes of an error box: `error-box`, and `flanked` when an icon or
/// a dismiss button stands beside its text.
pub fn error_box(flanked: bool) -> Classes {
    classes!(class::ERROR_BOX, flanked.then_some(class::FLANKED))
}

/// An error with what went wrong, its cause and a way to retry, and
/// optionally an icon before it and a dismiss button after it. It is an
/// alert; a retry in progress keeps the button's box.
#[function_component]
pub fn ErrorBox(props: &ErrorBoxProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let text = html! {
        <>
            if let Some(title) = &props.title {
                <p class={class::ERROR_BOX_TITLE}>{ title.clone() }</p>
            }
            <p class={class::ERROR_BOX_MESSAGE}>{ props.children.clone() }</p>
            if let Some(cause) = &props.cause {
                <p class={class::ERROR_BOX_CAUSE} title={cause.clone()}>{ cause.clone() }</p>
            }
            if !props.causes.is_empty() {
                <ul class={class::ERROR_BOX_CAUSES}>
                    { for props.causes.iter().map(|cause| html! { <li>{ cause.clone() }</li> }) }
                </ul>
            }
            if let Some(onretry) = &props.onretry {
                <div class={class::ERROR_BOX_ACTIONS}>
                    <Button size={ButtonSize::Small} busy={props.retrying}
                        onclick={onretry.clone()}>
                        { label(&phrases, &props.retry_label, "sockel-error-box-retry") }
                    </Button>
                </div>
            }
        </>
    };
    let flanked = props.icon.is_some() || props.ondismiss.is_some();

    with_testid(
        html! {
            <div class={classes!(error_box(flanked), props.class.clone())} role="alert"
                id={props.id.clone()}>
                if flanked {
                    if let Some(icon) = &props.icon {
                        <span class={class::ERROR_BOX_ICON} aria-hidden="true">{ icon.clone() }</span>
                    }
                    <div class={class::ERROR_BOX_BODY}>{ text }</div>
                    if let Some(ondismiss) = &props.ondismiss {
                        <span class={class::ERROR_BOX_CLOSE}>
                            <Button variant={ButtonVariant::Quiet}
                                icon={html! { <Icon glyph={Glyph::Close} /> }}
                                label={label(&phrases, &props.dismiss_label, "sockel-error-box-dismiss")}
                                title={label(&phrases, &props.dismiss_label, "sockel-error-box-dismiss")}
                                onclick={ondismiss.clone()} />
                        </span>
                    }
                } else {
                    { text }
                }
            </div>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(props: ErrorBoxProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<ErrorBox>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props() -> ErrorBoxProps {
        ErrorBoxProps {
            class: Classes::new(),
            title: None,
            cause: None,
            causes: Vec::new(),
            onretry: None,
            retry_label: None,
            retrying: false,
            icon: None,
            ondismiss: None,
            dismiss_label: None,
            testid: None,
            id: None,
            children: html! { "Saving failed" },
        }
    }

    #[test]
    fn modifiers() {
        let names = |c: Classes| c.into_iter().map(|c| c.to_string()).collect::<Vec<_>>();
        assert_eq!(names(error_box(false)), [class::ERROR_BOX]);
        assert_eq!(names(error_box(true)), [class::ERROR_BOX, class::FLANKED]);
    }

    #[test]
    fn plain_has_no_body_or_dismiss() {
        let html = render(props());
        assert!(!html.contains(class::ERROR_BOX_BODY), "{html}");
        assert!(!html.contains(class::FLANKED), "{html}");
    }

    #[test]
    fn a_dismiss_flanks_the_text() {
        let html = render(ErrorBoxProps {
            icon: Some(html! { <i /> }),
            ondismiss: Some(Callback::noop()),
            dismiss_label: Some("Dismiss error".into()),
            ..props()
        });
        assert!(html.contains(class::FLANKED), "{html}");
        assert!(html.contains(class::ERROR_BOX_ICON), "{html}");
        assert!(html.contains(class::ERROR_BOX_BODY), "{html}");
        assert!(html.contains(class::ERROR_BOX_CLOSE), "{html}");
        assert!(html.contains("aria-label=\"Dismiss error\""), "{html}");
        assert!(html.contains("title=\"Dismiss error\""), "{html}");
    }

    #[test]
    fn an_id_is_the_boxs_own() {
        assert!(!render(props()).contains("id="));
        let html = render(ErrorBoxProps {
            id: Some("err".into()),
            ..props()
        });
        assert!(html.contains("id=\"err\""), "{html}");
    }

    #[test]
    fn causes_are_a_line_each() {
        let html = render(props());
        assert!(!html.contains(class::ERROR_BOX_CAUSES), "{html}");
        let html = render(ErrorBoxProps {
            causes: vec!["connection refused".into(), "os error 111".into()],
            ..props()
        });
        assert!(html.contains(class::ERROR_BOX_CAUSES), "{html}");
        assert!(
            html.contains("<li>connection refused</li><li>os error 111</li>"),
            "{html}"
        );
    }
}
